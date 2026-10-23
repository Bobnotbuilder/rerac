//! GC tie geometry regions, NOT RAC1's header/packet layout. Independent reader
//! from owned-data observations and format facts pinned in rc2_native_assets.md.
//! GS slot emission, instance lighting, multipass and strip topology are NOT
//! inferred here. A class decoding successfully is not a renderable game object.
use crate::buf::{invalid, Buf, Result};

#[derive(Debug)]
pub struct Vertex {
    pub packed_xyz: [i16; 3],
    pub position: [f32; 3],
    /// Exact 16/24-byte record retained for unresolved GS slots/UV semantics.
    pub raw: Vec<u8>,
}
#[derive(Debug)]
pub struct Packet<'a> {
    pub lod: usize,
    pub header: [u8; 16],
    pub unpack: [u8; 12],
    pub strips: &'a [u8],
    pub dinky: Vec<Vertex>,
    pub fat: Vec<Vertex>,
}
#[derive(Debug)]
pub struct Class<'a> {
    pub class: i16,
    pub scale: f32,
    pub bounds: [f32; 4],
    pub material_packets: &'a [u8],
    pub packets: Vec<Packet<'a>>,
}

/// `bytes` must be an independently bounded class blob, not the entire core.
pub fn read(bytes: &[u8], expected_class: i32) -> Result<Class<'_>> {
    let b = Buf(bytes);
    b.check(0, 128, "GC tie class header")?;
    let class = b.i16(0x44)?;
    if class as i32 != expected_class {
        return invalid("RC2_TIE_CLASS_IDENTITY");
    }
    let scale = b.f32(0x40)?;
    let bounds = [b.f32(0x50)?, b.f32(0x54)?, b.f32(0x58)?, b.f32(0x5c)?];
    if !scale.is_finite() || scale <= 0. || bounds.iter().any(|v| !v.is_finite()) || bounds[3] < 0.
    {
        return invalid("RC2_TIE_NONFINITE_OR_INVALID_TRANSFORM");
    }
    let material_count = b.u8(0xf)? as usize;
    let material_at = b.u32(0x1c)? as usize;
    if material_count > 0 && material_at < 128 {
        return invalid("RC2_TIE_MATERIAL_HEADER_OVERLAP");
    }
    let material_packets = b
        .sub(material_at, material_count * 80, "GC tie material table")?
        .bytes();
    let mut packets = Vec::new();
    for lod in 0..3 {
        let table = b.u32(lod * 4)? as usize;
        let count = b.u8(0xc + lod)? as usize;
        if count > 0 && table < 128 {
            return invalid("RC2_TIE_PACKET_HEADER_OVERLAP");
        }
        b.check(table, count * 16, "GC tie packet table")?;
        for i in 0..count {
            let at = table + i * 16;
            let offset = b.i32(at)?;
            if offset < 0 {
                return invalid("RC2_TIE_NEGATIVE_PACKET_OFFSET");
            }
            let base = table
                .checked_add(offset as usize)
                .ok_or_else(|| crate::FormatError::Invalid("RC2_TIE_OFFSET_OVERFLOW".into()))?;
            if base < table + count * 16 {
                return invalid("RC2_TIE_PACKET_TABLE_OVERLAP");
            }
            b.check(base, 44, "GC tie control region")?;
            let header: [u8; 16] = bytes[at..at + 16].try_into().unwrap();
            let unpack: [u8; 12] = bytes[base + 32..base + 44].try_into().unwrap();
            let strips = b
                .sub(base + 44, unpack[3] as usize * 4, "GC tie strip region")?
                .bytes();
            let vbase = base + header[8] as usize * 16;
            let vlength = header[9] as usize * 16;
            b.check(vbase, vlength, "GC tie vertex region")?;
            if vbase < base + 44 + strips.len() {
                return invalid("RC2_TIE_VERTEX_CONTROL_OVERLAP");
            }
            if unpack[8] < 4 || (unpack[8] - 4) % 2 != 0 {
                return invalid("RC2_TIE_DINKY_COUNT");
            }
            let nd = (unpack[8] as usize - 4) / 2;
            let nf = unpack[11] as usize;
            // Vertex region is QW padded. Counts must agree with the observed
            // conversion-loop bound and fit; arbitrary leftover bytes fail.
            if nd != unpack[10] as usize {
                return invalid("RC2_TIE_DINKY_COUNT_MISMATCH");
            }
            let used = nd * 16 + nf * 24;
            if used > vlength || vlength - used >= 16 {
                return invalid("RC2_TIE_VERTEX_COUNT_MISMATCH");
            }
            let vertex = |pos: usize, record: usize, length: usize| -> Result<Vertex> {
                let packed_xyz = [b.i16(pos)?, b.i16(pos + 2)?, b.i16(pos + 4)?];
                let position = packed_xyz.map(|v| v as f32 * scale / 1024.);
                if position.iter().any(|v| !v.is_finite()) {
                    return invalid("RC2_TIE_VERTEX_NONFINITE");
                }
                Ok(Vertex {
                    packed_xyz,
                    position,
                    raw: b
                        .sub(record, length, "GC tie vertex record")?
                        .bytes()
                        .to_vec(),
                })
            };
            let dinky = (0..nd)
                .map(|j| vertex(vbase + j * 16, vbase + j * 16, 16))
                .collect::<Result<Vec<_>>>()?;
            let fat = (0..nf)
                .map(|j| {
                    let record = vbase + nd * 16 + j * 24;
                    vertex(record + 8, record, 24)
                })
                .collect::<Result<Vec<_>>>()?;
            packets.push(Packet {
                lod,
                header,
                unpack,
                strips,
                dinky,
                fat,
            });
        }
    }
    Ok(Class {
        class,
        scale,
        bounds,
        material_packets,
        packets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let mut b = vec![0u8; 256];
        b[0..4].copy_from_slice(&128u32.to_le_bytes());
        b[12] = 1;
        b[0x40..0x44].copy_from_slice(&2f32.to_le_bytes());
        b[0x44..0x46].copy_from_slice(&2592i16.to_le_bytes());
        b[128..132].copy_from_slice(&16u32.to_le_bytes());
        b[136] = 4;
        b[137] = 1; // data at144; vertex at208; one16-byte vertex
        b[144 + 40] = 6;
        b[144 + 42] = 1;
        b[208..210].copy_from_slice(&(-512i16).to_le_bytes());
        b
    }
    #[test]
    fn gc_not_rac1_header_and_packed_position() {
        let b = fixture();
        let c = read(&b, 2592).unwrap();
        assert_eq!(c.packets.len(), 1);
        assert_eq!(c.packets[0].dinky[0].position, [-1., 0., 0.]);
        assert_eq!(c.packets[0].dinky[0].raw.len(), 16);
        assert_eq!(c.packets[0].header[5], 0); // not RAC1's shader_count * 5
    }
    #[test]
    fn wrong_class_and_invalid_transform_fail() {
        let mut b = fixture();
        assert!(read(&b, 1).is_err());
        b[0x40..0x44].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(read(&b, 2592).is_err());
    }
    #[test]
    fn fat_vertex_position_follows_uv_prefix_with_qw_padding() {
        let mut b = fixture();
        b[137] = 2; // 24 bytes used, 32-byte QW region
        b[144 + 40] = 4;
        b[144 + 42] = 0;
        b[144 + 43] = 1;
        b[208..216].fill(0x7f); // UV prefix is NOT xyz
        b[216..222].fill(0);
        b[216..218].copy_from_slice(&512i16.to_le_bytes());
        let c = read(&b, 2592).unwrap();
        let p = &c.packets[0];
        assert!(p.dinky.is_empty());
        assert_eq!(p.fat[0].position, [1., 0., 0.]);
        assert_eq!(p.fat[0].raw.len(), 24);
    }
    #[test]
    fn count_and_bounds_do_not_guess_vertices() {
        let mut b = fixture();
        b[144 + 42] = 2;
        assert!(read(&b, 2592).is_err());
        b = fixture();
        b[144 + 43] = 255;
        assert!(read(&b, 2592).is_err());
        b = fixture();
        b[128..132].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(read(&b, 2592).is_err());
        assert!(read(&fixture()[..200], 2592).is_err());
    }
    #[test]
    fn arbitrary_input_does_not_panic() {
        for length in 0..300 {
            for fill in [0, 1, 127, 255] {
                assert!(
                    std::panic::catch_unwind(|| read(&vec![fill; length], 2592).map(|_| ()))
                        .is_ok()
                );
            }
        }
    }
}
