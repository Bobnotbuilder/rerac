//! GC39c870 state-zero adapter over the existing native Objects and pvars.
//! RAC1 crate_/stack_init is the reuse map, not a compatible title callback:
//! GC changes the hit mask, velocity offset, flag order and icon parameters.
//! Required39dfd8/39d0c0/39ff50 services must execute before success; missing
//! services retain the exact continuation, including already-written state1.
use crate::objects::Objects;
use rc_game::moby_runtime::MobyId;
use rc_formats::{buf::{Buf,invalid,Result}};

pub const UPDATE:u32=0x39c870;
pub const HIT_MASK:u32=0x0583_0001;

///3204e8's class test after the caller resolved the original pointer into
/// this actual object bank. Deleted state is deliberately NOT filtered: the
/// original only tests nonnull/bank bounds and unsigned class500..540.
pub fn source_crate_pointer(objects:&Objects,id:Option<MobyId>)->bool {
    id.and_then(|i|objects.get(i)).is_some_and(|m|(m.o_class as u16 as u32).wrapping_sub(0x1f4)<0x29)
}

pub trait CrateServices {
    fn objects(&self)->&Objects;
    fn objects_mut(&mut self)->&mut Objects;
    fn counter(&self)->u32;
    /// False means the actual required service is unavailable, with no effects.
    fn physics_39dfd8(&mut self,owner:MobyId)->Result<bool>;
    /// Must use the existing GC hit log/actual target slot. State0 ignores the
    /// record, but original rejected-mask clearing still must occur.
    fn hit_31dce8(&mut self,owner:MobyId,mask:u32,keep_rejected:bool)->Result<()>;
    /// Resolve this runtime pointer from its actual ownership binding. None
    /// means unresolved; false is the ORIGINAL range/class rejection, not a
    /// generic missing-resource fallback. Zero raw pointers never call this.
    fn crate_link(&mut self,owner:MobyId,field:usize,raw:u32)->Result<Option<bool>>;
    fn stack_39d0c0(&mut self,owner:MobyId)->Result<bool>;
    /// None means unsupported; Some(0) is a source-supported allocation
    /// failure, and Some(nonzero) must name an actual newly allocated actor
    /// in the caller's native pointer binding. No fake actor is synthesized.
    fn icon_39ff50(&mut self,owner:MobyId,mode:i32)->Result<Option<u32>>;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum InitResult {Complete,Pending(u32)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Step {Validate,Physics,Support,Hit,Above,Below,Clear,Stack,Zero,Class,Done}

#[derive(Clone,Debug)]
pub struct CrateInitProgress {next:Step,owner:Option<MobyId>}
impl Default for CrateInitProgress {
    fn default()->Self {Self {next:Step::Validate,owner:None}}
}

fn word(services:&dyn CrateServices,id:MobyId,offset:usize)->Result<u32> {
    let Some(m)=services.objects().get(id) else {return invalid("GC crate owner absent");};
    Buf(&m.pvars).u32(offset)
}
fn set_word(services:&mut dyn CrateServices,id:MobyId,offset:usize,value:u32) {
    services.objects_mut().get_mut(id).unwrap().pvars[offset..offset+4].copy_from_slice(&value.to_le_bytes());
}

impl CrateInitProgress {
    pub fn resume(&mut self,id:MobyId,services:&mut dyn CrateServices)->Result<InitResult> {
        if self.owner.is_some_and(|owner|owner!=id) {return invalid("GC crate continuation owner changed");}
        loop {
            let following=match self.next {
                Step::Validate=>{
                    let Some(m)=services.objects().get(id) else {return invalid("GC crate owner absent");};
                    if m.update_fn!=Some(UPDATE) || m.state!=0 || m.pvars.len()<0x100 {
                        return invalid("GC39c870 state0 requires its actual callback/owner/pvar block");
                    }
                    self.owner=Some(id);Step::Physics
                },
                Step::Physics=>{
                    if word(services,id,0xac)?&0xc!=0 && word(services,id,0xb4)?!=services.counter()
                        && services.objects().get(id).unwrap().state!=0
                        && !services.physics_39dfd8(id)? {return Ok(InitResult::Pending(0x39dfd8));}
                    //39dfd8 itself returns at39e014 when state0 (no effects
                    //or RNG). This is the original guard, not a missing-call
                    //fallback. Later nonzero states still need fullphysics.
                    Step::Support
                },
                Step::Support=>{
                    if word(services,id,0xa0)?==0 {
                        let m=services.objects_mut().get_mut(id).unwrap();
                        let flags=u16::from_le_bytes(m.pvars[0x3e..0x40].try_into().unwrap())|1;
                        m.pvars[0x3e..0x40].copy_from_slice(&flags.to_le_bytes());
                    }
                    // Unlike RAC1, a nonzero above pointer does NOT clearbit0.
                    Step::Hit
                },
                Step::Hit=>{services.hit_31dce8(id,HIT_MASK,false)?;Step::Above},
                Step::Above|Step::Below=>{
                    let field=if self.next==Step::Above {0xa0}else{0xa4};let raw=word(services,id,field)?;
                    if raw!=0 {
                        match services.crate_link(id,field,raw)? {
                            None=>return Ok(InitResult::Pending(0x3204e8)),
                            Some(false)=>set_word(services,id,field,0),Some(true)=>{}
                        }
                    }
                    if field==0xa0 {Step::Below}else{Step::Clear}
                },
                Step::Clear=>{
                    services.objects_mut().get_mut(id).unwrap().pvars[0x90..0xa0].fill(0);
                    services.objects_mut().get_mut(id).unwrap().state=1;
                    set_word(services,id,0xa8,0);
                    if services.objects().get(id).unwrap().o_class==0x1ff {
                        let next=word(services,id,0xb4)?.wrapping_add(1);set_word(services,id,0xb4,next);
                    }
                    if word(services,id,0xcc)?==0 {
                        let flags=word(services,id,0xac)?|0x14;set_word(services,id,0xac,flags);
                        if flags&1==0 {Step::Stack}else{Step::Zero}
                    }else{Step::Zero}
                },
                Step::Stack=>{
                    if !services.stack_39d0c0(id)? {return Ok(InitResult::Pending(0x39d0c0));}
                    Step::Zero
                },
                Step::Zero=>{set_word(services,id,0xfc,0);Step::Class},
                Step::Class=>{
                    let class=services.objects().get(id).unwrap().o_class;
                    if class==0x1ff {
                        services.objects_mut().get_mut(id).unwrap().pvars[0xc6..0xc8].copy_from_slice(&100i16.to_le_bytes());
                    }else if matches!(class,0x1f5|0x200) {
                        let Some(actor)=services.icon_39ff50(id,if class==0x200 {1}else{0})? else {
                            return Ok(InitResult::Pending(0x39ff50));
                        };
                        set_word(services,id,0xfc,actor);
                    }
                    // State0 jumps to39d098, bypassing hit-slot clear39d094.
                    Step::Done
                },
                Step::Done=>return Ok(InitResult::Complete),
            };
            self.next=following;
        }
    }
}

/// Original39d0c0 services for its ungrouped (groupff) branch. The grouped
/// branch needs the original31c9d8/31cac0 iteration order and shared path
/// state; it is not replaced with an independent per-crate floor test.
pub trait StackServices:CrateServices {
    fn rng(&mut self)->&mut rc_game::rng::Rng;
    fn line(&mut self,start:[f32;3],end:[f32;3],flags:u32,ignore:Option<MobyId>)
        ->Result<Option<rc_game::collision_query::CollOutput>>;
    /// Original1a73e0[mission] value, not an assumed unlocked/save value.
    fn mission_word(&mut self,mission:u8)->Result<Option<i32>>;
    fn delete_311688(&mut self,owner:MobyId)->Result<()>;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum StackStep {Begin,Group,Rotate,Mission,Fields,Probe,Resolve,Carrier,Done}

#[derive(Clone,Debug)]
pub struct StackInitProgress {next:StackStep,owner:Option<MobyId>,hit:Option<rc_game::collision_query::CollOutput>}
impl Default for StackInitProgress {
    fn default()->Self {Self {next:StackStep::Begin,owner:None,hit:None}}
}

impl StackInitProgress {
    /// Complete source ungrouped/world-floor branches using actual owner,
    /// existing one RNG and native collision. Group iteration, saved mission
    /// inputs and carrier attachment remain precise mandatory dependencies.
    pub fn resume(&mut self,id:MobyId,s:&mut dyn StackServices)->Result<InitResult> {
        if self.owner.is_some_and(|owner|owner!=id) {return invalid("GC stack continuation owner changed");}
        loop {
            let following=match self.next {
                StackStep::Begin=>{
                    let Some(m)=s.objects().get(id) else {return invalid("GC stack owner absent");};
                    if m.pvars.len()<0x100 {return invalid("GC stack owned pvar block too short");}
                    self.owner=Some(id);
                    let flags=Buf(&m.pvars).u32(0xac)?|4;
                    s.objects_mut().get_mut(id).unwrap().pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());
                    StackStep::Group
                },
                StackStep::Group=>{
                    if s.objects().get(id).unwrap().group as u8!=0xff {return Ok(InitResult::Pending(0x31c9d8));}
                    StackStep::Rotate
                },
                StackStep::Rotate=>{
                    if s.objects().get(id).unwrap().o_class==0x1f4 {
                        // The draw occurs BEFORE testing the two tilt angles,
                        // even on a tilted crate. Preserve the single stream.
                        let turn=s.rng().randi(2)!=0;
                        let m=s.objects_mut().get_mut(id).unwrap();
                        if turn && m.rotation[0]==0. && m.rotation[1]==0. {
                            m.rotation[2]=crate::hero::physics::add_rot(m.rotation[2],f32::from_bits(0x3fc9_0fdc));
                        }
                    }
                    StackStep::Mission
                },
                StackStep::Mission=>{
                    let m=s.objects().get(id).unwrap();let (class,mission)=(m.o_class,m.mission);
                    if matches!(class,0x1f5|0x200) && mission!=0xff {
                        let threshold=Buf(&m.pvars).i32(0xf8)?;
                        let Some(saved)=s.mission_word(mission)? else {return Ok(InitResult::Pending(0x39d1b4));};
                        if saved<threshold {s.delete_311688(id)?;self.next=StackStep::Done;return Ok(InitResult::Complete);}
                    }
                    StackStep::Fields
                },
                StackStep::Fields=>{
                    let m=s.objects_mut().get_mut(id).unwrap();let index=m.index as u16;
                    m.pvars[0xc4..0xc6].copy_from_slice(&index.to_le_bytes());
                    m.pvars[0xb0..0xb4].copy_from_slice(&1f32.to_le_bytes());
                    let flags=Buf(&m.pvars).u32(0xac)?|1;
                    m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());StackStep::Probe
                },
                StackStep::Probe=>{
                    let p=s.objects().get(id).unwrap().position;
                    self.hit=s.line([p[0],p[1],p[2]+0.5],[p[0],p[1],f32::from_bits(0x3dcc_cccd)],2,None)?;
                    StackStep::Resolve
                },
                StackStep::Resolve=>{
                    let Some(hit)=self.hit.as_ref() else {
                        s.objects_mut().get_mut(id).unwrap().pvars[0xa4..0xa8].fill(0);
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    };
                    if hit.moby.is_none() {
                        let m=s.objects_mut().get_mut(id).unwrap();m.position[2]=hit.point[2];
                        m.pvars[0xa4..0xa8].fill(0);
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    }
                    if source_crate_pointer(s.objects(),hit.moby) {
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    }
                    let m=s.objects_mut().get_mut(id).unwrap();m.pvars[0xa4..0xa8].fill(0);
                    let flags=Buf(&m.pvars).u32(0xac)?|8;m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());
                    StackStep::Carrier
                },
                StackStep::Carrier=>return Ok(InitResult::Pending(0x322028)),
                StackStep::Done=>return Ok(InitResult::Complete),
            };
            self.next=following;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::moby_runtime::Moby;
    struct Fixture {objects:Objects,counter:u32,events:Vec<u32>,rng:rc_game::rng::Rng,
        queries:std::collections::VecDeque<Option<rc_game::collision_query::CollOutput>>,mission:Option<i32>}
    impl CrateServices for Fixture {
        fn objects(&self)->&Objects {&self.objects}
        fn objects_mut(&mut self)->&mut Objects {&mut self.objects}
        fn counter(&self)->u32 {self.counter}
        fn physics_39dfd8(&mut self,_id:MobyId)->Result<bool> {self.events.push(0x39dfd8);Ok(false)}
        fn hit_31dce8(&mut self,_id:MobyId,mask:u32,keep:bool)->Result<()> {
            assert_eq!(mask,HIT_MASK);assert!(!keep);self.events.push(0x31dce8);Ok(())
        }
        fn crate_link(&mut self,_id:MobyId,_field:usize,_raw:u32)->Result<Option<bool>> {self.events.push(0x3204e8);Ok(None)}
        fn stack_39d0c0(&mut self,_id:MobyId)->Result<bool> {self.events.push(0x39d0c0);Ok(false)}
        fn icon_39ff50(&mut self,_id:MobyId,_mode:i32)->Result<Option<u32>> {self.events.push(0x39ff50);Ok(None)}
    }
    impl StackServices for Fixture {
        fn rng(&mut self)->&mut rc_game::rng::Rng {&mut self.rng}
        fn line(&mut self,a:[f32;3],b:[f32;3],flags:u32,ignore:Option<MobyId>)->Result<Option<rc_game::collision_query::CollOutput>> {
            assert_eq!(flags,2);assert_eq!(ignore,None);assert_eq!(b[2],f32::from_bits(0x3dcc_cccd));
            assert_eq!([a[0],a[1]],[b[0],b[1]]);self.events.push(0x2de878);
            Ok(self.queries.pop_front().expect("stack probe replay"))
        }
        fn mission_word(&mut self,_mission:u8)->Result<Option<i32>> {self.events.push(0x39d1b4);Ok(self.mission)}
        fn delete_311688(&mut self,id:MobyId)->Result<()> {self.events.push(0x311688);self.objects.delete(id,self.counter,|_|{})}
    }
    fn fixture(class:i16)->Fixture {
        let mut m=Moby::zeroed();m.o_class=class;m.state=0;m.update_fn=Some(UPDATE);m.hit_slot=9;
        m.pvars=vec![0;0x100];m.pvars[0x40..0x50].fill(0xa5);m.pvars[0x90..0xa0].fill(0xff);
        m.pvars[0xfc..0x100].copy_from_slice(&0xdead_beefu32.to_le_bytes());
        Fixture {objects:Objects::new(vec![m],0),counter:0,events:vec![],rng:rc_game::rng::Rng {state:1234},queries:Default::default(),mission:None}
    }
    #[test]
    fn required_stack_blocks_after_actual_owned_state_and_flags_without_replay() {
        let mut f=fixture(0x1ff);let mut progress=CrateInitProgress::default();
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        let m=f.objects.get(0).unwrap();assert_eq!(m.state,1);
        assert_eq!(&m.pvars[0x90..0xa0],&[0;16]);assert_eq!(&m.pvars[0x40..0x50],&[0xa5;16]);
        assert_eq!(word(&f,0,0xac).unwrap(),0x14);assert_eq!(word(&f,0,0xb4).unwrap(),1);
        assert_eq!(word(&f,0,0xfc).unwrap(),0xdead_beef);assert_eq!(m.hit_slot,9);
        assert_eq!(f.events,vec![0x31dce8,0x39d0c0]);
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        assert_eq!(word(&f,0,0xb4).unwrap(),1,"class1ff increment must not replay");
        assert_eq!(f.events,vec![0x31dce8,0x39d0c0,0x39d0c0]);
    }
    #[test]
    fn source_skip_stack_branch_finishes_bolt_crate_but_not_required_icon() {
        let mut f=fixture(0x1f4);set_word(&mut f,0,0xcc,1);let mut p=CrateInitProgress::default();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert_eq!(word(&f,0,0xac).unwrap(),0,"cc branch skips BOTH flags and stack");
        assert_eq!(word(&f,0,0xfc).unwrap(),0);assert_eq!(f.objects.get(0).unwrap().hit_slot,9);
        assert_eq!(f.events,vec![0x31dce8]);
        let mut f=fixture(0x1f5);set_word(&mut f,0,0xcc,1);
        assert_eq!(CrateInitProgress::default().resume(0,&mut f).unwrap(),InitResult::Pending(0x39ff50));
        assert_eq!(f.events,vec![0x31dce8,0x39ff50]);assert_eq!(word(&f,0,0xfc).unwrap(),0);
    }
    #[test]
    fn original_physics_state_zero_return_does_not_invent_a_physics_dependency() {
        let mut f=fixture(0x1f4);set_word(&mut f,0,0xac,4);f.counter=1;
        assert_eq!(CrateInitProgress::default().resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        assert_eq!(f.objects.get(0).unwrap().state,1);assert_eq!(word(&f,0,0x3c).unwrap(),0x10000);
        assert_eq!(f.events,vec![0x31dce8,0x39d0c0]);
    }
    #[test]
    fn original_crate_pointer_test_keeps_deleted_class_and_uses_unsigned_range() {
        let mut f=fixture(0x1f4);f.objects.get_mut(0).unwrap().state=0xff;
        assert!(source_crate_pointer(&f.objects,Some(0)));
        f.objects.get_mut(0).unwrap().o_class=540;assert!(source_crate_pointer(&f.objects,Some(0)));
        f.objects.get_mut(0).unwrap().o_class=541;assert!(!source_crate_pointer(&f.objects,Some(0)));
        f.objects.get_mut(0).unwrap().o_class=-1;assert!(!source_crate_pointer(&f.objects,Some(0)));
        assert!(!source_crate_pointer(&f.objects,None));assert!(!source_crate_pointer(&f.objects,Some(999)));
    }
    fn floor(moby:Option<usize>)->rc_game::collision_query::CollOutput {
        rc_game::collision_query::CollOutput {moby,primitive:None,kind:0x101f,point:[10.,20.,6.],pushed_centre:None,
            normal:[0.,0.,1.],tri:[[0.;3];3],best:0.}
    }
    #[test]
    fn ungrouped_floor_stack_uses_actual_owner_one_rng_and_preserves_other_lanes() {
        let mut f=fixture(0x1f4);let m=f.objects.get_mut(0).unwrap();m.group=-1;m.index=23;m.position=[10.,20.,7.,42.];
        m.rotation=[0.,0.,0.2,13.];let mut expected=f.rng;let turn=expected.randi(2)!=0;
        f.queries.push_back(Some(floor(None)));let mut progress=StackInitProgress::default();
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Complete);assert_eq!(f.rng,expected);
        let m=f.objects.get(0).unwrap();assert_eq!(m.position,[10.,20.,6.,42.]);assert_eq!(m.rotation[3],13.);
        assert_eq!(m.rotation[2],if turn {crate::hero::physics::add_rot(0.2,f32::from_bits(0x3fc90fdc))}else{0.2});
        assert_eq!(Buf(&m.pvars).u16(0xc4).unwrap(),23);assert_eq!(Buf(&m.pvars).f32(0xb0).unwrap(),1.);
        assert_eq!(word(&f,0,0xac).unwrap(),5);assert_eq!(f.events,vec![0x2de878]);
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Complete);assert_eq!(f.rng,expected);assert_eq!(f.events.len(),1);
    }
    #[test]
    fn tilted_bolt_crate_still_draws_and_grouped_crate_does_not_use_floor_shortcut() {
        let mut f=fixture(0x1f4);f.objects.get_mut(0).unwrap().group=-1;f.objects.get_mut(0).unwrap().rotation[0]=0.1;
        f.queries.push_back(None);let mut expected=f.rng;expected.randi(2);
        assert_eq!(StackInitProgress::default().resume(0,&mut f).unwrap(),InitResult::Complete);assert_eq!(f.rng,expected);
        assert_eq!(f.objects.get(0).unwrap().rotation[2],0.);
        let mut f=fixture(0x1f4);let expected=f.rng;let mut p=StackInitProgress::default();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x31c9d8));
        assert_eq!(word(&f,0,0xac).unwrap(),4);assert_eq!(f.rng,expected);assert!(f.events.is_empty());
    }
    #[test]
    fn mission_and_carrier_dependencies_retain_real_effects_without_query_replay() {
        let mut f=fixture(0x1f5);f.objects.get_mut(0).unwrap().group=-1;set_word(&mut f,0,0xf8,4);
        let mut p=StackInitProgress::default();assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d1b4));
        f.mission=Some(3);assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert!(f.objects.get(0).unwrap().state>=0x80);assert_eq!(f.events,vec![0x39d1b4,0x39d1b4,0x311688]);
        let mut f=fixture(0x1ff);f.objects.get_mut(0).unwrap().group=-1;
        let mut carrier=Moby::zeroed();carrier.o_class=100;f.objects=Objects::new(vec![f.objects.get(0).unwrap().clone(),carrier],0);
        f.queries.push_back(Some(floor(Some(1))));let mut p=StackInitProgress::default();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));assert_eq!(word(&f,0,0xac).unwrap(),13);
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));assert_eq!(f.events,vec![0x2de878]);
    }
}
