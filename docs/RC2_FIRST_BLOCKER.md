# First current blocker — 25 October 2026

**GC_ARANOS_CLASS_1330_SOURCE_3846D0_INITIALIZATION_SERVICES_UNPORTED**

Original-data CPU initialization fixture now stops at runtime object24, class0x1330, source callback0x3846d0, state0, initializing=true. It previously stopped at object23/class132d; that class's CPU initialization now executes successfully using its original owned descriptor and one world RNG. Initialization must not skip object24 to claim a complete world tick.

Reproduction: set RC2_ORIGINAL_ISO to the private USA v2.00 ISO and run rc-game2's `original_gc_static_world_spawn_ownership_and_first_native_callback_boundary` original-data integration test with --nocapture. World input is an explicit synthetic progress/save bank/RNG fixture (467spawned/5rejected,28preceding owners), not a captured New Game state. Initial seed at132d boundaryd26e35d0 becomesd795a3b6. Descriptor is copied to actual owner23; state1/update distance255. The same initializer cursor then reports object24. This diagnostic is a development blocker, not a newly observed physical-device crash; FIRST_FATAL_BLOCKER for current native device gameplay remains null/NOT_EXECUTED.

Next executable step: reconstruct class1330's required state0 dependencies, starting with byte1a7a12 query2f51a0, conditional322730(0x18), halfword152c26 service368560, camera table use from pvars+4, original class0xd54 allocation311320 and owner connection32a748. Continue through its remaining original table/pose/script effects before enabling the callback. Reuse native object/animation/camera services through GC adapters; do not create a replacement actor or assume an enemy/mission meaning.

Separately known missing: class132d state1 projection/query383fc0 and queued draw3842f8; original New Game/save/RNG context; remaining class callbacks/full world readiness; full original Aranos update gates/order; hero/item tick and wrench13/14/15 spatial hits; actual health/damage/enemy/crate logic, bolts/pickups/interactions/dynamic behavior; HUD, level audio/dialogue/effects/cutscenes/scripts/checkpoints/saving/death/respawn/completion/transition. Existing extracted27 slots do not establish27 playable levels.

Native CI compilation and synthetic graphics do not remove these original-data gameplay blockers. Current correction has not run on a physical iPhone. No incomplete IPA release; production RC2 Play disabled; Aranos and full game INCOMPLETE.
