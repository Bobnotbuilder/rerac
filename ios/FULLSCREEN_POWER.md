# iPhone native display and power pass — 0.3.4 build 4

## Preserve the physical checkpoint

Baseline branch commit: c9df823e939446e97b255af1a9d5ca28db874d79.
The user reports physical iPhone launch, Backbone menu navigation, gameplay and
progression to planet three. This is the accepted playable baseline, not an
automated-test result. Build 4 must be physically retested; no thermal/battery
improvement or new gameplay pass is claimed from source/build tests.

Bundle ID remains org.rerac.game.iospoc. Keep the same effective Sideloadly signing
identifier/account; install as an update, not by uninstalling. Documents/ReRACData,
Documents/ReRACSupport/memcard and existing game files are neither deleted nor
migrated. There is no DevStudio, GameStudio engine, Shared Core, dependency-version,
controller-mapping, extraction-format or progression-logic change.

## Source-level findings (not device energy measurements)

* display.rs defaulted to Aspect::Original and Resolution::Window: a 4:3
  framebuffer fitted into native physical Retina dimensions and centred on black.
  Width/height and several depth/copy consumers follow this GameFrame.
* The existing Hor+ view_tans / GameProjection path already supplies horizontal
  and vertical tangents to render culling/LOD, sky and scene/flight cameras. A
  native drawable ratio can extend that path without stretching a 4:3 image.
* frame_pace.rs has a 300 microsecond sleep/yield tail when a limiter is needed.
  At ~60 Hz it correctly trusts vsync; do not add a competing second 60 Hz clock.
* AA Original uses full-screen softening/copy passes. They remain enabled; reducing
  their target pixels reduces their theoretical pixel/bandwidth workload. No
  essential effect is disabled. Actual GPU time and energy have not been measured.
* Default Bevy WinitSettings requests unfocused updates at 60 Hz. This pass lowers
  the unfocused opportunity to 1 Hz and pauses the virtual clock and app-owned audio
  sinks on observed inactivity. UIKit/winit suspension and absence of background
  modes remain authoritative. Physical suspend/resume/audio verification is needed.
* AudioOut's mixer is a clocked 48 kHz buffered source, not a demonstrated busy loop.
  Its ring, SPU/reverb and sample order are untouched. Bevy's task-pool defaults are
  also untouched: excessive worker wakeups/allocations/overdraw are NOT established
  as device bottlenecks without profiling.

## Fullscreen and quality

iOS defaults to Native screen and a 720-line internal world target. Native aspect
is resolved from the actual Bevy drawable each resize/orientation change, not a
hard-coded phone model or 16:9. Only horizontal FoV widens; vertical FoV, near/far,
camera orientation and game input are unchanged. Existing 4:3/16:10/16:9 modes
remain selectable. Fullscreen iOS windows hide the status bar and prefer hiding the
home indicator; a controller does not need edge-touch gestures.

The final present camera targets the full physical display. HUD/menu/movie UI
composes there independently of the internal world resolution. Original HUD
textures retain their original pixel dimensions/graphics settings; no promise of
new high-resolution glyph assets is made. Menu/snapshot/canvas targets still
follow the shared framebuffer sizing/refresh machinery, including depth-target
refresh before camera updates. Movies retain a centred original 4:3 presentation.

Safe insets come from the active UIWindow's root view, normalised against its
bounds. Opposing insets are symmetrised, then HUD/menu controls are uniformly
inset about the screen centre. Full-screen fades, world-projected gun markers,
screen effects and in-world vendor surfaces are not shifted/scaled. Menu preview
canvases and frame views follow the safe control transform. This avoids moving
the aiming coordinate system to the notch's asymmetric safe-area centre.

Use Options -> Port Options -> Aspect ratio / Resolution to compare. New iOS-only
ios_aspect and ios_resolution keys preserve legacy desktop preferences in the
same existing settings file; saved-game data is not involved. 720p is nominal
Balanced quality, not a measured universal optimum. Choosing another fixed
height or Window is an explicit override. No lower-displayed-FPS profile is added.

