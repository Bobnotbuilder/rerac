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
    pub position_w:u32,
    pub euler_w:u32,
    pub trails:[TrackedObjects;2],
    pub source_18a1c0:u32,
    pub source_18a1c4:u32,
    pub byte_18c30f:u8,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TeleportStep {Pose,Motion,Trail0,Trail1,Flags,PutAway,Modifiers,State}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TeleportProgress {pub request:TeleportRequest,pub hero:MobyId,pub next:TeleportStep}

impl World {
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
            }});
            let following=match next {
                TeleportStep::Pose=>{
                    // Existing GC native-first Euler kernel (not bit-exact VU
                    // validation). +c0 is THREE rows, not a sphere/grid update.
                    hero.pos=request.position[..3].try_into().unwrap();
                    hero.teleport.position_w=request.position[3].to_bits();
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
                TeleportStep::State=>return Ok(next),
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
