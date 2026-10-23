# RC2 terrain reference capture (development only)

Purpose: resolve **RC2_NATIVE_RENDER_STATE_AND_VU_PARITY** with real Aranos
terrain evidence. Moby/tie completion is NOT a prerequisite to terrain-only
rendering. RC2 Play remains disabled; no RAC1 gameplay dispatch for GC.

## Preserved baseline and source mapping

Local reviewed CPU boundary commit a15f9cd60f14889678c84be447bea9758277979e
was preserved as remote25157d7b2b7ff7bd7f86092ce09c226b290804f2 on
ios/rc2-native, parent a974c687c3915b417d73913e1f1f7541c7e429ad. All7
changed blob hashes match. Different snapshot ancestry required a new SHA,
not force-pushing the local materialisation. Build7 application source4c94e535
and existing RAC1 branch/data/saves remain untouched. No private inputs uploaded.

## Current evidence dependency

On the managed macOS11.7.11 Intel host, the existing PCSX2 reference-capture
binary is x86_64 with minimum macOS11.0. Cocoa launch reports a macOS service
permission failure before a VM starts. QT_QPA_PLATFORM=offscreen fails because
only cocoa is packaged. No original RC2 capture was obtained by those attempts.
Don't describe a static table scan or these failed launches as a game trace.

A user-launched Aqua reference session is required unless another approved
reference environment becomes available. No emulator/runtime fallback is added
to ReRAC. No PCSX2 code or system/security settings changed. Proprietary ISO,
firmware, states, dump textures and screenshots stay outside the source repo.

Use a separate PCSX2 -datapath profile: native-resolution software GS, uncompressed
GS dumps, Deflate savestates, MTVU off, cheats/widescreen patches off, independent
save folders and memory cards disabled. It reads the existing ISO in place.
Do not copy shared RAC1 memory cards or change the user's existing settings.

While real Aranos terrain is visible, record:

1. Screenshot + **Save Single Frame GS Dump** (default Shift-F8; Mac may need Fn).
2. A savestate at the same viewpoint (F1 or System > Save State). A nearby state
   is not automatically the exact GS-dump frame: retain frame/tick consistency
   as UNPROVEN until compared. Avoid camera movement between the captures.
3. Exact reference build, SCUS-97268 identity, original renderer/settings and
   file hashes. Keep original files unmodified. No public upload.

Local profile/capture instructions and exact paths are in the private GameStudio
.work/rerac-rc2 handoff. A fresh profile has no old slot to overwrite. Original
PCSX2/RAC1 profile and ReRAC save/imported-data directories are not used.

## Read-only capture inspection

The existing tools/trace harness now has a development-only command:

```
cargo run -p rc-trace -- inspect-rc2-gs \
  --dump '<private single-frame .gs>' --out '<NEW private report.json>'
```

It reads only modern **uncompressed .gs**, bounded128MiB total/64MiB frozen
state/16MiB transfer. The report contains metadata and the first64 packed
vertex writes: raw float bits, GS12.4 XY,24/32-bit Z, fog/ADC and observed
register state. Original transfer/vertex sequence is retained, including packets
split across transfers. Sample limit does not stop counting. NaNs are retained,
not repaired. Raw PACKED STQ inputs are **not** asserted to be latched/sanitised
RGBAQ. PRMODE/PRMODECONT and both GS contexts remain visible when observed.

`--skip-packed-vertices N` selects a later sampling window without discarding
earlier state writes or reordering any data. Each sample retains its absolute
sequence. CLI output records source byte count/SHA1 for reproducibility.
The existing iOS workflow retains all previous gates and additionally runs8
synthetic capture tests plus full rc-trace CLI compilation. No private media
is provided to hosted tests. Local standalone tests are not a full CLI build.

Frozen GS state remains opaque. Unseen registers are UNKNOWN, never zero or
RAC1 defaults. Terrain attribution remains UNPROVEN: game serial alone cannot
identify which vertices are terrain rather than sky/moby/HUD. The original
private dump remains the complete evidence; the bounded sample is not a replay.
REGLIST/legacy/compressed captures fail explicitly rather than returning false
parity. Packed-vertex counts exclude A+D vertex writes and are not triangle or
game-object counts. IMAGE data is counted/skipped, not texture-decoded here.

The output is still **private game-derived evidence**. It is never uploaded or
committed automatically. NEW output is required; existing files cannot be
overwritten. Input dumps are never changed. A failed output write may leave a
partial new file; do not treat it as an accepted comparison receipt.

Format research only: local PCSX2v2.8.2 GSDump.h/.cpp, GSRegs.h, SaveState.cpp,
GSState.cpp and CLI/settings declarations; reference-only GPL, no implementation
copied/linked. Independent format parser, no CPU/VU emulation or rasteriser.

## Required comparison before native scene integration

Static comparison is now reproducible with the opt-in source example:

```
cargo run -p rc-trace --example rc2_vu_identity -- '<owned RAC1 ISO>' '<owned GC ISO>'
```

It verifies exact boot hashes, resolves93 original upload chunks via DVP overlay
metadata and ELF PT_LOAD and compares payload bytes privately. Terrain candidate
55907 (14944bytes) and903379 (7632bytes) match exactly. The earlier comparison
of custom debug-section file ranges was invalid and withdrawn; those bytes
are not the loaded instructions. Only metadata/hashes are output or committed.
No game code executes. A matching resident program does not prove its selection
on Aranos, identical EE matrices/light/fog/GS inputs or native shader parity.
Do not substitute this static receipt for the requested dynamic capture.

| Topic | Needed original evidence | Native comparison target | Current status |
|---|---|---|---|
| Terrain attribution | Matched Aranos frame + texture/packet/fragment identity |833 cached fragments/93 materials |Not captured |
| Transform/projection | VU input positions, uploaded matrix/constants, XYZ outputs |GC scene vertex path (not RAC1 defaults) |Missing |
| LOD/morph/clipping | VU program/input/mode and ADC/GS emission |GC LOD selection / triangle order |Missing |
| Material/sampling | Patched TEX0/1/2, CLAMP, MIPTBP1/2, raw STQ/UV |Existing texture backend + GC state adapter |Missing |
| Lighting | VU input/output RGBA and light/multiplier state |Native terrain colour processing |Missing |
| Fog | VU constants and XYZF2 F + FOGCOL |Native fog calculation |Missing |
| Alpha/depth | PRIM/PRMODE, TEST, ALPHA, FRAME, ZBUF, masks/context |Native GS material/pipeline selection |Missing |
| Ordering | Original GIF path/primitive/ADC sequence |Ordered terrain submissions |Parser synthetic-tested; no game comparison |

GS dumps supply post-VU output/state, not original VU input matrices. Retain
the accompanying savestate's VU1 memory/microcode, EE RAM/hardware state and
screen capture for correspondence. Don't invent matrix addresses from RAC1.
No fixed-slot VU read is called an original GC transform oracle without proof.

Next: ingest the actual private capture, identify terrain packets against the
existing Aranos cache, build deterministic matched input/output comparisons,
then implement the narrow isolated GC scene in the existing Metal-backed
renderer. Missing object classes stay explicit; don't block terrain on them.
Do not enable Play or label a diagnostic frame as gameplay.

Classification: PLATFORM_SPECIFIC capture format/GS fields; GAME_SPECIFIC
Aranos correspondence; SHARED_TESTING/SHARED_DIAGNOSTICS review candidates only.
No Shared Core promotion/adoption. Current new parser tests are synthetic,
not real-game/original-output or GPU parity validation.
