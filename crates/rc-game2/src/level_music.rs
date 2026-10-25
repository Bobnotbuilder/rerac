//! GC stereo music start32eb88 and callback32fcc0, using the existing native
//! Player/Streams/VoiceManager/SPU. The RAC1 EE Music::tick must NOT drive this:
//! GC adjacent table entries are left/right channels, not start/body tracks.
//! Only the initial request/callback is ported; state8 requires32ec80's separate
//! stereo body queue. Native hard-panned SPU transport is experimental: original
//! IOP gain/stream timing and complete GC EE update still need reference proof.
use rc_formats::{buf::{invalid,Buf,Result}};
use rc_game::audio::{music::{MusicTrack,Player,PlayerId,Streams,StreamCallback,StreamCommand},
    voices::{VoiceManager,VoiceUse},Spu};
use std::sync::Arc;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ChannelEntry {pub sector:u32,pub bytes:u32}

/// Original per-level AUDIO header copied by boot133aa0 to150838.
/// The source disc bytes stay immutable; absolute disc-sector binding is host
/// input, not a fabricated overwrite of the header's+4 runtime word.
pub struct MusicWad<'a> {file:&'a[u8]}
impl<'a> MusicWad<'a> {
    pub fn parse(file:&'a[u8])->Result<Self> {
        let b=Buf(file);
        if b.u32(0)?!=0x1018 {return invalid("GC level AUDIO header must match original1018-byte contract");}
        b.check(0,0x1018,"GC level AUDIO header")?;Ok(Self {file})
    }
    pub fn entry(&self,track:usize)->Result<ChannelEntry> {
        // Music table stops before the three auxiliary rows at+1000. Source
        // indexed reads are bounded here; unknown rows are never invented.
        if track>=(0x1000-8)/8 {return invalid("GC music channel index exceeds original table");}
        let b=Buf(self.file);let p=8+track*8;
        Ok(ChannelEntry {sector:b.u32(p)?,bytes:b.u32(p+4)?})
    }
    fn channel(&self,track:usize)->Result<Option<Arc<MusicTrack>>> {
        let entry=self.entry(track)?;
        if entry.sector==0 {return Ok(None);}
        if entry.bytes==0 {return invalid("GC music channel has no VAG bytes");}
        let offset=(entry.sector as usize).checked_mul(2048).ok_or_else(||rc_formats::FormatError::Invalid("GC music sector overflow".into()))?;
        let bytes=Buf(self.file).sub(offset,entry.bytes as usize,"GC music VAG")?;
        Ok(Some(Arc::new(MusicTrack::from_vag(bytes.bytes())?)))
    }
    pub fn stereo(&self,track:usize)->Result<Option<[Arc<MusicTrack>;2]>> {
        let Some(left)=self.channel(track)? else {return Ok(None);};
        let right=self.channel(track+1)?.ok_or_else(||rc_formats::FormatError::Invalid("GC stereo right channel absent".into()))?;
        if left.rate!=right.rate || left.body.len()!=right.body.len() {
            return invalid("GC stereo channels differ in rate or decoded sample duration");
        }
        Ok(Some([left,right]))
    }
}

/// Existing native Player plus the source's independent+14 word. No Default:
/// caller must bind original runtime state rather than assume New Game zeros.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MusicPlayer {pub player:Player,pub source_14:i32}

#[derive(Clone,Debug)]
pub struct StartRequest {
    pub track:i16,pub flags:u16,pub volume:i16,
    pub sectors:[u32;2],pub channels:[Arc<MusicTrack>;2],
}
impl MusicPlayer {
    /// Original32eb88 order/gates and owned player writes. A returned request
    /// is pending, not a successful IOP call. Native transport must execute and
    /// return a real handle before complete_start is called.
    pub fn request_start(&mut self,wad:&MusicWad<'_>,disc_sector:u32,track:i32,flags:i32,volume:i32)->Result<Option<StartRequest>> {
        if track<0 || self.player.handle!=0 {return Ok(None);}
        let entry=wad.entry(track as usize)?;if entry.sector==0 {return Ok(None);}
        let channels=wad.stereo(track as usize)?.unwrap();
        let right=wad.entry(track as usize+1)?;
        // Original ADDU wraps guestu32; the native reader uses relative sectors.
        let sectors=[disc_sector.wrapping_add(entry.sector),disc_sector.wrapping_add(right.sector)];
        self.player.state=1;self.source_14=10;self.player.remaining=48000;
        self.player.track=track as i16;self.player.handle=-1;
        self.player.flags=flags as u16;self.player.vol=volume as i16;self.player.stinger=0;
        Ok(Some(StartRequest {track:track as i16,flags:flags as u16,volume:volume as i16,sectors,channels}))
    }
    /// Fail closed before the unported GC EE body-queue step. Other states
    /// passing this narrow guard are not a claim of a complete EE music tick.
    pub fn require_body_queue_adapter(&self)->Result<()> {
        if matches!(self.player.state&0x7fff,8|9) {
            return invalid("GC_MUSIC_SOURCE_32EC80_STEREO_BODY_QUEUE_UNPORTED");
        }
        Ok(())
    }
    /// Complete original32fcc0: always store real returnedhandle; zero clears
    /// state, nonzero promotes only pendingstate1 to8 (body queue required).
    pub fn complete_start(&mut self,handle:i32) {
        self.player.handle=handle;
        if handle==0 {self.player.state=0;} else if self.player.state==1 {self.player.state=8;}
    }
}

