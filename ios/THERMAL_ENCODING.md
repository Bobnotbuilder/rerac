# iPhone build 5 — encoding/resource efficiency and measurement

Baseline inspected: ios/arm64-poc, 80d2fb1093429be32d0e1ff09ca828163be015cf
(0.3.4 build 4). Retains native drawable Hor+, safe-area controls, final-display UI,
movies' intended aspect, persistent game files/memcard, native Backbone input,
60 Hz logic, FIFO/display-aligned pacing, inactive clock/audio handling and bounded
local diagnostics. No engine/gameplay/progression change or Shared Core promotion.

## Evidence and what it does NOT establish

The user reports a 210 s A15 recording: Nominal through about 66 s, Fair by 68 s,
Serious by 102 s; predominantly 60 FPS with dips; frequent GPU time 9–11 ms;
85.61% encoding insight; a sample command-buffer CPU 14.52 ms / blit CPU 14.26 ms;
Game Mode Off. This is user-reported recording evidence, not independently parsed
frames. The recording's exact build/settings and battery/memory measurements are
not supplied here. Never add overlapping CPU intervals or interpret these as GPU
execution times, watts, temperatures or proven active CPU cost.

Apple defines encoder CPU time as encoder allocation to endEncoding:
https://developer.apple.com/documentation/xcode/understanding-metal-performance-hud-metrics
In pinned wgpu v29.0.4, queue.rs PendingWrites accumulates uploads until pre_submit;
metal/command.rs enter_blit retains the encoder until leave_blit/end_encoding.
An upload encoder's lifetime can therefore span other CPU work. This source fact
explains why the warning alone cannot establish the dominant CPU bottleneck.

Pinned sources inspected (research, not vendored code):
- https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-core/src/device/queue.rs
- https://github.com/gfx-rs/wgpu/blob/v29.0.4/wgpu-hal/src/metal/command.rs
- https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/src/renderer/render_context.rs
- https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/src/renderer/mod.rs
- https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_render/src/storage.rs

## Source-confirmed work and changes

1. Bevy flushes RenderContext's encoder at each render-system boundary. Original
   AA has four consecutive systems. iOS now encodes those same four shader draws,
   render passes, parameters, targets and order in one context: four AA command
   buffers become one when all pipelines are ready. No filters/effects disabled,
   merged mathematically or replaced. Desktop uses its original four-system path.
   For an isolated developer A/B, ios_batch_aa = off in the existing settings file
   restores the unbatched path on the next launch; graphics remain identical.
2. Post-process groups are now cached by the unordered ping-pong texture pair AND
   uniform-buffer identity. A role swap no longer rebuilds both groups, but resize
   or buffer replacement still invalidates them. Pass labels identify actual types.
3. Shadow vertex buffers previously recreated every nonempty frame; view uniform
   and both groups recreated inside every shadow pass. Vertex capacity now grows
   only when necessary, byte scratch storage is reused, view uniforms persist,
   unchanged matrices skip uploads and groups rebuild on texture/sample changes.
   Both original shadow passes, triangle order, draw count, shader and blend/depth
   rules remain. This trades recurring allocation for queue uploads; net physical
   cost must be measured, not assumed. Retained capacity can increase steady memory.
4. Existing Bevy ShaderBuffer already reuses compatible GPU allocations; it is NOT
   replaced. Dynamic palette/LOD/water buffers and changing images still upload.
   Movies' video updates and menu snapshots are functional work, not blindly removed.
   The four AA draws are render passes, NOT Metal blit encoders. Screenshot/readback
   tail submission in Bevy is left intact; no dependency fork merely to erase a HUD
   number. Shadows, AA, fog, mirrors and gameplay effects remain enabled.

No change to task pools/audio mixer: their contribution is not established.
No GPU waits, timestamp query readbacks, private Metal hooks or per-draw logging.

## Resolution and thermal policy

Port Options -> Resolution adds 540p, 576p and 720p fixed. Existing 416p is fixed.
720p auto keeps nominal 720, reducing after 10 s continuous Fair to 576, Serious
to 540, Critical to 416. Recovery requires 60 continuous seconds of lower pressure;
short fluctuations reset recovery. Only auto mode adapts. Fixed 720 bypasses it,
including under pressure, so stop a measurement if the device becomes uncomfortable.
No mode changes 60 Hz physics/game timing or intentionally lowers displayed FPS.
Native aspect and safe final UI are independent of world resolution.

