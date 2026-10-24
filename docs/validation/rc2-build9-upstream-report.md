# Build 9 pinned upstream integration — incomplete Aranos

Build 9 is **INCOMPLETE**. This experimental native player candidate does not satisfy the complete-level acceptance target. No start-to-finish Aranos playthrough or current physical-iPhone gameplay is demonstrated.

## Experimental movie PCM continuation — compiled and CPU-tested

Continuing from validated local `aac9ce74ffa79a790999bc8c7ce5cc61b86c656c` / public `85104ee8b6782cf001647a635aebf59d250706b6`, local `20eba01c7468f5326e108e0ca9d0674f97f7de46` / public `13ba7879b59985de9cd19850352627eae08d97de` connect original frontend movie audio to the existing native Ring/MixerStream/Bevy transport. Worker audio moves to the frontend before the first decoded picture; 800 original stereo samples are submitted per 60 Hz tick. Movie change/skip and NewGame flush queued sound. RAC1 AudioOut and gameplay/saves retain their existing path. RC_AUDIO=0 disables this output.

This is experimental decoded-PCM playback with unity gain: original GC SPU gain/options and audiovisual latency parity remain unverified. Speaker output has not run. This does not add menu SFX/music, level audio, dialogue scripts or playable level progression.

Locally, 9 native video unit tests pass; original ISO test passes with every sample of all ten original logo/title entries covered without channel changes, gaps or repetitions, padding only after audio ends (11.49s). iOS source/package/controller/display contracts: 59 pass. The three new engine transport tests passed within 83 engine unit passes (1 ignored): stereo conversion, bounded-buffer backpressure/underrun, flush/re-prime and worker PCM/picture tick handoff. Native ARM64/Intel compilation, package/symbol checks, headless/synthetic Metal and iOS 27 simulator validation all PASS in [run 40042654037](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037). The IPA below matches this runtime. Neither native speaker output nor original RC2 graphics/device gameplay was exercised.

## Source and preservation

| Checkpoint | Revision |
|---|---|
| Starting local implementation | `e15499e95aa48be7f41fb468db918b61cc58fa29` |
| Starting published implementation | `8829f79d8c210d1294cdbb4731438314960a40b2` |
| Pre-integration rollback, including previously dirty handoffs | `0fb24c6a312360e594b15abc48ca52e77894d031` |
| Adopted official upstream | `89c87e98f9d8296187df2a6310c67b32144090d3` |
| Final local runtime source | `20eba01c7468f5326e108e0ca9d0674f97f7de46` |
| Final published runtime source | `13ba7879b59985de9cd19850352627eae08d97de` |

Integration branch: `ios/rc2-upstream-integration`. The existing `ios/rc2-native` published baseline was not overwritten. Preserved rollback branches include `rollback/build9-pre-upstream-20261024` and `rollback/build8-known-working`. All 1,220 local tracked blobs match the published runtime; upstream's optional logo and clusters table remain preserved remotely. No reset or history replacement was used.

Build 8's existing IPA remains unchanged: SHA256 `3331f73993855769f6ea850e4aa3eb6068ff7e0c8e752f047f4e36fc5669ee1c`. Its terrain/textures/Backbone free-camera physical success is user-reported evidence. Original assets and RAC1 saved games were not replaced or migrated.

## Implementation and reuse

| Source | Adopted capability |
|---|---|
| `crates/rc-game2/` | Official GC frontend, native hero state/physics/jump logic and follow camera; shared pad, collision queries and RatchetAnim retained. |
| `crates/rc-formats/src/rac2/` | Official GC TOC, level, armor, boot GUI/font and gameplay readers. |
| `crates/rc-formats/src/{level,moby,moby_anim,tie,shrub}.rs` | Game-tagged format adapters, GC compact packets, original compressed animation to shared native frame/pose structures. Previous diagnostic API wrappers remain. |
| `crates/rc-engine/src/{rac2,rac2_level,rac2_play}.rs` | Official frontend/NewGame/player plugins and level data through the existing shared LoadedLevel/renderers. No second terrain renderer. |
| `crates/rc-engine/src/{main,disc_source,ios,rc2_native_data}.rs` | Isolated experimental native data route, strict original build identity, separate settings, RC2 saving disabled; RAC1 scheduler excluded from RC2. |
| `crates/rc-engine/src/{moby_render,moby_anim,tie_render,shrub_render,occlusion,level_load,level_switch,saves,hud_render}.rs` | Selective GC data dispatch and upstream generic HUD layer; existing iOS display/AA/shadow/Metal/controller/lifecycle policies preserved. |
| `crates/rc-game2/tests/original_aranos.rs`, `crates/rc-video/tests/rac2_original_frontend.rs` | Read-only original-data CPU validation. |
| `Cargo.toml`, `Cargo.lock`, iOS project/plist/tests, CI workflow | Added rc-game2 without third-party dependency upgrades; retained version0.3.4, bundle identity, winit iOS patch; build counter9; isolated branch validation/artifacts. |

Movie PCM source changes: `crates/rc-engine/src/audio_out.rs`, `crates/rc-engine/src/rac2.rs`, `crates/rc-video/src/movie.rs`, `crates/rc-video/tests/rac2_original_frontend.rs`, `.github/workflows/build-ios-arm64.yml`. The existing decoder and native backend are reused; no level sound bank is fabricated.

The [per-system matrix](rc2-upstream-integration.json) separates reused transport/rendering/evaluation from GC-specific hero/camera implementations already supplied by upstream and GC systems still absent. No compatibility percentage is claimed. Unrelated upstream RAC1 fixes were not wholesale merged.

## Validation actually executed

