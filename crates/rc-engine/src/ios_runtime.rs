//! iPhone-only display/power policy and bounded local observations. No game-state edits.
use bevy::audio::AudioSinkPlayback;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[repr(C)]
#[derive(Default)]
struct DeviceInfo {
    safe: [f32; 4],
    active: i32,
    thermal: i32,
    footprint: u64,
    battery: f32,
}
extern "C" {
    fn rr_ios_info(out: *mut DeviceInfo, expensive: i32) -> i32;
}

#[derive(Resource)]
pub struct DeviceState {
    pub ui_scale: f32,
    pub render_height: u32,
    pub active: bool,
    thermal: i32,
}
impl Default for DeviceState {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            render_height: 720,
            active: true,
            thermal: -1,
        }
    }
}

struct MainThread {
    _marker: Rc<()>,
    update_start: Instant,
    last_frame: Option<Instant>,
    last_report: Instant,
    started: Instant,
    governor: crate::ios_display_math::ResolutionGovernor,
    interval_ms: Vec<f64>,
    update_ms: Vec<f64>,
    paused_audio: HashSet<Entity>,
    paused_clock: bool,
    inactive_updates: u32,
}
impl Default for MainThread {
    fn default() -> Self {
        Self {
            _marker: Rc::new(()),
            update_start: Instant::now(),
            last_frame: None,
            last_report: Instant::now(),
            started: Instant::now(),
            governor: Default::default(),
            interval_ms: Vec::with_capacity(2048),
            update_ms: Vec::with_capacity(2048),
            paused_audio: HashSet::new(),
            paused_clock: false,
            inactive_updates: 0,
        }
    }
}

pub struct IOSRuntimePlugin;
impl Plugin for IOSRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeviceState>()
            .insert_non_send_resource(MainThread::default())
            // Stop the default 60 Hz unfocused wakeups. UIKit/winit suspend the renderer
            // in the background; no UIBackgroundModes/audio background entitlement is added.
            .insert_resource(bevy::winit::WinitSettings {
                focused_mode: bevy::winit::UpdateMode::Continuous,
                unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs(1)),
            })
            .add_systems(
                First,
                begin
                    .before(crate::display::sync_frame)
                    .before(crate::frame_pace::steps)
                    .before(bevy::time::TimeSystems),
            )
            .add_systems(Last, finish.before(crate::frame_pace::wait));
    }
}

fn begin(
    mut host: NonSendMut<MainThread>,
    mut state: ResMut<DeviceState>,
    mut time: ResMut<Time<Virtual>>,
    sinks: Query<(Entity, &bevy::audio::AudioSink)>,
) {
    let now = Instant::now();
    host.update_start = now;
    let mut info = DeviceInfo::default();
    let status = unsafe { rr_ios_info(&mut info, 0) };
    if status <= 0 {
        return;
    }
    if status == 1 {
        let scale = crate::ios_display_math::safe_scale(info.safe);
        if state.ui_scale != scale {
            state.ui_scale = scale;
        }
    }
    let active = info.active != 0;
    if state.active != active {
        state.active = active;
        host.last_frame = None;
        host.interval_ms.clear();
        host.update_ms.clear();
        crate::ios::diagnostic(
            "IOS_POWER_LIFECYCLE",
            if active { "FOREGROUND" } else { "INACTIVE" },
        );
    }
    if !active {
        host.inactive_updates = host.inactive_updates.saturating_add(1);
        if !time.is_paused() {
            time.pause();
            host.paused_clock = true;
        }
        for (entity, sink) in &sinks {
            if !sink.is_paused() {
                sink.pause();
                host.paused_audio.insert(entity);
            }
        }
    } else {
        if host.paused_clock {
            time.unpause();
            host.paused_clock = false;
        }
        for (entity, sink) in &sinks {
            if host.paused_audio.remove(&entity) {
                sink.play();
            }
        }
        host.paused_audio.clear();
        if let Some(last) = host.last_frame.replace(now) {
            if host.interval_ms.len() < 2048 {
                host.interval_ms.push((now - last).as_secs_f64() * 1000.0);
            }
        }
    }
    if state.thermal != info.thermal {
        state.thermal = info.thermal;
        crate::ios::diagnostic(
            "IOS_THERMAL_STATE",
            &format!(
                "state={} (0_nominal 1_fair 2_serious 3_critical)",
                info.thermal
            ),
        );
    }
    let elapsed = (now - host.started).as_millis() as u64;
    let target = host.governor.update(elapsed, info.thermal);
    if target != state.render_height {
        state.render_height = target;
        crate::ios::diagnostic(
            "IOS_ADAPTIVE_RESOLUTION",
            &format!("height={target} simulation_hz=60 display_target_hz=60"),
        );
    }
}