At nominal 720p, serious OS thermal pressure reduces the world to 576 lines,
critical pressure to 480; recovery requires 30 continuous seconds at the lower
pressure. Only the nominal 720p selection adapts. All cases retain the 60 Hz game
clock and preferred 60 displayed FPS. Internal size changes use the existing
camera/texture resize synchronisation. No frame duplication/FPS spoofing is used.

On iOS the limiter uses full sleeps instead of its desktop yield-spin tail, keeps
display-aligned pacing, and uses explicit FIFO vsync. Desktop pacing is unchanged.
Fewer pixels/scheduling wakeups are source-level workload reductions, not evidence
that a particular phone now stays cool or lasts longer.

## Local observations and limits

Documents/ReRACSupport/startup.log remains bounded/rotating. Once per 30 seconds:
app-frame cadence (app_fps), interval mean/p50/p95, main-schedule wall duration,
drawable/internal dimensions, current display settings, thermal state, self memory
footprint and battery fraction. Thermal/layout/lifecycle changes are bounded events.
Battery monitoring is local; battery percentage is not an accurate energy meter.
No device model, identifiers, game data or logs are uploaded automatically.

app_fps is the application update cadence, NOT confirmed screen presentations.
main_schedule_wall_ms is NOT total CPU utilisation and excludes the limiter wait
and potentially pipelined render work. presented_fps and gpu_ms explicitly remain
UNMEASURED. No GPU timestamp queries/readback overhead is enabled in this initial
pass. Use Xcode Instruments Metal System Trace, Time Profiler and Energy/Power
tools on the physical signed build where accessible; hosted simulator results
cannot establish iPhone GPU cost, memory pressure, battery or temperature.

## Validation

The existing iPhone Actions workflow compiles the real device executable and
validates ARM64/platform/minimum OS, system frameworks, shaders, unchanged bundle
ID, unsigned IPA and UUID-matched dSYM. It runs source/package contracts and real
compiled pure Rust geometry/policy tests. The simulator probe also checks actual
GameProjection matrices/frusta and safe UI primitives before the preserved
GameController synthetic test, then renders the no-data instructions through the
display pipeline. A bounded installer retry addresses the prior install timeout;
failures remain failures and their first startup log is retained.

A separate hosted headless engine/UI regression job uses the repository's
documented cargo xtask tier, not a desktop game launch or Intel release build.
Data-dependent checks may skip without extracted input. No copyrighted assets
are present on CI. None of these is another physical third-planet playthrough.

## iPhone comparison checklist

1. Back up ReRACSupport/memcard through Files before updating. Install build 4 as
   an update with the same effective signed bundle ID; verify existing saves/data.
2. Attach Backbone. Load the existing save and recheck Menu/Start, confirm/cancel,
   movement, camera, aim/markers, jump, weapons/combat and audio.
3. Check Native screen at both landscape orientations. World should reach both
   screen edges without squeezing Ratchet; HUD/menu text, preview widgets and
   bottom hints should stay inside safe edges. Movies should not be stretched.
4. Travel among the previously working planets, save, relaunch and load again.
   Check wide-view sky/fog/effects/culling for missing peripheral objects.
5. Background/lock for a minute, verify silence/no gameplay advancement, return
   and verify audio/controller responsiveness and no catch-up burst.
6. Compare the same save/route/camera for 15 minutes against build 3: same room
   temperature, brightness, audio level, power mode, case and disconnected charger.
   Record initial/final battery, subjective/device temperature and startup.log.
   Allow cooling between runs. Report quality changes/thermal resolution shifts.
7. Measure physical frame pacing/GPU/CPU with Instruments when possible. Do not
   infer energy savings solely from battery percentage or simulator FPS.

Classification candidates: SHARED_IOS / SHARED_METAL / SHARED_TESTING, scoped to
this ReRAC port. Nothing is automatically promoted into GameStudio Shared Core.
