"""Synthetic packaging fixtures only; these do not compile or run the game."""
import importlib.util
from pathlib import Path
import plistlib
import stat
import struct
import tempfile
import unittest
import zipfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('verify_ipa', ROOT/'ios/verify_ipa.py')
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)


def macho(platform=2, cpu=0x0100000c, minimum=16 << 16, signed=False, external=None):
    commands = struct.pack('<6I', 0x32, 24, platform, minimum, 27 << 16, 0)
    count = 1
    for name in ('/System/Library/Frameworks/UIKit.framework/UIKit',
                 '/System/Library/Frameworks/Metal.framework/Metal', external):
        if name:
            raw = name.encode()+b'\0'
            raw += b'\0' * (-len(raw) % 8)
            commands += struct.pack('<6I', 0xc, 24+len(raw), 24, 0, 0, 0)+raw
            count += 1
    if signed:
        commands += struct.pack('<4I', 0x1d, 16, 0, 0)
        count += 1
    return struct.pack('<8I', 0xfeedfacf, cpu, 0, 2, count, len(commands), 0, 0)+commands


class IOSPackage(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.app = self.root/'ReRAC.app'
        self.app.mkdir()
        self.info = dict(CFBundleExecutable='ReRAC', CFBundlePackageType='APPL',
                         CFBundleShortVersionString='0.3.4', CFBundleSupportedPlatforms=['iPhoneOS'],
                         UIDeviceFamily=[1], UIFileSharingEnabled=True, LSSupportsOpeningDocumentsInPlace=True,
                         MinimumOSVersion='16.0', UIRequiredDeviceCapabilities=['arm64', 'metal'])
        (self.app/'Info.plist').write_bytes(plistlib.dumps(self.info))
        (self.app/'ReRAC').write_bytes(macho())
        (self.app/'ReRAC').chmod(0o755)
        (self.app/'LaunchScreen.storyboardc').mkdir()
        (self.app/'LaunchScreen.storyboardc/Info.plist').write_bytes(plistlib.dumps({}))
        self.source = self.root/'source'
        shaders = self.source/'crates/rc-engine/assets/shaders'
        shaders.mkdir(parents=True)
        (shaders/'test.wgsl').write_text('// synthetic fixture')
        (self.app/'assets/shaders').mkdir(parents=True)
        (self.app/'assets/shaders/test.wgsl').write_text('// synthetic fixture')

    def files(self):
        return verify.bundle_files(self.app, self.source, '0.3.4')

    def test_unsigned_device_bundle_passes(self):
        _, row = self.files()
        self.assertEqual(row['architecture'], 'arm64')
        self.assertEqual(row['build_version']['platform'], 2)

    def test_simulator_mac_and_catalyst_rejected(self):
        for platform in (1, 6, 7):
            with self.subTest(platform=platform), self.assertRaisesRegex(ValueError, 'physical iOS'):
                verify.device_macho(macho(platform=platform))

    def test_intel_rejected(self):
        with self.assertRaisesRegex(ValueError, 'ARM64'):
            verify.device_macho(macho(cpu=0x01000007))

    def test_signature_rejected(self):
        with self.assertRaisesRegex(ValueError, 'LC_CODE_SIGNATURE'):
            verify.device_macho(macho(signed=True))

    def test_newer_minimum_rejected(self):
        with self.assertRaisesRegex(ValueError, 'minimum iOS'):
            verify.device_macho(macho(minimum=17 << 16))

    def test_non_system_dependency_rejected(self):
        with self.assertRaisesRegex(ValueError, 'non-system'):
            verify.device_macho(macho(external='@rpath/libbevy_dylib.dylib'))

    def test_truncated_command_rejected(self):
        with self.assertRaises(ValueError):
            verify.device_macho(macho()[:-4])

    def test_copyrighted_payload_rejected(self):
        (self.app/'Ratchet.iso').write_bytes(b'synthetic forbidden placeholder')
        with self.assertRaisesRegex(ValueError, 'unexpected'):
            self.files()

    def test_missing_shader_rejected(self):
        (self.app/'assets/shaders/test.wgsl').unlink()
        with self.assertRaisesRegex(ValueError, 'shader'):
            self.files()

    def test_modified_shader_rejected(self):
        (self.app/'assets/shaders/test.wgsl').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'shader'):
            self.files()

    def test_missing_executable_permissions_rejected(self):
        (self.app/'ReRAC').chmod(0o644)
        with self.assertRaisesRegex(ValueError, 'permissions'):
            self.files()

    def test_files_transfer_keys_required(self):
        self.info['UIFileSharingEnabled'] = False
        (self.app/'Info.plist').write_bytes(plistlib.dumps(self.info))
        with self.assertRaisesRegex(ValueError, 'UIFileSharingEnabled'):
            self.files()

    def test_wrong_supported_platform_rejected(self):
        self.info['CFBundleSupportedPlatforms'] = ['iPhoneSimulator']
        (self.app/'Info.plist').write_bytes(plistlib.dumps(self.info))
        with self.assertRaisesRegex(ValueError, 'CFBundleSupportedPlatforms'):
            self.files()

    def archive(self, files, extra=None):
        target = self.root/'fixture.ipa'
        with zipfile.ZipFile(target, 'w') as archive:
            for name, raw in files.items():
                item = zipfile.ZipInfo('Payload/ReRAC.app/'+name)
                item.create_system = 3
                item.external_attr = (stat.S_IFREG | 0o755) << 16
                archive.writestr(item, raw)
            if extra:
                archive.writestr(extra, b'invalid fixture')
        return target

    def test_ipa_payload_layout_and_bytes(self):
        files, _ = self.files()
        verify.validate_archive(self.archive(files), files)

    def test_ipa_traversal_rejected(self):
        files, _ = self.files()
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            verify.validate_archive(self.archive(files, '../escape'), files)

    def test_ipa_extra_file_rejected(self):
        files, _ = self.files()
        with self.assertRaisesRegex(ValueError, 'unexpected'):
            verify.validate_archive(self.archive(files, 'Payload/Ratchet.iso'), files)

    def test_xcode_native_device_and_signing_contract(self):
        project = (ROOT/'ios/ReRAC.xcodeproj/project.pbxproj').read_text()
        for value in ('product-type.application', 'SDKROOT = iphoneos', 'ARCHS = arm64',
                      'CODE_SIGNING_ALLOWED = NO', 'ReRAC.app', '../crates/rc-engine/assets'):
            self.assertIn(value, project)
        ET.parse(ROOT/'ios/ReRAC.xcodeproj/xcshareddata/xcschemes/ReRAC.xcscheme')
        ET.parse(ROOT/'ios/LaunchScreen.storyboard')

    def test_workflow_device_not_simulator(self):
        workflow = (ROOT/'.github/workflows/build-ios-arm64.yml').read_text()
        for value in ('runs-on: xcode-27', '--target aarch64-apple-ios', '--release --locked',
                      'generic/platform=iOS', 'CODE_SIGNING_ALLOWED=NO', 'if: always()',
                      'ios/verify_ipa.py', 'shasum -a 256', '--version-json'):
            self.assertIn(value, workflow)
        self.assertNotIn('--target aarch64-apple-ios-sim', workflow)
        self.assertNotIn('upload-artifact@v4\n        with:\n          name: game-data', workflow)

    def test_desktop_startup_preserved_and_ios_validates_extraction(self):
        main = (ROOT/'crates/rc-engine/src/main.rs').read_text()
        self.assertIn('#[cfg(not(target_os = "ios"))]\n    let root = disc_source::startup();', main)
        disc = (ROOT/'crates/rc-engine/src/disc_source.rs').read_text()
        self.assertIn('check(&root, Origin::Env).map_err', disc)
        host = (ROOT/'crates/rc-engine/src/ios.rs').read_text()
        for value in ('ReRACData', 'RC_SAVE_DIR', 'RC_SETTINGS_FILE', 'show_import_instructions'):
            self.assertIn(value, host)


if __name__ == '__main__':
    unittest.main()
