//! Recovered motion, put-away and key-reset services inside HeroTeleport2b9e68.
//! These services alone do not complete teleport, trail teardown or class1330.
//! Audit counterpart: RAC1 Hero::teleport; its broader/different reset cannot
//! replace this original GC write set. Shared PadState storage is reused.
use super::Hero;
use rc_game::pad::PadState;
use rc_game::hero::anim::{AnimCtl,RatchetAnim};
use rc_formats::{buf::{invalid,Result},moby_anim::MobyAnimClass};

/// Source fields used by2af848. The pose-layer node itself remains owned by
/// the existing animation system; its actual presence is supplied by caller.
/// These values are not evidence of the original New Game initialization.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct PutAwayStorage {
    pub byte_22b8:u8,
    pub byte_22ba:u8,
    pub word_24b0:i32,
    pub word_ea4:i32,
}

/// Original2d2760's key reset, using the existing decoded animation bank.
///2a4db0 copies/decompresses both keys into fixed scratch buffers; native
/// evaluation already reads those same immutable decoded keys from `class`.
/// Unlike generic `hard_cut`, this keeps Ratchet's playback rate, curve,
/// flags, loop state and snapshot storage unchanged and clears t. The return
/// value must be written to the actual hero moby's loop-sound byte+6c.
pub fn reset_teleport_animation(anim:&mut RatchetAnim,class:&MobyAnimClass,seq:u8,frame:u8)->Result<u8> {
    let Some(q)=class.sequence(seq) else {return invalid("GC2d2760 sequence absent");};
    let last=q.header.frame_count as i32-1;
    let next=frame.wrapping_add(1);
    let next=if last<(next as i32) {last as u8}else{next};
    // The guest dereferences both keys without a guard. Reject invalid native
    // resources before writes rather than fabricate a key or return success.
    if q.frames.get(frame as usize).is_none() || q.frames.get(next as usize).is_none() {
        return invalid("GC2d2760 key absent");
    }
    anim.state.frame_b=next;
    anim.state.frame_a=frame;
    anim.state.t=0.;
    anim.state.seq_b=seq;
    anim.state.seq_a=seq;
    anim.state.trigger_count=q.header.trigger_count;
    Ok(q.header.loop_sound)
}

/// Previously unrepresented source lanes, owned by the existing GC Hero.
/// Unknown blocks keep address names; no gameplay role is invented. XYZ of
/// the five named vectors stays solely in Hero's existing native V3 fields.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct MotionResetStorage {
    pub source_189ec0:[u32;4],
    pub source_189f20:[u32;4],
    /// W lanes: velocity189f10, displacement189f30, effective189f40,
    /// effective-vertical189f50, momentum189f80, in that order.
    pub vector_w:[u32;5],
}

