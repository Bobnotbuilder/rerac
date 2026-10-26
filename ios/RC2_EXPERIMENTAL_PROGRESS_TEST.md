# RC2 Experimental Progress Test — Build 10

One intermediate iPhone test explicitly authorized on 26 October 2026. This is an incomplete development build, not complete or playable-Aranos acceptance. Further user-facing test builds still require authorization.

Build source: public `a6c155ea4e540bee5c5d5987b032898288e55d71`, equivalent local committed source `cf9c7d130cb97fce3e6b9769c1270a0cfc07a6a7`. Version 0.3.4, CFBundleVersion 10, physical `aarch64-apple-ios`, bundle `org.rerac.game.iospoc`. Native build run: https://github.com/Bobnotbuilder/rerac/actions/runs/40304344555. Delivery requires successful current-source compilation, packaging, archive verification and matching device symbols; pending checks are not acceptance.

The test snapshot includes completed camera FOV/scalar helpers and earlier validated music, ground, object ownership and crate initialization work. The newer per-frame fog/culling adapters remain preserved in the original working tree and private checkpoint; they require the unfinished GcView owner bridge and are deliberately excluded from the build snapshot. No files or history were reset or discarded.

## Install as an update

1. Preserve a copy of existing ReRAC Documents, especially ReRACSupport and its memory card, before an app update. Do not uninstall ReRAC.
2. Extract the downloaded IPA artifact ZIP. Use the physical-device `.ipa` inside with the established Sideloadly workflow, using the same Apple ID and effective bundle-identity options as the installed app. Install over the existing app. The unsigned bundle identity remains `org.rerac.game.iospoc`.
3. Keep the existing private `Documents/RC2Native` resources in place. No ISO, WAD, converted model, texture or audio is included in the download. Do not copy RC2 data over `ReRACData`.
4. The existing explicit RC2 development entry is `Documents/RC2Native/ENABLE_NATIVE_RC2.txt`, containing exactly `RC2_NATIVE_ARANOS_EXPERIMENTAL_V1` followed by one newline. Existing valid markers need no replacement. This opt-in does not enable RC2 Play in the normal game library. Remove only this marker while the app is closed to return to the normal RAC1 launch route.
5. Launch with Backbone connected. Check title/menu, New Game, original Aranos/character rendering, movement, jumping, follow camera, ground/raised geometry and frontend/cinematic audio. Report what actually works on this device; none of these current-source device results is assumed from automated tests.

## Boundaries

The existing experimental movement/scene path is available. Its New Game menu entry does not establish complete original New Game/save/world startup. Original initializer execution remains blocked at object24/class1330: the engine needs camera-bank/live-default bindings at `0x2da160`; the explicitly bound original-data CPU fixture reaches `0x2d79b8 → 0x2e5640`, whose full native projection publication remains unfinished.

Do not expect crate destruction, bolt collection, wrench/weapon combat, enemies, complete HUD/level music, mission scripting, checkpoints, death/respawn or the legitimate Oozla transition to work. Native crate initialization and stereo body audio tests are CPU component/owner-chain evidence, not interactive crate or full in-app level-music support. No initializer callback is skipped to claim progress. Unsupported hero states and native-world errors retain their exact diagnostic identifiers.

## Logs and feedback

After reproducing an issue, close the app and copy `Documents/ReRACSupport/startup.log` and `startup.previous.log` if present. These contain source/build identity, startup/render status, Backbone connection/button events, frontend/menu sound requests, unsupported hero states, native-world blocker diagnostics when that path executes, performance samples and Rust panic reports. Also provide any iOS ReRAC crash `.ips`, screenshot/video and reproduction steps. An abrupt native crash may require the OS crash report rather than a Rust panic entry.

For a short collision/input capture only, `Documents/RC2Native/ENABLE_COLLISION_TRACE.txt` may contain exactly `RC2_COLLISION_INPUT_TRACE_V1` plus newline. This enables verbose per-tick diagnostics and can affect performance; remove only that optional marker after the capture. Settings for this RC2 route remain in `RC2NativeSupport/settings.json`; RAC1 saves remain in `ReRACSupport/memcard`. RC2 saving is not implemented.

Physical iPhone 13 Pro Max/Backbone validation, speaker output and frame-time/memory/stability measurements remain the user's next test. Automated Metal checks use synthetic assets; hosted runners do not contain the private original-game ISO.
