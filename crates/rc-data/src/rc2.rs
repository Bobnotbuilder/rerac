//! Native RC2 scene-asset preparation, not a replacement game implementation.
//! Only disc-backed geometry/materials/objects are emitted. Unknown update,
//! lighting, camera and object-class semantics remain explicit blockers.
use rc_formats::{buf::{invalid, Buf, Result}, rc2, texture, tfrag};

pub const SCENE_VERSION: u32 = 1;
pub const SCENE_MAGIC: &[u8; 8] = b"RC2MESH1";

#[derive(Debug)]
pub struct Assets {
    pub terrain: Vec<tfrag::Tfrag>,
    pub mip_images: Vec<Vec<texture::Texture>>,
    pub objects: Vec<rc2::Object>,
    pub core: rc2::Core,
    pub triangles: [usize; 3],
    pub missing_object_classes: Vec<i32>,
}

/// The native scene module's data boundary. Does not dispatch rc-game's RAC1
/// scheduler, infer class behaviour from numeric IDs, or execute MIPS/VU code.
pub fn prepare(primary: &[u8], gameplay: &[u8]) -> Result<Assets> {
    let p = rc2::primary(primary)?;
    let data = rc_formats::wad::decompress_bounded(p.compressed_core, rc2::MAX_DECODED_CORE)?;
    let core = rc2::core(p.index, data.len())?;
    if core.compressed_bytes != p.compressed_core.len() {
        return invalid("RC2_CORE_COMPRESSED_SIZE_MISMATCH");
    }
    let terrain = tfrag::parse_tfrags(&data[core.geometry.clone()])?;
    let triangles = rc2::validate_terrain(&terrain, core.terrain_textures.len())?;
    let mut mip_images = Vec::with_capacity(core.terrain_textures.len());
    let mut texture_bytes = 0usize;
    for entry in &core.terrain_textures {
        let images = texture::tfrag_mip_images_in(&data[core.textures_base..], p.gs_ram, entry)?;
        for image in &images {
            texture_bytes = texture_bytes.checked_add(image.indices.len().saturating_mul(4))
                .ok_or_else(|| rc_formats::FormatError::Invalid("RC2_TEXTURE_MEMORY_BUDGET".into()))?;
            if texture_bytes > 128 << 20 { return invalid("RC2_TEXTURE_MEMORY_BUDGET"); }
        }
        mip_images.push(images.iter().map(texture::IndexedImage::decode).collect::<Result<Vec<_>>>()?);
    }
    let objects = rc2::objects(gameplay)?;
    let mut missing_object_classes = objects.iter().map(|o| o.class)
        .filter(|id| !core.mobys.iter().any(|c| c.o_class == *id)).collect::<Vec<_>>();
    missing_object_classes.sort_unstable(); missing_object_classes.dedup();
    Ok(Assets { terrain, mip_images, objects, core, triangles, missing_object_classes })
}

