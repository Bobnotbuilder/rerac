//! Read-only original-data integration test. RC2_ORIGINAL_ISO is private and optional in CI.
//! CPU evidence only: never a claim of GPU rendering, controller hardware or full-level playability.
use rc_formats::{Game, iso9660::IsoImage, level, moby, moby_anim as ma, rac2::{self, level::{LevelWad,Lump,Part}}, wad};
use rc_game::{pad::{PadInput,PadState,button},hero::anim::RatchetAnim};
use rc_game2::{hero::{Hero,hero_update,HeroTick,physics::Env},camera::{FollowCamera,CamInput}};

#[test]
fn original_aranos_character_pose_movement_jump_and_camera() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {
        eprintln!("NOT_TESTED: private RC2_ORIGINAL_ISO unavailable; no original-data compatibility claim");return;
    };
    let iso=IsoImage::open(path).unwrap();
    let read=|path:&str|iso.read_file(iso.find(path).unwrap()).unwrap();
    let elf=read("/SCUS_972.68");
    assert_eq!(rc_formats::sha1::hex(&rc_formats::sha1::sha1(&elf)),"4124a79c1e5a1ceb56f514df0dbbc11f072af56f");
    let file=read("/G/LEVEL0.WAD");let w=LevelWad::parse(&file).unwrap();
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let wrench_blob=core.block(&data,"moby_class/0071").expect("original wrench class 0x47");
    let wrench=moby::parse_moby_class_for(Game::Rac2,wrench_blob).unwrap();
    let wrench_class=ma::MobyAnimClass::new(&wrench,ma::parse_sequences(wrench_blob,&wrench).unwrap());
    for seq in [3,4,5,10] {
        let q=wrench_class.sequence(seq).expect("original wrench action sequence");assert!(!q.frames.is_empty());
        let mut state=ma::AnimState::spawn(&wrench_class);assert!(ma::hard_cut(&mut state,&wrench_class,seq,0));
        assert!(ma::evaluate(&wrench_class,&state).iter().flatten().flatten().all(|v|v.is_finite()));
    }
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let mut ratchet=moby::parse_moby_class_for(Game::Rac2,core.block(&data,"moby_class/0000").unwrap()).unwrap();
    let hdr=read("/RC2.HDR");let toc=rac2::toc::Toc::parse(&hdr).unwrap();
    let a=rac2::armor::read(&read("/G/ARMOR.WAD"),rac2::armor::armors(&toc).unwrap()[0]).unwrap();
    moby::attach_packets(Game::Rac2,&mut ratchet,&a.mesh).unwrap();
    assert_eq!(ratchet.header.joint_count,111);assert_eq!(a.textures.len(),4);
    assert!(!ratchet.high_lod.is_empty());
    assert!(ratchet.high_lod.iter().all(|s|s.unresolved_duplicates==0));
    let sequences=(0..core.ratchet_seqs.len()).map(|i|core.block(&data,&format!("ratchet_seq/{i:03}")).map(|b|ma::parse_sequence_joints(b,0,Some(111)).unwrap())).collect::<Vec<_>>();
    let class=ma::MobyAnimClass::new(&ratchet,sequences);
    let mut pose_frames=0;
    for (s,q) in class.sequences.iter().enumerate() {
        let Some(q)=q else {continue};
        for f in 0..q.frames.len() {
            let mut state=ma::AnimState::spawn(&class);
            assert!(ma::hard_cut(&mut state,&class,s as u8,f as i32));
            let pose=ma::evaluate(&class,&state);assert_eq!(pose.len(),111);
            assert!(pose.iter().flatten().flatten().all(|v|v.is_finite()));pose_frames+=1;
        }
    }
    let collision=rc_formats::collision::parse_collision(&core,&data).unwrap();
    let instances=rac2::gameplay::moby_instances(&gameplay).unwrap();
    let spawn=&instances.iter().find(|i|i.base.o_class==0).unwrap().base;
    let mut hero=Hero::spawn(spawn.position,spawn.rotation[2]);

    let mut camera=FollowCamera::new(&camera_input(&hero,&collision,[0.;2]));
    let mut anim=RatchetAnim::new(&class);let mut pad=PadState::default();
    let mut grounded=0;let mut max_z=f32::NEG_INFINITY;let mut launch_z=0.;let mut jump_state=false;
    for tick in 0..240 {
        let input=if tick==60 {PadInput::neutral().press(button::CROSS)} else {PadInput::neutral()};
        pad.update(Some(&input.bytes()),false);
        let env=Env{mobys:None,hero_moby:None,coll:&collision,pad:&pad,cam_yaw:camera.yaw(),cam_rows:camera.rows,mirror:false};
        assert_eq!(hero_update(&mut hero,&env,&mut anim.ctl(&class)),HeroTick::Ran);
        if tick==59 {launch_z=hero.pos[2]}
        if tick>=60 {max_z=max_z.max(hero.pos[2]);jump_state|=hero.state==7;}
        if tick>180 && hero.grounded(){grounded+=1;}
        camera.update(&camera_input(&hero,&collision,if tick>210 {[0.25,0.]}else{[0.;2]}));
        assert!(hero.pos.iter().chain(camera.pos.iter()).all(|v|v.is_finite()));
    }
    assert!(jump_state,"original animation-driven jump transition missing");
    assert!(max_z>launch_z+0.1,"jump did not rise: {launch_z} -> {max_z}");
    assert!(grounded>0,"jump did not land on original collision: {:?}",hero.pos);
    let before=hero.pos;
    for _ in 0..12 {
        pad.update(Some(&PadInput::neutral().stick(0.,-1.).bytes()),false);
        let env=Env{mobys:None,hero_moby:None,coll:&collision,pad:&pad,cam_yaw:camera.yaw(),cam_rows:camera.rows,mirror:false};
        assert_eq!(hero_update(&mut hero,&env,&mut anim.ctl(&class)),HeroTick::Ran);
        camera.update(&camera_input(&hero,&collision,[0.;2]));
    }
    let delta=((hero.pos[0]-before[0]).powi(2)+(hero.pos[1]-before[1]).powi(2)).sqrt();
    assert!(delta>0.01,"original movement path did not move");
    println!("ORIGINAL_RC2_CPU_PASS joints=111 textures=4 pose_frames={pose_frames} jump_rise={} grounded_ticks={grounded} movement_distance={delta} spawn={:?} final={:?} camera={:?}; GPU_NOT_TESTED DEVICE_NOT_TESTED LEVEL_COMPLETION_NOT_TESTED",max_z-launch_z,spawn.position,hero.pos,camera.pos);
}

fn camera_input<'a>(hero: &'a Hero, coll: &'a rc_formats::collision::Collision, rstick: [f32;2]) -> CamInput<'a> {
    CamInput {hero,mobys:None,hero_moby:None,coll:Some(coll),rstick,yaw_normal:true,pitch_normal:true}
}

