//! Original GC SetState(0,2) dependencies recovered for teleport2b9e68.
//! This is not a substitute for the complete2c4560 prologue, surface branches
//! or common tail. R1 idle::clear_look provides the reuse model; GC owns its
//! source fields, while its single existing Rng and animation player are reused.
use super::Hero;
use rc_game::{hero::anim::AnimCtl,rng::Rng};
use rc_formats::buf::{invalid,Result};

/// Additional original idle-entry storage, owned by the existing Hero.
/// Word-address names avoid assigning meaning not established by evidence.
/// Default is a synthetic native initial value, not original New Game proof.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct StateZeroStorage {
    pub byte_22b5:u8,
    pub byte_22bc:u8,
    pub byte_22bd:u8,
    pub byte_22b7:u8,
    pub byte_22c2:u8,
    pub byte_22bf:u8,
    pub half_2504:i16,
    pub half_01fa:i16,
    pub word_1188:i32,
    pub byte_149a:u8,
    pub half_0348:i16,
    pub word_1298:i32,
    pub word_2304:i32,
    pub word_231c:i32,
    pub word_2320:i32,
    pub word_1248:i32,
    /// Existing Hero.cam_mode owns low byte24a0. These are SW's other lanes.
    pub cam_mode_upper:[u8;3],
    pub word_11a0:i32,
    pub look_1194_1198:[u32;2],
    /// Four stride10 source records11b0..11e0. Only words1/2 are cleared.
    pub secondary_11b0_11e0:[[u32;4];4],
}

