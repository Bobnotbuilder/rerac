//! GC wrench animation binding through existing native generic moby animation.
//! This is the animation portion of the item contract, not a class initializer,
//! dynamic allocator, ownership state, hand attachment or damage implementation.
use rc_formats::{buf::{invalid,Result},moby_anim::{self,AnimState,MobyAnimClass,MobyFrame,Rows}};
use crate::melee::ComboEntry;

pub struct WrenchAnimation {
    pub state:AnimState,
    snapshot:Option<MobyFrame>,
}

impl WrenchAnimation {
    /// 0x2afe2c..0x2afe50 requests sequence1/frame0/blend1 once class creation
    /// succeeds. The caller must separately prove that class lifecycle.
    pub fn initial_animation(class:&MobyAnimClass)->Result<Self> {
        let mut out=Self {state:AnimState::spawn(class),snapshot:None};
        out.set(class,1,0,1)?;Ok(out)
    }

    pub fn combo(&mut self,class:&MobyAnimClass,entry:ComboEntry)->Result<()> {
        self.set(class,entry.wrench_sequence,entry.start_frame,entry.blend)
    }

    fn set(&mut self,class:&MobyAnimClass,sequence:u8,frame:i32,ticks:i32)->Result<()> {
        let Some(seq)=class.sequence(sequence) else {return invalid("GC wrench required sequence is absent");};
        if frame<0 || frame as usize>=seq.frames.len() {return invalid("GC wrench required frame is absent");}
        if !moby_anim::set_sequence(&mut self.state,class,sequence,frame,ticks,&mut self.snapshot) {return invalid("GC wrench native sequence change failed");}
        Ok(())
    }

    pub fn advance(&mut self,class:&MobyAnimClass) {moby_anim::advance(&mut self.state,class);}
    pub fn pose(&self,class:&MobyAnimClass)->Vec<Rows> {moby_anim::evaluate_with_snapshot(class,&self.state,self.snapshot.as_ref())}
}