#[test]
fn original_aranos_fall_defers_steering_until_landing_lag_expires() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {
        eprintln!("NOT_TESTED: private original ISO unavailable");return;
    };
    let iso=IsoImage::open(path).unwrap();
    let file=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();
    let w=LevelWad::parse(&file).unwrap();
    let sections=rc_formats::font::parse_overlay_sections(w.lump(Lump::Overlay).unwrap().unwrap()).unwrap();
    // Pin this rule to the loaded Aranos instruction stream, not the boot
    // ELF's different code at these addresses. This is not MIPS execution.
    for (pc,word) in [(0x2c1140,0x860201e0u32),(0x2c1144,0x14400003),(0x2c1148,0),
        (0x2c114c,0x0c0adb34),(0x2c1150,0),(0x2c1154,0x0c0adf22)] {
        assert_eq!(rc_formats::font::read_overlay(&sections,pc,4).unwrap(),word.to_le_bytes());
    }
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let ratchet=moby::parse_moby_class_for(Game::Rac2,core.block(&data,"moby_class/0000").unwrap()).unwrap();
    let sequences=(0..core.ratchet_seqs.len()).map(|i|core.block(&data,&format!("ratchet_seq/{i:03}"))
        .map(|b|ma::parse_sequence_joints(b,0,Some(111)).unwrap())).collect();
    let class=ma::MobyAnimClass::new(&ratchet,sequences);
    let coll=rc_formats::collision::parse_collision(&core,&data).unwrap();
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let instances=rac2::gameplay::moby_instances(&gameplay).unwrap();
    let spawn=instances.iter().find(|i|i.base.o_class==0).unwrap().base.position;
    // Explicit positioned state fixture, not a New Game or savestate capture.
    let mut hero=Hero::spawn([spawn[0],spawn[1],spawn[2]+20.],0.);
    let mut anim=RatchetAnim::new(&class);let mut pad=PadState::default();
    pad.update(Some(&PadInput::neutral().stick(0.,-1.).bytes()),false);
    let env=Env{coll:&coll,mobys:None,hero_moby:None,pad:&pad,cam_yaw:0.,
        cam_rows:[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]],mirror:false};
    assert!(hero.set_state(&mut anim.ctl(&class),&env,6,true));
    hero.land_lag=2;
    for tick in 0..3 {
        assert_eq!(hero_update(&mut hero,&env,&mut anim.ctl(&class)),HeroTick::Ran);
        assert_eq!(hero.state,6);
        assert!(hero.target_speed>0.,"stick target must still update during lag");
        assert!(hero.vel[2]<0.,"gravity must still run during lag");
        if tick<2 {
            assert_eq!(&hero.vel[..2],&[0.,0.],"steering applied during landing lag");
            assert_eq!(&hero.pos[..2],&spawn[..2]);
        } else {
            assert!(hero.vel[0]>0.,"steering must resume on the first zero-lag tick");
            assert!(hero.pos[0]>spawn[0]);
        }
        let pose=ma::evaluate(&class,&anim.state);
        assert_eq!(pose.len(),111);assert!(pose.iter().flatten().flatten().all(|v|v.is_finite()));
    }
    println!("ORIGINAL_RC2_FALL_LAG_CPU_PASS original_overlay_guard=2c1140 original_animations=111_joints lag_ticks=2 resumed_tick=3; POSITIONED_FIXTURE_NOT_GAME_CAPTURE GPU_NOT_TESTED DEVICE_NOT_TESTED");
}

