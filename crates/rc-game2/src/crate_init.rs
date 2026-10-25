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
    /// Outer None is unresolved provenance; inner None is a proven null.
    /// World overrides this for loader-fixed native ids: 0 can be owner0,
    /// with -1 null. Only unrelocated generic fixtures use raw0=null here.
    fn resolve_stack_link(&mut self,owner:MobyId,field:usize)->Result<Option<Option<MobyId>>> {
        let raw=word(self.objects(),owner,field)?;
        Ok(if raw==0 {Some(None)}else{None})
    }
    fn write_stack_link(&mut self,owner:MobyId,field:usize,target:Option<MobyId>)->Result<bool> {
        if target.is_some() {return Ok(false);}
        self.objects_mut().get_mut(owner).unwrap().pvars[field..field+4].fill(0);Ok(true)
    }
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

fn word(objects:&Objects,id:MobyId,offset:usize)->Result<u32> {
    let Some(m)=objects.get(id) else {return invalid("GC crate owner absent");};
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
                    if word(services.objects(),id,0xac)?&0xc!=0 && word(services.objects(),id,0xb4)?!=services.counter()
                        && services.objects().get(id).unwrap().state!=0
                        && !services.physics_39dfd8(id)? {return Ok(InitResult::Pending(0x39dfd8));}
                    //39dfd8 itself returns at39e014 when state0 (no effects
                    //or RNG). This is the original guard, not a missing-call
                    //fallback. Later nonzero states still need fullphysics.
                    Step::Support
                },
                Step::Support=>{
                    let Some(above)=services.resolve_stack_link(id,0xa0)? else {return Ok(InitResult::Pending(0x3204e8));};
                    if above.is_none() {
                        let m=services.objects_mut().get_mut(id).unwrap();
                        let flags=u16::from_le_bytes(m.pvars[0x3e..0x40].try_into().unwrap())|1;
                        m.pvars[0x3e..0x40].copy_from_slice(&flags.to_le_bytes());
                    }
                    // Unlike RAC1, a nonzero above pointer does NOT clearbit0.
                    Step::Hit
                },
                Step::Hit=>{services.hit_31dce8(id,HIT_MASK,false)?;Step::Above},
                Step::Above|Step::Below=>{
                    let field=if self.next==Step::Above {0xa0}else{0xa4};
                    let Some(link)=services.resolve_stack_link(id,field)? else {return Ok(InitResult::Pending(0x3204e8));};
                    if link.is_some() && !source_crate_pointer(services.objects(),link)
                        && !services.write_stack_link(id,field,None)? {
                        return Ok(InitResult::Pending(0x3204e8));
                    }
                    if field==0xa0 {Step::Below}else{Step::Clear}
                },
                Step::Clear=>{
                    services.objects_mut().get_mut(id).unwrap().pvars[0x90..0xa0].fill(0);
                    services.objects_mut().get_mut(id).unwrap().state=1;
                    set_word(services,id,0xa8,0);
                    if services.objects().get(id).unwrap().o_class==0x1ff {
                        let next=word(services.objects(),id,0xb4)?.wrapping_add(1);set_word(services,id,0xb4,next);
                    }
                    if word(services.objects(),id,0xcc)?==0 {
                        let flags=word(services.objects(),id,0xac)?|0x14;set_word(services,id,0xac,flags);
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
    /// Required grouped branch, implemented through GroupedStackProgress and
    /// the same World service owner. Default is unresolved, never success.
    fn grouped_39d0c0(&mut self,_owner:MobyId)->Result<bool> {Ok(false)}
    fn group_begin(&mut self,_group:i32,_a2:i32,_a3:i32)->Result<Option<MobyId>> {invalid("GC stack group iterator unbound")}
    fn group_next(&mut self,_from:MobyId,_a2:i32,_a3:i32)->Result<Option<MobyId>> {invalid("GC stack group iterator unbound")}
    /// Some(false): actual322028 returned no carrier. None: unported.
    /// Some(true) must execute322570 and write D0/E0/F4, including the
    /// original optional previous-stack-top occlusion owner, without repair.
    fn carrier_attach(&mut self,_owner:MobyId,_hit:MobyId,_occlusion_owner:Option<MobyId>)->Result<Option<bool>> {Ok(None)}
    /// Must mutate this actual shared1fda00 entry, e.g. advance_path_counter.
    fn path_counter(&mut self,_owner:MobyId,_index:i32)->Result<bool> {Ok(false)}
}

fn rotate_bolt(id:MobyId,s:&mut dyn StackServices) {
    if s.objects().get(id).unwrap().o_class==0x1f4 {
        let turn=s.rng().randi(2)!=0;let m=s.objects_mut().get_mut(id).unwrap();
        if turn && m.rotation[0]==0. && m.rotation[1]==0. {
            m.rotation[2]=crate::hero::physics::add_rot(m.rotation[2],f32::from_bits(0x3fc9_0fdc));
        }
    }
}
fn settle_fields(id:MobyId,s:&mut dyn StackServices)->Result<()> {
    let m=s.objects_mut().get_mut(id).unwrap();let index=m.index as u16;
    m.pvars[0xc4..0xc6].copy_from_slice(&index.to_le_bytes());
    m.pvars[0xb0..0xb4].copy_from_slice(&1f32.to_le_bytes());
    let flags=Buf(&m.pvars).u32(0xac)?|1;m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());Ok(())
}
fn saved_delete(id:MobyId,s:&mut dyn StackServices)->Result<Option<bool>> {
    let m=s.objects().get(id).unwrap();let (class,mission)=(m.o_class,m.mission);
    if matches!(class,0x1f5|0x200) && mission!=0xff {
        let threshold=Buf(&m.pvars).i32(0xf8)?;
        let Some(saved)=s.mission_word(mission)? else {return Ok(None);};
        if saved<threshold {s.delete_311688(id)?;return Ok(Some(true));}
    }
    Ok(Some(false))
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
                    if s.objects().get(id).unwrap().group as u8!=0xff {
                        if !s.grouped_39d0c0(id)? {return Ok(InitResult::Pending(0x31c9d8));}
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    }
                    StackStep::Rotate
                },
                StackStep::Rotate=>{
                    rotate_bolt(id,s);
                    StackStep::Mission
                },
                StackStep::Mission=>{
                    match saved_delete(id,s)? {
                        None=>return Ok(InitResult::Pending(0x39d1b4)),
                        Some(true)=>{self.next=StackStep::Done;return Ok(InitResult::Complete);},Some(false)=>{}
                    }
                    StackStep::Fields
                },
                StackStep::Fields=>{
                    settle_fields(id,s)?;StackStep::Probe
                },
                StackStep::Probe=>{
                    let p=s.objects().get(id).unwrap().position;
                    self.hit=s.line([p[0],p[1],p[2]+0.5],[p[0],p[1],f32::from_bits(0x3dcc_cccd)],2,None)?;
                    StackStep::Resolve
                },
                StackStep::Resolve=>{
                    let Some(hit)=self.hit.as_ref() else {
                        if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d254));}
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    };
                    if hit.moby.is_none() {
                        let m=s.objects_mut().get_mut(id).unwrap();m.position[2]=hit.point[2];
                        if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d274));}
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    }
                    if source_crate_pointer(s.objects(),hit.moby) {
                        self.next=StackStep::Done;return Ok(InitResult::Complete);
                    }
                    if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d28c));}
                    let m=s.objects_mut().get_mut(id).unwrap();
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

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum GroupStep {Begin,FirstBegin,FirstCheck,Rotate,Mission,Fields,Probe,Resolve,LinkBelow,Pose,LinkAbove,FirstNext,
    SecondBegin,SecondCheck,SecondProbe,SecondResolve,SecondCarrierLink,ChainBegin,ChainInherit,ChainPath,ChainNext,SecondNext,
    ThirdBegin,ThirdOuter,ThirdInnerBegin,ThirdInner,ThirdInnerNext,ThirdOuterNext,Done}

