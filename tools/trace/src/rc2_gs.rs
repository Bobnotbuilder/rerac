//! Read-only, dev-only GC capture inventory. Never linked into the game.
//! Independently implements file-format facts in PCSX2 v2.8.2 GSDump.h/.cpp
//! and GSRegs.h (reference-only GPL; no PCSX2 implementation linked/copied).
//! Post-VU samples are evidence, not a reconstructed camera or proof of terrain
//! identity. Frozen GS state is opaque: unseen registers remain UNKNOWN.
use std::collections::BTreeMap;

pub const MAX_DUMP: usize = 128 << 20;
const MAX_TRANSFER: usize = 16 << 20;
const SAMPLE_LIMIT: usize = 64;

#[derive(Debug, Clone)]
pub struct Sample {
    pub sequence: usize,
    pub path: u8,
    pub register: u8,
    pub raw: [u8; 16],
    /// Last observed PACKED STQ input, not the GS's latched/sanitised RGBAQ.
    pub stq_bits: Option<[u32; 3]>,
    pub prim: Option<u64>,
    /// Only explicitly observed register writes, NOT assumed defaults or a decode
    /// of the version-dependent frozen state. Both GS contexts are retained.
    pub state: BTreeMap<u8, u64>,
}

impl Sample {
    /// Raw post-VU GS output:12.4 X/Y,24-bit XYZF Z or32-bit XYZ Z. This is
    /// not an inverse camera transform. XYZ3/XYZF3 and ADC never kick a draw.
    pub fn screen_fields(&self) -> ([u16; 2], u32, Option<u8>, bool) {
        let w2 = u32le(&self.raw[8..]);
        let w3 = u32le(&self.raw[12..]);
        let xyzf = matches!(self.register, 4 | 12);
        let z = if xyzf { (w2 >> 4) & 0xffffff } else { w2 };
        let fog = xyzf.then_some(((w3 >> 4) & 255) as u8);
        let kick = matches!(self.register, 4 | 5) && w3 & 0x8000 == 0;
        (
            [u32le(&self.raw) as u16, u32le(&self.raw[4..]) as u16],
            z,
            fog,
            kick,
        )
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub sample_skip: usize,
    pub crc: u32,
    pub state_version: u32,
    pub transfers: usize,
    pub vsyncs: usize,
    pub vertices: usize,
    pub tags: usize,
    pub image_qwords: usize,
    pub ad_writes: usize,
    pub samples: Vec<Sample>,
    pub writes_by_register: BTreeMap<u8, usize>,
}

fn take<'a>(data: &'a [u8], at: &mut usize, n: usize) -> Result<&'a [u8], String> {
    if *at > data.len() || n > data.len() - *at {
        return Err("RC2_GS_CAPTURE_TRUNCATED".into());
    }
    let v = &data[*at..*at + n];
    *at += n;
    Ok(v)
}
fn u32le(b: &[u8]) -> u32 {
    u32::from_le_bytes(b[..4].try_into().unwrap())
}
fn u64le(b: &[u8]) -> u64 {
    u64::from_le_bytes(b[..8].try_into().unwrap())
}

#[derive(Default)]
struct Path {
    pending: Vec<u8>,
    tag: Option<Tag>,
}
struct Tag {
    remaining: usize,
    cursor: usize,
    nreg: usize,
    regs: u64,
    image: bool,
}

