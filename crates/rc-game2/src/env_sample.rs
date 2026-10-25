//! GC EnvNearestSamplePoint 0x313008. Its packed 0x20-byte records differ
//! from RAC1's 0x30-byte sound-bank records; never call that parser on GC data.
//! Reuses the shared exact VU vector kernels for 0x2eeea0 / 0x2eeff8.
use rc_formats::{buf::{invalid,Buf,Result},rac2::gameplay};
use rc_game::{moby_runtime::Moby,ps2v};

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct SamplePoint {
    pub light:u32,
    pub position:[f32;3],
    pub reverb_depth:i16,
    pub music_track:i16,
    pub source_0e:u8,
    pub source_0f:u8,
    pub ambient:[u8;3],
    pub reverb_type:u8,
    pub reverb_delay:u8,
    pub reverb_feedback:u8,
    pub reverb_enable:u8,
    pub fog_color:[u8;3],
    pub fog_near:i16,
    pub fog_far:i16,
}

/// Loader 0x3034a0..0x3034e4: header+0x8c, count, records+0x10,
/// stride0x20. Reuses the existing GC section-bounds reader.
pub fn read(gameplay_bytes:&[u8])->Result<Vec<SamplePoint>> {
    let Some(section)=gameplay::section(gameplay_bytes,"unknown_35")? else {return Ok(Vec::new());};
    let b=Buf(section);let count=b.i32(0)?;
    if !(0..=1_000_000).contains(&count) {return invalid("GC environment sample count invalid");}
    b.check(0x10,count as usize*0x20,"GC environment samples")?;
    (0..count as usize).map(|i| {
        let r=b.sub(0x10+i*0x20,0x20,"GC environment sample")?;
        Ok(SamplePoint {
            light:r.u32(0)?,position:[r.u16(4)? as f32*0.25,r.u16(6)? as f32*0.25,r.u16(8)? as f32*0.25],
            reverb_depth:r.i16(0xa)?,music_track:r.i16(0xc)?,source_0e:r.u8(0xe)?,source_0f:r.u8(0xf)?,
            ambient:[r.u8(0x10)?,r.u8(0x11)?,r.u8(0x12)?],reverb_type:r.u8(0x13)?,
            reverb_delay:r.u8(0x14)?,reverb_feedback:r.u8(0x15)?,reverb_enable:r.u8(0x16)?,
            fog_color:[r.u8(0x17)?,r.u8(0x18)?,r.u8(0x19)?],fog_near:r.i16(0x1a)?,fog_far:r.i16(0x1c)?,
        })
    }).collect()
}

/// Strict squared-distance comparisons: exactly20units is accepted; the
/// first point wins ties. No sqrt, RNG, allocation or point-order sorting.
pub fn nearest(points:&[SamplePoint],position:[f32;3])->Option<usize> {
    let mut best=1_000_000.0f32;let mut selected=None;
    for (i,p) in points.iter().enumerate() {
        let d=ps2v::vsub(position.map(f32::to_bits),p.position.map(f32::to_bits));
        let distance=f32::from_bits(ps2v::dot_x(d,d));
        if distance<best {best=distance;selected=Some(i);}
    }
    if best>400.0 {None}else{selected}
}

/// Effect ordering at the source boundary. Generating a plan does NOT execute
/// the audio dispatcher32eb88 or fog/projection service2e4af0. A host must bind
/// all effects before treating the complete313008 callback as supported.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Effect {
    Ambient([u8;4]),
    Light(u32),
    /// Original1986c4/depth,1986c8..ca, dirty1986cb |=7.
    Reverb {depth:i32,kind:u8,delay:u8,feedback:u8},
    /// Exact call arguments32eb88(track,1,0x400), includingnegative tracks.
    Music {track:i16,mode:i32,volume:i32},
    /// Original1b1900..0c andGP+2920, then2e4af0.
    Fog {color:[u8;3],near:f32,far:f32,source_0e:f32,source_0f:f32},
}
impl SamplePoint {
    pub fn effects(&self)->Vec<Effect> {
        let mut out=vec![Effect::Ambient([self.ambient[0],self.ambient[1],self.ambient[2],0]),Effect::Light(self.light)];
        if self.reverb_enable!=0 {
            out.push(Effect::Reverb {depth:self.reverb_depth as i32,kind:self.reverb_type,delay:self.reverb_delay,feedback:self.reverb_feedback});
        }
        out.push(Effect::Music {track:self.music_track,mode:1,volume:0x400});
        if self.fog_near<self.fog_far {
            out.push(Effect::Fog {color:self.fog_color,near:self.fog_near as f32*1024.0,far:self.fog_far as f32*1024.0,
                source_0e:self.source_0e as f32,source_0f:self.source_0f as f32});
        }
        out
    }
    /// Exact3132e0 ambient write followed by313008's lightword overwrite.
    /// This is only the Moby-owned portion, not audio/fog service completion.
    pub fn apply_moby(&self,moby:&mut Moby) {
        moby.light=0;
        moby.ambient=[self.ambient[0],self.ambient[1],self.ambient[2],0];
        moby.light=self.light;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point()->SamplePoint {SamplePoint {light:0x12345678,position:[0.;3],reverb_depth:-12,music_track:-1,source_0e:4,source_0f:5,
        ambient:[6,7,8],reverb_type:3,reverb_delay:9,reverb_feedback:10,reverb_enable:1,fog_color:[11,12,13],fog_near:-2,fog_far:4}}
    #[test]
    fn squared_distance_boundary_and_first_tie() {
        let a=point();let mut b=a;b.position=[40.,0.,0.];
        assert_eq!(nearest(&[a,b],[20.,0.,0.]),Some(0));
        assert_eq!(nearest(&[a],[20.25,0.,0.]),None);
        assert_eq!(nearest(&[],[0.;3]),None);
        assert_eq!(nearest(&[b,a],[20.,0.,0.]),Some(0));
    }
    #[test]
    fn effects_preserve_order_conditions_and_signed_values() {
        let mut a=point();let e=a.effects();
        assert_eq!(e[0],Effect::Ambient([6,7,8,0]));assert_eq!(e[1],Effect::Light(0x12345678));
        assert_eq!(e[2],Effect::Reverb {depth:-12,kind:3,delay:9,feedback:10});
        assert_eq!(e[3],Effect::Music {track:-1,mode:1,volume:1024});
        assert_eq!(e[4],Effect::Fog {color:[11,12,13],near:-2048.,far:4096.,source_0e:4.,source_0f:5.});
        let mut m=Moby::init_instance(0,1,None);a.apply_moby(&mut m);
        assert_eq!(m.ambient,[6,7,8,0]);assert_eq!(m.light,0x12345678);
        a.reverb_enable=0;a.fog_near=a.fog_far;assert_eq!(a.effects().len(),3);
    }
    #[test]
    fn packed_reader_checks_bounds_and_signed_fields() {
        let mut g=vec![0;0xd0];g[0x8c..0x90].copy_from_slice(&0xa0i32.to_le_bytes());
        g[0xa0..0xa4].copy_from_slice(&1i32.to_le_bytes());
        g[0xb4..0xb6].copy_from_slice(&1001u16.to_le_bytes());
        g[0xba..0xbc].copy_from_slice(&(-17i16).to_le_bytes());
        let p=read(&g).unwrap();assert_eq!(p.len(),1);assert_eq!(p[0].position,[250.25,0.,0.]);assert_eq!(p[0].reverb_depth,-17);
        assert!(read(&g[..0xcf]).is_err());
    }
}
