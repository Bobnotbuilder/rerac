# Native iOS integration — 25 October 2026

Starting local handoff4cae0b30c4d24041c8c6ceaaeca351ff9373a4d9, implementation0e217fc64071fd0828ac270935afc8253f5327c6; starting published handoff5761c669226b36198b0066d0c4ac5369379ccacf, implementationd7105d3ec60bbf822c3853b242cfb77852939776. Local and published histories differ deliberately; neither was reset.

New implementation770c793e3180ec197d8746f572a207ff98c3ce36; CI wiring112c2aa968d18148c999237c7f1222e1e9cfa5ac. Published sourcea774e199aa7f05e92e630f3f764680303de0672d. Only sanitized source/tests/tooling published, based on the existing public tree with remote-only files retained.

| Component | Integration |
|---|---|
| rc-game2/src/hero/air.rs | Existing fall physics corrected at its original interface; reused input, movement, collision, animation and camera |
| rc-game2/src/controller_132d.rs | Aranos-only signature-checked descriptor reader and CPU controller initialization; existing Rng/Pf/VU cosine reused |
| rc-game2/src/world.rs | Source-bound callback state0 operates on actual owner/pvars and shared RNG; dependency validation precedes mutation; state1 remains unsupported |
| rc-engine/src/rac2_level.rs; rac2_play/native_world.rs | Pass original descriptor through existing LevelContract to existing opt-in world; no second renderer or gameplay engine |
| rc-game2/tests/original_aranos.rs | Original fall chain and same-cursor controller initialization/next blocker tests |
| tools/recovery/rc2_overlay_to_elf.py + tests | Private static-analysis wrapper; never linked into app; CI synthetic contract tests added |

Default renderer/audio/input/assets and native-world opt-in flag remain as before. Production RC2 Play remains disabled. No RAC1 implementation, imported data, saves, effective bundle identityorg.rerac.game.iospoc, signing setup or installed engine pointer changed. No general-purpose emulation, interpreter or JIT added. Generated C++ remains private reference material and is not compiled into the application.

Current native CI40144142983 is INTERNAL_EXPERIMENTAL. Prior sourced7105d3 run40116110147 completed successfully. Current-source original-data GPU capture, simulator original-game execution and physical iPhone/Backbone validation are NOT_TESTED. Synthetic hosted graphics probes must remain separately labelled. No user-facing IPA is authorized before complete-Aranos release gates pass.

Primary classifications GAME_SPECIFIC for native mechanics; SHARED_TESTING for test/CI work; overlay wrapper is GAME_SPECIFIC analysis tooling. No Shared Core promotion/adoption/version update; no other engine adoption/review forced. Any future generic PS2 overlay recovery promotion requires its own governance and cross-title validation.

Current run40144142983 ALL3JOBS PASS: Intel native compile, ARM64 unsigned app/package/matched symbols, headless/overlay/synthetic Metal and iOS27 batched/unbatched startup/controller/resize/AA/shadow validation. Hosted originals report NOT_TESTED without the private ISO; the10 original-data passes above are local, explicitly opted-in tests.

| Current feature | Implemented | Native app compiled | Original-data CPU exercised | Original-data GPU / device |
|---|---|---|---|---|
| Fall6 landing-lag steering gate |Yes|Intel/macOS and ARM64|Yes, full supported hero chain fixture + negative control|NOT_TESTED|
| Class132d state0 initialization |Yes|Intel/macOS and ARM64|Yes, actual owner/pvars/RNG through world initialization|NOT_TESTED|
| Class132d state1 query/draw |No|No implementation|Unsupported is explicitly asserted|NOT_TESTED|
| Class1330 state0 |No|No implementation|Actual next unsupported callback asserted|NOT_TESTED|
| Generated PS2Recomp C++ |Analysis material only|Not linked into app|Not executed|NOT_TESTED|

Internal IPA SHA256 `07bd7d50ab8ddd7bb2cdfff0579bfbb0fb24f55dacee19123494620f2189d697`; ARM64 executable/dSYM UUID `9355aa34-3708-3106-a056-700f00a03e41`. Effective bundleorg.rerac.game.iospoc preserved. No user-facing package delivery: complete-Aranos release gates remain false.
