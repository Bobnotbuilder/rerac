//! Going Commando level containers. Independent reader; never uses RAC1's
//! level-data field offsets or interprets GC's 0x80/0x84 words as gadgets.
//! Format facts/provenance: docs/formats/rc2_native_assets.md.
use crate::buf::{invalid, Buf, Result};
use crate::level::{ClassEntry, GsRamEntry, ShrubClassEntry, TextureEntry};
use std::ops::Range;

pub const SECTOR: usize = 2048;
pub const MAX_PRIMARY: usize = 64 << 20;
pub const MAX_DECODED_CORE: usize = 64 << 20;

#[derive(Clone, Debug)]
pub struct Lump { pub name: &'static str, pub bytes: Range<usize> }

/// File-relative GC level header, not the absolute-sector RC2.HDR duplicate.
#[derive(Debug)]
pub struct LevelPlan { pub level: u32, pub lumps: Vec<Lump> }

pub fn level_plan(header: &[u8], file_bytes: usize, expected_level: u32) -> Result<LevelPlan> {
    let b = Buf(header);
    b.check(0, 0x60, "GC level header")?;
    if b.u32(0)? != 0x60 || b.u32(4)? != 0 || b.u32(8)? != expected_level {
        return invalid("RC2_LEVEL_HEADER_IDENTITY_MISMATCH");
    }
    let names = ["primary", "sound_bank", "gameplay", "occlusion", "chunk0", "chunk1", "chunk2",
                 "chunk_sound0", "chunk_sound1", "chunk_sound2"];
    let mut lumps: Vec<Lump> = Vec::new();
    for (i, name) in names.into_iter().enumerate() {
        let at = 16 + i * 8;
        let offset = b.u32(at)? as u64 * SECTOR as u64;
        let length = b.u32(at + 4)? as u64 * SECTOR as u64;
        if length == 0 {
            if offset != 0 { return invalid("RC2_EMPTY_SECTOR_RANGE_WITH_OFFSET"); }
            continue;
        }
        // Final sector padding belongs to the file allocation, not necessarily
        // its byte size. Never read that padding as another file's payload.
        let end = offset + length;
        let allocated = (file_bytes as u64).div_ceil(SECTOR as u64) * SECTOR as u64;
        if offset < SECTOR as u64 || offset >= file_bytes as u64 || end > allocated {
            return invalid(format!("RC2_SECTOR_RANGE_OUT_OF_FILE:{name}"));
        }
        let range = offset as usize..end.min(file_bytes as u64) as usize;
        if lumps.iter().any(|l| range.start < l.bytes.end && l.bytes.start < range.end) {
            return invalid("RC2_OVERLAPPING_SECTOR_RANGES");
        }
        lumps.push(Lump { name, bytes: range });
    }
    if !lumps.iter().any(|l| l.name == "primary") { return invalid("RC2_PRIMARY_MISSING"); }
    Ok(LevelPlan { level: expected_level, lumps })
}

#[derive(Debug)]
pub struct Primary<'a> {
    pub index: &'a [u8], pub gs_ram: &'a [u8], pub compressed_core: &'a [u8],
    pub overlay: &'a [u8], pub ranges: Vec<Lump>,
}

pub fn primary(bytes: &[u8]) -> Result<Primary<'_>> {
    if bytes.len() > MAX_PRIMARY { return invalid("RC2_PRIMARY_MEMORY_LIMIT"); }
    let b = Buf(bytes);
    b.check(0, 0x58, "GC primary header")?;
    let names = ["overlay", "core_index", "gs_ram", "hud_header", "hud0", "hud1", "hud2", "hud3", "hud4", "core_data", "transition_textures"];
    let mut ranges: Vec<Lump> = Vec::new();
    for (i, name) in names.into_iter().enumerate() {
        let offset = b.i32(i * 8)?;
        let size = b.i32(i * 8 + 4)?;
        if size == 0 && (offset == -1 || offset == 0) { continue; }
        if offset < 0x58 || size <= 0 { return invalid(format!("RC2_INVALID_BYTE_RANGE:{name}")); }
        let range = offset as usize..offset as usize + size as usize;
        b.check(range.start, size as usize, "GC primary range")?;
        if ranges.iter().any(|l| range.start < l.bytes.end && l.bytes.start < range.end) {
            return invalid("RC2_OVERLAPPING_PRIMARY_RANGES");
        }
        ranges.push(Lump { name, bytes: range });
    }
    let get = |name| -> Result<&[u8]> {
        let r = ranges.iter().find(|l| l.name == name).ok_or_else(|| crate::buf::FormatError::Invalid(format!("RC2_REQUIRED_RANGE_MISSING:{name}")))?;
        Ok(&bytes[r.bytes.clone()])
    };
    Ok(Primary { index: get("core_index")?, gs_ram: get("gs_ram")?,
                 compressed_core: get("core_data")?, overlay: get("overlay")?, ranges })
}

