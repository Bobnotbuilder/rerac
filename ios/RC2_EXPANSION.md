# Native Ratchet 2 expansion — build 6

User-authorised GameStudio direction, 2026-10-22: use the working ReRAC iOS
native Rust/Bevy implementation as the active expansion baseline. The separate
rac1-native-game C++/Metal port remains preserved reference work, not this runtime.
No general PS2 emulator, interpreter, JIT or on-device executable generation.

## Preserved baseline

Parent source: c3653e8780d88faca31641aab358cfcc78e2ce24, iOS build 5.
Successful build evidence: https://github.com/Bobnotbuilder/rerac/actions/runs/39207822527
The historical device-playable build 3 source and third-planet checkpoint are
recorded in README.md here. Build 6 needs fresh device regression evidence.
Same bundle org.rerac.game.iospoc, scheme ReRAC, Rust 1.99.1, Bevy/winit pins,
iOS 16.0 requested minimum, physical aarch64-apple-ios target and xcode-27 CI.
Only build numbers/assertions change from 5 to 6; no signing identity changes.
The original release workflow and ios/arm64-poc branch are not changed.

## Implemented boundary (not Ratchet 2 gameplay)

- Existing disc identity tables moved into rc-formats::game_id and re-exported
  through rc-extract::build_db, preserving its public API and build database.
- Native launch gate refuses other titles/regions and conflicting manifest game
  identities before RAC1 loading. Legacy supported RAC1 manifests still work.
- Read-only `rerac-extract inspect --iso <owned-image> --json` records disc/boot
  identity, streaming image SHA-1, filesystem counts, RC2.HDR presence and WAD
  count, support flags, null runtime blocker and explicit missing capabilities.
  A successful inspection is NOT extraction, executable analysis or gameplay.
  Existing `extract` remains fail-closed for RC2. No invented offset/layout table.
- Small Games panel in existing iOS game/import apps, at final-display resolution.
  RAC1 keeps normal automatic launch. RC2 is shown as NOT IMPLEMENTED, without a
  Play action, replacement engine anchor, asset copying or save migration.

## Input and next implementation gate

No user-owned RC2 disc was available in the current authorised Downloads search.
Upstream already recognises SCUS_972.68 and other sequel serials. Its disc format
specification describes RC2.HDR and filesystem WADs for GC, unlike RAC1's sector
1500 TOC. This remains reference-derived until checked against the actual input.
Do not run the RAC1 TOC/archive/cache parser on a sequel merely because a serial
was identified. A supplied ISO must establish exact build, real archive layout,
offsets, sizes, compression, dependencies and native gameplay requirements.

Local, private preparation (never send the input/report to CI automatically):

```sh
cargo run --release -p rc-extract -- inspect --iso "/absolute/path/to/owned-RC2.iso" --json
shasum -a 256 "/absolute/path/to/owned-RC2.iso"
```

`inspect` writes only JSON/events to stdout. Redirect privately if needed. It
does not extract assets, create caches or declare the title supported.

## iPhone data and update safety

Keep existing Files → ReRAC → ReRACData contents intact. This remains exclusively
the supported RAC1 extractor output (toc.bin, extract-info.json, boot/, levels/,
global/ and cache/ as before). Saves/settings stay ReRACSupport/memcard and
ReRACSupport/settings.json. No RC2 storage migration is introduced before a
verified converter exists. Do NOT copy RC2 resources over RAC1's data folder.

Back up ReRACSupport, sign the unsigned test IPA through the existing legitimate
Sideloadly setup, and install as an UPDATE with the same effective bundle/account.
Do not uninstall/delete app data. Local/CI packaging is unsigned; without valid
Apple signing and provisioning assets it is not a signed-installable delivery.

Test launch, Games panel open/close, Backbone menu navigation, planet-three save,
movement/camera/combat, audio, planet travel, save/load and background/resume.
Export only diagnostics manually; no private input is uploaded automatically.

## Classification / adoption

Identity/extraction: PLATFORM_SPECIFIC; title dispatch/RC2 boundary: GAME_SPECIFIC;
iOS library surface: SHARED_IOS candidate; contracts: SHARED_TESTING candidate.
No generic Shared Core component version/adoption changes are made. Other
engines must not inherit Ratchet gameplay semantics. No RC2 playability or power
improvement is claimed. Baseline render/audio/controller/simulation code remains.
