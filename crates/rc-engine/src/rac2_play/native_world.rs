//! Internal world integration through the EXISTING engine tick and renderer.
//! Explicit synthetic save inputs: never a replacement for original New Game
//! initialization or an authorized production/user release mode.
use anyhow::{Context,Result};
use rc_formats::{level::LevelCore,moby_anim::{MobyAnimClass,AnimState},rac2::classes::AranosClassUpdates};
use rc_game2::{objects::{LoadedClass,ClassBinding},spawn::SpawnState,world::{World,TickError}};
use crate::{level_load::LoadedLevel,moby_render::MobyOcclusion,moby_anim::MobyAnim};

pub struct LevelContract {pub core:LevelCore,pub updates:AranosClassUpdates,pub rigid:rc_game2::collision::LiveRigidCollision,
    pub controller_data:rc_game2::controller_132d::AranosControllerData,
    pub initial_modifier_records:Vec<u8>}
pub fn enabled()->bool {std::env::var("RC2_NATIVE_WORLD_SAVE_FIXTURE").is_ok_and(|v|v=="1")}

pub struct NativeWorld {
    world:World,
    bindings:Vec<(ClassBinding,Option<usize>)>,
    empty:MobyAnimClass,
    blocked:bool,
    ready:bool,
}

impl NativeWorld {
    pub fn load_fixture(lv:&LoadedLevel)->Result<Self> {
        let contract=lv.rc2_world_contract.as_ref().context("GC original source contracts unavailable")?;
        let empty=MobyAnimClass {joint_count:0,skeleton:Vec::new(),rest:Vec::new(),parent_word:Vec::new(),sequences:Vec::new()};
        let mut animations=Vec::new();let mut classes=Vec::new();
        for (entry,c) in contract.core.moby_classes.iter().enumerate() {
            let anim=lv.mobys.classes.iter().position(|k|k.o_class==c.o_class);
            let loaded=if let Some(i)=anim {
                LoadedClass::from_core(0,&contract.core,entry,&lv.mobys.classes[i].class,&lv.mobys.anim[i],&contract.updates)?
            } else {LoadedClass::without_header(0,&contract.core,entry,&empty,&contract.updates)?};
            animations.push(anim);classes.push(loaded);
        }
        let mut state=SpawnState {progress:vec![0;16],fallback_progress:(0..16).map(|i|(i,3)).collect(),
            mission_status:(-1..16).map(|i|(i,0)).collect(),disabled:vec![0;2048],saved:vec![0;1024],
            reload_saved:Some(vec![0;1024]),reload:false,omitted:Default::default()};
        let collision=lv.collision.as_ref().context("GC native world requires original collision")?;
        let mut world=World::load(&lv.gameplay,&mut classes,&mut state,rc_game::rng::Rng {state:1234},|p|Ok(rc_game2::collision::root_height(collision,p)))?;
        let defs=lv.particles.textures.as_ref().context("GC native world requires original particle definitions")?.defs.clone();
        world.effects=Some(rc_game2::effects::Effects::new(defs));
        world.controller_data=Some(contract.controller_data.clone());
        // The existing opt-in is explicitly a SYNTHETIC save/runtime fixture.
        // These are not inferred New Game globals. No production default is
        // supplied and no original callback is skipped to unblock gameplay.
        world.state_1330=Some(rc_game2::state_1330::State1330 {byte_1a7a12:0,byte_1a7be8:0,
            words_139948:[0;29],halfword_152c26:0});
        let hero=world.objects.records().iter().position(|m|m.o_class==0 && m.state<0x80)
            .context("GC fixture requires actual hero owner")?;
        // Explicit initial-image and null-pointer fixtures, NOT recovered
        // original New Game/runtime modifier/global state.
        world.teleport_modifiers=Some(rc_game2::teleport_modifiers::ModifierTable::read(
            &contract.initial_modifier_records,rc_game2::teleport_modifiers::ModifierBindings {
                hero:Some(hero),source_18b134:None,source_18b040:None})?);
        let bindings=classes.iter().zip(animations).map(|(c,a)|(c.binding(),a)).collect();drop(classes);
        eprintln!("RC2_INTERNAL_WORLD_SAVE_FIXTURE: spawned={} rejected={} provenance=SYNTHETIC_SAVE_AND_RNG_INPUTS original_data=true not_original_New_Game_not_release",world.moby_to_instance.len(),world.instance_to_moby.iter().filter(|i|i.is_none()).count());
        Ok(Self {world,bindings,empty,blocked:false,ready:false})
    }

