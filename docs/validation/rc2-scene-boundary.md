# RC2 Aranos experimental preview — original GPU frame pending

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