#[test]
fn original_gc_static_world_spawn_ownership_and_first_native_callback_boundary() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let file=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();
    let wad=LevelWad::parse(&file).unwrap();let overlay=wad.lump(Lump::Overlay).unwrap().unwrap();
    let registry=rac2::classes::AranosClassUpdates::parse(0,overlay).unwrap();
    let data=rc_formats::wad::decompress(wad.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,wad.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let gameplay=rc_formats::wad::decompress(wad.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let placements=rac2::gameplay::moby_instances(&gameplay).unwrap();
    let classes=moby::parse_level_mobys(&core,&data).unwrap();
    let animations=classes.iter().map(|c| {
        let sequences=if c.o_class==0 {(0..core.ratchet_seqs.len()).map(|i|
            core.block(&data,&format!("ratchet_seq/{i:03}")).map(|b|ma::parse_sequence_joints(b,0,Some(c.class.header.joint_count as usize)).unwrap())).collect()}
        else {ma::parse_sequences(core.block(&data,&format!("moby_class/{:04}",c.o_class)).unwrap(),&c.class).unwrap()};
        ma::MobyAnimClass::new(&c.class,sequences)
    }).collect::<Vec<_>>();
    let empty=ma::MobyAnimClass {joint_count:0,skeleton:Vec::new(),rest:Vec::new(),parent_word:Vec::new(),sequences:Vec::new()};
    let mut loaded=core.moby_classes.iter().enumerate().map(|(entry,c)| {
        if let Some(i)=classes.iter().position(|k|k.o_class==c.o_class) {
            rc_game2::objects::LoadedClass::from_core(0,&core,entry,&classes[i].class,&animations[i],&registry).unwrap()
        } else {rc_game2::objects::LoadedClass::without_header(0,&core,entry,&empty,&registry).unwrap()}
    }).collect::<Vec<_>>();
    // Explicit save-input fixture, not a claimed capture of New Game state.
    // Fallback3 is 0x2e4450's zero-statistics result. No all-spawn shortcut.
    let mut state=rc_game2::spawn::SpawnState {progress:vec![0;16],fallback_progress:(0..16).map(|i|(i,3)).collect(),
        mission_status:(-1..16).map(|i|(i,0)).collect(),disabled:vec![0;2048],saved:vec![0;1024],
        reload_saved:Some(vec![0;1024]),reload:false,omitted:Default::default()};
    let coll=rc_formats::collision::parse_collision(&core,&data).unwrap();
    let mut world=rc_game2::world::World::load(&gameplay,&mut loaded,&mut state,rc_game::rng::Rng {state:1234},
        |p|Ok(rc_game2::collision::root_height(&coll,p))).unwrap();
    let count=world.moby_to_instance.len();assert!(count>0 && count<placements.len());
    for (id,&placement) in world.moby_to_instance.iter().enumerate() {
        assert_eq!(world.instance_to_moby[placement],Some(id));
        let source=&placements[placement];let m=world.objects.get(id).unwrap();
        assert_eq!(m.o_class as i32,source.base.o_class);assert_eq!(m.spawn_id,source.extra[0] as i16);
        if m.o_class==0 {assert!(m.pvars.is_empty());}
        else if source.base.pvar_index>=0 {
            assert_eq!(m.pvars.len(),rac2::gameplay::unrelocated_pvar(&gameplay,source.base.pvar_index).unwrap().unwrap().len());
            assert!(world.pvars.blocks[source.base.pvar_index as usize].is_none(),"pvar bytes must reside in live owner");
        }
    }
    let mut local_reads=0;
    for &(index,field) in &world.pvars.pointer_fields {
        for (id,&placement) in world.moby_to_instance.iter().enumerate() {
            if placements[placement].base.pvar_index!=index || placements[placement].base.o_class==0 {continue;}
            let raw=rc_formats::buf::Buf(&world.objects.get(id).unwrap().pvars).i32(field as usize).unwrap();
            assert_eq!(world.local_pvar_bytes(id,field as usize,0).is_ok(),raw>=0 && raw as usize<=world.objects.get(id).unwrap().pvars.len());
            local_reads+=1;
        }
    }
    assert!(local_reads>0);
    assert!(world.gc.iter().all(|s|*s==Default::default()));
    assert_eq!(world.objects.get(count).unwrap().state,0xfd,"original reserved class0 record is static and unavailable for reuse");
    let blocked=world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err();
    let rc_game2::world::TickError::Missing(missing)=blocked else {panic!("unexpected {blocked:?}")};
    assert!(missing.initializing);assert_eq!(registry.source_update(missing.class as i32),missing.source);
    let before=world.objects.get(missing.moby).unwrap().anim;
    assert_eq!(world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)),Err(rc_game2::world::TickError::Missing(missing)));
    assert_eq!(world.objects.get(missing.moby).unwrap().anim.t.to_bits(),before.t.to_bits());
    assert_eq!(world.tick(0,[0.;4],&loaded,|_,_,_|false),Err(rc_game2::world::TickError::NotInitialized));
    assert_eq!((missing.class,missing.source,missing.state),(0x12fa,0x383ce0,0));
    assert_eq!(world.objects.get(1).unwrap().state,0xfd,"original delete-only callback must execute");
    assert!(world.grid.cells_of(1).is_empty());
    for id in [2,3,4] {assert_eq!(world.objects.get(id).unwrap().state,1,"original link-wait init must execute");}
    let defs=rc_formats::particle_tex::parse_part_defs(wad.lump(Lump::CoreIndex).unwrap().unwrap(),core.header.part_defs_offset).unwrap();
    world.effects=Some(rc_game2::effects::Effects::new(defs));
    // Continue the SAME initialization cursor after providing the required
    // original effect resource. This fixture's RNG seed is explicit too.
    let next=world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err();
    let rc_game2::world::TickError::Missing(next_missing)=next else {panic!("unexpected {next:?}")};
    assert_eq!((next_missing.moby,next_missing.class,next_missing.source,next_missing.state),(23,0x132d,0x383eb8,0));
    let controller_seed=world.rng.state;
    assert!(!world.native_update(next_missing.moby,next_missing.source,0));
    assert_eq!(world.rng.state,controller_seed,"missing descriptor may not consume RNG");
    let controller_data=rc_game2::controller_132d::AranosControllerData::parse(0,overlay).unwrap();
    assert!(rc_game2::controller_132d::AranosControllerData::parse(1,overlay).is_err());
    let descriptor=*controller_data.descriptor();
    world.controller_data=Some(controller_data);
    let after_controller=world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err();
    assert!(matches!(after_controller,rc_game2::world::TickError::Missing(rc_game2::world::MissingUpdate {
        moby:24,class:0x1330,source:0x3846d0,state:0,initializing:true
    })),"initialization must advance to the actual next original callback: {after_controller:?}");
    let controller=world.controller_132d.as_ref().unwrap();
    assert_eq!(controller.owner,next_missing.moby);assert_eq!(controller.source_value,0x49);
    assert_eq!(world.objects.get(next_missing.moby).unwrap().state,1);
    assert_eq!(&world.objects.get(next_missing.moby).unwrap().pvars[..16],descriptor);
    assert_ne!(world.rng.state,controller_seed);
    let after_init_seed=world.rng.state;
    assert!(!world.native_update(next_missing.moby,next_missing.source,0),"required state1 query/draw remains unported");
    assert_eq!(world.rng.state,after_init_seed,"blocked query/draw must not replay initialization");
    println!("ORIGINAL_GC_CONTROLLER_INIT_CPU_PASS owner={} owned_descriptor=true shared_rng_before={controller_seed:08x} after={:08x} next_init={after_controller:?}; STATE1_QUERY_DRAW_NOT_IMPLEMENTED",next_missing.moby,world.rng.state);
    // Resume the SAME object24 cursor through the recovered services. These
    // explicit runtime globals are a branch fixture, not original New Game.
    let owner=24;let before_pvars=world.objects.get(owner).unwrap().pvars.clone();
    assert_eq!(rc_formats::buf::Buf(&before_pvars).i32(4).unwrap(),40);
    assert_eq!(rc_formats::buf::Buf(&before_pvars).i32(0x14).unwrap(),73);
    assert_eq!(world.pending_1330,Some(rc_game2::init_1330::Pending1330::RuntimeState));
    let mut state1330=rc_game2::state_1330::State1330 {byte_1a7a12:1,byte_1a7be8:0,
        words_139948:[0;29],halfword_152c26:0xbeef};
    state1330.words_139948[2]=7; // Hole-sensitive source scan, not RAC1 flag count.
    world.state_1330=Some(state1330);
    assert_eq!(world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err(),after_controller);
    let Some(rc_game2::init_1330::Pending1330::Teleport(request))=world.pending_1330 else {panic!("mandatory original teleport was not retained")};
    let cuboids=rc_game2::init_1330::cuboids(&gameplay).unwrap();let shape=cuboids[40];
    assert_eq!(request.owner,owner);assert_eq!(request.cuboid,40);
    assert_eq!(request.position.map(f32::to_bits),shape.matrix[3].map(f32::to_bits));
    assert_eq!(request.euler.map(f32::to_bits),[shape.euler[0],shape.euler[1],shape.euler[2],shape.unused_7c].map(f32::to_bits));
    assert_eq!((request.state,request.reset_camera),(0,true));
    let state1330=world.state_1330.as_mut().unwrap();
    assert_eq!(state1330.halfword_152c26,1);assert_eq!(state1330.byte_1a7be8,1);
    assert_eq!(state1330.words_139948[3],0x18);
    state1330.halfword_152c26=0xbeef;
    let globals=state1330.clone();
    assert_eq!(world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err(),after_controller);
    assert_eq!(world.state_1330.as_ref().unwrap(),&globals,"retry must not replay completed prefix writes");
    assert_eq!(world.objects.get(owner).unwrap().pvars,before_pvars);
    assert_eq!(world.objects.get(owner).unwrap().state,0);
    assert_eq!(world.rng.state,after_init_seed);
    assert_eq!(world.tick(0,[0.;4],&loaded,|_,_,_|false),Err(rc_game2::world::TickError::NotInitialized));
    println!("ORIGINAL_GC_1330_PREFIX_CPU_PASS actual_owner=24 cuboid=40 source=0x2b9e68 request={request:?}; INITIALIZER_CURSOR_REMAINS24 TELEPORT_CHAIN_UNPORTED");
    let receiver=missing.moby;assert_eq!(world.objects.get(receiver).unwrap().state,1);
    let e=world.effects.as_ref().unwrap();assert!(e.pool.count>0);
    let sprite=e.pool.live().find(|(_,r)|rc_game::particles::rec::u32(r,0x20)==receiver as u32+1).unwrap().0;
    assert_eq!(e.pool.recs[sprite][0],0x59);assert_eq!(e.pool.recs[sprite][9],0x34);
    assert_eq!(rc_game::particles::rec::i16(&e.pool.recs[sprite],10),-1);
    let camera=[247.,194.,52.,0.];world.effects.as_mut().unwrap().update(&world.objects,camera).unwrap();
    assert!(rc_game::particles::rec::v3(&world.effects.as_ref().unwrap().pool.recs[sprite],0x10).iter().all(|v|v.is_finite()));
    use rc_game::ps2v::Pf;
    assert_eq!(rc_formats::buf::Buf(&world.objects.get(receiver).unwrap().pvars).i32(0).unwrap(),0);
    // Direct delivery is explicitly injected; this is NOT a wrench swing,
    // spatial hit, health deduction or original destruction/effects claim.
    world.deliver_hit(receiver,Pf::ONE,Some(0),0x10000,[Pf::ZERO;4],[Pf::ZERO;4]).unwrap();
    assert_ne!(world.objects.get(receiver).unwrap().hit_slot,255);
    assert!(world.native_update(receiver,0x383ce0,1));
    assert_eq!(world.objects.get(receiver).unwrap().state,2);assert_eq!(world.objects.get(receiver).unwrap().hit_slot,255);
    assert!(!world.native_update(receiver,0x383ce0,2),"original impact/audio/debris may not be omitted");
    let rigid=rc_game2::collision::LiveRigidCollision::load(&core,&data).unwrap();
    let floor=world.instance_to_moby[164].unwrap();
    use rc_game::collision_query::{coll_line_m,QueryFlags};
    let (a,b)=([247.,198.,55.],[247.,198.,45.]);
    let raised=coll_line_m(&coll,Some(&rigid.scene(&world)),a,b,QueryFlags(2),Some(0)).unwrap();
    assert_eq!(raised.moby,Some(floor));assert!((raised.point[2]-50.33382).abs()<0.002);
    // Structural live-transform/grid checks, not a claim that the original
    // platform's gameplay callback moves it by this diagnostic displacement.
    let m=world.objects.get_mut(floor).unwrap();m.position[2]+=1.;
    let animation=&animations[classes.iter().position(|c|c.o_class==0xac1).unwrap()];
    rc_game::moby_update::scheduler::rebuild_matrix(m,Some(animation));world.grid.register(m);
    let moved=coll_line_m(&coll,Some(&rigid.scene(&world)),a,b,QueryFlags(2),Some(0)).unwrap();
    assert_eq!(moved.moby,Some(floor));assert!((moved.point[2]-raised.point[2]-1.).abs()<0.002);
    let grid=&mut world.grid;world.objects.delete(floor,2,|m|grid.remove(m)).unwrap();
    assert!(world.grid.cells_of(floor as u16).is_empty());
    let removed=coll_line_m(&coll,Some(&rigid.scene(&world)),a,b,QueryFlags(2),Some(0)).unwrap();
    assert_eq!(removed.moby,None);assert_eq!(removed.point[2],49.734375);
    println!("ORIGINAL_GC_LIVE_RIGID_CHAIN_PASS placement164_runtime={floor} floor={} moved={} removed_to_world={}; TEST_DISPLACEMENT_ONLY_NOT_ORIGINAL_MOVING_PLATFORM_BEHAVIOUR",raised.point[2],moved.point[2],removed.point[2]);
    println!("ORIGINAL_GC_LIVE_RECEIVER_CHAIN_PASS owner={receiver} sprite={sprite} type=89 explicit_hit_to_state2=true next_init={next_missing:?}; WRENCH_SPATIAL_HIT_AND_DESTRUCTION_UNIMPLEMENTED");
    println!("ORIGINAL_GC_NATIVE_WORLD_BOUNDARY spawned={count} rejected={} preceding_owners={} first_native_callback={missing:?}; EXPLICIT_SAVE_FIXTURE_NOT_NEW_GAME_CAPTURE LIVE_CLASS_GAMEPLAY_BLOCKED GPU_NOT_TESTED",placements.len()-count,world.preceding_owners.len());
    // Independently exercise the complete recovered allocation/ownership slice
    // with authentic loaded classes in THIS World. This is not an out-of-order
    // initializer bypass: no callback is reported supported or cursor advanced.
    let actor_data=rc_game2::actor_1330::Actor1330Data::parse(0,overlay).unwrap();
    assert!(rc_game2::actor_1330::Actor1330Data::parse(1,overlay).is_err());
    let actor_class=loaded.iter().find(|c|c.class()==0xd54).unwrap();
    let actor=world.objects.create(actor_class,0).unwrap();
    let from=world.objects.get(owner).unwrap().clone();
    world.connect_actor(actor,Some(owner),&loaded).unwrap();
    let connected=world.objects.get(actor).unwrap();
    assert_eq!(connected.parent,Some(owner));assert_eq!(connected.position,from.position);
    assert_eq!(connected.rotation,from.rotation);assert_eq!(connected.light,from.light);
    assert_eq!(connected.ambient,from.ambient);assert_eq!(connected.mode&7,0);
    assert_eq!((connected.update_dist,connected.draw_dist,connected.visible),(255,255,1));
    let word=0; // Explicit 1a7af8 fixture; not a recovered equipment/save value.
    let plan=actor_data.attachment_plan(word);
    let before_slots=world.objects.available_slots(0);
    let before_actor=world.objects.get(actor).unwrap().pvars.clone();
    assert!(world.rebuild_actor_attachments(actor,&actor_data,word,&[],0).is_err());
    assert_eq!(world.objects.available_slots(0),before_slots);
    assert_eq!(world.objects.get(actor).unwrap().pvars,before_actor);
    assert!(world.actor_attachments.iter().all(Option::is_none));
    let attachments=world.rebuild_actor_attachments(actor,&actor_data,word,&loaded,0).unwrap();
    assert_eq!(attachments.len(),plan.len());assert_eq!(attachments.len(),9);
    for (&id,allocation) in attachments.iter().zip(&plan) {
        let m=world.objects.get(id).unwrap();assert_eq!(m.o_class,allocation.class);
        assert_eq!(m.mode&0x8000!=0,allocation.flag_8000);assert_eq!(m.mode&0x124,0x124);
        assert_eq!(m.draw_dist,0x1ff);assert_eq!(m.pvars.len(),0x80);
        assert_eq!(rc_formats::buf::Buf(&m.pvars).i32(8).unwrap(),0x20);
        assert_eq!(world.local_pvar_bytes(id,8,0x60).unwrap().as_ptr(),m.pvars[0x20..].as_ptr());
        assert!(world.local_pvar_bytes(id,8,0x61).is_err());
        assert!(m.parent.is_none(),"360d28 does not create an invented parent link");
    }
    let second=world.rebuild_actor_attachments(actor,&actor_data,word,&loaded,0).unwrap();
    assert!(second.iter().all(|id|!attachments.contains(id)),"deleted slots cannot be reused before counter+2");
    for id in &attachments {let m=world.objects.get(*id).unwrap();assert_eq!((m.state,m.delete_tick),(0xfe,2));assert!(world.grid.cells_of(*id as u16).is_empty());}
    assert_eq!(world.actor_attachments[second.len()],None);
    assert_eq!(world.rng.state,after_init_seed,"allocation/connection consumes no RNG");
    assert_eq!(world.initialize(&loaded,|w,id,source|w.native_update(id,source,0)).unwrap_err(),after_controller);
    println!("ORIGINAL_GC_1330_ACTOR_ALLOCATION_CPU_PASS same_world=true owner={owner} actor={actor} attachment_classes={:?} delete_then_allocate=true actual_pvar_aliases=true; POSE_TELEPORT_FULL_CALLBACK_UNPORTED",plan.iter().map(|x|x.class).collect::<Vec<_>>());
}

