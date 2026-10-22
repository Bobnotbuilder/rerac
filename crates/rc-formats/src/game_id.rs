//! Shared disc identity, not evidence of native game support.
//! Identity tables moved from rc-extract/build_db.rs (ReRAC ISC licence).
//! Identification, verified extraction and native execution are separate gates.
/// Which game a disc is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Game {
    Rac1,
    Rac2,
    Rac3,
    Deadlocked,
    Unknown,
}

impl Game {
    /// Id used in the contract (`"game":"rac1"`).
    pub fn id(self) -> &'static str {
        match self { Game::Rac1 => "rac1", Game::Rac2 => "rac2", Game::Rac3 => "rac3", Game::Deadlocked => "racdl", Game::Unknown => "unknown" }
    }
    pub fn title(self) -> &'static str {
        match self {
            Game::Rac1 => "Ratchet & Clank",
            Game::Rac2 => "Ratchet & Clank: Going Commando",
            Game::Rac3 => "Ratchet & Clank: Up Your Arsenal",
            Game::Deadlocked => "Ratchet: Deadlocked",
            Game::Unknown => "",
        }
    }
}

/// A known product serial (the boot ELF's file name).
#[derive(Clone, Copy, Debug)]
pub struct Title {
    pub serial: &'static str,
    pub game: Game,
    /// Free text: which edition (from disc_layout.md §2.6).
    pub edition: &'static str,
}

const fn t(serial: &'static str, game: Game, edition: &'static str) -> Title { Title { serial, game, edition } }

pub const TITLES: &[Title] = &[
    t("SCUS_971.99", Game::Rac1, "original / Greatest Hits"),
    t("SCUS_972.09", Game::Rac1, "demo 1"),
    t("SCUS_972.40", Game::Rac1, "demo 2"),
    t("SCES_509.16", Game::Rac1, "Black Label / Platinum"),
    t("SCED_510.75", Game::Rac1, "demo"),
    t("SCPS_150.37", Game::Rac1, "original"),
    t("SCPS_150.56", Game::Rac2, ""), t("SCES_516.07", Game::Rac2, ""), t("SCUS_972.68", Game::Rac2, ""),
    t("SCUS_973.22", Game::Rac2, ""), t("SCUS_973.23", Game::Rac2, ""), t("SCUS_973.74", Game::Rac2, ""),
    t("SCKA_200.11", Game::Rac2, ""),
    t("PAPX_905.20", Game::Rac3, ""), t("SCED_528.47", Game::Rac3, ""), t("SCED_528.48", Game::Rac3, ""),
    t("SCES_524.56", Game::Rac3, ""), t("SCPS_150.84", Game::Rac3, ""), t("SCUS_973.53", Game::Rac3, ""),
    t("SCUS_974.11", Game::Rac3, ""), t("SCUS_974.13", Game::Rac3, ""), t("TCES_524.56", Game::Rac3, ""),
    t("SCKA_200.37", Game::Rac3, ""),
    t("PCPX_980.17", Game::Deadlocked, ""), t("SCED_536.60", Game::Deadlocked, ""), t("SCES_532.85", Game::Deadlocked, ""),
    t("SCPS_150.99", Game::Deadlocked, ""), t("SCPS_151.00", Game::Deadlocked, ""), t("SCUS_974.65", Game::Deadlocked, ""),
    t("SCUS_974.85", Game::Deadlocked, ""), t("SCUS_974.87", Game::Deadlocked, ""), t("SCKA_200.60", Game::Deadlocked, ""),
];

/// Lowercase with everything but `[a-z0-9]` removed: `SCUS-97199`, `SCUS_971.99` and `scus97199` compare equal.
pub fn normalize_serial(s: &str) -> String { s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect() }

pub fn title(serial: &str) -> Option<&'static Title> {
    let n = normalize_serial(serial);
    TITLES.iter().find(|t| normalize_serial(t.serial) == n)
}

