//! Native GC ground-combo timing kernel, from L00 0x2c7dd4 / 0x2bddd8 / 0x2ccb00.
//! Consumed by internal original-data tests while GC item ownership, target aim,
//! wrench poses/hits, damage callbacks and class lifecycle are being reconstructed.
//! This kernel alone is not combat and does not enable a production action.
use rc_formats::rac2::melee::{AranosMeleeTable, MeleeTiming};
use rc_game::{hero::anim::AnimCtl, pad::{button, PadState}, ps2v::Pf};

#[derive(Clone, Copy, Debug)]
pub struct EntryHistory {
    pub previous_state: i32,
    pub older_state: i32,
    /// +0x1b8 and +0x22a8 at entry, after SetState's bookkeeping/hook.
    pub timer: i32,
    pub saved_timer: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboEntry {
    pub step: usize,
    pub hero_sequence: u8,
    pub wrench_sequence: u8,
    pub start_frame: i32,
    pub blend: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpRequest { Plain, CrouchChooser }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery { Continue, Walk, Idle }

/// Original input-side item/hero fields used by 0x2c96d0, never a RAC1 pvar view.
#[derive(Clone, Copy, Debug)]
pub struct WrenchUse {
    pub state: i32,
    pub group: i32,
    pub timer: i32,
    pub frame: f32,
    pub blending: bool,
    /// Hero +0x1248: GC wrench item is 10 (RAC1 uses 8).
    pub item_id: i32,
    /// Hero +0x1244: item ownership state must be 2.
    pub item_state: i32,
    /// Original mask at +0x1230; it is not inferred from a hardware button slot.
    pub fire_mask: u32,
    /// Present item moby's GC class (+0xaa) and state (+0x20).
    pub hand: Option<(i16,u8)>,
    pub hero_mode: u8,
    pub item_disabled: u8,
    pub change_gate: u8,
    pub f348: i16,
    pub f149a: u8,
    pub f1e0: i16,
    pub landed: u8,
    pub descending: u8,
    pub flip_kind: i32,
    pub height: f32,
    pub f6ec: i32,
    pub f594: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponDecision {
    None,
    Enter(i32),
    /// Native 0x2ba610 needs the actual item/flight runtime, not a pretend throw.
    ThrowFromLook,
    UnsupportedItem(i32),
}

/// Input selection in GC 0x2c96d0 case 10 (0x2c9790..0x2c9b8c).
/// Returned entries are requests. The caller must provide the real item, state
/// entry, moby lifecycle and hit callbacks before executing them in gameplay.
pub fn wrench_request(c: &WrenchUse, row: &MeleeTiming, pad: &PadState) -> WeaponDecision {
    use WeaponDecision::*;
    if c.change_gate != 0 && (c.item_id != 10 || c.f348 != 1) { return None; }
    if c.hero_mode != 0 || c.hand.is_none() || c.item_disabled != 0 || c.item_state != 2 { return None; }
    if c.item_id != 10 { return UnsupportedItem(c.item_id); }
    if c.f149a != 0 { return None; }
    let pressed = |ticks| pad.pressed_within(c.fire_mask,ticks).is_some();
    if c.f1e0 == 0 && ((c.group as u32) < 2 || c.group == 4 && c.landed != 0) {
        if !pressed(10) || c.hand != Some((0x47,0)) { return None; }
        let look = matches!(c.state,1|0x1d) && c.timer >= 21;
        if pad.held & button::R1 == 0 && !look { return Enter(0x13); }
        return if look {ThrowFromLook}else{Enter(0x15)};
    }
    let air = matches!(c.state,7|9|0x89|8|6|0x2c|0x11)
        || c.state==0xe && c.frame>32.0
        || c.state==0xb && c.frame>if c.flip_kind==3 {20.0}else{18.0};
    if air && c.timer>=11 {
        if !pressed(if matches!(c.state,0xb|0xe) {18}else{14}) { return None; }
        let threshold = if c.descending == 0 {f32::from_bits(0x3dcccccd)}else{f32::from_bits(0x3f11eb85)};
        if c.height>threshold || c.state==8 && c.height>f32::from_bits(0x3f11eb85) {
            return if c.f6ec==0 && c.f594==0 {Enter(0x14)}else{None};
        }
        return Enter(0x13);
    }
    if c.group!=6 { return None; }
    if c.state==0x15 && c.timer>=31 && !c.blending && c.frame>=row.chain_from as f32
        && pad.held & button::R1!=0 && pressed(15) { return Enter(0x15); }
    if c.blending || c.frame<row.chain_from as f32 { return None; }
    let ticks=Pf::f(c.frame).to_i32().wrapping_sub(row.input_ref).wrapping_mul(2).clamp(1,15);
    if !pressed(ticks) { return None; }
    Enter(if pad.held & button::R1!=0 {0x15}else{0x13})
}

pub struct GroundCombo<'a> { table: &'a AranosMeleeTable, step: usize }

impl<'a> GroundCombo<'a> {
    pub fn new(table: &'a AranosMeleeTable) -> Self { Self {table,step:0} }
    pub fn timing(&self) -> &MeleeTiming { &self.table.rows()[self.step] }

    /// Original ground entry, before target-dependent blend adjustment.
    /// The short return-to-combo branch is new in GC; RAC1's group test differs.
    pub fn enter(&mut self, history: EntryHistory, old_frame: f32) -> ComboEntry {
        let old = self.timing();
        let chain = history.previous_state == 0x13 && old_frame < old.chain_before as f32
            || history.older_state == 0x13 && history.timer.wrapping_add(history.saved_timer) < 27;
        self.step = if chain { (old.step as usize + 1) % 3 } else { 0 };
        ComboEntry {step:self.step,hero_sequence:0x17+self.step as u8,wrench_sequence:3+self.step as u8,
            start_frame:self.step as i32,blend:if self.step==0 {5}else{7}}
    }

    /// Native hero sequence playback, using the existing shared animation player.
    /// Wrench sequence synchronization remains the item runtime's responsibility.
    pub fn set_hero_animation(entry: ComboEntry, anim: &mut dyn AnimCtl) {
        anim.set_anim(Pf::from_i32(entry.blend),entry.hero_sequence,entry.start_frame);
    }

    /// 0x2bddd8: prevent item swap while blending or before this record's end.
    pub fn hold_item(&self, frame: f32, blending: bool) -> bool { blending || frame < self.timing().hit_to as f32 }

    /// Target-free scalar part of 0x2bdfb4..0x2be114. No guessed aim target.
    pub fn target_speed(&self, frame: f32, blending: bool, speed_factor: f32) -> f32 {
        let c = match self.step {
            0 if frame < 18.0 => f32::from_bits(0x3d962fca),
            1 if blending || frame < 9.0 => if frame > 8.0 {f32::from_bits(0x3d4cccce)} else if frame > 7.0 {f32::from_bits(0x3d99999a)} else {f32::from_bits(0x3dc28f5c)},
            2 if blending || frame < 12.0 => f32::from_bits(0x3d7c9631),
            _ => 0.0,
        };
        c * speed_factor
    }

    /// First branch of 0x2ccb00. Call the real crouch chooser before recovery
    /// when requested; failure of that chooser does not suppress recovery.
    pub fn jump_request(&self, frame: f32, blending: bool, pad: &PadState) -> Option<JumpRequest> {
        if blending || frame <= self.timing().jump_after as f32 { return None; }
        let window = Pf::f(frame).to_i32().wrapping_sub(self.timing().input_ref).wrapping_mul(2);
        if pad.pressed_within(button::CROSS,window).is_none() { return None; }
        Some(if pad.held & button::R1 != 0 {JumpRequest::CrouchChooser}else{JumpRequest::Plain})
    }

    /// Ground-combo recovery from 0x2ccbc0..0x2cccbc (not the throw's path).
    pub fn recovery(&self, frame: f32, blending: bool, animation_flags: u16, stick_magnitude: f32) -> Recovery {
        if blending { return Recovery::Continue; }
        if frame > 34.0 && stick_magnitude > 0.9 { return Recovery::Walk; }
        let end = self.timing().idle_after as f32;
        if frame > end {
            if stick_magnitude > 0.8 { return Recovery::Walk; }
            if frame > end+5.0 || animation_flags & 2 != 0 { return Recovery::Idle; }
        }
        Recovery::Continue
    }
}
