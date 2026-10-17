#!/usr/bin/env python3
"""Launch the actual ReRAC simulator binary with NO game input; not device/gameplay acceptance."""
import json
import os
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT/'evidence'


def run(args, timeout=180, check=True):
    return subprocess.run(args,check=check,text=True,stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT,timeout=timeout).stdout


def main():
    EVIDENCE.mkdir(exist_ok=True)
    env = os.environ.copy()
    env['SDKROOT'] = run(['xcrun','--sdk','iphonesimulator','--show-sdk-path']).strip()
    env['CARGO_PROFILE_RELEASE_DEBUG'] = '0'
    env['CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO'] = 'off'
    with (EVIDENCE/'simulator-cargo.log').open('w') as log:
        subprocess.run(['cargo','build','--release','--locked','--target','aarch64-apple-ios-sim',
                        '-p','rc-engine','--bin','rerac'],cwd=ROOT,env=env,stdout=log,
                       stderr=subprocess.STDOUT,check=True,timeout=2400)
    binary = Path(os.environ['CARGO_TARGET_DIR'])/'aarch64-apple-ios-sim/release/rerac'
    env['RERAC_RUST_EXECUTABLE'] = str(binary)
    derived = Path(os.environ['RUNNER_TEMP'])/'rerac-ios-simulator'
    with (EVIDENCE/'simulator-xcodebuild.log').open('w') as log:
        subprocess.run(['xcodebuild','-project','ios/ReRAC.xcodeproj','-scheme','ReRAC',
                        '-configuration','Release','-sdk','iphonesimulator','-destination',
                        'generic/platform=iOS Simulator','-derivedDataPath',str(derived),
                        'ARCHS=arm64','ONLY_ACTIVE_ARCH=NO','CODE_SIGNING_ALLOWED=NO',
                        'CODE_SIGNING_REQUIRED=NO','CODE_SIGN_IDENTITY=','build'],
                       cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=300)
    app = derived/'Build/Products/Release-iphonesimulator/ReRAC.app'
    run(['codesign','--force','--sign','-',str(app)])  # simulator-only ad hoc; device IPA stays unsigned
    runtimes = json.loads(run(['xcrun','simctl','list','runtimes','--json']))['runtimes']
    runtime = next((r for r in runtimes if r.get('isAvailable') and r['version'].startswith('27.')),None)
    if not runtime:
        raise RuntimeError('IOS_27_SIMULATOR_RUNTIME_UNAVAILABLE')
    types = json.loads(run(['xcrun','simctl','list','devicetypes','--json']))['devicetypes']
    available = json.loads(run(['xcrun','simctl','list','devices','available','--json']))['devices']
    template = next((d for d in available.get(runtime['identifier'],[])
                     if d.get('isAvailable') and d['name'].startswith('iPhone')),None)
    if not template:
        raise RuntimeError('IOS_27_COMPATIBLE_IPHONE_TEMPLATE_UNAVAILABLE')
    device_type = next(d for d in types if d['name'] == template['name'])
    device = run(['xcrun','simctl','create','ReRACStartupSmoke',
                  device_type['identifier'],runtime['identifier']]).strip()
    try:
        run(['xcrun','simctl','boot',device])
        run(['xcrun','simctl','bootstatus',device,'-b'],timeout=240)
        run(['xcrun','simctl','install',device,str(app)])
        launch_env = os.environ.copy()
        launch_env['SIMCTL_CHILD_RERAC_IOS_CONTROLLER_SELFTEST'] = '1'
        launched = subprocess.run(['xcrun','simctl','launch',device,'org.rerac.game.iospoc'],
                                  env=launch_env,check=True,text=True,stdout=subprocess.PIPE,
                                  stderr=subprocess.STDOUT,timeout=180).stdout
        (EVIDENCE/'simulator-launch.txt').write_text(launched)
        pid = int(re.search(r': (\d+)\s*$',launched).group(1))
        container = Path(run(['xcrun','simctl','get_app_container',device,
                              'org.rerac.game.iospoc','data']).strip())
        startup = container/'Documents/ReRACSupport/startup.log'
        deadline = time.monotonic()+90
        while time.monotonic()<deadline:
            text = startup.read_text() if startup.exists() else ''
            if 'FIRST_UPDATE_REACHED' in text:
                break
            time.sleep(1)
        else:
            raise RuntimeError('SIMULATOR_STARTUP_FIRST_UPDATE_NOT_REACHED')
        time.sleep(5)
        os.kill(pid,0)  # Simulator app PID is a host process; signal zero checks aliveness only.
        text = startup.read_text()
        (EVIDENCE/'simulator-startup.log').write_text(text)
        for marker in ('IMPORT_SCREEN_REQUESTED','BEVY_STARTUP_SYSTEM_REACHED',
                       'RENDER_DEVICE_READY','toc=false','extract_info=false',
                       'GAMECONTROLLER_SYNTHETIC_SELFTEST_PASS','GAMECONTROLLER_READY'):
            if marker not in text:
                raise RuntimeError('SIMULATOR_MISSING_MARKER '+marker)
        run(['xcrun','simctl','io',device,'screenshot',
             str(EVIDENCE/'import-screen-ios27.png')])
        report = dict(simulator_runtime=runtime['version'],startup='PASS',
                      import_path_without_game_data='PASS',alive_after_first_update=True,
                      screenshot='import-screen-ios27.png',physical_device='NOT_TESTED',
                      gameplay='NOT_TESTED',backbone_physical='NOT_TESTED',
                      native_controller_to_bevy_to_pad_synthetic='PASS')
        (EVIDENCE/'simulator-startup-result.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report))
    finally:
        try:
            logs = run(['xcrun','simctl','spawn',device,'log','show','--last','2m',
                        '--style','compact','--predicate','process == "ReRAC"'],check=False)
            (EVIDENCE/'simulator-console-tail.log').write_text(logs[-65536:])
        finally:
            run(['xcrun','simctl','shutdown',device],check=False)


if __name__ == '__main__':
    main()
