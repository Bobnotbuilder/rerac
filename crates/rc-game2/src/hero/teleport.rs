//! Required motion service2b2cd8 inside original HeroTeleport2b9e68.
//! This service alone does not teleport, reset items/trails, or complete1330.
//! Audit counterpart: RAC1 Hero::teleport; its broader/different reset cannot
//! replace this original GC write set. Shared PadState storage is reused.
use super::Hero;
use rc_game::pad::PadState;

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
