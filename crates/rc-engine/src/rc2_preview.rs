//! Opt-in RC2 terrain diagnostic on the existing Bevy/Metal renderer.
//! Experimental colour/depth/projection policy, NOT original GS/VU parity.
//! No RAC1 Level, schedules, save paths, audio or gameplay plugins are installed.
use anyhow::{bail, Context, Result};
use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology},
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, BlendState, Extent3d, RenderPipelineDescriptor,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat,
        },
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    shader::ShaderRef,
};
use rc_data::rc2_preview::{diagnostic_position, Preview};
use std::path::{Path, PathBuf};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PreviewMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    order: u32,
}
impl Material for PreviewMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/rc2_preview.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "shaders/rc2_preview.wgsl".into()
    }
    // Ordered phase, with explicit replace/depth-write below. This diagnostic
    // ignores alpha rather than inventing an inherited GC TEST/ALPHA state.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn depth_bias(&self) -> f32 {
        self.order as f32 * 1.0e6
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(
        _: &MaterialPipeline,
        d: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> std::result::Result<(), SpecializedMeshPipelineError> {
        d.primitive.cull_mode = None;
        if let Some(depth) = &mut d.depth_stencil {
            depth.depth_write_enabled = true;
        }
        if let Some(f) = &mut d.fragment {
            for target in f.targets.iter_mut().flatten() {
                target.blend = Some(BlendState::REPLACE);
            }
        }
        Ok(())
    }
}

#[derive(Resource)]
struct Scene(Preview);
#[derive(Resource)]
struct Options {
    camera: Option<([f32; 3], [f32; 3])>,
    capture: Option<PathBuf>,
}
#[derive(Component)]
struct DiagnosticCamera;

fn camera_option(value: &str) -> Result<([f32; 3], [f32; 3])> {
    let v = value
        .split(',')
        .map(|v| v.trim().parse::<f32>())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if v.len() != 6 || v.iter().any(|v| !v.is_finite()) {
        bail!("RC2_PREVIEW_CAM needs six finite game-space floats");
    }
    let eye = diagnostic_position([v[0], v[1], v[2]]);
    let target = diagnostic_position([v[3], v[4], v[5]]);
    let dir = Vec3::from_array(target) - Vec3::from_array(eye);
    if dir.length_squared() < 1e-6 || dir.normalize().cross(Vec3::Y).length_squared() < 1e-6 {
        bail!("RC2_PREVIEW_CAM is degenerate or parallel to up");
    }
    Ok((eye, target))
}

pub fn run(root: &Path) -> Result<()> {
    let lod = std::env::var("RC2_PREVIEW_LOD")
        .ok()
        .map(|s| s.parse::<u32>())
        .transpose()?
        .unwrap_or(0);
    let scene = rc_data::rc2_preview::load_aranos(root, lod)
        .context("verified Build7 Aranos preview load")?;
    let camera = std::env::var("RC2_PREVIEW_CAM")
        .ok()
        .map(|v| camera_option(&v))
        .transpose()?;
    let capture_path = std::env::var_os("RC2_PREVIEW_CAPTURE").map(PathBuf::from);
    if capture_path.as_ref().is_some_and(|p| p.exists()) {
        bail!("RC2 preview capture must be a NEW file");
    }
    println!("RC2 SCENE PREVIEW — EXPERIMENTAL, NOT GAMEPLAY: LOD{lod}, {} triangles, {} ordered draws, {} verified mip0 images; bounds {:?}..{:?}",scene.triangles,scene.draws.len(),scene.images.len(),scene.min,scene.max);
    println!("Policy: diagnostic Z-up rotation, 60deg perspective, opaque depth-write, two-sided, base-RGB/128 texture modulation, mip0 bilinear, no fog/lights/morph/alpha. Original GC parity UNPROVEN.");
    let mut app = App::new();
    app.add_plugins(
        crate::runtime_plugins()
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ReRAC — RC2 scene preview (experimental; no gameplay)".into(),
                    resolution: (1024u32, 768u32).into(),
                    present_mode: bevy::window::PresentMode::Fifo,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: crate::asset_dir(),
                ..default()
            }),
    )
    .add_plugins((
        MaterialPlugin::<PreviewMaterial>::default(),
        crate::no_clusters::NoClustersPlugin,
    ))
    .insert_resource(Scene(scene))
    .insert_resource(Options {
        camera,
        capture: capture_path,
    })
    .insert_resource(ClearColor(Color::srgb(0.025, 0.025, 0.04)))
    .add_systems(Startup, spawn)
    .add_systems(Update, (diagnostic_camera, capture));
    #[cfg(target_os = "ios")]
    app.add_plugins(crate::ios_gamepad::IOSGameControllerPlugin);
    app.run();
    Ok(())
}