#[derive(Default)]
struct Gif {
    paths: [Path; 4],
    state: BTreeMap<u8, u64>,
    // GS state is shared by paths. Conservatively invalidate on each new tag
    // and A+D ST/RGBAQ; never resurrect a stale per-path sample.
    stq: Option<[u32; 3]>,
    report: Report,
}
impl Gif {
    fn feed(&mut self, path: u8, bytes: &[u8]) -> Result<(), String> {
        if path > 3 || bytes.len() % 16 != 0 {
            return Err("RC2_GS_CAPTURE_TRANSFER_LAYOUT".into());
        }
        let p = &mut self.paths[path as usize];
        if p.pending.len() + bytes.len() > MAX_TRANSFER {
            return Err("RC2_GS_CAPTURE_TRANSFER_BUDGET".into());
        }
        p.pending.extend_from_slice(bytes);
        let mut at = 0;
        while p.pending.len() - at >= 16 {
            let raw: [u8; 16] = p.pending[at..at + 16].try_into().unwrap();
            at += 16;
            if p.tag.is_none() {
                self.report.tags += 1;
                let lo = u64le(&raw);
                let regs = u64le(&raw[8..]);
                let loops = (lo & 0x7fff) as usize;
                let mode = (lo >> 58) & 3;
                if mode == 1 {
                    return Err("RC2_GS_CAPTURE_REGLIST_NOT_YET_DECODED".into());
                }
                let nr = ((lo >> 60) & 15) as usize;
                let nreg = if nr == 0 { 16 } else { nr };
                // PRE only acts in PACKED. Do not manufacture a primitive if
                // it is inherited from the opaque initial frozen state.
                if mode == 0 && (lo >> 46) & 1 != 0 {
                    self.state.insert(0, (lo >> 47) & 0x7ff);
                }
                if mode == 0 {
                    self.stq = None;
                } // GIF tag resets Q; no synthetic ST
                let image = mode >= 2;
                let remaining = if image { loops } else { loops * nreg };
                if remaining > 0 {
                    p.tag = Some(Tag {
                        remaining,
                        cursor: 0,
                        nreg,
                        regs,
                        image,
                    });
                }
                continue;
            }
            let t = p.tag.as_mut().unwrap();
            if t.image {
                self.report.image_qwords += 1;
            } else {
                let register = ((t.regs >> ((t.cursor % t.nreg) * 4)) & 15) as u8;
                match register {
                    0 => {
                        self.state.insert(0, u64le(&raw) & 0x7ff);
                    }
                    2 => {
                        self.stq = Some([u32le(&raw), u32le(&raw[4..]), u32le(&raw[8..])]);
                    }
                    4 | 5 | 12 | 13 => {
                        self.report.vertices += 1;
                        if self.report.vertices > self.report.sample_skip
                            && self.report.samples.len() < SAMPLE_LIMIT
                        {
                            let wanted = [
                                0, 6, 7, 8, 9, 0x14, 0x15, 0x18, 0x19, 0x1a, 0x1b, 0x34, 0x35,
                                0x36, 0x37, 0x16, 0x17, 0x3b, 0x3d, 0x40, 0x41, 0x42, 0x43, 0x44,
                                0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
                            ];
                            let state = wanted
                                .into_iter()
                                .filter_map(|r| self.state.get(&r).map(|&v| (r, v)))
                                .collect();
                            self.report.samples.push(Sample {
                                sequence: self.report.vertices,
                                path,
                                register,
                                raw,
                                stq_bits: self.stq,
                                prim: self.state.get(&0).copied(),
                                state,
                            });
                        }
                    }
                    14 => {
                        let address = raw[8];
                        self.report.ad_writes += 1;
                        *self.report.writes_by_register.entry(address).or_default() += 1;
                        self.state.insert(address, u64le(&raw));
                        // A+D ST/RGBAQ use different packing; never reuse a
                        // stale packed-STQ sample after those writes.
                        if matches!(address, 1 | 2) {
                            self.stq = None;
                        }
                    }
                    6..=9 => {
                        self.state.insert(register, u64le(&raw));
                    }
                    _ => {} // other packed register payloads remain in the original private dump
                }
                t.cursor += 1;
            }
            t.remaining -= 1;
            if t.remaining == 0 {
                p.tag = None;
            }
        }
        p.pending.drain(..at);
        Ok(())
    }
}

/// Modern uncompressed .gs only. XZ/Zstd, legacy headers and unsupported GIF
/// encodings fail explicitly instead of returning a misleading successful audit.
pub fn inspect(bytes: &[u8]) -> Result<Report, String> {
    inspect_at(bytes, 0)
}