## Local bounded instrumentation

Existing ReRACSupport/startup.log rotates at its existing limit. Every ~30 s:
- IOS_PERFORMANCE_SAMPLE: app cadence/interval mean/p50/p95, main-schedule wall,
  physical display/world dimensions, settings, self footprint, battery fraction,
  thermal state and inactive updates (existing implementation retained).
- IOS_RENDER_WORK: graph pending-buffer counts, custom post/shadow pass counts,
  group/allocation counts, shadow vertex upload bytes and mean wall time around
  post encoding, shadow preparation/encoding and prepare-to-submit interval.

Counters use fixed atomic slots; reports allocate/sort only at bounded intervals.
Scopes overlap and MUST NOT be summed. Graph pending buffers are NOT total Metal
command buffers: wgpu internal uploads and Bevy's tail submission are outside that
counter. Custom pass counts are NOT all GPU passes. GPU time and presented FPS
remain UNMEASURED in app logs; Metal HUD/Instruments supplies physical timings.
Battery fraction alone is not energy. No automatic upload or private game content.

## Metal 4 and Game Mode, reviewed separately

Apple lists A15 as Metal 4 capable on a supported OS:
https://developer.apple.com/metal/capabilities/
But this pinned wgpu Metal backend uses MTLCommandBuffer/MTLBlitCommandEncoder,
not MTL4CommandBuffer. No safe switch is exposed here. No backend rewrite, wgpu
upgrade or MetalFX integration is attempted for this pass.

iOS Game Mode eligibility is separate; Apple documents GCSupportsGameMode opt-in:
https://developer.apple.com/documentation/bundleresources/information-property-list/gcsupportsgamemode
https://developer.apple.com/videos/play/wwdc2024/10089/
The key is deliberately unchanged for the first controlled comparison. Off is not
proof of the thermal cause; enabling it is not an energy-savings claim. Record its
actual state and hold it constant between runs.

## Reproducible physical comparison (NOT yet performed)

Back up memcard; update without uninstalling using the same effective Sideloadly
identity/account. Do not change org.rerac.game.iospoc or game-data/save paths.

A. Source/encoding comparison: baseline build 4 vs build 5, fixed 720, same native
aspect/AA/shadows/textures/detail, save/planet/route/camera, brightness, audio,
Backbone, Game Mode, low-power mode, room/case and unplugged charger. Build 4's
auto 720 is comparable only while its actual resolution stays 720: exclude later
scaled segments or prepare a separately identified fixed-720 baseline build if a
long identical-resolution run is needed. Build 4 has no fixed-720 menu option.
Within build 5, ios_batch_aa = off/on isolates batching without changing graphics.
Do not compare unlike pixel counts as an encoding win.
Capture 210 s HUD reports, export logs and cool to comparable starting conditions.
If baseline settings cannot be established, label comparison uncontrolled.

B. Build 5 fixed-resolution sweep: 416p, 540p, 576p, 720p fixed, one separate cooled
run each, identical settings/route. Compare HUD CPU/GPU/frame-time distributions,
render/command-buffer counts, footprint, time to Fair/Serious and battery change.
Do not claim exact power from a short percentage change. Use longer matched runs
or physical power/Instruments tools when available. Fixed modes do not auto-scale.

C. Then test 720p auto separately for sustained quality/thermal behaviour. Do not
combine adaptive samples with fixed 720. Check no quality oscillation, stretched
geometry, clipped HUD or misaligned aim; test audio, combat, planets, background/
resume, save/relaunch/load and the previously working planet-three checkpoint.

| Metric | User baseline summary | Build 5 physical result |
|---|---|---|
| FPS | Mostly 60, occasional dips | NOT MEASURED |
| GPU frame time | Frequently 9–11 ms | NOT MEASURED |
| Encoder CPU interval | One CB 14.52 / blit 14.26 ms; overlapping | NOT MEASURED |
| Total Metal passes/command buffers | NOT PROVIDED | NOT MEASURED |
| Memory | NOT PROVIDED | NOT MEASURED |
| Fair / Serious time | ~68 / ~102 s from recording start | NOT MEASURED |
| Battery/energy | NOT PROVIDED | NOT MEASURED |

Build, pure/contract tests and procedural simulator rendering are distinct from
actual gameplay, sustained FPS, visual acceptance and physical energy results.
