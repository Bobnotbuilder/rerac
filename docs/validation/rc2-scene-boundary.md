# RC2 Aranos experimental preview — original GPU frame pending

## Build 9 movie PCM continuation — latest validated runtime

Build9 remains **INCOMPLETE**. Runtime local `20eba01c7468f5326e108e0ca9d0674f97f7de46` / public `13ba7879b59985de9cd19850352627eae08d97de` on `ios/rc2-upstream-integration`, adopting upstream89c87e. [Run40042654037](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037) passed native ARM64/Intel compilation, packaging/symbols, headless and synthetic Metal/iOS27 simulator checks. No original data on CI. Local original ISO tests cover all PCM samples of ten frontend movie entries; video units9pass, iOS contracts59pass, new engine transport cases3pass within83passes/1ignored. The RC2 frontend now reuses native Ring/MixerStream audio output; unity gain is experimental, GC gain/options/A-V latency and actual speaker output unverified. Menu sounds/music and in-level audio remain absent.

[Experimental unsigned IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037/artifacts/12539763258) / [matching symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/40042654037/artifacts/12540266378). Inner IPA SHA256 `900d8745c08e74e3303f31ea13361507e378bfe0aa08b507c872eb7af7b127d1`; ARM64 executable/dSYM UUID `f7ca22fb-d431-3ac8-b4f1-96451d849282`. Same bundle org.rerac.game.iospoc, version0.3.4/build9, iOS16minimum; existing update/signing identity and saves preserved. Production RC2 Play remains disabled, experimental RC2Native opt-in separate. Build8 rollback and private original/converted assets retained.

FIRST_FATAL_BLOCKER: none in executed tests; original-data native GPU startup untested. First execution blocker remains managed-host artifact-download DNS plus no accessible iPhone. Original Ratchet/Aranos graphics, current physical Backbone/player path, speaker output and full-level playthrough are NOT_TESTED. Known missing: GC visual parity, combat/enemies/interactions, original HUD/level audio/scripts, checkpoints/death/respawn and legitimate completion. No phone frame-time/memory/stability measurements. Next: run the source/checksum-verified native executable against private RC2 data, capture and inspect Ratchet/Aranos, then validate NewGame/move/jump/camera/collision/audio on iPhone. No Shared Core promotion or other-engine changes.


## Final Build 9 upstream integration receipt — 24 October 2026

Build9 remains **INCOMPLETE**. Final runtime local/public sources: `aac9ce74ffa79a790999bc8c7ce5cc61b86c656c` / `85104ee8b6782cf001647a635aebf59d250706b6`, selectively adopting official `89c87e98f9d8296187df2a6310c67b32144090d3` on `ios/rc2-upstream-integration`. Starting local/public source: `e15499e95aa48be7f41fb468db918b61cc58fa29` / `8829f79d8c210d1294cdbb4731438314960a40b2`; pre-integration rollback `0fb24c6a312360e594b15abc48ca52e77894d031` preserves prior dirty handoffs.

