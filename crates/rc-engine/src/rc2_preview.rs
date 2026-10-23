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
            depth.depth_compare = bevy::render::render_resource::CompareFunction::GreaterEqual;
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
    headless: bool,
    fixture: bool,
}
#[derive(Resource)]
struct CaptureTarget(Handle<Image>);
#[derive(Component)]
struct DiagnosticCamera;

/// Explicit Files transfer opt-in, separate from RAC1 import and saved games.
/// Removing the request file restores normal RAC1 startup without migrations.
#[cfg(target_os = "ios")]
pub fn requested_ios_root() -> Result<Option<PathBuf>> {
    use std::io::Read;
    let home = std::env::var_os("HOME").context("iOS HOME unavailable")?;
    let root = PathBuf::from(home).join("Documents/RC2ScenePreview");
    let mut file = match std::fs::File::open(root.join("ENABLE_SCENE_PREVIEW.txt")) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut request = Vec::new();
    (&mut file).take(64).read_to_end(&mut request)?;
    if request != b"ARANOS_TERRAIN_PREVIEW_V1\n" {
        bail!("invalid RC2 scene-preview request; remove ENABLE_SCENE_PREVIEW.txt to resume RAC1");
    }
    crate::ios::diagnostic(
        "RC2_SCENE_PREVIEW_REQUESTED",
        "explicit sidecar; not gameplay; RAC1 boot skipped",
    );
    Ok(Some(root))
}

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
    let fixture = std::env::var("RC2_PREVIEW_GPU_FIXTURE").is_ok_and(|v| v == "1");
    let headless = std::env::var("RC2_PREVIEW_HEADLESS").is_ok_and(|v| v == "1");
    let scene = if fixture {
        #[cfg(any(not(target_os = "ios"), target_abi = "sim"))]
        {
            gpu_fixture()
        }
        #[cfg(all(target_os = "ios", not(target_abi = "sim")))]
        {
            bail!("synthetic GPU fixture is unavailable in physical iPhone builds");
        }
    } else {
        rc_data::rc2_preview::load_aranos(root, lod)
            .context("verified Build7 Aranos preview load")?
    };
    let mut camera = std::env::var("RC2_PREVIEW_CAM")
        .ok()
        .map(|v| camera_option(&v))
        .transpose()?;
    let capture_path = std::env::var_os("RC2_PREVIEW_CAPTURE").map(PathBuf::from);
    if headless && capture_path.is_none() {
        bail!("headless RC2 preview requires RC2_PREVIEW_CAPTURE");
    }
    if fixture {
        println!("SYNTHETIC RC2 PREVIEW GPU PROBE — NO GAME ASSETS, NO ARANOS OR GAMEPLAY CLAIM");
        camera = Some(([0., 0., 5.], [0., 0., 0.]));
    }
    if capture_path.as_ref().is_some_and(|p| p.exists()) {
        bail!("RC2 preview capture must be a NEW file");
    }
    println!("RC2 SCENE PREVIEW — EXPERIMENTAL, NOT GAMEPLAY: synthetic={fixture}, LOD{lod}, {} triangles, {} ordered draws, {} images; bounds {:?}..{:?}",scene.triangles,scene.draws.len(),scene.images.len(),scene.min,scene.max);
    println!("Policy: diagnostic Z-up rotation, 60deg perspective, opaque depth-write, two-sided, base-RGB/128 texture modulation, mip0 bilinear, no fog/lights/morph/alpha. Original GC parity UNPROVEN.");
    let mut app = App::new();
    let mut plugins = crate::runtime_plugins()
        .set(WindowPlugin {
            primary_window: (!headless).then(|| Window {
                title: "ReRAC — RC2 scene preview (experimental; no gameplay)".into(),
                resolution: (1024u32, 768u32).into(),
                present_mode: bevy::window::PresentMode::Fifo,
                #[cfg(target_os = "ios")]
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                #[cfg(target_os = "ios")]
                resizable: false,
                #[cfg(target_os = "ios")]
                prefers_status_bar_hidden: true,
                #[cfg(target_os = "ios")]
                prefers_home_indicator_hidden: true,
                ..default()
            }),
            exit_condition: if headless {
                bevy::window::ExitCondition::DontExit
            } else {
                bevy::window::ExitCondition::OnAllClosed
            },
            ..default()
        })
        .set(AssetPlugin {
            file_path: crate::asset_dir(),
            ..default()
        });
    if headless {
        plugins = plugins.disable::<bevy::winit::WinitPlugin>();
    }
    app.add_plugins(plugins)
        .add_plugins((
            MaterialPlugin::<PreviewMaterial>::default(),
            crate::no_clusters::NoClustersPlugin,
        ))
        .insert_resource(Scene(scene))
        .insert_resource(Options {
            camera,
            capture: capture_path,
            headless,
            fixture,
        })
        .insert_resource(ClearColor(Color::srgb(0.025, 0.025, 0.04)))
        .add_systems(Startup, spawn)
        .add_systems(Update, (diagnostic_camera, capture));
    if headless {
        app.add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_secs_f64(1. / 60.),
        ));
    }
    #[cfg(target_os = "ios")]
    app.add_plugins(crate::ios_gamepad::IOSGameControllerPlugin);
    match app.run() {
        AppExit::Success => Ok(()),
        _ => bail!("RC2 preview exited with failure; inspect render/capture diagnostics"),
    }
}

