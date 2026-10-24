# Full-game internal checkpoint — native GC world integration

**Aranos and the campaign remain INCOMPLETE. No user-facing IPA is released.** Production RC2 Play stays disabled. All non-hardware complete-Aranos gates and the physical iPhone/Backbone start-to-finish gate remain open.

Starting implementation: local `928f0ed94232c4a4658306845b9f987a513c329a` / public `c8c578520025c21c4495543a0913b2f009992e3e`. Starting repository heads: local `23e3bf84bf7cd2f60e1c1e1f240e33fbf7ba2ee4` / public `373ff2cf25b565b790cef16e988f03050d4ca084`. Final implementation: local `0e217fc64071fd0828ac270935afc8253f5327c6` / public `d7105d3ec60bbf822c3853b242cfb77852939776`. Histories and rollback builds are preserved; publication used exact parent/expected-head checks and retained remote-only files.

## What actually advanced

The native GC world connects original static spawn decisions to consecutive runtime object IDs, owned pvar allocations, original class identities, initialization callbacks, live scheduling, hit slots and rigid collision. It reuses existing native object/particle allocators, animation evaluation, matrix and collision arithmetic; it does not invoke the RAC1 title scheduler or class handlers.

GC spawn fields now have explicit meanings. Its header-bearing and headerless classes use independent original slot banks. Accepted placements mutate the shared class-mode binding in original order. Camera/sound pvars allocate before moby pvars, actual spawn mappings relocate links, and block bytes move into their unique owners. Ambiguous owner aliases and unsupported talk registration fail explicitly. The original reserved static class0 record and excluded dynamic end-marker interval are retained.

Original-data execution under **explicit synthetic save/RNG inputs** constructs467 objects from472 placements, with28 camera/sound pvar owners. This is not a captured or certified original New Game state. The native load pass executes the original delete-only callback0x383b90, initial link-wait states of0x383bb0, and class0x12fa's attached sprite/hit-trigger states. Initialization stops at runtime object23, class0x132d, source0x383eb8. The app's internal world driver cannot advance hero input past missing initialization.

Class0x12fa's tested chain is owned pvar -> native type89 particle allocation -> attached position update -> direct test-injected hit in the actual owner's GC hit slot -> original mask check -> state2. **This is not a wrench swing, spatial attack, health deduction, complete destruction or combat.** State2 requires original debris/impact/audio calls and remains blocked. Type89 currently supports its attached no-joint/no-orientation branch only; its GPU sprite rendering is not connected. GC's allocator clears all64 reused bytes; RAC1 intentionally preserves the upper half, so that difference is adapted locally. Classes and effects share one native RNG stream.

Live rigid collision reads the actual world's transforms and spatial grid. Original placement164 becomes runtime159 after spawn filtering. Its floor query returns50.33382; a diagnostic transform change moves the hit to51.33382, and native deletion removes it from the grid and exposes the world floor49.734375. The displacement is a structural test, **not original moving-platform gameplay**. Posed primitives and original dynamic-object behavior are still missing.

The driver is connected to the existing engine FixedUpdate, object renderer, animation resources and hero/camera collision queries through internal `RC2_NATIVE_WORLD_SAVE_FIXTURE=1`. The normal baseline path remains the default. This flag deliberately identifies synthetic save/RNG inputs; it is neither original New Game initialization nor a production gameplay/release mode. No duplicate renderer was created.

## Validation actually executed

| Check | Result and limit |
|---|---|
| Local native rc-game2 build | PASS through existing offline CPU workspace, with product sources copied unchanged |
| Formats |179PASS /1ignored; unavailable optional extracted-directory fixture skip messages retained |
| RAC1 game regressions |715PASS; existing native gameplay tests, not compilation-only compatibility |
| GC units |26PASS, including spawn conditions, slot239/group ordering, state bookkeeping, GC pool reuse and shared RNG consumption |
| Original RC2 ISO |9PASS; includes real poses/movement/collision/menu PCM plus native-world receiver and live rigid-grid chains |
| iOS identity/controller/package contracts |59PASS |
| Local full engine compilation |BLOCKED by unavailable pinned winit git dependency in offline cache |
| First integration source e64a0ee |Native Intel and headless/synthetic Metal jobs PASS; ARM64/package/simulator job pending at this receipt |
| Final source d7105d3 |Internal native CI40116110147 pending; no compiled-target success inferred |
| Earlier c8c5785 |All jobs PASS in40092030462; this is the previous implementation, not final-source validation |
| Original-data GPU/current iPhone/Backbone |NOT_TESTED for this world integration |
| Full route, combat, saving, completion, performance/stability |NOT_TESTED / not implemented sufficiently to execute |

