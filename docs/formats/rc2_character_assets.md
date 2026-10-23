# GC character asset adapter — storage facts and limits

Independent Rust readers extend existing ReRAC structures, with unchanged RAC1
entry points and an explicit GC layout selection. No reference implementation
is linked or copied. Layout research uses pinned Wrench revision
`46e80430158a4a11e9cbd51424c84a015bbab058`:

- [Class/armor headers](https://github.com/chaoticgd/wrench/blob/46e80430158a4a11e9cbd51424c84a015bbab058/src/engine/moby_low.h)
- [Compact vertex layout facts](https://github.com/chaoticgd/wrench/blob/46e80430158a4a11e9cbd51424c84a015bbab058/src/engine/moby_vertex.cpp)
- [Armor archive ranges](https://github.com/chaoticgd/wrench/blob/46e80430158a4a11e9cbd51424c84a015bbab058/src/wrenchbuild/globals/armor_wad.cpp)
- [Special sequence storage](https://github.com/chaoticgd/wrench/blob/46e80430158a4a11e9cbd51424c84a015bbab058/src/engine/moby_animation.cpp)
- [Class0 Ratchet identity](https://github.com/chaoticgd/wrench/blob/46e80430158a4a11e9cbd51424c84a015bbab058/data/underlay/gc_moby_classes.asset)

Original owned SCUS_972.68 v2.00 data supplies independent structural checks.
Boot SHA1 is fixed to `4124a79c1e5a1ceb56f514df0dbbc11f072af56f`.
These facts do not establish original runtime behavior or GPU/animation parity.

GC regular packets use eight u16 fields in a 0x10-byte vertex header, versus
RAC1's eight u32 fields in0x20 bytes. GC vertex records/epilogue end at the
packet's `vertex_data_size*16`; header+0xe is not the RAC1 color-blob offset.
The adapter reuses existing bounded VIF parsing, index walking, duplicate cache
and native skin records, preserving per-LOD state. Structural success is not
proof that original GC VU0 scheduling, normal/light calculations or materials
match RAC1. Nonzero class+0xb legacy layouts are explicitly unsupported here.

ARMOR.WAD starts with a 0xf8-byte file-relative header, nine armor entries then
six wrench entries. Each entry has two sector ranges, mesh and PIF-list.
Mesh-only headers hold LOD counts, class-relative packet-table pointer and GIF
usage pointer. They contain neither a skeleton nor animation sequences.
Armor0's63 high/22 low/2 metal packets match Aranos class0 counts. The native
reader retains separate lists, packed positions/UVs/skin records and original
texture indices. World scale comes from the original class, not a replacement
constant. No render/upload or original loader-selection claim is made yet.

The armor texture list is u32 count followed by ordered blob-relative u32 PIF
offsets. Four original PIFs decode through existing ReRAC PIF/raw-alpha code:
256x256,128x128,128x128,32x32. Class0 has111 skeleton matrices and111 hierarchy
records. The private contract pins mesh, class, texture-list and decoded-image
hashes without committing original bytes.

Core index+0x78 points to256 data-relative hero sequence offsets. Each nonzero
slot begins an independent sequence blob. Original slot numbers and absent
entries remain intact, including resident slot157 beyond the class header's
157-slot pointer list. Do not infer an idle/run/jump meaning from slot numbers.

All96 resident Aranos hero blobs use flagged frame pointers (`0xf` high nibble,
28-bit blob-relative offset). Shared data follows sequence frame/trigger lists:
packed10/11/11-bit channel boundaries,111 triples of u16 base joint data,
two record counts and corresponding8-byte base records. Per-frame storage has
two raw u16 header words, three byte channels,ceil(joints/8) joint-flag bytes,
and variable record deltas. Each delta flag's three 2-bit widths encode0/1/2
following bytes; value3 consumes none. Unknown bits and raw data are preserved.
Every span is bounded by its blob and next distinct frame offset. Unsupported
mixed frame flags and malformed spans fail explicitly.

`rc2_anim` is a **storage reader**, not a quaternion/pose decoder. It assigns no
meaning to compressed channels, timing words, joint flags or delta arithmetic.
The regular reader is reused only for unflagged storage. The RAC1 evaluator and
title-specific animation state machine are never invoked for these blobs.
Verified original-pose output is the next dependency before native skinning.

The controlled engine adapter accepts GC0x88-byte object metadata and existing
Collision/MobyClass/Moby/MobyTable types. It never invokes RAC1 InitMobyInstance,
class handlers, load pass, `tick::Game`, checkpoint/save code or title routing.
Mission and pvar indices remain GC metadata. Diagnostic records disable update
and animation and have no update address. The source is not an RC2 gameplay module.

The existing moby GPU infrastructure also needs a title-policy adapter before GC
upload: `moby_render::glow_from` interprets class+0xa/+0xb as RAC1 glow boundaries,
but GC+0xb=0 is the compact-format selector. Passing those raw fields unchanged
would mark all low-LOD packets as glow. Existing `moby_light` also defaults an
absent multiplier record to0x80; that is not evidence of GC lighting parity.
Preserve the shared mesh/material/palette infrastructure and supply independently
validated GC decisions. No current compatibility path calls this renderer policy.

Animation counts are per resident table slot:96 slots reference95 distinct blob
hashes; slots156/157 alias. The2187 per-slot frame records include that alias
(2177 records across distinct blob hashes). Neither count denotes evaluated poses.
