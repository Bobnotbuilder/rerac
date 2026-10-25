# Native iOS integration — 25 October 2026

Verified follow-up: existing GC Hero::idle_seq now implements original2a4e48(argument0), which teleport's2af848 also requires. Original mode==1 returns0 (RAC1 differs); hp==1 selects51hex; separate byte18b2b2 selects67hex; otherwise0. The byte is NOT aliased to the sign-extended native surface id at18b2b0. Surface-reaction writers and nonzero weapon arguments remain unported. Source-hash-pinned original-data fixture passes health1 -> actual state0 entry -> original sequence51 ->111joint pose -> next animation/physics/collision tick. Damage was not fabricated to set this fixture.

Latest local code b993164bcba298b884e997c94a45e4cbb7fcd895 / publica4c188bf9ca2aa4e96143103adc9a363539bf180. GCunits35PASS and originalCPU10PASS. RAC1 original ISO checks rerun:19levels/2968 identical class outputs,29784 pvar-block comparisons/464 shared bytes; read-only parsing, not R1 gameplay execution. Native run40154498317 pending; queued intermediate40153555598 was replaced/cancelled before execution, not passed. Active prefix run40153085562 is retained separately. No current device/originalGPU/fulllevel claim; cursor24 remains blocked.


## Class1330 recovery continuation — current, 25 October 2026

**INCOMPLETE: initializer cursor remains object24.** The immediate acceptance criterion of advancing past24 is NOT achieved. First required service now narrowed to **GC_ARANOS_CLASS_1330_SOURCE_2B9E68_HERO_TELEPORT_SERVICES_UNPORTED**. Local source `db2d9f12ff17c1476b70bb117d27c1b50d84355a` / published `2abbf8bdbc42e4f90eba33bb3ee46b3f61a8fea4`; start local7925c78 / publicd1d38d0. Native run40153555598 pending; prior prefix run40153085562 separate. No claim of original-game GPU or physical-device execution.

The original-data fixture executes2f51a0 → conditional322730(18hex) →368560 through the existing World with explicit runtime globals. Retries preserve completed prefix effects without replaying them or consuming RNG. Actual owned pvars+4=40 selects gameplay header+68's **cuboid40**, not a camera. Its position is(247,193.16782,49.89,w=.01), Euler(0,-0,1.5707964,0), state argument0, camera-reset argument1. The required teleport is retained as a typed pending request, never treated as completed.

Native32a748 owner connection and360cd0/360d28 allocation/lifetime services reuse the same MobyTable, actual pvars, animation/matrix and grid. Original-data tests separately exercise original classd54 and nine source-word0 attachment allocations, replacement/deletion+2tick reuse rules, flag/order/pvar aliasing and no RNG draws. These are service integration tests, **not an out-of-order initializer bypass**. Full384d28 poses, scripts and callback tail remain required.

Motion service2b2cd8 is implemented through the existing GC Hero and shared PadState. Its source-hash-pinned test runs after original animation-driven movement, resets exact motion/history fields without clearing buttons/sequence, then executes the next original-animation/physics/collision tick. Two formerly absent vector blocks and W lanes have explicit Hero-owned storage; unidentified block189ec0 retains its address name. Full2b9e68 remains unported; this method is not invoked as a substitute teleport by the engine.

Tests executed: RAC1 units715PASS, formats179PASS/1optional fixture ignored, GCunits34PASS, original-data CPU10PASS, iOScontracts59PASS, recoveryPython13PASS plus nested native AOT harness6PASS. Original PS2 execution traces, complete world tick, current original-data GPU, physical iPhone/Backbone and complete Aranos acceptance remain untested/incomplete. Internal native build checks continue; no user-facing IPA or enabled production RC2 Play.

Reuse audit and automatic compatibility/AOT/callback grouping: [RC2_REUSE_COMPATIBILITY.md](RC2_REUSE_COMPATIBILITY.md). Fixes GAME_SPECIFIC; genuinely reusable validation SHARED_TESTING candidate only. No Shared Core promotion, other-engine adoption, R1 implementation/save or app identity changes.

Next executable step: Bind original2b9e68 to the existing GC Hero/Pad/World owners. Reuse the now implemented2b2cd8 reset; recover/apply324140 tracked-object teardown,2af848 item put-away,2a53b8 modifier reset,2c4560 state0 mode2,2d2760 animation,2b4a08 probe,3e0b28/403d98 optional actor effects,2da160 camera reset and313008 lighting/grid. Preserve original order, then execute311320(d54),32a748,table73,360d28 attachments,script gates and384c40/384d28 tail before advancing cursor24. Keep132d state1 query383fc0/queued3842f8 separate.

## Previous checkpoint evidence (historical)

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
