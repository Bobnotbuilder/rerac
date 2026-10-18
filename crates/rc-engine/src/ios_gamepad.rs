//! Apple GameController -> Bevy's normal Gamepad -> existing input_map -> libpad2.
//! Native access is main-thread-only; no keyboard emulation or replacement game logic.
use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent,
    RawGamepadEvent, RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent};
use bevy::prelude::*;
use std::rc::Rc;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Snapshot {
    identity: u64,
    axes: [f32; 4],
    buttons: [f32; 16],
}

extern "C" {
    fn rr_gc_poll(out: *mut Snapshot) -> i32;
    #[cfg(target_abi = "sim")]
    fn rr_gc_simulated_snapshot(mask: u32, axes: *const f32, out: *mut Snapshot) -> i32;
}

const AXES: [GamepadAxis; 4] = [GamepadAxis::LeftStickX, GamepadAxis::LeftStickY,
    GamepadAxis::RightStickX, GamepadAxis::RightStickY];
const BUTTONS: [GamepadButton; 16] = [GamepadButton::South, GamepadButton::East,
    GamepadButton::West, GamepadButton::North, GamepadButton::LeftTrigger,
    GamepadButton::RightTrigger, GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2,
    GamepadButton::Start, GamepadButton::Select, GamepadButton::LeftThumb,
    GamepadButton::RightThumb, GamepadButton::DPadUp, GamepadButton::DPadDown,
    GamepadButton::DPadLeft, GamepadButton::DPadRight];

// Rc makes this a NonSend resource: Bevy must execute poll on its main thread.
#[derive(Default)]
struct NativeController {
    _main_thread: Rc<()>,
    entity: Option<Entity>,
    identity: u64,
    first_input_logged: bool,
    thread_error_logged: bool,
}

pub struct IOSGameControllerPlugin;
impl Plugin for IOSGameControllerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(NativeController::default())
            .add_systems(PreUpdate, poll.before(bevy::input::InputSystems));
        crate::ios::diagnostic("GAMECONTROLLER_READY", "native_extended_gamepad main_thread_poll");
    }
}

fn value(v: f32, min: f32) -> f32 { if v.is_finite() { v.clamp(min, 1.0) } else { 0.0 } }

fn input_events(frame: &Snapshot, entity: Entity) -> Vec<RawGamepadEvent> {
    AXES.iter().zip(frame.axes).map(|(&axis, v)|
        RawGamepadAxisChangedEvent::new(entity, axis, value(v, -1.0)).into())
        .chain(BUTTONS.iter().zip(frame.buttons).map(|(&button, v)|
            RawGamepadButtonChangedEvent::new(entity, button, value(v, 0.0)).into()))
        .collect()
}

fn poll(mut native: NonSendMut<NativeController>, mut commands: Commands,
        mut connections: MessageWriter<GamepadConnectionEvent>,
        mut events: MessageWriter<RawGamepadEvent>) {
    let mut frame = Snapshot::default();
    // SAFETY: repr(C) layout matches the shim; NonSendMut enforces the main thread.
    let result = unsafe { rr_gc_poll(&mut frame) };
    if result < 0 {
        if !native.thread_error_logged {
            native.thread_error_logged = true;
            crate::ios::diagnostic("GAMECONTROLLER_WRONG_THREAD", "input_released");
        }
        frame = Snapshot::default();
    }
    if frame.identity != native.identity {
        if native.identity != 0 {
            if let Some(entity) = native.entity {
                for event in input_events(&Snapshot::default(), entity) { events.write(event); }
                connections.write(GamepadConnectionEvent::new(entity, GamepadConnection::Disconnected));
            }
            crate::ios::diagnostic("GAMECONTROLLER_DISCONNECTED", "held_input_cleared");
        }
        native.identity = frame.identity;
        native.first_input_logged = false;
        if frame.identity != 0 {
            let entity = *native.entity.get_or_insert_with(|| commands.spawn_empty().id());
            connections.write(GamepadConnectionEvent::new(entity, GamepadConnection::Connected {
                name: "iOS Extended Gamepad".into(), vendor_id: None, product_id: None,
            }));
            crate::ios::diagnostic("GAMECONTROLLER_CONNECTED", "extended_profile");
        }
    }
    if frame.identity != 0 {
        if !native.first_input_logged && (frame.axes.iter().any(|v| v.abs() > 0.2)
            || frame.buttons.iter().any(|v| *v > 0.5)) {
            native.first_input_logged = true;
            crate::ios::diagnostic("GAMECONTROLLER_INPUT_RECEIVED", "forwarded_to_existing_pad_mapping");
        }
        for event in input_events(&frame, native.entity.unwrap()) { events.write(event); }
    }
}