/// Borrowed services of the one existing native World. No object or RNG is
/// owned here. A false result denotes a genuinely unbound original service;
/// its implementation must make no effects before returning false.
pub trait StateZeroServices {
    fn clear_moby_source_9c(&mut self)->Result<()>;
    fn end_hero_trail(&mut self,record:&mut crate::teleport_world::TrackedObjects)->Result<()>;
    fn clear_global_1b8b80(&mut self)->Result<bool>;
    fn clear_camera_1b9000(&mut self)->Result<bool>;
    fn stop_group_audio(&mut self,hero:&mut Hero)->Result<bool>;
    fn rng(&mut self)->&mut Rng;
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StateZeroStep {Guard,History,Body,Trail,Global,Entry,BodyEntry,Surface,Animation,Tail,Camera,Audio,Timers,Done}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StateZeroResult {Rejected,Pending(u32),Complete}

/// Source-specific continuation, not an independent state machine: each
/// checkpoint ends at a mandatory native service of original2c4560(0,2).
/// Saved byte22b7 is the original s6 value, needed after body cleanup.
#[derive(Clone,Debug)]
pub struct StateZeroProgress {pub next:StateZeroStep,old_byte_22b7:u8}

impl Default for StateZeroProgress {
    fn default()->Self {Self {next:StateZeroStep::Guard,old_byte_22b7:0}}
}

impl StateZeroProgress {
    pub fn resume(&mut self,hero:&mut Hero,anim:&mut dyn AnimCtl,services:&mut dyn StateZeroServices,
        layer_present:Option<bool>)->Result<StateZeroResult> {
        loop {
            let following=match self.next {
                StateZeroStep::Guard=>{
                    if !hero.state_zero_mode2_allowed() {return Ok(StateZeroResult::Rejected);}
                    self.old_byte_22b7=hero.state_zero.byte_22b7;
                    StateZeroStep::History
                },
                StateZeroStep::History=>{hero.apply_state_zero_history()?;StateZeroStep::Body},
                StateZeroStep::Body=>{
                    if hero.mode==0 {
                        services.clear_moby_source_9c()?;
                        hero.clear_state_zero_body_bytes();
                        StateZeroStep::Trail
                    }else{StateZeroStep::Global}
                },
                StateZeroStep::Trail=>{
                    services.end_hero_trail(&mut hero.teleport.trails[0])?;
                    StateZeroStep::Global
                },
                StateZeroStep::Global=>{
                    if hero.state_zero.half_01fa==0 && !services.clear_global_1b8b80()? {
                        return Ok(StateZeroResult::Pending(0x2d6d08));
                    }
                    //2af450(previous,0) provably returns before effects.
                    hero.state_zero.word_1188=0;StateZeroStep::Entry
                },
                StateZeroStep::Entry=>{
                    if hero.state_zero.word_1248==0x1a && hero.putaway.byte_22b8!=0 && layer_present.is_none() {
                        return Ok(StateZeroResult::Pending(0x2af848));
                    }
                    hero.apply_state_zero_entry_through_2ab458(anim,services.rng(),layer_present.unwrap_or(false))?;
                    StateZeroStep::BodyEntry
                },
                StateZeroStep::BodyEntry=>{
                    if hero.mode!=0 {return Ok(StateZeroResult::Pending(0x2c8e30));}
                    hero.momentum=hero.eff;
                    hero.motion_reset.vector_w[4]=hero.motion_reset.vector_w[2];
                    StateZeroStep::Surface
                },
                StateZeroStep::Surface=>{
                    if hero.surface_flag_1492!=0 && f32::from_bits(0x3c08_8889)<super::physics::len(hero.momentum) {
                        return Ok(StateZeroResult::Pending(0x2c8734)); //recursive SetState2e/mode2.
                    }
                    if hero.state_zero.byte_149a!=0 && hero.f16c==0 {
                        return Ok(StateZeroResult::Pending(0x2c8734)); //recursive SetState76/mode2.
                    }
                    hero.state_zero.word_1188=0x68;StateZeroStep::Animation
                },
                StateZeroStep::Animation=>{
                    let seq=hero.idle_seq();hero.set_anim(anim,if seq==0x51 {18.}else{-1.},seq,0);
                    StateZeroStep::Tail
                },
                StateZeroStep::Tail=>{
                    if self.old_byte_22b7!=0 && hero.state_zero.byte_22b7==0 {hero.state_zero.word_231c=1;}
                    if hero.state_zero.half_0348==0 && hero.state_zero.word_1298==0x13 && hero.state_zero.word_2304!=0 {
                        hero.state_zero.word_2320=1;
                    }
                    if hero.state_zero.word_1298==0x14 && hero.state_zero.word_2304!=0 {hero.state_zero.word_2320=1;}
                    // New group/state are0 on this branch: other original
                    // groupd..f, state6/prev24, group14 branches cannot run.
                    StateZeroStep::Camera
                },
                StateZeroStep::Camera=>{
                    //2adff8 is false for state0/group0. Both outcomes of the
                    // subsequent floating compare clear1b9000 becausegroup0.
                    if !services.clear_camera_1b9000()? {return Ok(StateZeroResult::Pending(0x2c8d94));}
                    StateZeroStep::Audio
                },
                StateZeroStep::Audio=>{
                    if hero.group!=hero.prev_group && !services.stop_group_audio(hero)? {
                        return Ok(StateZeroResult::Pending(0x2a6708));
                    }
                    StateZeroStep::Timers
                },
                StateZeroStep::Timers=>{
                    hero.timer=0;hero.substate_timer=0;
                    if hero.group!=hero.prev_group {hero.group_timer=0;}
                    hero.seq_timer=0;StateZeroStep::Done
                },
                StateZeroStep::Done=>return Ok(StateZeroResult::Complete),
            };
            self.next=following;
        }
    }
}

/// Complete2d6d08: clears the first byte of eight16byte global records at
///1b8b80, from last to first; preserves every other byte. Caller retains the
/// actual global-table ownership and must establish its original inputs.
pub fn clear_source_1b8b80(records:&mut [[u8;16];8]) {
    for record in records.iter_mut().rev() {record[0]=0;}
}

impl Hero {
    /// Complete2c4560's guards specialized to id0/mode2. The global1a8f00
    /// guard only rejects a mode other than2, so needs no invented global
    /// value here. A rejection does not run the history or consume RNG.
    pub fn state_zero_mode2_allowed(&self)->bool {
        self.group!=0x14 && !(self.hp==0 && matches!(self.group,2|7))
    }