#[test]
fn original_aranos_class_binding_and_wrench_sweep_windows() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let file=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();
    let w=LevelWad::parse(&file).unwrap();let overlay=w.lump(Lump::Overlay).unwrap().unwrap();
    let registry=rac2::classes::AranosClassUpdates::parse(0,overlay).unwrap();
    assert_eq!(registry.entries().len(),233);
    assert_eq!(registry.source_update(0x47),0x38d318);
    assert_eq!(registry.source_update(0x7fff),0);
    assert!(rac2::classes::AranosClassUpdates::parse(1,overlay).is_err());
    let table=rac2::melee::AranosMeleeTable::parse(0,overlay).unwrap();
    let mut active_ticks=0;
    for (i,row) in table.rows().iter().enumerate() {
        assert!(row.hit_from<=row.hit_to);
        let state=if i==3 {0x2a}else{0x13};
        let group=if i==3 {0}else{6};
        // Original comparisons include both endpoints and reject blending.
        for frame in 0..=80 {
            let active=rc_game2::items::wrench_sweep_window(state,group,false,frame as f32,row);
            assert_eq!(active,frame>=row.hit_from && frame<=row.hit_to);
            assert!(!rc_game2::items::wrench_sweep_window(state,group,true,frame as f32,row));
            active_ticks+=active as usize;
        }
        assert!(!rc_game2::items::wrench_sweep_window(0,0,false,row.hit_from as f32,row));
    }
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let world=rc_formats::collision::parse_collision(&core,&data).unwrap();
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let placed=rac2::gameplay::moby_instances(&gameplay).unwrap();
    let hero=&placed.iter().find(|p|p.base.o_class==0).unwrap().base;
    let crate_=&placed.iter().find(|p|p.base.o_class==0x1f4).unwrap().base;
    assert_eq!(registry.source_update(0x1f4),0x39c870);
    use rc_game2::targeting::{Query,Target};
    let position=|p:[f32;3]|[Pf::f(p[0]),Pf::f(p[1]),Pf::f(p[2]),Pf::ONE];
    let query=Query {hero_position:position(hero.position),hero_state:0,ground_z:Pf::f(hero.position[2]),
        yaw:Pf::f(hero.rotation[2]),range:Pf::f(4096.),yaw_cone:Pf::f(-1.),pitch_cone:Pf::f(-1.),gravity_frame:0};
    // Scalar scoring over original placements only: this is not the original
    // active-target list, spawn initialization, health or enemy AI execution.
    let candidate=Target {id:0,class:0x1f4,class_type:0,mode:0,position:position(crate_.position),creature_health:None};
    let score=rc_game2::targeting::score(&query,&candidate).unwrap().unwrap();assert!(score.to_f32().is_finite());
    assert!(score>query.range*Pf::b(0x3f4ccccd));
    let mut native_hero=Hero::spawn(hero.position,hero.rotation[2]);let pad=PadState::default();
    let env=Env {coll:&world,mobys:None,hero_moby:None,pad:&pad,cam_yaw:hero.rotation[2],cam_rows:native_hero.rows,mirror:false};
    let mut aim=rc_game2::targeting::MeleeAim::default();
    aim.assist(&mut native_hero,&env,&[],10,4.,-1.,-1.,0).unwrap();
    assert_eq!(aim.yaw,hero.rotation[2]);assert_eq!(aim.direction[2],0.);assert_eq!(aim.target,None);
    native_hero.stick=[0.8,0.];native_hero.stick_mag=0.8;
    aim.assist(&mut native_hero,&env,&[],10,4.,-1.,-1.,0).unwrap();
    assert_eq!(aim.aimed,1);assert_eq!(aim.yaw,native_hero.target_yaw);
    let held=aim.yaw;native_hero.rot[2]+=0.4;
    aim.assist(&mut native_hero,&env,&[],10,4.,-1.,-1.,0).unwrap();assert_eq!(aim.yaw,held);
    use rc_game::{collision_query::{coll_line,QueryFlags},ps2v::Pf};
    let v=|z|[Pf::f(247.),Pf::f(198.),Pf::f(z),Pf::ONE];
    let mut queries=0;
    let hit=rc_game2::items::sweep_lines(v(55.),v(45.),v(55.),v(55.),5,|a,b| {
        queries+=1;
        coll_line(&world,[a[0].to_f32(),a[1].to_f32(),a[2].to_f32()],
            [b[0].to_f32(),b[1].to_f32(),b[2].to_f32()],QueryFlags(2))
    }).expect("original world floor must be hit by the interpolated line");
    assert_eq!(queries,3);assert_eq!(hit.point[2],49.734375);
    println!("ORIGINAL_GC_CLASS_BINDINGS_PASS entries=233 wrench_source=0x38d318 inclusive_sweep_ticks={active_ticks} native_sweep_world_queries={queries} floor_z={}; GC_RECEIVER_DAMAGE_AND_CLASS_UPDATE_SCHEDULING_NOT_IMPLEMENTED",hit.point[2]);
}

