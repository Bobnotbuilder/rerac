# RC2 native asset phase — durable public handoff

## Emergency continuation: experimental terrain preview candidate

Added an opt-in `RC2_SCENE_PREVIEW=<private Build7 Aranos directory>` app path
before RAC1 import/save/gameplay boot. It uses the existing Bevy/Metal renderer,
a separate material/shader and a diagnostic camera. RC2 Play remains disabled.
CPU upload planning has executed against original Aranos data for all three
LODs (33710/26522/26416 triangles, 2578 contiguous draws each, 93 hash-verified
mip0 images). The renderer candidate still needs compilation/GPU execution;
there is no scene screenshot, original-state parity or gameplay result yet.

`RC2_PREVIEW_LOD=0|1|2`, `RC2_PREVIEW_CAM=ex,ey,ez,tx,ty,tz` (game axes),
`RC2_PREVIEW_CAPTURE=<NEW image path>` configure the diagnostic. WASD/QE and
arrows, or gamepad sticks, move the diagnostic camera only. Native controller
code is reused but this new route has not been input-tested. The camera uses
a diagnostic orthonormal Z-up rotation and 60-degree perspective. Rendering
uses two-sided opaque depth-writing terrain, raw RGB/128 modulation, mip0
bilinear wrap/clamp; alpha, lighting, fog and morph remain explicitly absent.
These policies are experimental and must not be represented as verified GC
behaviour. Hash receipts bind exactly the preserved SCUS97268 v2 slot0 package;
no game payload is included in source/CI and no prepared assets regenerated.

Local full-engine compilation still fails resolving uncached pinned winit,
and shell GitHub DNS fails. Hosted build is the next compilation gate.
Build7 remains the validated rollback; no new playable build is claimed.
Classification GAME_SPECIFIC; no Shared Core promotion or other-engine change.


## Current terrain evidence step (2026-10-23)

Local a15f9cd source/tests/docs now preserved on true remote history as
25157d7b2b7ff7bd7f86092ce09c226b290804f2, parent a974c687;7/7 blob hashes
match. Do not push the different local snapshot root. Build7 remains preserved.
See docs/workflows/rc2-terrain-reference.md for capture protocol and evidence
matrix. The existing trace harness has a bounded read-only original-GC GS-dump
inventory; synthetic parser tests are NOT original-game comparisons.

Managed Cocoa launch fails before VM start; packaged Qt has no offscreen plugin.
An isolated private reference profile is ready for user Terminal/Aqua launch.
No new RC2 capture, native scene or graphics claim. Need actual Aranos GS dump
and corresponding VU/EE savestate before selecting state/shaders. Object-class
completion is NOT a terrain prerequisite. Reuse existing833 fragments/textures;
Play remains disabled and RAC1 runtime paths unchanged. No private uploads.

## Latest CPU scene-boundary checkpoint (2026-10-23)

See `docs/validation/rc2-scene-boundary.md`. Build7 source and all prepared data
are preserved. New rc-data terrain reader and independent GC tie vertex-region
reader compile/test locally; private Aranos tests accept833 terrain fragments,
279 images,74 tie classes/1876 packets/93418 vertices. No native RC2 rendering
or gameplay path exists yet. Do not mistake a typed CPU reader for scene parity.
RC2 Play remains disabled; no current-candidate device build/new IPA claimed.
Exact next action: correlate original GC terrain DMA/VU state and register
patching, then connect the isolated scene to the existing renderer. Do not use
RAC1's hardcoded fog/camera/alpha/depth defaults without evidence. No media upload.

The historical Build7 receipt below remains valid for that app source only.

Continue `ios/rc2-native`; preserve Build6 source
15b0da96480627e54cfb86941c96858ab6b51c11 and its successful run39785625401.
Never replace original ios/arm64-poc or the separate C++ RAC1 project.

## What actually advanced

Existing production WAD/VIF/texture readers are now connected to independently
written GC container readers, the rc-data native scene asset boundary and
`rerac-extract prepare-rc2`. Owned local Aranos data produced655 hashed payload
files (24,084,002bytes), plus a completion manifest. No game data was uploaded.
This is partial asset conversion, NOT complete RC2 extraction or gameplay.

Real scoped measurements:27/27 terrain/core container and terrain mip sets parse;
Aranos833 fragments,93 materials/279 mip images,472 objects. Triangle counts per
LOD:33710,26522,26416. Native mesh14,210,288bytes,**86,648** triangles
across the three LOD lists (not unique geometry). Round-trip validator passed.
Aranos object class resolution472/472 against238 local moby definitions;74 tie
and7 shrub definitions recorded. Geometry for those classes is not converted.

All655 actual output files passed byte-count/SHA-1 checks; RGBA image lengths and
exact file-set checks passed. Manifest SHA256
a71834e2191f8cc10a358b7ce967a8621d173b8ac97c0bfa5095639161b216ce.
This digest is provenance, not an independent PS2/game rendering oracle.