All final hosted jobs PASS in [run40015874520](https://github.com/Bobnotbuilder/rerac/actions/runs/40015874520): native Intel, iPhone ARM64 compilation/package/UUID-matched symbols, headless, synthetic Metal and iOS27 simulator startup/input/display/resize/batched-unbatched AA/shadow probes. CI contains no original game data. Local original-data CPU tests cover Ratchet's111 joints/all2187 poses, original Aranos geometry/frontend, native move/jump/grounded return/follow camera and original title-movie decoding. RAC1 formats175/data16/game715 and2968 identical populated class outputs across19 available levels pass; game2 units9 and iOS contracts59 pass.

[Experimental unsigned Build9 IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/40015874520/artifacts/12529371991) / [matching symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/40015874520/artifacts/12529826025). Inner IPA SHA256 `2cf9aae8cf517810248610075ba482618c598310024721eff655ef965bb3c0ef`; ARM64 UUID `b201b45d-11dc-3b46-9d48-41664b37cda5`. Version0.3.4/build9, same `org.rerac.game.iospoc`, minimum iOS16. Use the existing Sideloadly identity/update workflow; preserve RAC1 data/saves and do not uninstall.

Original Ratchet/Aranos through this new shared gameplay renderer: **not GPU-tested**. Current physical iPhone/Backbone gameplay: **not tested**. Movement/collision are CPU-tested; no game checkpoints or full-level completion are supported/claimed. Build8's user-reported physical terrain/free-camera success remains preserved. The first unavoidable original-data execution blocker is the managed host's failed artifact-download DNS and lack of accessible iPhone; the native executable is not local and a browser-download path request is pending. FIRST_FATAL_BLOCKER remains none in executed tests; no original-data GPU startup has run.

Next: obtain the source/checksum-verified native executable, capture authentic Ratchet/Aranos through the shared renderer, then validate the same NewGame/move/jump/follow-camera/grounded-collision path on iPhone. Combat/enemies, original HUD/audio, scripted events, checkpoints, death/respawn and legitimate completion remain missing. Production RC2 Play stays disabled; experimental Documents/RC2Native opt-in has isolated settings and saving disabled. No Shared Core promotion or other-engine changes.

[Complete session report](rc2-build9-upstream-report.md) and [per-system compatibility matrix](rc2-upstream-integration.json). Older entries below are historical and superseded where they describe the now-adopted compressed-pose/hero/camera work or pending compilation.


## Build 9 upstream integration — current, 24 October 2026

Build 9 remains **INCOMPLETE**. Pinned upstream `89c87e98f9d8296187df2a6310c67b32144090d3` is selectively adopted on `ios/rc2-upstream-integration`; it supersedes the earlier independent compressed-pose blocker. Pre-integration rollback: `0fb24c6a312360e594b15abc48ca52e77894d031`. Initial integrated local/public sources: `35a163e` / `a301e92ed077bc5d3066f133e4fb2a9e7fb97aa6`; follow-up validations and strict loader changes are still being finalized.

Original-data CPU evidence: 833 tfrags, 351 texture entries, 74 tie classes / 2,000 tie instances, 187 moby classes / 472 placements; authentic Ratchet armor/four textures/111 joints, all 2,187 pose frames finite; original frontend New Game request; native hero move 0.330 units, jump rise 1.051 units and landing on original collision; follow camera updated. This is not GPU or physical-controller evidence. RAC1: 715 game tests, 175 formats tests, 16 data tests, all 2,968 populated class outputs unchanged across 19 available levels. Upstream game2: 9 tests. iOS contracts: 59 tests.

ARM64/shared-engine GPU validation is pending in [run 40012155135](https://github.com/Bobnotbuilder/rerac/actions/runs/40012155135). No new original Ratchet render, physical iPhone gameplay or complete Aranos claim. Build 8's user-reported physical terrain/Backbone free-camera baseline and IPA remain preserved.

The pinned upstream extractor prepared 96 original archives (3,807,638,923 bytes) privately, validating every file against its size/SHA1 table. Existing converted Build8 payloads were not regenerated. Experimental native RC2 uses Documents/RC2Native + ENABLE_NATIVE_RC2.txt (`RC2_NATIVE_ARANOS_EXPERIMENTAL_V1` newline); RC2 saves disabled and settings isolated. The production RC2 Play gate and RAC1 data/saves remain unchanged.

First runtime fatal: none observed in CPU tests; GPU startup has not yet run. Next: complete native compilation, capture authentic Ratchet/Aranos through the shared renderer and test the route on iPhone. Combat/enemies, original HUD/audio, scripts, checkpoints/death/respawn and legitimate completion are still missing. No Shared Core promotion or other-engine changes. See [integration matrix](rc2-upstream-integration.md) and upstream [bring-up plan](../plan/rac2_bringup.md).


Current state: [Build9 full-engine compatibility](rc2-engine-compatibility.md).
User now reports original Build8 terrain and Backbone free camera on physical
iPhone. Historical GPU-unverified notes below predate that report. Original
Ratchet poses/gameplay/completion remain unimplemented and unvalidated.

## Build 8 scene-preview validation receipt — 2026-10-23

Runtime source `cd1311505774f52e87d006e6bb8cae9a0bbcc5ad` (local source
`d38272251c01b7004baed9e94e981371c5f7e7b9`). The isolated experimental
terrain renderer compiled for native Intel/macOS, ARM64/macOS, iPhone ARM64
and the iOS simulator. The new **synthetic** Metal probe captured a frame and
passed independent texture/colour, depth and submission-order pixel checks.
Original Aranos upload planning passed on all three preserved LODs, but an
**original Aranos GPU frame has not been captured**. No Ratchet model,
player movement/collision, RC2 gameplay or physical iPhone result is claimed.
RC2 Play remains disabled; normal RAC1 startup and data/save paths are retained.

[Run 39943428819](https://github.com/Bobnotbuilder/rerac/actions/runs/39943428819)
passed all compilation, regression, package/symbol and batched/unbatched iOS 27
simulator validation steps. The workflow conclusion is **failure** because the
final device-evidence upload timed out after five ArtifactService attempts.
The IPA and symbols had already uploaded successfully; full job logs retain
the validation results. No assertion was bypassed or reclassified as a pass.

[Build 8 unsigned scene-preview IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/39943428819/artifacts/12498465782)
and [matched symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/39943428819/artifacts/12498027423).
Inner IPA SHA256: `3331f73993855769f6ea850e4aa3eb6068ff7e0c8e752f047f4e36fc5669ee1c`.
UUID: `198ebb96-e8fc-3fd0-9537-18eedbcd20fa`; bundle `org.rerac.game.iospoc`,
version 0.3.4/build 8, minimum iOS 16, SDK 27.1, 25 source-matched shaders.
Sign with the existing effective identity and install as an update; preserve
RAC1 data/saves and do not uninstall. This is a scene-preview build, not gameplay.

[Native Intel validation ZIP](https://github.com/Bobnotbuilder/rerac/actions/runs/39943428819/artifacts/12497867130)
is compiled, but shell DNS prevents retrieving it into the local workspace.
Immediate next step: obtain that binary locally, run `RC2_SCENE_PREVIEW` against
the unchanged private Aranos package, capture and inspect an authentic frame.
Original GS/VU state remains uncorrelated; the diagnostic material/camera policy
is explicitly experimental. No private game assets were uploaded/regenerated.
Full scoped results: `docs/validation/rc2-scene-preview-build8.json`.

The implementation/checkpoint notes below predate this receipt. Pending compile
statements there are historical; original-game rendering/gameplay limits remain.

## Historical CPU scene-boundary evidence

Baseline application source: `4c94e5350fd28afed19633c51fa403d340758765`,
Build7. Latest baseline documentation commit:
`a974c687c3915b417d73913e1f1f7541c7e429ad`. Build7 remains the deliverable;
this candidate does **not** yet add an iOS scene, change shaders or enable Play.

## Implemented and executed

- `rc_data::rc2_scene::Terrain`: bounded borrowed reader for Build7 RC2MESH1,
  per-LOD ordered iteration, exact five A+D register destinations and texture
  index agreement, finite vertices/ST, original colour bytes. No global material
  sort, cross-LOD concatenation, invented defaults or RAC1 LoadedLevel.
- `rgba_image`: exact power-of-two dimensions and payload lengths, no substitutes.
- `rc_formats::rc2_tie`: independent GC header and packet-region reader, signed
  packed position decoding, class scale, dinky/fat UV-prefix distinction and
  QW padding; original records retained. No strip topology/GS slots, lighting,
  instance transforms, multipass or fake colours inferred.
- Owned local Aranos:833 fragments;33710/26522/26416 triangles by LOD;93
  materials/279 images accepted. All655 Build7 payload lengths/SHA1 and exact
  file set rechecked. Manifest SHA256 unchanged:
  `a71834e2191f8cc10a358b7ce967a8621d173b8ac97c0bfa5095639161b216ce`.
- GC ties:74/74 classes,1876 packets and93418 vertex records decoded without
  bounds/count failures. These are records across all LODs, not unique objects
  or proven renderable classes. No output assets regenerated.
- On-disc material K values have zero-extended low16 signed fields (observed
  signed range -137..-89, min-filter field4). A test checks0x0000ff77 -> -137.
  These are **fixup inputs**, not complete live TEX0/TEX1/CLAMP registers.

## Rendering parity / integration matrix

Follow-up static original-source check: the actual loadable payloads for terrain
candidate programs55907/903379 are byte-identical between the two verified boot
ELFs. See rc2-vu-upload-identity.json and tools/trace/examples/rc2_vu_identity.rs.
This reduces the microcode difference question; the CPU/GPU table below still
does NOT claim original execution, runtime program selection or identical EE
matrices, light/fog inputs and GS state. Custom DVP debug-section file offsets
were rejected as code evidence; only ovlytab EE addresses resolved through
PT_LOAD were compared. No original instruction bytes committed or uploaded.

| Feature | Implemented | CPU executed | GPU/device verified | Remaining requirement |
|---|---|---|---|---|
| Owned package lengths/hashes | Build7 converter/audit |655/655 |No |App importer/identity binding |
| Native terrain scene reader |Yes |833 fragments |No |Connect isolated native scene |
| One LOD, original triangle order |Yes |33710/26522/26416 |No |GC distance/morph/collapse rules |
| Terrain positions/ST/base RGBA |Build7 cache + reader |86648 triangles structurally |No |Original VU output comparison |
| Texture images/CLUT/indices |Build7 |93 /279 images |No |Actual GC upload/sampling comparison |
| Material register destinations |Yes |All terrain records |No |GC load-time register patching |
| K signed fixup field |Yes |Synthetic + actual data |No |Live GS LOD/Q rule correlation |
| Coordinate frame/camera/FOV |No RC2 runtime |No |No |GC view constants + reference frame |
| TEST/ALPHA/ZBUF/texture alpha |No RC2 runtime |Static candidates only |No |Correlate terrain DMA/state writers |
| Fog/directional/point lighting |No RC2 runtime |No |No |GC VU/constants/light state |
| Primitive/GS draw order |Cache order retained only |CPU stream |No |VU emission and phase ordering |
| Tie header/packed positions |Partial |74/74 classes |No |Strip/slot emission + instance data |
| Moby classes |No new decoder |Prior183 nontrivial rejects |No |GC transfer/skin/packet layout |
| Shrubs/sky/chunks/effects |No RC2 runtime |No |No |Title-specific dependencies |
| Native gameplay/audio/menu |Missing |Not run |No |Real RC2 native game module |

## Original evidence and rejected shortcuts

A bounded private boot/level0 overlay A+D inventory found candidate TEST values
0x30000,0x3180b,0x5360b and0x52001, plus different ALPHA values. This is a
heuristic16-byte-aligned nonzero-data scan, **not exhaustive**. It does not prove
which table is inherited by Aranos tfrags. Do not pick the matching RAC1 value.
Runtime fields can be rewritten; raw tables are not a captured draw state.

Existing ReRAC references inspected: tfrag_render::{build_storage,spawn_tfrags_on},
TfragMaterial::{alpha_mode,specialize}, TfragAdGifs::{lod_k_raw,tex1_reg},
gs_state::{Test1,GsPass}, game_camera_fog and tfrag WGSL storage/lighting/morph
path. Those documents identify SCUS97199 addresses and explicit draw-order/blend
approximations; they are **not** GC VU or draw-state proof.

Format research remains pinned Wrench46e80430158a4a11e9cbd51424c84a015bbab058,
GPL-3.0-or-later reference-only (tie.h/tie.cpp field facts; no code linked/copied).
Additional public research candidate `platypet2217-star/RAC2Decomp` revision
`cda64fba7f7664403068b7014e1bae84b5068bc4` was inspected (README, MIT licence,
render_init.c, ps2_gs.c). It targets PAL SCES51607, describes functional rather
than byte-matching reconstruction, and supplies no verified Aranos tfrag/VU
state in those files. Do not adopt its compatibility approximations as an oracle.

## Reproduction and limits

Synthetic tests: `cargo test -p rc-data --lib`; formats lib includes rc2_tie.
Private opt-in tests (never put media on CI):

```
RC2_PRIVATE_SCENE='<private Build7 prepared directory>' \
RC2_PRIVATE_ISO='<local owned SCUS97268 v2 ISO>' \
cargo test -p rc-data --test rc2_scene -- --ignored
```

Tests fail explicitly without those inputs when opted in; default CI reports
them ignored rather than fabricating game-data coverage. Tie test checks image
size/boot hash, not a fresh whole-disc hash; the preserved image receipt remains
authoritative. No game bytes or original generated code in tests/repository.

Current native CPU tests executed locally via the existing offline minimal
extractor workspace (identical product source), not a Bevy/iOS build. Full local
engine cargo remains blocked by uncached pinned winit; Xcode13 cannot replace
the existing hosted Xcode27 device validation. No current-candidate ARM64/GPU
build, original-game oracle, scene screenshot or physical game test is claimed.

**First implementation blocker:** correlate GC terrain DMA/VU state (projection,
fog/lighting, TEST/ALPHA/ZBUF and register fixups) to original execution before
using RAC1's shader/material pipeline. Next: trace the actual boot A+D table
references and level0 terrain/VU setup; preserve original primitive order and
unknown GC class emission. A packet fixture/reference frame is needed for an
honest rendering comparison. RC2 Play stays disabled; no new IPA for a scene
that has not been implemented. RAC1 render/audio/controller/save paths unchanged.

Classification: GAME_SPECIFIC GC scene/class boundary; supporting
PLATFORM_SPECIFIC format semantics and SHARED_TESTING candidate. No Shared Core
version/adoption changes. Other engines remain untouched; generic test/GS work
requires separate review and originating/canary regressions before promotion.
