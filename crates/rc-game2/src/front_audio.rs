//! Original boot frontend sound requests through the existing native 989snd/SPU.
//! Boot 0x26f0ac loads frontend part1 with 0x2b7700. 0x26f240/264
//! bind 20 SoundDefs at 0x238c00; 0x2e6c90 indexes them in 0x20-byte
//! steps and calls 0x2e6908 with volume0x400. Navigation calls it with3
//! at 0x348d24. No RAC1 level, emitter, music or gameplay tick runs here.
//! GC master-volume/options and full IOP parity still require comparison.
use rc_formats::{buf::{invalid, Result}, rac2::{boot::BootElf, front_end::FrontEnd},
    sound_bank::{self, LevelSounds, SoundDef}};
use rc_game::audio::{AudioSystem, LevelAudio, SAMPLES_PER_FRAME};
use std::sync::Arc;

pub struct FrontAudio {
    system: AudioSystem,
    defs: Vec<SoundDef>,
}

impl FrontAudio {
    pub fn load(front: &FrontEnd, boot: &BootElf) -> Result<Self> {
        let bank = sound_bank::parse_bank(front.part(rc_formats::rac2::front_end::SOUND_BANK))?;
        let defs = boot.frontend_sound_defs()?;
        if defs.iter().any(|d| d.index != 0xffff && d.index as usize >= bank.sounds.len()) {
            return invalid("GC frontend SoundDef index exceeds original bank");
        }
        // The common bank container holds real frontend resources. There is
        // no level emitter/music/collision data in this frontend adapter.
        let data = LevelAudio { bank: Arc::new(bank), sounds: LevelSounds {
            level_defs: defs.clone(), map: Vec::new(), classes: Vec::new() },
            instances: Vec::new(), instance_pvars: Vec::new(), music: Default::default(),
            env_points: Vec::new(), collision: None };
        Ok(Self { system: AudioSystem::new(data), defs })
    }

    /// The supported 2-D one-shot request, with the original bank/def id.
    pub fn play(&mut self, request: u32) -> Result<u32> {
        let Some(def) = self.defs.get(request as usize) else { return invalid("GC frontend sound request out of range"); };
        if def.looped != 0 || def.pb_lo != def.pb_hi {
            return invalid("GC frontend looping/random-pitch request requires its own verified adapter");
        }
        let s = &mut self.system;
        let handle = s.snd.play_sound(&s.data.bank, &mut s.spu, def.index, 0x400, 0, 0, def.pb_hi);
        if handle == 0 { return invalid("GC frontend original bank rejected sound request"); }
        Ok(handle)
    }

    pub fn render(&mut self, out: &mut Vec<[i16; 2]>) {
        out.clear();
        // Native IOP/SPU only: exactly four240Hz steps /800samples. The
        // RAC1 EE sound_update/game_frame and level scheduler are not called.
        self.system.render(SAMPLES_PER_FRAME, out);
    }

    pub fn unsupported_grains(&self) -> usize { self.system.snd.unimplemented.len() }
    pub fn bank_sound_count(&self) -> usize { self.system.data.bank.sounds.len() }
}
