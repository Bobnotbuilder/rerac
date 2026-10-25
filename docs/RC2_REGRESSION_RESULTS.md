# RC2 mechanics regression results — 25 October 2026

## Class1330 recovery continuation — current, 25 October 2026

**INCOMPLETE: initializer cursor remains object24.** The immediate acceptance criterion of advancing past24 is NOT achieved. First required service now narrowed to **GC_ARANOS_CLASS_1330_SOURCE_2B9E68_HERO_TELEPORT_SERVICES_UNPORTED**. Local source `db2d9f12ff17c1476b70bb117d27c1b50d84355a` / published `2abbf8bdbc42e4f90eba33bb3ee46b3f61a8fea4`; start local7925c78 / publicd1d38d0. Native run40153555598 pending; prior prefix run40153085562 separate. No claim of original-game GPU or physical-device execution.

The original-data fixture executes2f51a0 → conditional322730(18hex) →368560 through the existing World with explicit runtime globals. Retries preserve completed prefix effects without replaying them or consuming RNG. Actual owned pvars+4=40 selects gameplay header+68's **cuboid40**, not a camera. Its position is(247,193.16782,49.89,w=.01), Euler(0,-0,1.5707964,0), state argument0, camera-reset argument1. The required teleport is retained as a typed pending request, never treated as completed.

Native32a748 owner connection and360cd0/360d28 allocation/lifetime services reuse the same MobyTable, actual pvars, animation/matrix and grid. Original-data tests separately exercise original classd54 and nine source-word0 attachment allocations, replacement/deletion+2tick reuse rules, flag/order/pvar aliasing and no RNG draws. These are service integration tests, **not an out-of-order initializer bypass**. Full384d28 poses, scripts and callback tail remain required.

Motion service2b2cd8 is implemented through the existing GC Hero and shared PadState. Its source-hash-pinned test runs after original animation-driven movement, resets exact motion/history fields without clearing buttons/sequence, then executes the next original-animation/physics/collision tick. Two formerly absent vector blocks and W lanes have explicit Hero-owned storage; unidentified block189ec0 retains its address name. Full2b9e68 remains unported; this method is not invoked as a substitute teleport by the engine.

Tests executed: RAC1 units715PASS, formats179PASS/1optional fixture ignored, GCunits34PASS, original-data CPU10PASS, iOScontracts59PASS, recoveryPython13PASS plus nested native AOT harness6PASS. Original PS2 execution traces, complete world tick, current original-data GPU, physical iPhone/Backbone and complete Aranos acceptance remain untested/incomplete. Internal native build checks continue; no user-facing IPA or enabled production RC2 Play.

Reuse audit and automatic compatibility/AOT/callback grouping: [RC2_REUSE_COMPATIBILITY.md](RC2_REUSE_COMPATIBILITY.md). Fixes GAME_SPECIFIC; genuinely reusable validation SHARED_TESTING candidate only. No Shared Core promotion, other-engine adoption, R1 implementation/save or app identity changes.

Next executable step: Bind original2b9e68 to the existing GC Hero/Pad/World owners. Reuse the now implemented2b2cd8 reset; recover/apply324140 tracked-object teardown,2af848 item put-away,2a53b8 modifier reset,2c4560 state0 mode2,2d2760 animation,2b4a08 probe,3e0b28/403d98 optional actor effects,2da160 camera reset and313008 lighting/grid. Preserve original order, then execute311320(d54),32a748,table73,360d28 attachments,script gates and384c40/384d28 tail before advancing cursor24. Keep132d state1 query383fc0/queued3842f8 separate.

## Previous checkpoint evidence (historical)

Source local112c2aa968d18148c999237c7f1222e1e9cfa5ac / publica774e199aa7f05e92e630f3f764680303de0672d. Implementation770c793e3180ec197d8746f572a207ff98c3ce36.