fn finish(
    mut host: NonSendMut<MainThread>,
    state: Res<DeviceState>,
    frame: Option<Res<crate::display::GameFrame>>,
    settings: Option<Res<crate::display::DisplaySettings>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
) {
    let now = Instant::now();
    let update = (now - host.update_start).as_secs_f64() * 1000.0;
    if state.active && host.update_ms.len() < 2048 {
        host.update_ms.push(update);
    }
    if now - host.last_report < Duration::from_secs(30) {
        return;
    }
    let mut info = DeviceInfo::default();
    let valid = unsafe { rr_ios_info(&mut info, 1) } > 0;
    let stats = |v: &mut Vec<f64>| {
        if v.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        v.sort_by(f64::total_cmp);
        (
            v.iter().sum::<f64>() / v.len() as f64,
            v[v.len() / 2],
            v[(v.len() * 95 / 100).min(v.len() - 1)],
        )
    };
    let (mean, p50, p95) = stats(&mut host.interval_ms);
    let (update_mean, _, update_p95) = stats(&mut host.update_ms);
    let size = frame.map(|f| f.size).unwrap_or(UVec2::ZERO);
    let drawable = window.map(|w| w.physical_size()).unwrap_or(UVec2::ZERO);
    crate::ios::diagnostic("IOS_PERFORMANCE_SAMPLE", &format!(
        "app_fps={:.2} frame_ms_mean={mean:.2} p50={p50:.2} p95={p95:.2} main_schedule_wall_ms={update_mean:.2} main_schedule_p95={update_p95:.2} gpu_ms=UNMEASURED display={}x{} render={}x{} settings={settings:?} thermal={} footprint_bytes={} battery_fraction={} inactive_updates={} ui_scale={:.3} presented_fps=UNMEASURED",
        if mean>0.0 {1000.0/mean} else {0.0},drawable.x,drawable.y,size.x,size.y,
        state.thermal,if valid {info.footprint} else {0},if valid {info.battery} else {-1.0},host.inactive_updates,state.ui_scale));
    host.last_report = now;
    host.interval_ms.clear();
    host.update_ms.clear();
    host.inactive_updates = 0;
}

/// CI-only renderer fixture inside the actual import application, never the
/// device gameplay path. Exercises shared depth/colour sizes through resizes.
#[cfg(target_abi = "sim")]
pub(crate) fn install_simulator_probe(app: &mut App) {
    if std::env::var("RERAC_IOS_CONTROLLER_SELFTEST").as_deref() != Ok("1") {
        return;
    }
    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            for (order, layer) in [(10, 50), (11, 51)] {
                commands.spawn((
                    Camera3d::default(),
                    Camera {
                        order,
                        clear_color: if order == 10 {
                            bevy::camera::ClearColorConfig::Custom(Color::srgb(0.04, 0.04, 0.06))
                        } else {
                            bevy::camera::ClearColorConfig::None
                        },
                        ..default()
                    },
                    crate::game_camera::game_projection(),
                    bevy::render::view::Msaa::Off,
                    bevy::camera::visibility::RenderLayers::layer(layer),
                    Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
                    DisplayProbe,
                ));
            }
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.4))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.1, 0.2, 0.3),
                    unlit: true,
                    ..default()
                })),
                bevy::camera::visibility::RenderLayers::layer(51),
                Transform::from_xyz(0.0, -0.8, 0.0),
            ));
        },
    )
    .add_systems(PostUpdate, probe.before(bevy::camera::CameraUpdateSystems));
}

#[cfg(target_abi = "sim")]
#[derive(Component)]
struct DisplayProbe;

#[cfg(target_abi = "sim")]
fn probe(
    mut count: Local<u32>,
    mut settings: ResMut<crate::display::DisplaySettings>,
    frame: Res<crate::display::GameFrame>,
    mut cameras: Query<(&Camera, &mut Projection), With<DisplayProbe>>,
) {
    use crate::display::{Aspect, Resolution};
    *count += 1;
    for (_, mut p) in &mut cameras {
        crate::display::set_view_tans(&mut p, crate::game_camera::TAN_HALF_FOV_X, settings.aspect);
    }
    match *count {
        30 => {
            settings.aspect = Aspect::Original;
            settings.resolution = Resolution::Height(416);
        }
        60 => {
            settings.aspect = Aspect::Wide16x9;
            settings.resolution = Resolution::Height(720);
        }
        90 => {
            settings.aspect = Aspect::Native {
                width: 16,
                height: 9,
            };
            settings.resolution = Resolution::Height(576);
        }
        120 => {
            settings.resolution = Resolution::Height(720);
        }
        180 => {
            for (camera, _) in &mut cameras {
                assert_eq!(camera.physical_target_size(), Some(frame.size));
                assert!(frame.size.x > 0 && frame.size.y > 0);
            }
            crate::ios::diagnostic("IOS_RENDER_RESIZE_PROBE_PASS","actual_simulator_graph shared_target native_4_3_16_9_576_720 physical_gameplay=NOT_TESTED");
        }
        _ => {}
    }
}
