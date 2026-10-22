//! The build DB: which disc is which, and which builds ReRAC supports.
//!
//! Two tables, like OpenGOAL's serial → ELF-hash DB (`docs/plan/launcher_extractor.md` §2.1):
//! - [`TITLES`]: every known Ratchet & Clank boot-ELF name (= product serial) of RC1 and the sequels
//!   (`docs/formats/disc_layout.md` §2.6), so any R&C disc is named even when it is refused.
//! - [`BUILDS`]: exact builds pinned by serial + boot-ELF SHA-1. Only these can be supported; each supported build
//!   carries its Tier 0 size/SHA-1 table (`data/<serial>.tsv`, hashes and sizes only, no disc bytes).
//!
//! Adding a build: run `rerac-extract identify --iso <image>` on it. For an unknown build it prints an info line
//! `new build DB row: Build { … }`; paste that row into [`BUILDS`]. Supporting it additionally needs its table
//! (`rerac-extract table --iso <image> --output data/<serial>.tsv`), `supported: true` and, for a new region, the
//! runtime's address maps.

// Shared identity metadata is also used by the native runtime's launch gate.
pub use rc_formats::game_id::{Game, Title, TITLES, normalize_serial, title, region_of, game_from_elf};

/// One exact build: serial + boot-ELF SHA-1.
#[derive(Clone, Copy, Debug)]
pub struct Build<'a> {
    pub serial: &'a str,
    /// `VER` in `SYSTEM.CNF`.
    pub version: &'a str,
    pub region: &'a str,
    pub elf_size: u64,
    /// Lowercase hex.
    pub elf_sha1: &'a str,
    /// Sectors of the reference dump (informational; files are checked by hash).
    pub image_sectors: u32,
    pub supported: bool,
    /// The Tier 0 table (`path\tsize\tsha1` lines) for supported builds.
    pub table: Option<&'a str>,
}

/// NTSC-U v1.00 reference: boot ELF and ISO hashes as recorded in docs/plan/decisions.md (2026-09-26).
pub const BUILDS: &[Build<'static>] = &[Build {
    serial: "SCUS_971.99",
    version: "1.00",
    region: "NTSC-U",
    elf_size: 1_383_028,
    elf_sha1: "72fd1de379dafd8f243dceeb3be50980f3fd3cd5",
    image_sectors: 2_057_664,
    supported: true,
    table: Some(include_str!("../data/scus_971_99.tsv")),
}];

/// The ready-to-paste [`BUILDS`] row for a build not in the DB (OpenGOAL's `log_potential_new_db_entry`).
pub fn db_row(serial: &str, version: &str, region: &str, elf_size: u64, elf_sha1: &str, image_sectors: u32) -> String {
    format!(
        "Build {{ serial: \"{serial}\", version: \"{version}\", region: \"{region}\", elf_size: {elf_size}, elf_sha1: \"{elf_sha1}\", \
         image_sectors: {image_sectors}, supported: false, table: None }},"
    )
}

/// One row of a Tier 0 table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableEntry {
    pub path: String,
    pub size: u64,
    pub sha1: [u8; 20],
}

/// Parses `path\tsize\tsha1` lines; `#` lines and blank lines are comments.
pub fn parse_table(text: &str) -> Result<Vec<TableEntry>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') { continue; }
        let mut f = line.split('\t');
        let (Some(path), Some(size), Some(sha), None) = (f.next(), f.next(), f.next(), f.next()) else {
            return Err(format!("table line {}: expected 3 tab-separated fields", i + 1));
        };
        let size = size.parse().map_err(|_| format!("table line {}: bad size", i + 1))?;
        let sha1 = crate::sha1::parse_hex(sha).ok_or_else(|| format!("table line {}: bad SHA-1", i + 1))?;
        out.push(TableEntry { path: path.to_string(), size, sha1 });
    }
    Ok(out)
}

/// Renders a table in the committed format.
pub fn format_table(serial: &str, version: &str, entries: &[TableEntry]) -> String {
    let total: u64 = entries.iter().map(|e| e.size).sum();
    let mut s = format!(
        "# ReRAC Tier 0 table: {serial} v{version}. {} files, {total} bytes.\n\
         # Sizes and SHA-1 hashes only (no disc bytes). Columns: path<TAB>size<TAB>sha1.\n\
         # Regenerate: rerac-extract table --iso <image> --output crates/rc-extract/data/<serial>.tsv\n",
        entries.len()
    );
    for e in entries { s += &format!("{}\t{}\t{}\n", e.path, e.size, crate::sha1::hex(&e.sha1)); }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serials_regions_and_games() {
        assert_eq!(title("scus97199").unwrap().serial, "SCUS_971.99");
        assert_eq!(title("SCUS-97199").unwrap().game, Game::Rac1);
        assert_eq!(title("SCUS_972.68").unwrap().game, Game::Rac2);
        assert!(title("SLUS_200.00").is_none());
        assert_eq!(region_of("SCES_509.16"), Some("PAL"));
        assert_eq!(region_of("SCPS_150.37"), Some("NTSC-J"));
        assert_eq!(region_of("XXXX_000.00"), None);
        assert_eq!(game_from_elf(b"..Ratchet & Clank: Going Commando.."), Game::Rac2);
        assert_eq!(game_from_elf(b"..Ratchet & Clank.."), Game::Rac1);
        assert_eq!(game_from_elf(b"Jak"), Game::Unknown);
        let serials: std::collections::HashSet<String> = TITLES.iter().map(|t| normalize_serial(t.serial)).collect();
        assert_eq!(serials.len(), TITLES.len(), "duplicate serial in TITLES");
    }

    #[test]
    fn builtin_table_parses_and_matches_its_build() {
        let b = &BUILDS[0];
        let table = parse_table(b.table.unwrap()).unwrap();
        assert!(table.len() > 2900, "{} rows", table.len());
        let boot = table.iter().find(|e| e.path == "boot/SCUS_971.99").unwrap();
        assert_eq!((boot.size, crate::sha1::hex(&boot.sha1)), (b.elf_size, b.elf_sha1.to_string()));
        let paths: std::collections::HashSet<&str> = table.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths.len(), table.len(), "duplicate path");
        // Hashes and sizes only: every line is exactly path, decimal size, 40 hex digits.
        for line in b.table.unwrap().lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split('\t').collect();
            assert_eq!(f.len(), 3, "{line}");
            assert!(f[0].chars().all(|c| c.is_ascii_alphanumeric() || "/_.".contains(c)), "{line}");
            assert!(f[1].chars().all(|c| c.is_ascii_digit()) && f[2].len() == 40, "{line}");
        }
    }

    #[test]
    fn table_round_trip_and_errors() {
        let e = vec![TableEntry { path: "toc.bin".into(), size: 3, sha1: crate::sha1::sha1(b"abc") }];
        assert_eq!(parse_table(&format_table("X", "1", &e)).unwrap(), e);
        assert!(parse_table("a\t1").is_err());
        assert!(parse_table("a\tx\tda39a3ee5e6b4b0d3255bfef95601890afd80709").is_err());
        assert!(parse_table("a\t1\tzz").is_err());
        assert!(db_row("SCUS_971.99", "1.01", "NTSC-U", 5, "ab", 7).starts_with("Build { serial: \"SCUS_971.99\", version: \"1.01\""));
    }
}
