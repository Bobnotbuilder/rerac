//! Complete GC2b48b0 segment builder used by ground probe2b4a08.
//! Reuses RAC1's existing pure vector/clip kernels (31ff00 =2720f8).
//! Building/querying a line is not the complete ground/owner/liquid/tilt update.
use rc_game::{hero::{clip_to_world,physics::{set_len3,vadd}},ps2v::Pf};

/// Actual original runtime inputs. No unknown global defaults are supplied.
#[derive(Clone,Copy,Debug)]
pub struct Frame {
    pub source_1b8ff0:i32,
    pub source_18a0e0:[f32;4],
    pub moby_up:[f32;4],
}

pub fn segment(position:[f32;4],up:f32,down:f32,use_frame:bool,frame:Frame)->([f32;4],[f32;4]) {
    let pos=position.map(Pf::f);
    let (start,end)=if use_frame {
        let reversed=frame.source_1b8ff0!=0;
        let axis=if reversed {frame.source_18a0e0}else{frame.moby_up}.map(Pf::f);
        let start=vadd(pos,set_len3(axis,Pf::f(if reversed {-up}else{up})));
        let mut end=vadd(pos,set_len3(axis,Pf::f(if reversed {-down}else{down})));
        // Original31ff00 pulls negative XYZ back along the segment, it is
        // not an independent z clamp or a maximum-world-coordinate clamp.
        clip_to_world(start,&mut end);(start,end)
    } else {
        let mut end=pos;end[2]=end[2]+Pf::f(down);
        if end[2]<Pf::ZERO {end[2]=Pf::ZERO;}
        let mut start=pos;start[2]=start[2]+Pf::f(up);(start,end)
    };
    (start.map(Pf::to_f32),end.map(Pf::to_f32))
}

/// Original2e2410 differs from RAC1's sound_class: class3 stays3,
/// negative kinds return−1 (stored asff by the caller), not0.
pub fn sound_class(kind:i32)->i32 {if kind<0 {-1}else{(kind&0x60)>>5}}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gc_sound_class_does_not_use_rac1_remapping() {
        for class in 0..4 {assert_eq!(sound_class(0x1000|(class<<5)),class);}
        assert_eq!(sound_class(-1),-1);
    }
    #[test]
    fn native_probe_counter_write_uses_original_halfword_wrap() {
        let mut h=super::super::Hero::spawn([10.,10.,10.],0.);h.air_ticks=i16::MAX;h.f16c=i16::MAX;
        let coll=rc_formats::collision::Collision::default();let pad=rc_game::pad::PadState::default();
        let env=super::super::physics::Env {coll:&coll,mobys:None,hero_moby:None,pad:&pad,cam_yaw:0.,cam_rows:h.rows,mirror:false};
        h.ground_probe(&env);assert_eq!((h.air_ticks,h.f16c),(i16::MIN,i16::MIN));
    }
    #[test]
    fn normal_and_reversed_frames_use_owned_axis_length_and_preserve_w() {
        // Power-of-two axis lengths avoid confusing the shared PS2 model
        // rounding of normalization with frame-selection semantics.
        let p=[10.,20.,30.,0.01];let frame=Frame {source_1b8ff0:0,source_18a0e0:[0.,-4.,0.,123.],moby_up:[0.,2.,0.,456.]};
        let (a,b)=segment(p,2.,-4.,true,frame);
        assert_eq!(a,[10.,22.,30.,0.01]);assert_eq!(b,[10.,16.,30.,0.01]);
        let (c,d)=segment(p,2.,-4.,true,Frame {source_1b8ff0:1,..frame});
        assert_eq!((c,d),(a,b));
        let (a,b)=segment([10.,20.,2.,7.],2.,-48.,false,frame);
        assert_eq!(a,[10.,20.,4.,7.]);assert_eq!(b,[10.,20.,0.,7.]);
    }
    #[test]
    fn original_frame_branch_uses_shared_segment_clip_at_point_one() {
        let (a,b)=segment([5.,5.,2.,1.],2.,-48.,true,Frame {source_1b8ff0:0,source_18a0e0:[0.;4],moby_up:[0.,0.,1.,0.]});
        assert_eq!(a,[5.,5.,4.,1.]);assert!((b[2]-0.1).abs()<0.00001);
        assert_eq!([b[0],b[1],b[3]],[5.,5.,1.]);
    }
}
