# Original behaviour recovered — 25 October 2026

## Class1330 recovery continuation — current, 25 October 2026

**INCOMPLETE: initializer cursor remains object24.** The immediate acceptance criterion of advancing past24 is NOT achieved. First required service now narrowed to **GC_ARANOS_CLASS_1330_SOURCE_2B9E68_HERO_TELEPORT_SERVICES_UNPORTED**. Local source `db2d9f12ff17c1476b70bb117d27c1b50d84355a` / published `2abbf8bdbc42e4f90eba33bb3ee46b3f61a8fea4`; start local7925c78 / publicd1d38d0. Native run40153555598 pending; prior prefix run40153085562 separate. No claim of original-game GPU or physical-device execution.

The original-data fixture executes2f51a0 → conditional322730(18hex) →368560 through the existing World with explicit runtime globals. Retries preserve completed prefix effects without replaying them or consuming RNG. Actual owned pvars+4=40 selects gameplay header+68's **cuboid40**, not a camera. Its position is(247,193.16782,49.89,w=.01), Euler(0,-0,1.5707964,0), state argument0, camera-reset argument1. The required teleport is retained as a typed pending request, never treated as completed.

Native32a748 owner connection and360cd0/360d28 allocation/lifetime services reuse the same MobyTable, actual pvars, animation/matrix and grid. Original-data tests separately exercise original classd54 and nine source-word0 attachment allocations, replacement/deletion+2tick reuse rules, flag/order/pvar aliasing and no RNG draws. These are service integration tests, **not an out-of-order initializer bypass**. Full384d28 poses, scripts and callback tail remain required.

Motion service2b2cd8 is implemented through the existing GC Hero and shared PadState. Its source-hash-pinned test runs after original animation-driven movement, resets exact motion/history fields without clearing buttons/sequence, then executes the next original-animation/physics/collision tick. Two formerly absent vector blocks and W lanes have explicit Hero-owned storage; unidentified block189ec0 retains its address name. Full2b9e68 remains unported; this method is not invoked as a substitute teleport by the engine.

Tests executed: RAC1 units715PASS, formats179PASS/1optional fixture ignored, GCunits34PASS, original-data CPU10PASS, iOScontracts59PASS, recoveryPython13PASS plus nested native AOT harness6PASS. Original PS2 execution traces, complete world tick, current original-data GPU, physical iPhone/Backbone and complete Aranos acceptance remain untested/incomplete. Internal native build checks continue; no user-facing IPA or enabled production RC2 Play.

Reuse audit and automatic compatibility/AOT/callback grouping: [RC2_REUSE_COMPATIBILITY.md](RC2_REUSE_COMPATIBILITY.md). Fixes GAME_SPECIFIC; genuinely reusable validation SHARED_TESTING candidate only. No Shared Core promotion, other-engine adoption, R1 implementation/save or app identity changes.

Next executable step: Bind original2b9e68 to the existing GC Hero/Pad/World owners. Reuse the now implemented2b2cd8 reset; recover/apply324140 tracked-object teardown,2af848 item put-away,2a53b8 modifier reset,2c4560 state0 mode2,2d2760 animation,2b4a08 probe,3e0b28/403d98 optional actor effects,2da160 camera reset and313008 lighting/grid. Preserve original order, then execute311320(d54),32a748,table73,360d28 attachments,script gates and384c40/384d28 tail before advancing cursor24. Keep132d state1 query383fc0/queued3842f8 separate.

## Previous checkpoint evidence (historical)

The uploaded boot recompiler output is evidence, not a gameplay runtime. All ZIP entries passed CRC checking. The boot ELF matches the current private source exactly.255,241 generated instruction-comment records covering253,705 unique PCs match the boot ELF's file-backed words, with zero mismatches/unmapped words. This checks source provenance, not C++ semantics.

Boot discovery2884/recompiled2778/stub106 is reconciled with2885 per-function files: address11cda0 has two configured-stub files. The handoff index therefore has107 stub rows but106 unique stub addresses. Six of112 configured addresses have no discovered function:11f130,130b80,130c68,130db8,130e88,131430. They must not be counted as six unimplemented gameplay systems. Generated runtime execution remains NOT_TESTED; supplied runtime.log says RUNTIME_NOT_BUILT.

The boot entry131ae8 leads through131d98's constant indirect handoff26ede8. Its dispatch is not evidence that player movement has been recovered. Actual Aranos code comes from LEVEL0's overlay. The derived analysis ELF enables a separate PS2Recomp pass:3620 discovered/recompiled,0 stubs/skipped/decode failures/unhandled instructions/errors;128 indirect-warning records with20508 promoted fallback entries and43247 additional entrypoints. These are static tool counts, not coverage of all original mechanics or correctness of promoted targets.

A reproducible original chain is level entry2a3b88→pad wrapper330dc0→3303b8→3306b0, and level entry2a3b88→update380c50→gated hero2a6dd0→input/physics2b37e8→state physics2ba8e0, followed by state transitions2cb650. The full original update also includes required object, camera and other services not yet integrated. Native baseline scheduling is not declared equivalent merely because these components parse/compile.

Two supported native improvements result:

- Falling with nonzero landing lag keeps target selection, drag and gravity but suppresses air steering/momentum writes until lag expires. Original-data CPU fixture executes PadState→hero_update→original animation→physics→collision→transitions and verifies steering resumes on the first zero-lag tick. Restoring old unconditional steering makes that test fail.
- Class132d initialization now copies the original Aranos descriptor into the actual pvar owner and generates its controller arrays through the one world RNG in original conditional draw order. It advances the same original-data initialization cursor from object23 to the next required callback at object24/class1330. Later132d query/draw services remain blocked.

No new weapon, HP receiver, enemy AI, pickup, checkpoint or level-completion correctness is claimed. Existing movement/jump/camera/pose fixtures still pass. Native world's save/RNG inputs are explicit synthetic fixtures, not original New Game state. See the function map for exact fields, calls and confidence. Full Aranos and the full campaign remain INCOMPLETE.