/// The original three passes, using ONE caller-provided World iterator.
/// No cloned group owns any moby; all saved fields are native owner references
/// and source continuation locals. In particular pass3's inner walk clobbers
/// the same iterator that its outer walk must subsequently seek within.
#[derive(Clone,Debug)]
pub struct GroupedStackProgress {
    next:GroupStep,owner:Option<MobyId>,group:i32,current:Option<MobyId>,bottom:Option<MobyId>,
    below:Option<MobyId>,above:Option<MobyId>,stack_top:Option<MobyId>,outer:Option<MobyId>,inner:Option<MobyId>,
    hit:Option<rc_game::collision_query::CollOutput>,chain_seen:Vec<MobyId>,
    pub last_distance:Option<f32>,
}
impl Default for GroupedStackProgress {
    fn default()->Self {Self {next:GroupStep::Begin,owner:None,group:0,current:None,bottom:None,below:None,above:None,
        stack_top:None,outer:None,inner:None,hit:None,chain_seen:Vec::new(),last_distance:None}}
}
impl GroupedStackProgress {
    /// Starts at39d2e4, AFTER StackInitProgress wrote caller.ac|4. Every
    /// group_begin/group_next uses original arguments(0,0); their global
    /// cursor/current/index effects belong to the existing World service.
    pub fn resume(&mut self,owner:MobyId,s:&mut dyn StackServices)->Result<InitResult> {
        if self.owner.is_some_and(|id|id!=owner) {return invalid("GC grouped stack caller changed");}
        loop {
            let following=match self.next {
                GroupStep::Begin=>{
                    let Some(m)=s.objects().get(owner) else {return invalid("GC grouped stack owner absent");};
                    if m.pvars.len()<0x100 || Buf(&m.pvars).u32(0xac)?&4==0 {return invalid("GC grouped stack requires completed source prefix");}
                    self.group=m.group as u8 as i32;
                    if self.group==0xff {return invalid("GC ungrouped owner routed to grouped stack");}
                    self.owner=Some(owner);GroupStep::FirstBegin
                },
                GroupStep::FirstBegin=>{self.current=s.group_begin(self.group,0,0)?;GroupStep::FirstCheck},
                GroupStep::FirstCheck=>{
                    match self.current {
                        None=>GroupStep::SecondBegin,
                        Some(id) if !source_crate_pointer(s.objects(),Some(id))=>GroupStep::FirstNext,
                        Some(id)=>{
                            if s.objects().get(id).unwrap().pvars.len()<0x100 {return invalid("GC group crate pvar block too short");}
                            GroupStep::Rotate
                        }
                    }
                },
                GroupStep::Rotate=>{rotate_bolt(self.current.unwrap(),s);GroupStep::Mission},
                GroupStep::Mission=>{
                    match saved_delete(self.current.unwrap(),s)? {
                        None=>return Ok(InitResult::Pending(0x39d3d0)),Some(true)=>GroupStep::FirstNext,Some(false)=>GroupStep::Fields
                    }
                },
                GroupStep::Fields=>{settle_fields(self.current.unwrap(),s)?;GroupStep::Probe},
                GroupStep::Probe=>{
                    let p=s.objects().get(self.current.unwrap()).unwrap().position;
                    self.hit=s.line([p[0],p[1],p[2]+0.5],[p[0],p[1],f32::from_bits(0x3dcc_cccd)],2,None)?;
                    GroupStep::Resolve
                },
                GroupStep::Resolve=>{
                    let id=self.current.unwrap();
                    match self.hit.as_ref() {
                        None=>{
                            if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d490));}
                            GroupStep::FirstNext
                        },
                        Some(hit) if hit.moby.is_none()=>{
                            s.objects_mut().get_mut(id).unwrap().position[2]=hit.point[2];
                            if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d490));}
                            GroupStep::FirstNext
                        },
                        Some(hit)=>{
                            let target=hit.moby.unwrap();
                            let Some(other)=s.objects().get(target) else {return invalid("GC stack collision owner absent");};
                            let crate_hit=source_crate_pointer(s.objects(),Some(target));
                            let special=(other.o_class as u16 as u32).wrapping_sub(0x128)<2;
                            let same_group=other.group==s.objects().get(id).unwrap().group;
                            if !crate_hit && !special {
                                if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d4bc));}
                                let m=s.objects_mut().get_mut(id).unwrap();m.update_dist=0xff;
                                let flags=Buf(&m.pvars).u32(0xac)?|8;m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());
                                GroupStep::FirstNext
                            }else if other.pvars.is_empty() || !same_group {
                                if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d4f4));}
                                GroupStep::FirstNext
                            }else if crate_hit {self.below=Some(target);GroupStep::LinkBelow}
                            else{
                                if !s.write_stack_link(id,0xa4,None)? {return Ok(InitResult::Pending(0x39d504));}
                                s.objects_mut().get_mut(id).unwrap().position[2]=hit.point[2];GroupStep::FirstNext
                            }
                        }
                    }
                },
                GroupStep::LinkBelow=>{
                    if !s.write_stack_link(self.current.unwrap(),0xa4,self.below)? {return Ok(InitResult::Pending(0x39d50c));}
                    GroupStep::Pose
                },
                GroupStep::Pose=>{
                    let lower=s.objects().get(self.below.unwrap()).unwrap();
                    let position=rc_game::hero::physics::vadd(lower.position.map(rc_game::ps2v::Pf::f),lower.rows[2].map(rc_game::ps2v::Pf::f));
                    s.objects_mut().get_mut(self.current.unwrap()).unwrap().position=position.map(rc_game::ps2v::Pf::to_f32);
                    GroupStep::LinkAbove
                },
                GroupStep::LinkAbove=>{
                    let id=self.current.unwrap();
                    if !source_crate_pointer(s.objects(),Some(id)) {return Ok(InitResult::Pending(0x2eeb90));}
                    if !s.write_stack_link(self.below.unwrap(),0xa0,Some(id))? {return Ok(InitResult::Pending(0x39d550));}
                    GroupStep::FirstNext
                },
                GroupStep::FirstNext=>{self.current=s.group_next(self.current.unwrap(),0,0)?;GroupStep::FirstCheck},
                GroupStep::SecondBegin=>{self.bottom=s.group_begin(self.group,0,0)?;GroupStep::SecondCheck},
                GroupStep::SecondCheck=>{
                    match self.bottom {
                        None=>GroupStep::ThirdBegin,
                        Some(id) if !source_crate_pointer(s.objects(),Some(id))=>GroupStep::SecondNext,
                        Some(id)=>{
                            let Some(below)=s.resolve_stack_link(id,0xa4)? else {return Ok(InitResult::Pending(0x39d5d4));};
                            if below.is_some() {GroupStep::SecondNext}else{GroupStep::SecondProbe}
                        }
                    }
                },
                GroupStep::SecondProbe=>{
                    let id=self.bottom.unwrap();let m=s.objects().get(id).unwrap();let p=m.position;
                    let up=(rc_game::ps2v::Pf::f(Buf(&m.pvars).f32(0xb0)?)*rc_game::ps2v::Pf::f(0.5)).to_f32();
                    //Sourcea3 is sp40, the previous stack's top, not always0.
                    self.hit=s.line([p[0],p[1],p[2]+up],[p[0],p[1],f32::from_bits(0x3dcc_cccd)],2,self.stack_top)?;
                    GroupStep::SecondResolve
                },
                GroupStep::SecondResolve=>{
                    if let Some(target)=self.hit.as_ref().and_then(|h|h.moby) {
                        match s.carrier_attach(self.bottom.unwrap(),target,self.stack_top)? {
                            None=>return Ok(InitResult::Pending(0x322028)),Some(false)=>GroupStep::ChainBegin,
                            Some(true)=>{
                                let m=s.objects_mut().get_mut(self.bottom.unwrap()).unwrap();
                                let flags=Buf(&m.pvars).u32(0xac)?|8;m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());
                                GroupStep::SecondCarrierLink
                            }
                        }
                    }else{GroupStep::ChainBegin}
                },
                GroupStep::SecondCarrierLink=>{
                    let target=self.hit.as_ref().and_then(|h|h.moby);
                    if !s.write_stack_link(self.bottom.unwrap(),0xf0,target)? {return Ok(InitResult::Pending(0x39d6a0));}
                    GroupStep::ChainBegin
                },
                GroupStep::ChainBegin=>{
                    self.current=self.bottom;self.stack_top=self.bottom;
                    self.chain_seen.clear();self.chain_seen.push(self.bottom.unwrap());GroupStep::ChainInherit
                },
                GroupStep::ChainInherit=>{
                    let id=self.current.unwrap();
                    let Some(below)=s.resolve_stack_link(id,0xa4)? else {return Ok(InitResult::Pending(0x39d6e4));};
                    if let Some(lower)=below {
                        let Some(m)=s.objects().get(lower) else {return invalid("GC stack below owner absent");};
                        if !m.pvars.is_empty() {
                            if let result@InitResult::Pending(_)=inherit_carrier_pose(id,lower,s)? {return Ok(result);}
                        }
                    }
                    GroupStep::ChainPath
                },
                GroupStep::ChainPath=>{
                    let id=self.current.unwrap();let Some(above)=s.resolve_stack_link(id,0xa0)? else {return Ok(InitResult::Pending(0x39d76c));};
                    self.above=above;let m=s.objects().get(id).unwrap();
                    let needs=Buf(&m.pvars).i16(0xc6)?!=0 || matches!(m.o_class,0x1f5|0x200);
                    if needs && above.is_some() {
                        let index=Buf(&m.pvars).i32(0xc0)?;
                        if index!=-1 && !s.path_counter(id,index)? {return Ok(InitResult::Pending(0x39d784));}
                    }
                    GroupStep::ChainNext
                },
                GroupStep::ChainNext=>{
                    if let Some(above)=self.above {
                        if self.chain_seen.contains(&above) {return invalid("GC stack above cycle watchdog");}
                        self.chain_seen.push(above);self.current=Some(above);self.stack_top=Some(above);GroupStep::ChainInherit
                    }else{GroupStep::SecondNext}
                },
                GroupStep::SecondNext=>{self.bottom=s.group_next(self.bottom.unwrap(),0,0)?;GroupStep::SecondCheck},
                GroupStep::ThirdBegin=>{self.outer=s.group_begin(self.group,0,0)?;GroupStep::ThirdOuter},
                GroupStep::ThirdOuter=>{
                    match self.outer {None=>GroupStep::Done,Some(id) if source_crate_pointer(s.objects(),Some(id))=>GroupStep::ThirdInnerBegin,
                        Some(_)=>GroupStep::ThirdOuterNext}
                },
                GroupStep::ThirdInnerBegin=>{self.inner=s.group_next(self.outer.unwrap(),0,0)?;GroupStep::ThirdInner},
                GroupStep::ThirdInner=>{
                    match self.inner {
                        None=>GroupStep::ThirdOuterNext,
                        Some(id)=>{
                            if source_crate_pointer(s.objects(),Some(id)) {
                                let a=s.objects().get(self.outer.unwrap()).unwrap().position.map(rc_game::ps2v::Pf::f);
                                let b=s.objects().get(id).unwrap().position.map(rc_game::ps2v::Pf::f);
                                self.last_distance=Some(rc_game::hero::physics::dist3(a,b).to_f32());
                            }
                            GroupStep::ThirdInnerNext
                        }
                    }
                },
                GroupStep::ThirdInnerNext=>{self.inner=s.group_next(self.inner.unwrap(),0,0)?;GroupStep::ThirdInner},
                GroupStep::ThirdOuterNext=>{self.outer=s.group_next(self.outer.unwrap(),0,0)?;GroupStep::ThirdOuter},
                GroupStep::Done=>return Ok(InitResult::Complete),
            };
            self.next=following;
        }
    }
}