/// Sampling window does not discard preceding state or assume the first sky/
/// HUD vertices belong to terrain. Re-read the same private dump as needed.
pub fn inspect_at(bytes: &[u8], sample_skip: usize) -> Result<Report, String> {
    if bytes.len() > MAX_DUMP {
        return Err("RC2_GS_CAPTURE_FILE_BUDGET".into());
    }
    let mut at = 0;
    if take(bytes, &mut at, 4)? != [255; 4] {
        return Err("RC2_GS_CAPTURE_REQUIRES_MODERN_UNCOMPRESSED_GS".into());
    }
    let header_len = u32le(take(bytes, &mut at, 4)?) as usize;
    if !(36..=16 << 20).contains(&header_len) {
        return Err("RC2_GS_CAPTURE_HEADER_BUDGET".into());
    }
    let h = take(bytes, &mut at, header_len)?;
    let state_size = u32le(&h[4..]) as usize;
    if state_size > 64 << 20 {
        return Err("RC2_GS_CAPTURE_STATE_BUDGET".into());
    }
    let mut serial_at = u32le(&h[8..]) as usize;
    let serial_len = u32le(&h[12..]) as usize;
    if serial_at < 36 || take(h, &mut serial_at, serial_len)? != b"SCUS-97268" {
        return Err("RC2_GS_CAPTURE_WRONG_GAME".into());
    }
    take(bytes, &mut at, state_size)?; // opaque, not a register default
    take(bytes, &mut at, 0x2000)?; // privileged register block
    let mut g = Gif::default();
    g.report.sample_skip = sample_skip;
    g.report.crc = u32le(&h[16..]);
    g.report.state_version = u32le(h);
    while at < bytes.len() {
        let event = take(bytes, &mut at, 1)?[0];
        match event {
            0 => {
                let path = take(bytes, &mut at, 1)?[0];
                let length = u32le(take(bytes, &mut at, 4)?) as usize;
                if length > MAX_TRANSFER {
                    return Err("RC2_GS_CAPTURE_TRANSFER_BUDGET".into());
                }
                let payload = take(bytes, &mut at, length)?;
                g.feed(path, payload)?;
                g.report.transfers += 1;
            }
            1 => {
                take(bytes, &mut at, 1)?;
                g.report.vsyncs += 1;
            }
            2 => {
                take(bytes, &mut at, 4)?;
            } // FIFO size only, not payload
            3 => {
                take(bytes, &mut at, 0x2000)?;
            }
            _ => return Err("RC2_GS_CAPTURE_EVENT_UNSUPPORTED".into()),
        }
    }
    if g.paths
        .iter()
        .any(|p| p.tag.is_some() || !p.pending.is_empty())
    {
        return Err("RC2_GS_CAPTURE_INCOMPLETE_GIF_PACKET".into());
    }
    if g.report.vsyncs == 0 || g.report.transfers == 0 {
        return Err("RC2_GS_CAPTURE_NO_COMPLETED_FRAME".into());
    }
    Ok(g.report)
}

