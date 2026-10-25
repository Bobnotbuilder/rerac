# First current blocker — 25 October 2026

**GC_ARANOS_CLASS_1330_SOURCE_2B9E68_HERO_TELEPORT_SERVICES_UNPORTED**

The object24/class1330/source3846d0 state-zero initializer has **not** completed or advanced. The verified World-owned prefix executes2f51a0, conditional322730(18hex),368560 and resolves actual cuboid40. The next original call2b9e68 requires the real shared Hero/Pad/object/item/trail/joint/camera/audio/lighting integration, not position-only teleport or a camera-table adapter. The motion dependency2b2cd8 is now implemented/tested, but the enclosing chain is not.

Local source `db2d9f12ff17c1476b70bb117d27c1b50d84355a` / published `2abbf8bdbc42e4f90eba33bb3ee46b3f61a8fea4`; start local7925c78 / publicd1d38d0. Native run40153555598 pending; prior prefix run40153085562 separate. No claim of original-game GPU or physical-device execution.

Reproduce with private RC2_ORIGINAL_ISO and `cargo test -p rc-game2 --test original_aranos original_gc_static_world_spawn_ownership_and_first_native_callback_boundary -- --nocapture`. Source-bound World loader uses explicit synthetic progress/save/runtime/RNG inputs, not a captured New Game. Same cursor24 remains state0, animation retained, world.tick returns NotInitialized. Owner pvars+4=40, +14=73; cuboid-table loader303210..3032a4 establishes header68/stride80. Missing runtime input state remains an additional production dependency; no New Game zero globals inferred.

Required continuation: Bind original2b9e68 to the existing GC Hero/Pad/World owners. Reuse the now implemented2b2cd8 reset; recover/apply324140 tracked-object teardown,2af848 item put-away,2a53b8 modifier reset,2c4560 state0 mode2,2d2760 animation,2b4a08 probe,3e0b28/403d98 optional actor effects,2da160 camera reset and313008 lighting/grid. Preserve original order, then execute311320(d54),32a748,table73,360d28 attachments,script gates and384c40/384d28 tail before advancing cursor24. Keep132d state1 query383fc0/queued3842f8 separate.

Native actor connection/attachment allocator and motion-reset service tests operate on actual owners/original resources but do not certify full1330. Source-word0 and nonzero query input are explicit branch fixtures, not verified equipment/save state. No original actor joint0 pose, class script completion or hero/item damage chain is invented.

Separate unresolved work: class132d state-one query383fc0 and queued3842f8; authentic New Game/global/save/RNG context; full original380c50 driver gates; hero/items/wrench spatial hits/health receivers; enemies/crates/bolts/pickups/interactions; moving/posed collision; HUD/music/dialogue/effects/cutscenes/scripts/checkpoints/saving/death/respawn; legitimate completion/transition; current full original-data GPU and physical iPhone acceptance. Extracted27 slots are not27 playable levels.

FIRST_FATAL_BLOCKER remains null/NOT_EXECUTED on current device gameplay. Current host discovers only the Mac, no iPhone. This is a development blocker, not a newly observed crash. No complete Aranos claim, no incomplete user-facing IPA, production RC2 Play disabled.
