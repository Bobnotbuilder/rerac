//! A Going Commando gameplay file (the level file's gameplay part, WAD-compressed; `rac2::level::Part::Gameplay`):
//! 40 header words of section offsets (0 = absent), then the sections. The order is R&C1's
//! (`crate::gameplay::SECTION_NAMES`) with sections added; the records that the shared readers take are R&C1's or
//! map to R&C1's structures here (docs/plan/rac2_bringup.md §6.1).
//!
//! **How each name is known** (Aranos, level 0; [`SECTIONS`]): *content* = the payload has R&C1's layout or carries
//! counts that match the level (e.g. the occlusion mappings hold 833 tfrags and 2000 ties, the core's counts);
//! *order* = only its place in R&C1's order says what it is (to confirm in R&C2's level loader).

use crate::buf::{invalid, Buf, Result};
use crate::gameplay::MobyInstance;
use crate::shrub::{self, ShrubInstance};
use crate::tie::TieInstance;

/// Header words (0x28 × 4 = 0xa0 bytes; the last is 0 on every level seen).
pub const HEADER_WORDS: usize = 40;

/// `(name, known by)` per header word.
pub const SECTIONS: [(&str, &str); HEADER_WORDS] = [
    ("level_settings", "content"),
    ("directional_lights", "content"),
    ("cameras", "content"),
    ("sound_instances", "content"),
    ("help_us_english", "content"),
    ("help_uk_english", "content"),
    ("help_french", "content"),
    ("help_german", "content"),
    ("help_spanish", "content"),
    ("help_italian", "content"),
    ("help_japanese", "content"),
    ("help_korean", "content"),
    ("tie_classes", "content"),
    ("tie_instances", "content"),
    // New in R&C2: a count (0x1b on Aranos) and 0x800-byte records.
    ("tie_groups", "order"),
    ("shrub_classes", "content"),
    ("shrub_instances", "content"),
    // New in R&C2: a count (1 on Aranos).
    ("shrub_groups", "order"),
    ("moby_classes", "content"),
    ("moby_instances", "content"),
    ("moby_groups", "order"),
    // L00 0x302f40..0x302fec, including a signed high16 field offset.
    ("shared_data", "content"),
    // L00 0x302d30..0x302da0: pairs remap instance ids through the s16 map.
    ("pvar_moby_links", "content"),
    ("pvar_table", "content"),
    ("pvar_data", "order"),
    // L00 0x302da8..0x302e0c: pairs add the copied pvar block base.
    ("pvar_pointer_fixups", "content"),
    ("cuboids", "content"),
    ("spheres", "order"),
    ("cylinders", "order"),
    ("pills", "order"),
    ("paths", "content"),
    ("grind_paths", "order"),
    ("unknown_32", "order"),
    ("unknown_33", "order"),
    ("unknown_34", "order"),
    ("unknown_35", "order"),
    ("occlusion_mappings", "content"),
    // New in R&C2: RGBA5551 colours for the ties' vertices (the colour indices of R&C1's tie packets are gone).
    ("tie_colors", "content"),
    ("unknown_38", "order"),
    ("unused_39", "order"),
];

/// The index of a named section in [`SECTIONS`].
pub fn index(name: &str) -> Option<usize> { SECTIONS.iter().position(|s| s.0 == name) }

/// The present sections as (name, byte range) in file order: each runs to the next section's start.
pub fn sections(gameplay: &[u8]) -> Result<Vec<(&'static str, std::ops::Range<usize>)>> {
    let g = Buf(gameplay);
    let mut starts = Vec::new();
    for (i, (name, _)) in SECTIONS.iter().enumerate() {
        let o = g.i32(4 * i)?;
        if o > 0 { starts.push((o as usize, *name)); }
    }
    starts.sort();
    let mut out = Vec::with_capacity(starts.len());
    for (k, &(start, name)) in starts.iter().enumerate() {
        let end = starts.get(k + 1).map_or(gameplay.len(), |s| s.0);
        if end > gameplay.len() { return invalid(format!("gameplay section {name} runs past the end")); }
        out.push((name, start..end));
    }
    Ok(out)
}

/// One named section's bytes, `None` when absent.
pub fn section<'a>(gameplay: &'a [u8], name: &str) -> Result<Option<&'a [u8]>> {
    Ok(sections(gameplay)?.into_iter().find(|s| s.0 == name).map(|(_, r)| &gameplay[r]))
}

/// The level-settings section (R&C1's fields where they are the same; header word 0).
pub fn level_settings(gameplay: &[u8]) -> Result<Option<&[u8]>> { section(gameplay, "level_settings") }

