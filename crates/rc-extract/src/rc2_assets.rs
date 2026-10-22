//! Explicit, experimental RC2 asset conversion. Never writes RAC1's archive,
//! cache stamp or extract-info.json and never marks RC2 playable.
use crate::{build_db::BUILDS, identify, json, sha1, Code, Emit, Error};
use rc_formats::{buf::Buf, rc2, wad};
use std::{fs, io::Read, path::Path};

const IMAGE_BYTES: u64 = 3_824_615_424;
const IMAGE_SHA1: &str = "eeba0fff2118b14cbf838e6638ef7af3e72c0c31";
const BOOT_SHA1: &str = "4124a79c1e5a1ceb56f514df0dbbc11f072af56f";
fn bad(e: impl std::fmt::Display) -> Error { Error::new(Code::VerifyFailed, e.to_string()) }
fn write_error(e: impl std::fmt::Display) -> Error { Error::new(Code::WriteFailed, e.to_string()) }

/// Convert one archive slot. Filename slot != logical level ID (slot21 ->30).
/// Output is private disc-derived content; never publish it as CI evidence.
pub fn prepare(path: &Path, out: &Path, slot: u32, emit: Emit) -> Result<String, Error> {
    if slot > 26 { return Err(bad("RC2_ARCHIVE_SLOT_OUT_OF_VERIFIED_SCOPE")); }
    if out.exists() { return Err(write_error("RC2_OUTPUT_EXISTS_USE_NEW_DESTINATION")); }
    let (id, iso) = identify::probe(path, BUILDS, emit)?;
    if id.serial != "SCUS_972.68" || id.version != "2.00" || sha1::hex(&id.elf_sha1) != BOOT_SHA1 {
        return Err(bad("RC2_CONVERSION_BUILD_IDENTITY_MISMATCH"));
    }
    // Source bytes are pinned, not inferred from the filename/product serial.
    let before = fs::metadata(path).map_err(bad)?;
    if before.len() != IMAGE_BYTES { return Err(bad("RC2_CONVERSION_IMAGE_SIZE_MISMATCH")); }
    let mut file = fs::File::open(path).map_err(bad)?;
    let mut hash = sha1::Sha1::new(); let mut buffer = [0; 64*1024];
    loop { let n = file.read(&mut buffer).map_err(bad)?; if n == 0 { break; } hash.update(&buffer[..n]); }
    if sha1::hex(&hash.finish()) != IMAGE_SHA1 { return Err(bad("RC2_CONVERSION_IMAGE_HASH_MISMATCH")); }
    let entry = iso.find(&format!("/G/LEVEL{slot}.WAD")).ok_or_else(|| bad("RC2_LEVEL_FILE_MISSING"))?;
    let header = iso.read_bytes(entry.lba as u64 * 2048, 96).map_err(bad)?;
    let level_id = Buf(&header).u32(8).map_err(bad)?;
    // Independently checked relocation duplicate in RC2.HDR ties this file to
    // the exact verified disc's archive table. Never substitute another level.
    let toc_entry = iso.find("/RC2.HDR").ok_or_else(|| bad("RC2_HDR_MISSING"))?;
    let table_offset = toc_entry.lba as u64*2048 + 0x4ff8 + slot as u64*24;
    let table = iso.read_bytes(table_offset, 8).map_err(bad)?;
    let duplicate_pos = Buf(&table).u32(0).map_err(bad)? as u64*2048;
    let count = Buf(&table).u32(4).map_err(bad)? as u64;
    if duplicate_pos < toc_entry.lba as u64*2048 || duplicate_pos+96 > toc_entry.lba as u64*2048+toc_entry.size as u64 ||
       count != (entry.size as u64).div_ceil(2048) {
        return Err(bad("RC2_HDR_LEVEL_RANGE_MISMATCH"));
    }
    let duplicate = iso.read_bytes(duplicate_pos,96).map_err(bad)?;
    if Buf(&duplicate).u32(4).map_err(bad)? != entry.lba || duplicate[..4] != header[..4] || duplicate[8..] != header[8..] {
        return Err(bad("RC2_HDR_LEVEL_IDENTITY_MISMATCH"));
    }
    let plan = rc2::level_plan(&header, entry.size as usize, level_id).map_err(bad)?;
    let read = |name| -> Result<Vec<u8>,Error> {
        let l = plan.lumps.iter().find(|l| l.name == name).ok_or_else(|| bad("RC2_REQUIRED_LUMP_MISSING"))?;
        if l.bytes.len() > rc2::MAX_PRIMARY { return Err(bad("RC2_LUMP_MEMORY_BUDGET")); }
        iso.read_bytes(entry.lba as u64*2048+l.bytes.start as u64, l.bytes.len() as u64).map_err(bad)
    };
    let primary = read("primary")?;
    let gameplay = wad::decompress_bounded(&read("gameplay")?, 16<<20).map_err(bad)?;
    let assets = rc_data::rc2::prepare(&primary,&gameplay).map_err(bad)?;
    let mesh = rc_data::rc2::mesh_bytes(&assets).map_err(bad)?;
    let triangles = rc_data::rc2::validate_mesh(&mesh,assets.mip_images.len()).map_err(bad)?;
    if triangles != assets.triangles.iter().sum::<usize>() { return Err(bad("RC2_NATIVE_MESH_ROUNDTRIP_MISMATCH")); }
    let after = fs::metadata(path).map_err(bad)?;
    if before.len()!=after.len() || before.modified().ok()!=after.modified().ok() { return Err(bad("RC2_SOURCE_CHANGED_DURING_CONVERSION")); }
    // A sibling staging directory and create-new writes preserve every previous
    // conversion/checkpoint. Failed staging remains inspectable, never ACTIVE.
    if out.exists() { return Err(write_error("RC2_OUTPUT_EXISTS_USE_NEW_DESTINATION")); }
    let file_name = out.file_name().ok_or_else(|| write_error("RC2_OUTPUT_REQUIRES_DIRECTORY_NAME"))?;
    let mut stage_name = file_name.to_os_string(); stage_name.push(".partial");
    let stage = out.with_file_name(stage_name);
    fs::create_dir(&stage).map_err(write_error)?;
    let mut records = Vec::new();
    let mut save = |name: &str, bytes: &[u8]| -> Result<(),Error> {
        use std::io::Write;
        let mut f = fs::OpenOptions::new().write(true).create_new(true).open(stage.join(name)).map_err(write_error)?;
        f.write_all(bytes).map_err(write_error)?; f.sync_all().map_err(write_error)?;
        records.push(json::object(&[("file",name.into()),("bytes",bytes.len().into()),("sha1",sha1::hex(&sha1::sha1(bytes)).into())]));
        Ok(())
    };
    save("terrain.mesh", &mesh)?;
    for (i,mips) in assets.mip_images.iter().enumerate() {
        for (mip,image) in mips.iter().enumerate() {
            let mut bytes = Vec::with_capacity(8+image.rgba.len());
            bytes.extend_from_slice(&image.width.to_le_bytes()); bytes.extend_from_slice(&image.height.to_le_bytes()); bytes.extend_from_slice(&image.rgba);
            save(&format!("terrain-{i:03}-mip{mip}.rgba"), &bytes)?;
        }
    }
    let objects = assets.objects.iter().enumerate().map(|(i,o)| json::object(&[
        ("instance",i.into()),("class",o.class.into()),("mission",o.mission.into()),("pvar_index",o.pvar.into()),
        ("position_f32_bits",json::Value::StrList(o.position.iter().map(|x|format!("{:08x}",x.to_bits())).collect())),
        ("rotation_f32_bits",json::Value::StrList(o.rotation.iter().map(|x|format!("{:08x}",x.to_bits())).collect())),
        ("scale_f32_bits",format!("{:08x}",o.scale.to_bits()).into())])).collect::<Vec<_>>().join("\n");
    save("objects.jsonl",objects.as_bytes())?;
    let class_tables = [(&assets.core.mobys,"moby"),(&assets.core.ties,"tie")];
    let mut classes = Vec::new();
    for (table,kind) in class_tables { for c in table { classes.push(json::object(&[
        ("kind",kind.into()),("class",c.o_class.into()),("core_data_offset",c.offset_in_asset_wad.into()),
        ("texture_slots",json::Value::StrList(c.textures.iter().map(|v|v.to_string()).collect())),
        ("native_geometry_converted",false.into()),("native_update_implemented",false.into())])); }}
    for s in &assets.core.shrubs { let c=&s.base; classes.push(json::object(&[
        ("kind","shrub".into()),("class",c.o_class.into()),("core_data_offset",c.offset_in_asset_wad.into()),
        ("texture_slots",json::Value::StrList(c.textures.iter().map(|v|v.to_string()).collect())),
        ("native_geometry_converted",false.into()),("native_update_implemented",false.into())])); }
    save("class-dependencies.jsonl",classes.join("\n").as_bytes())?;
    // Preserve exact raw palette alpha and index data alongside the decoded
    // images. The existing display-oriented RGBA decoder saturates alpha>=128;
    // a future GS material adapter must not infer the raw alpha from that image.
    let p=rc2::primary(&primary).map_err(bad)?;
    let core_data=wad::decompress_bounded(p.compressed_core,rc2::MAX_DECODED_CORE).map_err(bad)?;
    let mut palette_entries = Vec::new();
    for (i,e) in assets.core.terrain_textures.iter().enumerate() {
        let images=rc_formats::texture::tfrag_mip_images_in(&core_data[assets.core.textures_base..],p.gs_ram,e).map_err(bad)?;
        save(&format!("terrain-{i:03}.clut"), images[0].clut)?;
        for (mip,image) in images.iter().enumerate() { save(&format!("terrain-{i:03}-mip{mip}.indices"),image.indices)?; }
        palette_entries.push(json::object(&[("material",i.into()),("width",(e.width as i32).into()),("height",(e.height as i32).into()),
            ("mip_levels",(e.ty as i32).into()),("base_data_offset",e.data_offset.into()),("clut_block",(e.palette as i32).into()),
            ("mip2_block",(e.mipmap as i32).into()),("mip3_block",(e.pad as i32).into())]));
    }
    save("terrain-materials.jsonl",palette_entries.join("\n").as_bytes())?;
    drop(save);
    let summary = json::object(&[
        ("game","rac2".into()),("schema_version",1u32.into()),("native_scene_version",rc_data::rc2::SCENE_VERSION.into()),
        ("status","TERRAIN_AND_OBJECT_METADATA_CONVERTED_NOT_A_COMPLETE_GAME_PACKAGE".into()),
        ("serial",id.serial.into()),("version",id.version.into()),("image_sha1",IMAGE_SHA1.into()),("boot_sha1",BOOT_SHA1.into()),
        ("archive_slot",slot.into()),("logical_level",level_id.into()),("terrain_fragments",assets.terrain.len().into()),
        ("triangles_all_lods",triangles.into()),("materials",assets.mip_images.len().into()),("objects",assets.objects.len().into()),
        ("moby_classes",assets.core.mobys.len().into()),("tie_classes",assets.core.ties.len().into()),("shrub_classes",assets.core.shrubs.len().into()),
        ("missing_local_moby_classes",json::Value::StrList(assets.missing_object_classes.iter().map(|v|v.to_string()).collect())),
        ("runtime_executed",false.into()),("native_gameplay_implemented",false.into()),("FIRST_FATAL_BLOCKER",json::Value::Null),
        ("KNOWN_MISSING_CAPABILITIES",json::Value::StrList(vec!["RC2_NATIVE_RENDER_STATE_AND_VU_PARITY".into(),"RC2_OBJECT_GEOMETRY_CONVERSION".into(),"RC2_NATIVE_GAMEPLAY_MODULE".into(),"RC2_AUDIO_AND_SCENE_DEPENDENCIES".into()])),
    ]);
    let manifest = format!("{{\"summary\":{summary},\"files\":[{}]}}\n",records.join(","));
    { use std::io::Write; let mut f=fs::OpenOptions::new().write(true).create_new(true).open(stage.join("rc2-assets.json")).map_err(write_error)?;
      f.write_all(manifest.as_bytes()).map_err(write_error)?; f.sync_all().map_err(write_error)?; }
    fs::rename(&stage,out).map_err(write_error)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn wrong_scope_does_not_touch_input_or_output() {
        let result=prepare(Path::new("missing.iso"),Path::new("missing-output"),27,&mut |_|{}).unwrap_err();
        assert!(result.message.contains("SLOT_OUT_OF_VERIFIED_SCOPE"));
    }
    #[test] fn existing_destination_is_never_overwritten() {
        let result=prepare(Path::new("missing.iso"),Path::new(env!("CARGO_MANIFEST_DIR")),0,&mut |_|{}).unwrap_err();
        assert_eq!(result.code,Code::WriteFailed);
        assert!(result.message.contains("OUTPUT_EXISTS"));
    }
}