    /// Ordered2c467c..2c46cc history/byte writes for the accepted state0
    /// specialization. Caller still owes the moby/trail/global prologue and
    /// entry/tail; this intentionally does not return SetState success.
    pub fn apply_state_zero_history(&mut self)->Result<()> {
        if !self.state_zero_mode2_allowed() {return invalid("GC state0/mode2 rejected by original guard");}
        self.prev2_state=self.prev_state;
        self.prev2_group=self.prev_group;
        self.prev_timer=self.timer;
        self.prev_state=self.state;
        self.prev_group=self.group;
        self.state=0;
        self.substate=0;
        if self.state_zero.byte_22b5==1 {self.state_zero.byte_22b5=0;}
        self.state_zero.byte_22bc=0;
        Ok(())
    }

    /// Writes2c46d0..2c46f0 for body0. The caller lends the actual moby's
    /// source+9c storage (not collision-disable+98) and must then execute
    ///324140 on Hero.teleport.trails[0] before `state_zero_after_trail`.
    /// Returns whether that trail teardown is required; no owner is copied.
    pub fn state_zero_body_prologue(&mut self,moby_word_9c:&mut u32)->bool {
        if self.mode!=0 {return false;}
        *moby_word_9c=0;
        self.clear_state_zero_body_bytes();
        true
    }

    fn clear_state_zero_body_bytes(&mut self) {
        self.state_zero.byte_22bd=0;
        self.state_zero.byte_22b7=0;
        self.state_zero.byte_22c2=0;
        self.state_zero.byte_22bf=0;
        self.state_zero.half_2504=0;
    }

    ///2c46f4..2c471c, after the required trail teardown. For id0 the call
    ///2af450(prevstate,0) returns before all side effects at2af4a4, for every
    /// valid input. This is a source-specific return, not a skipped callback.
    pub fn state_zero_after_trail(&mut self,source_1b8b80:&mut [[u8;16];8])->Result<()> {
        if self.state!=0 {return invalid("GC state-zero prologue requires id0");}
        if self.state_zero.half_01fa==0 {clear_source_1b8b80(source_1b8b80);}
        self.state_zero.word_1188=0;
        Ok(())
    }
    /// Complete2ab458, shared in shape with RAC1 Idle::clear_look. It writes
    /// exactly ten words; no RNG, animation, state or velocity reset.
    pub fn clear_original_idle_look(&mut self) {
        self.state_zero.look_1194_1198=[0;2];
        for record in &mut self.state_zero.secondary_11b0_11e0 {
            record[1]=0;record[2]=0;
        }
    }

    /// Ordered state-zero entry2c473c..2c4774 ONLY, after the caller has run
    /// the required SetState prologue. In particular this does not set state,
    /// reset state timers, run surface transitions, or report SetState success.
    /// One31b9e0(50,100) draw is made through the existing world's one stream.
    pub fn apply_state_zero_entry_through_2ab458(&mut self,anim:&mut dyn AnimCtl,
        rng:&mut Rng,layer_present:bool)->Result<()> {
        if self.state!=0 {return invalid("GC state-zero entry requires original SetState prologue");}
        self.group=0;
        self.cam_mode=0;self.state_zero.cam_mode_upper=[0;3];
        if self.state_zero.word_1248==0x1a {self.put_away_teleport(anim,layer_present);}
        //31b9e0 matches the shared native rand_range, including the one boot
        //1163b0 LCG draw and signed remainder. Do not create/reseed an RNG.
        //2c4768 is the JAL delay slot: this precedes the RNG draw.
        self.teleport.byte_18c30f=0; //hero+24ef: same owner as teleport writes.
        self.state_zero.word_11a0=rng.rand_range(0x32,0x64);
        self.clear_original_idle_look();
        Ok(())
    }

