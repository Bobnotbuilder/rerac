//! Read-only RC2 terrain upload boundary. No RAC1 Level, shader defaults,
//! material sorting, camera, gameplay or filesystem side effects are inferred.
//! Callers must verify package identity/hashes before using these bytes.
use rc_formats::buf::{invalid, Buf, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Corner {
    pub position: [f32; 3],
    pub st: [f32; 2],
    /// Original GS colour bytes, NOT display RGBA8 with alpha rescaled.
    pub rgba: [u8; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialPacket(pub [u8; 80]);

impl MaterialPacket {
    /// On-disc fixup fields, not complete live GS registers. TEX0 contains a
    /// texture-table index; CLAMP has separate WMS/WMT words. Do not decode
    /// these as live TEX0/CLAMP bitfields or infer TEST/ALPHA/ZBUF from them.
    pub fn validate(&self, material: u32) -> Result<()> {
        let b = Buf(&self.0);
        for (i, address) in [6, 20, 8, 52, 54].into_iter().enumerate() {
            if b.u8(i * 16 + 8)? != address {
                return invalid("RC2_TERRAIN_ADGIF_REGISTER_SEQUENCE");
            }
        }
        if b.u32(0)? != material {
            return invalid("RC2_TERRAIN_PACKET_TEXTURE_MISMATCH");
        }
        Ok(())
    }
    pub fn wrap(&self) -> (i32, i32) {
        let b = Buf(&self.0);
        (b.i32(32).unwrap(), b.i32(36).unwrap())
    }
    pub fn lod_fields(&self) -> (i32, i32) {
        let b = Buf(&self.0);
        // The disc stores the signed fixed-point K in the LOW 16 bits (upper
        // bits need not sign extend). Keep the original packet for auditing.
        (b.u16(16).unwrap() as i16 as i32, b.i32(20).unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub fragment: u32,
    pub lod: u32,
    pub material: u32,
    pub packet: MaterialPacket,
    pub corners: [Corner; 3],
}

/// Borrowed verified structure, so filtering one LOD does not allocate/copy all
/// three. No unbounded count from an input header becomes an allocation.
pub struct Terrain<'a> {
    bytes: &'a [u8],
    pub fragments: u32,
    pub counts: [usize; 3],
}

impl<'a> Terrain<'a> {
    pub fn read(bytes: &'a [u8], materials: usize) -> Result<Self> {
        if bytes.len() > 128 << 20 {
            return invalid("RC2_MESH_MEMORY_BUDGET");
        }
        crate::rc2::validate_mesh(bytes, materials)?;
        let mut scene = Self {
            bytes,
            fragments: Buf(bytes).u32(12)?,
            counts: [0; 3],
        };
        let mut counts = [0; 3];
        for triangle in scene.triangles() {
            triangle.packet.validate(triangle.material)?;
            counts[triangle.lod as usize] += 1;
        }
        scene.counts = counts;
        Ok(scene)
    }
    pub fn triangles(&self) -> impl Iterator<Item = Triangle> + '_ {
        self.bytes[16..].chunks_exact(164).map(|record| {
            let b = Buf(record);
            Triangle {
                fragment: b.u32(0).unwrap(),
                lod: b.u32(4).unwrap(),
                material: b.u32(8).unwrap(),
                packet: MaterialPacket(record[12..92].try_into().unwrap()),
                corners: std::array::from_fn(|i| {
                    let at = 92 + i * 24;
                    Corner {
                        position: std::array::from_fn(|k| b.f32(at + k * 4).unwrap()),
                        st: [b.f32(at + 12).unwrap(), b.f32(at + 16).unwrap()],
                        rgba: record[at + 20..at + 24].try_into().unwrap(),
                    }
                }),
            }
        })
    }
    /// Original triangle/packet order inside a single LOD. Never concatenate
    /// all LODs into a scene or globally sort by texture. Morph/LOD selection
    /// and original VU parity are separate unresolved runtime requirements.
    pub fn lod(&self, lod: u32) -> Result<impl Iterator<Item = Triangle> + '_> {
        if lod > 2 {
            return invalid("RC2_TERRAIN_LOD_OUT_OF_RANGE");
        }
        Ok(self.triangles().filter(move |t| t.lod == lod))
    }
}

/// Native RGBA cache image: exact dimensions/payload, no fallback/magenta image.
pub fn rgba_image(bytes: &[u8]) -> Result<(u32, u32, &[u8])> {
    let b = Buf(bytes);
    b.check(0, 8, "RC2 RGBA image header")?;
    let (w, h) = (b.u32(0)?, b.u32(4)?);
    if w == 0 || h == 0 || w > 1024 || h > 1024 || !w.is_power_of_two() || !h.is_power_of_two() {
        return invalid("RC2_TERRAIN_IMAGE_DIMENSIONS");
    }
    if bytes.len() != 8 + w as usize * h as usize * 4 {
        return invalid("RC2_TERRAIN_IMAGE_LENGTH");
    }
    Ok((w, h, &bytes[8..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mesh() -> Vec<u8> {
        let mut out = crate::rc2::SCENE_MAGIC.to_vec();
        out.extend(1u32.to_le_bytes());
        out.extend(2u32.to_le_bytes());
        for (fragment, lod, material) in [(0u32, 0u32, 1u32), (0, 0, 0), (0, 1, 0), (1, 0, 1)] {
            let mut record = [0u8; 164];
            for (i, v) in [fragment, lod, material].into_iter().enumerate() {
                record[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
            }
            record[12..16].copy_from_slice(&material.to_le_bytes());
            for (i, a) in [6, 20, 8, 52, 54].into_iter().enumerate() {
                record[20 + i * 16] = a;
            }
            record[112..116].copy_from_slice(&[7, 8, 9, 128]);
            out.extend(record);
        }
        out
    }
    #[test]
    fn lod_isolation_preserves_packet_order_and_raw_alpha() {
        let bytes = mesh();
        let t = Terrain::read(&bytes, 2).unwrap();
        assert_eq!(t.counts, [3, 1, 0]);
        let ts = t.lod(0).unwrap().collect::<Vec<_>>();
        assert_eq!(
            ts.iter().map(|t| t.material).collect::<Vec<_>>(),
            vec![1, 0, 1]
        );
        assert_eq!(ts[0].corners[0].rgba, [7, 8, 9, 128]);
        assert!(t.lod(3).is_err());
    }
    #[test]
    fn packet_not_inferred_from_material_index() {
        let mut b = mesh();
        b[44..48].copy_from_slice(&(-13i32).to_le_bytes());
        let t = Terrain::read(&b, 2).unwrap();
        assert_eq!(t.triangles().next().unwrap().packet.lod_fields().0, -13);
        b[44..48].copy_from_slice(&0x0000ff77u32.to_le_bytes());
        assert_eq!(
            Terrain::read(&b, 2)
                .unwrap()
                .triangles()
                .next()
                .unwrap()
                .packet
                .lod_fields()
                .0,
            -137
        );
        b[20] = 7;
        assert!(Terrain::read(&b, 2).is_err());
        b = mesh();
        b[12 + 16..16 + 16].copy_from_slice(&9u32.to_le_bytes());
        assert!(Terrain::read(&b, 2).is_err());
    }
    #[test]
    fn malformed_mesh_rejected() {
        let mut b = mesh();
        b[16 + 92..16 + 96].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(Terrain::read(&b, 2).is_err());
        assert!(Terrain::read(&mesh()[..20], 2).is_err());
        assert!(Terrain::read(&mesh(), 1).is_err());
    }
    #[test]
    fn image_dimensions_length_and_no_trailing_data() {
        let mut b = vec![];
        b.extend(2u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend([0u8; 8]);
        assert_eq!(rgba_image(&b).unwrap().0, 2);
        b.push(0);
        assert!(rgba_image(&b).is_err());
        b[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(rgba_image(&b).is_err());
        assert!(rgba_image(&[]).is_err());
    }
}
