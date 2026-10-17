//! iOS-only host adaptation. Gameplay, decoding, rendering and audio stay upstream.
//! Documents are transferred while the app is not running; no ISO is opened here.
use bevy::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_LOCK: Mutex<()> = Mutex::new(());

/// Local diagnostics only. No game bytes, environment dumps or account details.
pub fn diagnostic(event: &'static str, detail: &str) {
    let detail: String = detail.chars().take(1500).map(|c| if c == '\n' || c == '\r' { ' ' } else { c }).collect();
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let line = format!("{time} {event} {detail}\n");
    eprint!("{line}");
    if let Some(path) = LOG_PATH.get() {
        let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        if std::fs::metadata(path).is_ok_and(|m| m.len() > 256 * 1024) {
            let _ = std::fs::rename(path, path.with_extension("previous.log"));
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
}

pub fn begin_launch() {
    if let Some(home) = std::env::var_os("HOME") {
        let support = PathBuf::from(home).join("Documents/ReRACSupport");
        if std::fs::create_dir_all(&support).is_ok() {
            let _ = LOG_PATH.set(support.join("startup.log"));
        }
    }
    diagnostic("IOS_ENTRY", &format!("version={} source={}",
        env!("CARGO_PKG_VERSION"), option_env!("RERAC_SOURCE_COMMIT").unwrap_or("UNKNOWN")));
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        diagnostic("RUST_PANIC", &info.to_string());
        previous(info);
    }));
}

pub fn startup_diagnostics(device: Res<bevy::render::renderer::RenderDevice>) {
    diagnostic("BEVY_STARTUP_SYSTEM_REACHED", "");
    diagnostic("RENDER_DEVICE_READY", &format!("features={:?} max_texture_2d={}",
        device.features(), device.limits().max_texture_dimension_2d));
}

pub fn update_diagnostics(
    mut first: Local<bool>,
    mut lifecycle: MessageReader<bevy::window::AppLifecycle>,
) {
    if !*first {
        *first = true;
        diagnostic("FIRST_UPDATE_REACHED", "");
    }
    for state in lifecycle.read() {
        diagnostic("APP_LIFECYCLE", &format!("{state:?}"));
    }
}

pub fn prepare() -> anyhow::Result<PathBuf> {
    // UIKit supplies the application sandbox HOME, not the build machine's home.
    let home = PathBuf::from(std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("iOS application sandbox HOME unavailable"))?);
    anyhow::ensure!(home.is_absolute(), "iOS sandbox HOME must be absolute");
    let documents = home.join("Documents");
    let data = documents.join("ReRACData");
    let support = documents.join("ReRACSupport");
    std::fs::create_dir_all(&data)?;
    std::fs::create_dir_all(&support)?;
    diagnostic("SANDBOX_STORAGE_READY", "Documents/ReRACData Documents/ReRACSupport");
    diagnostic("STARTUP_RESOURCE_CHECK", &format!(
        "toc={} extract_info={} levels={} global={} bundled_shaders={}",
        data.join("toc.bin").is_file(), data.join("extract-info.json").is_file(),
        data.join("levels").is_dir(), data.join("global").is_dir(),
        std::path::Path::new(&crate::asset_dir()).join("shaders").is_dir()));
    // Before App/task-pool initialization. All existing readers see one data root.
    std::env::set_var("RC_DATA_DIR", &data);
    std::env::set_var("RC_SAVE_DIR", support.join("memcard"));
    std::env::set_var("RC_SETTINGS_FILE", support.join("settings.json"));
    let root = crate::disc_source::mobile_startup(data)?;
    diagnostic("GAME_DATA_VALIDATED", "data_format=1");
    Ok(root)
}

pub fn show_import_instructions(error: &str) {
    diagnostic("BEVY_IMPORT_APP_BUILD", "");
    let text = format!(
        "ReRAC — game data required\n\nCopy your extracted game folder's CONTENTS into:\n\
         Files → On My iPhone → ReRAC → ReRACData\n\n\
         ReRACData must contain toc.bin, extract-info.json, levels/ and global/.\n\
         Close ReRAC before transferring. Finish copying, then relaunch.\n\
         No ISO or copyrighted game data is included in this app.\n\n{error}"
    );
    let mut app = App::new();
    app
        .add_plugins(crate::runtime_plugins().set(AssetPlugin { file_path: crate::asset_dir(), ..default() })
            .set(WindowPlugin { primary_window: Some(Window {
                title: "ReRAC".into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                resizable: false, ..default()
            }), ..default() }))
        .add_plugins(crate::ios_gamepad::IOSGameControllerPlugin)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
        .add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn((
                Text::new(text.clone()),
                TextFont { font_size: FontSize::Px(18.0), ..default() },
                Node { margin: UiRect::all(px(28)), max_width: percent(90), ..default() },
            ));
        })
        .add_systems(Startup, startup_diagnostics)
        .add_systems(Update, update_diagnostics);
    diagnostic("BEVY_IMPORT_RUN_REQUESTED", "");
    app.run();
}
