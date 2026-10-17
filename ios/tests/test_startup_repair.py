import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('symbols',ROOT/'ios/collect_symbols.py')
symbols = importlib.util.module_from_spec(spec)
spec.loader.exec_module(symbols)
PIN = '94d6a23c425023810a0879aa7a03abb2e4f1decc'


class StartupRepair(unittest.TestCase):
    def test_winit_is_exactly_pinned_without_bevy_upgrade(self):
        cargo = (ROOT/'Cargo.toml').read_text()
        lock = (ROOT/'Cargo.lock').read_text()
        self.assertIn('rev = "'+PIN+'"',cargo)
        self.assertIn('git+https://github.com/molenick/winit?rev='+PIN+'#'+PIN,lock)
        self.assertIn('dpi = { version = "=0.1.2" }',cargo)
        self.assertIn('name = "bevy"\nversion = "0.19.1"',lock)

    def test_startup_logging_is_bounded_and_local(self):
        src = (ROOT/'crates/rc-engine/src/ios.rs').read_text()
        for token in ('256 * 1024','take(1500)','startup.log','RUST_PANIC',
                      'STARTUP_RESOURCE_CHECK','BEVY_STARTUP_SYSTEM_REACHED','FIRST_UPDATE_REACHED'):
            self.assertIn(token,src)
        self.assertNotIn('std::process::exit',src)
        self.assertNotIn('std::process::abort',src)

    def test_game_data_load_error_uses_existing_import_ui(self):
        src = (ROOT/'crates/rc-engine/src/main.rs').read_text()
        self.assertIn('BOOT_LEVEL_LOAD_FAILED',src)
        self.assertIn('ios::show_import_instructions(&format!',src)

    def test_symbols_match(self):
        value = 'UUID: 11266921-EBDA-3D4F-A34D-9714854DECEA (arm64) file'
        self.assertEqual(symbols.check_pair(value,value),'11266921-ebda-3d4f-a34d-9714854decea')

    def test_wrong_symbol_uuid_rejected(self):
        a = 'UUID: 11266921-EBDA-3D4F-A34D-9714854DECEA (arm64) file'
        b = 'UUID: 22266921-EBDA-3D4F-A34D-9714854DECEA (arm64) file'
        with self.assertRaisesRegex(ValueError,'does not match'):
            symbols.check_pair(a,b)

    def test_non_device_arch_symbols_rejected(self):
        with self.assertRaises(ValueError):
            symbols.parse_uuid('UUID: 11266921-EBDA-3D4F-A34D-9714854DECEA (x86_64) file')


if __name__ == '__main__':
    unittest.main()
