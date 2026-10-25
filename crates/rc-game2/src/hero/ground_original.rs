//! Resumable original GC2b4a08 ground service. Reuses the existing Hero,
//! collision query contract, RAC1 pure vector kernels and SideProbes storage.
//! Water, hit-moby services and alternate gravity remain explicit blockers.
//! Euler/trigonometric angles retain GC's existing native-f32 policy; original
//! VU microcode/runtime-table bit parity has not been validated by this port.
use super::{Hero,probe};
use rc_game::{collision_query::CollOutput,hero::{physics as shared,pose::SideProbes},ps2v::Pf};
use rc_formats::buf::Result;

/// Only previously absent source storage; existing Hero owns all ground XYZ,
/// counters, slope, semantic surface id and last-ground XYZ.
#[derive(Clone,Debug,Default)]
pub struct GroundStorage {
    pub normal_w:u32,
    /// SQ from VF25 copies an inherited W lane; native XYZ query does not
    /// establish its value. None retains that uncertainty without inventing0.
    pub point_w:Option<u32>,
    pub last_point_w:Option<u32>,
    pub half_034a:i16,
    pub source_0330:f32,
    pub source_0334:f32,
    pub source_0338:i32,
    pub ground_moby:Option<usize>,
    pub ground_owner_c10:Option<usize>,
    pub source_18c0:i32,
    pub sound_149d:u8,
    pub gravity_mode_22c3:u8,
    pub gravity_direction:[f32;4],
    pub pitch_0324:f32,
    pub roll_0328:f32,
    pub slope_yaw_032c:f32,
    pub side:SideProbes,
    pub byte_049c:u8,
}

#[derive(Clone,Copy,Debug)]
pub struct Inputs {
    pub frame:probe::Frame,
    pub source_1a79f8:i32,
    pub source_gp_minus_6000:i32,
    /// Actual current hero moby Euler fields, not an assumed Hero.rot alias.
    pub moby_euler:[f32;4],
}

