//! Read-only original Aranos character resources. No RAC1 scheduler/evaluator,
//! disk cache, save access, guessed animation mapping or playable-mode enablement.
use rc_formats::{buf::{invalid, Result}, iso9660::IsoImage, moby, pif::Pif, rc2, rc2_anim, sha1, wad, Buf};

pub struct Character {
    pub class: moby::MobyClass,
    pub mesh: moby::GcArmorMesh,
    pub images: Vec<rc_formats::texture::Texture>,
    pub class_bytes: Vec<u8>,
    pub mesh_bytes: Vec<u8>,
    pub texture_bytes: Vec<u8>,
    /// Original slot numbers preserved, including absent slots. No idle/run IDs inferred.
    pub sequences: Vec<Option<Vec<u8>>>,
}

fn disc_error(e: rc_formats::iso9660::DiscError) -> rc_formats::FormatError { rc_formats::FormatError::Invalid(format!("GC_CHARACTER_DISC:{e}")) }

fn read(iso: &IsoImage, path: &str, limit: usize) -> Result<Vec<u8>> {
    let f = iso.find(path).ok_or_else(||rc_formats::FormatError::Invalid(format!("GC_CHARACTER_FILE_MISSING:{path}")))?;
    if f.size as usize > limit { return invalid("GC_CHARACTER_FILE_BUDGET"); }
    iso.read_bytes(f.lba as u64*2048, f.size as u64).map_err(disc_error)
}

/// Reads the nine armor/six wrench archive's first armor mesh. Its packet counts
/// must match Aranos class 0. Association is supported by the GC class database
/// and matching resource headers; original selection/animation behaviour is not
/// established by this static check.
pub fn load_aranos(iso: &IsoImage) -> Result<Character> {
    let elf = read(iso,"/SCUS_972.68",3<<20)?;
    if sha1::hex(&sha1::sha1(&elf)) != "4124a79c1e5a1ceb56f514df0dbbc11f072af56f" {
        return invalid("GC_CHARACTER_BUILD_IDENTITY");
    }
    let f = iso.find("/G/LEVEL0.WAD").ok_or_else(||rc_formats::FormatError::Invalid("GC_CHARACTER_LEVEL_MISSING".into()))?;
    let header = iso.read_bytes(f.lba as u64*2048,96).map_err(disc_error)?;
    let plan = rc2::level_plan(&header,f.size as usize,0)?;
    let primary = &plan.lumps.iter().find(|v| v.name=="primary").unwrap().bytes;
    let bytes = iso.read_bytes(f.lba as u64*2048+primary.start as u64,primary.len() as u64).map_err(disc_error)?;
    let p = rc2::primary(&bytes)?;
    let data = wad::decompress_bounded(p.compressed_core,rc2::MAX_DECODED_CORE)?;
    let core = rc2::core(p.index,data.len())?;
    let offsets = rc2::hero_sequence_offsets(p.index,data.len())?;
    let bounds = rc2::core_block_bounds(p.index,&core)?;
    let hero = core.mobys.iter().find(|v|v.o_class==0 && v.offset_in_asset_wad>0)
        .ok_or_else(||rc_formats::FormatError::Invalid("GC_HERO_CLASS_MISSING".into()))?;
    let at = hero.offset_in_asset_wad as usize;
    let class_bytes = data[rc2::core_block_range(&bounds,at)?].to_vec();
    let class = moby::parse_gc_moby_class(&class_bytes)?;
    let archive = read(iso,"/G/ARMOR.WAD",8<<20)?;
    let armor = rc2::armor_plan(&archive,archive.len())?;
    let mesh_bytes = archive[armor[0].mesh.clone()].to_vec();
    let texture_bytes = archive[armor[0].textures.clone()].to_vec();
    let mesh = moby::parse_gc_armor_mesh(&mesh_bytes)?;
    if [mesh.high_lod.len(),mesh.low_lod.len(),mesh.metal.len()] != [class.header.high_lod_count as usize,class.header.low_lod_count as usize,class.header.metal_count as usize] {
        return invalid("GC_HERO_ARMOR_COUNTS");
    }
    for s in mesh.high_lod.iter().chain(&mesh.low_lod).chain(&mesh.metal) {
        if s.unresolved_duplicates>0 { return invalid("GC_HERO_UNRESOLVED_VERTEX"); }
        for v in &s.vertices {
            if v.skin.joints[..v.skin.count as usize].iter().any(|j|*j>=class.header.joint_count) { return invalid("GC_HERO_SKIN_JOINT_RANGE"); }
        }
    }
    let images = armor_images(&texture_bytes)?;
    for t in mesh.high_lod.iter().chain(&mesh.low_lod).chain(&mesh.metal).flat_map(|s|&s.triangles) {
        if t.texture>=0 && t.texture as usize>=images.len() { return invalid("GC_HERO_TEXTURE_SLOT"); }
    }
    let sequences = offsets.into_iter().map(|at| {
        if at==0 { return Ok(None); }
        let blob=&data[rc2::core_block_range(&bounds,at)?];
        rc2_anim::parse(blob,class.header.joint_count as usize)?;
        Ok(Some(blob.to_vec()))
    }).collect::<Result<Vec<_>>>()?;
    Ok(Character { class,mesh,images,class_bytes,mesh_bytes,texture_bytes,sequences })
}

/// Bounded, uncompressed GC armor PIF list; no borrowed pixels outlive the blob.
pub fn armor_images(blob: &[u8]) -> Result<Vec<rc_formats::texture::Texture>> {
    let b=Buf(blob); let count=b.u32(0)? as usize;
    if count==0 || count>16 { return invalid("GC_ARMOR_TEXTURE_COUNT"); }
    let offsets: Vec<u32>=b.pod_slice(4,count,"GC armor PIF offsets")?;
    let mut images=Vec::new();
    for i in 0..count {
        let start=offsets[i] as usize;
        let end=offsets.get(i+1).map_or(blob.len(),|v|*v as usize);
        if start<4+count*4 || start>=end { return invalid("GC_ARMOR_TEXTURE_RANGE"); }
        images.push(Pif::parse(b.sub(start,end-start,"GC armor PIF")?.bytes())?.decode_raw()?);
    }
    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn armor_pif_ranges_and_raw_alpha_are_preserved() {
        let mut b=vec![0;8+0x421];b[..4].copy_from_slice(&1u32.to_le_bytes());b[4..8].copy_from_slice(&8u32.to_le_bytes());
        b[8..12].copy_from_slice(b"2FIP");
        for at in [16,20] { b[at..at+4].copy_from_slice(&1u32.to_le_bytes()); }
        b[24..28].copy_from_slice(&0x13u32.to_le_bytes());
        b[8+0x20..8+0x24].copy_from_slice(&[20,40,60,128]);
        assert_eq!(armor_images(&b).unwrap()[0].rgba,[20,40,60,128]);
        assert!(armor_images(&b[..b.len()-1]).is_err());
        b[4..8].copy_from_slice(&4u32.to_le_bytes());assert!(armor_images(&b).is_err());
        b[..4].copy_from_slice(&17u32.to_le_bytes());assert!(armor_images(&b).is_err());
    }
}
