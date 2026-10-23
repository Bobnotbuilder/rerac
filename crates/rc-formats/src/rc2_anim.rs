//! GC sequence storage, independently decoded from the bounded file layout.
//! This is NOT an animation evaluator. In particular, special-frame channel
//! arithmetic and the original GC pose/state transitions remain unverified.
//! Provenance and limits: docs/formats/rc2_character_assets.md.
use crate::{buf::{invalid, Result}, moby_anim::{parse_sequence, MobySequence}, Buf};

#[derive(Debug)]
pub struct SpecialFrame<'a> {
    pub header_words: [u16; 2],
    pub channels: [&'a [u8]; 3],
    pub joint_flags: &'a [u8],
    pub record_deltas: [&'a [u8]; 2],
}

#[derive(Debug)]
pub struct SpecialSequence<'a> {
    pub base_joint_data: &'a [u8],
    pub base_records: [&'a [u8]; 2],
    pub triggers: Vec<u32>,
    pub trigger_data: Option<&'a [u8]>,
    pub frames: Vec<SpecialFrame<'a>>,
}

#[derive(Debug)]
pub enum Sequence<'a> { Regular(MobySequence), Special(SpecialSequence<'a>) }

fn delta_end(b: Buf<'_>, mut at: usize, records: usize) -> Result<usize> {
    for _ in 0..records {
        let flags = b.u8(at)?; at += 1;
        for shift in [0, 2, 4] {
            let n = ((flags >> shift) & 3) as usize;
            let n = if n == 3 { 0 } else { n };
            b.check(at, n, "GC special record delta")?; at += n;
        }
    }
    Ok(at)
}

/// One exact sequence blob; frame offsets are relative to this blob. Regular
/// storage reuses the existing bounded reader, never the RAC1 evaluator.
pub fn parse(blob: &[u8], joints: usize) -> Result<Sequence<'_>> {
    if joints > 111 { return invalid("GC_ANIM_JOINT_LIMIT"); }
    let b = Buf(blob); b.check(0, 0x1c, "GC sequence header")?;
    let frames = b.u8(0x10)? as usize;
    if frames == 0 { return invalid("GC_ANIM_NO_FRAMES"); }
    let pointers: Vec<u32> = b.pod_slice(0x1c, frames, "GC frame offsets")?;
    if pointers.iter().all(|p| p >> 28 == 0) { return parse_sequence(blob, 0).map(Sequence::Regular); }
    if pointers.iter().any(|p| p >> 28 != 15) { return invalid("GC_ANIM_MIXED_OR_UNKNOWN_FRAME_FLAGS"); }
    let triggers_at = 0x1c + frames*4;
    let triggers: Vec<u32> = b.pod_slice(triggers_at, b.u8(0x12)? as usize, "GC animation triggers")?;
    let shared = triggers_at + triggers.len()*4;
    let packed = b.u32(shared)?;
    let split = [4, 4+(packed & 1023) as usize, 4+((packed >> 10)&2047) as usize, 4+(packed >> 21) as usize];
    if !split.windows(2).all(|w| w[0] <= w[1]) { return invalid("GC_ANIM_CHANNEL_OFFSETS"); }
    let base_joint_data = b.sub(shared+4, joints*6, "GC special base joints")?.bytes();
    let at = shared+4+joints*6;
    let counts = [b.u8(at)? as usize, b.u8(at+1)? as usize];
    let records_at = at+2;
    let base_records = [b.sub(records_at, counts[0]*8, "GC special base records 0")?.bytes(),
        b.sub(records_at+counts[0]*8, counts[1]*8, "GC special base records 1")?.bytes()];
    let shared_end = records_at+(counts[0]+counts[1])*8;
    let tp = b.u32(0x14)? as usize;
    let trigger_data = if tp == 0 { None } else { Some(b.sub(tp, 32, "GC sequence-relative trigger data")?.bytes()) };
    let offsets: Vec<usize> = pointers.iter().map(|p| (p & 0x0fff_ffff) as usize).collect();
    let mut out = Vec::new();
    for &offset in &offsets {
        if offset < shared_end { return invalid("GC_ANIM_FRAME_OVERLAPS_SHARED"); }
        let end = offsets.iter().copied().filter(|v| *v > offset).min().unwrap_or(blob.len());
        let f = b.sub(offset, end.checked_sub(offset).ok_or_else(|| crate::FormatError::Invalid("GC_ANIM_FRAME_RANGE".into()))?, "GC special frame")?;
        let channels = [f.sub(split[0],split[1]-split[0],"GC animation channel 0")?.bytes(),
            f.sub(split[1],split[2]-split[1],"GC animation channel 1")?.bytes(),
            f.sub(split[2],split[3]-split[2],"GC animation channel 2")?.bytes()];
        let joint_flags = f.sub(split[3], joints.div_ceil(8), "GC animation joint flags")?.bytes();
        let a = split[3]+joint_flags.len(); let c = delta_end(f,a,counts[0])?; let d = delta_end(f,c,counts[1])?;
        out.push(SpecialFrame { header_words: [f.u16(0)?,f.u16(2)?], channels, joint_flags,
            record_deltas: [f.sub(a,c-a,"GC deltas 0")?.bytes(),f.sub(c,d-c,"GC deltas 1")?.bytes()] });
    }
    Ok(Sequence::Special(SpecialSequence { base_joint_data, base_records, triggers, trigger_data, frames: out }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn special_ranges_are_bounded_without_assuming_pose_math() {
        let mut b = vec![0u8; 96]; b[0x10]=1;
        b[0x1c..0x20].copy_from_slice(&0xf0000040u32.to_le_bytes());
        b[0x20..0x24].copy_from_slice(&(2u32 | (4<<10) | (6<<21)).to_le_bytes());
        b[0x44..0x4a].copy_from_slice(&[1,2,3,4,5,6]);
        let Sequence::Special(s)=parse(&b,1).unwrap() else { panic!() };
        assert_eq!(s.frames[0].channels, [&[1,2][..],&[3,4][..],&[5,6][..]]);
        assert_eq!(s.base_joint_data.len(),6);
        assert!(parse(&b[..0x4a],1).is_err());
        b[0x1f]=0x80; assert!(parse(&b,1).is_err());
    }
    #[test]
    fn variable_record_deltas_reject_truncation() {
        // widths 1,2,0 for the three stored components; high two flag bits retained.
        let b = [0xc9,10,20,30];
        assert_eq!(delta_end(Buf(&b),0,1).unwrap(),4);
        assert!(delta_end(Buf(&b[..3]),0,1).is_err());
        assert!(parse(&[],111).is_err()); assert!(parse(&[0;32],112).is_err());
    }
}
