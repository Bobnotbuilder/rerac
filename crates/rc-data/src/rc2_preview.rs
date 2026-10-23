//! Experimental static terrain upload plan, not GS/VU parity or gameplay.
//! Reads the preserved Build7 Aranos package without writing cache/save data.
use crate::{
    rc2_aranos_receipt as receipt,
    rc2_scene::{rgba_image, MaterialPacket, Terrain},
};
use rc_formats::buf::{invalid, Result};
use std::{io::Read, path::Path};

#[derive(Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub struct Draw {
    pub fragment: u32,
    pub material: u32,
    pub packet: MaterialPacket,
    pub positions: Vec<[f32; 3]>,
    pub st: Vec<[f32; 2]>,
    /// Raw bytes as floats for interpolation; shader policy is explicit.
    pub rgba: Vec<[f32; 4]>,
}

#[derive(Debug)]
pub struct Preview {
    pub draws: Vec<Draw>,
    pub images: Vec<Image>,
    pub triangles: usize,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub lod: u32,
}

/// Diagnostic coordinate policy: an orthonormal Z-up to Y-up rotation.
/// Does not assert original GC camera/projection parity. No scaling/mirroring.
pub fn diagnostic_position(p: [f32; 3]) -> [f32; 3] {
    [p[0], p[2], -p[1]]
}

/// Contiguous runs only: no global texture sort, no cross-fragment merging,
/// no mixing LODs, no loss of material packet fields or original base colours.
pub fn plan(terrain: &Terrain<'_>, lod: u32) -> Result<Preview> {
    let mut out = Preview {
        draws: vec![],
        images: vec![],
        triangles: 0,
        min: [f32::INFINITY; 3],
        max: [f32::NEG_INFINITY; 3],
        lod,
    };
    for t in terrain.lod(lod)? {
        let (s, v) = t.packet.wrap();
        if !matches!(s, 0 | 1) || !matches!(v, 0 | 1) {
            return invalid("RC2_PREVIEW_UNSUPPORTED_WRAP");
        }
        if out.draws.last().is_none_or(|d| {
            d.fragment != t.fragment || d.material != t.material || d.packet != t.packet
        }) {
            out.draws.push(Draw {
                fragment: t.fragment,
                material: t.material,
                packet: t.packet,
                positions: vec![],
                st: vec![],
                rgba: vec![],
            });
        }
        let draw = out.draws.last_mut().unwrap();
        for c in t.corners {
            let p = diagnostic_position(c.position);
            for i in 0..3 {
                out.min[i] = out.min[i].min(p[i]);
                out.max[i] = out.max[i].max(p[i]);
            }
            draw.positions.push(p);
            draw.st.push(c.st);
            draw.rgba.push(c.rgba.map(f32::from));
        }
        out.triangles += 1;
    }
    if out.triangles == 0 {
        return invalid("RC2_PREVIEW_EMPTY_LOD");
    }
    Ok(out)
}

