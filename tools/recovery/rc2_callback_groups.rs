//! Read-only Aranos callback grouping through the existing validated readers.
//! Build against rc-formats, then pass a private LEVEL0.WAD and private output.
//! Source addresses are metadata, never runtime call targets. A registry's
//! third word remains opaque: it is not labelled initialization/damage/draw.
use std::{collections::BTreeMap, env, fs};
use rc_formats::{rac2::{classes::AranosClassUpdates, gameplay, level::{LevelWad,Lump,Part}}, wad};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len()!=3 {return Err("usage: rc2-callback-groups <private LEVEL0.WAD> <private report.json>".into());}
    let file=fs::read(&args[1])?;
    let level=LevelWad::parse(&file)?;
    let registry=AranosClassUpdates::parse(0,level.lump(Lump::Overlay)?.ok_or("missing overlay")?)?;
    let gameplay=wad::decompress(level.part_bytes(Part::Gameplay)?.ok_or("missing gameplay")?)?;
    let placements=gameplay::moby_instances(&gameplay)?;
    let mut counts=BTreeMap::<i32,usize>::new();
    for p in &placements {*counts.entry(p.base.o_class as i32).or_default()+=1;}
    let mut groups=BTreeMap::<u32,Vec<(i32,u32,usize)>>::new();
    for entry in registry.entries() {
        groups.entry(entry.source_update).or_default().push((entry.class,entry.source_metadata,*counts.get(&entry.class).unwrap_or(&0)));
    }
    let mut output=format!("{{\n  \"status\":\"STATIC_CALLBACK_GROUPS_NOT_GAMEPLAY_SUPPORT\",\n  \"placements\":{},\n  \"unique_placement_classes\":{},\n  \"registry_entries\":{},\n  \"unique_update_addresses\":{},\n  \"groups\":[\n",placements.len(),counts.len(),registry.entries().len(),groups.len());
    for (i,(source,entries)) in groups.iter().enumerate() {
        if i!=0 {output.push_str(",\n");}
        output.push_str(&format!("    {{\"source_update\":\"{source:08x}\",\"classes\":["));
        for (j,(class,metadata,count)) in entries.iter().enumerate() {
            if j!=0 {output.push(',');}
            output.push_str(&format!("{{\"class\":\"{class:04x}\",\"opaque_metadata\":\"{metadata:08x}\",\"placement_count\":{count}}}"));
        }
        output.push_str("]}");
    }
    output.push_str("\n  ],\n  \"unbound_placement_classes\":[");
    let mut first=true;
    for (class,count) in &counts {
        if registry.entries().iter().all(|e|e.class!=*class) {
            if !first {output.push(',');} first=false;
            output.push_str(&format!("{{\"class\":\"{class:04x}\",\"placement_count\":{count},\"fallback_source\":\"{:08x}\"}}",registry.source_update(*class)));
        }
    }
    output.push_str("],\n  \"limitations\":[\"Registry update address equality is not proof of identical class parameters or state\",\"Does not infer damage, draw, destruction or initialization identity from opaque metadata\",\"Static placements are not filtered native spawn decisions\"]\n}\n");
    fs::write(&args[2],output)?;
    println!("STATIC_CALLBACK_GROUPS_PASS placements={} placement_classes={} registry_entries={} update_addresses={}; NOT_GAMEPLAY_VALIDATION",placements.len(),counts.len(),registry.entries().len(),groups.len());
    Ok(())
}