fn spawn(
    mut commands: Commands,
    scene: Res<Scene>,
    options: Res<Options>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<PreviewMaterial>>,
) {
    // Separate images per sampler key; no mutation of a sampler shared by
    // different packet wrap modes. No derived mipmaps or generated textures.
    let mut textures = std::collections::HashMap::new();
    for (order, draw) in scene.0.draws.iter().enumerate() {
        let wrap = draw.packet.wrap();
        let texture = textures
            .entry((draw.material, wrap))
            .or_insert_with(|| {
                let source = &scene.0.images[draw.material as usize];
                let mut image = Image::new(
                    Extent3d {
                        width: source.width,
                        height: source.height,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    source.rgba.clone(),
                    TextureFormat::Rgba8Unorm,
                    RenderAssetUsages::default(),
                );
                let mode = |v| {
                    if v == 0 {
                        ImageAddressMode::Repeat
                    } else {
                        ImageAddressMode::ClampToEdge
                    }
                };
                image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: mode(wrap.0),
                    address_mode_v: mode(wrap.1),
                    min_filter: ImageFilterMode::Linear,
                    mag_filter: ImageFilterMode::Linear,
                    ..default()
                });
                images.add(image)
            })
            .clone();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, draw.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, draw.st.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, draw.rgba.clone());
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(PreviewMaterial {
                texture,
                order: order as u32,
            })),
            Transform::default(),
        ));
    }
    let min = Vec3::from_array(scene.0.min);
    let max = Vec3::from_array(scene.0.max);
    let center = (min + max) * 0.5;
    let radius = (max - min).length() * 0.5;
    let (eye, target) = options
        .camera
        .map(|(a, b)| (Vec3::from_array(a), Vec3::from_array(b)))
        .unwrap_or((
            center + Vec3::new(0., 0.65, 1.).normalize() * radius.max(1.) * 2.3,
            center,
        ));
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 60f32.to_radians(),
            near: 0.05,
            far: (radius * 10.).max(1000.),
            ..default()
        }),
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        Transform::from_translation(eye).looking_at(target, Vec3::Y),
        DiagnosticCamera,
    ));
}

// Camera inspection only: no player entity, collision or original input claim.
fn diagnostic_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    mut cameras: Query<&mut Transform, With<DiagnosticCamera>>,
) {
    for mut t in &mut cameras {
        let axis = |positive, negative| {
            (keys.pressed(positive) as u8 as f32) - (keys.pressed(negative) as u8 as f32)
        };
        let mut movement = Vec3::new(
            axis(KeyCode::KeyD, KeyCode::KeyA),
            axis(KeyCode::KeyE, KeyCode::KeyQ),
            axis(KeyCode::KeyS, KeyCode::KeyW),
        );
        let mut look = Vec2::new(
            axis(KeyCode::ArrowLeft, KeyCode::ArrowRight),
            axis(KeyCode::ArrowUp, KeyCode::ArrowDown),
        );
        if let Some(pad) = pads.iter().next() {
            let left = pad.left_stick();
            movement += Vec3::new(left.x, 0., -left.y);
            look -= pad.right_stick();
        }
        let dt = time.delta_secs().min(0.05);
        let speed = if keys.pressed(KeyCode::ShiftLeft) {
            100.
        } else {
            20.
        };
        let local = movement.clamp_length_max(1.) * speed * dt;
        let rotation = t.rotation;
        t.translation += rotation * local;
        let (yaw, pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
        t.rotation = Quat::from_euler(
            EulerRot::YXZ,
            yaw + look.x * dt,
            (pitch + look.y * dt).clamp(-1.5, 1.5),
            0.,
        );
    }
}

fn capture(mut commands: Commands, options: Res<Options>, mut frames: Local<u32>) {
    *frames += 1;
    // A submitted screenshot proves capture only after its observer succeeds.
    if *frames != 120 {
        return;
    }
    let Some(path) = options.capture.clone() else {
        return;
    };
    commands.spawn(Screenshot::primary_window()).observe(
        move |shot: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            let result = (|| -> Result<()> {
                // create_new preserves prior evidence even if it appeared after boot.
                let image = shot
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                drop(file);
                image.save(&path)?;
                Ok(())
            })();
            match result {
                Ok(()) => {
                    println!(
                        "RC2_PREVIEW_CAPTURE_SAVED {} (experimental scene, parity unproven)",
                        path.display()
                    );
                    exit.write(AppExit::Success);
                }
                Err(e) => {
                    eprintln!("RC2_PREVIEW_CAPTURE_FAILED: {e}");
                    exit.write(AppExit::error());
                }
            }
        },
    );
}
