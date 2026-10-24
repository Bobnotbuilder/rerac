# Build 9 device-gap continuation — incomplete Aranos

Build 9 remains **INCOMPLETE**. The supplied iPhone screenshot supports authentic Ratchet/Aranos rendering; the user reports walking and introductory audio. The image does not establish an exact executable revision, visual parity, full gameplay or completion. The original image, directive and diagnostics remain private.

## Source and preservation

Starting local handoff: `2b440458b91479c915e0d9d29556c44f3ed9eb35`; public handoff: `16b219566e221360341472ba1c275edb1d2c654a`; prior packaged runtime: `13ba7879b59985de9cd19850352627eae08d97de`.

Current runtime local: `6f460ee242b44f93227b2e8f43d7df2a7519b33e`; public: `79be9c3db6e5383b1efa3264c72fe067c96dff85`. Branch `ios/rc2-upstream-integration`; upstream pin `89c87e98f9d8296187df2a6310c67b32144090d3` retained. A relevant upstream-history check found no newer rc-game2 weapon/enemy implementation to adopt. No wholesale merge or reset.

Rollback `rollback/build9-physical-render-audio-20261024` preserves the device-tested baseline alongside the existing Build 8 rollback. Existing private 96 original archives and converted Build 8 assets were reused without re-extraction. RAC1 data, saved games, identity and gameplay path remain separate. Production RC2 Play remains disabled.

## Implemented changes

| Source / primary classification | Change and limits |
|---|---|
| `rc-game2/src/collision.rs`; GAME_SPECIFIC | Original rigid class collision meshes registered through existing MobyGrid, transform/bounds builder and collision kernels. No render-mesh colliders or added floor planes. Only unconditional, unrooted triangle placements without posed primitives or kept matrices are registered: 30 placements on original Aranos. Omitted placements include noncolliding/unsupported records; this is not a completeness percentage. |
| `rc-game2/src/hero/physics.rs`, `camera.rs`; GAME_SPECIFIC | Hero line/capsule and camera line/sphere queries receive the original placement scene. Physics constants and shared kernels retained. Dynamic platform carry, class initialization, conditional spawns and posed collision remain missing. |
| Engine `rac2_level`, `level_load`, `title_world`, `rac2_play`; GAME_SPECIFIC | Load/route the GC collision scene; RAC1 and its title world retain None and their existing scheduler scene. |
| Engine `ios_gamepad`, `rac2_play`, `ios`; SHARED_DIAGNOSTICS | RC2-only raw button edges, native pad/action-state edges, unsupported-state reporting, opt-in per-tick collision trace with source identity. Input mappings are unchanged. This adds diagnostics, not weapons or new hero actions. |
| Formats `rac2/front_end.rs`, `rac2/boot.rs`; GAME_SPECIFIC | Correct frontend part 1 from the old GS label to its actual native sound bank; read 20 original boot sound definitions. Code evidence: boot 0x26f0ac loads part 1, 0x26f264 binds definitions at 0x238c00, navigation 0x348d24 requests 3 through 0x2e6c90. |
| `rc-game2/src/front_audio.rs`, engine `rac2.rs`; GAME_SPECIFIC | Navigation request 3 maps to original bank sound 2 and existing native 989snd/SPU rendering. Mixed into existing PCM output, retaining movie PCM. No RAC1 EE/gameplay tick, invented beep or level music is added. GC gain/options and device output remain unverified. |
| `rc-game2/tests/original_aranos.rs`; SHARED_TESTING | Reproducible original-data floor/body/jump/camera and menu-sound tests. Private media required; CI reports NOT_TESTED without it. |

## Tests actually executed

