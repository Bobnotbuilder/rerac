# RC2 mechanics regression results — 25 October 2026

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
| Hosted native validation |Current internal run40144142983 Intel/macOS compile and hosted headless/overlay/synthetic Metal PASS; ARM64 packaging pending; prior d7105d3 run40116110147 all jobs SUCCESS |
| Current original-data GPU / physical iPhone / Backbone |NOT_TESTED |
| Full route, combat, scripted completion, sustained performance |NOT_TESTED / INCOMPLETE |

Initial controller-vector unit run failed on one phase's last bit: an independent IEEE truncate-only expectation omitted the existing PS2 adder guard-bit rule. An independent integer arithmetic reconstruction corroborated the native model's3fbcb84b; the test now explicitly states MODEL evidence, not captured PS2 floats. Original failed log retained. No native math model was weakened to pass a test; possible FPU/VU adder and rare multiplier differences remain unvalidated.

Private logs: `evidence/ps2recomp-mechanics-{formats,rac1,game2}-20261025.log`, `mechanics-fall-original-20261025.log`, `ps2recomp-mechanics-fall-negative-control-20261025.log`, `ps2recomp-mechanics-initial-oracle-failure-20261025.log`, `ps2recomp-mechanics-ios-contracts-20261025.log`, `ps2recomp-mechanics-local-engine-check-20261025.log`. Original-data tests were actually opted into the owned ISO; generic CI without that private input reports NOT_TESTED for those fixtures.

No user-facing IPA produced. Internal CI packaging is development evidence only; complete-Aranos acceptance remains false.

Additional original-overlay consistency audit:426951 instruction-comment records/425370 unique PCs, zero word mismatches/unmapped. Boot ELF is byte-identical to the existing private extracted source. Local xctrace discovery lists only the Mac; Intel HD Graphics5000 reports Metal support, but connector artifact retrieval could not be materialized because the shell cannot resolve the temporary download host. No local GPU execution was fabricated.
