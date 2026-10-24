//! Original GC static spawn decisions (L00 0x3025e0..0x30291c).
//! Save/mission inputs are explicit. This never defaults an absent save input
//! to zero or calls RAC1's title-specific spawn tests.
use rc_formats::{buf::{invalid,Result},rac2::gameplay::Rac2MobyInstance};
use std::collections::BTreeMap;

#[derive(Clone,Debug)]
pub struct SpawnState {
    /// This level's 16 packed mission progress bytes (0x1ab2e8 + level*16).
    pub progress:Vec<u8>,
    /// Results of 0x2e4450 when the selected two-bit progress is zero.
    /// The original helper depends on difficulty statistics; it is not zero.
    pub fallback_progress:BTreeMap<i32,i32>,
    /// 0x1b1c60[mission]; retain signed indices (some original records use -1).
    pub mission_status:BTreeMap<i32,u8>,
    /// 0x22e1c4[id], visit-local exclusion bytes.
    pub disabled:Vec<u8>,
    /// Packed nibbles at 0x1ab4a8 + current_level*0x400.
    pub saved:Vec<u8>,
    /// Same bank for 0x1a79f8, required only by flag8.
    pub reload_saved:Option<Vec<u8>>,
    /// Loader's second argument (stack+0), NOT inferred from the pad/camera.
    pub reload:bool,
    /// Native equivalents of 0x1c54c0 / 0x1c5400 accumulators.
    pub omitted:BTreeMap<(i32,i32),(i32,i32)>,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Decision {pub spawn:bool,pub b1:u8,pub b4:i16,pub b6:i16}

fn at(bytes:&[u8],index:i32)->Result<u8> {
    let Some(v)=usize::try_from(index).ok().and_then(|i|bytes.get(i)) else {
        return invalid("GC spawn decision requires a present state byte");
    };Ok(*v)
}
fn nibble(bytes:&[u8],id:i32)->Result<u8> {Ok((at(bytes,id>>1)? >> ((id&1)*4))&15)}
fn clear_low(bytes:&mut [u8],id:i32)->Result<()> {
    at(bytes,id>>1)?;bytes[(id>>1) as usize]&=if id&1!=0 {0x8f}else{0xf8};Ok(())
}

impl SpawnState {
    pub fn decide(&mut self,instance:&Rac2MobyInstance)->Result<Decision> {
        let f=instance.spawn_fields();let mut reject=false;let (mut b4,mut b6)=(f.b4,f.b6);
        if f.mission>=0 && f.threshold>0 {
            let progress=((at(&self.progress,f.mission)? as u32) >> ((f.selector.wrapping_mul(2) as u32)&31))&3;
            let progress=if progress!=0 {progress as i32}else {
                *self.fallback_progress.get(&f.mission).ok_or_else(||rc_formats::FormatError::Invalid("GC spawn mission fallback is unavailable".into()))?
            };
            if matches!(instance.base.o_class,0x1f5|0x200) {reject=f.threshold<progress;}
            else if progress<f.threshold {
                reject=true;
                let entry=self.omitted.entry((f.mission,f.selector)).or_default();
                entry.0=entry.0.wrapping_add(b4);entry.1=entry.1.wrapping_add(b6);
            }
        }
        if !reject && f.flags!=0 {
            if at(&self.disabled,f.id)?!=0 {reject=true;}
            else if f.flags&3!=0 {
                let status=*self.mission_status.get(&f.mission).ok_or_else(||rc_formats::FormatError::Invalid("GC spawn mission-status byte is unavailable".into()))?;
                if status&0x80!=0 {
                    if f.flags&2==0 {reject=true;}
                    else {
                        if nibble(&self.saved,f.id)?&8==0 {
                            let i=(f.id>>1) as usize;
                            self.saved[i]|=if f.id&1!=0 {0x80}else{8};
                            clear_low(&mut self.saved,f.id)?;
                        }
                        b4=f.alternate_b4;b6=f.alternate_b6;
                    }
                } else {reject=((f.flags^1)&1)!=0;}
            } else if f.flags&8!=0 {
                let bytes=self.reload_saved.as_mut().ok_or_else(||rc_formats::FormatError::Invalid("GC flag8 spawn requires reload-level save bank".into()))?;
                if self.reload {clear_low(bytes,f.id)?;}
                else {reject=nibble(bytes,f.id)?&7!=0;}
            } else if f.flags&12==4 {reject=nibble(&self.saved,f.id)?&7!=0;}
        }
        Ok(Decision {spawn:!reject,b1:(f.selector as u8&15)|if f.flags==0 {0x80}else{0},b4:b4 as i16,b6:b6 as i16})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_formats::{buf::Buf,gameplay::MobyInstance};
    fn state()->SpawnState {SpawnState {progress:vec![0;16],fallback_progress:[(0,3)].into(),
        mission_status:[(0,0)].into(),disabled:vec![0;16],saved:vec![0;8],reload_saved:Some(vec![0;8]),
        reload:false,omitted:Default::default()}}
    fn instance(flags:i32,id:i32)->Rac2MobyInstance {
        let mut base:MobyInstance=Buf(&[0;0x78]).pod(0,"test instance").unwrap();
        base.spawn_id=flags;base.o_class=0x1f4;base.unknown_10=12;
        Rac2MobyInstance {base,extra:[id,4,9,7]}
    }
    #[test]
    fn gc_flag2_uses_alternates_and_changes_only_its_own_nibble() {
        let mut s=state();s.mission_status.insert(0,128);s.saved[0]=0x65;
        let d=s.decide(&instance(2,1)).unwrap();assert!(d.spawn);assert_eq!((d.b4,d.b6),(9,12));
        assert_eq!(s.saved[0],0x85);
        // Already visited: the original does not clear low bits again.
        s.saved[0]=0xd5;s.decide(&instance(2,1)).unwrap();assert_eq!(s.saved[0],0xd5);
        assert!(!s.decide(&instance(1,0)).unwrap().spawn);
    }
    #[test]
    fn gc_threshold_crate_exception_and_omitted_accumulators_are_ordered() {
        let mut s=state();s.progress[0]=1;
        let mut i=instance(0,0);i.base.unknown_14=2;
        assert!(!s.decide(&i).unwrap().spawn);assert_eq!(s.omitted[&(0,0)],(4,7));
        i.base.o_class=0x1f5;assert!(s.decide(&i).unwrap().spawn);
        s.progress[0]=3;assert!(!s.decide(&i).unwrap().spawn);
        assert_eq!(s.omitted[&(0,0)],(4,7));
    }
    #[test]
    fn gc_flag8_reload_bank_and_flag4_current_bank_differ() {
        let mut s=state();s.saved[0]=3;s.reload_saved.as_mut().unwrap()[0]=0x95;
        assert!(!s.decide(&instance(8,0)).unwrap().spawn);
        s.reload=true;assert!(s.decide(&instance(8,0)).unwrap().spawn);
        assert_eq!(s.reload_saved.as_ref().unwrap()[0],0x90);assert_eq!(s.saved[0],3);
        assert!(!s.decide(&instance(4,0)).unwrap().spawn);
        assert!(s.decide(&instance(0,0)).unwrap().spawn);
    }
    #[test]
    fn gc_missing_state_is_an_error_and_always_spawn_never_reads_it() {
        let mut s=state();let mut i=instance(1,0);s.mission_status.clear();assert!(s.decide(&i).is_err());
        i.base.spawn_id=0;s.disabled.clear();assert_eq!(s.decide(&i).unwrap().b1,128);
        i.base.spawn_flags=16;i.base.unknown_14=1;s.progress[0]=255;
        // EE variable shift uses low five bits, so selector16 selects bit0.
        assert!(s.decide(&i).unwrap().spawn);
    }
}
