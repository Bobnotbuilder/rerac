//! GC wrench animation binding through existing native generic moby animation.
//! This is the animation portion of the item contract, not a class initializer,
//! dynamic allocator, ownership state, hand attachment or damage implementation.
use rc_formats::{buf::{invalid,Result},moby_anim::{self,AnimState,MobyAnimClass,MobyFrame,Rows}};
use crate::melee::ComboEntry;
use rc_formats::rac2::melee::MeleeTiming;
use rc_game::{hero::physics::{V4,vadd,vsub,vscale},ps2v::Pf};

/// 0x38d638..0x38d698 in the original class-0x47 update. This gates
/// collision work, not a hit or damage result. The full callback must also
/// provide its pvar, hand poses, sweep geometry and original damage dispatch.
pub fn wrench_sweep_window(hero_state:i32,hero_group:i32,blending:bool,frame:f32,row:&MeleeTiming)->bool {
    (hero_group==6 || hero_state==0x2a) && !blending
        && !(frame<row.hit_from as f32 || (row.hit_to as f32)<frame)
}

/// GC 0x31d648: interpolate the two endpoints from previous to current and
/// stop at the first successful line. The wrench passes five samples. Reuse
/// the native shared PS2 vector arithmetic, with sequential scalar addition
/// (the last sample is not forcibly replaced by the current endpoints).
/// The caller supplies the GC query/damage contract; no RAC1 hit log is used.
pub fn sweep_lines<T>(hand:V4,tip:V4,previous_hand:V4,previous_tip:V4,samples:i32,
    mut query:impl FnMut(V4,V4)->Option<T>)->Option<T> {
    if samples<=0 {return None;}
    let step=Pf::ONE/Pf::from_i32(samples);
    let (dh,dt)=(vsub(hand,previous_hand),vsub(tip,previous_tip));
    let mut t=step;
    for _ in 0..samples {
        let a=vadd(vscale(dh,t),previous_hand);
        let b=vadd(vscale(dt,t),previous_tip);
        if let Some(hit)=query(a,b) {return Some(hit);}
        t+=step;
    }
    None
}

#[derive(Clone,Copy,Debug)]
pub struct WrenchAttachment {pub position:[f32;3],pub rows:[moby_anim::V4;3]}

/// Normal attached hand in GC 0x2b0bac..0x2b0cd8. Its world bone matrix is
/// produced by 0x2aa1f0 -> 0x3128f0 with the list read from the GC definition.
/// These matrix and column-normalization operations match the existing native
/// generic helpers. Input is a partial joint-chain pose, never a skinned matrix.
/// Detached flight, rotation history smoothing and alternate body modes are
/// separate paths and must not call this as their complete attachment driver.
pub fn attach_wrench(partial:&Rows,hero_rows:&[moby_anim::V4;3],position:[f32;3],scale:f32)->WrenchAttachment {
    let world=moby_anim::attach_matrix(partial,hero_rows,position,scale);
    let mut rows=[0,1,2].map(|i|world[i].map(f32::to_bits));
    moby_anim::normalise_columns(&mut rows);
    WrenchAttachment {position:[world[3][0],world[3][1],world[3][2]],rows}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sweep_preserves_first_hit_order_and_does_not_test_start_pose() {
        let v=|x|[Pf::f(x),Pf::ZERO,Pf::ZERO,Pf::ONE];
        let mut seen=Vec::new();
        let hit=sweep_lines(v(8.),v(12.),v(0.),v(4.),4,|a,b| {
            seen.push((a[0].to_f32(),b[0].to_f32()));
            (a[0]>=Pf::f(4.)).then_some(37)
        });
        assert_eq!(hit,Some(37));assert_eq!(seen,vec![(2.,6.),(4.,8.)]);
        let mut calls=0;
        assert_eq!(sweep_lines(v(8.),v(12.),v(0.),v(4.),0,|_,_|{calls+=1;Some(1)}),None);
        assert_eq!(calls,0);
    }
}

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
