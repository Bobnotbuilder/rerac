#!/usr/bin/env python3
"""Device-target and bundle checks, never a device launch or gameplay claim."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import plistlib
import stat
import struct
import subprocess
import zipfile


def device_macho(raw):
    if len(raw) < 32 or raw[:4] != b'\xcf\xfa\xed\xfe':
        raise ValueError('expected thin 64-bit Mach-O')
    _, cpu, subtype, kind, count, size, _, _ = struct.unpack_from('<8I', raw)
    if cpu != 0x0100000c or kind != 2 or subtype & 0x00ffffff != 0:
        raise ValueError('expected ordinary ARM64 executable, not Intel/arm64e')
    end, offset = 32 + size, 32
    if end > len(raw) or count > size // 8:
        raise ValueError('invalid load-command bounds')
    platforms, dependencies, signatures = [], [], 0
    for _ in range(count):
        if offset + 8 > end:
            raise ValueError('truncated load command')
        command, length = struct.unpack_from('<II', raw, offset)
        if length < 8 or offset + length > end:
            raise ValueError('invalid load command size')
        if command == 0x32:
            if length < 24:
                raise ValueError('truncated build version')
            platform, minimum, sdk = struct.unpack_from('<3I', raw, offset + 8)
            if platform != 2:
                raise ValueError('not physical iOS: simulator/macOS/Catalyst rejected')
            if minimum > 16 << 16:
                raise ValueError('minimum iOS exceeds requested 16.0 deployment')
            platforms.append({'platform': platform, 'minimum': minimum, 'sdk': sdk})
        if command == 0x1d:
            signatures += 1
        if command in (0xc, 0x80000018, 0x8000001f, 0x20, 0x80000023):
            if length < 24:
                raise ValueError('invalid dylib load command')
            start = struct.unpack_from('<I', raw, offset+8)[0]
            if not 24 <= start < length:
                raise ValueError('invalid dylib name')
            value = raw[offset+start:offset+length].split(b'\0', 1)[0].decode()
            if not value.startswith(('/System/Library/', '/usr/lib/')):
                raise ValueError('unpackaged non-system dependency: '+value)
            dependencies.append(value)
        offset += length
    if offset != end or len(platforms) != 1:
        raise ValueError('one iOS LC_BUILD_VERSION required')
    if signatures:
        raise ValueError('unsigned deliverable must not contain LC_CODE_SIGNATURE')
    return {'architecture': 'arm64', 'build_version': platforms[0], 'dependencies': dependencies}


def bundle_files(app, source, version):
    if app.name != 'ReRAC.app' or not app.is_dir():
        raise ValueError('ReRAC.app required')
    paths = list(app.rglob('*'))
    if any(p.is_symlink() for p in paths):
        raise ValueError('bundle symlinks prohibited')
    files = {str(p.relative_to(app)): p.read_bytes() for p in paths if p.is_file()}
    if not {'ReRAC', 'Info.plist'} <= files.keys():
        raise ValueError('missing app executable/plist')
    info = plistlib.loads(files['Info.plist'])
    for key, value in {'CFBundleExecutable':'ReRAC', 'CFBundlePackageType':'APPL',
                       'CFBundleIdentifier':'org.rerac.game.iospoc',
                       'CFBundleShortVersionString':version,
                       'CFBundleSupportedPlatforms':['iPhoneOS'], 'UIDeviceFamily':[1],
                       'UIFileSharingEnabled':True, 'LSSupportsOpeningDocumentsInPlace':True}.items():
        if info.get(key) != value:
            raise ValueError('bundle plist mismatch: '+key)
    if info.get('MinimumOSVersion') != '16.0':
        raise ValueError('unexpected minimum iOS')
    if info.get('GCSupportedGameControllers') != [{'ProfileName':'ExtendedGamepad'}]:
        raise ValueError('native extended controller declaration missing')
    manifest = info.get('UIApplicationSceneManifest', {})
    scenes = manifest.get('UISceneConfigurations', {}).get('UIWindowSceneSessionRoleApplication', [])
    if manifest.get('UIApplicationSupportsMultipleScenes') is not False or len(scenes) != 1:
        raise ValueError('single UIKit scene manifest required')
    if scenes[0].get('UISceneClassName') != 'UIWindowScene':
        raise ValueError('UIWindowScene class required')
    if files.get('ThirdPartyNotices.txt') != (source/'ios/ThirdPartyNotices.txt').read_bytes():
        raise ValueError('dependency license notices missing or changed')
    if info.get('UIRequiredDeviceCapabilities') != ['arm64', 'metal']:
        raise ValueError('ARM64/Metal capability declaration required')
    if not (app/'ReRAC').stat().st_mode & 0o111:
        raise ValueError('executable permissions missing')
    shaders = {str(p.relative_to(source/'crates/rc-engine')):p.read_bytes()
               for p in (source/'crates/rc-engine/assets/shaders').glob('*.wgsl')}
    if not shaders:
        raise ValueError('upstream shaders absent')
    for path, raw in files.items():
        if path in ('ReRAC', 'Info.plist', 'PkgInfo', 'ThirdPartyNotices.txt'):
            continue
        if path.startswith('LaunchScreen.storyboardc/'):
            continue
        if path not in shaders:
            raise ValueError('unexpected bundled payload: '+path)
    for path, raw in shaders.items():
        if files.get(path) != raw:
            raise ValueError('shader bytes differ: '+path)
    if not any(p.startswith('LaunchScreen.storyboardc/') for p in files):
        raise ValueError('compiled launch storyboard absent')
    macho = device_macho(files['ReRAC'])
    for framework in ('UIKit.framework/', 'Metal.framework/', 'GameController.framework/'):
        if not any(framework in d for d in macho['dependencies']):
            raise ValueError('required system framework missing: '+framework)
    return files, macho


def validate_archive(ipa, files):
    with zipfile.ZipFile(ipa) as archive:
        names = archive.namelist()
        if len(set(names)) != len(names) or archive.testzip() is not None:
            raise ValueError('duplicate/corrupt IPA entries')
        expected = {'Payload/ReRAC.app/'+name:raw for name, raw in files.items()}
        actual = {}
        for item in archive.infolist():
            path = PurePosixPath(item.filename)
            if path.is_absolute() or '..' in path.parts or '\\' in item.filename:
                raise ValueError('unsafe IPA path')
            if stat.S_ISLNK(item.external_attr >> 16):
                raise ValueError('IPA symlink prohibited')
            if item.is_dir():
                if not any(n.startswith(item.filename) for n in expected):
                    raise ValueError('unexpected IPA directory')
                continue
            if item.filename not in expected:
                raise ValueError('unexpected IPA file')
            if item.file_size != len(expected[item.filename]):
                raise ValueError('IPA size mismatch')
            if item.filename.endswith('/ReRAC') and not (item.external_attr >> 16) & 0o111:
                raise ValueError('IPA executable permission lost')
            actual[item.filename] = archive.read(item)
        if actual != expected:
            raise ValueError('IPA/app content mismatch')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', required=True, type=Path)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--evidence', required=True, type=Path)
    parser.add_argument('--version', required=True)
    args = parser.parse_args()
    files, macho = bundle_files(args.app, args.source, args.version)
    args.output.mkdir(parents=True, exist_ok=True)
    ipa = args.output/'ReRAC-iOS-arm64-unsigned.ipa'
    with zipfile.ZipFile(ipa, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, raw in sorted(files.items()):
            entry = zipfile.ZipInfo('Payload/ReRAC.app/'+name)
            entry.create_system = 3
            entry.external_attr = (stat.S_IFREG | (0o755 if name == 'ReRAC' else 0o644)) << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(entry, raw)
    validate_archive(ipa, files)
    sha = hashlib.sha256(ipa.read_bytes()).hexdigest()
    (args.output/(ipa.name+'.sha256')).write_text(sha+'  '+ipa.name+'\n')
    commit = subprocess.check_output(['git','-C',str(args.source),'rev-parse','HEAD'], text=True).strip()
    report = {'source_commit':commit, 'version':args.version, 'target':'aarch64-apple-ios',
              'ipa_sha256':sha, 'macho':macho, 'shader_count':sum(n.endswith('.wgsl') for n in files),
              'bundle_and_archive_checks':'PASS', 'signing':'UNSIGNED',
              'physical_device_launch':'NOT_TESTED', 'audio':'NOT_TESTED', 'gameplay':'NOT_TESTED'}
    args.evidence.mkdir(parents=True, exist_ok=True)
    (args.evidence/'ipa-verification.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
