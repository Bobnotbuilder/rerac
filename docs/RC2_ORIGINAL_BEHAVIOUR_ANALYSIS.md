# Original behaviour recovered — 25 October 2026

The uploaded boot recompiler output is evidence, not a gameplay runtime. All ZIP entries passed CRC checking. The boot ELF matches the current private source exactly.255,241 generated instruction-comment records covering253,705 unique PCs match the boot ELF's file-backed words, with zero mismatches/unmapped words. This checks source provenance, not C++ semantics.

Boot discovery2884/recompiled2778/stub106 is reconciled with2885 per-function files: address11cda0 has two configured-stub files. The handoff index therefore has107 stub rows but106 unique stub addresses. Six of112 configured addresses have no discovered function:11f130,130b80,130c68,130db8,130e88,131430. They must not be counted as six unimplemented gameplay systems. Generated runtime execution remains NOT_TESTED; supplied runtime.log says RUNTIME_NOT_BUILT.

The boot entry131ae8 leads through131d98's constant indirect handoff26ede8. Its dispatch is not evidence that player movement has been recovered. Actual Aranos code comes from LEVEL0's overlay. The derived analysis ELF enables a separate PS2Recomp pass:3620 discovered/recompiled,0 stubs/skipped/decode failures/unhandled instructions/errors;128 indirect-warning records with20508 promoted fallback entries and43247 additional entrypoints. These are static tool counts, not coverage of all original mechanics or correctness of promoted targets.

A reproducible original chain is level entry2a3b88→pad wrapper330dc0→3303b8→3306b0, and level entry2a3b88→update380c50→gated hero2a6dd0→input/physics2b37e8→state physics2ba8e0, followed by state transitions2cb650. The full original update also includes required object, camera and other services not yet integrated. Native baseline scheduling is not declared equivalent merely because these components parse/compile.

Two supported native improvements result:

- Falling with nonzero landing lag keeps target selection, drag and gravity but suppresses air steering/momentum writes until lag expires. Original-data CPU fixture executes PadState→hero_update→original animation→physics→collision→transitions and verifies steering resumes on the first zero-lag tick. Restoring old unconditional steering makes that test fail.
- Class132d initialization now copies the original Aranos descriptor into the actual pvar owner and generates its controller arrays through the one world RNG in original conditional draw order. It advances the same original-data initialization cursor from object23 to the next required callback at object24/class1330. Later132d query/draw services remain blocked.

No new weapon, HP receiver, enemy AI, pickup, checkpoint or level-completion correctness is claimed. Existing movement/jump/camera/pose fixtures still pass. Native world's save/RNG inputs are explicit synthetic fixtures, not original New Game state. See the function map for exact fields, calls and confidence. Full Aranos and the full campaign remain INCOMPLETE.
