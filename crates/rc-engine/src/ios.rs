//! iOS-only host adaptation. Gameplay, decoding, rendering and audio stay upstream.
//! Documents are transferred while the app is not running; no ISO is opened here.
use bevy::prelude::*;
use std::path::PathBuf;

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
    // Before App/task-pool initialization. All existing readers see one data root.
    std::env::set_var("RC_DATA_DIR", &data);
    std::env::set_var("RC_SAVE_DIR", support.join("memcard"));
    std::env::set_var("RC_SETTINGS_FILE", support.join("settings.json"));
    crate::disc_source::mobile_startup(data)
}

pub fn show_import_instructions(error: &str) {
    let text = format!(
        "ReRAC — game data required\n\nCopy your extracted game folder's CONTENTS into:\n\
         Files → On My iPhone → ReRAC → ReRACData\n\n\
         ReRACData must contain toc.bin, extract-info.json, levels/ and global/.\n\
         Close ReRAC before transferring. Finish copying, then relaunch.\n\
         No ISO or copyrighted game data is included in this app.\n\n{error}"
    );
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin { file_path: crate::asset_dir(), ..default() })
            .set(WindowPlugin { primary_window: Some(Window {
                title: "ReRAC".into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                resizable: false, ..default()
            }), ..default() }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
        .add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn((
                Text::new(text.clone()),
                TextFont { font_size: FontSize::Px(18.0), ..default() },
                Node { margin: UiRect::all(px(28)), max_width: percent(90), ..default() },
            ));
        })
        .run();
}