/// Original GC pvar bytes referenced by an instance, before relocation or any
/// class initializer. The table is content-identified (header word 23); GC's
/// section positions differ from RAC1. Never run RAC1 spawn/fixup code here.
pub fn unrelocated_pvar(gameplay: &[u8], index: i32) -> Result<Option<&[u8]>> {
    if index == -1 { return Ok(None); }
    if index < -1 { return invalid("invalid GC pvar index"); }
    let table = section(gameplay,"pvar_table")?.ok_or_else(|| crate::buf::FormatError::Invalid("GC instance references an absent pvar table".into()))?;
    let table=Buf(table);
    let pos=(index as usize).checked_mul(8).ok_or_else(|| crate::buf::FormatError::Invalid("GC pvar index overflow".into()))?;
    let (offset,size)=(table.i32(pos)?,table.i32(pos+4)?);
    if offset<0 || size<0 { return invalid("negative GC pvar extent"); }
    let data=section(gameplay,"pvar_data")?.ok_or_else(|| crate::buf::FormatError::Invalid("GC instance references absent pvar bytes".into()))?;
    Ok(Some(Buf(data).sub(offset as usize,size as usize,"GC original pvar")?.bytes()))
}

/// Original GC pair lists, with each terminator bounded to its own section.
/// Header offsets 0x58 / 0x64 are confirmed by L00 0x302d30 / 0x302da8;
/// never pass this gameplay file to RAC1's public section/layout reader.
pub fn pvar_fixups(gameplay:&[u8])->Result<crate::gameplay::PvarFixups> {
    let ranges=sections(gameplay)?;
    let list=|name:&str,header:usize|->Result<Vec<(i32,i32)>> {
        let Some((_,range))=ranges.iter().find(|(n,_)|*n==name) else {return Ok(Vec::new());};
        crate::gameplay::fixup_list(Buf(&gameplay[..range.end]),header)
    };
    Ok(crate::gameplay::PvarFixups {moby_links:list("pvar_moby_links",0x58)?,pointers:list("pvar_pointer_fixups",0x64)?})
}

#[derive(Clone,Debug,Default)]
pub struct SharedPvars {pub data:Vec<u8>,pub fields:Vec<(u16,i16,i32)>}

/// GC shared pvar blob at header0x54. Reuses the generic byte/record readers,
/// bounded to this section. GC reads the high half as signed (`0x302fb8`),
/// whereas RAC1's public reader retains its existing u16 field representation.
pub fn shared_pvars(gameplay:&[u8])->Result<SharedPvars> {
    let ranges=sections(gameplay)?;
    let Some((_,range))=ranges.iter().find(|(n,_)|*n=="shared_data") else {return Ok(SharedPvars::default());};
    let bounded=&gameplay[..range.end];
    Ok(SharedPvars {data:crate::gameplay::pvar_shared_data_at(bounded,0x54)?,
        fields:crate::gameplay::pvar_shared_records_at(bounded,0x54)?.into_iter().map(|(i,o,d)|(i,o as i16,d)).collect()})
}

/// R&C2's tie instance (0x60 bytes): R&C1's record without the 64 per-light-slot colours (0x50..0xd0), so
/// `{o_class, draw distance, 0, occlusion index, matrix, directional lights, uid, 0, 0}`.
const TIE_INSTANCE: usize = 0x60;

/// The tie instances (`s32 count`, 12 bytes pad, `count` × 0x60), as R&C1's structure with no slot colours.
pub fn tie_instances(gameplay: &[u8]) -> Result<Vec<TieInstance>> {
    let Some(s) = section(gameplay, "tie_instances")? else { return Ok(Vec::new()) };
    let b = Buf(s);
    let count = b.i32(0)?;
    if !(0..=100_000).contains(&count) { return invalid("implausible tie instance count"); }
    (0..count as usize)
        .map(|i| {
            let o = 0x10 + i * TIE_INSTANCE;
            let r = b.sub(o, TIE_INSTANCE, "tie instance")?;
            Ok(TieInstance {
                o_class: r.i32(0)?,
                draw_distance: r.i32(4)?,
                pad_8: r.i32(8)?,
                occlusion_index: r.i32(0xc)?,
                matrix: r.pod(0x10, "tie matrix")?,
                ambient_rgbas: [0; 64],
                directional_lights: r.i32(0x50)?,
                uid: r.i32(0x54)?,
                pad_d8: r.i32(0x58)?,
                pad_dc: r.i32(0x5c)?,
            })
        })
        .collect()
}

/// The shrub instances: R&C1's 0x70-byte record, unchanged.
pub fn shrub_instances(gameplay: &[u8]) -> Result<Vec<ShrubInstance>> {
    match section(gameplay, "shrub_instances")? { Some(s) => shrub::parse_shrub_instance_section(s), None => Ok(Vec::new()) }
}

/// R&C2's moby instance (0x88 bytes): R&C1's 0x78-byte record with four words inserted at 0x10 (after the spawn
/// id; their meaning is not mapped yet), so the class is at 0x28 and the position at 0x40.
pub const MOBY_INSTANCE: usize = 0x88;

/// A moby instance: R&C1's fields, and the four words R&C2 adds.
#[derive(Clone, Copy, Debug)]
pub struct Rac2MobyInstance {
    pub base: MobyInstance,
    pub extra: [i32; 4],
}

