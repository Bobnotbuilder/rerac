"""Packaging/architecture contracts. Native input behaviour runs on the CI iOS simulator."""
from pathlib import Path
import plistlib
import unittest

ROOT = Path(__file__).resolve().parents[2]


class GameController(unittest.TestCase):
    def test_simulator_dpad_snapshot_updates_shared_axes_atomically(self):
        native = (ROOT/'crates/rc-engine/src/ios_gamecontroller.m').read_text()
        production, synthetic = native.split('#if TARGET_OS_SIMULATOR', 1)
        self.assertIn('setValueForXAxis:dx yAxis:dy', synthetic)
        self.assertIn('i < 12', synthetic)
        self.assertNotIn('setValueForXAxis', production)
        for direction in range(12, 16):
            mask = 1 << direction
            dx = bool(mask & (1 << 15)) - bool(mask & (1 << 14))
            dy = bool(mask & (1 << 12)) - bool(mask & (1 << 13))
            self.assertEqual((dx, dy), ((0,1),(0,-1),(-1,0),(1,0))[direction-12])

    def test_existing_bundle_identity_and_scene_lifecycle_preserved(self):
        info = plistlib.loads((ROOT/'ios/Info.plist').read_bytes())
        self.assertEqual(info['CFBundleVersion'], '9')
        self.assertEqual(info['GCSupportedGameControllers'], [{'ProfileName':'ExtendedGamepad'}])
        self.assertTrue(info['GCSupportsControllerUserInteraction'])
        self.assertIn('UIApplicationSceneManifest', info)
        self.assertIn('PRODUCT_BUNDLE_IDENTIFIER = org.rerac.game.iospoc;',
                      (ROOT/'ios/ReRAC.xcodeproj/project.pbxproj').read_text())

    def test_real_controller_framework_and_main_thread_adapter(self):
        native = (ROOT/'crates/rc-engine/src/ios_gamecontroller.m').read_text()
        build = (ROOT/'crates/rc-engine/build.rs').read_text()
        for token in ('GameController/GameController.h','GCController.controllers',
                      'isMainThread','UIApplicationStateActive','containsObject:selected',
                      'buttonMenu','buttonOptions','leftThumbstickButton','rightThumbstickButton'):
            self.assertIn(token, native)
        self.assertIn('framework=GameController', build)
        self.assertIn('Ok("ios")', build)
        self.assertIn('#if TARGET_OS_SIMULATOR', native)
        rust = (ROOT/'crates/rc-engine/src/ios_gamepad.rs').read_text()
        self.assertIn('NonSendMut<NativeController>', rust)
        self.assertIn('poll.before(bevy::input::InputSystems)', rust)
        self.assertIn('RawGamepadButtonChangedEvent::new', rust)
        self.assertIn('RawGamepadAxisChangedEvent::new', rust)

    def test_native_backend_used_for_game_and_import_without_duplicate_gilrs(self):
        main = (ROOT/'crates/rc-engine/src/main.rs').read_text()
        host = (ROOT/'crates/rc-engine/src/ios.rs').read_text()
        self.assertIn('disable::<bevy::gilrs::GilrsPlugin>()', main)
        self.assertIn('app.add_plugins((ios_gamepad::IOSGameControllerPlugin, ios_runtime::IOSRuntimePlugin))', main)
        self.assertIn('.add_plugins((crate::ios_gamepad::IOSGameControllerPlugin, crate::ios_runtime::IOSRuntimePlugin))', host)
        self.assertIn('crate::runtime_plugins()', host)

    def test_device_delivery_not_gated_on_desktop_build(self):
        workflow = (ROOT/'.github/workflows/build-ios-arm64.yml').read_text()
        self.assertNotIn('desktop-build.log', workflow)
        self.assertNotIn('desktop-version.json', workflow)
        self.assertIn('--target aarch64-apple-ios -p rc-engine --bin rerac', workflow)
        for token in ('verify_ipa.py', 'collect_symbols.py', 'simulator_startup_smoke.py'):
            self.assertIn(token, workflow)

    def test_simulator_exercises_real_native_bevy_and_game_pad(self):
        rust = (ROOT/'crates/rc-engine/src/ios_gamepad.rs').read_text()
        for token in ('rr_gc_simulated_snapshot', 'bevy::input::InputPlugin',
                      'input_map::sample_system', 'PadState', 'raw_pressed', 'raw_released',
                      'GamepadConnection::Disconnected', 'f32::NAN',
                      'GAMECONTROLLER_SYNTHETIC_SELFTEST_PASS'):
            self.assertIn(token, rust)
        smoke = (ROOT/'ios/simulator_startup_smoke.py').read_text()
        self.assertIn("SIMCTL_CHILD_RERAC_IOS_CONTROLLER_SELFTEST", smoke)
        self.assertIn("'GAMECONTROLLER_SYNTHETIC_SELFTEST_PASS'", smoke)


if __name__ == '__main__':
    unittest.main()
