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
    use rc_game::{collision_query::{coll_line,QueryFlags},ps2v::Pf};
    let v=|z|[Pf::f(247.),Pf::f(198.),Pf::f(z),Pf::ONE];
    let mut queries=0;
    let hit=rc_game2::items::sweep_lines(v(55.),v(45.),v(55.),v(55.),5,|a,b| {
        queries+=1;
        coll_line(&world,[a[0].to_f32(),a[1].to_f32(),a[2].to_f32()],
            [b[0].to_f32(),b[1].to_f32(),b[2].to_f32()],QueryFlags(2))
    }).expect("original world floor must be hit by the interpolated line");
    assert_eq!(queries,3);assert_eq!(hit.point[2],49.734375);
    println!("ORIGINAL_GC_CLASS_BINDINGS_PASS entries=233 wrench_source=0x38d318 inclusive_sweep_ticks={active_ticks} native_sweep_world_queries={queries} floor_z={}; GC_MOBY_DAMAGE_AND_NATIVE_CLASS_LIFECYCLE_NOT_IMPLEMENTED",hit.point[2]);
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
    println!("ORIGINAL_RC2_GROUND_COMBO_TIMING_CPU_PASS original_table=0x25d910 sequences=0x17..0x19 poses={pose_count} original_hand_attachment_matrices={pose_count}; ITEM_OWNERSHIP_RECEIVER_DAMAGE_PHYSICS_GPU_NOT_IMPLEMENTED_OR_TESTED");
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
