//! Private original-data contracts. No copyrighted fixtures in the repository.
use rc_formats::{iso9660::IsoImage, rc2_anim::{self, Sequence}, sha1};

#[test]
#[ignore="requires RC2_PRIVATE_ISO; no original game data in public CI"]
fn original_aranos_character_storage() {
    let iso=IsoImage::open(std::env::var_os("RC2_PRIVATE_ISO").expect("RC2_PRIVATE_ISO")).unwrap();
    let c=rc_data::rc2_character::load_aranos(&iso).unwrap();
    let hash=|b:&[u8]|sha1::hex(&sha1::sha1(b));
    assert_eq!(hash(&c.class_bytes),"cab23b88099cc8e7a1196828122050cba5936e76");
    assert_eq!(hash(&c.mesh_bytes),"9398661d6d3f5eec65b54d29bffe03a06360a88f");
    assert_eq!(hash(&c.texture_bytes),"3843c6dfabc355bf60e53ce7a0a362c177d9fc17");
    assert_eq!(c.class.header.joint_count,111);
    assert_eq!(c.class.skeleton.matrices.len(),111); assert_eq!(c.class.skeleton.trans.len(),111);
    assert!(c.class.skeleton.matrices.iter().flatten().flatten().all(|v|v.is_finite()));
    assert_eq!(c.images.iter().map(|v|(v.width,v.height)).collect::<Vec<_>>(),[(256,256),(128,128),(128,128),(32,32)]);
    assert_eq!(c.images.iter().map(|v|hash(&v.rgba)).collect::<Vec<_>>(),[
        "660d5881a45a95c8b5e64017a8bce42c7e6635c0","cc48c997d9d2722b76bc117a86a2dfbf69333efa",
        "a09569d01dc1396d8f7c9b9ad10b8c18a9a364cb","1736e94e424356bed3f08313fb4358ae33c05b9a"]);
    for (list,packets,vertices,triangles) in [(&c.mesh.high_lod,63,5637,6856),(&c.mesh.low_lod,22,2084,2550),(&c.mesh.metal,2,110,102)] {
        assert_eq!(list.len(),packets);
        assert_eq!(list.iter().map(|v|v.vertices.len()).sum::<usize>(),vertices);
        assert_eq!(list.iter().map(|v|v.triangles.len()).sum::<usize>(),triangles);
        assert!(list.iter().all(|v|v.unresolved_duplicates==0));
    }
    let mut resident=0; let mut frames=0;
    assert_eq!(c.sequences.len(),256);
    for b in c.sequences.iter().flatten() {
        resident+=1;
        let Sequence::Special(s)=rc2_anim::parse(b,111).unwrap() else { panic!("unexpected regular storage") };
        assert_eq!(s.base_joint_data.len(),111*6);
        assert!(s.frames.iter().all(|v|v.joint_flags.len()==14));frames+=s.frames.len();
    }
    assert_eq!((resident,frames),(96,2187));
    println!("PASS: original mesh, four textures, 111 skeleton/hierarchy records, 96 special sequences/2187 stored frames; NO pose/gameplay claim");
}
