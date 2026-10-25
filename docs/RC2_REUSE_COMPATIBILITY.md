# RC2 reuse and bounded AOT experiment — 25 October 2026

This is an evidence-backed engineering inventory, not a full-game compatibility claim. The original-data initializer still requires complete class0x1330 behavior; state0 prefix helpers alone do not advance that cursor. Class0x132d state1 query0x383fc0 and queued callback0x3842f8 remain separately unresolved. No user-facing IPA is authorized by these results.

## Reuse decisions

The decision labels below describe reuse. They do not replace GameStudio fix classifications. Source inspection must precede adding a GC adapter; original behavior and end-to-end validation remain required.

| Requirement | Decision | Existing source and necessary boundary |
|---|---|---|
| Native object storage/free-list ownership | REUSE_WITH_ADAPTER | `rc-game/src/moby_runtime.rs` MobyTable is already reused by `rc-game2/objects.rs`; GC defaults, class metadata, lifetime and pvar ownership stay in GC adapters. Do not instantiate a competing object world. |
| CreateMoby service0x311320 | REUSE_WITH_ADAPTER | Audit RAC1 `moby_update/services.rs::create_moby` and MobyTable::create. Existing GC allocator is the starting point; allocation alone does not reproduce owner connection, animation setup or class0xd54 initialization. |
| Owner service0x32a748 | REUSE_WITH_ADAPTER | World::connect_actor now copies original owner position/Euler/light/ambient, sets distances/visibility, rebuilds the same native matrix/grid, then clears low mode bits. Original-data CPU tests exercise the actual owner and original class0xd54. This does not implement the later pose/script calls. |
| Attachment allocation360cd0/360d28 | REUSE_WITH_ADAPTER | World::rebuild_actor_attachments uses the original signed selector tables and the existing allocator, twenty-entry lifecycle list and each actual owned pvar block; no duplicate object world. Nine original fixture allocations and replacement/deletion are CPU tested. Joint pose384d28 remains required. |
| Pose evaluation/math | REUSE_UNCHANGED | Existing `rc-formats/moby_anim` and RAC1 pose/math machinery already evaluate original GC resources in original-data tests. This decision applies to those helpers, not unverified GC class-specific pose services. |
| Renderer / native controller transport / audio backend | REUSE_UNCHANGED | Existing Bevy/Metal, GameController and native audio implementations remain the foundation. New class-specific draw, input-action or sound semantics are not implied by backend reuse. No new GPU/device evidence is produced by this analysis. |
| GC scheduling and spawn decisions | REUSE_WITH_ADAPTER | Current `rc-game2/world.rs`, `spawn.rs` and `objects.rs` preserve GC ordering and explicit original state. RAC1's group/class counts and spawn fields cannot be substituted. |
| Cuboid teleport record and script-state transitions | REUSE_WITH_ADAPTER | Original header+68 supplies cuboid40, not a camera record. Reuse RAC1 Hero::teleport architecture through a GC adapter: motion/item/trail/joint/camera/lighting services differ and remain incomplete. |
| Progress table service0x322730(0x18) | RC2_SPECIFIC_IMPLEMENTATION | `State1330` recovers last-nonzero-word indexing, including adjacent word28 write. RAC1 `GameState::unlock_planet` counts nonzero flags and differs on holes/negative values. |
| Query0x2f51a0 and store0x368560 | SELECTIVE_AOT_TRANSLATION_CANDIDATE | Exact original signatures gate an experimental build-time native Rust emitter. Exhaustive differential checks agree with maintained `State1330` hand ports. Production retains the hand port; two leaves do not justify a new compiler/runtime. |
| Idle animation selector2a4e48(0) | REUSE_WITH_ADAPTER | Existing GC Hero::idle_seq/state/animation architecture; original GCmode==1 guard and sequences51/67 differ from RAC1 mode!=0 and54/6e. Original low-health animation/pose/physics chain tested; damage and surface reaction remain unported. |
| GC combat/damage, enemies, pickups, mission completion | UNKNOWN_REQUIRES_EVIDENCE | Existing RAC1 hit/weapon/object infrastructure must be audited per original GC callback and receiver. A shared class ID or structural similarity is insufficient. |
| Checkpoints, saving, death/respawn, cinematics and level transition | UNKNOWN_REQUIRES_EVIDENCE | Preserve existing native infrastructure and save identity; original GC state/trigger dependencies and complete Aranos chains remain open. |

## Existing cross-game tooling: useful, with a strict interpretation

The preserved upstream `89c87e98f9d8296187df2a6310c67b32144090d3` tree contains `tools/ghidra/scripts/crossgame_diff.py`. The current local integration checkout has Ghidra overlay/name tooling and `rc-trace overlay-diff`, but not that upstream cross-game script. Its normalizer masks relocation fields and normalizes stack frames/scheduling. Its header's interpretation of `=` as an already-correct port must **not** be used as a semantic gate: masked globals, callees and constants still require validation. No Ghidra GUI project/function export was available for this bounded run; current `overlay_names.csv` contains only its header. The existing normalizer was used directly without starting or modifying a Ghidra project.

`tools/recovery/rc2_reuse_candidates.py` runs the upstream normalizer on the original RAC1 boot ELF and two verified Aranos leaf bodies. The real-input result is **one candidate**, RC2 `0x368560` → RAC1 `0x216b08`. The write binding differs: GC `0x152c26`, RAC1 `0x1516f0`. It stays `UNKNOWN_REQUIRES_EVIDENCE`, not a claimed semantic counterpart. No corresponding query candidate was found in the scanned RAC1 boot sections; RAC1 overlays were not scanned. Function boundaries, wider state purpose and lifetime were not independently established. This is actionable evidence to review the corresponding native state service, not permission to copy an absolute guest address.