#[test]
fn original_oozla_legacy_gc_class_preserves_geometry_and_native_conversion_semantics() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let file=iso.read_file(iso.find("/G/LEVEL1.WAD").unwrap()).unwrap();
    let w=LevelWad::parse(&file).unwrap();let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let blob=core.block(&data,"moby_class/2700").unwrap();let hash=rc_formats::sha1::sha1(blob);
    assert_eq!(blob[0xb],255,"original GC old-format marker");
    // The original GC converter explicitly consumes RAC1-format vertex tables.
    // This is a geometry oracle, never a RAC1 gameplay/scheduler invocation.
    let old=moby::parse_moby_class_for(Game::Rac1,blob).unwrap();
    let gc=moby::parse_moby_class_for(Game::Rac2,blob).unwrap();
    assert_eq!((gc.header.rac1_byte_b,gc.header.bangles,gc.header.mip_dist,gc.header.rac1_short_2e),(0,0,5,0));
    assert_eq!(gc.high_lod.len(),2);assert_eq!(gc.low_lod.len(),1);
    let mut vertices=0;let mut triangles=0;
    for (a,b) in old.high_lod.iter().chain(&old.low_lod).zip(gc.high_lod.iter().chain(&gc.low_lod)) {
        assert_eq!(a.vertices.len(),b.vertices.len());assert_eq!(a.triangles,b.triangles);
        assert_eq!(a.index_bytes,b.index_bytes);assert_eq!(a.texture_indices,b.texture_indices);
        assert!(!a.rgba_multipliers.is_empty());assert!(b.rgba_multipliers.is_empty(),"GC clears the old multiplier pointer");
        assert_eq!(b.unresolved_duplicates,0);
        for (a,b) in a.vertices.iter().zip(&b.vertices) {
            assert_eq!((a.raw,a.x,a.y,a.z,a.st,a.normal_azimuth,a.normal_elevation,a.id,a.duplicate),
                (b.raw,b.x,b.y,b.z,b.st,b.normal_azimuth,b.normal_elevation,b.id,b.duplicate));
            assert_eq!((a.skin.count,a.skin.joints,a.skin.weights),(b.skin.count,b.skin.joints,b.skin.weights));
        }
        vertices+=b.vertices.len();triangles+=b.triangles.len();
    }
    assert_eq!(hash,rc_formats::sha1::sha1(blob));
    println!("ORIGINAL_RC2_LEGACY_CLASS_CPU_PASS level=1 class=0xa8c packets=3 vertices={vertices} triangles={triangles}; GPU_AND_LEVEL_TRANSITION_NOT_TESTED");
}

#[test]
fn original_gc_pvar_links_follow_explicit_mapping_without_host_pointer_writes() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let file=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();
    let w=LevelWad::parse(&file).unwrap();let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let original_hash=rc_formats::sha1::sha1(&gameplay);
    let instances=rac2::gameplay::moby_instances(&gameplay).unwrap();let fixups=rac2::gameplay::pvar_fixups(&gameplay).unwrap();
    let copied=instances.iter().map(|i|i.base.pvar_index).chain(fixups.moby_links.iter().chain(&fixups.pointers).map(|f|f.0)).filter(|&i|i>=0).collect::<Vec<_>>();
    let first=fixups.moby_links.iter().find_map(|&(index,offset)| {
        let block=rac2::gameplay::unrelocated_pvar(&gameplay,index).unwrap().unwrap();
        let value=rc_formats::buf::Buf(block).i32(offset as usize).unwrap();
        (value>=0).then_some((index,offset,value as usize))
    }).expect("original Aranos has a nonnegative pvar moby link");
    // Explicit test-only spawn mask: one real referenced placement is absent.
    // This is a loader contract test, never the original level's spawn result.
    let mut next=0;let mapping=(0..instances.len()).map(|i|if i==first.2 {None}else{let id=next;next+=1;Some(id)}).collect::<Vec<_>>();
    let plan=rc_game2::pvars::PvarPlan::copy_and_link(&gameplay,&copied,&mapping).unwrap();
    let block=plan.blocks[first.0 as usize].as_ref().unwrap();
    assert_eq!(rc_formats::buf::Buf(block).i32(first.1 as usize).unwrap(),-1);
    assert!(rc_game2::pvars::PvarPlan::copy_and_link(&gameplay,&copied,&mapping[..mapping.len()-1]).is_err());
    assert_eq!(original_hash,rc_formats::sha1::sha1(&gameplay));
    let mut local_reads=0;
    for &(index,offset) in &plan.pointer_fields {
        let original=rac2::gameplay::unrelocated_pvar(&gameplay,index).unwrap().unwrap();
        if !fixups.moby_links.contains(&(index,offset)) {
            let o=offset as usize;assert_eq!(&original[o..o+4],&plan.blocks[index as usize].as_ref().unwrap()[o..o+4]);
            let relative=rc_formats::buf::Buf(original).i32(o).unwrap();
            if relative>=0 && (relative as usize)<original.len() {
                assert_eq!(plan.local_bytes(index as usize,o,1).unwrap(),&original[relative as usize..relative as usize+1]);local_reads+=1;
            }
        }
    }
    assert!(local_reads>0);assert!(plan.local_bytes(first.0 as usize,first.1 as usize,usize::MAX).is_err());
    println!("ORIGINAL_GC_PVAR_PLAN_CPU_PASS moby_link_records={} relative_pointer_fields={} bounded_local_reads={local_reads}; EXPLICIT_TEST_MAPPING_ONLY_NO_CLASS_INITIALIZERS_OR_GAMEPLAY",fixups.moby_links.len(),plan.pointer_fields.len());
}

