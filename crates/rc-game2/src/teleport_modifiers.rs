//! GC2a53b8/2a4fe8/3124f8 on the existing native object's modifier lists.
//! RAC1 manip::detach has the same unlink/clear semantics, but assumes pvar
//! ownership. This table is GLOBAL25bd90, not any moby's pvars. Keep one table
//! and reuse Moby.joint_mods/keys with explicit GC global-node identities.
use crate::objects::Objects;
use rc_formats::buf::{invalid,Buf,Result};
use rc_game::moby_runtime::MobyId;
pub const COUNT:usize=40;
pub const STRIDE:usize=0xb0;
/// Namespace outside native object ids (World ids do not reach0xffff),
/// distinct from existing scene head/noggin/arm keys. Never a guest pointer.
pub fn node_key(index:usize)->u32 {assert!(index<COUNT);0xffff_1000+index as u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ModifierBindings {
    pub hero:Option<MobyId>,
    pub source_18b134:Option<MobyId>,
    pub source_18b040:Option<MobyId>,
}
impl ModifierBindings {
    pub fn resolve(self,selector:i16)->Option<MobyId> {
        match selector {0|2|3|4|5=>self.hero,1=>self.source_18b134,6=>self.source_18b040,_=>None}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ModifierTable {pub records:[[u8;STRIDE];COUNT],pub bindings:ModifierBindings}
impl ModifierTable {
    /// Explicit runtime snapshot or labelled fixture input, not an inferred
    /// New Game state. Original initialized data alone is not runtime proof.
    pub fn read(bytes:&[u8],bindings:ModifierBindings)->Result<Self> {
        let b=Buf(bytes).sub(0,COUNT*STRIDE,"GC global modifier records")?;
        let mut records=[[0;STRIDE];COUNT];
        for (k,r) in records.iter_mut().enumerate() {r.copy_from_slice(b.sub(k*STRIDE,STRIDE,"GC modifier record")?.bytes());}
        Ok(Self {records,bindings})
    }
    pub fn reset(&mut self,objects:&mut Objects)->Result<()> {
        // Validate native identities before mutating the original ordered list.
        for r in &self.records {
            if r[1]==0 {continue;}
            if let Some(id)=self.bindings.resolve(i16::from_le_bytes([r[0xa2],r[0xa3]])) {
                let Some(m)=objects.get(id) else {return invalid("GC modifier target binding absent");};
                if m.joint_mod_keys.len()!=m.joint_mods.len() {return invalid("GC modifier key/value list mismatch");}
            }
        }
        for (k,r) in self.records.iter_mut().enumerate() {
            if r[1]!=0 {
                if let Some(id)=self.bindings.resolve(i16::from_le_bytes([r[0xa2],r[0xa3]])) {
                    let m=objects.get_mut(id).unwrap();
                    if let Some(i)=m.joint_mod_keys.iter().position(|&key|key==node_key(k)) {
                        m.joint_mod_keys.remove(i);m.joint_mods.remove(i);
                    }
                    // Source clears even when the node wasn't in that list.
                    r[..0x40].fill(0);
                }
            }
            // Keep velocities, current translation, joint/kind, spring data,
            // and a null-target node including its attached byte untouched.
            r[0x40..0x50].fill(0);r[0x60..0x70].fill(0);r[0x90..0xa0].fill(0);
            r[0xac..0xb0].copy_from_slice(&1f32.to_le_bytes());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::moby_runtime::Moby;
    use rc_formats::moby_anim::JointModifier;
    #[test]
    fn actual_owned_lists_preserve_unrelated_order_and_null_target_nodes() {
        let bindings=ModifierBindings {hero:Some(0),source_18b134:Some(1),source_18b040:None};
        let mut t=ModifierTable::read(&vec![0xa5;COUNT*STRIDE],bindings).unwrap();
        for r in &mut t.records {r[1]=0;}
        for (k,selector) in [(0,0i16),(1,1),(2,6),(3,-1),(4,2)] {
            t.records[k][1]=1;t.records[k][0xa2..0xa4].copy_from_slice(&selector.to_le_bytes());
        }
        let mut a=Moby::init_instance(0,0,None);let mut b=Moby::init_instance(1,1,None);
        a.joint_mod_keys=vec![123,node_key(0),456];a.joint_mods=vec![JointModifier::compose(0);3];
        b.joint_mod_keys=vec![node_key(1)];b.joint_mods=vec![JointModifier::compose(1)];
        let mut o=Objects::new(vec![a,b],0);let before=t.clone();t.reset(&mut o).unwrap();
        assert_eq!(o.get(0).unwrap().joint_mod_keys,[123,456]);assert!(o.get(1).unwrap().joint_mods.is_empty());
        for k in [0,1,4] {assert_eq!(t.records[k][..0x40],[0;0x40]);}
        for k in [2,3] {assert_eq!(t.records[k][..0x40],before.records[k][..0x40]);}
        for (k,r) in t.records.iter().enumerate() {
            for range in [0x40..0x50,0x60..0x70,0x90..0xa0] {assert!(r[range].iter().all(|&v|v==0));}
            assert_eq!(&r[0x50..0x60],&before.records[k][0x50..0x60]);
            assert_eq!(&r[0x70..0x90],&before.records[k][0x70..0x90]);
            assert_eq!(&r[0xa0..0xac],&before.records[k][0xa0..0xac]);
            assert_eq!(&r[0xac..],&1f32.to_le_bytes());
        }
    }
    #[test]
    fn selector_and_bounds_contract() {
        let b=ModifierBindings {hero:Some(10),source_18b134:Some(11),source_18b040:Some(12)};
        for k in [0,2,3,4,5] {assert_eq!(b.resolve(k),Some(10));}
        assert_eq!(b.resolve(1),Some(11));assert_eq!(b.resolve(6),Some(12));assert_eq!(b.resolve(-1),None);
        assert!(ModifierTable::read(&vec![0;COUNT*STRIDE-1],b).is_err());
    }
}
