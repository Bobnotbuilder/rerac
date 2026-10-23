//! Private read-only static microcode comparison, NOT a captured execution.
//! Only ELF section metadata and hashes printed; no original bytes emitted.
use rc_formats::{iso9660::IsoImage, sha1, Buf};
use std::collections::BTreeMap;
fn boot(path: &std::ffi::OsStr, name: &str, hash: &str) -> Vec<u8> {
    let iso = IsoImage::open(path).unwrap();
    let f = iso.find(name).unwrap();
    let b = iso.read_bytes(f.lba as u64 * 2048, f.size as u64).unwrap();
    assert_eq!(sha1::hex(&sha1::sha1(&b)), hash);
    b
}
fn programs(bytes: &[u8], label: &str) -> BTreeMap<String, Vec<(u32, Vec<u8>)>> {
    let b = Buf(bytes);
    assert_eq!(&bytes[..4], b"\x7fELF");
    assert_eq!(&bytes[4..6], &[1, 1]);
    let sh = b.u32(32).unwrap() as usize;
    let stride = b.u16(46).unwrap() as usize;
    let count = b.u16(48).unwrap() as usize;
    if count == 0 {
        println!("{label} no_section_table=YES");
        return BTreeMap::new();
    }
    assert!(stride >= 40);
    b.check(sh, count * stride, "ELF sections").unwrap();
    let ns = sh + b.u16(50).unwrap() as usize * stride;
    let strings = b
        .sub(
            b.u32(ns + 16).unwrap() as usize,
            b.u32(ns + 20).unwrap() as usize,
            "section names",
        )
        .unwrap()
        .bytes();
    let str_at = |s: &[u8], off: usize| {
        let tail = s.get(off..).unwrap();
        let n = tail.iter().position(|&x| x == 0).unwrap();
        String::from_utf8(tail[..n].to_vec()).unwrap()
    };
    let sections = (0..count)
        .map(|i| {
            let at = sh + i * stride;
            (str_at(strings, b.u32(at).unwrap() as usize), at)
        })
        .collect::<Vec<_>>();
    let overlay_table = sections
        .iter()
        .find(|(name, _)| name == ".DVP.ovlytab")
        .unwrap()
        .1;
    let overlay_strings = sections
        .iter()
        .find(|(name, _)| name == ".DVP.ovlystrtab")
        .unwrap()
        .1;
    let st = b
        .sub(
            b.u32(overlay_strings + 16).unwrap() as usize,
            b.u32(overlay_strings + 20).unwrap() as usize,
            "DVP name table",
        )
        .unwrap()
        .bytes();
    let table = b
        .sub(
            b.u32(overlay_table + 16).unwrap() as usize,
            b.u32(overlay_table + 20).unwrap() as usize,
            "DVP overlay table",
        )
        .unwrap();
    assert_eq!(table.len() % 12, 0);
    let mut loaded = BTreeMap::new();
    for at in (0..table.len()).step_by(12) {
        let name = str_at(st, table.u32(at).unwrap() as usize);
        let ee = table.u32(at + 4).unwrap();
        let vu = table.u32(at + 8).unwrap();
        loaded.insert(name, (ee, vu));
    }
    let mut out: BTreeMap<String, Vec<(u32, Vec<u8>)>> = BTreeMap::new();
    for i in 0..count {
        let at = sh + i * stride;
        let name = str_at(strings, b.u32(at).unwrap() as usize);
        if !name.starts_with(".DVP.overlay") {
            continue;
        }
        let offset = b.u32(at + 16).unwrap() as usize;
        let n = b.u32(at + 20).unwrap() as usize;
        let address = b.u32(at + 12).unwrap();
        let kind = b.u32(at + 4).unwrap();
        println!("{label} section={name} address=0x{address:x} file_offset=0x{offset:x} bytes={n} type={kind}");
        if kind == 8 {
            continue;
        }
        let words = name.split('.').collect::<Vec<_>>();
        if words.len() < 8 {
            continue;
        }
        let id = words[5].to_string();
        let vu_address = u32::from_str_radix(words[4].trim_start_matches("0x"), 16).unwrap();
        let declared = b.sub(offset, n, "DVP debug section range").unwrap().bytes();
        let &(ee, vu) = loaded
            .get(&name)
            .expect("DVP chunk must have original upload table reference");
        assert_eq!(vu, vu_address);
        let ph = b.u32(28).unwrap() as usize;
        let pn = b.u16(44).unwrap() as usize;
        let ps = b.u16(42).unwrap() as usize;
        let mut actual = None;
        for i in 0..pn {
            let p = ph + i * ps;
            if b.u32(p).unwrap() != 1 {
                continue;
            }
            let va = b.u32(p + 8).unwrap();
            let len = b.u32(p + 16).unwrap();
            if ee >= va && (ee - va) as u64 + n as u64 <= len as u64 {
                let off = b.u32(p + 4).unwrap() as usize + (ee - va) as usize;
                actual = Some(b.sub(off, n, "loaded VU program bytes").unwrap().bytes());
            }
        }
        let actual = actual.expect("DVP upload must resolve through PT_LOAD");
        let data = actual.to_vec();
        println!("{label} upload_chunk_verified program={id} vu_offset=0x{vu:x} ee_address=0x{ee:x} bytes={n} debug_section_matches_loaded={}",declared==actual);
        out.entry(id).or_default().push((vu_address, data));
    }
    for (id, chunks) in &mut out {
        chunks.sort_by_key(|(a, _)| *a);
        let data = chunks
            .iter()
            .flat_map(|(_, v)| v.iter().copied())
            .collect::<Vec<_>>();
        println!(
            "{label} program={id} chunks={} total_bytes={} sha1={}",
            chunks.len(),
            data.len(),
            sha1::hex(&sha1::sha1(&data))
        );
    }
    out
}
fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();
    let r1 = boot(
        &args[1],
        "/SCUS_971.99",
        "72fd1de379dafd8f243dceeb3be50980f3fd3cd5",
    );
    let r2 = boot(
        &args[2],
        "/SCUS_972.68",
        "4124a79c1e5a1ceb56f514df0dbbc11f072af56f",
    );
    let a = programs(&r1, "rac1");
    let c = programs(&r2, "gc");
    for (aid, av) in &a {
        let aa = av
            .iter()
            .flat_map(|(_, v)| v.iter().copied())
            .collect::<Vec<_>>();
        for (cid, cv) in &c {
            let cc = cv
                .iter()
                .flat_map(|(_, v)| v.iter().copied())
                .collect::<Vec<_>>();
            if aa == cc {
                println!(
                    "exact_static_match rac1={aid} gc={cid} bytes={} executed_parity=NO",
                    aa.len()
                );
            }
        }
    }
    println!(
        "static_program_inventory rac1={} gc={} live_execution=NO",
        a.len(),
        c.len()
    );
}