- Local CPU library compilation: PASS. Private harness product source is kept identical to product formats/game/game2 source; it is not used to build IPAs.
- RC2 unit tests: 11 pass, including transformed placement queries, ignore filtering and omitted unsupported placements.
- Local original-data integration: 4 pass, 5.61 s. Existing character/2187 stored poses, environment/frontend, movement/jump/collision and camera tests retained.
- New original raised floor: at `(247,198)` world-only ground `49.734375`; original placement 164/class `0xac1` supplies `50.33382`. Native hero lands there, jumps to `51.38469`, returns grounded; native follow camera updates against the placement scene. Exact screenshot location and PS2 behavioral oracle remain unverified.
- Original menu sound: request 3/definition index 2, 135-bank-sound resource, 48,000 rendered stereo frames, 3,480 nonzero, zero unsupported grains. CPU waveform evidence, not speaker or gain-parity proof.
- iOS source/package/controller/display contracts: 59 pass.
- Final hosted RAC1/native regression cases: rc-game 715, rc-formats 175 (1 ignored), rc-data 16, rc-engine 83 (1 ignored); all pass. The workflow repeats common cases in multiple tiers; these counts are not additive unique coverage. No physical RAC1 playthrough was run this session.
- Collision-only native run [40066408738](https://github.com/Bobnotbuilder/rerac/actions/runs/40066408738): ALL JOBS PASS, including ARM64/Intel builds, package/symbol verification, headless regressions, synthetic Metal and iOS 27 simulator probes. This source does not contain the new menu sound adapter.
- Combined runtime [40067825252](https://github.com/Bobnotbuilder/rerac/actions/runs/40067825252): ALL JOBS PASS: native ARM64/Intel compilation, verified IPA/dSYM, RAC1/engine/UI headless regressions, synthetic Metal, and iOS 27 simulator startup/input/display probes. No original data on CI. Current candidate physical iPhone/Backbone, corrected floor, menu sound and complete-level tests: NOT_TESTED.

No current physical frame-time/memory/stability measurements were made. Supplied baseline screenshot observations (~59.91 FPS, 11.48 ms GPU, 582.5 MB app, 241.35 MB Metal, Low Power Mode on) are one observed scene, not a full-level benchmark. Game Mode/encoding warnings were not treated as causes of missing gameplay; no Metal migration was attempted.

## Delivery

[Experimental unsigned Build 9 IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/40067825252/artifacts/12550182516) / [matching debugging symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/40067825252/artifacts/12549843892).

Inner IPA SHA256 `b14ba41f60c6cb8d983f1ccd512abfefd64abbe5d98bd8cd111cd43d16f37480`; ARM64 executable/dSYM UUID `09cce03d-595a-393f-93d5-d179d7a265e0`. Runtime source `79be9c3db6e5383b1efa3264c72fe067c96dff85`.

Same bundle `org.rerac.game.iospoc`, version 0.3.4/build 9, minimum iOS 16. Update with the established Sideloadly signing identity, preserving installed data; do not uninstall. The existing `Documents/RC2Native` data folder and native opt-in marker stay. No game assets are bundled or published.

Optional short reproduction trace: copy `ENABLE_COLLISION_TRACE.txt` containing exactly `RC2_COLLISION_INPUT_TRACE_V1` plus newline into `Documents/RC2Native` while closed. Reproduce walking/jumping/raised-floor contact and action-button presses, close the app and preserve `Documents/ReRACSupport/startup.log` and `startup.previous.log`. Remove the marker for normal testing. Per-tick file I/O affects performance; the trace is not a benchmark. Raw controller slot bits and native pad/state events are separate evidence. Each current collision trace record includes the source revision.

## Remaining blockers and complete-level acceptance

FIRST_FATAL_BLOCKER: none observed in executed CPU tests. First device validation gate: retest the exact raised surface and physical jump/actions, then navigation audio on the new source, with coordinates and source-linked logs. The managed host still sees only the Mac through xctrace; GitHub download DNS fails locally. User-provided baseline rendering/audio evidence is accepted and is not erased by these host limitations.

First remaining gameplay development blocker: native GC class initialization/scheduling, weapon/item/damage state and original mission progression are not reconstructed. The rigid collision adapter does not implement moving objects or their scripts. Existing RAC1 class handlers cannot be enabled with GC pvars as a substitute.

Known missing capabilities remain: additional posed/dynamic collision and platform carry; required Backbone attack/weapon/select/pause/context actions; original weapon resources and behavior, enemy AI/damage/health, crates/bolts/pickups/doors/interactions; authentic HUD, level music/ambience/voice/cutscenes; original save-slot flow, scripts/checkpoints/death/respawn and legitimate completion/transition. No start-to-finish Aranos route or physical playthrough is demonstrated.

Static original-code follow-up identifies native dispatch targets for unported states 0x13/0x14/0x15: `0x2bddd8`/`0x2bd378`/`0x2bd98c` in level 0, from the `0x2a0160` physics table. State 0x13 indexes records at `0x25d910`, stride 0x2c. The first records differ from RAC1 COMBO timing values, confirming that unchanged RAC1 melee is not a validated adapter. This is analysis evidence; no melee implementation or execution is claimed.

Exact next objective: validate these collision/audio/input changes at the reported device location, then reconstruct the original GC moby/class/hero-item initialization, state 0x13 entry/physics/transitions and wrench hit/damage callbacks against its actual table, reusing the existing engine kernels. Continue through the actual level scripts and completion; acceptance stays open until the full original route is demonstrated on iPhone/Backbone.

No Shared Core component/version/adoption changed. Installed PS2 engine pointer and other engines remain unchanged; no other engine adoption/review is claimed. RAC1 shared-kernel/code regressions must pass on the final source; physical RAC1 playthrough remains untested here.
