//! Experimental upstream RC2 data route. Never authorizes RC2 in the RAC1 importer.
//! GAME_SPECIFIC / PLATFORM_SPECIFIC: separate, explicit opt-in and save namespace.
use anyhow::{Context, Result, ensure};
use std::path::Path;
#[cfg(target_os = "ios")]
use std::path::PathBuf;

pub const REQUEST: &[u8] = b"RC2_NATIVE_ARANOS_EXPERIMENTAL_V1\n";
pub const MARKER: &str = "ENABLE_NATIVE_RC2.txt";

pub fn validate(root: &Path) -> Result<()> {
    ensure!(std::fs::read(root.join(MARKER)).context("RC2 experimental opt-in required")? == REQUEST,
        "invalid RC2 experimental request");
    ensure!(!root.join("toc.bin").exists(), "RC2 data must not share a RAC1 data folder");
    let manifest = std::fs::read_to_string(root.join("extract-info.json"))?;
    // Identity and data_format are checked by disc_source, before this function.
    ensure!(manifest.len() < 64 * 1024, "RC2 extraction manifest too large");
    let boot = std::fs::read(root.join("SCUS_972.68"))?;
    ensure!(rc_formats::sha1::hex(&rc_formats::sha1::sha1(&boot)) == "4124a79c1e5a1ceb56f514df0dbbc11f072af56f",
        "RC2 original executable identity mismatch: only NTSC-U v2.00 is validated");
    let hdr = std::fs::read(root.join("RC2.HDR"))?;
    rc_formats::rac2::toc::Toc::parse(&hdr)?;
    for rel in ["G/LEVEL0.WAD", "G/ARMOR.WAD", "G/MISC.WAD", "G/HUD.WAD", "G/MPEG.WAD", "G/MPEG1.WAD", "G/MPEG2.WAD"] {
        ensure!(std::fs::metadata(root.join(rel)).with_context(|| format!("RC2 missing original resource {rel}"))?.len() > 0,
            "RC2 empty resource {rel}");
    }
    Ok(())
}

#[cfg(target_os = "ios")]
pub fn requested_ios_root() -> Result<Option<PathBuf>> {
    let home = std::env::var_os("HOME").context("iOS sandbox HOME unavailable")?;
    let root = PathBuf::from(home).join("Documents/RC2Native");
    match std::fs::metadata(root.join(MARKER)) {
        Ok(_) => Ok(Some(root)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn experimental_request_never_authorizes_a_rac1_folder() {
        let root=std::env::temp_dir().join(format!("rerac-rc2-native-gate-{}",std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let error=validate(&root).unwrap_err().to_string();assert!(error.contains("opt-in"));
        std::fs::write(root.join(MARKER),b"ARANOS_TERRAIN_PREVIEW_V1\n").unwrap();
        assert!(validate(&root).unwrap_err().to_string().contains("invalid RC2"));
        std::fs::write(root.join(MARKER),REQUEST).unwrap();
        std::fs::write(root.join("toc.bin"),b"RAC1").unwrap();
        assert!(validate(&root).unwrap_err().to_string().contains("RAC1 data folder"));
        assert_eq!(std::fs::read(root.join("toc.bin")).unwrap(),b"RAC1");
        assert!(!root.join("memcard").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
