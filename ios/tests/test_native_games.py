"""Source contracts complement executed Rust identity/extractor tests and CI device build."""
from pathlib import Path
import plistlib
import unittest

ROOT = Path(__file__).resolve().parents[2]

class NativeGames(unittest.TestCase):
    def test_identity_is_shared_not_a_second_disc_database(self):
        db = (ROOT/'crates/rc-extract/src/build_db.rs').read_text()
        self.assertIn('pub use rc_formats::game_id::', db)
        self.assertNotIn('pub enum Game', db)
        self.assertIn('SCUS_972.68', (ROOT/'crates/rc-formats/src/game_id.rs').read_text())

    def test_runtime_routes_before_level_loading(self):
        disk = (ROOT/'crates/rc-engine/src/disc_source.rs').read_text()
        self.assertIn('require_native_game(', disk)
        self.assertIn('another_game_cannot_use_rac1_loaders_or_saves', disk)
        main = (ROOT/'crates/rc-engine/src/main.rs').read_text()
        self.assertLess(main.index('ios::prepare()'), main.index('level_switch::load_bundle'))

    def test_inspection_is_observation_not_extraction_or_execution(self):
        src = (ROOT/'crates/rc-extract/src/inspection.rs').read_text()
        for marker in ['identify::probe', '64 * 1024', 'image_sha1', 'rc2_hdr_present',
                       'DEEP_SCAN_FILESYSTEM_ONLY', 'FIRST_FATAL_BLOCKER', 'json::Value::Null',
                       'KNOWN_MISSING_CAPABILITIES', 'RAC2_NATIVE_GAMEPLAY_MODULE']:
            self.assertIn(marker, src)
        for operation in ['File::create', 'create_dir', 'std::fs::write', 'extract::extract', 'Disc::new']:
            self.assertNotIn(operation, src)

    def test_existing_storage_and_application_identity_are_preserved(self):
        ios = (ROOT/'crates/rc-engine/src/ios.rs').read_text()
        self.assertIn('documents.join("ReRACData")', ios)
        self.assertIn('support.join("memcard")', ios)
        self.assertIn('support.join("settings.json")', ios)
        project = (ROOT/'ios/ReRAC.xcodeproj/project.pbxproj').read_text()
        self.assertIn('PRODUCT_BUNDLE_IDENTIFIER = org.rerac.game.iospoc;', project)
        info = plistlib.loads((ROOT/'ios/Info.plist').read_bytes())
        self.assertEqual(info['CFBundleVersion'], '6')
        self.assertTrue(info['UIFileSharingEnabled'])

    def test_library_panel_is_persistent_and_does_not_dispatch_a_fake_module(self):
        src = (ROOT/'crates/rc-engine/src/ios_library.rs').read_text()
        for marker in ['KeepAcrossLevels', 'UiTargetCamera', 'PresentCamera',
                       'NOT IMPLEMENTED', 'Not launchable', 'Continue Ratchet & Clank 1']:
            self.assertIn(marker,src)
        for operation in ['remove_file', 'rename(', 'set_var', 'load_bundle', 'std::fs::write']:
            self.assertNotIn(operation,src)

    def test_ci_preserves_previous_gates_and_adds_extraction_contracts(self):
        src = (ROOT/'.github/workflows/build-ios-arm64.yml').read_text()
        for marker in ['ios/arm64-poc', 'ios/rc2-native',
                       'cargo xtask test-job engine ui --cargo-test',
                       'cargo xtask test-job formats extract data --cargo-test',
                       '--target aarch64-apple-ios -p rc-engine', 'collect_symbols.py',
                       'verify_ipa.py', 'game_id.rs']:
            self.assertIn(marker,src)

if __name__=='__main__': unittest.main()
