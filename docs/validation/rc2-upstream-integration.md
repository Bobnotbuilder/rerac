# Build 9 upstream integration (in progress)

Classification: GAME_SPECIFIC, PLATFORM_SPECIFIC, SHARED_TESTING. No Shared Core promotion.

Pinned upstream: re-rac/rerac 89c87e98f9d8296187df2a6310c67b32144090d3.
Preserved local baseline: e15499e95aa48be7f41fb468db918b61cc58fa29; pre-integration rollback 0fb24c6a312360e594b15abc48ca52e77894d031. Published baseline: 8829f79d8c210d1294cdbb4731438314960a40b2. Integration branch: ios/rc2-upstream-integration.

This selective integration replaces the earlier independent compressed-animation blocker with the upstream implementation. It does not merge upstream's unrelated RAC1 changes. Rust/Bevy, winit's iOS lifecycle patch, native GameController input, display/thermal adaptations, bundle identity and RAC1 saves are retained. Build 8 and its converted preview assets remain separate rollback paths.

| System | Reuse classification | Evidence / limitation |
|---|---|---|
| RC2 original resource loading | REUSED_WITH_ADAPTER | Upstream rac2 format readers and LevelWad; explicit experimental identity gate in disc_source/rc2_native_data. |
| Terrain, texture, sky, moby renderer | REUSED_WITH_ADAPTER | Upstream rac2_level fills shared LoadedLevel; existing Metal pipeline retained; RC2 material/lighting parity unfinished upstream. |
| Original character and compressed animation | REUSED_WITH_ADAPTER | Upstream armor reader, compact packets, compressed frame decoder and existing pose evaluator. Device rendering not yet established by integration. |
| Movement, jump, follow camera | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | Adopted existing upstream rc-game2 implementations; no independent replacement. Native runtime testing pending. |
| Collision queries, animation player, input mapping | REUSED_UNCHANGED | Existing rc-game collision_query, RatchetAnim, engine input_map and iOS GameController plugin. Original-data validation pending. |
| Title/menu/New Game to Aranos | REUSED_WITH_ADAPTER | Upstream rac2 frontend and generic HUD layer; iOS presentation requires validation. Save-slot page absent upstream. |
| Combat, enemies, objects, scripts, HUD/audio, checkpoints/completion | NOT_YET_TESTED | Existing RAC1 implementations do not establish RC2 semantics. Upstream documents these as incomplete. Full Aranos not playable/complete. |

## Experimental data contract

The production RAC1 importer and RC2 Play gate remain unchanged. iOS opts in using a separate Documents/RC2Native folder containing the original upstream Tier 0 files, extract-info.json (game rac2, disc SCUS_972.68, data_format 1), and ENABLE_NATIVE_RC2.txt containing exactly `RC2_NATIVE_ARANOS_EXPERIMENTAL_V1` followed by newline. Only the original NTSC-U v2.00 executable hash is accepted. Required archives: RC2.HDR, SCUS_972.68, G/LEVEL0.WAD, G/ARMOR.WAD, G/MISC.WAD, G/HUD.WAD and G/MPEG{,1,2}.WAD. No ISO or private game data is committed.

RC2 settings use RC2NativeSupport; RC2 saving is disabled. Remove the RC2 request marker to return to the prior startup paths. The existing RC2ScenePreview marker still selects Build 8's preview when the native RC2 marker is absent. Do not move RAC1 saves or replace ReRACData.

## Validation status at first integration checkpoint

Implemented: selective upstream format/game2/engine routing. Local offline CPU library compilation succeeded (rc-formats, rc-game, rc-game2, rc-data). iOS packaging/controller/display contracts: 59 passed. Further CPU original-data, hosted ARM64 compilation, GPU and physical iPhone validation remain pending; none are implied by these results. This is an experimental incomplete build, not a complete-level release.

## Original-data and regression evidence

Read-only private reproduction: set RC2_ORIGINAL_ISO to the owned NTSC-U v2.00 ISO, then `cargo test -p rc-game2 --test original_aranos -- --nocapture`. The test explicitly reports NOT_TESTED when private media is absent; public CI does not contain original assets.

Both original-data integration tests passed (4.71 s): authentic boot glyphs/three original menu rows/230 GUI sprites produced 68 menu quads and the New Game request; 833 tfrags/all three triangle LODs, 351 texture entries, 74 tie classes, 2,000 tie instances, 187 moby classes and 472 placements decoded. Original armor has four textures and 111 joints; all 2,187 stored pose frames evaluated to finite native joint matrices. The adopted hero jumped 1.0508652 units, returned to original collision (59 grounded ticks after landing), moved 0.32995287 units from scripted pad input and updated the native follow camera. This checks the adopted CPU path, not GS parity, hardware controllers or GPU output.

Local regressions with available original RAC1 data: formats 175 passed/1 ignored; data 16 passed; game 715 passed (73.89 s); upstream game2 9 passed; iOS contracts 59 passed. The preserved RAC1 parser comparison found 2,968 identical populated class outputs across levels 0–18; level 19 data unavailable and not tested.

The original upstream extractor compiled unchanged and copied 96 Tier 0 files (3,807,638,923 bytes) to private storage in 213.871 s, checking the upstream size/SHA1 table. This adds the raw archives required by the upstream runtime; converted Build8 assets were not regenerated.

First fatal runtime blocker: none in CPU testing; native GPU startup not yet executed. First development dependency: hosted native engine compilation and original-data GPU validation. No physical iPhone is visible to this managed session; xctrace lists only the Mac and local CoreSimulator services reject connections. No frame time/memory/energy claim or full-level completion claim is made.

[Per-system machine-readable compatibility matrix](rc2-upstream-integration.json) separates already-adopted GC-specific player/camera implementations from missing GC combat/progression. The existing native video decoder also passed an original-data CPU test decoding the first three 512×416 frames of each logo/title movie (0xb2–0xbb), including PCM decoding; frontend audio output remains unwired.
