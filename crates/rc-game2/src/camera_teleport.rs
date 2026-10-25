//! Source-ordered metadata adapter for GC camera reset2da160.
//! The existing FollowCamera remains the only live follow pose/history owner.
//! This borrows ONLY record metadata. Motion, type init, actual deep backup,
//! pose publication and projection are mandatory typed host calls; merely
//! producing/acknowledging test tokens is not proof of a completed game reset.
use rc_formats::{buf::{invalid,Result},font};

/// Native binding of the original record+70 data reference. An opaque original
/// binding is not reinterpreted as a host pointer or dereferenced guest memory.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DataBinding {Original(u32),Follow,Backup}

/// Only source metadata, never copies of the existing FollowCamera pose/data.
/// No Default: caller must provide actual recovered/explicit fixture state.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Metadata {
    pub class_86:i16,pub type_8c:i16,pub active_8e:i16,
    pub byte_7d:u8,pub half_7e:i16,pub data_70:DataBinding,
}

/// Borrowed original native record metadata/current-id/global owners.
/// The caller owns initialization provenance; this is NOT a New Game loader.
pub struct Runtime<'a> {
    pub records:&'a mut[Metadata],
    pub current:&'a mut Option<usize>,
    pub half_1b8e90:&'a mut i16,
    pub word_1b9004:&'a mut u32,
}

