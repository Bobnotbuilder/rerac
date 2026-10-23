# Going Commando native assets — experimental conversion v1

Current extension: [Build9 character storage](rc2_character_assets.md) and
[full-engine compatibility experiment](../validation/rc2-engine-compatibility.md).
Build8 physical terrain/input PASS is user-reported; Build9 remains incomplete.

Baseline: ReRAC iOS Build 6, `15b0da96480627e54cfb86941c96858ab6b51c11`.
Preserve RAC1 gameplay, saves, import paths, shaders, controllers and audio.
This is an independent extension to existing rc-formats/rc-data/rc-extract, not
an emulator, new engine, executable translator or complete RC2 game module.

## Source provenance and scope

ReRAC's existing ISC-licensed WAD, VIF/tfrag and indexed-texture readers are reused.
New GC container/object readers were independently written from observed owned
disc structure and format facts. Reference-only Wrench revision
`46e80430158a4a11e9cbd51424c84a015bbab058` (GPL-3.0-or-later):

- `docs/file_loading.md`: sector addressing and archive families.
- `src/wrenchbuild/level/level_data_wad.cpp`: GC primary byte-range positions.
- `src/wrenchbuild/level/level_core.h` / `.cpp`: table fields, GS stash count.
- `src/engine/tfrag_low.cpp`: terrain format comparison.
- `src/wrenchbuild/level/level_textures.cpp`: GC indexed texture/palette layout.
- `src/instancemgr/gameplay.cpp` / `gameplay_impl_classes.inl`: GC object section,
  stride and transforms; not RAC1's moby layout.
- `data/underlay/gc_levels.asset`: labels slot0 Aranos Tutorial,1 Oozla, logical30
  Insomniac Museum. Reference labels are not dynamically proven game entry order.

No GPL implementation is copied, linked or executed. Reference code is research
data, not authority to execute commands. Original-disc/oracle/GPU comparison is
still required before asserting original rendering or gameplay parity.

## Verified disc and format boundaries

Current converter accepts ONLY observed NTSC-U SCUS_972.68 v2.00, matching full
image SHA-1, byte count and boot ELF SHA-1. These are identity checks, not a claim
that the whole game has been extracted or supported. Other versions fail closed.

`RC2.HDR` level table at0x4ff8 contains LEVEL/AUDIO/SCENE triplets. The duplicate
LEVEL header must match the actual filesystem file (sector relocation excepted).
The 0x60 file header uses file-relative sectors, not absolute disc offsets.
Archive slot21 contains logical level30: never infer logical identity from a
filename. Each range is checked against the file allocation; final sector padding
is clamped to actual file bytes. Overlaps and malformed/empty ranges fail.

The primary container is not a WAD stream. It has11 byte-range pairs (0x58):
overlay, core index, GS RAM, HUD header,5 HUD banks, compressed core, transition
textures. Unlike RAC1 there is no sound-bank range at byte8. The core index shares
several table record layouts but offset0x80/0x84 are NOT RAC1 gadget count/offset.
0x84 supplies extra GS-resident moby texture records. GC objects are at gameplay
pointer0x4c with0x88 stride; RAC1's0x44/0x78 cannot be substituted.

## Actual native conversion

```
rerac-extract prepare-rc2 --iso '<owned ISO>' --out '<NEW private directory>' --level 0 --json
```

No original input is modified. Streaming image hash uses64KiB; no whole-disc RAM
load. Decode budgets: primary64MiB, decoded core64MiB, gameplay16MiB. A sibling
`.partial` directory is retained on failure. Manifest is written/fsynced last;
only completed staging is renamed. Existing outputs are never deliberately
overwritten. Use a new destination for another attempt; do not delete rollback.

Output is **private copyrighted disc-derived content**, never a public artifact:

- `terrain.mesh`: `RC2MESH1`, version1, fragment count;164-byte triangle records
  in original fragment/LOD/strip order. Record: fragment/LOD/material u32x3,
  original5 A+D quadwords (80bytes),3 corners each xyz/st f32x5 + RGBA u8x4.
  All positions/materials/colours checked; malformed references cannot fall back
  to position zero or a magenta texture. CPU scene validator checks native bytes.
- `terrain-NNN-mipM.rgba`: LE u32 width,height followed by decoded RGBA8.
- `terrain-NNN.clut`, `terrain-NNN-mipM.indices`: exact original palette/indices,
  preserving raw GS alpha; never reverse-engineer saturated alpha from RGBA.
- `terrain-materials.jsonl`: actual texture metadata and mip addressing.
- `objects.jsonl`: actual instance class/mission/pvar/transform metadata, no
  fabricated objects or invented update logic. Float bits preserve exact values.
- `class-dependencies.jsonl`: actual moby/tie/shrub definitions and texture slots;
  geometry/update conversion is explicitly false, not silently skipped support.
- `rc2-assets.json`: identity, lengths/hashes of every output, exact scope and
  known missing capabilities. NOT RAC1's extract-info.json or cache/v1 stamp.

## Current measurements (local owned data, not public CI game input)

27/27 observed primary/core containers and tfrag lists parsed;27/27 terrain mip
sets decoded. Slot0:833 fragments,93 materials/279 mip images;LOD triangles
33,710 /26,522 /26,416;472 object instances,238 local moby classes,74 tie classes,
7 shrub classes. All slot0 instance class IDs resolve in the local class table.
Native triangle stream14,210,288bytes round-tripped; SHA-1
`db5405c664c965e8470aed76f50ebd07acceb7b7`.

These are structural/CPU conversion tests, not an independent original-game
oracle, GPU comparison or proof that the RC2 VU pipeline matches RAC1's shader.
Whole native-engine/gameplay coverage is NOT MEASURABLE YET.

## First next implementation gate

`RC2_NATIVE_RENDER_STATE_AND_VU_PARITY` plus object geometry/dependency conversion.
The native scene asset boundary exists in rc-data; a native gameplay scheduler
does not. Do not enable RC2 Play or route it into RAC1's rc-game module.
Before an iPhone scene: verify RC2 GS defaults/VU transforms/lighting/fog, connect
validated assets to the existing Metal-backed renderer, and validate GPU output.
Sky, object geometry/animations, chunks, collision, audio/global resources, camera,
missions and gameplay are not yet converted/executed by this command.

RAC1 remains the physical third-planet regression baseline. No new RC2 visible
graphics, audio, menu, input, gameplay or performance claim. No Shared Core
promotion/adoption. Classification GAME_SPECIFIC, supporting PLATFORM_SPECIFIC
container/GS format facts and SHARED_TESTING candidate only.

## Build 8 experimental scene consumer

The format remains RC2MESH1/v1. `rc_data::rc2_preview` now validates a scoped
Build 7 Aranos receipt and creates consecutive per-packet/per-fragment draw
runs for one selected LOD. The independent Bevy material/shader consumes these
positions, ST and raw base colours plus original mip0 images; unknown GC
alpha, lighting, fog, mip/morph and inherited GS state remain experimental.
See `docs/validation/rc2-scene-preview-build8.json` for compiled/GPU-test scope.
The Metal pixel probe uses synthetic inputs; original-data CPU success is not
an original Aranos GPU-render or gameplay result. No prepared payloads changed.