    /// Complete2adff8, called by the original SetState common tail. No role
    /// beyond the verified group/state predicate is assigned here.
    pub fn query_2adff8(&self)->bool {
        matches!(self.group,0x11|0x12) || matches!(self.state,0x67|0x7f|0x73|0x72)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::hero::anim::RecordingAnim;
    #[test]
    fn source_clear_services_preserve_every_unwritten_word_and_byte() {
        let mut h=Hero::new();h.state_zero=StateZeroStorage {word_1248:7,cam_mode_upper:[8;3],word_11a0:9,
            byte_22b5:2,byte_22bc:3,..Default::default()
        };h.state_zero.look_1194_1198=[u32::MAX,0x80000000];h.state_zero.secondary_11b0_11e0=[[11,12,13,14];4];
        h.clear_original_idle_look();
        assert_eq!(h.state_zero.look_1194_1198,[0;2]);
        assert_eq!(h.state_zero.secondary_11b0_11e0,[[11,0,0,14];4]);
        assert_eq!((h.state_zero.word_1248,h.state_zero.cam_mode_upper,h.state_zero.word_11a0),(7,[8;3],9));
        let mut records=[[0xab;16];8];clear_source_1b8b80(&mut records);
        assert!(records.iter().all(|r| r[0]==0 && r[1..]==[0xab;15]));
    }
    #[test]
    fn state_zero_mode2_guards_and_history_preserve_unwritten_timers() {
        let mut h=Hero::new();h.state=7;h.group=2;h.hp=0;
        assert!(!h.state_zero_mode2_allowed());assert!(h.apply_state_zero_history().is_err());assert_eq!(h.state,7);
        h.hp=8;h.prev_state=3;h.prev_group=1;h.timer=17;h.group_timer=18;h.substate_timer=19;h.seq_timer=20;
        h.state_zero.byte_22b5=1;h.state_zero.byte_22bc=7;
        h.apply_state_zero_history().unwrap();
        assert_eq!((h.prev2_state,h.prev2_group,h.prev_timer,h.prev_state,h.prev_group,h.state,h.substate),(3,1,17,7,2,0,0));
        assert_eq!((h.timer,h.group_timer,h.substate_timer,h.seq_timer),(17,18,19,20));
        assert_eq!((h.state_zero.byte_22b5,h.state_zero.byte_22bc),(0,0));
        h.state_zero.byte_22b5=2;h.apply_state_zero_history().unwrap();assert_eq!(h.state_zero.byte_22b5,2);
        h.group=0x14;assert!(!h.state_zero_mode2_allowed());
    }
    #[test]
    fn state_zero_slice_consumes_exact_one_existing_rng_draw_and_keeps_pending_tail() {
        let mut h=Hero::new();h.state=0;h.group=7;h.timer=123;h.seq_timer=456;
        h.state_zero.word_1248=0x1a;h.cam_mode=77;h.state_zero.cam_mode_upper=[78;3];
        h.putaway.byte_22b8=1;h.putaway.byte_22ba=1;h.putaway.word_24b0=0;
        h.teleport.byte_18c30f=3;let mut a=RecordingAnim::default();
        let mut rng=Rng {state:0xffff_ffff};let mut reference=rng;
        let expected=reference.rand_range(50,100);
        h.apply_state_zero_entry_through_2ab458(&mut a,&mut rng,false).unwrap();
        assert_eq!(rng,reference);assert_eq!(h.state_zero.word_11a0,expected);
        assert_eq!((h.group,h.cam_mode,h.state_zero.cam_mode_upper,h.teleport.byte_18c30f),(0,0,[0;3],0));
        assert_eq!((h.state,h.timer,h.seq_timer),(0,123,456),"not a complete SetState call");
        assert_eq!(a.calls.len(),1);assert_eq!(h.putaway.byte_22b8,0);
        let before=rng;h.state=2;
        assert!(h.apply_state_zero_entry_through_2ab458(&mut a,&mut rng,false).is_err());assert_eq!(rng,before);
    }
    #[test]
    fn original_common_tail_query_uses_both_group_and_state() {
        let mut h=Hero::new();assert!(!h.query_2adff8());
        for group in [0x11,0x12] {h.group=group;assert!(h.query_2adff8());}
        h.group=0;
        for state in [0x67,0x7f,0x73,0x72] {h.state=state;assert!(h.query_2adff8());}
        h.state=0x71;assert!(!h.query_2adff8());
    }
}
