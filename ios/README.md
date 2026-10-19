# ReRAC native iPhone ARM64 port

## Verified baseline and current work

The user physically verified version 0.3.4 build 3, source
c9df823e939446e97b255af1a9d5ca28db874d79: native iPhone launch, Backbone One menu
navigation, gameplay and progression to planet three. This is not a claim that
the entire game is complete. The previous build and matching symbols remain in:
https://github.com/Bobnotbuilder/rerac/actions/runs/39077637396

Build 4 added native drawable aspect, safe HUD/menu placement and bounded power
diagnostics; its complete hosted run passed at commit 80d2fb1. Build 5 extends that
baseline with command-encoding/resource reuse, fixed resolution comparisons and
additional local counters. See [THERMAL_ENCODING.md](THERMAL_ENCODING.md) for the
user's A15 HUD observations, source findings, exact comparison protocol and
measurement limitations. No build-5 physical power improvement is claimed.

Preserve the existing Rust/Bevy architecture. No Swift/C++ game rewrite, emulator,
GameStudio engine change or Shared Core promotion is part of this port.

## Architecture and preserved fixes

Upstream game version 0.3.4; original upstream baseline:
8b2c9a92c48661bdc04fecd87c7e199d376d0b6a.
Bevy 0.19.1, wgpu 29.0.4 and Rust 1.99.1 remain pinned by the existing workspace/lock.
Keep the scene lifecycle backport and packaged licence notices described in
[SCENE_LIFECYCLE_REPAIR.md](SCENE_LIFECYCLE_REPAIR.md).
The physical startup failure was UIKit's missing UIScene adoption; it must not be
reintroduced. [BACKBONE_INPUT.md](BACKBONE_INPUT.md) describes the retained native
Apple GameController -> Bevy -> libpad2 input path.

The Xcode application target embeds the ordinary Rust executable using Bevy's
mobile pattern. The physical target is aarch64-apple-ios (not simulator/Catalyst),
device family iPhone, minimum requested iOS 16.0, with Metal required.
This is not an assertion that every iOS 16 phone meets game memory/performance needs.

## Build and package

Use the existing .github/workflows/build-ios-arm64.yml on ios/arm64-poc.
Manual dispatch and implementation-path pushes on this branch are supported.
The hosted xcode-27 ARM64 runner builds the physical device executable and native
unsigned Xcode app. There is no desktop game release/Intel compatibility gate.

Artifacts:
- ReRAC-iOS-arm64-unsigned.ipa and its SHA-256 file
- UUID-matched device dSYM, retained 90 days
- source/runner/toolchain/build/package evidence and simulator diagnostics
- separate headless engine/UI regression results using the documented xtask tier

The physical package verifier requires true ARM64 iOS Mach-O, the unchanged
org.rerac.game.iospoc bundle ID, system framework linkage, scene configuration,
file sharing, upstream shader bytes and licence notices. It rejects simulator/
Mac/Catalyst binaries, unbundled dylibs, embedded signatures/provisioning,
unexpected payloads, game data and archive traversal.
The IPA always contains Payload/ReRAC.app; the simulator never supplies its binary.

The source branch is isolated. Do not merge to main or modify release.yml merely
to produce an iPhone test artifact. Refer to Actions for the exact source SHA and
each actual result, not a prospective status copied from an older handoff.

## Signing, game files and saves

Sign/provision the unsigned IPA using Sideloadly. For updates use the same account
and effective signed bundle identifier; do not uninstall first. Signing changes
the unsigned checksum. No signing credentials are requested by the workflow.

The user supplies their own supported NTSC-U SCUS_971.99 extracted data.
Close ReRAC and copy the CONTENTS of the extractor output into:
Files -> On My iPhone -> ReRAC -> ReRACData.
toc.bin and extract-info.json must be directly inside it alongside levels/,
global/, boot/ and all other extracted directories. Do not copy an ISO or nest
another extracted folder. Complete transfer before launching.

Saves/settings/local diagnostics remain in Documents/ReRACSupport (memory cards:
memcard). Back up this folder through Files before updates. Nothing in build 4
deletes or migrates game input or saved progress. Missing/invalid input uses the
existing clear import/error screen rather than intentionally exiting.

No proprietary game data, device reports, credentials or signing material are
uploaded by CI. No on-device ISO extraction or automatic asset download is added.

## Acceptance boundary

Native compilation, package contracts, host headless tests and simulator fixtures
are distinct from iPhone gameplay. Data-dependent headless tests can skip without
extracted input. Hosted machines cannot establish physical Backbone operation,
actual presented FPS, iPhone battery/thermal behaviour or another planet-three
playthrough. Keep those outcomes separate in reports.
