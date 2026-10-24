# Full-game development checkpoint — internal only

Aranos and the campaign remain **INCOMPLETE**. No new user-facing IPA is authorized until every non-hardware complete-Aranos gate passes. Production RC2 Play remains disabled. This report supersedes historical feature-IPA offers, which are retained only as rollback evidence.

Started local `a21594d6251d1a535719dfac18d763177f1c8f96` / public `df9b97c24b3d18266686e7b2e68770731e7d3401`. Current code local `928f0ed94232c4a4658306845b9f987a513c329a` / public `c8c578520025c21c4495543a0913b2f009992e3e`. Local/public histories remain distinct and preserved; public source was published with parent/expected-head checks, not a wholesale merge or reset.

## Engineering advanced

Original GC melee timing/input and initial wrench ownership/create gates now use original tables and item10/class0x47. Native hand attachment uses the original attachment list and shared partial-pose/matrix operations. Target scoring follows GC's range-weighted crate penalty and conditional health filter, rather than RAC1's differing rules. Five-sample line sweep, GC hit delivery/filter and loaded-class allocation adapters are native CPU-tested. Shared math, animation, object storage/allocator and loaded-header initializer are reused; no RAC1 title scheduler or class behavior is invoked.

Normal gravity-frame scoring and normal attached-hand transforms are the supported contracts. First-person/gravity-frame targeting, flight and complete item switching are explicitly unsupported. The services are not connected into a complete combat tick, and hero melee states13/14/15 remain unported. No wrench hit, enemy death, pickup, checkpoint or completion in the running app is claimed.

Read-only scan of all27 original file slots records4561 included classes,318 external class references,1050 sections and54 audio/scene dependencies. One real cross-level incompatibility was corrected: Oozla class0xa8c uses the original GC old-format conversion. All4561 included class records now parse without errors. This is structural inventory, not execution of27 levels. Slot21's original internal header id is30.

## Evidence and limits

- Local formats179pass/1ignored; RAC1 game715pass; iOS contracts59pass.
- RAC1 original ISO:19levels and2968 class outputs identical to preserved parser baseline4a86d31. The missing old extracted-directory check had zero coverage and was rejected, then replaced by this read-only ISO comparison.
- RC2 units18pass; original-data integration tests8pass. Includes2187 character pose frames,126 paired wrench/hero poses and attachment matrices, native creation/reuse/ownership gates, original floor sweep, raised-floor/jump/follow-camera and menu PCM checks. These are CPU tests. Optional extracted-tree format fixtures are unavailable; their skip messages are retained, with original-ISO regressions executed separately.
- Hosted run40089826366, source95bf206: ARM64/Intel, package/symbol verification, headless regressions, synthetic Metal and iOS27 simulator checks PASS. Source c8c5785's internal validation run40092030462 is pending at this checkpoint. CI contains no original game assets.
- A verified internal unsigned package/dSYM exists for95bf206. It is **INTERNAL_EXPERIMENTAL**, not a user delivery. Current service-source package validation is pending.
- Ratchet/Aranos rendering, Backbone walking and introductory audio remain prior user-reported physical baseline evidence. No new original-data GPU capture or physical iPhone test was performed for this checkpoint. Full-level playthrough/performance acceptance remains untested.

## Reuse compatibility

| System | Classification | Reproducible evidence / missing work |
|---|---|---|
| Native renderer and input | REUSED_WITH_ADAPTER | Preserved upstream/iOS baseline; previous user device evidence; current synthetic CI |
| Mesh/resource readers | REUSED_WITH_ADAPTER | Original Aranos/Oozla tests and27-slot deep scan |
| Skeleton/animation/math | REUSED_WITH_ADAPTER | Original compressed poses and126 hand transforms through existing native evaluators |
| Object storage/allocator/loaded initializer | REUSED_WITH_ADAPTER | Original GC class0x47 create/delete/reuse tests; no RAC1 class scheduler |
| Pad record, low-level vector/matrix helpers | REUSED_UNCHANGED | Existing native modules and local/hosted regressions |
| Movement/camera/collision | REUSED_WITH_ADAPTER | Adopted upstream GC semantics plus original rigid placement floor/body/jump tests; full route missing |
| Wrench, target scoring and hit records | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | Native kernels pass; full tick, spatial physics and receivers not connected |
| Class behavior/enemies/combat | BLOCKED | Original class initialization, live target list, update order and receiver callbacks not ported |
| HUD, level audio, scripts, checkpoints, death, completion | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | Required acceptance remains open; no full-level execution claim |
| Full campaign | NOT_YET_TESTED |27 slots inventoried only |

First development dependency: `GC_FULL_HERO_ITEM_TICK_MELEE_PHYSICS_LIVE_TARGET_LIST_AND_RECEIVER_CALLBACKS_UNCONNECTED`. FIRST_FATAL_BLOCKER is null for executed tests; this does not certify unexecuted original-data GPU startup.

Next: original GC hero/item tick and full melee physics, native live target list, wrench delivery and crate/enemy damage callbacks. Then continue required gameplay/audio/HUD/scripts/recovery and legitimate progression. GAME_SPECIFIC / SHARED_TESTING; no Shared Core promotion, installed pointer change or other-engine adoption. Original/private assets and RAC1 saves/app identity/rollback remain preserved. See structured receipt `rc2-full-game-internal.json` for exact scope.


The GC pvar path now confirms header0x58/0x64 link/relative lists and shared-data0x54 against the original loader. Aranos fixtures cover21 moby links and357 bounded local reads under an explicit test-only spawn map. All27 slots contain3070 moby-link records,22863 relative-pointer records and1739 shared-pvar fields over1632 shared bytes. The native plan applies shared relocation after local relocation and tags pointer kind; it never writes original or host pointers into payloads. Actual owner allocation, spawn decisions and class callbacks are not implemented by this plan. RAC1 pvar/shared outputs match the pre-refactor reader over19 original levels and29784 block comparisons with two explicit masks (464 shared bytes); no gameplay execution is claimed.

Full-level frame timing, memory and stability: NOT_TESTED. Current source files and exact evidence are listed in the structured receipt. Next loader work is concrete: static spawn loop0x3025e0..0x302ad0; pvar/link/shared passes0x302bc0..0x302ff0; loader initialization calls0x32e290 at0x30534c. Do not mistake that initialization pass for the unverified main gameplay scheduler. GC enemy class0xb0a/0xb0b additionally uses a state byte at moby+0x95, distinct from the common+0x20; retain it in an explicit GC adapter.
