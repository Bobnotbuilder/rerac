# RC2 native asset phase — durable public handoff

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
