//! GC direct hit delivery (Aranos 0x31d4c8), independently typed from RAC1.
//! Target slot is GC moby+0xa8; record target is +0x38 (RAC1 +0x34).
//! Native records do not contain or execute original pointers. This service
//! queues a hit; receiver damage, death, rewards and effects remain callbacks.
use rc_game::{hero::physics::{V4,len3},ps2v::Pf};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MobyId(pub u32);

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct HitSlot(u8);
impl Default for HitSlot {fn default()->Self {Self(0xff)}}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct HitRecord {
    pub point:V4,
    pub direction:V4,
    pub attacker:Option<MobyId>,
    pub flags:u32,
    /// +0x28: not overwritten by direct delivery; other query producers own it.
    pub tags:u32,
    pub damage:Pf,
    /// +0x30: original length(direction) > 0x38d1b717.
    pub has_direction:bool,
    /// +0x34: original damage copied here as well as +0x2c.
    pub initial_damage:Pf,
    pub target:Option<MobyId>,
    pub primitive:i32,
}

impl Default for HitRecord {
    fn default()->Self {Self {point:[Pf::ZERO;4],direction:[Pf::ZERO;4],attacker:None,flags:0,tags:0,
        damage:Pf::ZERO,has_direction:false,initial_damage:Pf::ZERO,target:None,primitive:0}}
}

pub struct HitLog {records:[HitRecord;64],next:u8}
impl Default for HitLog {fn default()->Self {Self {records:[HitRecord::default();64],next:0}}}

impl HitLog {
    pub fn current(&self,target:MobyId,slot:HitSlot)->Option<&HitRecord> {
        self.records.get(slot.0 as usize).filter(|r|r.target==Some(target))
    }

    /// GC 0x31dce8: filter the target's current hit. A rejected mask clears
    /// the target slot unless a2 requested retention; an absent/stale record
    /// leaves it untouched. A match is not consumed by this lookup.
    pub fn find(&self,target:MobyId,slot:&mut HitSlot,mask:u32,keep_rejected:bool)->Option<&HitRecord> {
        let record=self.current(target,*slot)?;
        if record.flags&mask!=0 {return Some(record);}
        if !keep_rejected {*slot=HitSlot::default();}
        None
    }

    /// Weaker hits merge flags into the existing record without advancing the
    /// ring. Equal/stronger hits allocate and retain existing flags. Reset the
    /// target's HitSlot on its native lifetime initialization, as 0x311478 does.
    #[allow(clippy::too_many_arguments)]
    pub fn deliver(&mut self,target:MobyId,slot:&mut HitSlot,damage:Pf,attacker:Option<MobyId>,
        flags:u32,point:V4,direction:V4) {
        let old=self.current(target,*slot).copied();
        let flags=flags|old.map_or(0,|r|r.flags);
        if old.is_some_and(|r|damage<r.damage) {
            self.records[slot.0 as usize].flags=flags;return;
        }
        let index=self.next as usize;
        let tags=self.records[index].tags;
        self.records[index]=HitRecord {point,direction,attacker,flags,tags,damage,
            has_direction:len3(direction)>Pf::b(0x38d1b717),initial_damage:damage,target:Some(target),primitive:0};
        *slot=HitSlot(self.next);self.next=(self.next+1)&0x3f;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weaker_hit_preserves_damage_and_merges_flags_without_consuming_ring() {
        let mut log=HitLog::default();let mut slot=HitSlot::default();let id=MobyId(4);
        let z=[Pf::ZERO;4];let d=[Pf::ONE,Pf::ZERO,Pf::ZERO,Pf::ZERO];
        log.deliver(id,&mut slot,Pf::f(3.),Some(MobyId(1)),0x10000,z,d);
        log.deliver(id,&mut slot,Pf::ONE,None,0x20000,d,z);
        let r=log.current(id,slot).unwrap();assert_eq!(r.damage,Pf::f(3.));assert_eq!(r.flags,0x30000);
        assert_eq!(r.attacker,Some(MobyId(1)));assert_eq!(r.point,z);assert!(r.has_direction);assert_eq!(log.next,1);
        // Equal damage is a fresh record, with flags retained and new fields.
        log.deliver(id,&mut slot,Pf::f(3.),None,0x40000,d,z);
        let r=log.current(id,slot).unwrap();assert_eq!(r.flags,0x70000);assert_eq!(r.point,d);
        assert!(!r.has_direction);assert_eq!(r.initial_damage,Pf::f(3.));assert_eq!(log.next,2);
    }
    #[test]
    fn wrapped_ring_does_not_merge_flags_from_another_target() {
        let mut log=HitLog::default();let mut first=HitSlot::default();let z=[Pf::ZERO;4];
        log.deliver(MobyId(0),&mut first,Pf::f(100.),None,0x40000,z,z);
        for id in 1..=64 {log.deliver(MobyId(id),&mut HitSlot::default(),Pf::ONE,None,0x20000,z,z);}
        assert!(log.current(MobyId(0),first).is_none());
        log.deliver(MobyId(0),&mut first,Pf::ONE,None,0x10000,z,z);
        assert_eq!(log.current(MobyId(0),first).unwrap().flags,0x10000);
    }
    #[test]
    fn receiver_mask_lookup_preserves_or_clears_only_the_rejected_record() {
        let mut log=HitLog::default();let mut slot=HitSlot::default();let id=MobyId(2);let z=[Pf::ZERO;4];
        log.deliver(id,&mut slot,Pf::ONE,None,0x10000,z,z);
        assert!(log.find(id,&mut slot,0x20000,true).is_none());assert!(log.current(id,slot).is_some());
        assert!(log.find(id,&mut slot,0x10000,false).is_some());assert!(log.current(id,slot).is_some());
        let mut stale=slot;assert!(log.find(MobyId(3),&mut stale,0,false).is_none());assert_eq!(stale,slot);
        assert!(log.find(id,&mut slot,0x20000,false).is_none());assert_eq!(slot,HitSlot::default());
    }
}
