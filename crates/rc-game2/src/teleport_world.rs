//! Ordered, resumable native prefix of GC HeroTeleport2b9e68. A pending
//! service is a blocker, never permission to complete the caller's initializer.
//! RAC1 Trail::kill is not compatible: GC retains ten actual allocated owners
//! and deletion must go through the existing World/Objects lifetime and grid.
use crate::{hero::Hero,init_1330::{Pending1330,TeleportRequest},world::World};
use rc_formats::{buf::{invalid,Result},moby_anim::MobyAnimClass};
use rc_game::{hero::anim::RatchetAnim,moby_runtime::MobyId,pad::PadState};

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct TrackedObjects {
    /// Original inline slots +120..+144; these are references, not owners.
    pub slots:[Option<MobyId>;10],
    pub count:i32,
    pub active:u32,
}

/// Newly represented lanes remain on the single existing GC Hero. These
/// defaults describe a native test fixture, not recovered original New Game.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct TeleportStorage {
    /// Unknown after a native XYZ-only collision result is copied by SQ.
    pub position_w:Option<u32>,
    pub euler_w:u32,
    pub trails:[TrackedObjects;2],
    pub source_18a1c0:u32,
    pub source_18a1c4:u32,
    pub byte_18c30f:u8,
}

/// Borrowed-runtime globals with one native owner. Missing values are not
/// inferred zeros/null pointers; only explicit fixtures may supply such inputs.
#[derive(Clone,Debug)]
pub struct TeleportGlobals {
    pub source_1b8b80:Option<[[u8;16];8]>,
    pub word_1b9000:Option<i32>,
    pub actors_3e0b28:Option<crate::teleport_actors::ActorBindings>,
    pub actors_403d98:Option<crate::teleport_actors::ActorBindings>,
}
struct StateServices<'a> {world:&'a mut World,hero:MobyId,counter:u32}
impl crate::hero::teleport_state::StateZeroServices for StateServices<'_> {
    fn clear_moby_source_9c(&mut self)->Result<()> {
        self.world.gc[self.hero].source_9c=0;Ok(())
    }
    fn end_hero_trail(&mut self,record:&mut TrackedObjects)->Result<()> {
        self.world.end_tracked_objects(record,self.counter)
    }
    fn clear_global_1b8b80(&mut self)->Result<bool> {
        let Some(records)=self.world.teleport_globals.as_mut().and_then(|g|g.source_1b8b80.as_mut()) else {return Ok(false);};
        crate::hero::teleport_state::clear_source_1b8b80(records);Ok(true)
    }
    fn clear_camera_1b9000(&mut self)->Result<bool> {
        let Some(word)=self.world.teleport_globals.as_mut().and_then(|g|g.word_1b9000.as_mut()) else {return Ok(false);};
        *word=0;Ok(true)
    }
    fn stop_group_audio(&mut self,_hero:&mut Hero)->Result<bool> {
        // Requires actual nine handle/voice ownership and3602f0 transport.
        // No opaque voice id is reinterpreted as a native AudioSystem handle.
        Ok(false)
    }
    fn rng(&mut self)->&mut rc_game::rng::Rng {&mut self.world.rng}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TeleportStep {Pose,Motion,Trail0,Trail1,Flags,PutAway,Modifiers,State,Animation,GroupFlag,Probe,Snap,Actor0,Actor1,Camera,Lighting,Done}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TeleportProgress {pub request:TeleportRequest,pub hero:MobyId,pub next:TeleportStep}

/// Explicit caller-bound original globals; no inferred New Game defaults.
#[derive(Clone,Copy,Debug)]
pub struct GroundGlobals {
    pub source_1b8ff0:i32,
    pub source_1a79f8:i32,
    pub source_gp_minus_6000:i32,
}

