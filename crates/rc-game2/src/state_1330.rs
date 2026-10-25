//! Storage-only adapters required by class0x1330's state-zero prefix.
//!
//! Original Aranos 0x2f51a0 reads byte0x1a7a12, 0x322730(0x18)
//! conditionally writes byte0x1a7be8 and table0x139948, and0x368560
//! writes halfword0x152c26. Their wider gameplay purpose is not inferred.
//! RAC1 GameState::unlock_planet cannot be reused unchanged: its table index
//! counts nonzero flags, whereas GC scans for the last nonzero table WORD.
//! No Default is supplied: these runtime inputs are not recovered New Game
//! values, and callers must retain their original provenance explicitly.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State1330 {
    pub byte_1a7a12: u8,
    pub byte_1a7be8: u8,
    /// Source scans28 words but can write word28 when word27 is nonzero.
    /// The final element explicitly owns that adjacent write; it is not
    /// silently dropped or wrapped into another existing native table.
    pub words_139948: [i32; 29],
    pub halfword_152c26: u16,
}

impl State1330 {
    /// 0x2f51a0, including SLTU in the return delay slot. No writes/RNG.
    pub fn query_2f51a0(&self) -> bool { self.byte_1a7a12 != 0 }

    /// Only the verified argument used by class0x1330 is exposed here.
    /// Returns the written word index for diagnostics; the source has no
    /// consumed return value. The original dead busyloop has no memory/RNG
    /// effects and is not retained as native application delay.
    pub fn service_322730_18(&mut self) -> Option<usize> {
        if self.byte_1a7be8 != 0 { return None; }
        self.byte_1a7be8 = 1;
        let index = self.words_139948[..28].iter()
            .rposition(|&word| word != 0).map_or(0, |i| i + 1);
        self.words_139948[index] = 0x18;
        Some(index)
    }

    /// 0x368560: SH1, including the store in the return delay slot.
    pub fn service_368560(&mut self) { self.halfword_152c26 = 1; }

    /// Execute only the first three recovered services, in source order.
    /// This is not a replacement for the rest of callback0x3846d0.
    pub fn initialize_prefix(&mut self) -> Option<usize> {
        let written = if self.query_2f51a0() { self.service_322730_18() } else { None };
        self.service_368560();
        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn explicit_fixture(query: u8) -> State1330 {
        State1330 { byte_1a7a12: query, byte_1a7be8: 0,
            words_139948: [0;29], halfword_152c26: 0xabcd }
    }
    #[test]
    fn zero_query_skips_flag_and_table_but_writes_halfword() {
        let mut s = explicit_fixture(0);
        assert_eq!(s.initialize_prefix(), None);
        assert_eq!(s.byte_1a7be8, 0);
        assert_eq!(s.words_139948, [0;29]);
        assert_eq!(s.halfword_152c26, 1);
    }
    #[test]
    fn nonzero_byte_is_boolean_and_existing_flag_deduplicates() {
        let mut s = explicit_fixture(0xff);
        assert!(s.query_2f51a0());
        assert_eq!(s.initialize_prefix(), Some(0));
        assert_eq!(s.words_139948[0], 0x18);
        s.words_139948[4] = -7;
        let before = s.words_139948;
        s.byte_1a7be8 = 0x80;
        assert_eq!(s.initialize_prefix(), None);
        assert_eq!(s.words_139948, before);
        assert_eq!(s.byte_1a7be8, 0x80);
    }
    #[test]
    fn table_holes_and_negative_words_use_last_nonzero_not_count() {
        let mut s = explicit_fixture(1);
        s.words_139948[0] = 3;
        s.words_139948[9] = -1;
        s.words_139948[28] = 0x7777; // Not inspected by the source scan.
        assert_eq!(s.initialize_prefix(), Some(10));
        assert_eq!(s.words_139948[10], 0x18);
        assert_eq!(s.words_139948[28], 0x7777);
        assert_eq!(s.words_139948[1], 0);
    }
    #[test]
    fn final_scanned_word_produces_adjacent_word28_write() {
        let mut s = explicit_fixture(1);
        s.words_139948[27] = 8;
        s.words_139948[28] = -9;
        assert_eq!(s.initialize_prefix(), Some(28));
        assert_eq!(s.words_139948[28], 0x18);
        assert_eq!(s.words_139948[27], 8);
        assert_eq!(s.byte_1a7be8, 1);
    }
}