/// Complete39d6f4..39d748 carrier inheritance after the caller has resolved
/// the real below owner. Reuses actual pvar buffers and typed links. Unlike
/// RAC1 stack_init, GC does NOT copy below.pvarcc into the upper crate.
pub fn inherit_carrier_pose(upper:MobyId,below:MobyId,s:&mut dyn StackServices)->Result<InitResult> {
    let Some(carrier)=s.resolve_stack_link(below,0xf0)? else {return Ok(InitResult::Pending(0x39d6fc));};
    let Some(carrier)=carrier else {return Ok(InitResult::Complete);};
    let Some(source)=s.objects().get(below) else {return invalid("GC below carrier owner absent");};
    if source.pvars.len()<0xf8 || s.objects().get(upper).is_none_or(|m|m.pvars.len()<0xf8) {
        return invalid("GC carrier inheritance pvar block too short");
    }
    let d0:[u8;16]=source.pvars[0xd0..0xe0].try_into().unwrap();
    let e0:[u8;16]=source.pvars[0xe0..0xf0].try_into().unwrap();
    let height=Buf(&source.pvars).f32(0xf4)?;
    let m=s.objects_mut().get_mut(upper).unwrap();m.update_dist=0xff;
    let flags=Buf(&m.pvars).u32(0xac)?|8;m.pvars[0xac..0xb0].copy_from_slice(&flags.to_le_bytes());
    m.pvars[0xd0..0xe0].copy_from_slice(&d0);m.pvars[0xe0..0xf0].copy_from_slice(&e0);
    if !s.write_stack_link(upper,0xf0,Some(carrier))? {return Ok(InitResult::Pending(0x39d738));}
    let height=(rc_game::ps2v::Pf::f(height)+rc_game::ps2v::Pf::ONE).to_f32();
    s.objects_mut().get_mut(upper).unwrap().pvars[0xf4..0xf8].copy_from_slice(&height.to_le_bytes());
    Ok(InitResult::Complete)
}