/// Independent synthetic render contract, never an authentic RC2 scene.
/// Left: near green BEFORE farther red tests depth. Right: coplanar blue BEFORE
/// yellow tests submission order independently (reverse order cannot pass both).
#[cfg(any(not(target_os = "ios"), target_abi = "sim"))]
fn gpu_fixture() -> Preview {
    use rc_data::{
        rc2_preview::{Draw, Image},
        rc2_scene::MaterialPacket,
    };
    let draws = [(-1.2, 0.), (-1.2, -1.), (1.2, 0.), (1.2, 0.)]
        .into_iter()
        .enumerate()
        .map(|(material, (x, z))| Draw {
            fragment: material as u32,
            material: material as u32,
            packet: MaterialPacket([0; 80]),
            positions: vec![[x - 0.8, -0.8, z], [x + 0.8, -0.8, z], [x, 0.8, z]],
            st: vec![[0.5, 0.5]; 3],
            rgba: vec![[128., 128., 128., 128.]; 3],
        })
        .collect();
    Preview {
        draws,
        images: vec![
            Image {
                width: 1,
                height: 1,
                rgba: vec![0, 255, 0, 128],
            },
            Image {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 128],
            },
            Image {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 255, 128],
            },
            Image {
                width: 1,
                height: 1,
                rgba: vec![255, 255, 0, 128],
            },
        ],
        triangles: 4,
        min: [-2., -0.8, -1.],
        max: [2., 0.8, 0.],
        lod: 0,
    }
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
    let camera = commands
        .spawn((
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
        ))
        .id();
    if options.headless {
        let mut image = Image::new_target_texture(1024, 768, TextureFormat::Rgba8UnormSrgb, None);
        image.texture_descriptor.usage |= bevy::render::render_resource::TextureUsages::COPY_SRC;
        let image = images.add(image);
        commands
            .entity(camera)
            .insert(bevy::camera::RenderTarget::Image(image.clone().into()));
        commands.insert_resource(CaptureTarget(image));
    } else {
        commands.spawn((Text::new("RC2 TERRAIN PREVIEW — EXPERIMENTAL\nNo gameplay · original GS/VU parity unverified\nSticks: diagnostic camera"),
            TextFont {font_size:18.,..default()},TextColor(Color::WHITE),
            Node {position_type:PositionType::Absolute,top:px(12),left:px(12),..default()},
            BackgroundColor(Color::srgba(0.,0.,0.,0.8))));
    }
}

// Camera inspection only: no player entity, collision or original input claim.
fn diagnostic_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    options: Res<Options>,
    mut cameras: Query<&mut Transform, With<DiagnosticCamera>>,
) {
    if options.headless || options.fixture {
        return;
    }
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
        if look == Vec2::ZERO {
            continue;
        }
        let (yaw, pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
        t.rotation = Quat::from_euler(
            EulerRot::YXZ,
            yaw + look.x * dt,
            (pitch + look.y * dt).clamp(-1.5, 1.5),
            0.,
        );
    }
}

fn capture(
    mut commands: Commands,
    options: Res<Options>,
    target: Option<Res<CaptureTarget>>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;
    if *frames == 1200 && options.capture.is_some() {
        eprintln!("RC2_PREVIEW_CAPTURE_TIMEOUT");
        exit.write(AppExit::error());
        return;
    }
    // A submitted screenshot proves capture only after its observer succeeds.
    if *frames != 120 {
        return;
    }
    let Some(path) = options.capture.clone() else {
        return;
    };
    let fixture = options.fixture;
    let screenshot = target
        .map(|t| Screenshot::image(t.0.clone()))
        .unwrap_or_else(Screenshot::primary_window);
    commands.spawn(screenshot).observe(
        move |shot: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            let result = (|| -> Result<()> {
                // create_new preserves prior evidence even if it appeared after boot.
                let image = shot
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| anyhow::anyhow!("{e:?}"))?;
                if fixture {
                    let rgb=image.to_rgb8(); let depth=rgb.get_pixel(352,416).0; let order=rgb.get_pixel(672,416).0;
                    if depth[0]>10 || depth[1]<240 || depth[2]>10 || order[0]<240 || order[1]<240 || order[2]>10 { bail!("RC2_PREVIEW_GPU_PROBE_PIXEL_MISMATCH depth={depth:?} order={order:?}"); }
                    println!("RC2_PREVIEW_GPU_PROBE_PASS depth={depth:?} order={order:?}; synthetic texture/colour/depth/order only, not Aranos parity");
                }
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
