# Device-launch repair from the first physical iPhone crash

Baseline: ReRAC iOS commit 9b48bbd0f1ce66685dcc9e368a109830f7904fde.
The six supplied device reports consistently terminate with EXC_BREAKPOINT /
SIGTRAP in UIKit's UIApplicationEvaluateRuntimeIssueForNoSceneLifecycleAdoption.
They contain ReRAC's show_import_instructions frame underneath UIApplicationMain.
This proves the error/import branch was entered; the UIKit lifecycle assertion,
not an unhandled missing-data exit, is the first recorded fatal failure.
The exact data validation error cannot be recovered from these reports alone.

The reports do not show a dyld missing-library, code-signature or jetsam
termination. They do not prove every signing entitlement, GPU path or asset
is correct. No private crash report is committed or uploaded by the workflow.

Apple documents scene lifecycle adoption as mandatory for apps built with the
iOS 27 SDK:
https://developer.apple.com/documentation/uikit/transitioning-to-the-uikit-scene-based-life-cycle

## Narrow dependency change

Bevy remains 0.19.1; winit remains 0.30.13. Pin the unmerged iOS scene backport:

https://github.com/molenick/winit/commit/94d6a23c425023810a0879aa7a03abb2e4f1decc

Referenced by upstream:
https://github.com/rust-windowing/winit/pull/4723

Reviewed relative to version commit e9809ef54b18499bb4f2cac945719ecc2a61061b:
one commit changes Cargo.toml (iOS UIKit feature selection),
src/platform_impl/ios/app_state.rs (OS capability),
src/platform_impl/ios/window.rs (connect a new UIWindow to UIWindowScene),
and its changelog. Non-iOS window implementations are unchanged.
Keep the already locked released dpi 0.1.2 via a source override rather than
silently taking the git repository's older path copy.

The dependency's Apache-2.0 LICENSE was reviewed and is retained verbatim inside
ThirdPartyNotices.txt in the application. No fork code is copied into ReRAC's
gameplay. The patch is not claimed merged or proven on the user's device.

Add one UIWindowScene configuration, explicitly disable multiple scenes and
keep winit's UIApplicationMain entry. There is no SDK spoof, assertion swizzle,
older-SDK workaround, renderer rewrite or replacement game loop.

## Diagnostics and data handling

Documents/ReRACSupport/startup.log records bounded stages: entry/source version,
sandbox preparation, existence of toc.bin, extract-info.json and directories,
bundled shader directory, data validation, import/game app run, Bevy Startup,
render-device availability, first update and lifecycle changes. It records Rust
panics locally and keeps the prior crash hook. No environment dump or game
contents are captured. Logs are never automatically uploaded from the device.

Missing/invalid extraction metadata keeps the import screen path. A boot-level
load error now also opens that path on iOS instead of returning an error out of
main. The desktop Result/CLI behavior remains unchanged.

## Evidence to collect

The device Rust profile retains debug information and a packed dSYM. Packaging
must compare the executable and DWARF UUIDs and require actual debug sections.
Retain the symbol artifact for 90 days and download it for long-term storage.
The original build did not retain a matching dSYM; a new build's symbols cannot
be used to claim source-line symbolication of the old executable.

The same workflow builds the real app for the iOS 27 simulator as an additional
diagnostic, launches it without game data, waits for the first update, confirms
the process survives and saves startup logs and a screenshot. The simulator
binary is never the device IPA input. This is not physical-device or gameplay
acceptance; failures remain visible in Actions and evidence.

After the corrected unsigned device build: sign/install, launch without data,
confirm a visible import screen, inspect startup.log, transfer your own extracted
data while stopped, then relaunch and capture the next genuine blocker.
No successful physical retest, graphics, audio or gameplay is yet claimed.
