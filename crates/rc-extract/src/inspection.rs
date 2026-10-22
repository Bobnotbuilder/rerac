//! Non-executing disc inspection. Never calls RAC1's TOC/archive/cache parsers
//! for another title and never writes or extracts proprietary disc bytes.
use crate::{build_db::Build, identify, json, Code, Emit, Error};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn inspect(path: &Path, builds: &[Build], emit: Emit) -> Result<String, Error> {
    let (id, iso) = identify::probe(path, builds, emit)?;
    for entry in iso.entries() {
        let end = entry.lba as u64 * 2048 + entry.size as u64;
        if end > iso.sector_count() as u64 * 2048 {
            return Err(Error::new(Code::CannotRead, "ISO filesystem extent exceeds the image"));
        }
    }
    // Stream rather than allocating a whole DVD. This hash is a dump identity,
    // not a claim that unreviewed archives or native gameplay are supported.
    let mut file = File::open(path).map_err(|e| Error::new(Code::CannotRead, e.to_string()))?;
    let before = file.metadata().map_err(|e| Error::new(Code::CannotRead, e.to_string()))?;
    let mut hash = crate::sha1::Sha1::new();
    let mut buf = [0u8; 64 * 1024];
    let mut bytes = 0u64;
    loop {
        let n = file.read(&mut buf).map_err(|e| Error::new(Code::CannotRead, e.to_string()))?;
        if n == 0 { break; }
        bytes += n as u64;
        hash.update(&buf[..n]);
    }
    let after = file.metadata().map_err(|e| Error::new(Code::CannotRead, e.to_string()))?;
    if bytes != before.len() || bytes != after.len() || before.modified().ok() != after.modified().ok() {
        return Err(Error::new(Code::VerifyFailed, "ISO changed during inspection; retry with an immutable original"));
    }
    let (files, dirs) = iso.entries().iter().fold((0u64, 0u64), |(f,d),e| {
        if e.is_directory { (f,d+1) } else { (f+1,d) }
    });
    Ok(json::object(&[
        ("type", "inspection".into()), ("schema_version", 1u32.into()),
        ("mode", "DEEP_SCAN_FILESYSTEM_ONLY".into()),
        ("game", id.game.id().into()), ("serial", id.serial.into()),
        ("region", id.region.into()), ("version", id.version.into()),
        ("image_bytes", bytes.into()), ("image_sha1", crate::sha1::hex(&hash.finish()).into()),
        ("boot_elf_sha1", crate::sha1::hex(&id.elf_sha1).into()),
        ("filesystem_files", files.into()), ("filesystem_directories", dirs.into()),
        ("rc2_hdr_present", iso.find("RC2.HDR").is_some().into()),
        ("filesystem_wads", iso.entries().iter().filter(|e| !e.is_directory && e.name.ends_with(".WAD")).count().into()),
        ("identity_verification", if id.supported { "SUPPORTED_BUILD_BOOT_HASH_MATCH" } else { "SERIAL_AND_BOOT_HASH_RECORDED_ONLY" }.into()),
        ("extraction_supported", id.supported.into()),
        ("native_module_implemented", rc_formats::game_id::native_module_available(id.game).into()),
        ("runtime_executed", false.into()), ("FIRST_FATAL_BLOCKER", json::Value::Null),
        ("KNOWN_MISSING_CAPABILITIES", json::Value::StrList(if id.game == crate::build_db::Game::Rac2 {
            vec!["RAC2_ARCHIVE_LAYOUT_AND_VERIFIED_EXTRACTION".into(),
                 "RAC2_NATIVE_GAMEPLAY_MODULE".into(), "RAC2_ASSET_CACHE_CONVERTERS".into()]
        } else if !id.supported { vec!["VERIFIED_DISC_BUILD_AND_EXTRACTION_TABLE".into()] } else { vec![] })),
    ]))
}