impl Hero {
    /// Complete2af848. `layer_present` is the original+e98 link's actual
    /// non-null status; it is not inferred from the hand-item allocation.
    /// No item is deleted, no RNG is consumed, and the zero-byte early return
    /// leaves even22ba unchanged. Animation uses the shared native player.
    pub fn put_away_teleport(&mut self,anim:&mut dyn AnimCtl,layer_present:bool) {
        if self.putaway.byte_22b8==0 {return;}
        if layer_present {self.putaway.word_ea4=1;}
        else if self.state==0 && anim.view().seq_b as i32==self.putaway.word_24b0 {
            self.set_anim(anim,-2.,self.idle_seq(),0);
        }
        self.putaway.byte_22ba=0;
        self.putaway.byte_22b8=0;
    }
    /// Complete2b2cd8/2b3ae0/2eee38/330228 write set, in source order.
    /// No RNG, input button clear, horizontal-effective reset, or invented
    /// state transition occurs here. Callers still owe the rest of2b9e68.
    pub fn reset_teleport_motion(&mut self,pad:&mut PadState) {
        self.disp=[0.;3];self.motion_reset.vector_w[1]=0;
        //2b3ae0 writes speed first, target speed in the return delay slot.
        self.speed=0.;self.target_speed=0.;
        self.motion_reset.source_189f20=[0;4];
        self.vel=[0.;3];self.motion_reset.vector_w[0]=0;
        self.disp=[0.;3];self.motion_reset.vector_w[1]=0;
        self.eff=[0.;3];self.motion_reset.vector_w[2]=0;
        self.eff_v=[0.;3];self.motion_reset.vector_w[3]=0;
        self.momentum=[0.;3];self.motion_reset.vector_w[4]=0;
        self.motion_reset.source_189ec0=[0;4];
        self.eff_len=0.;self.yaw_vel=0.;self.fwd_speed=0.;
        //330228 resets history metadata only. PadState::clear has different
        // semantics, and clearing samples/buttons would lose authentic input.
        pad.hist_count=0;pad.hist_index=0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::hero::anim::RecordingAnim;
    use rc_game::ps2v::Pf;
    use rc_formats::moby_anim::{MobySequence,MobySequenceHeader,MobyFrame,MobyFrameHeader};
    #[test]
    fn putaway_executes_original_guards_and_preserves_layer_ownership() {
        let mut h=Hero::new();let mut a=RecordingAnim::default();
        h.putaway=PutAwayStorage {byte_22b8:0,byte_22ba:9,word_24b0:0,word_ea4:7};
        h.put_away_teleport(&mut a,true);
        assert_eq!((h.putaway.byte_22ba,h.putaway.word_ea4),(9,7));assert!(a.calls.is_empty());
        h.putaway.byte_22b8=2;h.put_away_teleport(&mut a,true);
        assert_eq!(h.putaway,PutAwayStorage {word_ea4:1,..Default::default()});assert!(a.calls.is_empty());
        h.putaway.byte_22b8=1;h.putaway.word_24b0=-1;h.put_away_teleport(&mut a,false);
        assert!(a.calls.is_empty(),"full signed word comparison, not truncated byte");
        h.putaway.byte_22b8=1;h.putaway.word_24b0=0;h.state=2;h.put_away_teleport(&mut a,false);
        assert!(a.calls.is_empty());
        h.putaway.byte_22b8=1;h.state=0;h.hp=1;h.put_away_teleport(&mut a,false);
        assert_eq!(a.calls,vec![(rc_game::ps2v::Pf::f(-2.),0x51,0)]);
        assert_eq!(h.putaway.byte_22b8,0);assert_eq!(h.putaway.byte_22ba,0);
        assert_eq!(h.putaway.word_ea4,1,"no layer fade clear is in original2af848");
    }
    #[test]
    fn animation_reset_keeps_global_rate_curve_loop_and_unwritten_fields() {
        let f=MobyFrame {header:MobyFrameHeader {rate:0.3,..Default::default()},quats:vec![],scales:vec![],trans:vec![],payload:vec![]};
        let q=MobySequence {header:MobySequenceHeader {frame_count:3,loop_sound:42,trigger_count:7,..Default::default()},frames:vec![f;3],triggers:vec![]};
        let c=MobyAnimClass {joint_count:0,skeleton:vec![],rest:vec![],parent_word:vec![],sequences:vec![Some(q)]};
        let mut a=RatchetAnim::new(&c);a.rate=Pf::f(0.125);a.curve=2;a.curve_pos=8;a.flags=3;
        a.loop_range=Some((1,2));a.exit=true;a.state.flags=3;a.state.speed=0.4;a.state.rate=0.2;a.state.t=0.9;
        assert_eq!(reset_teleport_animation(&mut a,&c,0,2).unwrap(),42);
        assert_eq!((a.state.frame_a,a.state.frame_b,a.state.seq_a,a.state.seq_b,a.state.t,a.state.trigger_count),(2,2,0,0,0.,7));
        assert_eq!((a.rate,a.curve,a.curve_pos,a.flags,a.loop_range,a.exit),(Pf::f(0.125),2,8,3,Some((1,2)),true));
        assert_eq!((a.state.flags,a.state.speed,a.state.rate),(3,0.4,0.2));
        let before=a.state;assert!(reset_teleport_animation(&mut a,&c,0,255).is_err());assert_eq!(a.state,before);
        assert!(reset_teleport_animation(&mut a,&c,1,0).is_err());assert_eq!(a.state,before);
        reset_teleport_animation(&mut a,&c,0,0).unwrap();assert_eq!((a.state.frame_a,a.state.frame_b),(0,1));
    }
    #[test]
    fn idle_selector_preserves_gc_guard_priority_and_independent_surface_byte() {
        let mut h=Hero::new();h.surface=-1;
        assert_eq!(h.idle_seq(),0,"signed semantic surface id is not byte1492");
        h.surface_flag_1492=0xff;assert_eq!(h.idle_seq(),0x67);
        h.hp=1;assert_eq!(h.idle_seq(),0x51);
        h.mode=1;assert_eq!(h.idle_seq(),0);
        h.mode=2;assert_eq!(h.idle_seq(),0x51,"GC only excludes body1, unlike RAC1's any nonzero guard");
    }
    #[test]
    fn reset_preserves_unwritten_motion_and_input_fields() {
        let mut h=Hero::spawn([1.,2.,3.],0.7);
        h.vel=[1.;3];h.disp=[2.;3];h.eff=[3.;3];h.eff_v=[4.;3];h.momentum=[5.;3];
        h.motion_reset=MotionResetStorage {source_189ec0:[u32::MAX;4],source_189f20:[42;4],vector_w:[7;5]};
        h.speed=6.;h.target_speed=7.;h.eff_len=8.;h.yaw_vel=9.;h.fwd_speed=10.;
        h.eff_h=[11.;3];h.eff_len_xy=12.;h.slope_ratio=13.;h.yaw_residual=14.;h.land_lag=15;
        let mut pad=PadState::default();pad.hist_count=30;pad.hist_index=29;
        pad.raw=0xffff;pad.held=0x4000;pad.hist_len[0]=rc_game::ps2v::Pf::f(0.75);
        let before=h.clone();h.reset_teleport_motion(&mut pad);
        for v in [h.vel,h.disp,h.eff,h.eff_v,h.momentum] {assert_eq!(v.map(f32::to_bits),[0;3]);}
        assert_eq!(h.motion_reset,MotionResetStorage::default());
        assert_eq!([h.speed,h.target_speed,h.eff_len,h.yaw_vel,h.fwd_speed].map(f32::to_bits),[0;5]);
        assert_eq!((h.eff_h,h.eff_len_xy,h.slope_ratio,h.yaw_residual,h.land_lag),
            (before.eff_h,before.eff_len_xy,before.slope_ratio,before.yaw_residual,before.land_lag));
        assert_eq!((h.pos,h.rot,h.state,h.hp),(before.pos,before.rot,before.state,before.hp));
        assert_eq!((pad.hist_count,pad.hist_index,pad.raw,pad.held),(0,0,0xffff,0x4000));
        assert_eq!(pad.hist_len[0],rc_game::ps2v::Pf::f(0.75));
    }
}
