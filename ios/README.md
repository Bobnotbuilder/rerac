# ReRAC native iPhone ARM64 proof of concept

Status: source implementation and synthetic checks only. Not compiled, installed,
launched, or gameplay-tested. No real IPA exists yet.

## Baseline and approach

Official source: https://github.com/re-rac/rerac

ReRAC 0.3.4, commit 8b2c9a92c48661bdc04fecd87c7e199d376d0b6a.
This is also the inspected Bobnotbuilder/rerac fork baseline. Bevy is 0.19.1,
wgpu is 29.0.4. Rust 1.99.1 is pinned for the proposed hosted build.

This source overlay requires the full repository at that exact baseline. It is
not a standalone source checkout, executable or IPA. Apply its files to a clean
checkout of the fork after reviewing the two modified Rust modules. Do not
overwrite newer upstream work with the overlay without rebasing it.

The application uses the ordinary Rust binary/Xcode application-target pattern
documented by the matching Bevy mobile example, not a new engine or emulator:

- https://github.com/bevyengine/bevy/tree/v0.19.1/examples/mobile
- https://github.com/bevyengine/bevy/blob/v0.19.1/examples/mobile/build_rust_deps.sh
- https://doc.rust-lang.org/rustc/platform-support/apple-ios.html

The new ios.rs module is compiled only on iOS. Desktop retains disc_source::startup,
CLI handling and window behavior. iOS uses the same extraction checks without
exiting immediately when data is missing; it opens a Bevy instructions screen.
The gameplay, graphics plugin chain, WGSL shaders, audio mixer, data decoder and
existing input/controller code remain unchanged. There is no added touch-control
overlay. Controller operation on iPhone is not yet proven.

## Native app build

The Xcode target is ReRAC, SDK iphoneos, device family iPhone, architecture arm64.
The Rust target is strictly aarch64-apple-ios, never simulator or Catalyst.
The chosen proof-of-concept deployment target is iOS 16.0; this does not claim
that every device on iOS 16 supports the renderer's features or memory needs.

The workflow .github/workflows/build-ios-arm64.yml runs on xcode-27, which GitHub
lists as an ARM64 hosted runner. It first compiles the desktop runtime and checks
its version CLI, then compiles the physical-device binary, builds the unsigned
Xcode app with bundled upstream shaders and a compiled launch storyboard, verifies
the actual Mach-O platform/dependencies, and produces:

  ReRAC-iOS-arm64-unsigned.ipa
  ReRAC-iOS-arm64-unsigned.ipa.sha256

It records exact source HEAD, baseline diff, runner/SDK/toolchain information,
Cargo and Xcode logs, and bundle/IPA verification in a separate evidence artifact.
Build failures also upload available logs. It does not upload Documents, extracted
data, ISOs, signing credentials or game assets.

Code signing is disabled in Xcode and the device linker. The verifier rejects a
code signature, embedded provisioning/signature payloads, wrong CPU/platform,
non-system dynamic dependencies and extra payload files. System UIKit/Metal/audio
frameworks are linked from iOS, not copied out of an SDK into the IPA. No unbundled
Bevy dynamic library is permitted.

An unsigned IPA cannot be installed directly on an ordinary iPhone. Legitimate
device signing/provisioning is a separate user-owned step; no Apple signing
credentials are requested or embedded in this workflow. Re-signing changes the
artifact checksum.

Runner reference:
https://github.com/actions/runner-images/blob/main/images/macos/xcode-27-arm64-Readme.md

## Loading your own extracted game data

1. Sign/provision the built app legitimately and install it.
2. Launch once. With no data it should show transfer instructions, create
   Documents/ReRACData, and make its Documents folder available for file sharing.
   This screen itself still requires a functioning Metal device/Bevy startup.
3. Close ReRAC. Using Finder Files sharing or the iPhone Files app, copy the
   CONTENTS of your existing rerac-extract output into:
   On My iPhone → ReRAC → ReRACData.
4. Ensure toc.bin and extract-info.json are directly inside ReRACData alongside
   levels/, global/ and all other extracted directories. Do not add an extra
   nested extracted/ directory. Complete transfer before launching the game.
5. Relaunch. The engine performs its normal strict data_format check and reads
   only persistent sandbox data, never an ISO in the app bundle.

Upstream currently documents user-owned NTSC-U SCUS_971.99 input. Extract it on
your desktop using the existing extractor/launcher. Do not upload it to GitHub.
The existing lazy cache can be built under ReRACData/cache; a prepared desktop
cache may also be transferred, subject to upstream validation/rebuild rules.
Saves and settings live in Documents/ReRACSupport. Keep the app closed while
changing shared files; live external file coordination/import is not implemented.
No cloud-download, on-device ISO extraction or runtime network access is added.

File sharing follows Apple's documented keys:
https://developer.apple.com/documentation/fileprovider

## Publishing and hosted execution

The connection was reauthenticated and the isolated ios/arm64-poc branch was
created successfully. Implementation changes are published only on that branch,
preserving main and release.yml. No tokens are requested or recorded.
The current connector does not expose workflow dispatch, so a narrowly scoped
push trigger for ios/arm64-poc starts the build when its implementation changes.
Manual workflow_dispatch remains available after GitHub registers the workflow.
Record actual publication and execution evidence separately from this handoff.

## Evidence boundary

Local tests are synthetic Mach-O/IPA fixtures and source/configuration contracts.
They are not a real device build. Plist, project and YAML parsing is not Xcode
compilation. Rust and Cargo are unavailable locally; no local engine compilation
was attempted. GitHub compilation has not yet run at source-handoff time.

Outstanding validation includes Rust dependency cross-compilation, Xcode 27
packaging/linking and UIKit lifecycle, Metal shader/feature/limit support, memory,
asset loading, audio/background behavior, controller input, data transfer,
physical installation, graphics and gameplay. No such result is claimed.

## Resume sequence

Publish isolated development branch → inspect branch-triggered Actions build →
inspect real compiler/build logs → fix actual failures → verify downloadable IPA
and checksum → legitimately sign/install → test on a real iPhone with user-owned
data. No platform-engine or DevStudio changes are part of this work.
