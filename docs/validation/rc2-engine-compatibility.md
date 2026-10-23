# Build 9 full-engine compatibility experiment

Build 9 remains **INCOMPLETE**. The full RAC1 runtime cannot execute original RC2 unchanged. Existing native infrastructure can be reused through explicit adapters; RAC1 reconstructed title behavior is not a universal RC2 implementation. This experiment adds no second renderer and never starts the RAC1 scheduler with RC2 data.

Build8 terrain/textures, Backbone and free-camera operation on a physical iPhone are **user-reported PASS**. This phase performed native CPU tests only; it did not capture a new GPU frame, render Ratchet or test gameplay on iPhone. The preserved runtime source is `cd1311505774f52e87d006e6bb8cae9a0bbcc5ad`; local starting checkpoint is `4a86d31b361f9cc724dc759dfd44cd84250889e2`.

| System | Classification | Evidence and limits |
|---|---|---|
| native_backend | REUSED_UNCHANGED | Existing Rust/Bevy/Metal ARM64 backend; Build8 physical terrain baseline reported by user. No new GPU execution this phase. |
| native_controller_transport | REUSED_UNCHANGED | Existing GameController/Backbone transport; Build8 free-camera input physically confirmed by user. No RC2 player input mapping established. |
| rc2_level_loading | REUSED_WITH_ADAPTER | Explicit GC level/primary/core readers preserve game identity; existing bounded WAD codec reused. |
| terrain_native_structures | REUSED_WITH_ADAPTER | 833 authentic tfrags accepted; 33710/26522/26416 triangles in separate LODs; existing prepared payloads unchanged. |
| original_gs_material_parity | BLOCKED | Live original GS/VU state not correlated. Build8 material policy remains experimental. |
| pif_texture_decoding | REUSED_UNCHANGED | Existing PIF/raw-alpha decoding accepted four original armor textures with fixed hashes. |
| moby_mesh_loading | REUSED_WITH_ADAPTER | 187 populated GC classes structurally decoded; RAC1 direct parser rejects183 and accepts4 with zero vertices. Compact 16-bit headers and no RAC1 color blob. |
| hero_resource_binding | REUSED_WITH_ADAPTER | Class0 skeleton111; ARMOR.WAD armor0 mesh63 high/22 low/2 metal packets; matching header counts. No original player rendering yet. |
| hero_gpu_skinning | NOT_YET_TESTED | Native skin/mesh/skeleton structures populated; actual GC character upload and GPU pose output not exercised. |
| compressed_animation_storage | REUSED_WITH_ADAPTER | All96 resident special blobs/2187 frames bounded and preserved in original slots; shared regular reader used only for unflagged storage. |
| compressed_animation_pose | BLOCKED | RAC1 reader rejects all96 flagged blobs. RC2 special channel-to-pose evaluation and runtime sequence meanings not reconstructed; RAC1 evaluator is not called. |
| player_physics_and_states | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | RAC1 hero code is a reconstructed title state machine with RAC1 constants, IDs and animation hooks. Utilities can be shared, but original RC2 equivalence is unproven. No movement test claimed. |
| third_person_camera | NOT_YET_TESTED | Existing follow-camera architecture retained. RC2 camera parameters/state behavior not established; Build8 free camera is diagnostic. |
| collision_storage | REUSED_WITH_ADAPTER | Existing collision reader accepts12079 cells/3 hero groups/81113 triangle records including cell duplicates; GC container bounds adapter. |
| collision_query_kernels | REUSED_UNCHANGED | Existing line/sphere/capsule kernels hit original start floor z49.890625; independent horizontal-plane/segment assertion. Diagnostic dimensions, not RC2 grounded movement or original-query parity. |
| native_entity_storage | REUSED_WITH_ADAPTER | 472 genuine instances fit existing Moby/MobyTable through explicit GC metadata adapter. Update functions absent; NO_UPDATE/NO_ANIM; no RAC1 init or tick. |
| object_scheduler_and_handlers | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | GC gameplay instances have0x88 stride; RAC1 reader expects0x78 and rejects. GC mission/pvar/update semantics not mapped to RAC1 handlers. |
| weapons_combat_enemy_ai | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | No GC behavior handlers, weapon states, enemy AI or original combat traces executed. Existing RAC1 code retained as reusable infrastructure/reference. |
| sky_storage | REUSED_UNCHANGED | Existing sky reader accepts6 original shells. Original sky GPU rendering not tested. |
| sound_bank_storage | REUSED_UNCHANGED | Existing sound-bank reader accepts190 original Aranos sounds. Playback/grain semantics/scene triggers not validated. |
| music_dialogue_audio_output | NOT_YET_TESTED | Existing native audio backend retained; RC2 music/dialogue scheduling, grain semantics and playback untested. |
| hud_menus | NOT_YET_TESTED | Existing UI renderer retained; original RC2 HUD data/state/equipment mappings not integrated. |
| cutscene_playback | NOT_YET_TESTED | Existing movie/scene infrastructure retained; original GC scene formats, actor animation and transition sequencing not exercised. |
| checkpoints_saves_death_respawn | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | RAC1 save/state logic is title-specific; no RC2 save serialization or restart semantics implemented. Existing saves untouched. |
| scripted_progression_completion | RC2_SPECIFIC_IMPLEMENTATION_REQUIRED | Native original start/objectives/mission events/completion transition not reconstructed. No complete playthrough. |