pub trait GroundServices {
    /// Existing native World collision, ignoring the actual hero owner.
    fn line(&mut self,start:[f32;3],end:[f32;3],flags:u32)->Result<Option<CollOutput>>;
    /// The current query's original+2c lane, absent from native CollOutput.
    /// None leaves raw lane parity unresolved; all supported arithmetic below
    /// is XYZ-only, so that does not fabricate a coordinate or skip a service.
    fn point_w(&mut self)->Option<u32>;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Missing {Water,MobyServices,AlternateGravity}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GroundResult {Complete,Pending {source:u32,missing:Missing}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Step {Init,Query,Headers,Surface,Owner,Point,Height,Normal,Slope,Classify,Counters,Gravity,Tilt,Side0,Side1,SideApply0,SideApply1,Done}

#[derive(Clone,Debug)]
pub struct GroundProgress {next:Step,segment:([f32;4],[f32;4]),hit:Option<CollOutput>}
impl Default for GroundProgress {
    fn default()->Self {Self {next:Step::Init,segment:([0.;4],[0.;4]),hit:None}}
}

fn p(v:[f32;3])->shared::V4 {shared::from_f32x3(v)}
fn pending(source:u32,missing:Missing)->GroundResult {GroundResult::Pending {source,missing}}

/// GC2b8f50: transpose the current hero-moby Euler rows, transform raw normal,
/// atan2(local.y,sqrt(local.x²+local.z²)), -atan2(local.x,local.z).
/// Same architecture as RAC1 frame_tilt; no second hero is constructed.
fn tilt(n:[f32;3],euler:[f32;4])->(f32,f32) {
    let m=super::physics::euler_rows(euler[..3].try_into().unwrap());
    let n:[f32;3]=std::array::from_fn(|k|(m[k][0]*n[0]+m[k][1]*n[1])+m[k][2]*n[2]);
    let h=(n[0]*n[0]+n[2]*n[2]).sqrt();
    (n[1].atan2(h),-n[0].atan2(n[2]))
}

fn local_side(hero:&Hero,angle:f32,z:f32)->[f32;3] {
    let xy=[angle.cos()*0.2,angle.sin()*0.2,z];
    std::array::from_fn(|k|((hero.rows[0][k]*xy[0]+hero.rows[1][k]*xy[1])+hero.rows[2][k]*xy[2])+hero.pos[k])
}

impl GroundProgress {
    /// Complete only for supported source branches. Each mandatory query is
    /// issued once; unsupported services retain the current continuation.
    /// Complete means the supported native branch ran; point_w=None still
    /// explicitly withholds full original memory/VU-lane parity validation.
    pub fn resume(&mut self,hero:&mut Hero,s:&mut GroundStorage,input:Inputs,
        services:&mut dyn GroundServices)->Result<GroundResult> {
        loop {
            let following=match self.next {
                Step::Init=>{
                    let air=hero.air_ticks.wrapping_add(1);let f16c=hero.f16c.wrapping_add(1);
                    hero.ground_normal[2]=1.;hero.air_ticks=air;hero.f16c=f16c;
                    hero.ground_normal[0]=0.;hero.ground_normal[1]=0.; //W preserved.
                    hero.slope=0.;hero.ground_z=0.;hero.ground_point=[0.;3];s.point_w=Some(0);
                    s.half_034a=0;hero.height=32.;s.ground_moby=None;hero.on_moby=false;
                    if s.source_0338!=0 {s.source_0338=s.source_0338.wrapping_add(1);}
                    let mut up=hero.cap_radius*if hero.group==2 {f32::from_bits(0x4039_999a)}else{f32::from_bits(0x400c_cccd)};
                    if hero.query_2adff8() {up=0.;}
                    let position=[hero.pos[0],hero.pos[1],hero.pos[2],f32::from_bits(hero.teleport.position_w)];
                    self.segment=probe::segment(position,up,-48.,true,input.frame);Step::Query
                },
                Step::Query=>{
                    self.hit=services.line(self.segment.0[..3].try_into().unwrap(),self.segment.1[..3].try_into().unwrap(),2)?;
                    if self.hit.is_some() {Step::Headers}else{Step::Counters}
                },
                Step::Headers=>{
                    let hit=self.hit.as_ref().unwrap();s.sound_149d=probe::sound_class(hit.kind) as u8;
                    if hit.moby.is_some() {s.sound_149d=0;if input.source_1a79f8==7 {s.sound_149d=1;}}
                    hero.surface=hit.surface_id() as i16 as i32;Step::Surface
                },
                Step::Surface=>{
                    let surface=hero.surface;
                    if matches!(surface,0|0xd|3|0xb) || surface==1 && matches!(hero.state,0x79|0x7a) {
                        return Ok(pending(if surface==0 {0x31d7a0}else{0x2b4c04},Missing::Water));
                    }
                    Step::Owner
                },
                Step::Owner=>{
                    let hit=self.hit.as_ref().unwrap();
                    if hit.moby.is_some() {return Ok(pending(0x3204e8,Missing::MobyServices));}
                    if hit.kind>0 {Step::Point}else{Step::Counters}
                },
                Step::Point=>{
                    hero.ground_point=self.hit.as_ref().unwrap().point;s.point_w=services.point_w();
                    hero.ground_z=hero.ground_point[2];Step::Height
                },
                Step::Height=>{
                    if input.frame.source_1b8ff0!=0 {return Ok(pending(0x326820,Missing::AlternateGravity));}
                    hero.height=shared::dist3(p(hero.pos),p(hero.ground_point)).to_f32();
                    s.ground_moby=self.hit.as_ref().unwrap().moby;hero.on_moby=s.ground_moby.is_some();Step::Normal
                },
                Step::Normal=>{
                    let raw=self.hit.as_ref().unwrap().normal;
                    //RC2boot1290e0 byte-identical to RAC1boot125180 (60bytes).
                    let normal=shared::unit(p(raw));hero.ground_normal=shared::to_f32x3(normal);s.normal_w=normal[3].0;
                    Step::Slope
                },
                Step::Slope=>{
                    if s.gravity_mode_22c3!=0 {return Ok(pending(0x2d4248,Missing::AlternateGravity));}
                    let raw=self.hit.as_ref().unwrap().normal;
                    hero.slope=super::physics::slope_of(raw);Step::Classify
                },
                Step::Classify=>{
                    //Source2ef510 takesf12=raw.x,f13=raw.y.
                    //Slope yaw is additional source032c, stored below.
                    let raw=self.hit.as_ref().unwrap().normal;
                    s.slope_yaw_032c=raw[1].atan2(raw[0]);
                    if shared::dot3(shared::vsub(p(hero.ground_point),p(hero.pos)),p(hero.ground_normal))>Pf::ZERO {hero.height=-hero.height;}
                    if hero.height<f32::from_bits(0x3ca3_d70a) {
                        hero.f16c=0;
                        if hero.slope<=f32::from_bits(0x3f5f_66f4) || s.gravity_mode_22c3!=0 {
                            hero.air_ticks=0;s.ground_owner_c10=self.hit.as_ref().unwrap().moby;
                        }
                    }
                    Step::Counters
                },
                Step::Counters=>{
                    if s.source_0330<hero.pos[2] {s.source_0338=0;}
                    if hero.air_ticks!=0 {hero.grounded_ticks=0;}else{
                        hero.grounded_ticks=hero.grounded_ticks.wrapping_add(1);
                        hero.last_ground_point=hero.ground_point;s.last_point_w=s.point_w;
                        if hero.state!=0x38 {s.source_18c0=0;}
                    }
                    s.gravity_direction=[0.;4];Step::Gravity
                },
                Step::Gravity=>{
                    if s.gravity_mode_22c3!=0 {return Ok(pending(0x2b4f44,Missing::AlternateGravity));}
                    s.gravity_direction[2]=-1.;
                    s.pitch_0324=0.;s.roll_0328=0.;s.side=SideProbes::default();Step::Tilt
                },
                Step::Tilt=>{
                    let eligible=matches!(hero.group,0|0xc|9|7|1) || hero.group==4 && s.byte_049c!=0;
                    if !(hero.height<0.25) || !eligible {Step::Done}else{
                        let (roll,pitch)=tilt(hero.ground_normal,input.moby_euler);
                        let q=f32::from_bits(0x3f49_0fdc);
                        if -q<pitch && pitch<q {s.pitch_0324=pitch;}
                        if -q<roll && roll<q {s.roll_0328=roll;}
                        Step::Side0
                    }
                },
                Step::Side0|Step::Side1=>{
                    let k=if self.next==Step::Side0 {0}else{1};
                    let angle=if k==0 {f32::from_bits(0x3fc9_0fdc)}else{-f32::from_bits(0x3fc9_0fdc)};
                    let a=local_side(hero,angle,f32::from_bits(0x3ee6_6666));
                    let b=local_side(hero,angle,f32::from_bits(0xbee6_6666));
                    self.hit=services.line(a,b,0x22)?;
                    if k==0 {Step::SideApply0}else{Step::SideApply1}
                },
                Step::SideApply0|Step::SideApply1=>{
                    let k=if self.next==Step::SideApply0 {0}else{1};
                    match self.hit.as_ref() {
                        None=>{s.side.dz[k]=0.;s.side.pitch[k]=s.pitch_0324;s.side.roll[k]=s.roll_0328;},
                        Some(hit)=>{
                            if input.source_gp_minus_6000!=0 {return Ok(pending(0x326820,Missing::AlternateGravity));}
                            let dz=hit.point[2]-hero.ground_z;s.side.dz[k]=if 0.4<dz.abs() {0.}else{dz};
                            let (roll,pitch)=tilt(hit.normal,input.moby_euler);let q=f32::from_bits(0x3f49_0fdc);
                            if -q<pitch && pitch<q {s.side.pitch[k]=pitch;}
                            if -q<roll && roll<q {s.side.roll[k]=roll;}
                        }
                    }
                    if k==0 {Step::Side1}else{Step::Done}
                },
                Step::Done=>return Ok(GroundResult::Complete),
            };
            self.next=following;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Queries {answers:std::collections::VecDeque<Option<CollOutput>>,calls:Vec<u32>,w:Option<u32>}
    impl GroundServices for Queries {
        fn line(&mut self,_a:[f32;3],_b:[f32;3],flags:u32)->Result<Option<CollOutput>> {
            self.calls.push(flags);Ok(self.answers.pop_front().expect("unexpected query replay"))
        }
        fn point_w(&mut self)->Option<u32> {self.w}
    }
    fn hit()->CollOutput {CollOutput {moby:None,primitive:None,kind:0x101f,point:[10.,10.,0.],pushed_centre:None,
        normal:[0.,0.,16.],tri:[[0.;3];3],best:0.}}
    fn input()->Inputs {Inputs {frame:probe::Frame {source_1b8ff0:0,source_18a0e0:[0.,0.,-1.,0.],moby_up:[0.,0.,1.,0.]},
        source_1a79f8:0,source_gp_minus_6000:0,moby_euler:[0.;4]}}
    #[test]
    fn unknown_original_point_w_stays_metadata_and_completed_probe_does_not_replay() {
        for w in [None,Some(0x7fc0_1234)] {
            let mut h=Hero::spawn([10.,10.,0.005],0.);let mut s=GroundStorage::default();s.normal_w=42;
            let mut q=Queries {answers:[Some(hit()),None,None].into(),calls:vec![],w};
            let mut progress=GroundProgress::default();
            assert_eq!(progress.resume(&mut h,&mut s,input(),&mut q).unwrap(),GroundResult::Complete);
            assert_eq!((s.point_w,s.last_point_w,s.normal_w),(w,w,0));
            assert_eq!((h.air_ticks,h.f16c,h.grounded_ticks),(0,0,1));
            assert_eq!(q.calls,vec![2,0x22,0x22]);
            assert_eq!(s.gravity_direction,[0.,0.,-1.,0.]);
            assert_eq!(h.surface,-1);assert_eq!(s.sound_149d,0);
            assert_eq!(progress.resume(&mut h,&mut s,input(),&mut q).unwrap(),GroundResult::Complete);
            assert_eq!((q.calls.len(),h.grounded_ticks),(3,1));
        }
    }
    #[test]
    fn unsupported_water_owner_and_gravity_stop_at_source_bound_services() {
        for (surface,moby,source,missing) in [(0,None,0x31d7a0,Missing::Water),(0x1f,Some(7),0x3204e8,Missing::MobyServices)] {
            let mut h=Hero::spawn([10.,10.,1.],0.);let mut s=GroundStorage::default();
            let mut output=hit();output.kind=0x1000|surface;output.moby=moby;
            let mut q=Queries {answers:[Some(output)].into(),calls:vec![],w:Some(0)};
            let mut progress=GroundProgress::default();
            assert_eq!(progress.resume(&mut h,&mut s,input(),&mut q).unwrap(),pending(source,missing));
            assert_eq!(h.ground_point,[0.;3]);assert_eq!(q.calls,vec![2]);
        }
        let mut h=Hero::spawn([10.,10.,1.],0.);let mut s=GroundStorage::default();
        let mut q=Queries {answers:[Some(hit())].into(),calls:vec![],w:Some(17)};
        let mut progress=GroundProgress::default();let mut i=input();i.frame.source_1b8ff0=1;
        assert_eq!(progress.resume(&mut h,&mut s,i,&mut q).unwrap(),pending(0x326820,Missing::AlternateGravity));
        assert_eq!(h.ground_point,[10.,10.,0.]);assert_eq!(s.point_w,Some(17));
        assert_eq!(progress.resume(&mut h,&mut s,i,&mut q).unwrap(),pending(0x326820,Missing::AlternateGravity));
        assert_eq!(q.calls,vec![2]);
    }
    #[test]
    fn gc_walk_group_does_not_inherit_rac1_side_probe_filter() {
        let mut h=Hero::spawn([10.,10.,0.005],0.);h.group=2;
        let mut s=GroundStorage::default();let mut q=Queries {answers:[Some(hit())].into(),calls:vec![],w:Some(0)};
        assert_eq!(GroundProgress::default().resume(&mut h,&mut s,input(),&mut q).unwrap(),GroundResult::Complete);
        assert_eq!(q.calls,vec![2]);
    }
}
