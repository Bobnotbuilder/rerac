//! Original RC2 frontend movie compatibility through the existing native video decoder.
//! No movie bytes/images are emitted. This is CPU decoding, not playback or audio output.
use rc_formats::{iso9660::IsoImage,rac2::toc};

#[test]
fn original_rac2_logo_and_title_movies_decode() {
    let Some(path)=std::env::var_os("RC2_ORIGINAL_ISO") else {eprintln!("NOT_TESTED: RC2_ORIGINAL_ISO unavailable");return;};
    let iso=IsoImage::open(path).unwrap();let hdr=iso.read_file(iso.find("/RC2.HDR").unwrap()).unwrap();let t=toc::Toc::parse(&hdr).unwrap();
    for id in 0xb2..=0xbb {
        let row=toc::movie_row(id).unwrap();let m=t.movie(row).unwrap().unwrap();
        let (mut at,mut remaining)=(m.sector as u64*2048,m.bytes as u64);let mut bytes=Vec::new();
        for path in toc::MOVIE_FILES {
            let f=iso.find(&format!("/{path}")).unwrap();let size=f.size as u64;
            if at>=size {at-=size;continue;}
            let n=remaining.min(size-at);bytes.extend(iso.read_bytes(f.lba as u64*2048+at,n).unwrap());remaining-=n;at=0;
            if remaining==0 {break;}
        }
        assert_eq!(remaining,0);let mut movie=rc_video::Movie::open(&bytes,0).unwrap();
        let mut rgba=Vec::new();
        for _ in 0..3 {
            let frame=movie.next_frame().unwrap().expect("original movie frame");
            assert_eq!((frame.width,frame.height),(512,416));
            rc_video::movie::frame_to_rgba(&frame,&mut rgba);assert_eq!(rgba.len(),512*416*4);
        }
        println!("ORIGINAL_RC2_MOVIE_CPU_PASS id={id:#x} frames=3 dimensions=512x416 fps={} audio_samples={} GPU_NOT_TESTED AUDIO_OUTPUT_NOT_TESTED",movie.fps(),movie.audio.as_ref().map_or(0,Vec::len));
    }
}