/// Two actual native stream handles share the source's one stereo request.
/// Voice storage/ADPCM decoding/mixing remain exclusively in the existing SPU.
#[derive(Debug)]
pub struct StereoStream {pub handles:[i32;2]}
impl StereoStream {
    pub fn start(request:&StartRequest,streams:&mut Streams,vm:&mut VoiceManager,spu:&mut Spu)->Result<Self> {
        let mut tracks:[Option<Arc<MusicTrack>>;15]=Default::default();
        tracks[0]=Some(Arc::clone(&request.channels[0]));tracks[1]=Some(Arc::clone(&request.channels[1]));
        let mut handles=[0;2];
        for channel in 0..2 {
            let command=StreamCommand::Play {player:PlayerId::Main,track:channel,vol:request.volume as i32,
                group:1,queue:0,flags:0x20,cb:StreamCallback::Start};
            let reply=streams.execute(&command,&tracks,vm,spu).ok_or_else(||rc_formats::FormatError::Invalid("GC native stream start has no callback".into()))?;
            if reply.value<=0 {
                for handle in handles.into_iter().filter(|h|*h>0) {
                    streams.execute(&StreamCommand::Stop {handle},&tracks,vm,spu);
                }
                return invalid("GC native stereo stream could not allocate both channels");
            }
            handles[channel]=reply.value;
            let voice=vm.voices.iter().position(|slot|slot.owner==VoiceUse::Stream {handle:reply.value as u32})
                .ok_or_else(||rc_formats::FormatError::Invalid("GC native stereo handle lost actual voice owner".into()))?;
            // Reuse the native 989snd volume/group law; channel routing is a
            // native transport decision, not a recovered original IOP gain claim.
            let volume=((127*request.volume as i32)>>10).clamp(0,127);
            let base=vm.make_volume(127,if channel==0 {-90}else{90},volume,0,127,0);
            vm.voices[voice].basevol=base;spu.voices[voice].vol=vm.voice_registers(base,1);
        }
        Ok(Self {handles})
    }
    pub fn stop(self,streams:&mut Streams,vm:&mut VoiceManager,spu:&mut Spu) {
        let tracks=Default::default();
        for handle in self.handles {streams.execute(&StreamCommand::Stop {handle},&tracks,vm,spu);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn channel(nibble:u8)->Arc<MusicTrack> {
        let mut body=vec![nibble;16*4];for frame in body.chunks_mut(16) {frame[0]=0x04;frame[1]=0;}body[16*3+1]=1;
        Arc::new(MusicTrack {name:String::new(),rate:48000,body:Arc::from(body)})
    }
    #[test]
    fn native_stereo_uses_existing_stream_owners_and_preserves_channel_identity() {
        let request=StartRequest {track:0,flags:1,volume:0x400,sectors:[3,4],channels:[channel(0x77),channel(0x00)]};
        let(mut streams,mut vm,mut spu)=(Streams::default(),VoiceManager::default(),Spu::new(Arc::from([])));
        let pair=StereoStream::start(&request,&mut streams,&mut vm,&mut spu).unwrap();
        assert_ne!(pair.handles[0],pair.handles[1]);
        assert_eq!(vm.voices.iter().filter(|v|matches!(v.owner,VoiceUse::Stream {..})).count(),2);
        let pcm=(0..100).map(|_|spu.mix()).collect::<Vec<_>>();
        assert!(pcm.iter().any(|p|p[0]!=0));assert!(pcm.iter().all(|p|p[1]==0));
        pair.stop(&mut streams,&mut vm,&mut spu);
        assert_eq!(vm.voices.iter().filter(|v|matches!(v.owner,VoiceUse::Stream {..})).count(),0);
    }
    #[test]
    fn callback_requires_real_handle_and_respects_nonpending_state() {
        let mut m=MusicPlayer {player:Player {handle:-1,state:1,..Player::default()},source_14:27};
        m.complete_start(14);assert_eq!((m.player.handle,m.player.state),(14,8));assert!(m.require_body_queue_adapter().is_err());
        m.player.state=5;m.complete_start(99);assert_eq!(m.player.state,5);
        m.complete_start(0);assert_eq!(m.player.state,0);assert_eq!(m.source_14,27);assert!(m.require_body_queue_adapter().is_ok());
    }
    #[test]
    fn missing_track_gate_preserves_explicit_state() {
        let mut b=vec![0;0x1018];b[..4].copy_from_slice(&0x1018u32.to_le_bytes());let w=MusicWad::parse(&b).unwrap();
        let mut m=MusicPlayer {player:Player {track:7,vol:333,..Player::default()},source_14:42};let before=m;
        assert!(m.request_start(&w,1234,-1,1,1024).unwrap().is_none());assert_eq!(m,before);
        assert!(m.request_start(&w,1234,0,1,1024).unwrap().is_none());assert_eq!(m,before);
        assert!(w.stereo(511).is_err());
    }
}