| Validation | Result |
|---|---|
| Local offline rc-formats/rc-game/rc-game2/rc-data libraries | Compiled; harness source files byte-identical to product source. |
| Formats with available original RAC1 data | 175 passed, 0 failed, 1 ignored. |
| Data / RAC1 game / upstream game2 units | 16 / 715 / 9 passed; original RAC1 root supplied to local data-dependent tests. |
| RAC1 original populated moby reader comparison | 2,968 identical outputs across levels 0–18; level 19 data unavailable/not tested. |
| iOS source/package/controller/display contracts | 59 passed. |
| Original RC2 environment/frontend and character/player integration | 2 passed: 833 fragments/all three triangle LODs,351 texture entries,74 tie classes/2,000 instances,187 moby classes/472 placements; original menu tables/230 sprites/NewGame request. |
| Original Ratchet CPU resources and poses | Four armor textures,111 joints; all 2,187 stored frames evaluated finite poses; preserved earlier storage contract also passed. Visual pose parity not established. |
| Original GC hero and camera CPU path | Scripted pad moved0.32995287 game units; jump rose1.0508652 units and returned to original collision (59 grounded ticks observed); native follow camera updated. No hardware-controller claim. |
| Original frontend video decoder | First three 512×416 frames of each logo/title movie0xb2–0xbb decoded through existing rc-video; PCM decoded and wired through the experimental shared transport; speaker output and GC gain/A/V parity not tested. |
| Hosted final runtime native Intel + ARM64 compilation/package/symbol checks | PASS. |
| Hosted final runtime headless + synthetic Metal fixture | PASS; fixture depth[0,255,0],order[255,255,0]. No original media on CI. |
| Hosted final runtime iOS 27 simulator | PASS: startup/import, synthetic controller, projection/safe UI/resize and batched/unbatched AA/shadow GPU probes; no original game data. |

[Final runtime workflow](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037). Private original ISO tests ran locally; CI explicitly prints NOT_TESTED for these cases when private media is absent. CI success must not be treated as original-data graphics/gameplay evidence. CPU and simulator synthetic results are separate from physical hardware and full-level acceptance.

## Delivery

[Build 9 experimental unsigned ARM64 IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037/artifacts/12539763258) and [matching symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037/artifacts/12540266378).

Runtime source: `13ba7879b59985de9cd19850352627eae08d97de`. Version 0.3.4/build 9; bundle `org.rerac.game.iospoc`; minimum iOS 16. Signing unavailable: use the established Sideloadly workflow with the same effective identity and install as an update, preserving data. Do not uninstall RAC1.

Inner IPA SHA256: `900d8745c08e74e3303f31ea13361507e378bfe0aa08b507c872eb7af7b127d1`.
Matching executable/dSYM ARM64 UUID: `f7ca22fb-d431-3ac8-b4f1-96451d849282`.
Outer artifact ZIP SHA256 (different object): `5aa4c3a0c7ed300e25c57fc81a22e797a1b16c517e12e93f7bdb0f7ed5e56b8e`.

The pinned upstream extractor prepared 96 original Tier0 archives privately (3,807,638,923 bytes), validating its size/SHA1 table. Build8 converted mesh/texture/manifests were not regenerated. Native RC2 opts in through Documents/RC2Native, containing the original archives, manifest and ENABLE_NATIVE_RC2.txt with exactly `RC2_NATIVE_ARANOS_EXPERIMENTAL_V1` plus newline. Settings are separate; RC2 saving is disabled. Removing this marker restores the previous startup selection (Build8 preview if its own marker remains, otherwise RAC1). The production RC2 Play gate remains disabled.

## Current evidence boundary and first blocker

- Original Aranos on the new shared gameplay renderer: **not GPU-tested**. Build8 physical preview success is preserved user evidence.
- Original Ratchet rendered/animated on GPU: **not demonstrated**; original mesh/skeleton/poses are CPU-tested.
- Current RC2 Backbone movement/jump/camera/collision on iPhone: **not tested**. CPU scripted movement/jump/landing passed; simulator controller transport passed without original game data.
- Original checkpoints, death/recovery, legitimate completion or start-to-finish level route: **not demonstrated or implemented by this integration**.
- Current physical iPhone execution and frame-time/memory/stability/performance measurements: **not available**.

FIRST_FATAL_BLOCKER: none in executed tests; original-data native GPU startup has not run. The first genuine execution blocker is environmental: this managed host cannot download the compiled native executable because GitHub/artifact-host DNS fails, and no physical iPhone is accessible. A browser-downloaded/extracted native executable's local path was requested; no compatible executable is present locally. Do not substitute synthetic images for the missing original render.

Known missing capabilities remain separate: GC lighting/fog/material visual parity; required weapons/attacks, enemy AI/damage/health, interactive/destructible objects and dynamic platforms; original HUD, level audio/dialogue/cutscenes; scripted objectives, checkpoints, death/respawn, legitimate completion/transition; complete route and physical phone playthrough/regressions. The original save-slot menu is absent upstream; NewGame enters the level directly.

## Governance and exact next objective

Fix classifications: GAME_SPECIFIC, PLATFORM_SPECIFIC, SHARED_TESTING. No Shared Core promotion, installed engine switch, legacy PS2 replacement or other-platform adoption. Other engines require no review for these isolated GC/iOS changes; reusable promotion remains subject to its usual governance. RAC1 physical regression on this exact candidate remains untested despite passing CPU/source/synthetic regressions.

Next: execute the checksum/source-verified native artifact against the prepared private RC2 archives; capture and visually inspect authentic Ratchet/Aranos through the shared renderer; validate the same NewGame/move/jump/follow-camera/grounded-collision path on the target iPhone/Backbone. Then implement GC gameplay, audio/HUD, scripts, checkpoints and completion in dependency order. Build9 stays incomplete until a genuine original start-to-finish iPhone playthrough satisfies acceptance.

