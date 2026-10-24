//! Read-only original-data requirements inventory. Not execution or playability.
//! cargo run -p rc-game2 --example rc2_deep_scan -- PRIVATE_TIER0_DIR PRIVATE_OUTPUT.tsv
use std::{collections::BTreeMap,fs,io::{BufWriter,Write},path::Path};
use rc_formats::{Game,buf::Buf,level,moby,moby_collision,rac2::{gameplay,level::{LevelWad,Lump,Part}},sha1,wad};

fn scan(root:&Path, level_id:u32, out:&mut impl Write)->Result<usize,Box<dyn std::error::Error>> {
    let file=fs::read(root.join(format!("G/LEVEL{level_id}.WAD")))?;
    let w=LevelWad::parse(&file)?;
    // File slot and internal header id are distinct original metadata. In this
    // disc slot 21 carries id 30; do not rewrite it or refuse its inventory.
    writeln!(out,"LEVEL_HEADER\t{level_id}\t{}\t-\t-\t-\t-\t-\t-\tORIGINAL_INTERNAL_ID_NOT_ASSUMED_TO_EQUAL_FILE_SLOT",w.id)?;
    let data=wad::decompress(w.lump(Lump::CoreData)?.ok_or("no core data")?)?;
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex)?.ok_or("no core index")?,data.len())?;
    let gp=wad::decompress(w.part_bytes(Part::Gameplay)?.ok_or("no gameplay part")?)?;
    let instances=gameplay::moby_instances(&gp)?;
    let collision=moby_collision::parse_level(&core,&data)?;
    let mut placed:BTreeMap<i32,(usize,usize,usize)>=BTreeMap::new();
    for i in &instances {
        let p=gameplay::unrelocated_pvar(&gp,i.base.pvar_index)?;
        let record=placed.entry(i.base.o_class).or_default();record.0+=1;
        record.1+=p.map_or(0,|p|p.len());record.2+=(i.base.spawn_flags!=0) as usize;
    }
    let gp_hash=sha1::hex(&sha1::sha1(&gp));
    writeln!(out,"LEVEL\t{level_id}\t-\t{}\t{}\t{}\t{}\t{}\t{gp_hash}\tSTATIC_ONLY_GAMEPLAY_NOT_RUN",
        file.len(),data.len(),instances.len(),core.moby_classes.iter().filter(|c|c.offset_in_asset_wad>0).count(),collision.len())?;
    let mut class_errors=0;
    for c in &core.moby_classes {
        if c.offset_in_asset_wad<=0 {continue;}
        let Some(blob)=core.block(&data,&format!("moby_class/{:04}",c.o_class)) else {return Err("missing class block".into())};
        let (n,pvars,conditional)=placed.remove(&c.o_class).unwrap_or_default();
        let h:moby::MobyClassHeader=Buf(blob).pod(0,"GC class header")?;
        if h.rac1_byte_b!=0 {writeln!(out,"LEGACY_CLASS\t{level_id}\t{:#x}\t-\t-\t-\t-\t-\t-\tORIGINAL_GC_OLD_FORMAT_CONVERSION_REQUIRED",c.o_class)?;}
        let coll=collision.iter().find(|(id,_)|*id==c.o_class).map(|(_,c)|c);
        let triangles=coll.map_or(0,|c|c.faces.len());let primitives=coll.map_or(0,|c|c.prims.len());
        // Parsing geometry never grants a class a gameplay handler.
        let status=match moby::parse_moby_class_for(Game::Rac2,blob) {
            Ok(_)=>"STRUCTURAL_CLASS_PASS_NATIVE_CLASS_BEHAVIOUR_NOT_PORTED".to_owned(),
            Err(e)=>{class_errors+=1;format!("CLASS_PARSE_BLOCKER:{}",e.to_string().replace(['\t','\n','\r']," "))},
        };
        writeln!(out,"CLASS\t{level_id}\t{:#x}\t{n}\t{pvars}\t{conditional}\t{}\t{}\t{triangles}:{primitives}\t{status}",c.o_class,h.joint_count,h.sequence_count)?;
    }
    for (class,(n,bytes,conditional)) in placed {
        writeln!(out,"EXTERNAL_CLASS\t{level_id}\t{class:#x}\t{n}\t{bytes}\t{conditional}\t-\t-\t-\tREQUIRES_EXTERNAL_RESOURCE_OR_DYNAMIC_REGISTRATION")?;
    }
    for (name,_) in gameplay::SECTIONS {
        if let Some(bytes)=gameplay::section(&gp,name)? {writeln!(out,"SECTION\t{level_id}\t{name}\t{}\t-\t-\t-\t-\t-\tEXTENT_ONLY_SEMANTICS_NOT_IMPLIED",bytes.len())?;}
    }
    for kind in ["AUDIO","SCENE"] {
        let path=root.join(format!("G/{kind}{level_id}.WAD"));
        let len=fs::metadata(path)?.len();
        writeln!(out,"DEPENDENCY\t{level_id}\t{kind}\t{len}\t-\t-\t-\t-\t-\tPRESENT_CONTENT_NOT_DECODED_OR_EXECUTED")?;
    }
    Ok(class_errors)
}

fn main()->Result<(),Box<dyn std::error::Error>> {
    let args=std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=2 {return Err("usage: rc2_deep_scan PRIVATE_TIER0_DIR PRIVATE_OUTPUT.tsv".into());}
    let root=Path::new(&args[0]);let boot=fs::read(root.join("SCUS_972.68"))?;
    let hash=sha1::hex(&sha1::sha1(&boot));
    if hash!="4124a79c1e5a1ceb56f514df0dbbc11f072af56f" {return Err("requires original SCUS_972.68 NTSC-U v2.00 executable".into());}
    let mut out=BufWriter::new(fs::File::create_new(&args[1])?);
    writeln!(out,"# RC2 STATIC_DEEP_SCAN; original boot SHA1={hash}; no gameplay/GPU/audio execution; zero native GC class handlers certified")?;
    writeln!(out,"kind\tlevel\tclass_or_section\tcount_or_bytes\tpvar_bytes_or_core_bytes\tconditional_or_instances\tjoints_or_classes\tsequences_or_collision_classes\tcollision_or_gameplay_sha1\tstatus")?;
    let mut failures=0;let mut class_errors=0;
    for level in 0..27 {
        match scan(root,level,&mut out) {
            Ok(errors)=>class_errors+=errors,
            Err(e)=>{failures+=1;writeln!(out,"LEVEL_ERROR\t{level}\t-\t-\t-\t-\t-\t-\t-\t{}",e.to_string().replace(['\t','\n','\r']," "))?;},
        }
        out.flush()?;
    }
    println!("RC2_STATIC_DEEP_SCAN_FINISHED levels_attempted=27 level_errors={failures} class_errors={class_errors}; NO_PLAYABILITY_CLAIM");
    if failures!=0 || class_errors!=0 {return Err(format!("{failures} level inventories / {class_errors} class parses blocked; see output").into());}
    Ok(())
}