/// Portable little-endian native triangle stream. Original fragment, LOD and
/// strip order is retained (no global material sorting). Every triangle carries
/// the original 5 x 16-byte material packet; no invented default GS state.
/// Each corner: game-space xyz f32, normalised st f32, original RGBA u8x4.
/// This is a static asset representation, NOT evidence of RC2 VU/shader parity.
pub fn mesh_bytes(assets: &Assets) -> Result<Vec<u8>> {
    let counts = rc2::validate_terrain(&assets.terrain, assets.mip_images.len())?;
    if counts.iter().sum::<usize>() > (128 << 20) / 164 { return invalid("RC2_MESH_MEMORY_BUDGET"); }
    let mut bytes = Vec::new(); bytes.extend_from_slice(SCENE_MAGIC);
    let u = |v: &mut Vec<u8>, n: u32| v.extend_from_slice(&n.to_le_bytes());
    u(&mut bytes, SCENE_VERSION); u(&mut bytes, assets.terrain.len() as u32);
    for (fragment, t) in assets.terrain.iter().enumerate() {
        for lod in 0..3 {
            let tris = tfrag::tfrag_triangles(t, lod)?;
            for tri in tris {
                u(&mut bytes, fragment as u32); u(&mut bytes, lod as u32);
                u(&mut bytes, t.texture_index(tri.ad_gif as usize) as u32);
                let a = t.ad_gifs[tri.ad_gif as usize];
                for q in [a.tex0, a.tex1, a.clamp, a.miptbp1, a.miptbp2] {
                    for word in [q.data_lo as u32, q.data_hi as u32] {
                        u(&mut bytes, word);
                    }
                    bytes.push(q.address); bytes.extend_from_slice(&q.pad);
                }
                for v in [tri.a, tri.b, tri.c] {
                    // prepare() checked the raw position reference, never use
                    // the old exporter fallback for malformed position indices.
                    let vi = &t.vertex_info[v as usize]; let p = vi.vertex as usize / 2;
                    let xyz = t.world_position(v as usize);
                    for value in xyz.into_iter().chain([vi.s as f32 / 4096., vi.t as f32 / 4096.]) {
                        bytes.extend_from_slice(&value.to_le_bytes());
                    }
                    let c = t.rgba[p]; bytes.extend_from_slice(&[c.r,c.g,c.b,c.a]);
                }
            }
        }
    }
    Ok(bytes)
}

/// Validate persisted native mesh structure before a future renderer uploads it.
/// Returns triangle count. Asset identity/integrity is checked by the manifest.
pub fn validate_mesh(bytes: &[u8], materials: usize) -> Result<usize> {
    let b = Buf(bytes); b.check(0, 16, "RC2 native mesh header")?;
    if &bytes[..8] != SCENE_MAGIC || b.u32(8)? != SCENE_VERSION { return invalid("RC2_NATIVE_MESH_VERSION_MISMATCH"); }
    const STRIDE: usize = 12 + 80 + 3 * 24;
    if (bytes.len() - 16) % STRIDE != 0 { return invalid("RC2_NATIVE_MESH_TRUNCATED"); }
    let fragments = b.u32(12)?;
    let mut previous = None;
    for at in (16..bytes.len()).step_by(STRIDE) {
        let key = (b.u32(at)?,b.u32(at+4)?);
        if key.0 >= fragments || key.1 > 2 || b.u32(at+8)? as usize >= materials {
            return invalid("RC2_NATIVE_MESH_REFERENCE_INVALID");
        }
        if previous.is_some_and(|p| p > key) { return invalid("RC2_NATIVE_MESH_DRAW_ORDER_INVALID"); }
        previous = Some(key);
        for corner in 0..3 { for component in 0..5 {
            if !b.f32(at + 92 + corner * 24 + component * 4)?.is_finite() {
                return invalid("RC2_NATIVE_MESH_NONFINITE");
            }
        }}
    }
    Ok((bytes.len()-16)/STRIDE)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty_mesh_and_truncation() {
        let mut b = SCENE_MAGIC.to_vec(); b.extend_from_slice(&1u32.to_le_bytes()); b.extend_from_slice(&0u32.to_le_bytes());
        assert_eq!(validate_mesh(&b,0).unwrap(),0);
        b.push(0); assert!(validate_mesh(&b,0).is_err());
        assert!(prepare(&[],&[]).is_err());
    }
    #[test] fn invalid_material_nan_and_draw_order_are_rejected() {
        let mut b = SCENE_MAGIC.to_vec(); b.extend_from_slice(&1u32.to_le_bytes()); b.extend_from_slice(&1u32.to_le_bytes());
        b.resize(16+164,0); assert_eq!(validate_mesh(&b,1).unwrap(),1);
        b[24..28].copy_from_slice(&1u32.to_le_bytes()); assert!(validate_mesh(&b,1).is_err());
        b[24..28].fill(0); b[108..112].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
        assert!(validate_mesh(&b,1).is_err());
    }
}