    fn report(&mut self,error:TickError) {
        eprintln!("RC2_NATIVE_WORLD_BLOCKED: {error:?}; service={:?}; no complete gameplay tick",self.world.pending_1330);
        #[cfg(target_os="ios")]
        crate::ios::diagnostic("RC2_NATIVE_WORLD_BLOCKED",&format!("{error:?}; service={:?}; INTERNAL_SAVE_FIXTURE_NOT_RELEASE",self.world.pending_1330));
        self.blocked=true;
    }

    pub fn initialize(&mut self,lv:&LoadedLevel,placement:usize,hero:&mut rc_game2::hero::Hero,
        pad:&mut rc_game::pad::PadState,anim:&mut rc_game::hero::anim::RatchetAnim,
        hero_class:&MobyAnimClass)->bool {
        if self.blocked {return false;}if self.ready {return true;}
        let classes=self.bindings.iter().map(|(b,a)|b.bind(a.map_or(&self.empty,|i|&lv.mobys.anim[i]))).collect::<Vec<_>>();
        let hero_id=self.world.instance_to_moby.get(placement).copied().flatten();
        let result=self.world.initialize(&classes,|w,id,source| {
            let done=w.native_update(id,source,0);
            if !done && source==0x3846d0 && matches!(w.pending_1330,
                Some(rc_game2::init_1330::Pending1330::Teleport(_)) |
                Some(rc_game2::init_1330::Pending1330::TeleportService {..})) {
                if let Some(hero_id)=hero_id {
                    if let Err(error)=w.resume_1330_teleport(hero_id,hero,pad,anim,hero_class,None,0) {
                        eprintln!("RC2_TELEPORT_BINDING_BLOCKED: {error}");
                    }
                }
            }
            done // Only the complete original callback may advance the cursor.
        });
        drop(classes);
        if let Err(error)=result {self.report(error);return false;}
        self.ready=true;true
    }

    pub fn scene<'a>(&'a self,lv:&'a LoadedLevel)->rc_game::collision_query::MobyScene<'a> {
        lv.rc2_world_contract.as_ref().expect("native world contract").rigid.scene(&self.world)
    }

    pub fn tick(&mut self,lv:&LoadedLevel,counter:u32,placement:usize,hero:&rc_game2::hero::Hero,
        mut anim:AnimState,camera:[f32;3])->bool {
        if self.blocked || !self.ready {return false;}
        if let Some(id)=self.world.instance_to_moby.get(placement).copied().flatten() {
            let m=self.world.objects.get_mut(id).expect("mapped native hero");
            m.position[..3].copy_from_slice(&hero.pos);
            for k in 0..3 {m.rows[k][..3].copy_from_slice(&hero.rows[k]);}
            anim.skip_advance=true;m.anim=anim;
        }
        let classes=self.bindings.iter().map(|(b,a)|b.bind(a.map_or(&self.empty,|i|&lv.mobys.anim[i]))).collect::<Vec<_>>();
        let result=self.world.tick(counter,[camera[0],camera[1],camera[2],0.],&classes,|w,id,source|w.native_update(id,source,counter));
        drop(classes);
        match result {
            Ok(_)=>true,Err(error)=>{self.report(error);false}
        }
    }

    pub fn drive(&self,occlusion:&mut MobyOcclusion,animation:&mut MobyAnim) {
        for (placement,mapped) in self.world.instance_to_moby.iter().enumerate() {
            let Some(id)=mapped else {
                occlusion.drive(placement,[0.;3],[[0.;3];3],1.,None,true);continue;
            };
            let m=self.world.objects.get(*id).expect("GC render map");
            let position=[m.position[0],m.position[1],m.position[2]];
            let rows=std::array::from_fn(|i|[m.rows[i][0],m.rows[i][1],m.rows[i][2]]);
            occlusion.drive(placement,position,rows,m.scale,None,m.state>=0x80 || m.mode&0x81!=0);
            if let Some(k)=occlusion.anim_index(placement) {let mut state=m.anim;state.skip_advance=true;animation.drive(k,state,None,&m.joint_mods);}
        }
    }
}