impl World {
    /// Existing ordered teleport plus actual collision services. The legacy
    /// entry point still stops at Probe when no original collision is bound.
    /// This does not complete class1330 or bypass its remaining camera/tail.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_1330_teleport_with_collision(&mut self,hero_id:MobyId,hero:&mut Hero,pad:&mut PadState,
        anim:&mut RatchetAnim,class:&MobyAnimClass,layer_present:Option<bool>,counter:u32,
        collision:&rc_formats::collision::Collision,rigid:&crate::collision::LiveRigidCollision,
        globals:GroundGlobals)->Result<TeleportStep> {
        let next=self.resume_1330_teleport(hero_id,hero,pad,anim,class,layer_present,counter)?;
        if next!=TeleportStep::Probe {return Ok(next);}
        use crate::hero::ground_original::{GroundServices,GroundResult,Inputs};
        struct Queries<'a> {
            world:&'a World,rigid:&'a crate::collision::LiveRigidCollision,
            mesh:&'a rc_formats::collision::Collision,hero:MobyId,
        }
        impl GroundServices for Queries<'_> {
            fn line(&mut self,a:[f32;3],b:[f32;3],flags:u32)->Result<Option<rc_game::collision_query::CollOutput>> {
                Ok(rc_game::collision_query::coll_line_m(self.mesh,Some(&self.rigid.scene(self.world)),a,b,
                    rc_game::collision_query::QueryFlags(flags),Some(self.hero)))
            }
            fn point_w(&mut self)->Option<u32> {None} // Native query exposes XYZ only.
        }
        let owner=self.objects.get(hero_id).unwrap();
        let input=Inputs {frame:crate::hero::probe::Frame {
            source_1b8ff0:globals.source_1b8ff0,source_18a0e0:hero.original_ground.gravity_direction,
            moby_up:owner.rows[2]},source_1a79f8:globals.source_1a79f8,
            source_gp_minus_6000:globals.source_gp_minus_6000,moby_euler:owner.rotation};
        let mut progress=self.teleport_ground_progress.take().unwrap_or_default();
        let mut storage=std::mem::take(&mut hero.original_ground);
        let result=progress.resume(hero,&mut storage,input,&mut Queries {world:self,rigid,mesh:collision,hero:hero_id});
        hero.original_ground=storage;self.teleport_ground_progress=Some(progress);
        match result? {
            GroundResult::Pending {source,..}=>{
                let request=self.teleport_progress.as_ref().unwrap().request;
                self.pending_1330=Some(Pending1330::TeleportService {request,source});Ok(TeleportStep::Probe)
            },
            GroundResult::Complete=>{
                self.teleport_progress.as_mut().unwrap().next=TeleportStep::Snap;
                self.resume_1330_teleport(hero_id,hero,pad,anim,class,layer_present,counter)
            },
        }
    }
    /// Complete324140, preserving count, inactive slots and deletion order.
    /// Bad native bindings fail before mutation; no new owner or RNG stream.
    pub fn end_tracked_objects(&mut self,record:&mut TrackedObjects,counter:u32)->Result<()> {
        if record.active==0 {return Ok(());}
        let count=record.count.max(0) as usize;
        if count>record.slots.len() {return invalid("GC tracked object count exceeds original inline capacity");}
        for id in record.slots[..count].iter().flatten() {
            if self.objects.get(*id).is_none() {return invalid("GC tracked object binding absent");}
        }
        for slot in &mut record.slots[..count] {
            if let Some(id)=*slot {
                let grid=&mut self.grid;
                self.objects.delete(id,counter,|m|grid.remove(m))?;
                *slot=None;
            }
        }
        record.active=0;Ok(())
    }

    /// Run only recovered services, in original order, on actual World/Hero/
    /// Pad/animation owners. Returning a pending step is NOT teleport completion.
    /// Unknown layer presence is required only on the source nonzero branch.
    #[allow(clippy::too_many_arguments)]
    pub fn resume_1330_teleport(&mut self,hero_id:MobyId,hero:&mut Hero,pad:&mut PadState,
        anim:&mut RatchetAnim,class:&MobyAnimClass,layer_present:Option<bool>,counter:u32)->Result<TeleportStep> {
        let request=match self.pending_1330 {
            Some(Pending1330::Teleport(r))=>r,
            Some(Pending1330::TeleportService {request,..})=>request,
            _=>return invalid("GC no pending class1330 teleport"),
        };
        let Some(m)=self.objects.get(hero_id).filter(|m|m.o_class==0 && m.state<0x80) else {
            return invalid("GC teleport requires actual live hero owner");
        };
        let _=m;
        let progress=self.teleport_progress.get_or_insert(TeleportProgress {request,hero:hero_id,next:TeleportStep::Pose});
        if progress.request!=request || progress.hero!=hero_id {return invalid("GC resumed teleport owner or request changed");}
        loop {
            let next=self.teleport_progress.as_ref().unwrap().next;
            self.pending_1330=Some(Pending1330::TeleportService {request,source:match next {
                TeleportStep::Pose=>0x2ef6d8,TeleportStep::Motion=>0x2b2cd8,
                TeleportStep::Trail0|TeleportStep::Trail1=>0x324140,TeleportStep::Flags=>0x2b9ef8,
                TeleportStep::PutAway=>0x2af848,TeleportStep::Modifiers=>0x2a53b8,TeleportStep::State=>0x2c4560,
                TeleportStep::Animation=>0x2d2760,TeleportStep::GroupFlag=>0x2b9f50,
                TeleportStep::Probe=>0x2b4a08,TeleportStep::Snap=>0x2b9f7c,TeleportStep::Actor0=>0x3e0b28,
                TeleportStep::Actor1=>0x403d98,TeleportStep::Camera=>0x2da160,
                TeleportStep::Lighting=>0x313008,TeleportStep::Done=>0x2b9ffc,
            }});
            let following=match next {
                TeleportStep::Pose=>{
                    // Existing GC native-first Euler kernel (not bit-exact VU
                    // validation). +c0 is THREE rows, not a sphere/grid update.
                    hero.pos=request.position[..3].try_into().unwrap();
                    hero.teleport.position_w=Some(request.position[3].to_bits());
                    self.objects.get_mut(hero_id).unwrap().position=request.position;
                    hero.rot=request.euler[..3].try_into().unwrap();
                    hero.teleport.euler_w=request.euler[3].to_bits();
                    hero.rows=crate::hero::physics::euler_rows(hero.rot);
                    let m=self.objects.get_mut(hero_id).unwrap();
                    for k in 0..3 {m.rows[k]=[hero.rows[k][0],hero.rows[k][1],hero.rows[k][2],0.];}
                    TeleportStep::Motion
                },
                TeleportStep::Motion=>{hero.reset_teleport_motion(pad);TeleportStep::Trail0},
                TeleportStep::Trail0=>{self.end_tracked_objects(&mut hero.teleport.trails[0],counter)?;TeleportStep::Trail1},
                TeleportStep::Trail1=>{self.end_tracked_objects(&mut hero.teleport.trails[1],counter)?;TeleportStep::Flags},
                TeleportStep::Flags=>{
                    hero.air_ticks=1;hero.teleport.source_18a1c0=0;
                    hero.teleport.source_18a1c4&=!3;hero.teleport.byte_18c30f=0;
                    TeleportStep::PutAway
                },
                TeleportStep::PutAway=>{
                    if hero.putaway.byte_22b8!=0 && layer_present.is_none() {return Ok(next);}
                    hero.put_away_teleport(&mut anim.ctl(class),layer_present.unwrap_or(false));
                    // The single actual moby must see SetAnim's writeback.
                    self.objects.get_mut(hero_id).unwrap().anim=anim.state;
                    TeleportStep::Modifiers
                },
                TeleportStep::Modifiers=>{
                    let Some(table)=self.teleport_modifiers.as_mut() else {return Ok(next);};
                    table.reset(&mut self.objects)?;TeleportStep::State
                },
                TeleportStep::State=>{
                    if request.state!=-1 {
                        if request.state!=0 {return Ok(next);}
                        let mut progress=self.teleport_state_progress.take().unwrap_or_default();
                        let result=progress.resume(hero,&mut anim.ctl(class),
                            &mut StateServices {world:self,hero:hero_id,counter},layer_present);
                        self.teleport_state_progress=Some(progress);
                        self.objects.get_mut(hero_id).unwrap().anim=anim.state;
                        match result? {
                            crate::hero::teleport_state::StateZeroResult::Pending(source)=>{
                                self.pending_1330=Some(Pending1330::TeleportService {request,source});return Ok(next);
                            },
                            crate::hero::teleport_state::StateZeroResult::Rejected |
                            crate::hero::teleport_state::StateZeroResult::Complete=>{},
                        }
                    }
                    TeleportStep::Animation
                },
                TeleportStep::Animation=>{
                    let m=self.objects.get(hero_id).unwrap();let (seq,frame)=(m.anim.seq_b,m.anim.frame_b);
                    let sound=crate::hero::teleport::reset_teleport_animation(anim,class,seq,frame)?;
                    let m=self.objects.get_mut(hero_id).unwrap();m.anim=anim.state;m.b7c=sound;
                    TeleportStep::GroupFlag
                },
                TeleportStep::GroupFlag=>{
                    let Some(word)=self.teleport_globals.as_mut().and_then(|g|g.word_1b9000.as_mut()) else {return Ok(next);};
                    *word=(hero.group==0x11) as i32;TeleportStep::Probe
                },
                // A caller without bound collision stops here. The collision
                // entry point executes the recovered branch and retains any
                // unsupported water/owner/gravity service as a blocker.
                TeleportStep::Probe=>return Ok(next),
                TeleportStep::Snap=>{
                    //2b9f7c..2b9fac, after the complete supported probe and
                    //before either optional actor service. The source copies
                    //the global position only; it does not repeat the earlier
                    //moby-position/grid write here. Preserve unknown queryW.
                    use rc_game::hero::physics::{dist3,from_f32x3};
                    if dist3(from_f32x3(hero.pos),from_f32x3(hero.ground_point))<rc_game::ps2v::Pf::ONE {
                        hero.pos=hero.ground_point;hero.teleport.position_w=hero.original_ground.point_w;
                    }
                    TeleportStep::Actor0
                },
                TeleportStep::Actor0=>{
                    let Some(bindings)=self.teleport_globals.as_ref().and_then(|g|g.actors_3e0b28) else {return Ok(next);};
                    bindings.service_3e0b28(&mut self.objects,0,0)?;TeleportStep::Actor1
                },
                TeleportStep::Actor1=>{
                    let Some(bindings)=self.teleport_globals.as_ref().and_then(|g|g.actors_403d98) else {return Ok(next);};
                    bindings.service_403d98(&mut self.objects,0,0)?;
                    if request.reset_camera {TeleportStep::Camera}else{TeleportStep::Lighting}
                },
                TeleportStep::Camera|TeleportStep::Lighting|TeleportStep::Done=>return Ok(next),
            };
            self.teleport_progress.as_mut().unwrap().next=following;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tracked_default_is_empty_native_fixture_not_original_new_game() {
        let t=TrackedObjects::default();assert_eq!((t.count,t.active),(0,0));
        assert!(t.slots.iter().all(Option::is_none));
    }
}
