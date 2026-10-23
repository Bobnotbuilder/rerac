//! Opt-in PRIVATE data checks. Not run by public CI and no disc bytes in source.
//! RC2_PRIVATE_SCENE=<Build7 prepared dir> RC2_PRIVATE_ISO=<owned ISO>
//! cargo test -p rc-data --test rc2_scene -- --ignored
use rc_formats::{iso9660::IsoImage, rc2, rc2_tie, sha1, wad};

#[test]
#[ignore = "requires private Build7 Aranos conversion; no game data on public CI"]
fn aranos_preview_upload_plan() {
    let root = std::path::PathBuf::from(std::env::var_os("RC2_PRIVATE_SCENE").expect("RC2_PRIVATE_SCENE"));
    for (lod, expected) in [33710, 26522, 26416].into_iter().enumerate() {
        let scene = rc_data::rc2_preview::load_aranos(&root, lod as u32).unwrap();
        assert_eq!(scene.triangles, expected);
        assert_eq!(scene.images.len(), 93);
        assert_eq!(scene.draws.iter().map(|d| d.positions.len()).sum::<usize>(), expected*3);
        assert!(scene.min.iter().chain(scene.max.iter()).all(|v| v.is_finite()));
        for draw in &scene.draws {
            assert_eq!(draw.positions.len(), draw.st.len());
            assert_eq!(draw.positions.len(), draw.rgba.len());
            assert!(draw.material < 93);
        }
        println!("Aranos preview LOD{lod}: {} triangles, {} ordered draws, bounds {:?}..{:?}; CPU only", scene.triangles, scene.draws.len(), scene.min, scene.max);
    }
}

#[test]
#[ignore = "requires private Build7 Aranos conversion; no game data on public CI"]
fn aranos_build7_scene() {
    let root =
        std::path::PathBuf::from(std::env::var_os("RC2_PRIVATE_SCENE").expect("RC2_PRIVATE_SCENE"));
    let path = root.join("terrain.mesh");
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 14_210_288);
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        sha1::hex(&sha1::sha1(&bytes)),
        "db5405c664c965e8470aed76f50ebd07acceb7b7"
    );
    let scene = rc_data::rc2_scene::Terrain::read(&bytes, 93).unwrap();
    assert_eq!(scene.fragments, 833);
    assert_eq!(scene.counts, [33710, 26522, 26416]);
    for lod in 0..3 {
        assert_eq!(scene.lod(lod).unwrap().count(), scene.counts[lod as usize]);
    }
    for material in 0..93 {
        for mip in 0..3 {
            let path = root.join(format!("terrain-{material:03}-mip{mip}.rgba"));
            assert!(std::fs::metadata(&path).unwrap().len() <= 8 + 1024 * 1024 * 4);
            let bytes = std::fs::read(path).unwrap();
            rc_data::rc2_scene::rgba_image(&bytes).unwrap();
        }
    }
}

#[test]
#[ignore = "requires local owned SCUS97268 v2 ISO; no ISO on public CI"]
fn aranos_gc_tie_vertex_regions() {
    let path = std::env::var_os("RC2_PRIVATE_ISO").expect("RC2_PRIVATE_ISO");
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 3_824_615_424);
    let iso = IsoImage::open(path).unwrap();
    let boot = iso.find("/SCUS_972.68").unwrap();
    assert_eq!(boot.size, 2_621_256);
    let elf = iso
        .read_bytes(boot.lba as u64 * 2048, boot.size as u64)
        .unwrap();
    assert_eq!(
        sha1::hex(&sha1::sha1(&elf)),
        "4124a79c1e5a1ceb56f514df0dbbc11f072af56f"
    );
    let file = iso.find("/G/LEVEL0.WAD").unwrap();
    let header = iso.read_bytes(file.lba as u64 * 2048, 96).unwrap();
    let plan = rc2::level_plan(&header, file.size as usize, 0).unwrap();
    let r = &plan
        .lumps
        .iter()
        .find(|r| r.name == "primary")
        .unwrap()
        .bytes;
    let bytes = iso
        .read_bytes(file.lba as u64 * 2048 + r.start as u64, r.len() as u64)
        .unwrap();
    let p = rc2::primary(&bytes).unwrap();
    let data = wad::decompress_bounded(p.compressed_core, rc2::MAX_DECODED_CORE).unwrap();
    let c = rc2::core(p.index, data.len()).unwrap();
    let mut bounds = vec![data.len(), c.textures_base, c.sky, c.collision];
    bounds.extend(
        c.mobys
            .iter()
            .chain(&c.ties)
            .chain(c.shrubs.iter().map(|s| &s.base))
            .map(|c| c.offset_in_asset_wad as usize),
    );
    bounds.sort_unstable();
    bounds.dedup();
    assert_eq!(c.ties.len(), 74);
    let (mut packets, mut vertices) = (0, 0);
    for e in &c.ties {
        let start = e.offset_in_asset_wad as usize;
        let end = *bounds.iter().find(|&&end| end > start).unwrap();
        let class = rc2_tie::read(&data[start..end], e.o_class).unwrap();
        packets += class.packets.len();
        vertices += class
            .packets
            .iter()
            .map(|p| p.dinky.len() + p.fat.len())
            .sum::<usize>();
    }
    assert_eq!(packets, 1876);
    assert_eq!(vertices, 93418);
    // No assertion of GS-slot topology, lit colour, instance transform or pixels.
}