Local tests:163 formats passed,1 ignored;7 data passed;23 extractor library
passed,1 ignored;2 CLI passed;59 existing iOS Python contracts passed. Existing
data-dependent formats cases may return early without extracted RAC1 data.
No new real-device regression or RC2 GPU/game execution was performed.

## Build and preservation

## Final hosted validation receipt

Application source `4c94e5350fd28afed19633c51fa403d340758765`.
[Run39859007049](https://github.com/Bobnotbuilder/rerac/actions/runs/39859007049)
completed SUCCESS for both jobs. Engine/UI1087 passed5 ignored;
formats/extract/data1096 passed3 ignored;0 failures.1052 unit cases repeat.
Actual iOS27 simulator startup, synthetic GameController, safe display/resize,
batched/unbatched AA and shadow GPU probes passed. No owned game data on CI;
not a physical Backbone, real game scene or RC2 runtime test.

[Unsigned IPA](https://github.com/Bobnotbuilder/rerac/actions/runs/39859007049/artifacts/12461623495),
[matched symbols](https://github.com/Bobnotbuilder/rerac/actions/runs/39859007049/artifacts/12463070281),
[build evidence](https://github.com/Bobnotbuilder/rerac/actions/runs/39859007049/artifacts/12462584969).
Inner IPA SHA256 `8f8071b774aa8a9d5f9888749536bd3b71a05c79f5f588eea13bbaa963898d1a`.
Executable/dSYM UUID `2056fe6b-44a2-3d51-9718-03a81afcbbb8`, ARM64,
minimum iOS16.0, SDK27.1,24 original WGSL shaders, unsigned/no provisioning.
Bundle `org.rerac.game.iospoc`,0.3.4/build7. No valid local signing identity.
Download artifact, unzip, sign inner IPA with the existing Sideloadly account/
effective identifier and update without uninstalling; back up ReRACSupport first.
Do not replace RAC1 data with the partial RC2 conversion. RC2 Play is disabled.

RAC1 game logic/rendering/audio/controllers/saves remain unchanged; no new
physical playthrough claimed. This is an asset-preparation checkpoint, not a
new playable RC2 app. Any later documentation-only commit is not the binary
source revision; use the application source above for exact reproduction.

Further local research: all4 old-parser moby accepts contain0 packets/vertices;
183 nontrivial blobs reject and51 have no local geometry offset. No usable
RC2 model support follows from those accepts. Independent GC tie-header/region
probe checks74 classes/1876 packets, not GS-order/colour/multipass correctness.
Do not port reference approximations as hardware truth or forge RAC1 headers.

The following pending-build notes are retained as the earlier execution stage;
the final receipt above supersedes them.

Candidate build number7; same0.3.4/bundleorg.rerac.game.iospoc/schemeReRAC/iOS16.
Existing workflow builds actual device ARM64 and simulator checks; current
candidate hosted result must be checked before declaring an IPA built.
Build7 is an asset-library/preparation checkpoint: app RC2 Play stays disabled.
No scene-preview/scene-rendering feature is claimed in this build.
Existing native RAC1 gameplay/controller/audio/rendering/save code unchanged.
No signing credentials available; any hosted IPA is unsigned until user signs.

## Earliest unresolved implementation

`RC2_NATIVE_RENDER_STATE_AND_VU_PARITY`, object geometry/converter dependencies,
then the real RC2 gameplay module. Runtime FIRST_FATAL_BLOCKER=null: no RC2
runtime executed. Do not activate rc-game's RAC1 scheduler for RC2 IDs.
Whole engine/game coverage NOT MEASURABLE YET. No visible scene/audio/menu/play.

Safe resume:

1. Inspect branch and dirty tree; match candidate commit and workflow evidence.
2. Read docs/formats/rc2_native_assets.md, then private local GameStudio
   .work/rerac-rc2 progress/handoff. Preserve Build6 and completed Aranos output.
3. Do not regenerate/hash the ISO or assets without input/source change.
4. Trace GC global GS state, VU tfrag transforms, lighting/fog and per-material
   packets before sharing RAC1's renderer. Existing parsers accepting a stream
   do not prove the game's different VU/shader semantics.
5. Convert actual class geometry and instance dependencies; preserve unknown
   classes as missing, never draw fabricated replacements.
6. Integrate a validated native RC2 scene path only after those gates; run RAC1
   regressions, native ARM64 workflow and then physical iPhone visual tests.
7. Keep all ISO, extracted/converted bytes and private logs local. Public CI uses
   only source and synthetic tests. No automatic Shared Core adoption.

Research Wrench GPLv3 reference-only, no implementation copied/linked. See format
document for pinned revision and files. Classification GAME_SPECIFIC, supporting
PLATFORM_SPECIFIC format facts/SHARED_TESTING candidate; other engines unchanged.