Reproduce the original-input probes locally (owned NTSC-U v2.00 media required; never upload it to CI):

```sh
cargo run -p rc-trace --example rc2_engine_compatibility -- "$RC2_PRIVATE_ISO"
cargo run -p rc-data --example rc2_character_inventory -- "$RC2_PRIVATE_ISO"
cargo test -p rc-data --test rc2_character -- --ignored --nocapture
RC_EXTRACTED=/private/existing-rac1-data cargo test -p rc-game --lib
python3 -m unittest discover -s ios/tests -v
```

Local validation used byte-identical product sources in the existing offline test workspace because full-workspace dependency resolution lacks the pinned winit checkout. The format harness uses cached thiserror/bytemuck libraries directly because Cargo format dev-tests need uncached anyhow. No dependencies or assertions were replaced. Native library compilation covered rc-formats, rc-data and the full rc-game library; no current ARM64/GPU claim follows.

Results: 172 format tests pass/1 ignored;16 data tests pass;715 game unit cases pass both without and with original RAC1 data (same cases repeated);59 iOS contracts pass;the original character contract passes. Every populated class output across the19 original RAC1 level archives (2968 classes) matches the preserved Build8 reader. This protects parser behavior, not a fresh physical RAC1 playthrough.

The shared collision probe uses original class0 position metadata and valid positive-world endpoints. It hits z49.890625; sphere/capsule queries hit with explicitly diagnostic dimensions. Its plane/segment check is independent of the query kernel. An initial query crossed below the existing kernel's accepted world range and returned no hit; correcting the harness endpoints required no collision-code change. This is not grounded player movement or proof that RC2 query flags/material meanings equal RAC1.

The only populated level0 lump families are primary, sound bank, gameplay and occlusion: no secondary terrain chunk is present. Global armor and level0 audio/scene archives are distinct dependencies. This does not establish the mission's legitimate completion boundary or all scripted/cutscene/transition resources.

**First development blocker:** RC2 special-frame channel decoding into a verified native joint pose. All96 resident hero sequences use flag0xf; the RAC1 reader rejects all96. New code bounds/preserves their storage; it does not evaluate or substitute poses. `FIRST_FATAL_BLOCKER` is null because no RC2 gameplay runtime was launched.

Next: reconstruct the original GC compressed-pose calculation and runtime sequence mapping, validate joint outputs, then feed the existing moby GPU skinning/render structures. Follow with original RC2 hero/camera state adapters using the existing collision kernels, GC class handlers, HUD/audio, scripted checkpoints and genuine completion. Native entity metadata storage alone does not authorize the RAC1 scheduler.

No Build9 IPA is produced for these CPU-only changes. The preserved Build8 package remains the user-confirmed scene-preview baseline; it must not be called a playable or complete RC2 build. App identity, version/build counter, RAC1 import/save/controller/audio paths and Rust/Bevy remain unchanged.

Classifications: GAME_SPECIFIC and PLATFORM_SPECIFIC implementation; SHARED_TESTING candidate experiment. No Shared Core promotion/adoption or installed engine activation. No other-engine change is required; any later generic promotion must review PS3, PS4, Xbox360, XboxOne and PC applicability. No original media, converted payloads, instruction dumps or private logs are included in source.