fn read_verified(root: &Path, name: &str, size: usize, digest: &str) -> Result<Vec<u8>> {
    // Fixed receipt names; never traverse a manifest-controlled path. Open once,
    // bound the read even if the file changes, then hash exactly the uploaded bytes.
    let mut file = std::fs::File::open(root.join(name)).map_err(io_error)?;
    if file.metadata().map_err(io_error)?.len() != size as u64 {
        return invalid("RC2_PREVIEW_PAYLOAD_LENGTH");
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(size as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() != size || rc_formats::sha1::hex(&rc_formats::sha1::sha1(&bytes)) != digest {
        return invalid("RC2_PREVIEW_PAYLOAD_IDENTITY");
    }
    Ok(bytes)
}

fn io_error(error: std::io::Error) -> rc_formats::FormatError {
    rc_formats::FormatError::Invalid(format!("RC2_PREVIEW_IO: {error}"))
}

/// Deliberately scoped to the already validated Build7 Aranos receipt. All
/// uploaded bytes must match it; unrelated object payloads are not consumed.
/// A different conversion/build needs its own reviewed receipt, never fallback.
pub fn load_aranos(root: &Path, lod: u32) -> Result<Preview> {
    if lod > 2 {
        return invalid("RC2_TERRAIN_LOD_OUT_OF_RANGE");
    }
    let manifest_len = std::fs::metadata(root.join("rc2-assets.json"))
        .map_err(io_error)?
        .len();
    if manifest_len > 256 << 10 {
        return invalid("RC2_PREVIEW_MANIFEST_BUDGET");
    }
    read_verified(
        root,
        "rc2-assets.json",
        manifest_len as usize,
        receipt::MANIFEST_SHA1,
    )?;
    let mesh = read_verified(root, "terrain.mesh", 14_210_288, receipt::MESH_SHA1)?;
    let terrain = Terrain::read(&mesh, receipt::IMAGES.len())?;
    let mut out = plan(&terrain, lod)?;
    for (file, len, hash) in receipt::IMAGES {
        let bytes = read_verified(root, file, len, hash)?;
        let (width, height, pixels) = rgba_image(&bytes)?;
        out.images.push(Image {
            width,
            height,
            rgba: pixels.to_vec(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upload_rejects_changed_length_and_same_length_corruption_without_writing() {
        let root = std::env::temp_dir().join(format!(
            "rc2-preview-integrity-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let bytes = b"synthetic upload receipt";
        let hash = rc_formats::sha1::hex(&rc_formats::sha1::sha1(bytes));
        std::fs::write(root.join("fixture"), bytes).unwrap();
        assert_eq!(
            read_verified(&root, "fixture", bytes.len(), &hash).unwrap(),
            bytes
        );
        assert!(read_verified(&root, "fixture", bytes.len() + 1, &hash).is_err());
        let mut changed = bytes.to_vec();
        changed[0] ^= 1;
        std::fs::write(root.join("fixture"), &changed).unwrap();
        assert!(read_verified(&root, "fixture", bytes.len(), &hash).is_err());
        assert_eq!(std::fs::read(root.join("fixture")).unwrap(), changed);
        assert!(read_verified(&root, "missing", 1, &hash).is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    fn mesh() -> Vec<u8> {
        let mut b = crate::rc2::SCENE_MAGIC.to_vec();
        b.extend(1u32.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        for (fragment, lod, material) in [
            (0u32, 0u32, 1u32),
            (0, 0, 0),
            (0, 0, 1),
            (0, 1, 0),
            (1, 0, 1),
        ] {
            let mut r = [0u8; 164];
            for (i, n) in [fragment, lod, material].into_iter().enumerate() {
                r[i * 4..i * 4 + 4].copy_from_slice(&n.to_le_bytes());
            }
            r[12..16].copy_from_slice(&material.to_le_bytes());
            for (i, a) in [6, 20, 8, 52, 54].into_iter().enumerate() {
                r[20 + i * 16] = a;
            }
            for c in 0..3 {
                for (i, x) in [1f32, 2., 3., 0.25, 0.75].into_iter().enumerate() {
                    let at = 92 + c * 24 + i * 4;
                    r[at..at + 4].copy_from_slice(&x.to_le_bytes());
                }
                r[112 + c * 24..116 + c * 24].copy_from_slice(&[64, 128, 255, 128]);
            }
            b.extend(r);
        }
        b
    }
    #[test]
    fn upload_preserves_order_uv_raw_colour_and_single_lod() {
        let b = mesh();
        let t = Terrain::read(&b, 2).unwrap();
        let p = plan(&t, 0).unwrap();
        assert_eq!(p.triangles, 4);
        assert_eq!(p.draws.len(), 4);
        assert_eq!(
            p.draws.iter().map(|d| d.material).collect::<Vec<_>>(),
            [1, 0, 1, 1]
        );
        assert_eq!(p.draws[0].positions[0], [1., 3., -2.]);
        assert_eq!(p.draws[0].st[0], [0.25, 0.75]);
        assert_eq!(p.draws[0].rgba[0], [64., 128., 255., 128.]);
        assert_eq!(p.min, p.max);
        assert_eq!(plan(&t, 1).unwrap().triangles, 1);
        assert!(plan(&t, 2).is_err());
        assert!(plan(&t, 3).is_err());
    }
    #[test]
    fn unsupported_sampling_is_explicit() {
        let mut b = mesh();
        b[16 + 44..16 + 48].copy_from_slice(&2u32.to_le_bytes());
        assert!(plan(&Terrain::read(&b, 2).unwrap(), 0).is_err());
    }
    #[test]
    fn coordinate_rotation_preserves_orientation_and_length() {
        assert_eq!(diagnostic_position([1., 0., 0.]), [1., 0., 0.]);
        assert_eq!(diagnostic_position([0., 1., 0.]), [0., 0., -1.]);
        assert_eq!(diagnostic_position([0., 0., 1.]), [0., 1., 0.]);
    }
}