/// The static moby instances (`s32 count, s32 spawnable, pad[2]`, then records stepped by their size word).
pub fn moby_instances(gameplay: &[u8]) -> Result<Vec<Rac2MobyInstance>> {
    let Some(s) = section(gameplay, "moby_instances")? else { return Ok(Vec::new()) };
    let b = Buf(s);
    let count = b.i32(0)?;
    if !(0..=10_000).contains(&count) { return invalid("implausible moby instance count"); }
    let mut out = Vec::with_capacity(count as usize);
    let mut p = 0x10;
    for i in 0..count as usize {
        let size = b.i32(p)?;
        if size as usize != MOBY_INSTANCE { return invalid(format!("moby instance {i}: size {size:#x}, expected 0x88")); }
        // R&C1's record is this one without bytes 0x10..0x20.
        let mut rec = Vec::with_capacity(crate::gameplay::MOBY_INSTANCE_SIZE);
        rec.extend_from_slice(b.sub(p, 0x10, "moby instance")?.bytes());
        rec.extend_from_slice(b.sub(p + 0x20, MOBY_INSTANCE - 0x20, "moby instance")?.bytes());
        let mut base: MobyInstance = Buf(&rec).pod(0, "moby instance")?;
        base.size = crate::gameplay::MOBY_INSTANCE_SIZE as i32;
        out.push(Rac2MobyInstance { base, extra: [b.i32(p + 0x10)?, b.i32(p + 0x14)?, b.i32(p + 0x18)?, b.i32(p + 0x1c)?] });
        p += MOBY_INSTANCE;
    }
    Ok(out)
}

#[cfg(test)]
mod pvar_tests {
    use super::*;
    fn fixture()->Vec<u8> {
        let mut data=vec![0;0xb4];
        data[23*4..24*4].copy_from_slice(&0xa0u32.to_le_bytes());
        data[24*4..25*4].copy_from_slice(&0xb0u32.to_le_bytes());
        data[0xa4..0xa8].copy_from_slice(&4u32.to_le_bytes());
        data[0xb0..].copy_from_slice(&[1,2,3,4]);data
    }
    #[test]
    fn gc_pvars_use_gc_section_positions_and_original_unrelocated_bytes() {
        let data=fixture();assert_eq!(unrelocated_pvar(&data,0).unwrap(),Some(&[1,2,3,4][..]));
        assert_eq!(unrelocated_pvar(&data,-1).unwrap(),None);
        assert!(unrelocated_pvar(&data,-2).is_err());
    }
    #[test]
    fn bad_gc_pvar_extents_and_indices_are_rejected() {
        let mut data=fixture();assert!(unrelocated_pvar(&data,2).is_err());
        data[0xa4..0xa8].copy_from_slice(&5u32.to_le_bytes());assert!(unrelocated_pvar(&data,0).is_err());
        data[0xa4..0xa8].copy_from_slice(&(-1i32).to_le_bytes());assert!(unrelocated_pvar(&data,0).is_err());
        data[23*4..24*4].fill(0);assert!(unrelocated_pvar(&data,0).is_err());
    }
    #[test]
    fn gc_fixups_require_a_terminator_inside_their_own_section() {
        let mut data=vec![0;0xc0];
        data[22*4..23*4].copy_from_slice(&0xa0u32.to_le_bytes());
        data[25*4..26*4].copy_from_slice(&0xb0u32.to_le_bytes());
        data[0xa0..0xa4].copy_from_slice(&2i32.to_le_bytes());
        data[0xa4..0xa8].copy_from_slice(&8i32.to_le_bytes());
        data[0xa8..0xac].copy_from_slice(&(-1i32).to_le_bytes());
        data[0xb0..0xb4].copy_from_slice(&(-1i32).to_le_bytes());
        let f=pvar_fixups(&data).unwrap();assert_eq!(f.moby_links,vec![(2,8)]);assert!(f.pointers.is_empty());
        // A terminator in the next section must not rescue the first list.
        data[0xa8..0xac].fill(0);assert!(pvar_fixups(&data).is_err());
    }
    #[test]
    fn gc_shared_field_offset_is_signed_and_records_are_section_bounded() {
        let mut data=vec![0;0xbc];
        data[21*4..22*4].copy_from_slice(&0xa0u32.to_le_bytes());
        data[0xa0..0xa4].copy_from_slice(&4i32.to_le_bytes());
        data[0xa4..0xa8].copy_from_slice(&1i32.to_le_bytes());
        data[0xb0..0xb4].copy_from_slice(&[9,8,7,6]);
        data[0xb4..0xb8].copy_from_slice(&0xfffc0002u32.to_le_bytes());
        let shared=shared_pvars(&data).unwrap();assert_eq!(shared.data,[9,8,7,6]);assert_eq!(shared.fields,vec![(2,-4,0)]);
        data.pop();assert!(shared_pvars(&data).is_err());
    }
}