#[test]
fn original_aranos_ground_combo_timing_and_native_pose_playback() {
    use rc_game::{hero::anim::AnimCtl,ps2v::Pf};
    use rc_game2::melee::{GroundCombo,EntryHistory,Recovery,JumpRequest,WrenchUse,WeaponDecision,wrench_request};
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let file=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();
    let w=LevelWad::parse(&file).unwrap();let overlay=w.lump(Lump::Overlay).unwrap().unwrap();
    let table=rac2::melee::AranosMeleeTable::parse(w.id as u32,overlay).unwrap();
    assert!(rac2::melee::AranosMeleeTable::parse(1,overlay).is_err(),"never apply level-0 addresses to another level");
    // Independently inspected original records differ from RAC1's combo timings.
    assert_eq!((table.rows()[0].chain_from,table.rows()[0].idle_after,table.rows()[1].idle_after),(25,26,15));
    assert_ne!(table.rows()[0].idle_after,rc_game::hero::melee::COMBO[0][6]);
    let c=WrenchUse {state:0,group:0,timer:30,frame:0.,blending:false,item_id:10,item_state:2,fire_mask:button::SQUARE,hand:Some((0x47,0)),hero_mode:0,item_disabled:0,change_gate:0,f348:0,f149a:0,f1e0:0,landed:0,descending:0,flip_kind:0,height:0.,f6ec:0,f594:0};
    let mut pad=PadState::default();pad.update(Some(&PadInput::neutral().press(button::SQUARE).bytes()),false);
    assert_eq!(wrench_request(&c,&table.rows()[0],&pad),WeaponDecision::Enter(0x13));
    assert_eq!(wrench_request(&WrenchUse{item_id:8,..c},&table.rows()[0],&pad),WeaponDecision::UnsupportedItem(8));
    assert_eq!(wrench_request(&WrenchUse{hand:None,..c},&table.rows()[0],&pad),WeaponDecision::None);
    assert_eq!(wrench_request(&WrenchUse{hand:Some((0x47,1)),..c},&table.rows()[0],&pad),WeaponDecision::None);
    assert_eq!(wrench_request(&WrenchUse{item_disabled:1,..c},&table.rows()[0],&pad),WeaponDecision::None);
    assert_eq!(wrench_request(&WrenchUse{state:7,group:4,timer:11,height:2.,..c},&table.rows()[4],&pad),WeaponDecision::Enter(0x14));
    assert_eq!(wrench_request(&WrenchUse{state:7,group:4,timer:11,height:2.,f6ec:1,..c},&table.rows()[4],&pad),WeaponDecision::None);
    assert_eq!(wrench_request(&WrenchUse{state:0x13,group:6,frame:24.9,..c},&table.rows()[0],&pad),WeaponDecision::None);
    assert_eq!(wrench_request(&WrenchUse{state:0x13,group:6,frame:25.,..c},&table.rows()[0],&pad),WeaponDecision::Enter(0x13));
    pad.update(Some(&PadInput::neutral().press(button::SQUARE|button::R1).bytes()),false);
    assert_eq!(wrench_request(&c,&table.rows()[0],&pad),WeaponDecision::Enter(0x15));
    assert_eq!(wrench_request(&WrenchUse{state:1,timer:21,..c},&table.rows()[0],&pad),WeaponDecision::ThrowFromLook);
    // The original 10-tick ground buffer is distinct from RAC1's 7 ticks.
    for _ in 0..8 {pad.update(Some(&PadInput::neutral().bytes()),false);}
    assert_eq!(wrench_request(&c,&table.rows()[0],&pad),WeaponDecision::Enter(0x13));
    for _ in 0..2 {pad.update(Some(&PadInput::neutral().bytes()),false);}
    assert_eq!(wrench_request(&c,&table.rows()[0],&pad),WeaponDecision::None);
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let ratchet=moby::parse_moby_class_for(Game::Rac2,core.block(&data,"moby_class/0000").unwrap()).unwrap();
    let seqs=(0..core.ratchet_seqs.len()).map(|i|core.block(&data,&format!("ratchet_seq/{i:03}")).map(|b|ma::parse_sequence_joints(b,0,Some(111)).unwrap())).collect();
    let class=ma::MobyAnimClass::new(&ratchet,seqs);let mut anim=RatchetAnim::new(&class);
    let boot_bytes=iso.read_file(iso.find("/SCUS_972.68").unwrap()).unwrap();
    let boot=rac2::boot::BootElf::parse(&boot_bytes).unwrap();
    let definition=rac2::items::AranosWrenchDefinition::parse(0,&boot,overlay).unwrap();
    assert_eq!((definition.definition_index,definition.class),(10,0x47));
    assert_eq!((definition.attachment_list,definition.ground_damage,definition.jump_damage),(0,1.,2.));
    let (chain,_)=rc_formats::gadget::joint_list(core.block(&data,"moby_class/0000").unwrap(),&ratchet.header,definition.attachment_list).unwrap();
    assert!(!chain.is_empty());
    let host_rows=[[1f32.to_bits(),0,0,0],[0,1f32.to_bits(),0,0],[0,0,1f32.to_bits(),0]];
    let host_position=[247.,194.,49.89];
    let blob=core.block(&data,&format!("moby_class/{:04}",definition.class)).unwrap();
    let wrench=moby::parse_moby_class_for(Game::Rac2,blob).unwrap();
    let wrench_class=ma::MobyAnimClass::new(&wrench,ma::parse_sequences(blob,&wrench).unwrap());
    let updates=rac2::classes::AranosClassUpdates::parse(0,overlay).unwrap();
    let slot=core.moby_classes.iter().position(|c|c.o_class==0x47).unwrap();
    let loaded=rc_game2::objects::LoadedClass::from_core(0,&core,slot,&wrench,&wrench_class,&updates).unwrap();
    assert!(rc_game2::objects::LoadedClass::from_core(1,&core,slot,&wrench,&wrench_class,&updates).is_err());
    assert_eq!(loaded.class(),0x47);assert_eq!(loaded.source_update(),Some(0x38d318));
    let mut objects=rc_game2::objects::Objects::new(Vec::new(),2);
    objects.free_slot_pass(0);assert_eq!(objects.free_slots(),2);
    let first=objects.create(&loaded,0).unwrap();
    let m=objects.get(first).unwrap();assert_eq!((m.o_class,m.hit_slot,m.alpha,m.group),(0x47,255,128,-1));
    let seq0=wrench_class.sequence(0).unwrap();assert_eq!((m.b7c,m.anim.trigger_count),(seq0.header.loop_sound,seq0.header.trigger_count));
    assert_eq!(m.pvars,vec![0;128]);assert_eq!(m.scale,wrench.header.scale);
    assert_eq!(m.update_fn,Some(0x38d318));assert!(m.has_class);
    objects.get_mut(first).unwrap().pvars[0]=93;
    let mut removed=false;
    objects.delete(first,10,|m|{assert_eq!((m.state,m.delete_tick),(254,12));removed=true;}).unwrap();assert!(removed);
    assert_eq!(objects.create(&loaded,11),Some(1));
    let reused=objects.create(&loaded,12).unwrap();assert_eq!(reused,first);
    assert_eq!(objects.get(reused).unwrap().pvars,vec![0;128]);
    use rc_game2::items::{HandSlot,CreateContext,CreateResult};
    let mut slot=HandSlot::default();let mut hand_objects=rc_game2::objects::Objects::new(Vec::new(),1);
    let c=CreateContext {frame:1,counter:80,request:0,target:0,wrench_flag:0,saved_item:0,ammo:0,hero_light:0x1234,hero_ambient:[40,50,60,0]};
    assert_eq!(slot.create_wrench(CreateContext{frame:0,..c},&definition,&loaded,&wrench_class,&mut hand_objects).unwrap(),CreateResult::Wait);
    assert_eq!(slot.create_wrench(CreateContext{saved_item:12,..c},&definition,&loaded,&wrench_class,&mut hand_objects).unwrap(),CreateResult::OtherItemRequired(12));
    assert_eq!(slot.create_wrench(CreateContext{request:12,..c},&definition,&loaded,&wrench_class,&mut hand_objects).unwrap(),CreateResult::Wait);
    assert_eq!(slot.create_wrench(c,&definition,&loaded,&wrench_class,&mut hand_objects).unwrap(),CreateResult::Created(0));
    assert_eq!((slot.item,slot.item_id,slot.ownership,slot.fire_mask),(Some(0),10,2,button::SQUARE));
    assert_eq!((slot.disabled,slot.modifier_joints),(0,[-1;3]));
    let m=hand_objects.get(0).unwrap();assert_eq!((m.draw_dist,m.visible,m.light,m.ambient),(32,1,0x1234,[40,50,60,0]));
    assert_eq!(m.anim.seq_b,1);assert_eq!(m.pvars,vec![0;128]);
    assert_eq!(slot.create_wrench(c,&definition,&loaded,&wrench_class,&mut hand_objects).unwrap(),CreateResult::Wait);
    let mut hand=rc_game2::items::WrenchAnimation::initial_animation(&wrench_class).unwrap();
    assert_eq!(hand.state.seq_b,1);
    let mut combo=GroundCombo::new(&table);let mut pose_count=0;
    for step in 0..3 {
        let history=EntryHistory {previous_state:if step==0 {0}else{0x13},older_state:0,timer:0,saved_timer:0};
        let entry=combo.enter(history,20.0);assert_eq!(entry.step,step);
        assert_eq!((entry.hero_sequence,entry.wrench_sequence,entry.start_frame),(0x17+step as u8,3+step as u8,step as i32));
        GroundCombo::set_hero_animation(entry,&mut anim.ctl(&class));
        hand.combo(&wrench_class,entry).unwrap();assert_eq!(hand.state.seq_b,entry.wrench_sequence);
        let mut recovered=false;
        for _ in 0..100 {
            let mut ctl=anim.ctl(&class);ctl.advance(Pf::ONE);let v=ctl.view();
            assert!(v.frame.is_finite());
            assert_eq!(v.seq_b,entry.hero_sequence);
            let bones=ma::evaluate_with_snapshot(&class,&anim.state,anim.snapshot.as_ref());assert_eq!(bones.len(),111);
            assert!(bones.iter().flatten().flatten().all(|x|x.is_finite()));pose_count+=1;
            hand.advance(&wrench_class);let hand_pose=hand.pose(&wrench_class);
            assert_eq!(hand_pose.len(),10);assert!(hand_pose.iter().flatten().flatten().all(|x|x.is_finite()));
            let partial=anim.ctl(&class).eval_chains(&[chain.as_slice()])[0];
            let attached=rc_game2::items::attach_wrench(&partial,&host_rows,host_position,ratchet.header.scale);
            assert!(attached.position.iter().all(|x|x.is_finite()));
            // Independent translation-only kernel: small rounding differences
            // are expected because the original multi-list matrix rotates first.
            let point=ma::bone_points(&[partial[3].map(f32::to_bits)],&host_rows,host_position,ratchet.header.scale)[0];
            for k in 0..3 {assert!((attached.position[k]-f32::from_bits(point[k])).abs()<0.002);}
            for k in 0..3 {let norm=(0..3).map(|j|f32::from_bits(attached.rows[j][k]).powi(2)).sum::<f32>();assert!((norm-1.).abs()<0.002);}
            if combo.recovery(v.frame,v.blending(),v.flags as u16,0.)==Recovery::Idle {recovered=true;break;}
        }
        assert!(recovered,"original combo step {step} never reached recovery");
        let end=combo.timing().idle_after as f32;
        assert_eq!(combo.recovery(end,false,0,0.81),Recovery::Continue);
        assert_eq!(combo.recovery(end+0.1,false,0,0.81),Recovery::Walk);
        assert_eq!(combo.recovery(end+5.,false,0,0.),Recovery::Continue);
        assert_eq!(combo.recovery(end+5.1,false,0,0.),Recovery::Idle);
        assert_eq!(combo.recovery(80.,true,2,1.),Recovery::Continue);
        let mut pad=PadState::default();pad.update(Some(&PadInput::neutral().press(button::CROSS).bytes()),false);
        let frame=combo.timing().jump_after as f32+1.;
        assert_eq!(combo.jump_request(frame,false,&pad),Some(JumpRequest::Plain));
        assert_eq!(combo.jump_request(frame,true,&pad),None);
        assert!(combo.target_speed(0.,false,1.)>0.);
        assert_eq!(combo.target_speed(20.,false,1.),0.);
    }
    let recent=EntryHistory {previous_state:0,older_state:0x13,timer:5,saved_timer:21};
    assert_eq!(combo.enter(recent,100.).step,0,"recent third swing wraps");
    assert_eq!(combo.enter(recent,100.).step,1,"GC short return can chain through a different state");
    assert_eq!(combo.enter(EntryHistory{saved_timer:22,..recent},100.).step,0,"27 ticks closes short return");
    println!("ORIGINAL_RC2_GROUND_COMBO_TIMING_CPU_PASS original_table=0x25d910 sequences=0x17..0x19 poses={pose_count} original_hand_attachment_matrices={pose_count} native_gc_dynamic_creation_and_reuse=true; FULL_ITEM_TICK_RECEIVER_DAMAGE_MELEE_PHYSICS_GPU_NOT_IMPLEMENTED_OR_TESTED");
}

