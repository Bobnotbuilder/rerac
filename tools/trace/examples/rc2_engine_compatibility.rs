//! Read-only full-engine reuse experiment. No game launch, original bytes in
//! output, save access, or RAC1 identity bypass. Run privately with an owned ISO:
//! cargo run -p rc-trace --example rc2_engine_compatibility -- /private/owned.iso
use rc_formats::{collision, gameplay, iso9660::IsoImage, moby, moby_anim, rc2, rc2_anim, sha1, sky, sound_bank, tfrag, wad};
use rc_game::{collision_query::{self,QueryFlags}, moby_runtime::mode, rc2_compat};

fn main() -> Result<(),Box<dyn std::error::Error>> {
    let iso=IsoImage::open(std::env::args_os().nth(1).ok_or("expected owned ISO path")?)?;
    let boot=iso.find("/SCUS_972.68").ok_or("GC executable missing")?;
    let elf=iso.read_bytes(boot.lba as u64*2048,boot.size as u64)?;
    if sha1::hex(&sha1::sha1(&elf))!="4124a79c1e5a1ceb56f514df0dbbc11f072af56f" { return Err("unsupported GC executable".into()); }
    let file=iso.find("/G/LEVEL0.WAD").ok_or("level missing")?;
    let plan=rc2::level_plan(&iso.read_bytes(file.lba as u64*2048,96)?,file.size as usize,0)?;
    let read=|name|->Result<Vec<u8>,Box<dyn std::error::Error>> {
        let range=&plan.lumps.iter().find(|v|v.name==name).ok_or("missing level lump")?.bytes;
        if range.len()>rc2::MAX_PRIMARY { return Err("lump memory budget".into()); }
        Ok(iso.read_bytes(file.lba as u64*2048+range.start as u64,range.len() as u64)?)
    };
    let primary=read("primary")?;let p=rc2::primary(&primary)?;
    let data=wad::decompress_bounded(p.compressed_core,rc2::MAX_DECODED_CORE)?;
    let g=wad::decompress_bounded(&read("gameplay")?,16<<20)?;
    let s=rc2_compat::Scene::read(p.index,&data,&g)?;
    let bounds=rc2::core_block_bounds(p.index,&s.core)?;
    let terrain=tfrag::parse_tfrags(&data[s.core.geometry.clone()])?;
    println!("native_terrain fragments={} triangles_by_lod={:?}",terrain.len(),rc2::validate_terrain(&terrain,s.core.terrain_textures.len())?);
    let (mut old_ok,mut old_fail,mut old_vertices)=(0,0,0);
    for c in &s.classes {
        let blob=&data[rc2::core_block_range(&bounds,c.entry.offset_in_asset_wad as usize)?];
        match moby::parse_moby_class(blob) { Ok(v)=>{old_ok+=1;old_vertices+=v.high_lod.iter().chain(&v.low_lod).chain(&v.metal).map(|s|s.vertices.len()).sum::<usize>();},Err(_)=>old_fail+=1 }
    }
    println!("moby_classes gc_adapter={} rac1_parser_accepts={old_ok} rac1_parser_rejects={old_fail} accepted_vertex_records={old_vertices}; acceptance_alone_is_not_compatibility",s.classes.len());
    println!("native_objects={} hero_record={:?} scheduler_started=false",s.metadata_table.mobys.len(),s.metadata_table.hero());
    assert_eq!(s.metadata_table.mobys.len(),s.objects.len());
    assert!(s.metadata_table.mobys.iter().all(|m|m.update_fn.is_none() && m.mode&(mode::NO_UPDATE|mode::NO_ANIM)==mode::NO_UPDATE|mode::NO_ANIM && m.anim.skip_advance));
    let h=&s.objects[s.metadata_table.hero().ok_or("hero metadata missing")?];
    println!("collision cells={} hero_groups={} triangle_records_including_cell_duplicates={}",s.collision.cells.len(),s.collision.hero_groups.len(),collision::collision_triangles(&s.collision).len());
    let a=[h.position[0],h.position[1],h.position[2]+10.];let b=[a[0],a[1],(h.position[2]-40.).max(0.)];
    // The preserved shared kernel requires both endpoints inside [0,1024)^3.
    let hit=collision_query::coll_line(&s.collision,a,b,QueryFlags::NONE);
    println!("shared_line_query_at_original_start={hit:?}");
    assert_eq!((s.classes.len(),s.objects.len(),s.collision.cells.len(),s.collision.hero_groups.len()),(187,472,12079,3));
    assert_eq!((old_ok,old_fail,old_vertices),(4,183,0));
    assert!(hit.is_some(),"supported original Aranos start must find the stored floor");
    if let Some(hit)=hit {
        assert!(hit.point.iter().all(|v|v.is_finite()));
        assert!((hit.point[2]-49.890625).abs()<0.0001);
        // Independent plane/segment check for this original horizontal floor.
        assert!(hit.tri.iter().all(|v|(v[2]-hit.point[2]).abs()<0.0001));
        assert!((a[2]+hit.best*(b[2]-a[2])-hit.point[2]).abs()<0.0001);assert!((0.0..=1.0).contains(&hit.best));
        let point=[hit.point[0],hit.point[1],hit.point[2]+0.25];
        let sphere=collision_query::coll_sphere(&s.collision,point,0.5,QueryFlags::NONE);
        let capsule=collision_query::coll_capsule(&s.collision,point,1.0,0.5,QueryFlags::NONE);
        assert!(sphere.is_some() && capsule.is_some());
        println!("shared_sphere_query_hit={} shared_capsule_query_hit={} diagnostic_sizes_not_hero_physics=true",sphere.is_some(),capsule.is_some());
    }
    let (mut regular,mut special,mut rejected,mut frames)=(0,0,0,0);
    for at in rc2::hero_sequence_offsets(p.index,data.len())?.into_iter().filter(|v|*v>0) {
        let blob=&data[rc2::core_block_range(&bounds,at)?];
        if moby_anim::parse_sequence(blob,0).is_err(){rejected+=1;}
        match rc2_anim::parse(blob,111)? { rc2_anim::Sequence::Regular(s)=>{regular+=1;frames+=s.frames.len();},rc2_anim::Sequence::Special(s)=>{special+=1;frames+=s.frames.len();} }
    }
    assert_eq!((regular,special,rejected,frames),(0,96,96,2187));
    println!("hero_animation_storage regular={regular} special={special} frames={frames} rejected_by_rac1_reader={rejected} pose_evaluated=false");
    let old=gameplay::parse_moby_instances(&g);
    println!("rac1_gameplay_reader={}; native_gc_objects={}",match old {Ok(v)=>format!("ACCEPTED_{}_REQUIRES_SEMANTIC_REVIEW",v.len()),Err(e)=>format!("REJECTED:{e}")},s.objects.len());
    let sky=sky::parse_sky_block(&data[rc2::core_block_range(&bounds,s.core.sky)?]);
    println!("shared_sky_reader={}",match sky {Ok(v)=>format!("ACCEPTED_{}_SHELLS_GPU_NOT_TESTED",v.shells.len()),Err(e)=>format!("REJECTED:{e}")});
    let sound=read("sound_bank")?;
    println!("shared_sound_bank_reader={}",match sound_bank::parse_bank(&sound) {Ok(v)=>format!("ACCEPTED_{}_SOUNDS_AUDIO_NOT_PLAYED",v.sounds.len()),Err(e)=>format!("REJECTED:{e}")});
    println!("RAC1_Game_tick=false RC2_Play=false movement=false combat=false progression=false save_access=false");
    Ok(())
}