#[derive(Debug)]
pub struct Core {
    pub geometry: Range<usize>, pub sky: usize, pub collision: usize,
    pub textures_base: usize,
    pub gs: Vec<GsRamEntry>, pub resident_moby_textures: usize,
    pub mobys: Vec<ClassEntry>, pub ties: Vec<ClassEntry>, pub shrubs: Vec<ShrubClassEntry>,
    pub terrain_textures: Vec<TextureEntry>, pub moby_textures: Vec<TextureEntry>,
    pub tie_textures: Vec<TextureEntry>, pub shrub_textures: Vec<TextureEntry>,
    pub decoded_bytes: usize, pub compressed_bytes: usize,
}

pub fn core(index: &[u8], data_bytes: usize) -> Result<Core> {
    let b = Buf(index); b.check(0, 0xbc, "GC core index header")?;
    let compressed_bytes = b.u32(0x88)? as usize;
    let decoded_bytes = b.u32(0x8c)? as usize;
    if decoded_bytes != data_bytes || data_bytes > MAX_DECODED_CORE {
        return invalid("RC2_CORE_DECODED_SIZE_MISMATCH");
    }
    let data_offset = |at| -> Result<usize> {
        let v = b.i32(at)?;
        if v < 0 || v as usize >= data_bytes { return invalid("RC2_CORE_DATA_OFFSET_OUT_OF_BOUNDS"); }
        Ok(v as usize)
    };
    fn table<T: bytemuck::Pod>(b: Buf<'_>, at: usize, extra: usize) -> Result<Vec<T>> {
        let n = b.i32(at)?; let offset = b.i32(at + 4)?;
        if n < 0 || n > 65536 || extra > 65536 { return invalid("RC2_CORE_INVALID_TABLE_COUNT"); }
        let n = n as usize + extra;
        if n == 0 { return Ok(Vec::new()); }
        if offset < 0xbc { return invalid("RC2_CORE_TABLE_OVERLAPS_HEADER"); }
        b.pod_slice(offset as usize, n, "GC core table")
    }
    let stash = b.i32(0x84)?;
    if !(0..=65536).contains(&stash) { return invalid("RC2_INVALID_STASH_COUNT"); }
    let start = data_offset(8)?;
    let end = [0xc, 0x10, 0x14].into_iter().map(|at| b.u32(at).map(|v| v as usize))
        .collect::<Result<Vec<_>>>()?.into_iter().filter(|v| *v > start).min()
        .ok_or_else(|| crate::buf::FormatError::Invalid("RC2_TERRAIN_END_UNKNOWN".into()))?;
    if end > data_bytes { return invalid("RC2_TERRAIN_RANGE_OUT_OF_BOUNDS"); }
    let c = Core {
        geometry: start..end, sky: data_offset(0x10)?, collision: data_offset(0x14)?,
        textures_base: data_offset(0x60)?, gs: table(b, 0, stash as usize)?,
        resident_moby_textures: stash as usize,
        mobys: table(b, 0x18, 0)?, ties: table(b, 0x20, 0)?, shrubs: table(b, 0x28, 0)?,
        terrain_textures: table(b, 0x30, 0)?, moby_textures: table(b, 0x38, 0)?,
        tie_textures: table(b, 0x40, 0)?, shrub_textures: table(b, 0x48, 0)?,
        decoded_bytes, compressed_bytes,
    };
    for entry in c.mobys.iter().chain(&c.ties).chain(c.shrubs.iter().map(|s| &s.base)) {
        if entry.offset_in_asset_wad < 0 || entry.offset_in_asset_wad as usize >= data_bytes {
            return invalid("RC2_CLASS_DATA_OFFSET_OUT_OF_BOUNDS");
        }
    }
    Ok(c)
}

/// Observed GC moby-instance metadata. This is not a moby update scheduler or
/// permission to apply RAC1 behaviours to the matching class numbers.
#[derive(Debug)]
pub struct Object {
    pub class: i32, pub position: [f32; 3], pub rotation: [f32; 3],
    pub scale: f32, pub pvar: i32, pub mission: i32,
}

pub fn objects(gameplay: &[u8]) -> Result<Vec<Object>> {
    let b = Buf(gameplay);
    b.check(0, 0xa0, "GC gameplay pointer table")?;
    let offset = b.i32(0x4c)?;
    if offset < 0xa0 { return invalid("RC2_MOBY_INSTANCE_SECTION_MISSING"); }
    let offset = offset as usize;
    let count = b.i32(offset)?;
    if !(0..=65536).contains(&count) { return invalid("RC2_MOBY_INSTANCE_COUNT_INVALID"); }
    b.check(offset + 16, count as usize * 0x88, "GC moby instances")?;
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let at = offset + 16 + i * 0x88;
        if b.u32(at)? != 0x88 { return invalid("RC2_MOBY_INSTANCE_STRIDE_INVALID"); }
        let position = [b.f32(at+0x40)?, b.f32(at+0x44)?, b.f32(at+0x48)?];
        let rotation = [b.f32(at+0x4c)?, b.f32(at+0x50)?, b.f32(at+0x54)?];
        let scale = b.f32(at+0x2c)?;
        if !position.iter().chain(&rotation).chain([&scale]).all(|x| x.is_finite()) {
            return invalid("RC2_NONFINITE_OBJECT_TRANSFORM");
        }
        out.push(Object { class: b.i32(at+0x28)?, position, rotation, scale,
                          pvar: b.i32(at+0x68)?, mission: b.i32(at+4)? });
    }
    Ok(out)
}