| Executed check | Result / scope |
|---|---|
| Uploaded ZIP integrity | Both archives CRC PASS;7 handoff entries and2915 output entries; complete entry inventory retained privately |
| Boot source-comment instruction consistency |255241 records/253705 unique PCs;0 word mismatches/0 unmapped; not C++ semantic validation |
| Static call index |9517 boot caller/callee pairs;16784 logged call sites; overlay30211 logged sites/16341 pairs; computed dispatch remains incomplete |
| Aranos overlay recompiler |3620 discovered/recompiled;0 decode failures,0 errors;128 indirect warnings; translated code NOT executed |
| Offline native rc-game2 library build | PASS in byte-identical private CPU validation workspace; no Bevy/iOS build implied |
| rc-formats unit tests |179 PASS,1 ignored; optional extracted-tree skip messages retained |
| Existing RAC1 game units |715 PASS; this is not a fresh device playthrough or save compatibility capture |
| RC2 game units |27 PASS, including conditional RNG consumption and existing PS2 math-model phase vectors |
| Original RC2 ISO CPU tests |10 PASS: original character111 joints/all original pose fixture, movement/jump/camera, collision, audio, pvars/items/targeting/world plus fall lag and controller init |
| Fall old-code negative control |EXPECTED FAIL exit101: unconditional air steering violates lag assertion; production source not modified by negative test |
| Overlay-wrapper synthetic contracts |4 PASS; initialized bytes, memory-only NOBITS, malformed/truncated/overlapping/entry validation |
| Existing iOS Python contracts |59 PASS |
| Local full rc-engine cargo check --offline |BLOCKED_ENVIRONMENT: pinned winit revision94d6a23c425023810a0879aa7a03abb2e4f1decc unavailable offline |
| Hosted native validation |Current internal run40144142983 ALL3JOBS PASS: Intel/macOS, ARM64 package/matched symbols, headless/overlay, synthetic Metal and iOS27 batched/unbatched startup/controller/resize/AA/shadow probes; prior d7105d3 run40116110147 all jobs SUCCESS |
| Current original-data GPU / physical iPhone / Backbone |NOT_TESTED |
| Full route, combat, scripted completion, sustained performance |NOT_TESTED / INCOMPLETE |

Initial controller-vector unit run failed on one phase's last bit: an independent IEEE truncate-only expectation omitted the existing PS2 adder guard-bit rule. An independent integer arithmetic reconstruction corroborated the native model's3fbcb84b; the test now explicitly states MODEL evidence, not captured PS2 floats. Original failed log retained. No native math model was weakened to pass a test; possible FPU/VU adder and rare multiplier differences remain unvalidated.

Private logs: `evidence/ps2recomp-mechanics-{formats,rac1,game2}-20261025.log`, `mechanics-fall-original-20261025.log`, `ps2recomp-mechanics-fall-negative-control-20261025.log`, `ps2recomp-mechanics-initial-oracle-failure-20261025.log`, `ps2recomp-mechanics-ios-contracts-20261025.log`, `ps2recomp-mechanics-local-engine-check-20261025.log`. Original-data tests were actually opted into the owned ISO; generic CI without that private input reports NOT_TESTED for those fixtures.

No user-facing IPA released. Internal unsigned IPA and UUID-matched symbols were generated and verified; package sourcea774e199, run40144142983. It remains INTERNAL_EXPERIMENTAL and withheld; complete-Aranos acceptance remains false.

Additional original-overlay consistency audit:426951 instruction-comment records/425370 unique PCs, zero word mismatches/unmapped. Boot ELF is byte-identical to the existing private extracted source. Local xctrace discovery lists only the Mac; Intel HD Graphics5000 reports Metal support, but connector artifact retrieval could not be materialized because the shell cannot resolve the temporary download host. No local GPU execution was fabricated.

Final source provenance:728 validation-workspace source files checked byte-identical; native rc-game2 rebuilt offline successfully. Recovered LEVEL0.WAD byte-identical to the file in the original integration-test ISO (15465088bytes, SHA1 892d1c0f4a8d94fdf479273483d6e0eaa425000f). Native CI reports original-data fixtures NOT_TESTED where its private ISO is absent.
