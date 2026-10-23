# RC2 Aranos scene boundary — CPU candidate, not a rendered scene

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
