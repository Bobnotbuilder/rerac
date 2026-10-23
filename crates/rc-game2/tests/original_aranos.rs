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
        let env=Env{coll:&collision,pad:&pad,cam_yaw:camera.yaw(),cam_rows:camera.rows,mirror:false};
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
        let env=Env{coll:&collision,pad:&pad,cam_yaw:camera.yaw(),cam_rows:camera.rows,mirror:false};
        assert_eq!(hero_update(&mut hero,&env,&mut anim.ctl(&class)),HeroTick::Ran);
        camera.update(&camera_input(&hero,&collision,[0.;2]));
    }
    let delta=((hero.pos[0]-before[0]).powi(2)+(hero.pos[1]-before[1]).powi(2)).sqrt();
    assert!(delta>0.01,"original movement path did not move");
    println!("ORIGINAL_RC2_CPU_PASS joints=111 textures=4 pose_frames={pose_frames} jump_rise={} grounded_ticks={grounded} movement_distance={delta} spawn={:?} final={:?} camera={:?}; GPU_NOT_TESTED DEVICE_NOT_TESTED LEVEL_COMPLETION_NOT_TESTED",max_z-launch_z,spawn.position,hero.pos,camera.pos);
}

fn camera_input<'a>(hero: &'a Hero, coll: &'a rc_formats::collision::Collision, rstick: [f32;2]) -> CamInput<'a> {
    CamInput {hero,coll:Some(coll),rstick,yaw_normal:true,pitch_normal:true}
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
