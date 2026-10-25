//! Original teleport's optional-actor services 0x3e0b28 and 0x403d98.
//! Reuses the sole native Objects owner. Bindings are explicit recovered globals;
//! absence of a binding must never be inferred from an unknown New Game state.
use crate::objects::Objects;
use rc_formats::buf::{invalid,Result};
use rc_game::moby_runtime::MobyId;

/// Two independent pointers, even when a particular snapshot aliases them.
/// The first is the wrapper's actor; the second belongs to its reset callee.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ActorBindings {
    pub actor:Option<MobyId>,
    pub reset_actor:Option<MobyId>,
}

/// Verified Aranos loader-reset source profile; no Default or public raw
/// constructor. This proves the four leaf identities, not whole37f858 or
/// original New Game/save state. World applies them at its loader boundary.
#[derive(Clone,Copy,Debug)]
pub struct AranosLoaderReset { _verified:() }
impl AranosLoaderReset {
    pub fn from_overlay(overlay:&[u8])->Result<Self> {
        let sections=rc_formats::font::parse_overlay_sections(overlay)?;
        for (pc,hash) in [(0x403ec0,"d839d04c936e7acd545f34eee0d6a60b129769d3"),
            (0x404330,"af872012d8848c38a9ef3c790eea212a16609f87"),
            (0x3e0b88,"aa6240fb354dcfb139222b0e24adb68717c53b01"),
            (0x3b9610,"42f25c239ff000b7b238e226a883102cc6694c00")] {
            let bytes=rc_formats::font::read_overlay(&sections,pc,8).ok_or_else(||rc_formats::FormatError::Invalid("GC actor reset leaf absent".into()))?;
            if rc_formats::sha1::hex(&rc_formats::sha1::sha1(bytes))!=hash {return invalid("GC actor loader-reset source differs from Aranos contract");}
        }
        Ok(Self {_verified:()})
    }
}

impl ActorBindings {
    /// 0x3e0b28, including complete leaf 0x3b9578. Globals are GP+13fc
    /// (actor) and GP+0b80 (reset actor); trailing write is owned pvars+20.
    pub fn service_3e0b28(self,objects:&mut Objects,select:i32,value:i32)->Result<()> {
        self.apply(objects,select,value,0x20)
    }
    /// 0x403d98, including complete leaf 0x404298. Globals are GP+1c70
    /// (actor) and GP+1d68 (reset actor); trailing write is owned pvars+60.
    pub fn service_403d98(self,objects:&mut Objects,select:i32,value:i32)->Result<()> {
        self.apply(objects,select,value,0x60)
    }
    fn apply(self,objects:&mut Objects,select:i32,value:i32,offset:usize)->Result<()> {
        // The original null branch does not inspect/call the reset pointer.
        let Some(actor)=self.actor else {return Ok(());};
        let Some(m)=objects.get(actor) else {return invalid("GC teleport optional actor owner absent");};
        if m.state>=0xfd || m.pvars.len()<offset+4 {
            return invalid("GC teleport optional actor pvars or lifetime invalid");
        }
        if let Some(reset)=self.reset_actor {
            let Some(m)=objects.get(reset) else {return invalid("GC teleport reset actor owner absent");};
            if m.state>=0xfd || m.pvars.len()<0x2c {
                return invalid("GC teleport reset actor pvars or lifetime invalid");
            }
        }
        // Source writes SB +20 directly, not SetState: no flags/previous-state
        // bookkeeping. The reset leaf writes its separate owner's pvars next.
        objects.get_mut(actor).unwrap().state=if select==0 {7}else{5};
        if let Some(reset)=self.reset_actor {
            objects.get_mut(reset).unwrap().pvars[0x28..0x2c].copy_from_slice(&1i32.to_le_bytes());
        }
        objects.get_mut(actor).unwrap().pvars[offset..offset+4]
            .copy_from_slice(&(if value == -1 {10}else{value}).to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::moby_runtime::Moby;
    fn objects()->Objects {
        let mut a=Moby::init_instance(0,1,None);a.pvars=vec![0x5a;0x80];a.state=2;
        let mut b=a.clone();b.o_class=2;
        Objects::new(vec![a,b],0)
    }
    fn word(o:&Objects,id:usize,p:usize)->i32 {
        i32::from_le_bytes(o.get(id).unwrap().pvars[p..p+4].try_into().unwrap())
    }
    #[test]
    fn distinct_owners_and_both_original_variants() {
        let mut o=objects();
        ActorBindings {actor:Some(0),reset_actor:Some(1)}.service_3e0b28(&mut o,0,0).unwrap();
        assert_eq!(o.get(0).unwrap().state,7);assert_eq!(word(&o,0,0x20),0);
        assert_eq!(word(&o,1,0x28),1);assert_eq!(word(&o,0,0x28),0x5a5a5a5a);
        assert_eq!(o.get(1).unwrap().state,2);
        ActorBindings {actor:Some(1),reset_actor:Some(0)}.service_403d98(&mut o,-3,-1).unwrap();
        assert_eq!(o.get(1).unwrap().state,5);assert_eq!(word(&o,1,0x60),10);
        assert_eq!(word(&o,0,0x28),1);
    }
    #[test]
    fn aliases_use_same_owner_and_null_branch_ignores_reset() {
        let mut o=objects();
        ActorBindings {actor:Some(0),reset_actor:Some(0)}.service_3e0b28(&mut o,0,42).unwrap();
        assert_eq!(word(&o,0,0x20),42);assert_eq!(word(&o,0,0x28),1);
        ActorBindings {actor:None,reset_actor:Some(usize::MAX)}.service_403d98(&mut o,0,0).unwrap();
    }
    #[test]
    fn invalid_live_binding_fails_before_mutation() {
        let mut o=objects();o.get_mut(1).unwrap().pvars.truncate(0x28);
        assert!(ActorBindings {actor:Some(0),reset_actor:Some(1)}.service_3e0b28(&mut o,0,0).is_err());
        assert_eq!(o.get(0).unwrap().state,2);assert_eq!(word(&o,0,0x20),0x5a5a5a5a);
    }
}