/// Runs only in the CI simulator, with an explicit environment opt-in. Uses Apple's
/// synthetic extended profile, then the actual Bevy InputPlugin + input_map::sample_system
/// and rc_game::PadState, not a second mapping or a physical-controller claim.
pub fn simulator_self_test() {
    #[cfg(target_abi = "sim")]
    if std::env::var("RERAC_IOS_CONTROLLER_SELFTEST").as_deref() == Ok("1") {
        crate::display::simulator_contracts();
        crate::hud_render::simulator_safe_ui_contracts();
        use rc_game::pad::{button, PadState};
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .init_resource::<crate::input_map::PadFrame>()
            .add_systems(PreUpdate, crate::input_map::sample_system.after(bevy::input::InputSystems));
        let entity = app.world_mut().spawn_empty().id();
        let connect = |app: &mut App| { app.world_mut().write_message(GamepadConnectionEvent::new(entity,
            GamepadConnection::Connected { name: "CI Synthetic Extended Gamepad".into(),
                vendor_id: None, product_id: None })); };
        connect(&mut app);
        app.update();
        let sample = |app: &mut App, mask: u32, axes: [f32; 4]| {
            let mut frame = Snapshot::default();
            assert_eq!(unsafe { rr_gc_simulated_snapshot(mask, axes.as_ptr(), &mut frame) }, 1);
            assert_eq!(frame.identity, 1);
            for event in input_events(&frame, entity) { app.world_mut().write_message(event); }
            app.update();
            app.world().resource::<crate::input_map::PadFrame>().0
        };
        let expected = [button::CROSS, button::CIRCLE, button::SQUARE, button::TRIANGLE,
            button::L1, button::R1, button::L2, button::R2, button::START, button::SELECT,
            button::L3, button::R3, button::UP, button::DOWN, button::LEFT, button::RIGHT];
        let mut pad = PadState::default();
        for (i, bit) in expected.into_iter().enumerate() {
            let input = sample(&mut app, 1 << i, [0.0; 4]);
            pad.update(Some(&input.bytes()), false);
            assert_eq!(pad.raw & 0xffff, bit, "native button {i}");
            assert_ne!(pad.raw_pressed & bit, 0);
            let held = sample(&mut app, 1 << i, [0.0; 4]);
            pad.update(Some(&held.bytes()), false);
            assert_eq!(pad.raw_pressed & bit, 0, "held buttons must not repeat presses");
            let release = sample(&mut app, 0, [0.0; 4]);
            pad.update(Some(&release.bytes()), false);
            assert_eq!(pad.raw & 0xffff, 0);
            assert_ne!(pad.raw_released & bit, 0);
        }
        let sticks = sample(&mut app, 0, [1.0, 1.0, -1.0, -1.0]);
        assert_eq!((sticks.lx, sticks.ly, sticks.rx, sticks.ry), (255, 0, 0, 255));
        app.world_mut().write_message(GamepadConnectionEvent::new(entity, GamepadConnection::Disconnected));
        app.update();
        assert_eq!(app.world().resource::<crate::input_map::PadFrame>().0, rc_game::pad::PadInput::neutral());
        connect(&mut app);
        app.update();
        let resumed = sample(&mut app, 1 << 8, [0.0; 4]);
        pad.update(Some(&resumed.bytes()), false);
        assert_eq!(pad.raw & 0xffff, button::START);
        let mut frame = Snapshot { axes: [f32::NAN; 4], buttons: [f32::INFINITY; 16], ..Default::default() };
        frame.identity = 1;
        for event in input_events(&frame, entity) { app.world_mut().write_message(event); }
        app.update();
        assert_eq!(app.world().resource::<crate::input_map::PadFrame>().0, rc_game::pad::PadInput::neutral());
        crate::ios::diagnostic("GAMECONTROLLER_SYNTHETIC_SELFTEST_PASS",
            "16_buttons sticks hold_release disconnect reconnect finite_values physical_backbone=NOT_TESTED");
    }
}