Reproduction (all binaries and output private):

```sh
python3 tools/recovery/rc2_reuse_candidates.py \
  --normalizer "$UPSTREAM/tools/ghidra/scripts/crossgame_diff.py" \
  --rac1-elf "$RAC1_BOOT_ELF" --rc2-elf "$ARANOS_ANALYSIS_ELF" \
  --out "$PRIVATE_EVIDENCE/rc2-rac1-leaf-candidates.json"
```

The tool records input hashes and the normalizer hash, not original instruction bodies. Equality is always candidate-only, even when absolute bindings match. Tests prove hidden binding differences are retained and cannot be reported as compatibility.

## Group shared callbacks before porting more classes

`tools/recovery/rc2_callback_groups.rs` calls the existing validated `LevelWad`, `AranosClassUpdates` and GC gameplay readers. It does not introduce another WAD parser. The original existing private LEVEL0.WAD yielded:

- 472 static placements and76 distinct placement classes.
- 233 registry entries and185 distinct update addresses, including null.
- Callback `0x39c870` is shared by classes `0x1f4` (43 placements), `0x1f5` (8), `0x1ff` (10):61 original placements behind one source update. Recover its shared service path and retain authentic per-class data.
- Callback `0x3d0f50` serves7 classes with one placement each: `0xac9`, `0xc0a`..`0xc0d`, `0xe01`, `0xe02`.
- 18 placement classes use the registry fallback, which is null. This does not mean their rendering, hero behavior, collision or ownership can be skipped.

The complete private report groups all registered classes, including classes not statically placed. The registry's third word remains **opaque metadata**, not an invented init/draw/damage callback. Dynamic class requirements are separate from static placement counts. Callback grouping does not change original spawn filtering/order, RNG, pvars or lifetime.

Build using the existing offline workspace's rc-formats library (substitute its actual current hash):

```sh
rustc --edition=2021 tools/recovery/rc2_callback_groups.rs \
  --extern rc_formats="$DEPS/librc_formats-$FORMATS_HASH.rlib" \
  -L "dependency=$DEPS" -o "$PRIVATE_TOOLS/rc2-callback-groups"
"$PRIVATE_TOOLS/rc2-callback-groups" "$PRIVATE_LEVEL0_WAD" "$PRIVATE_EVIDENCE/callback-groups.json"
```

## Small native AOT pilot: measured scope

`tools/recovery/rc2_leaf_aot.py` accepts only exact hashed original bodies at `0x2f51a0` and `0x368560`. It emits fixed native Rust ahead of time: no instruction dispatch, runtime guest decoder, interpreter, JIT or emulator. Both return-delay-slot effects are included. Checked guest bindings explicitly read one byte at `0x1a7a12` or write one little-endian halfword at `0x152c26`; scratch outputs v0/v1 are exposed. Other registers, floating-point state, ownership and RNG are untouched. Missing memory bindings produce errors.

Actual original Aranos analysis ELF SHA256 `dc4962b52b7d03a2238a4f96a4c4c8c65a7a2f08b01994b22a9ed93ff75405aa` passed signature verification. Generated Rust was compiled natively and compared against the maintained `State1330` implementation across all256 byte inputs and all65,536 previous halfword values. Read/write counts, address/width bindings, scratch outputs, unchanged surrounding typed state and missing-map errors are checked. This is independent native implementation comparison plus exact source-signature review, **not** an original PS2 execution trace. The unchanged nested harness also passed on the hosted ARM64 macOS runner in40153085562 (xcode-27-arm64,1.99.1-aarch64-apple-darwin). No floating-point, calls/native-service ABI, loop/switch translation, iOS app execution or device gameplay is validated by this tiny pilot. It is not wired into the app and does not improve gameplay by itself. Hand-port retention is the appropriate production decision until a larger measured case demonstrates a correctness/engineering benefit.

```sh
python3 tools/recovery/rc2_leaf_aot.py --elf "$ARANOS_ANALYSIS_ELF" \
  --output-rust "$PRIVATE_ANALYSIS/leaf-aot-pilot.rs" \
  --report "$PRIVATE_EVIDENCE/leaf-aot-pilot.json"
python3 -m unittest discover -s tools/recovery/tests -p 'test_rc2_*.py' -v
```

Latest run:13 Python recovery contracts PASS (existing4 overlay +5 AOT +4 candidate contracts); nested native harness6 PASS (4 existing State1330 tests, exhaustive differential, missing-mapping errors). Mutating any one of the two leaves' four instructions fails closed. Proprietary inputs/generated source and reports remain private. Fix classifications: `GAME_SPECIFIC` bounded RC2 recovery/emitter; `SHARED_TESTING` candidate/differential infrastructure candidate only. No Shared Core component/version/adoption changed, and no other engine is declared upgraded.

## Additional confirmed renderer integration gap

GC NativeWorld::drive currently walks instance_to_moby (static placements) only. The source-supported D54/attachment allocations above have NOT been uploaded/rendered by this path. Existing RAC1 gameplay.rs::DynMobys/upload_dynamic and moby_render.rs dynamic slot/material/palette code are the reuse starting point (REUSE_WITH_ADAPTER), with GC draw/pose/lighting semantics requiring validation. Do not write a second renderer or claim dynamic GPU rendering from allocator tests. This is separate additional missing capability GC_DYNAMIC_ACTOR_GPU_SYNC, not a replacement for the earlier2b9e68 initializer blocker.