/// Validate every drawn terrain reference, without the RAC1 export helper's
/// out-of-range position -> zero fallback. Keep original tfrag/LOD/strip order.
pub fn validate_terrain(tfrags: &[crate::tfrag::Tfrag], textures: usize) -> Result<[usize; 3]> {
    let mut triangles = [0; 3];
    for (fragment, t) in tfrags.iter().enumerate() {
        if !t.header.bsphere.iter().all(|v| v.is_finite()) { return invalid("RC2_NONFINITE_TFRAG_BOUND"); }
        for (lod, count) in triangles.iter_mut().enumerate() {
            for tri in crate::tfrag::tfrag_triangles(t, lod)? {
                let material = t.texture_index(tri.ad_gif as usize);
                if material < 0 || material as usize >= textures { return invalid(format!("RC2_TERRAIN_MATERIAL_MISSING:{fragment}:{material}")); }
                for v in [tri.a, tri.b, tri.c] {
                    let Some(entry) = t.vertex_info.get(v as usize) else { return invalid("RC2_TERRAIN_VERTEX_INFO_MISSING"); };
                    if entry.vertex < 0 || entry.vertex % 2 != 0 || entry.vertex as usize / 2 >= t.positions.len() {
                        return invalid("RC2_TERRAIN_POSITION_MISSING");
                    }
                    if entry.vertex as usize / 2 >= t.rgba.len() { return invalid("RC2_TERRAIN_COLOUR_MISSING"); }
                }
                *count += 1;
            }
        }
    }
    Ok(triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(v: &mut [u8], at: usize, n: u32) { v[at..at+4].copy_from_slice(&n.to_le_bytes()); }
    fn header() -> Vec<u8> {
        let mut h = vec![0; 96]; put(&mut h, 0, 96); put(&mut h, 8, 1);
        put(&mut h, 16, 1); put(&mut h, 20, 1); h
    }
    #[test] fn sectors_and_last_partial_sector() {
        assert_eq!(level_plan(&header(), 3000, 1).unwrap().lumps[0].bytes, 2048..3000);
    }
    #[test] fn wrong_level_absolute_duplicate_and_truncation_rejected() {
        let mut h = header(); assert!(level_plan(&h, 4096, 2).is_err());
        put(&mut h, 4, 1000); assert!(level_plan(&h, 4096, 1).is_err());
        assert!(level_plan(&h[..95], 4096, 1).is_err());
    }
    #[test] fn overlapping_and_escaping_sectors_rejected() {
        let mut h = header(); put(&mut h, 24, 1); put(&mut h, 28, 1);
        assert!(level_plan(&h, 4096, 1).is_err());
        put(&mut h, 24, 0); put(&mut h, 28, 0); put(&mut h, 20, u32::MAX);
        assert!(level_plan(&h, 4096, 1).is_err());
    }
    #[test] fn rac1_or_invalid_primary_rejected() {
        assert!(primary(&[0; 87]).is_err());
        assert!(primary(&[0; 88]).is_err());
        let mut v = vec![0; 256]; put(&mut v, 0, 128); put(&mut v, 4, 200);
        assert!(primary(&v).is_err());
    }
    #[test] fn core_negative_or_excessive_tables_rejected() {
        let mut i = vec![0; 188]; put(&mut i, 0x8c, 1024);
        put(&mut i, 0x14, 512); put(&mut i, 0x10, 256);
        put(&mut i, 0x18, u32::MAX); assert!(core(&i, 1024).is_err());
        put(&mut i, 0x18, 100000); assert!(core(&i, 1024).is_err());
        put(&mut i, 0x18, 1); put(&mut i, 0x1c, 188); assert!(core(&i, 1024).is_err());
    }
    #[test] fn logical_id_is_not_assumed_to_be_archive_slot() {
        let mut h = header(); put(&mut h, 8, 30);
        assert_eq!(level_plan(&h, 4096, 30).unwrap().level, 30);
        assert!(level_plan(&h, 4096, 21).is_err());
    }
    #[test] fn gc_object_offsets_and_stride_are_not_rac1() {
        let mut g = vec![0; 0xa0 + 16 + 0x88];
        put(&mut g, 0x4c, 0xa0); put(&mut g, 0xa0, 1);
        put(&mut g, 0xb0, 0x88); put(&mut g, 0xb0+0x28, 100);
        put(&mut g, 0xb0+0x2c, 1f32.to_bits());
        put(&mut g, 0xb0+0x40, 5f32.to_bits());
        let objects = objects(&g).unwrap();
        assert_eq!(objects[0].class, 100); assert_eq!(objects[0].position, [5.,0.,0.]);
        put(&mut g, 0xb0, 0x78); assert!(super::objects(&g).is_err());
        put(&mut g, 0xb0, 0x88); put(&mut g, 0xb0+0x2c, f32::NAN.to_bits());
        assert!(super::objects(&g).is_err());
    }
}
