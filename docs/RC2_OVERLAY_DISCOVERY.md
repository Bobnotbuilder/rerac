# RC2 executable overlays and recovery workflow

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