/// Original2d8120's init callbacks, read from the actual Aranos type table.
/// The final sentinel record is retained; no invented callback fallback.
#[derive(Clone,Debug)]
pub struct Types {init:[u32;9]}
impl Types {
    pub fn from_aranos_overlay(overlay:&[u8])->Result<Self> {
        let sections=font::parse_overlay_sections(overlay)?;
        let bytes=font::read_overlay(&sections,0x2a3a00,9*0x14).ok_or_else(||rc_formats::FormatError::Invalid("GC original camera type table absent".into()))?;
        let init=std::array::from_fn(|i|u32::from_le_bytes(bytes[i*0x14+8..i*0x14+12].try_into().unwrap()));
        // Recover only the source table identity tested by this adapter, never
        // accept arbitrary code/table as an executable native implementation.
        if init[0]!=0x424908 {return invalid("GC camera type0 init source differs from Aranos contract");}
        Ok(Self {init})
    }
    fn init(&self,index:i16)->Result<u32> {
        self.init.get(usize::try_from(index).unwrap_or(usize::MAX)).copied()
            .ok_or_else(||rc_formats::FormatError::Invalid("GC camera type index outside original table".into()))
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Call {
    /// Existing FollowCamera motion pre-step, preserving current global/history.
    Motion {source:u32},
    ///2d8120 table-selected init (type0=424908), separate from motion because
    /// original word1b9004 is cleared BETWEEN these services.
    Init {source:u32,record:usize},
    ///2d7d80 copies source recorda0bytes plus actual followdata280bytes and
    /// rewires backup+70. Host must use a real saved copy of existing owners;
    /// this module does not create another live FollowCamera or raw pose mirror.
    Backup {source:u32,record:usize},
    ///2da1dc..2da230: publish full position16bytes, currentrecord prevXYZ,
    /// and heroXYZ while preserving its existingW lane.
    Publish {source:u32,record:usize},
    ///Full2d79b8, including2e5640 native renderer projection/frustum effects.
    /// A record reset or computed FOV alone does not satisfy this call.
    Projection {source:u32,required_projection_service:u32},
}
impl Call {
    pub fn source(self)->u32 {match self {Self::Motion {source}|Self::Init {source,..}|Self::Backup {source,..}|Self::Publish {source,..}|Self::Projection {source,..}=>source}}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Stage {Select,Motion,Init,Backup,Publish,Projection,Done}

/// Resumable source wrapper. No host operation is skipped automatically.
/// Keep this on the existing teleport progress owner, not a separate scheduler.
#[derive(Clone,Copy,Debug)]
pub struct Reset {
    stage:Stage,
    pending:Option<Call>,
}
impl Reset {
    pub fn begin(runtime:&mut Runtime<'_>)->Result<Self> {
        Self::validate(runtime)?;
        if let Some(old)=*runtime.current {
            let record=&mut runtime.records[old];
            record.active_8e=0;record.byte_7d=0;record.half_7e=0;
        }
        Ok(Self {stage:Stage::Select,pending:None})
    }
    fn validate(runtime:&Runtime<'_>)->Result<()> {
        if runtime.records.len()!=48 {return invalid("GC camera reset requires actual48-record bank metadata");}
        if runtime.current.is_some_and(|i|i>=runtime.records.len()) {return invalid("GC current camera binding out of range");}
        Ok(())
    }
    pub fn next(&mut self,runtime:&mut Runtime<'_>,types:&Types)->Result<Option<Call>> {
        Self::validate(runtime)?;
        if let Some(call)=self.pending {return Ok(Some(call));}
        loop {
            let call=match self.stage {
                Stage::Select=>{
                    //2d7e50(0): scans ALL48 slots, first signedclass match;
                    // active flags are not part of this lookup.
                    let selected=runtime.records.iter().position(|r|r.class_86==0)
                        .ok_or_else(||rc_formats::FormatError::Invalid("GC original camera class0 record absent".into()))?;
                    *runtime.current=Some(selected);
                    let record=&mut runtime.records[selected];record.active_8e=1;
                    record.data_70=DataBinding::Follow;record.byte_7d=0;
                    *runtime.half_1b8e90=0;self.stage=Stage::Motion;continue;
                },
                Stage::Motion=>Call::Motion {source:0x2d9760},
                Stage::Init=>{
                    let id=runtime.current.ok_or_else(||rc_formats::FormatError::Invalid("GC selected camera owner absent".into()))?;
                    let source=types.init(runtime.records[id].type_8c)?;
                    if source==0 {self.stage=Stage::Backup;continue;} // actual null callback source branch
                    Call::Init {source,record:id}
                },
                Stage::Backup=>Call::Backup {source:0x2d7d80,record:runtime.current.ok_or_else(||rc_formats::FormatError::Invalid("GC camera backup owner absent".into()))?},
                Stage::Publish=>Call::Publish {source:0x2da1dc,record:runtime.current.ok_or_else(||rc_formats::FormatError::Invalid("GC camera publish owner absent".into()))?},
                Stage::Projection=>Call::Projection {source:0x2d79b8,required_projection_service:0x2e5640},
                Stage::Done=>return Ok(None),
            };
            self.pending=Some(call);return Ok(Some(call));
        }
    }
    /// Resume ONLY after the exact pending native service actually executed.
    /// Explicit acknowledgement is a host contract, not evidence of execution.
    /// Wrong/stale completions fail before changing any progress/global field.
    pub fn complete(&mut self,runtime:&mut Runtime<'_>,call:Call)->Result<()> {
        Self::validate(runtime)?;
        if self.pending!=Some(call) {return invalid("GC camera service completion differs from pending original call");}
        self.pending=None;
        self.stage=match self.stage {
            Stage::Motion=>{*runtime.word_1b9004=0;Stage::Init},
            Stage::Init=>Stage::Backup,
            Stage::Backup=>Stage::Publish,
            Stage::Publish=>Stage::Projection,
            Stage::Projection=>Stage::Done,
            _=>return invalid("GC camera completion has no pending executable stage"),
        };
        Ok(())
    }
    pub fn pending(&self)->Option<Call> {self.pending}
}

/// Bound original render-context inputs needed by the small parameter bridge
/// below. All are explicit source bits; no PAL/display/near/far/NewGame default.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ProjectionInputs {
    /// C=1bd240: +b0, +a0/+a4, +218/+21c.
    pub horizontal_tangent:u32,pub near:u32,pub far:u32,
    pub fog_near:u32,pub fog_far:u32,
    pub word_1a7b98:i32,pub byte_1a7bb9:u8,
}
/// Values directly adaptable to the EXISTING renderer's GameProjection and
/// LevelFog; these fields do not constitute all2e5640 matrices/GS effects.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ProjectionParameters {
    pub tan_x:u32,pub tan_y:u32,pub near:u32,pub far:u32,pub fog_far:u32,
}
/// Verified2e5640 scalar parameter slice, using existing shared PS2 float math:
/// 2e5648..2e56d4 display tangent and2e58b0..2e58d0 fog far clamp.
/// This executes real recovered math but does NOT complete the projection call;
/// frustum/inverse/packet uniforms and36a058 remain mandatory host work.
pub fn native_projection_parameters(input:ProjectionInputs)->ProjectionParameters {
    use rc_game::ps2v::Pf;
    let ratio=if input.word_1a7b98!=0 {0x3f41_8937}else{0x3f46_6666};
    let mut tan_y=Pf::b(input.horizontal_tangent)*Pf::b(ratio);
    if input.byte_1a7bb9!=0 {tan_y=tan_y*Pf::b(0x3f40_0001);}
    let minimum=Pf::b(input.fog_near)+Pf::b(0x4480_0000);
    let fog_far=if Pf::b(input.fog_far)<minimum {minimum.0}else{input.fog_far};
    ProjectionParameters {tan_x:input.horizontal_tangent,tan_y:tan_y.0,
        near:input.near,far:input.far,fog_far}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_bridge_uses_explicit_display_bits_and_exact_fog_boundary() {
        let mut input=ProjectionInputs {horizontal_tangent:1f32.to_bits(),near:32f32.to_bits(),far:745472f32.to_bits(),
            fog_near:2048f32.to_bits(),fog_far:3000f32.to_bits(),word_1a7b98:0,byte_1a7bb9:0};
        let p=native_projection_parameters(input);
        assert_eq!(p.tan_y,0x3f46_6666);assert_eq!(p.tan_x,1f32.to_bits());
        assert_eq!((p.near,p.far),(input.near,input.far));assert_eq!(p.fog_far,3072f32.to_bits());
        input.word_1a7b98=-7;input.fog_far=3072f32.to_bits();
        assert_eq!(native_projection_parameters(input).tan_y,0x3f41_8937);
        assert_eq!(native_projection_parameters(input).fog_far,input.fog_far);
        input.word_1a7b98=0;input.horizontal_tangent=0;input.byte_1a7bb9=1;input.fog_far=4096f32.to_bits();
        assert_eq!(native_projection_parameters(input).tan_y,0);
        assert_eq!(native_projection_parameters(input).fog_far,input.fog_far);
    }
    fn metadata()->Metadata {Metadata {class_86:7,type_8c:0,active_8e:6,byte_7d:9,half_7e:12,data_70:DataBinding::Original(0x1234)}}
    #[test]
    fn source_order_preserves_metadata_and_requires_every_host_service() {
        let mut records=[metadata();48];records[4].class_86=0;records[9].class_86=0;
        let(mut current,mut half,mut word)=(Some(2),17,0xabcd);
        let mut runtime=Runtime {records:&mut records,current:&mut current,half_1b8e90:&mut half,word_1b9004:&mut word};
        let types=Types {init:[0x424908;9]};let mut reset=Reset::begin(&mut runtime).unwrap();
        assert_eq!(runtime.records[2].active_8e,0);assert_eq!(runtime.records[2].half_7e,0);
        let motion=reset.next(&mut runtime,&types).unwrap().unwrap();assert_eq!(motion,Call::Motion {source:0x2d9760});
        assert_eq!(*runtime.current,Some(4));assert_eq!(*runtime.half_1b8e90,0);assert_eq!(*runtime.word_1b9004,0xabcd);
        assert_eq!(runtime.records[4].data_70,DataBinding::Follow);assert_eq!(runtime.records[4].half_7e,12,"onlyoldrecord+7e cleared");
        assert_eq!(reset.next(&mut runtime,&types).unwrap(),Some(motion),"pending service cannot be replayed or skipped");
        assert!(reset.complete(&mut runtime,Call::Backup {source:0x2d7d80,record:4}).is_err());
        assert_eq!(*runtime.word_1b9004,0xabcd);
        reset.complete(&mut runtime,motion).unwrap();assert_eq!(*runtime.word_1b9004,0);
        for call in [Call::Init {source:0x424908,record:4},Call::Backup {source:0x2d7d80,record:4},
            Call::Publish {source:0x2da1dc,record:4},Call::Projection {source:0x2d79b8,required_projection_service:0x2e5640}] {
            assert_eq!(reset.next(&mut runtime,&types).unwrap(),Some(call));reset.complete(&mut runtime,call).unwrap();
        }
        assert_eq!(reset.next(&mut runtime,&types).unwrap(),None);
        assert_eq!(runtime.records[9],Metadata {class_86:0,..metadata()});
    }
    #[test]
    fn missing_or_invalid_original_bindings_fail_closed() {
        let mut records=[metadata();48];let(mut current,mut half,mut word)=(None,17,99);
        let mut runtime=Runtime {records:&mut records,current:&mut current,half_1b8e90:&mut half,word_1b9004:&mut word};
        let mut reset=Reset::begin(&mut runtime).unwrap();assert!(reset.next(&mut runtime,&Types {init:[0;9]}).is_err());
        assert_eq!((*runtime.current,*runtime.half_1b8e90,*runtime.word_1b9004),(None,17,99));
        *runtime.current=Some(48);assert!(Reset::begin(&mut runtime).is_err());
    }
}
