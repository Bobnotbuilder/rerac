# Aranos initializer dependency batches

GAME_SPECIFIC analysis using the existing World loader and original owned ISO. The explicit save/RNG fixture produces 442 scheduled initializer owners, 51 callback groups and 86 direct services shared between those groups. This is dependency inventory, not proof of original New Game inputs or callback support. Source order, class parameters, pvars, RNG and object lifetime remain mandatory.

Reproduce: set `RC2_ORIGINAL_ISO` and `RC2_INITIALIZER_INVENTORY` to private paths, execute the original native-world test, then run `python3 tools/recovery/rc2_initializer_inventory.py <world-metadata> <original-call-relationships> <private-report>`.

The next batch after object24/class1330 is shared crate callback39c870, beginning owner25: classes1f4/1f5/1ff. Reuse RAC1 crate/stack infrastructure with GC adapters; GC mask05830001 and expanded pvars+100 differ. Its original stack initialization39d0c0 and physics39dfd8 cannot be replaced with a state change.

| Source | Owners | Classes | Direct call sites | Native evidence scope |
| --- | ---: | --- | ---: | --- |
| null | 42 | 0000, 09f5, 0a2a, 0a33, 0a34, 0e4e, 138b | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 00383b90 | 1 | 12f5 | 1 | COMPLETE_DELETE_CALLBACK |
| 00383bb0 | 3 | 12f9 | 4 | STATE0_LINK_WAIT_SUPPORTED_STATE2_EFFECTS_MISSING |
| 00383ce0 | 18 | 12fa | 9 | STATE0_TYPE89_RESOURCE_DEPENDENT_LATER_DESTRUCTION_MISSING |
| 00383eb8 | 1 | 132d | 3 | STATE0_SUPPORTED_STATE1_QUERY_AND_DRAW_MISSING |
| 003846d0 | 1 | 1330 | 46 | STATE0_PREFIX_ONLY_TELEPORT_AND_TAIL_REQUIRED |
| 0039c870 | 56 | 01f4, 01f5, 01ff | 36 | STATE0_SHARED_ADAPTER_ORIGINAL_WORLD_TESTED_PENDING_SAVE36_NOT_GLOBAL_CURSOR_ADVANCE |
| 0039f588 | 1 | 02a7 | 14 | UNKNOWN_REQUIRES_EVIDENCE |
| 0039fbd8 | 4 | 0325 | 8 | UNKNOWN_REQUIRES_EVIDENCE |
| 003a2030 | 1 | 0340 | 4 | UNKNOWN_REQUIRES_EVIDENCE |
| 003b9868 | 8 | 0919 | 7 | UNKNOWN_REQUIRES_EVIDENCE |
| 003bb120 | 13 | 099c | 57 | UNKNOWN_REQUIRES_EVIDENCE |
| 003c3be8 | 2 | 09c7 | 24 | UNKNOWN_REQUIRES_EVIDENCE |
| 003c4358 | 6 | 0a38 | 3 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ce5b0 | 4 | 0ac0 | 28 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ced30 | 1 | 0ac1 | 49 | UNKNOWN_REQUIRES_EVIDENCE |
| 003cf6c8 | 1 | 0ac2 | 52 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d00a0 | 2 | 0ac3 | 5 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d01a0 | 4 | 0ac4 | 32 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d0888 | 2 | 0ac5 | 6 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d0980 | 2 | 0ac6 | 6 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d0a60 | 3 | 0ac7 | 20 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d0e00 | 1 | 0ac8 | 7 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d0f50 | 7 | 0ac9, 0c0a, 0c0b, 0c0c, 0c0d, 0e01, 0e02 | 24 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d1458 | 1 | 0aca | 50 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d1d60 | 9 | 0adb | 58 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d6900 | 14 | 0b0a | 106 | UNKNOWN_REQUIRES_EVIDENCE |
| 003d9020 | 31 | 0b0b | 171 | UNKNOWN_REQUIRES_EVIDENCE |
| 003dead0 | 1 | 0b51 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 003e8be8 | 1 | 0c22 | 28 | UNKNOWN_REQUIRES_EVIDENCE |
| 003eb888 | 55 | 0c4c | 2 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ec140 | 7 | 0c5c | 16 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ec378 | 7 | 0c63 | 3 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ec540 | 2 | 0c66 | 13 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ec838 | 1 | 0c8b | 1 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ec880 | 11 | 0c8c | 6 | UNKNOWN_REQUIRES_EVIDENCE |
| 003ef708 | 4 | 0cda | 9 | UNKNOWN_REQUIRES_EVIDENCE |
| 003f45e0 | 21 | 0ce9 | 9 | UNKNOWN_REQUIRES_EVIDENCE |
| 003fa808 | 51 | 0d1e | 6 | UNKNOWN_REQUIRES_EVIDENCE |
| 003fefc0 | 2 | 0ddf | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0040eb48 | 1 | 0f13 | 24 | UNKNOWN_REQUIRES_EVIDENCE |
| 0040ffa0 | 3 | 0f6e | 30 | UNKNOWN_REQUIRES_EVIDENCE |
| 00414368 | 27 | 1143 | 9 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b058 | 1 | 0b41 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b080 | 1 | 0b42 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b0a8 | 2 | 0b45 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b0d0 | 1 | 0b4d | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b0f8 | 1 | 0b52 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b120 | 1 | 0b53 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b148 | 1 | 0c02 | 0 | UNKNOWN_REQUIRES_EVIDENCE |
| 0041b170 | 1 | 0c3f | 0 | UNKNOWN_REQUIRES_EVIDENCE |

The graph retains named external symbols such as DPRINT without inventing addresses. Direct call-site order is static address order, not an executed branch trace. Indirect dispatch and inferred function boundaries remain incomplete. The registry third word remains opaque. Null update identity does not establish absence of all loader services. Private reports contain complete ordered owner references and shared-service groups.

Current batch receipt: all ten group19 state0 bodies execute through real World owners with three correct linked stacks and nine RNG draws. Broader independent-service probe completes25–35, then36 waits original mission/save word at39d3d0. Actual initializer remains24. See current regression/handoff; carrier, icon, path and state1/destruction remain mandatory.
