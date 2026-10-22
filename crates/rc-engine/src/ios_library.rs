//! Small iOS library surface. No game replacement, data copying or save migration.
//! RAC1 retains automatic launch. RAC2 is visible but never dispatches RAC1 code.
use bevy::prelude::*;
use rc_formats::game_id::{Game, native_module_available};

pub struct GameLibraryPlugin;
impl Plugin for GameLibraryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup).add_systems(Update, buttons);
    }
}

#[derive(Component)]
struct LibraryPanel;
#[derive(Component)]
enum Action { Open, Close }

fn setup(mut commands: Commands, camera: Single<Entity, With<crate::display::PresentCamera>>,
         game: Option<Res<crate::Level>>) {
    // Render at display resolution with conservative landscape safe margins,
    // independent of the internal world target. Root survives planet changes.
    let target = UiTargetCamera(*camera);
    commands.spawn((Button, Action::Open, target.clone(),
        crate::level_switch::KeepAcrossLevels,
        Node { position_type: PositionType::Absolute, right: percent(6), top: percent(5),
            padding: UiRect::all(px(8)), ..default() },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.85)), ZIndex(100),
    )).with_children(|p| { p.spawn((Text::new("Games"), TextFont { font_size: FontSize::Px(16.0), ..default() })); });
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let rac1 = home.map(|h| h.join("Documents/ReRACData"));
    let rac1_status = rac1.as_deref().map(crate::disc_source::inspect_mobile_data);
    // Metadata alone cannot make an import-error screen claim the game is ready.
    let ready = game.is_some() && matches!(rac1_status, Some(Ok(())));
    let text = format!("Game Library\n\nRatchet & Clank 1\n{}\nData: ReRACData | Saves unchanged\n\nRatchet & Clank 2 — native module: {}\nVerified extraction/cache support missing.\nNot launchable. Never replace RAC1 data.\n\nUse your owned disc with the desktop extractor.",
        if ready { "Current native game loaded." } else { "Resources required — see import instructions." },
        if native_module_available(Game::Rac2) { "available" } else { "NOT IMPLEMENTED" });
    commands.spawn((LibraryPanel, target, crate::level_switch::KeepAcrossLevels,
        Node { display: Display::None, position_type: PositionType::Absolute,
            left: percent(10), right: percent(10), top: percent(8), bottom: percent(8),
            padding: UiRect::all(px(16)), flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween, ..default() },
        BackgroundColor(Color::srgb(0.04,0.04,0.06)), ZIndex(101),
    )).with_children(|p| {
        p.spawn((Text::new(text), TextFont { font_size: FontSize::Px(17.0), ..default() }));
        p.spawn((Button, Action::Close, Node { padding: UiRect::all(px(10)), ..default() },
            BackgroundColor(Color::srgb(0.15,0.2,0.3))))
            .with_children(|b| { b.spawn(Text::new(if ready { "Continue Ratchet & Clank 1" } else { "Close library" })); });
    });
    crate::ios::diagnostic("GAME_LIBRARY_READY", "rac1_module=YES rac2_module=NO rac2_launch=DISABLED storage_migration=NO");
}

fn buttons(actions: Query<(&Interaction, &Action), Changed<Interaction>>,
           mut panels: Query<&mut Node, With<LibraryPanel>>) {
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed { continue; }
        for mut panel in &mut panels {
            panel.display = match action { Action::Open => Display::Flex, Action::Close => Display::None };
        }
    }
}