Read-only RAC1 ISO class/pvar regressions from the preceding checkpoint remain valid historical evidence:19levels/2968 identical classes and29784 identical pvar-block comparisons. Their affected RAC1 parser sources did not change here; these ISO comparisons were not rerun or misreported as new tests. All27 RC2 slots were previously inventoried structurally; no27-level gameplay claim follows.

The earlier user-reported physical baseline remains authentic Ratchet/Aranos rendering, Backbone walking and introductory audio. Current source has no new visible-graphics, hardware-action or complete-playthrough evidence. Full-level frame timing, memory and stability remain NOT_TESTED.

## Reuse compatibility

| System | Classification | Evidence / remaining boundary |
|---|---|---|
| Native object storage, particle pool and low-level math |REUSED_WITH_ADAPTER|Existing native allocators; GC-owned pvars/slot banks/delete counter/full-record particle clear |
| Collision kernels and spatial grid |REUSED_UNCHANGED|Original-data line query, moved owned transform and actual grid removal |
| Renderer, animation buffers, camera/input host |REUSED_WITH_ADAPTER|Internal driver uses existing engine resources; current original-data GPU/hardware untested |
| GC static spawn/class loader and scheduler |REUSED_WITH_ADAPTER|Native full chain under explicit input fixture; mandatory class132d blocks readiness |
| Original New Game/save context |RC2_SPECIFIC_IMPLEMENTATION_REQUIRED|Synthetic fixture must be replaced by original initialization/save semantics |
| Hero/item tick, wrench, real damage and enemies |BLOCKED|State13/14/15 and required callbacks still missing; no controllable combat claim |
| GC sprite rendering and remaining class effects |RC2_SPECIFIC_IMPLEMENTATION_REQUIRED|Native record/update branch only; required render/query/audio services missing |
| HUD/audio/scripts/checkpoints/death/completion |RC2_SPECIFIC_IMPLEMENTATION_REQUIRED|Complete original progression not executed |
| Full campaign |NOT_YET_TESTED|Prior structural27-slot inventory only |

## First blocker and exact continuation

`FIRST_FATAL_BLOCKER` is null for completed tests; this does not certify unexecuted GPU startup. The first **executed native-world initialization blocker** is `GC_ARANOS_CLASS_132D_SOURCE_383EB8_INITIALIZATION_QUERY_AND_DRAW_SERVICES_UNPORTED`. Original class0x132d requires initialization0x3840b8, query0x383fc0 and queued callback0x3842f8 (registration0x2e7718). Do not replace these with a no-op or assume RAC1's differing services are equivalent.

Next executable objective: Port Aranos class0x132d source0x383eb8 with original initialization0x3840b8, query0x383fc0 and queued draw0x3842f8; preserve native RNG order and required render/query services. Resume World.initialize at moby23; do not skip callbacks. Then finish original New Game/save inputs, hero/item tick, wrench state0x13/spatial hits and actual crate/enemy health receivers.

Additional missing capabilities include remaining required class callbacks, effect rendering/animation audio, posed collision and actual moving objects, enemies/AI/combat/weapons, crates/bolts/pickups/interactions, HUD/music/dialogue/cutscenes/scripts, checkpoints/saves/death/respawn, legitimate Aranos completion and next-level transition. The implemented live hit receiver does not satisfy those acceptance gates.

## Preservation and delivery

No user-facing IPA was generated or offered. CI packages, if generated, remain INTERNAL_EXPERIMENTAL and are not a complete-Aranos candidate. Current package and target validation are pending. Native Rust/Bevy/ARM64/Metal/CoreAudio architecture, RAC1 saved data/effective app identity, Build8 and later rollback points, original ISO and96 private original archives remain preserved. No copyrighted payload or raw disassembly was published; existing conversions were not regenerated.

Classifications: GAME_SPECIFIC / SHARED_TESTING. No Shared Core promotion, installed-engine pointer change or other-engine adoption. RAC1's generic allocators/math/collision implementation is unchanged; the engine's optional GC contract is None on RAC1. PS3/PS4/Xbox/PC have no adoption or regression claim. See `rc2-full-game-internal.json` for the15 source paths and structured evidence.