#[test]
fn original_aranos_environment_and_frontend_contract() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let read=|path:&str|iso.read_file(iso.find(path).unwrap()).unwrap();
    let hdr=read("/RC2.HDR");let toc=rac2::toc::Toc::parse(&hdr).unwrap();
    let misc=read("/G/MISC.WAD");let range=toc.misc_part(rac2::toc::MISC_FRONT_END).unwrap();
    let blob=rac2::front_end::FrontEnd::parse(wad::decompress(&misc[range.bytes()]).unwrap()).unwrap();
    assert_eq!(blob.textures().unwrap().len(),3);
    for i in 0..3 {let p=blob.splash(i).unwrap();assert_eq!(p.rgba.len(),p.width as usize*p.height as usize*4);}
    let hud=read("/G/HUD.WAD");let sprites=rac2::gui_sprite::parse(&hud[toc.gui_block().unwrap().bytes()]).unwrap();
    let elf=read("/SCUS_972.68");let boot=rac2::boot::BootElf::parse(&elf).unwrap();
    let assets=rc_game2::front_end::Assets{glyphs:boot.glyphs(rac2::boot::GLYPHS).unwrap(),glyphs_2:boot.glyphs(rac2::boot::GLYPHS_2).unwrap(),strings:blob.strings(0).unwrap(),main_menu:boot.menu(rac2::boot::MAIN_MENU).unwrap(),sprites};
    let new_game=assets.main_menu.iter().position(|r|r.kind==4).expect("original New Game row");
    let mut front=rc_game2::front_end::FrontEnd::new(7);
    for _ in 0..500 {if front.movie().is_some(){break;}front.tick(0,false,&assets);}
    assert_eq!(front.movie(),Some(rc_game2::front_end::MOVIE_LOGO));
    front.tick(0,true,&assets);assert!(front.movie().is_some());
    front.tick(0,false,&assets);front.tick(rc_game2::front_end::button::START,false,&assets);
    for _ in 0..3 {front.tick(0,false,&assets);}
    for _ in 0..new_game {front.tick(rc_game2::front_end::button::DOWN,false,&assets);front.tick(0,false,&assets);}
    let draws=front.draw(&assets);assert!(!draws.is_empty());
    let requests=front.tick(rc_game2::front_end::button::CROSS,false,&assets);
    assert!(requests.iter().any(|r|matches!(r,rc_game2::front_end::Request::Row{kind:4,..})));
    let file=read("/G/LEVEL0.WAD");let w=LevelWad::parse(&file).unwrap();
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let fragments=rc_formats::tfrag::parse_level_tfrags(&core,&data).unwrap();
    assert_eq!(fragments.len(),833);
    for t in &fragments {for lod in 0..3 {rc_formats::tfrag::tfrag_triangles(t,lod).unwrap();}}
    let textures=rc_formats::texture::parse_textures(&core,&data,w.lump(Lump::GsRam).unwrap().unwrap()).unwrap();
    let ties=rc_formats::tie::parse_level_ties(&core,&data).unwrap();
    let shrubs=rc_formats::shrub::parse_level_shrubs(&core,&data).unwrap();
    let mobys=moby::parse_level_mobys(&core,&data).unwrap();
    let mobs=rac2::gameplay::moby_instances(&gameplay).unwrap();
    assert_eq!(mobys.len(),187);assert_eq!(mobs.len(),472);
    let instances=rac2::gameplay::tie_instances(&gameplay).unwrap();
    println!("ORIGINAL_RC2_ENVIRONMENT_FRONTEND_CPU_PASS fragments={} textures={} tie_classes={} tie_instances={} shrub_classes={} gui_sprites={} main_menu_rows={} menu_quads={} moby_classes=187 moby_instances=472 NewGame_request=true; GPU_NOT_TESTED",fragments.len(),textures.len(),ties.len(),instances.len(),shrubs.len(),assets.sprites.len(),assets.main_menu.len(),draws.len());
}

