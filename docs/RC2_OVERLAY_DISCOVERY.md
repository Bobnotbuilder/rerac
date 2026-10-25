# RC2 executable overlays and recovery workflow

## Final native validation receipt — source a4c188bf9ca2aa4e96143103adc9a363539bf180

Run40154498317 passed all three jobs: current-source native Intel/macOS compilation, iPhone ARM64 compilation/internal unsigned packaging/UUID-matched symbols, headless regressions, synthetic Metal texture/depth/order and iOS27 batched/unbatched startup/controller/display/AA/shadow/resize probes. Local original-data tests use the owned ISO; hosted original-data cases lack that private input and are NOT_TESTED. No current original-data GPU scene, physical iPhone/Backbone or start-to-finish gameplay/performance validation occurred.

**Checkpointed INCOMPLETE.** Class1330 cursor24 remains blocked by full2b9e68 and later pose/script/tail services;132d state1 query/draw and dynamic actor GPU sync remain separate missing capabilities. No source callback is bypassed, no production RC2 Play, no user-facing IPA, no claim of background engineering after this checkpoint. Source implementation localb993164bcba298b884e997c94a45e4cbb7fcd895/publica4c188bf9ca2aa4e96143103adc9a363539bf180. Internal package/debug identity is retained in the structured validation receipt; it is withheld until complete-Aranos non-hardware gates pass.


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

The supplied boot pass inventories97 disc records, including27 LEVEL WADs,27 SCENE WADs and RC2.HDR, but does not process their contents. The current project already has validated RC2.HDR TOC, WAD/decompression, core, gameplay and scene readers. Preserve and reuse them; extraction is not gameplay compatibility.

The existing original G/LEVEL0.WAD has a data-part table. Lump0 is an uncompressed ratchet-executable container, not an ELF. Original overlay entry2a3b88. Its seven `{destination,size,type,entry}` records are:

| Destination | Size | Kind |
|---|---|---|
|1a7c80|997c|1 PROGBITS|
|1b1600|aa520|8 NOBITS|
|25bb80|472d0|1|
|2a2e80|b14|1|
|2a3a00|b4|1|
|2a3b00|20|1|
|2a3b80|1a51b0|1|

Values above are hex. The list terminates when a subsequent header's entry differs. Addresses overlap boot mappings; they are separate loading contexts. Boot131ae8 stays bootstrap; Aranos hero2a6dd0 and class383eb8 are overlay code. Boot-only2778 translated functions cannot establish complete level/game coverage.

Reproducible private workflow (no original assets in the repository):

```sh
python3 tools/recovery/rc2_overlay_to_elf.py PRIVATE/G/LEVEL0.WAD PRIVATE/aranos.elf
ps2_analyzer PRIVATE/aranos.elf PRIVATE/aranos.toml
# Set TOML output to a NEW private directory; low_memory_mode=true,
# output_worker_threads=1, single_file_output=false.
ps2_recomp PRIVATE/aranos.toml
```

The wrapper preserves initialized section bytes, encodes NOBITS as memory-only, retains the original entry and writes hashes/provenance. It marks only the entry-containing section executable; other initialized sections stay data. This is an explicit analysis-section classification, not recovered original ELF flags. It deliberately excludes boot dependencies and .reginfo; generated GP-relative operations still need the real boot GP1aeff0. Never launch the derived ELF as a complete game or infer GP=0 from its analysis header.

This workflow was executed on original Aranos: derived ELF2063272 bytes; second export byte-identical; analyzer/recompiler both exit0.3620 functions translated,0 decode failures,128 indirect warnings. Original-code hashing, section bytes, container extents and generated source references retained privately. It did not build or execute a PS2Recomp runtime.

Prior27-slot structural scan remains preserved; no other level overlay recompiled this session. Scene WADs and additional data tables may add dependencies: boot+one overlay is not full-game code coverage. Before other-level recovery, verify that level's section/entry/GP context and required external code/data; do not reuse Aranos addresses blindly.

Private working directory: `.work/rerac-rc2/private-analysis/ps2recomp-20261025/` outside source checkout. It holds supplied archives' inventories, extracted original boot/generated sources, derived Aranos ELF/provenance, logs and call edges. Original ISO/assets remain immutable. Public repository contains only the wrapper, tests and sanitized findings.

The documented heuristic JAL-scanner fallback was used (no Ghidra function map). Reported function boundaries, indirect fallback promotions and any SDK symbol match remain static candidates. Only source branches explicitly checked against original instructions underpin the native changes. A richer symbol/overlay loading map is still required for full-game recovery.