impl Report {
    /// Private report: deliberately includes no screenshot, texture or RAM bytes.
    /// Raw vertex/register samples are still game-derived evidence; don't upload.
    pub fn json(&self) -> String {
        let samples=self.samples.iter().map(|s|{
            let state=s.state.iter().map(|(r,v)|format!("\"{r:02x}\":\"{v:016x}\"")).collect::<Vec<_>>().join(",");
            let raw=s.raw.iter().map(|b|format!("{b:02x}")).collect::<String>();
            let stq=s.stq_bits.map(|v|format!("[{}, {}, {}]",v[0],v[1],v[2])).unwrap_or("null".into());
            let prim=s.prim.map(|v|v.to_string()).unwrap_or("null".into());
            let (xy,z,fog,kick)=s.screen_fields();
            let fog=fog.map(|v|v.to_string()).unwrap_or("null".into());
            format!("{{\"sequence\":{},\"path\":{},\"register\":{},\"raw\":\"{raw}\",\"packed_stq_input_bits\":{stq},\"prim\":{prim},\"xy_fixed\":[{},{}],\"z_raw\":{z},\"fog_raw\":{fog},\"draw_kick\":{kick},\"observed_register_state\":{{{state}}}}}",s.sequence,s.path,s.register,xy[0],xy[1])
        }).collect::<Vec<_>>().join(",");
        format!("{{\"contract\":\"RC2_GS_CAPTURE_1\",\"serial\":\"SCUS-97268\",\"crc\":{},\"state_version\":{},\"transfers\":{},\"vsyncs\":{},\"packed_vertex_writes\":{},\"tags\":{},\"ad_writes\":{},\"image_qwords\":{},\"frozen_state_decoded\":false,\"terrain_attribution\":\"UNPROVEN\",\"samples\":[{samples}]}}\n",self.crc,self.state_version,self.transfers,self.vsyncs,self.vertices,self.tags,self.ad_writes,self.image_qwords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dump(parts: &[&[u8]]) -> Vec<u8> {
        let mut b = vec![255; 4];
        b.extend(46u32.to_le_bytes());
        let mut h = vec![0u8; 36];
        h[8..12].copy_from_slice(&36u32.to_le_bytes());
        h[12..16].copy_from_slice(&10u32.to_le_bytes());
        b.extend(h);
        b.extend(b"SCUS-97268");
        b.extend([0; 0x2000]);
        for p in parts {
            b.extend([0, 1]);
            b.extend((p.len() as u32).to_le_bytes());
            b.extend(*p);
        }
        b.extend([1, 0]);
        b
    }
    fn packet() -> Vec<u8> {
        let mut b = vec![];
        let tag = 1 | (3u64 << 60);
        b.extend(tag.to_le_bytes());
        b.extend(0x42eu64.to_le_bytes());
        let mut ad = [0u8; 16];
        ad[..8].copy_from_slice(&0x5360bu64.to_le_bytes());
        ad[8] = 0x47;
        b.extend(ad);
        b.extend(0.25f32.to_le_bytes());
        b.extend(0.5f32.to_le_bytes());
        b.extend(1f32.to_le_bytes());
        b.extend([0; 4]);
        b.extend([0; 16]);
        b
    }
    #[test]
    fn fragmented_transfer_preserves_state_and_float_bits() {
        let p = packet();
        let r = inspect(&dump(&[&p[..32], &p[32..]])).unwrap();
        assert_eq!(r.vertices, 1);
        assert_eq!(
            r.samples[0].stq_bits,
            Some([0.25f32.to_bits(), 0.5f32.to_bits(), 1f32.to_bits()])
        );
        assert_eq!(r.samples[0].state.get(&0x47), Some(&0x5360b));
        assert!(!r.samples[0].state.contains_key(&0x42));
        assert_eq!(r.samples[0].prim, None);
    }
    #[test]
    fn missing_frame_wrong_identity_and_truncated_fail() {
        let p = packet();
        let mut b = dump(&[&p]);
        b.truncate(b.len() - 2);
        assert!(inspect(&b).is_err());
        b = dump(&[&p]);
        b[44] = b'X';
        assert!(inspect(&b).is_err());
        assert!(inspect(&dump(&[&p[..32]])).is_err());
        assert!(inspect(&[0; 4]).is_err());
    }
    #[test]
    fn unsupported_reglist_fails_explicitly() {
        let mut p = packet();
        let tag = 1 | (1u64 << 58) | (3u64 << 60);
        p[..8].copy_from_slice(&tag.to_le_bytes());
        assert_eq!(
            inspect(&dump(&[&p])).unwrap_err(),
            "RC2_GS_CAPTURE_REGLIST_NOT_YET_DECODED"
        );
    }
    #[test]
    fn sample_budget_does_not_stop_counting() {
        let p = packet();
        let refs = vec![p.as_slice(); 100];
        let r = inspect(&dump(&refs)).unwrap();
        assert_eq!(r.vertices, 100);
        assert_eq!(r.samples.len(), SAMPLE_LIMIT);
        let r = inspect_at(&dump(&refs), 98).unwrap();
        assert_eq!(r.vertices, 100);
        assert_eq!(r.samples.len(), 2);
        assert_eq!(r.samples[0].sequence, 99);
    }
    #[test]
    fn xyzf_fog_adc_and_xyz_width_are_not_interchanged() {
        let p = packet();
        let mut r = inspect(&dump(&[&p])).unwrap();
        let s = &mut r.samples[0];
        s.raw[..4].copy_from_slice(&0x1234u32.to_le_bytes());
        s.raw[4..8].copy_from_slice(&0x5678u32.to_le_bytes());
        s.raw[8..12].copy_from_slice(&(0xabcdefu32 << 4).to_le_bytes());
        s.raw[12..].copy_from_slice(&((173u32 << 4) | 0x8000).to_le_bytes());
        assert_eq!(
            s.screen_fields(),
            ([0x1234, 0x5678], 0xabcdef, Some(173), false)
        );
        s.register = 5;
        s.raw[8..12].copy_from_slice(&0xfedcba98u32.to_le_bytes());
        s.raw[12..].fill(0);
        assert_eq!(
            s.screen_fields(),
            ([0x1234, 0x5678], 0xfedcba98, None, true)
        );
        s.register = 13;
        assert!(!s.screen_fields().3);
    }
    #[test]
    fn interleaved_path_does_not_restore_stale_stq() {
        let p = packet();
        let mut g = Gif::default();
        g.feed(1, &p[..48]).unwrap();
        // A new path's tag without STQ invalidates the diagnostic input.
        let mut q = vec![];
        q.extend((1u64 | (1u64 << 60)).to_le_bytes());
        q.extend(4u64.to_le_bytes());
        q.extend([0; 16]);
        g.feed(2, &q).unwrap();
        g.feed(1, &p[48..]).unwrap();
        assert!(g.report.samples.iter().all(|s| s.stq_bits.is_none()));
    }
    #[test]
    fn captured_nan_is_preserved_not_repaired() {
        let mut p = packet();
        p[40..44].copy_from_slice(&0xffffffffu32.to_le_bytes());
        let r = inspect(&dump(&[&p])).unwrap();
        assert_eq!(r.samples[0].stq_bits.unwrap()[2], 0xffffffff);
        assert!(r.json().contains("4294967295"));
    }
    #[test]
    fn every_prefix_returns_error_or_bounded_report_without_panic() {
        let p = packet();
        let b = dump(&[&p]);
        for end in 0..b.len() {
            assert!(std::panic::catch_unwind(|| inspect(&b[..end])).is_ok());
        }
    }
}