#[test]
fn original_aranos_raised_placement_floor_and_jump() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let read=|p:&str|iso.read_file(iso.find(p).unwrap()).unwrap();
    let file=read("/G/LEVEL0.WAD");let w=LevelWad::parse(&file).unwrap();
    let data=wad::decompress(w.lump(Lump::CoreData).unwrap().unwrap()).unwrap();
    let core=level::parse_level_core_for(Game::Rac2,w.lump(Lump::CoreIndex).unwrap().unwrap(),data.len()).unwrap();
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let instances=rac2::gameplay::moby_instances(&gameplay).unwrap().into_iter().map(|m|m.base).collect::<Vec<_>>();
    let placements=rc_game2::collision::PlacementCollision::load(&core,&data,&instances).unwrap();
    let scene=placements.scene();let world=rc_formats::collision::parse_collision(&core,&data).unwrap();
    use rc_game::collision_query::{coll_line,coll_line_m,coll_capsule_m,QueryFlags};
    let (a,b)=([247.,198.,55.],[247.,198.,45.]);
    let old=coll_line(&world,a,b,QueryFlags(2)).unwrap();
    let raised=coll_line_m(&world,Some(&scene),a,b,QueryFlags(2),Some(0)).unwrap();
    assert_eq!(old.point[2],49.734375);
    let id=raised.moby.expect("raised surface must come from original placement geometry");
    assert_eq!(id,164);assert_eq!(instances[id].o_class,0xac1);
    assert!((raised.point[2]-50.33382).abs()<0.002);
    let body=coll_capsule_m(&world,Some(&scene),[247.,198.,50.5],0.1,0.45,QueryFlags(0x24),Some(0)).unwrap();
    assert_eq!(body.moby,Some(id));assert!(body.pushed_centre.unwrap().iter().all(|v|v.is_finite()));
    let ratchet=moby::parse_moby_class_for(Game::Rac2,core.block(&data,"moby_class/0000").unwrap()).unwrap();
    let sequences=(0..core.ratchet_seqs.len()).map(|i|core.block(&data,&format!("ratchet_seq/{i:03}")).map(|b|ma::parse_sequence_joints(b,0,Some(111)).unwrap())).collect();
    let class=ma::MobyAnimClass::new(&ratchet,sequences);
    let run=|with_placements:bool| {
        let mut hero=Hero::spawn([247.,198.,51.2],0.);let mut anim=RatchetAnim::new(&class);let mut pad=PadState::default();
        let mut camera=FollowCamera::new(&CamInput{hero:&hero,coll:Some(&world),mobys:with_placements.then_some(&scene),hero_moby:Some(0),rstick:[0.;2],yaw_normal:true,pitch_normal:true});
        let mut max_jump=f32::NEG_INFINITY;
        for tick in 0..240 {
            let input=if tick==90 {PadInput::neutral().press(button::CROSS)}else{PadInput::neutral()};
            pad.update(Some(&input.bytes()),false);
            let env=Env{coll:&world,mobys:with_placements.then_some(&scene),hero_moby:Some(0),pad:&pad,cam_yaw:camera.yaw(),cam_rows:camera.rows,mirror:false};
            assert_eq!(hero_update(&mut hero,&env,&mut anim.ctl(&class)),HeroTick::Ran);
            camera.update(&CamInput{hero:&hero,coll:Some(&world),mobys:with_placements.then_some(&scene),hero_moby:Some(0),rstick:[0.;2],yaw_normal:true,pitch_normal:true});
            assert!(camera.pos.iter().all(|v|v.is_finite()));
            if tick>=90 {max_jump=max_jump.max(hero.pos[2]);}
            assert!(hero.pos.iter().all(|v|v.is_finite()));
        }
        (hero,max_jump)
    };
    let (before,_)=run(false);let (after,max_jump)=run(true);
    assert!(before.grounded());assert!(after.grounded());
    assert!((before.pos[2]-old.point[2]).abs()<0.03);
    assert!((after.pos[2]-raised.point[2]).abs()<0.03);
    assert!(max_jump>raised.point[2]+0.5);
    println!("ORIGINAL_RC2_RAISED_COLLISION_CPU_PASS placement={id} class=0xac1 registered={} old_ground={} corrected_ground={} jump_apex={max_jump}; EXACT_DEVICE_SPOT_AND_PS2_ORACLE_NOT_VERIFIED DEVICE_RETEST_REQUIRED",placements.registered,before.pos[2],after.pos[2]);
}

#[test]
fn original_frontend_navigation_sound_renders_through_shared_native_mixer() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: private original ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let read=|p:&str|iso.read_file(iso.find(p).unwrap()).unwrap();
    let hdr=read("/RC2.HDR");let toc=rac2::toc::Toc::parse(&hdr).unwrap();let misc=read("/G/MISC.WAD");
    let blob=rac2::front_end::FrontEnd::parse(wad::decompress(&misc[toc.misc_part(rac2::toc::MISC_FRONT_END).unwrap().bytes()]).unwrap()).unwrap();
    let elf=read("/SCUS_972.68");let boot=rac2::boot::BootElf::parse(&elf).unwrap();
    let defs=boot.frontend_sound_defs().unwrap();assert_eq!(defs.len(),20);assert_eq!(defs[3].index,2);
    let mut audio=rc_game2::front_audio::FrontAudio::load(&blob,&boot).unwrap();
    assert_eq!(audio.bank_sound_count(),135);
    assert!(audio.play(u32::MAX).is_err());
    let mut frame=Vec::new();let mut nonzero=0usize;
    audio.render(&mut frame);assert!(frame.iter().all(|s|*s==[0,0]));
    assert!(audio.play(3).unwrap()>0);
    for _ in 0..60 {audio.render(&mut frame);assert_eq!(frame.len(),800);nonzero+=frame.iter().filter(|s|**s!=[0,0]).count();}
    assert!(nonzero>100);assert_eq!(audio.unsupported_grains(),0);
    println!("ORIGINAL_RC2_MENU_SOUND_CPU_PASS request=3 bank_sound=2 bank_sounds=135 samples=48000 nonzero={nonzero} unsupported_grains=0; DEVICE_OUTPUT_GAIN_AND_ORIGINAL_ORACLE_NOT_TESTED");
}
