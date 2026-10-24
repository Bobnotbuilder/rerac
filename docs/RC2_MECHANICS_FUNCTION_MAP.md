# RC2 mechanics function map — 25 October 2026

Static source identity: SCUS_972.68 USA v2.00, SHA256 `64dbf0d4c527d90a2a4ceb47b23d81848b62165a6449156729ca44711aa3e3ee`, SHA1 `4124a79c1e5a1ceb56f514df0dbbc11f072af56f`. Recovery tool revision `e5636981a1c9de368f6b71e819c3dc188dfedbf4`. These addresses identify original code; the native application never executes them as guest code.

`B` means the uploaded boot run's `generated_cpp/source/`. `L` means the separately generated private `aranos-generated/`. File convention is `sub_XXXXXXXX_0xlowercase.cpp`, for example `L/sub_002BA8E0_0x2ba8e0.cpp`. Do not substitute a boot file at the same numeric address for an overlay file.

| Guest entry / source | Observed caller and site | Selected called routines | Native counterpart / status |
|---|---|---|---|
| `131ae8` B/sub_00131AE8_0x131ae8.cpp | ELF entry | `_InitSys` at131c78,11aea0 at131c80,131d98 at131c98 | SDK/bootstrap register/FPU/BSS initialization; ANALYSED_STATIC, no native gameplay change |
| `131d98` B/sub_00131D98_0x131d98.cpp |131ae8 |11fc48; indirect131db8 loads constant26ede8;131cb8;11aea0 | SDK entry wrapper; explicit constant target recovered, no mechanics identity inferred |
| `26ede8` B/sub_0026EDE8_0x26ede8.cpp |131d98 indirect handoff |2914d8,2b8a38,299cf0,298f20,287140 and others | Boot title/initialization dispatch context; not the Aranos hero routine. Boot-to-overlay loader edge still needs end-to-end recovery |
| `2a3b88` L/sub_002A3B88_0x2a3b88.cpp | Original LEVEL0 overlay header entry |330dc0 at2a411c;380c50 at2a4124; numerous other mode branches | Loaded level entry/driver, ANALYSED_STATIC subset; original mode/global gating not fully native |
| `330dc0` L/sub_00330DC0_0x330dc0.cpp |2a3b88 at2a411c and other branches;366c68;367400;445db8 |3303b8 at330dcc/330dd8 | Pad update wrapper; existing rc-game pad reused; hardware/libpad transport not executed by recovery |
| `3303b8` L/sub_003303B8_0x3303b8.cpp |330dc0 |3306b0 at3305e4;330648 | Pad sample processing wrapper; existing native input architecture retained |
| `3306b0` L/sub_003306B0_0x3306b0.cpp |3303b8 at3305e4 |2effa8,2eefd0,2ef510,2eff48 | Active-low two-byte buttons inverted to PAD+1a0/+1b0, stick normalization/history; existing rc-game::pad reused; subset corroborated, no new blanket identity claim |
| `380c50` L/sub_00380C50_0x380c50.cpp |2a3b88 at2a4124 |2f1198;313718 at380cdc;313dc0;2a6dd0 at380da4;2d9f38;37fec8;331150 | Original Aranos update branch: object activity/update before gated hero. Entire driver and its global gates remain incomplete |
| `2a6dd0` L/sub_002A6DD0_0x2a6dd0.cpp |380c50 at380da4;364260;3706f0;3716f8;375f40 |2b37e8 at2a6e60;2cb650 at2a6e90;39 other logged call sites | rc-game2::hero::hero_update is partial. Water, items, damage, sounds and other original calls still missing |
| `2b37e8` L/sub_002B37E8_0x2b37e8.cpp |2a6dd0 at2a6e60;2a65d8 |2ba8e0 at2b3900 and histories/movement helpers | Hero::input_physics_move; existing architecture reused |
| `2ba8e0` L/sub_002BA8E0_0x2ba8e0.cpp |2b37e8 at2b3900 | Fall path2b3e18, conditional2b6cd0 at2c114c,2b7c88,2b5600; jump paths also call2b6668 | Hero::state_physics / air::phys_fall; landing-lag steering gate IMPLEMENTED, COMPILED, ORIGINAL-DATA CPU TESTED |
| `2b6cd0` L/sub_002B6CD0_0x2b6cd0.cpp |2ba8e0 at2c114c/2c1cd0/2c2a04 | Air turn/acceleration and drag helpers | Hero::air_control reused. Call is now suppressed while hero+1e0 !=0 in supported fall6 branch |
| `2cb650` L/sub_002CB650_0x2cb650.cpp |2a6dd0 at2a6e90;2a65d8 |2c4560;2d2508;2cb038 and many state-specific calls | Hero::transitions partial; not all dispatch cases implemented. No correction made to timer reset merely because it initially looked suspicious: source reloads the reset timer too |
| `383eb8` L/sub_00383EB8_0x383eb8.cpp | Class132d registry, indirectly dispatched by initializer/scheduler |3840b8 at383f7c;383fc0 at383f8c;2e7718 at383fa4 | World::native_update: state0 implemented/tested, state1 explicitly BLOCKED; other switch defaults do nothing as original |
| `3840b8` L/sub_003840B8_0x3840b8.cpp |383eb8 at383f7c |31b9a0,31ba38,2efef0,2ef448,2eedb0 | Controller132d::initialize uses existing native Rng, PS2 math and VU cosine. CPU model tested, no device/original-running-game oracle |
| `383fc0` L/sub_00383FC0_0x383fc0.cpp |383eb8 at383f8c |3251e8,2eb3d0 | Projection/queued query dependency. ANALYSED_STATIC subset, UNPORTED; do not call it a depth query without further evidence |
| `3842f8` L/sub_003842F8_0x3842f8.cpp | Address queued by383eb8 through2e7718 | Draw-dependent services | UNPORTED. Initialization does not invoke it; later state1 must stop rather than queue a fake callback |
| `3846d0` L/sub_003846D0_0x3846d0.cpp | Class1330 registry, next original-data initialization callback |2f51a0,322730,368560,2b9e68,311320,32a748,360d28 and others | FIRST_CURRENT_INITIALIZATION_BLOCKER object24/state0. Full callback NOT IMPLEMENTED |

