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

/// Actual native host boundary; returning None means no transport call ran.
/// Returned handles are fed through the original32fcc0 callback, including
/// source-supported failures. This adapter never fabricates a successful voice.
pub trait EnvironmentHost {
    fn start_music(&mut self,request:&crate::level_music::StartRequest)->Result<Option<i32>>;
    /// Store original fog globals, THEN execute2e4af0. Each step may suspend.
    fn write_fog(&mut self,effect:Effect)->Result<bool>;
    fn update_fog_2e4af0(&mut self)->Result<bool>;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ApplyResult {Complete,Pending(u32)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum ApplyStep {Ambient,Light,Reverb,Music,MusicTransport,Fog,FogUpdate,Done}

/// Continuation for313008 after its original nearest-point selection. Borrows
/// the existing Moby, reverb request and GC music owner; no second audio engine,
/// voice manager, RNG or invented global defaults. Store this with the original
/// caller until Complete, including when a real native host is unavailable.
pub struct Apply {
    point:Option<SamplePoint>,
    owner:(u32,u32),
    next:ApplyStep,
    pending_music:Option<crate::level_music::StartRequest>,
}
impl Apply {
    pub fn begin(points:&[SamplePoint],moby:&Moby)->Self {
        Self {point:nearest(points,moby.position[..3].try_into().unwrap()).map(|i|points[i]),
            owner:(moby.index,moby.uid_hi),next:ApplyStep::Ambient,pending_music:None}
    }
    #[allow(clippy::too_many_arguments)]
    pub fn resume(&mut self,moby:&mut Moby,
        mut reverb:Option<&mut rc_game::audio::reverb::ReverbRequest>,
        mut music:Option<&mut crate::level_music::MusicPlayer>,
        wad:&crate::level_music::MusicWad<'_>,disc_sector:u32,
        host:&mut dyn EnvironmentHost)->Result<ApplyResult> {
        if self.owner!=(moby.index,moby.uid_hi) {return invalid("GC environment continuation owner changed");}
        let Some(point)=self.point else {return Ok(ApplyResult::Complete);};
        loop {
            self.next=match self.next {
                ApplyStep::Ambient=>{moby.light=0;moby.ambient=[point.ambient[0],point.ambient[1],point.ambient[2],0];ApplyStep::Light},
                ApplyStep::Light=>{moby.light=point.light;ApplyStep::Reverb},
                ApplyStep::Reverb=>{
                    if point.reverb_enable!=0 {
                        let Some(request)=reverb.as_deref_mut() else {return Ok(ApplyResult::Pending(0x313184));};
                        // GC directly overwrites1986c4..ca, dirty1986cb|=7.
                        // R1set() has changed-value dirty semantics; reuse only
                        // the existing request OWNER/fields, not that setter.
                        request.depth=point.reverb_depth as i32;request.kind=point.reverb_type;
                        request.delay=point.reverb_delay;request.feedback=point.reverb_feedback;request.dirty|=7;
                    }
                    ApplyStep::Music
                },
                ApplyStep::Music=>{
                    if let Some(player)=music.as_deref_mut() {
                        self.pending_music=player.request_start(wad,disc_sector,point.music_track as i32,1,0x400)?;
                    }else{
                        //32eb88 has no writes before its track/handle/sector
                        // guards. A negative track or a valid absent entry has
                        // no effects for ANY handle, so no invented player is
                        // needed. Unknown/out-of-range/nonzero entries still
                        // require the real player, preserving the early handle
                        // guard rather than speculatively reading channel data.
                        let track=point.music_track as i32;
                        let no_effect=track<0 || wad.entry(track as usize).is_ok_and(|e|e.sector==0);
                        if !no_effect {return Ok(ApplyResult::Pending(0x32eb88));}
                        self.pending_music=None;
                    }
                    if self.pending_music.is_some() {ApplyStep::MusicTransport}else{ApplyStep::Fog}
                },
                ApplyStep::MusicTransport=>{
                    let Some(player)=music.as_deref_mut() else {return Ok(ApplyResult::Pending(0x32fcc0));};
                    let Some(handle)=host.start_music(self.pending_music.as_ref().unwrap())? else {
                        return Ok(ApplyResult::Pending(0x133350));
                    };
                    player.complete_start(handle);self.pending_music=None;ApplyStep::Fog
                },
                ApplyStep::Fog=>{
                    if let Some(effect)=point.effects().into_iter().find(|e|matches!(e,Effect::Fog {..})) {
                        if !host.write_fog(effect)? {return Ok(ApplyResult::Pending(0x313200));}
                        ApplyStep::FogUpdate
                    }else{ApplyStep::Done}
                },
                ApplyStep::FogUpdate=>{
                    if !host.update_fog_2e4af0()? {return Ok(ApplyResult::Pending(0x2e4af0));}
                    ApplyStep::Done
                },
                ApplyStep::Done=>return Ok(ApplyResult::Complete),
            };
        }
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
    struct Host {events:Vec<u32>,allow_fog:bool}
    impl EnvironmentHost for Host {
        fn start_music(&mut self,_:&crate::level_music::StartRequest)->Result<Option<i32>> {panic!("missing original track cannot start transport")}
        fn write_fog(&mut self,_:Effect)->Result<bool> {self.events.push(0x313200);Ok(true)}
        fn update_fog_2e4af0(&mut self)->Result<bool> {self.events.push(0x2e4af0);Ok(self.allow_fog)}
    }
    fn empty_music_wad()->Vec<u8> {let mut b=vec![0;0x1018];b[..4].copy_from_slice(&0x1018u32.to_le_bytes());b}
    #[test]
    fn owned_environment_chain_preserves_pending_writes_and_never_skips_fog() {
        use rc_game::audio::{music::Player,reverb::ReverbRequest};
        let mut m=Moby::init_instance(7,1,None);let mut q=point();q.music_track=4;
        let bytes=empty_music_wad();let wad=crate::level_music::MusicWad::parse(&bytes).unwrap();
        let mut host=Host {events:vec![],allow_fog:false};let mut apply=Apply::begin(&[q],&m);
        assert_eq!(apply.resume(&mut m,None,None,&wad,0,&mut host).unwrap(),ApplyResult::Pending(0x313184));
        assert_eq!((m.light,m.ambient),(q.light,[6,7,8,0]));
        let mut reverb=ReverbRequest {dirty:0x80,..Default::default()};
        assert_eq!(apply.resume(&mut m,Some(&mut reverb),None,&wad,0,&mut host).unwrap(),ApplyResult::Pending(0x2e4af0));
        assert_eq!((reverb.depth,reverb.kind,reverb.delay,reverb.feedback,reverb.dirty),(-12,3,9,10,0x87));
        m.light=99;reverb.depth=42; // Completed writes MUST NOT replay on retry.
        let mut music=crate::level_music::MusicPlayer {player:Player::default(),source_14:27};let before=music;
        assert_eq!(apply.resume(&mut m,Some(&mut reverb),Some(&mut music),&wad,0,&mut host).unwrap(),ApplyResult::Pending(0x2e4af0));
        assert_eq!(music,before);assert_eq!((m.light,reverb.depth),(99,42));
        assert_eq!(host.events,vec![0x313200,0x2e4af0,0x2e4af0]);host.allow_fog=true;
        assert_eq!(apply.resume(&mut m,Some(&mut reverb),Some(&mut music),&wad,0,&mut host).unwrap(),ApplyResult::Complete);
        assert_eq!(host.events,vec![0x313200,0x2e4af0,0x2e4af0,0x2e4af0]);
        assert_eq!(apply.resume(&mut m,None,None,&wad,0,&mut host).unwrap(),ApplyResult::Complete);
        assert_eq!(host.events.len(),4);
    }
    #[test]
    fn absent_music_guard_needs_no_guessed_player_but_present_track_still_blocks() {
        let mut point=point();point.music_track=4;point.reverb_enable=0;point.fog_far=point.fog_near;
        let mut m=Moby::init_instance(7,1,None);let mut host=Host {events:vec![],allow_fog:false};
        let mut bytes=empty_music_wad();let wad=crate::level_music::MusicWad::parse(&bytes).unwrap();
        let mut apply=Apply::begin(&[point],&m);
        assert_eq!(apply.resume(&mut m,None,None,&wad,0,&mut host).unwrap(),ApplyResult::Complete);
        bytes[8+4*8..12+4*8].copy_from_slice(&1u32.to_le_bytes());
        let wad=crate::level_music::MusicWad::parse(&bytes).unwrap();let mut apply=Apply::begin(&[point],&m);
        assert_eq!(apply.resume(&mut m,None,None,&wad,0,&mut host).unwrap(),ApplyResult::Pending(0x32eb88));
        assert!(host.events.is_empty());
    }
    #[test]
    fn environment_continuation_cannot_rebind_to_another_owner() {
        let mut m=Moby::init_instance(7,1,None);let mut apply=Apply::begin(&[point()],&m);
        let b=empty_music_wad();let wad=crate::level_music::MusicWad::parse(&b).unwrap();
        let mut host=Host {events:vec![],allow_fog:false};m.index=8;
        assert!(apply.resume(&mut m,None,None,&wad,0,&mut host).is_err());assert!(host.events.is_empty());
    }

}