/// Complete39d774..39d838 once the actual1fda00 table entry has been resolved
/// to its native mutable point words. RAC1 crate_::path_counter supplies the
/// identical control-flow reuse map; its private R1World wrapper is not used.
/// Caller must supply this owner's real selected table, never a fixture clone
/// standing in for shared state. XYZ and all other owned pvars stay untouched.
pub fn advance_path_counter(objects:&mut Objects,owner:MobyId,points:&mut [[u32;4]])->Result<()> {
    let Some(m)=objects.get_mut(owner) else {return invalid("GC path-counter owner absent");};
    if m.pvars.len()<0xc4 || points.len()>i32::MAX as usize {return invalid("GC path-counter extent unsupported");}
    let count=points.len() as i32;let state=points.first().map_or(0,|p|p[3] as i32);
    if count==0 || matches!(state,1|3|0x111) {m.pvars[0xc0..0xc4].copy_from_slice(&(-1i32).to_le_bytes());return Ok(());}
    if (state.wrapping_sub(0x11) as u32)<0x100 {
        if state.wrapping_sub(0x10)<count {points[0][3]=state.wrapping_add(1) as u32;}
        else {m.pvars[0xc0..0xc4].copy_from_slice(&(-1i32).to_le_bytes());}
    }else{
        points[0][3]=0x11;
        // Source really clears the first word again as well as all others.
        // Preserve that order rather than repairing the original behavior.
        for point in points {point[3]=0;}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::moby_runtime::Moby;
    struct Fixture {objects:Objects,counter:u32,events:Vec<u32>,rng:rc_game::rng::Rng,
        queries:std::collections::VecDeque<Option<rc_game::collision_query::CollOutput>>,mission:Option<i32>,native_ids:bool,
        groups:Vec<Vec<MobyId>>,walk:Vec<(char,usize)>,current:Option<MobyId>,ignores:Vec<Option<MobyId>>}
    impl CrateServices for Fixture {
        fn objects(&self)->&Objects {&self.objects}
        fn objects_mut(&mut self)->&mut Objects {&mut self.objects}
        fn counter(&self)->u32 {self.counter}
        fn physics_39dfd8(&mut self,_id:MobyId)->Result<bool> {self.events.push(0x39dfd8);Ok(false)}
        fn hit_31dce8(&mut self,_id:MobyId,mask:u32,keep:bool)->Result<()> {
            assert_eq!(mask,HIT_MASK);assert!(!keep);self.events.push(0x31dce8);Ok(())
        }
        fn stack_39d0c0(&mut self,_id:MobyId)->Result<bool> {self.events.push(0x39d0c0);Ok(false)}
        fn icon_39ff50(&mut self,_id:MobyId,_mode:i32)->Result<Option<u32>> {self.events.push(0x39ff50);Ok(None)}
        fn resolve_stack_link(&mut self,id:MobyId,field:usize)->Result<Option<Option<MobyId>>> {
            let raw=Buf(&self.objects.get(id).unwrap().pvars).i32(field)?;
            if self.native_ids {return Ok(Some(if raw==-1 {None}else{Some(raw as usize)}));}
            Ok(if raw==0 {Some(None)}else{None})
        }
        fn write_stack_link(&mut self,id:MobyId,field:usize,target:Option<MobyId>)->Result<bool> {
            let value=if self.native_ids {target.map_or(-1,|id|id as i32)}else{
                if target.is_some() {return Ok(false);}0
            };
            self.objects.get_mut(id).unwrap().pvars[field..field+4].copy_from_slice(&value.to_le_bytes());Ok(true)
        }
    }
    impl StackServices for Fixture {
        fn rng(&mut self)->&mut rc_game::rng::Rng {&mut self.rng}
        fn line(&mut self,a:[f32;3],b:[f32;3],flags:u32,ignore:Option<MobyId>)->Result<Option<rc_game::collision_query::CollOutput>> {
            assert_eq!(flags,2);self.ignores.push(ignore);assert_eq!(b[2],f32::from_bits(0x3dcc_cccd));
            assert_eq!([a[0],a[1]],[b[0],b[1]]);self.events.push(0x2de878);
            Ok(self.queries.pop_front().expect("stack probe replay"))
        }
        fn mission_word(&mut self,_mission:u8)->Result<Option<i32>> {self.events.push(0x39d1b4);Ok(self.mission)}
        fn delete_311688(&mut self,id:MobyId)->Result<()> {self.events.push(0x311688);self.objects.delete(id,self.counter,|_|{})}
        fn group_begin(&mut self,group:i32,a2:i32,a3:i32)->Result<Option<MobyId>> {
            assert_eq!((a2,a3),(0,0));self.walk.push(('b',group as usize));
            self.current=self.groups[group as usize].iter().copied().find(|&id|self.objects.get(id).unwrap().state<0x80);Ok(self.current)
        }
        fn group_next(&mut self,from:MobyId,a2:i32,a3:i32)->Result<Option<MobyId>> {
            assert_eq!((a2,a3),(0,0));self.walk.push(('n',from));
            let group=self.objects.get(from).unwrap().group as u8 as usize;
            let list=&self.groups[group];let at=list.iter().position(|&id|id==from).unwrap();
            self.current=list[at+1..].iter().copied().find(|&id|self.objects.get(id).unwrap().state<0x80);Ok(self.current)
        }
    }
    fn fixture(class:i16)->Fixture {
        let mut m=Moby::zeroed();m.o_class=class;m.state=0;m.update_fn=Some(UPDATE);m.hit_slot=9;
        m.pvars=vec![0;0x100];m.pvars[0x40..0x50].fill(0xa5);m.pvars[0x90..0xa0].fill(0xff);
        m.pvars[0xfc..0x100].copy_from_slice(&0xdead_beefu32.to_le_bytes());
        Fixture {objects:Objects::new(vec![m],0),counter:0,events:vec![],rng:rc_game::rng::Rng {state:1234},queries:Default::default(),mission:None,native_ids:false,
            groups:vec![],walk:vec![],current:None,ignores:vec![]}
    }
    #[test]
    fn required_stack_blocks_after_actual_owned_state_and_flags_without_replay() {
        let mut f=fixture(0x1ff);let mut progress=CrateInitProgress::default();
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        let m=f.objects.get(0).unwrap();assert_eq!(m.state,1);
        assert_eq!(&m.pvars[0x90..0xa0],&[0;16]);assert_eq!(&m.pvars[0x40..0x50],&[0xa5;16]);
        assert_eq!(word(&f.objects,0,0xac).unwrap(),0x14);assert_eq!(word(&f.objects,0,0xb4).unwrap(),1);
        assert_eq!(word(&f.objects,0,0xfc).unwrap(),0xdead_beef);assert_eq!(m.hit_slot,9);
        assert_eq!(f.events,vec![0x31dce8,0x39d0c0]);
        assert_eq!(progress.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        assert_eq!(word(&f.objects,0,0xb4).unwrap(),1,"class1ff increment must not replay");
        assert_eq!(f.events,vec![0x31dce8,0x39d0c0,0x39d0c0]);
    }
    #[test]
    fn source_skip_stack_branch_finishes_bolt_crate_but_not_required_icon() {
        let mut f=fixture(0x1f4);set_word(&mut f,0,0xcc,1);let mut p=CrateInitProgress::default();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert_eq!(word(&f.objects,0,0xac).unwrap(),0,"cc branch skips BOTH flags and stack");
        assert_eq!(word(&f.objects,0,0xfc).unwrap(),0);assert_eq!(f.objects.get(0).unwrap().hit_slot,9);
        assert_eq!(f.events,vec![0x31dce8]);
        let mut f=fixture(0x1f5);set_word(&mut f,0,0xcc,1);
        assert_eq!(CrateInitProgress::default().resume(0,&mut f).unwrap(),InitResult::Pending(0x39ff50));
        assert_eq!(f.events,vec![0x31dce8,0x39ff50]);assert_eq!(word(&f.objects,0,0xfc).unwrap(),0);
    }
    #[test]
    fn original_physics_state_zero_return_does_not_invent_a_physics_dependency() {
        let mut f=fixture(0x1f4);set_word(&mut f,0,0xac,4);f.counter=1;
        assert_eq!(CrateInitProgress::default().resume(0,&mut f).unwrap(),InitResult::Pending(0x39d0c0));
        assert_eq!(f.objects.get(0).unwrap().state,1);assert_eq!(word(&f.objects,0,0x3c).unwrap(),0x10000);
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
    #[test]
    fn verified_native_id_zero_is_owner_not_null_and_clear_uses_bound_null_encoding() {
        let mut f=fixture(0x1f4);let mut crate_m=f.objects.get(0).unwrap().clone();
        crate_m.pvars[0xa4..0xa8].copy_from_slice(&(-1i32).to_le_bytes());
        crate_m.pvars[0xcc..0xd0].copy_from_slice(&1u32.to_le_bytes());
        let mut hero=Moby::zeroed();hero.o_class=0;
        f.objects=Objects::new(vec![hero,crate_m],0);f.native_ids=true;
        assert_eq!(CrateInitProgress::default().resume(1,&mut f).unwrap(),InitResult::Complete);
        let m=f.objects.get(1).unwrap();
        assert_eq!(Buf(&m.pvars).i32(0xa0).unwrap(),-1);
        assert_eq!(Buf(&m.pvars).u16(0x3e).unwrap(),0,"nonnull owner0 is tested before sourceclass rejection clears it");
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
        assert_eq!(word(&f.objects,0,0xac).unwrap(),5);assert_eq!(f.events,vec![0x2de878]);
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
        assert_eq!(word(&f.objects,0,0xac).unwrap(),4);assert_eq!(f.rng,expected);assert!(f.events.is_empty());
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
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));assert_eq!(word(&f.objects,0,0xac).unwrap(),13);
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));assert_eq!(f.events,vec![0x2de878]);
    }
    #[test]
    fn grouped_carrier_inheritance_copies_exact_pose_lanes_without_rac1_cc_write() {
        let mut f=fixture(0x1f4);let mut lower=f.objects.get(0).unwrap().clone();
        for i in 0..32 {lower.pvars[0xd0+i]=i as u8+1;}
        lower.pvars[0xf0..0xf4].copy_from_slice(&2i32.to_le_bytes());
        lower.pvars[0xf4..0xf8].copy_from_slice(&2.5f32.to_le_bytes());
        lower.pvars[0xcc..0xd0].copy_from_slice(&0xdefu32.to_le_bytes());
        let mut upper=f.objects.get(0).unwrap().clone();upper.pvars[0xcc..0xd0].copy_from_slice(&0xabcu32.to_le_bytes());
        let carrier=Moby::zeroed();f.objects=Objects::new(vec![upper,lower,carrier],0);f.native_ids=true;
        assert_eq!(inherit_carrier_pose(0,1,&mut f).unwrap(),InitResult::Complete);
        let (u,l)=(f.objects.get(0).unwrap(),f.objects.get(1).unwrap());
        assert_eq!(&u.pvars[0xd0..0xf0],&l.pvars[0xd0..0xf0]);
        assert_eq!(Buf(&u.pvars).i32(0xf0).unwrap(),2);assert_eq!(Buf(&u.pvars).f32(0xf4).unwrap(),3.5);
        assert_eq!(Buf(&u.pvars).u32(0xcc).unwrap(),0xabc);assert_eq!(u.update_dist,0xff);
        assert_eq!(Buf(&u.pvars).u32(0xac).unwrap(),8);
    }
    #[test]
    fn shared_path_counter_preserves_xyz_and_original_first_point_clear_quirk() {
        let mut f=fixture(0x1f4);set_word(&mut f,0,0xc0,7);
        let mut points=[[1,2,3,0x11],[4,5,6,77]];
        advance_path_counter(&mut f.objects,0,&mut points).unwrap();assert_eq!(points[0][3],0x12);
        assert_eq!(word(&f.objects,0,0xc0).unwrap(),7);
        advance_path_counter(&mut f.objects,0,&mut points).unwrap();assert_eq!(word(&f.objects,0,0xc0).unwrap(),u32::MAX);
        points[0][3]=0;advance_path_counter(&mut f.objects,0,&mut points).unwrap();
        assert_eq!(points,[[1,2,3,0],[4,5,6,0]],"source sets11 then clears everypointW includingfirst");
        for state in [1,3,0x111] {
            set_word(&mut f,0,0xc0,7);points[0][3]=state;
            advance_path_counter(&mut f.objects,0,&mut points).unwrap();assert_eq!(word(&f.objects,0,0xc0).unwrap(),u32::MAX);
            assert_eq!(points[0][3],state);
        }
    }
    fn grouped_fixture()->Fixture {
        let mut f=fixture(0x1f4);let mut a=f.objects.get(0).unwrap().clone();let mut b=a.clone();
        a.group=130u8 as i8;b.group=130u8 as i8;a.position=[10.,20.,7.,1.];b.position=[12.,20.,7.,1.];
        a.pvars[0xac..0xb0].copy_from_slice(&4u32.to_le_bytes());
        f.objects=Objects::new(vec![a,b],0);f.groups=vec![vec![];131];f.groups[130]=vec![0,1];f
    }
    #[test]
    fn grouped_three_passes_preserve_unsigned_group_rng_nested_walk_and_previous_top_ignore() {
        let mut f=grouped_fixture();f.queries=[Some(floor(None)),Some(floor(None)),None,None].into();
        let mut expected=f.rng;expected.randi(2);expected.randi(2);
        let mut p=GroupedStackProgress::default();assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert_eq!(f.rng,expected);assert_eq!(f.current,None);assert_eq!(p.last_distance,Some(2.));
        assert_eq!(f.ignores,vec![None,None,None,Some(0)]);
        assert_eq!(f.walk,vec![('b',130),('n',0),('n',1),('b',130),('n',0),('n',1),
            ('b',130),('n',0),('n',1),('n',0),('n',1),('n',1)]);
        assert_eq!(word(&f.objects,0,0xac).unwrap(),5);assert_eq!(word(&f.objects,1,0xac).unwrap(),1);
        let walks=f.walk.len();assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert_eq!(f.walk.len(),walks);assert_eq!(f.rng,expected);
    }
    #[test]
    fn grouped_carrier_pending_preserves_prior_queries_and_single_rng_stream() {
        let mut f=grouped_fixture();let carrier=Moby::zeroed();let a=f.objects.get(0).unwrap().clone();let b=f.objects.get(1).unwrap().clone();
        f.objects=Objects::new(vec![a,b,carrier],0);
        f.queries=[Some(floor(None)),Some(floor(None)),Some(floor(Some(2)))].into();
        let mut p=GroupedStackProgress::default();assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));
        let rng=f.rng;let queries=f.events.len();let walks=f.walk.len();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x322028));
        assert_eq!(f.rng,rng);assert_eq!(f.events.len(),queries);assert_eq!(f.walk.len(),walks);
    }
    fn grouped_chain_fixture()->Fixture {
        let mut f=grouped_fixture();f.native_ids=true;
        for id in 0..2 {
            let m=f.objects.get_mut(id).unwrap();m.rows[2]=[0.,0.,1.,0.];
            for field in [0xa0,0xa4,0xf0] {m.pvars[field..field+4].copy_from_slice(&(-1i32).to_le_bytes());}
        }
        f.queries=[Some(floor(None)),Some(floor(Some(0))),None].into();f
    }
    #[test]
    fn grouped_first_pass_builds_actual_stack_with_native_owner_zero_and_all_passes_finish() {
        let mut f=grouped_chain_fixture();let mut p=GroupedStackProgress::default();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Complete);
        assert_eq!(f.resolve_stack_link(0,0xa0).unwrap(),Some(Some(1)));
        assert_eq!(f.resolve_stack_link(1,0xa4).unwrap(),Some(Some(0)));
        assert_eq!(f.resolve_stack_link(1,0xa0).unwrap(),Some(None));
        assert_eq!(f.objects.get(1).unwrap().position,[10.,20.,7.,1.]);
        assert_eq!(f.ignores,vec![None,None,None]);assert_eq!(p.last_distance,Some(1.));
        assert_eq!(f.walk.iter().filter(|(op,_)|*op=='b').count(),3);
    }
    #[test]
    fn grouped_required_path_state_blocks_before_third_pass_without_replaying_prior_effects() {
        let mut f=grouped_chain_fixture();
        f.objects.get_mut(0).unwrap().pvars[0xc6..0xc8].copy_from_slice(&1i16.to_le_bytes());
        let mut p=GroupedStackProgress::default();assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d784));
        assert_eq!(f.walk.iter().filter(|(op,_)|*op=='b').count(),2);
        let rng=f.rng;let calls=f.events.len();let walks=f.walk.len();
        assert_eq!(p.resume(0,&mut f).unwrap(),InitResult::Pending(0x39d784));
        assert_eq!((f.rng,f.events.len(),f.walk.len()),(rng,calls,walks));
    }
}