## Dataflow and confidence

**Fall6:** hero block `189e20`; signed landing lag `18a000` (+1e0); velocity `189f10`; target direction/speed `189fc0/189fd0`; displacement `189f30`; carried momentum `189f80`. At2c1140 `lh` reads lag,2c1144 branches to2c1154 if nonzero, skipping only call2b6cd0. Target selection, subsequent drag and gravity still run. At zero the existing air-control code may turn, accelerate and write momentum. Native post_move decrements lag after physics. HIGH static confidence in this branch; original-animation/collision CPU state fixture passes and old-code negative control fails. No PS2 execution capture or iPhone validation of the correction.

**Class132d state0:** sets state1 directly, update distance255; clears global1b9044, selects level0 value49 into1b26bc, copies16 bytes at GP−380 (`GP=1aeff0`, descriptor1aec70) to owned pvars+0, calls3840b8 and RETURNS. It does not fall through to state1. Global block1b9040 stores owner, initialization marker, then arrays at+10/+30/+50/+90/+d0. Native owner ids replace guest pointers. Initializer fills indices1..13, then special index0. Per-index random colour has conditional extra draws; counts=randi(30)/10+1; phase=randf(−0.95,2.4); weight=1.4·abs(cos(FastModRot(phase+1)))³ in original operation order. Native math retains existing model limitations, including possible COP1/VU adder differences; no claim of hardware float parity. Purpose of global1b26bc remains unknown. HIGH control-flow/dataflow confidence; CPU integration tested; render meaning unverified.

**Class132d state1:**383fc0 returns a float stored1b9048;2e7718 queues3842f8 with owner. Required native query/render transport absent. Repeated state1 calls return unsupported without consuming initialization RNG.

**Class1330 state0:**2f51a0 returns whether byte1a7a12 is nonzero, conditionally calls322730(18hex);368560 writes1 to halfword152c26. Optional camera record is selected by owned pvars+4 with stride80hex;311320(0xd54) allocates an original class,32a748 connects it to owner, and pvars+18 stores the result. Further table/pose/script state is required. These source observations do not establish an enemy/mission identity. Dependencies must be ported together before executing the callback; no substitute actor is created.

Private evidence includes full generated C++, original overlay sections, analyzer logs, all30,211 logged overlay call-site records (16,341 distinct pairs), and address/word consistency receipts. Direct-call graphs omit computed dispatch; an empty callers list does not imply unreachable code. No generated proprietary C++, ELF or converted payload is committed.
