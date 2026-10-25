//! Original GC bounded queue services2f4e00/2f4e48/2f4ee8/2f5030.
//! Borrow the actual runtime owner's bytes; do not create a second queue or
//! equate queued dispatch with successful script/progression execution.
use rc_formats::buf::{invalid,Result};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Dispatch {pub kind:i32,pub mode:i32,pub arg2:i32,pub arg3:i32,pub arg4:i32}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DispatchStep {Complete(bool), Pending(Dispatch)}

pub struct Queue<'a> {bytes:&'a mut[u8]}
impl<'a> Queue<'a> {
    pub fn bind(bytes:&'a mut[u8])->Result<Self> {
        if bytes.len()<0x48 {return invalid("GC original queue owner too short");}
        Ok(Self {bytes})
    }
    fn word(&self,o:usize)->i32 {i32::from_le_bytes(self.bytes[o..o+4].try_into().unwrap())}
    fn write(&mut self,o:usize,v:i32) {self.bytes[o..o+4].copy_from_slice(&v.to_le_bytes());}
    fn validate(&self)->Result<()> {
        if !(0..5).contains(&self.word(0x30)) || !(0..5).contains(&self.word(0x34)) || !(0..=5).contains(&self.word(0x38)) {
            return invalid("GC original queue cursor/count invalid");
        } Ok(())
    }
    ///2f4e00/2f4f60: source memset(+8,0xcd,0x28); leaves+0,+4,+3c intact.
    pub fn initialize(&mut self) {
        self.write(0x30,0);self.write(0x34,0);self.write(0x38,0);
        self.bytes[8..0x30].fill(0xcd);self.write(0x44,0);self.write(0x40,1);
    }
    ///2f4ee8: full or dispatch-latched queues reject without writes.
    pub fn append(&mut self,value:i32)->Result<bool> {
        self.validate()?;
        if self.word(0x38)==5 || self.word(0x44)!=0 {return Ok(false);}
        let cursor=self.word(0x30);let p=8+cursor as usize*8;
        self.write(p,value);self.write(p+4,((value as u32).wrapping_sub(0x20)<0x91) as i32);
        let next=cursor+1;self.write(0x38,self.word(0x38)+1);self.write(0x30,if next==5 {0}else{next});Ok(true)
    }
    ///2f4e48: consumed entries become two−1words; last pop clears+44.
    pub fn pop(&mut self)->Result<Option<(i32,i32)>> {
        self.validate()?;
        if self.word(0x38)==0 {return Ok(None);}
        let remaining=self.word(0x38)-1;self.write(0x38,remaining);
        if remaining==0 {self.write(0x44,0);}
        let cursor=self.word(0x34);let p=8+cursor as usize*8;
        let entry=(self.word(p),self.word(p+4));self.write(p,-1);self.write(p+4,-1);
        self.write(0x34,if cursor+1==5 {0}else{cursor+1});Ok(Some(entry))
    }
    ///2f5030 prefix. Known entry kinds return an explicit mandatory32c5f0
    /// call. The host must retain it and obtain a real dispatcher result before
    /// invoking finish_dispatch; producing this token is not script completion.
    pub fn begin_dispatch(&mut self,source_gp_minus_60f0:i32)->Result<DispatchStep> {
        self.validate()?;
        if self.word(0x38)==0 {return Ok(DispatchStep::Complete(false));}
        self.write(0x3c,0);self.write(0x44,1);
        let (value,kind)=self.pop()?.unwrap();
        let mode=if (source_gp_minus_60f0 as u32).wrapping_sub(1)<=1 {2}else{1};self.write(0x40,mode);
        Ok(match kind {
            0=>DispatchStep::Pending(Dispatch {kind:1,mode,arg2:0,arg3:value,arg4:0}),
            1=>DispatchStep::Pending(Dispatch {kind:2,mode,arg2:value,arg3:0,arg4:0}),
            _=>DispatchStep::Complete(true), // original s1 remains0; no dispatcher call
        })
    }
    ///2f5030 resume after the mandatory32c5f0 call actually returned. Negative
    /// resets the queue; only0 yields true. This is dispatch return, not scene end.
    pub fn finish_dispatch(&mut self,handle:i32)->bool {
        if handle<0 {self.initialize();}
        handle==0
    }
    /// Read-only diagnostic view, never a copied runtime owner.
    pub fn bytes(&self)->&[u8] {self.bytes}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(b:&[u8],o:usize)->i32 {rc_formats::buf::Buf(b).i32(o).unwrap()}
    #[test]
    fn original_init_ring_wrap_and_classification() {
        let mut owner=[0x5a;0x48];let mut q=Queue::bind(&mut owner).unwrap();q.initialize();
        assert_eq!(word(q.bytes(),0),0x5a5a5a5a);assert_eq!(word(q.bytes(),0x3c),0x5a5a5a5a);
        assert!(q.bytes()[8..0x30].iter().all(|&b|b==0xcd));
        for v in [0x1f,0x20,0xb0,0xb1,-1] {assert!(q.append(v).unwrap());}
        assert!(!q.append(9).unwrap());
        assert_eq!(q.pop().unwrap(),Some((0x1f,0)));assert!(q.append(0x21).unwrap());
        for entry in [(0x20,1),(0xb0,1),(0xb1,0),(-1,0),(0x21,1)] {assert_eq!(q.pop().unwrap(),Some(entry));}
        assert_eq!(q.pop().unwrap(),None);assert_eq!(word(q.bytes(),0x44),0);
    }
    #[test]
    fn dispatch_order_latch_mode_return_and_failed_reset() {
        let mut owner=[0;0x48];let mut q=Queue::bind(&mut owner).unwrap();q.initialize();
        q.append(0x21).unwrap();q.append(3).unwrap();
        assert_eq!(q.begin_dispatch(2).unwrap(),DispatchStep::Pending(Dispatch {kind:2,mode:2,arg2:0x21,arg3:0,arg4:0}));
        assert!(!q.finish_dispatch(7));
        assert_eq!(word(q.bytes(),0x44),1);assert!(!q.append(5).unwrap());
        assert_eq!(q.begin_dispatch(0).unwrap(),DispatchStep::Pending(Dispatch {kind:1,mode:1,arg2:0,arg3:3,arg4:0}));
        assert!(!q.finish_dispatch(-1));
        assert_eq!(word(q.bytes(),0x38),0);assert_eq!(word(q.bytes(),0x44),0);assert_eq!(word(q.bytes(),0x40),1);
        q.append(0x21).unwrap();assert!(matches!(q.begin_dispatch(1).unwrap(),DispatchStep::Pending(_)));assert!(q.finish_dispatch(0));
        assert_eq!(word(q.bytes(),0x44),0,"last pop clears latch before dispatch");
    }
}