/// Region from the product-code prefix (`SCUS` → NTSC-U, `SCES`/`SCED` → PAL, `SCPS` → NTSC-J, `SCKA` → NTSC-K).
pub fn region_of(serial: &str) -> Option<&'static str> {
    let p: String = serial.chars().take(4).collect::<String>().to_ascii_uppercase();
    Some(match p.as_str() {
        "SCUS" | "SLUS" => "NTSC-U",
        "SCES" | "SCED" | "SLES" | "TCES" => "PAL",
        "SCPS" | "SLPS" | "SLPM" | "PAPX" | "PCPX" => "NTSC-J",
        "SCKA" | "SLKA" => "NTSC-K",
        _ => return None,
    })
}

/// When the serial is unknown: the game from the boot ELF's strings, in the order of disc_layout.md §2.6
/// (the sequels also contain "Ratchet & Clank").
pub fn game_from_elf(elf: &[u8]) -> Game {
    let has = |s: &str| elf.windows(s.len()).any(|w| w == s.as_bytes());
    if has("Deadlocked") { Game::Deadlocked }
    else if has("Up Your Arsenal") { Game::Rac3 }
    else if has("Going Commando") { Game::Rac2 }
    else if has("Ratchet & Clank") { Game::Rac1 }
    else { Game::Unknown }
}

/// Compiled native modules, not a promise that every build/region is supported.
pub fn native_module_available(game: Game) -> bool { game == Game::Rac1 }

/// Runtime routing: never feed another game's archive into RAC1's native systems.
/// Legacy RAC1 extraction manifests do not have a `game` field; keep accepting
/// them only when the existing supported retail serial matches exactly.
pub fn require_native_game(serial: &str, declared_game: Option<&str>) -> Result<Game, &'static str> {
    let game = title(serial).map(|t| t.game).ok_or("GAME_IDENTITY_UNKNOWN")?;
    if declared_game.is_some_and(|id| id != game.id()) {
        return Err("GAME_MANIFEST_IDENTITY_MISMATCH");
    }
    if !native_module_available(game) { return Err("NATIVE_GAME_MODULE_NOT_IMPLEMENTED"); }
    if normalize_serial(serial) != "scus97199" { return Err("NATIVE_GAME_BUILD_NOT_SUPPORTED"); }
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_rac1_routes_without_changing_identity() {
        for serial in ["SCUS_971.99", "SCUS-97199", "scus97199"] {
            assert_eq!(require_native_game(serial, None), Ok(Game::Rac1));
            assert_eq!(require_native_game(serial, Some("rac1")), Ok(Game::Rac1));
        }
    }
    #[test]
    fn every_other_known_disc_is_not_a_rac1_native_module() {
        for entry in TITLES {
            if normalize_serial(entry.serial) == "scus97199" { continue; }
            assert!(require_native_game(entry.serial, None).is_err(), "{}", entry.serial);
        }
    }
    #[test]
    fn rac2_can_be_named_but_cannot_launch_or_inherit_rac1() {
        let entries: Vec<_> = TITLES.iter().filter(|t| t.game == Game::Rac2).collect();
        assert_eq!(entries.len(), 7);
        for t in entries {
            assert_eq!(require_native_game(t.serial, Some("rac2")), Err("NATIVE_GAME_MODULE_NOT_IMPLEMENTED"));
            assert_eq!(require_native_game(t.serial, Some("rac1")), Err("GAME_MANIFEST_IDENTITY_MISMATCH"));
        }
    }
    #[test]
    fn unknown_and_mismatched_manifests_fail_closed() {
        assert_eq!(require_native_game("UNKNOWN", None), Err("GAME_IDENTITY_UNKNOWN"));
        assert_eq!(require_native_game("SCUS_971.99", Some("rac2")), Err("GAME_MANIFEST_IDENTITY_MISMATCH"));
        assert_eq!(require_native_game("SCES_509.16", Some("rac1")), Err("NATIVE_GAME_BUILD_NOT_SUPPORTED"));
    }
}
