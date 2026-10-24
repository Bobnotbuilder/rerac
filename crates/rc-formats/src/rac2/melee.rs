//! Aranos v2.00 melee records, read from the original level overlay.
//! L00 0x2bddd8 / 0x2ccb00 index 0x25d910 with stride 0x2c.
//! This is deliberately level-0-only: other overlays relocate their code/data.
use crate::{buf::{invalid, Buf, Result}, font::parse_overlay_sections};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeleeTiming {
    pub kind: i32,
    pub step: i32,
    pub chain_before: i32,
    pub input_ref: i32,
    pub chain_from: i32,
    pub jump_after: i32,
    pub idle_after: i32,
    pub hit_from: i32,
    pub hit_to: i32,
    pub aux24: i32,
    pub aux28: i32,
}

/// The five records referenced by the ground combo, throw and jump-attack entries.
#[derive(Clone, Debug)]
pub struct AranosMeleeTable { rows: [MeleeTiming; 5] }

impl AranosMeleeTable {
    pub fn parse(level: u32, overlay: &[u8]) -> Result<Self> {
        if level != 0 { return invalid("GC melee address contract is validated only for Aranos level 0"); }
        let sections = parse_overlay_sections(overlay)?;
        let read = |address: u32, len: usize| -> Result<&[u8]> {
            let mut found = None;
            for s in &sections {
                if s.kind == 8 { continue; }
                if let Some(offset) = address.checked_sub(s.dest) {
                    if let Some(data) = s.data.get(offset as usize..).and_then(|b| b.get(..len)) {
                        if found.is_some() { return invalid("ambiguous GC melee overlay mapping"); }
                        found = Some(data);
                    }
                }
            }
            found.ok_or_else(|| crate::buf::FormatError::Invalid(format!("unmapped GC melee resource {address:#x}")))
        };
        // Reject a different overlay/namespace rather than reading plausible unrelated words.
        for (slot, target) in [(0x2a01ac, 0x2bddd8), (0x2a03fc, 0x2c791c), (0x2a072c, 0x2ccb00)] {
            if Buf(read(slot, 4)?).u32(0)? != target { return invalid("GC Aranos melee dispatcher signature mismatch"); }
        }
        let data = Buf(read(0x25d910, 5 * 0x2c)?);
        let mut rows = Vec::with_capacity(5);
        for i in 0..5 {
            let b = data.sub(i * 0x2c, 0x2c, "GC melee record")?;
            let r = MeleeTiming {kind:b.i32(0)?,step:b.i32(4)?,chain_before:b.i32(8)?,input_ref:b.i32(12)?,chain_from:b.i32(16)?,jump_after:b.i32(20)?,idle_after:b.i32(24)?,hit_from:b.i32(28)?,hit_to:b.i32(32)?,aux24:b.i32(36)?,aux28:b.i32(40)?};
            if r.kind != [0,0,0,2,1][i] || (i<3 && r.step!=i as i32)
                || [r.chain_before,r.input_ref,r.chain_from,r.jump_after,r.idle_after,r.hit_from,r.hit_to,r.aux24,r.aux28].iter().any(|&v| !(0..=0x7fff).contains(&v)) {
                return invalid("invalid GC Aranos melee timing record");
            }
            rows.push(r);
        }
        Ok(Self { rows: rows.try_into().expect("five checked melee records") })
    }

    pub fn rows(&self) -> &[MeleeTiming; 5] { &self.rows }
}
