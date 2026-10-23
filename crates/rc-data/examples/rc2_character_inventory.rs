//! Read-only original-input check. Prints counts/hashes only; no asset output.
//! cargo run -p rc-data --example rc2_character_inventory -- /private/owned.iso
use rc_formats::{iso9660::IsoImage, rc2_anim::{self, Sequence}, sha1};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let path=std::env::args_os().nth(1).ok_or("expected owned ISO path")?;
    let iso=IsoImage::open(path)?;
    let c=rc_data::rc2_character::load_aranos(&iso)?;
    let hash=|b:&[u8]|sha1::hex(&sha1::sha1(b));
    println!("class_sha1={} mesh_sha1={} texture_list_sha1={}",hash(&c.class_bytes),hash(&c.mesh_bytes),hash(&c.texture_bytes));
    println!("joints={} skeleton_matrices={} hierarchy_records={} textures={}",c.class.header.joint_count,c.class.skeleton.matrices.len(),c.class.skeleton.trans.len(),c.images.len());
    for (name,list) in [("high",&c.mesh.high_lod),("low",&c.mesh.low_lod),("metal",&c.mesh.metal)] {
        println!("lod={name} packets={} vertices={} triangles={}",list.len(),list.iter().map(|s|s.vertices.len()).sum::<usize>(),list.iter().map(|s|s.triangles.len()).sum::<usize>());
    }
    for (i,image) in c.images.iter().enumerate() { println!("texture={i} dimensions={}x{} raw_rgba_sha1={}",image.width,image.height,hash(&image.rgba)); }
    let (mut resident,mut regular,mut special,mut frames)=(0,0,0,0);
    for (slot,blob) in c.sequences.iter().enumerate() {
        if let Some(b)=blob {
            let seq=rc2_anim::parse(b,c.class.header.joint_count as usize)?;
            let n=match seq { Sequence::Regular(s)=>{regular+=1;s.frames.len()},Sequence::Special(s)=>{special+=1;s.frames.len()} };
            resident+=1;frames+=n;println!("sequence={slot} bytes={} frames={n} sha1={}",b.len(),hash(b));
        }
    }
    println!("resident_sequences={resident} regular={regular} special={special} stored_frames={frames}");
    println!("pose_evaluated=false model_rendered=false gameplay=false");
    Ok(())
}
