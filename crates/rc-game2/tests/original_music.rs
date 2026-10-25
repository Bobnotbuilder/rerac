//! Optional private original ISO -> GC music request -> native stereo SPU tests.
//! CPU PCM evidence only; no audio-device output, iPhone or complete level claim.
use rc_formats::{iso9660::IsoImage,rac2::level::{LevelWad,Lump,Part},wad};
use rc_game::audio::{music::{Player,Streams},voices::{VoiceManager,VoiceUse},Spu};
use rc_game2::level_music::{MusicWad,MusicPlayer,StereoStream};
use std::sync::Arc;

#[test]
fn original_aranos_teleport_track4_takes_original_missing_entry_guard() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {
        eprintln!("NOT_TESTED: private RC2_ORIGINAL_ISO unavailable; original music guard not exercised");return;
    };
    let iso=IsoImage::open(path).unwrap();
    let level=iso.read_file(iso.find("/G/LEVEL0.WAD").unwrap()).unwrap();let w=LevelWad::parse(&level).unwrap();
    let sections=rc_formats::font::parse_overlay_sections(w.lump(Lump::Overlay).unwrap().unwrap()).unwrap();
    for (pc,len,hash) in [(0x32eb88,0xf8,"963328ce37c95828be2ddc9d621734a71520b36b"),
        (0x32fcc0,0x3c,"0fb34f3986a34fe0697f9cd4659676111f29e277")] {
        assert_eq!(rc_formats::sha1::hex(&rc_formats::sha1::sha1(rc_formats::font::read_overlay(&sections,pc,len).unwrap())),hash);
    }
    let gameplay=wad::decompress(w.part_bytes(Part::Gameplay).unwrap().unwrap()).unwrap();
    let cuboids=rc_game2::init_1330::cuboids(&gameplay).unwrap();let pos=cuboids[40].matrix[3];
    let samples=rc_game2::env_sample::read(&gameplay).unwrap();
    let i=rc_game2::env_sample::nearest(&samples,[pos[0],pos[1],pos[2]]).unwrap();
    assert_eq!(i,4);assert_eq!(samples[i].music_track,4);
    let entry=iso.find("/G/AUDIO0.WAD").unwrap();let bytes=iso.read_file(entry).unwrap();
    assert_eq!(rc_formats::sha1::hex(&rc_formats::sha1::sha1(&bytes)),"df44da7eb84286bdfdabc151a111c95025889b0d");
    let wad=MusicWad::parse(&bytes).unwrap();assert_eq!(wad.entry(4).unwrap().sector,0);assert_eq!(wad.entry(5).unwrap().sector,0);
    // Explicit fixture state, not a recovered original New Game. The source's
    // missing-sector guard must preserve every field and emit no native stream.
    let mut player=MusicPlayer {player:Player {track:9,vol:321,state:7,..Player::default()},source_14:0x12345678};
    let before=player;
    assert!(player.request_start(&wad,entry.lba,samples[i].music_track as i32,1,0x400).unwrap().is_none());
    assert_eq!(player,before);
    println!("ORIGINAL_GC_TELEPORT_MUSIC_GUARD_PASS cuboid=40 sample=4 track=4 original_sector=0 player_unchanged=true no_stream_request=true source_hashes_verified=true; NOT_NEW_GAME_NOT_IN_APP_AUDIO");
}

#[test]
fn original_aranos_available_stereo_music_reuses_native_streams_and_spu() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {
        eprintln!("NOT_TESTED: private RC2_ORIGINAL_ISO unavailable; original stereo music not exercised");return;
    };
    let iso=IsoImage::open(path).unwrap();let entry=iso.find("/G/AUDIO0.WAD").unwrap();let bytes=iso.read_file(entry).unwrap();
    assert_eq!(rc_formats::sha1::hex(&rc_formats::sha1::sha1(&bytes)),"df44da7eb84286bdfdabc151a111c95025889b0d");
    let wad=MusicWad::parse(&bytes).unwrap();
    let mut player=MusicPlayer {player:Player::default(),source_14:37};
    // Deliberately request a known available original track0 for transport
    // validation; do NOT substitute it for teleport's legitimately absent4.
    let request=player.request_start(&wad,entry.lba,0,1,0x400).unwrap().unwrap();
    assert_eq!(request.sectors,[entry.lba.wrapping_add(3),entry.lba.wrapping_add(0xb9)]);
    assert_eq!(request.channels[0].name,"rc2_l00_main_al");assert_eq!(request.channels[1].name,"rc2_l00_main_ar");
    assert_eq!(request.channels[0].rate,44100);assert_eq!(request.channels[1].rate,44100);
    assert_eq!((player.player.handle,player.player.state,player.source_14),(-1,1,10));
    let(mut streams,mut vm,mut spu)=(Streams::default(),VoiceManager::default(),Spu::new(Arc::from([])));
    let pair=StereoStream::start(&request,&mut streams,&mut vm,&mut spu).unwrap();
    assert_ne!(pair.handles[0],pair.handles[1]);player.complete_start(pair.handles[0]);
    assert_eq!(player.player.state,8,"original next state requires separate stereo body queue");
    assert!(player.require_body_queue_adapter().is_err(),"unported body step must fail closed");
    let(mut nonzero,mut different,mut peaks)=([0usize;2],0usize,[0i32;2]);
    for _ in 0..96000 {
        let p=spu.mix();if p[0]!=p[1]{different+=1;}
        for c in 0..2 {if p[c]!=0{nonzero[c]+=1;}peaks[c]=peaks[c].max((p[c]as i32).abs());}
    }
    assert!(nonzero[0]>0&&nonzero[1]>0&&different>0);
    pair.stop(&mut streams,&mut vm,&mut spu);
    assert_eq!(vm.voices.iter().filter(|v|matches!(v.owner,VoiceUse::Stream {..})).count(),0);
    println!("ORIGINAL_GC_STEREO_NATIVE_PCM_PASS track=0 input_rate=44100 output_rate=48000 frames=96000 nonzero={nonzero:?} distinct_stereo={different} peaks={peaks:?} actual_stream_voices=2; EXPERIMENTAL_IOP_TRANSPORT_GAIN_TIMING_UNVERIFIED BODY_QUEUE_UNPORTED NO_AUDIO_DEVICE_OR_IPHONE_TEST");
}
