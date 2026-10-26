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
pub struct Types {init:[u32;9],classes:[i32;9]}
impl Types {
    pub fn from_aranos_overlay(overlay:&[u8])->Result<Self> {
        let sections=font::parse_overlay_sections(overlay)?;
        let bytes=font::read_overlay(&sections,0x2a3a00,9*0x14).ok_or_else(||rc_formats::FormatError::Invalid("GC original camera type table absent".into()))?;
        let init=std::array::from_fn(|i|u32::from_le_bytes(bytes[i*0x14+8..i*0x14+12].try_into().unwrap()));
        let classes=std::array::from_fn(|i|i32::from_le_bytes(bytes[i*0x14..i*0x14+4].try_into().unwrap()));
        // Recover only the source table identity tested by this adapter, never
        // accept arbitrary code/table as an executable native implementation.
        if init[0]!=0x424908 {return invalid("GC camera type0 init source differs from Aranos contract");}
        if !classes.contains(&-1) {return invalid("GC original camera table sentinel absent");}
        Ok(Self {init,classes})
    }
    ///2da110: first matching class OR sentinel index. The original returns
    /// the sentinel's index for an unknown class, not an invented type0.
    pub fn type_index(&self,class:i16)->Result<i16> {
        self.classes.iter().position(|&c|c==class as i32 || c==-1).map(|i|i as i16)
            .ok_or_else(||rc_formats::FormatError::Invalid("GC camera type lookup lacks sentinel".into()))
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
    projection:ProjectionReset,
}
impl Reset {
    pub fn begin(runtime:&mut Runtime<'_>)->Result<Self> {
        Self::validate(runtime)?;
        if let Some(old)=*runtime.current {
            let record=&mut runtime.records[old];
            record.active_8e=0;record.byte_7d=0;record.half_7e=0;
        }
        Ok(Self {stage:Stage::Select,pending:None,projection:ProjectionReset::begin()})
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

    /// Execute2d79b8 only for the actual pending projection stage. The host
    /// must finish2e5640 through its existing renderer/context/LOD owners.
    /// A true result does NOT acknowledge Reset.complete or prove GPU work.
    pub fn resume_projection(&mut self,globals:&mut FovGlobals<'_>,
        host:impl FnMut(u32)->Result<bool>)->Result<bool> {
        if self.pending!=Some(Call::Projection {source:0x2d79b8,required_projection_service:0x2e5640}) {
            return invalid("GC FOV reset requires pending original projection call");
        }
        self.projection.resume(globals,host)
    }
}

/// Borrow the existing camera/render-context fields; no default values or
/// second live FOV owner. Names are guest addresses, not inferred meanings.
pub struct FovGlobals<'a> {
    pub word_1b8fd0:&'a mut u32,pub word_1b8fd4:&'a mut u32,
    pub word_1b8fd8:&'a mut u32,pub word_1b8fdc:&'a mut u32,
    pub word_1b8fe0:&'a mut u32,pub word_1b8fe4:&'a mut u32,
    pub byte_1b8fe9:&'a mut u8,pub tangent_1bd2f0:&'a mut u32,
}
#[derive(Clone,Copy,Debug)]
enum ProjectionStage {Pre,Host(u32),Done}
/// Complete source wrapper2d79b8 with an explicitly mandatory host service.
/// Native f32 sin/cos is the existing native-camera policy, not PS2 bit parity.
#[derive(Clone,Copy,Debug)]
pub struct ProjectionReset {stage:ProjectionStage}
impl ProjectionReset {
    pub fn begin()->Self {Self {stage:ProjectionStage::Pre}}
    pub fn resume(&mut self,g:&mut FovGlobals<'_>,mut host:impl FnMut(u32)->Result<bool>)->Result<bool> {
        if matches!(self.stage,ProjectionStage::Pre) {
            //2d79f0, f4, fc in original store order; no source write+3e8.
            *g.word_1b8fd8=0x3f8f_ea69;*g.byte_1b8fe9=0;*g.word_1b8fd4=0x3f8f_ea69;
            let half=f32::from_bits(0x3f0f_ea69);
            let sine=half.sin();let cosine=half.cos();
            let tangent=(sine/cosine).to_bits();
            *g.tangent_1bd2f0=tangent;self.stage=ProjectionStage::Host(tangent);
        }
        if let ProjectionStage::Host(tangent)=self.stage {
            // False/error retains the call boundary and the completed prefix.
            // The host owns any internal resumption of2e5640 side effects.
            if !host(tangent)? {return Ok(false);}
            //2d7a34..40 happen ONLY after the original host call returns.
            *g.word_1b8fdc=0x3ba3_d70a;*g.word_1b8fe0=0x3e4c_cccd;
            *g.word_1b8fe4=0;*g.word_1b8fd0=0;
            self.stage=ProjectionStage::Done;
        }
        Ok(true)
    }
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

/// Explicit source inputs for2e5640's remaining scalar context writes. The
/// source addresses are offsets from C=1bd240; no viewport/fog defaults.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ProjectionContextInputs {
    pub projection:ProjectionInputs,
    pub source_200_20c:[u32;4],
    pub source_228:u32,pub source_22c:u32,pub word_1b8f00:i32,
}
/// Scalar results to publish through existing native projection/frustum/fog
/// owners. This does not execute context memcpy, GS packet publication or LOD.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct NativeContextScalars {
    pub parameters:ProjectionParameters,
    pub ratio_a8_ac:[u32;2],pub reciprocal_cos_b8_bc:[u32;2],
    pub frustum_scale_1e8:u32,pub fog_210_214_220_224:[u32;4],
}
///2e56e4..2e599c scalar dataflow. Shared Pf preserves source scalar operation
/// order. Transcendentals use native f32 atan2/cos, matching existing camera
/// policy; this is deliberately not a claim of PS2 transcendental bit parity.
pub fn native_projection_context_scalars(i:ProjectionContextInputs)->NativeContextScalars {
    use rc_game::ps2v::Pf;
    let p=native_projection_parameters(i.projection);
    let [a,b,c,d]=i.source_200_20c.map(Pf::b);
    let (rx,ry)=(c/a,d/b);
    let sec=|t:Pf|Pf::ONE/Pf::f(1f32.atan2(f32::from_bits(t.0)).cos());
    let (sx,sy)=(sec(Pf::b(p.tan_x)),sec(Pf::b(p.tan_y)));
    let mut scale=(sec(Pf::b(p.tan_x)*rx)/sx+sec(Pf::b(p.tan_y)*ry)/sy)*Pf::b(0x3f00_0000);
    if i.word_1b8f00==2 && Pf::b(p.tan_x)<Pf::b(0x3f1e_b852) {scale=scale*Pf::b(0x3f93_3333);}
    let (near,far,lo,hi)=(Pf::b(i.projection.fog_near),Pf::b(p.fog_far),Pf::b(i.source_228),Pf::b(i.source_22c));
    let delta=far-near;let unit=Pf::b(0x3a80_0000);
    let range=hi-lo;
    let fog220=range/(delta*unit);
    let fog210=(range*Pf::b(p.near))/delta;
    let fog214=(lo*far-hi*near)/delta;
    let fog224=lo-near*(fog220*unit);
    NativeContextScalars {parameters:p,ratio_a8_ac:[rx.0,ry.0],reciprocal_cos_b8_bc:[sx.0,sy.0],
        frustum_scale_1e8:scale.0,fog_210_214_220_224:[fog210.0,fog214.0,fog220.0,fog224.0]}
}

/// GC Tfrag distance profile read by36a058 / raw-threshold TfragProc.
/// These are parameters for the EXISTING native tfrag LOD uniform/shader,
/// not a second terrain renderer or a claim that its GPU update occurred.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct LodProfile {pub distance_bits:[u32;3],pub scale_bits:u32}

impl LodProfile {
    ///36a058's CVT.W.S threshold writes are independent of the unknown FCSR
    /// rounding mode when their actual f32 operands are exact in-range integers.
    /// Return None for fractional/nonfinite/out-of-range cases, never choose an
    /// invented rounding mode. This does not add another raw group/LOD owner.
    pub fn exact_raw_thresholds(&self)->Option<[i32;3]> {
        use rc_game::ps2v::Pf;
        let mut out=[0;3];
        for (i,&distance) in self.distance_bits.iter().enumerate() {
            let bits=(Pf::b(distance)*Pf::b(0x4480_0000)).0;
            if bits&0x7f80_0000==0x7f80_0000 {return None;}
            let value=f32::from_bits(bits);
            if !value.is_finite() || value.fract()!=0. || (value as f64)<i32::MIN as f64 || (value as f64)>i32::MAX as f64 {return None;}
            out[i]=value as i32;
        }
        Some(out)
    }
}

///300d18's initial [6L,4L,2L] uses the already compatible shared format helper.
/// The actual original block header is required; no invented base distance.
pub fn initial_lod_profile(block:&rc_formats::tfrag::TfragBlockHeader)->LodProfile {
    LodProfile {distance_bits:block.lod_distances().map(f32::to_bits),scale_bits:1f32.to_bits()}
}
///36a1c0's dynamic profile. Preserve source multiplication order rather than
/// scaling the three precomputed distances: `(L*6)*s` can round differently
/// from original `L*(s*6)`. Use exact_raw_thresholds only when operands prove
/// round-mode independence; other conversions need the original FCSR context.
pub fn rescaled_lod_profile(block:&rc_formats::tfrag::TfragBlockHeader,horizontal_tangent:u32,force_unscaled:i32)->LodProfile {
    use rc_game::ps2v::Pf;
    let mut scale=Pf::b(0x3f1e_b852)/Pf::b(horizontal_tangent);
    if scale<Pf::ONE {scale=Pf::ONE;}
    if Pf::b(0x4080_0000)<scale {scale=Pf::b(0x4080_0000);}
    if force_unscaled!=0 {scale=Pf::ONE;}
    let base=Pf::f(block.unknown_8);
    LodProfile {distance_bits:[(base*(scale*Pf::b(0x40c0_0000))).0,
        (base*(scale*Pf::b(0x4080_0000))).0,((base+base)*scale).0],scale_bits:scale.0}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fov_host_pending_keeps_source_prefix_and_defers_postwrites() {
        let mut words=[11,12,13,14,15,16,17];let mut byte=9;
        let [d0,d4,d8,dc,e0,e4,tangent]=&mut words;
        let mut g=FovGlobals {word_1b8fd0:d0,word_1b8fd4:d4,word_1b8fd8:d8,
            word_1b8fdc:dc,word_1b8fe0:e0,word_1b8fe4:e4,byte_1b8fe9:&mut byte,tangent_1bd2f0:tangent};
        let mut p=ProjectionReset::begin();let mut observed=None;
        assert!(!p.resume(&mut g,|v|{observed=Some(v);Ok(false)}).unwrap());
        assert_eq!((*g.word_1b8fd8,*g.word_1b8fd4,*g.byte_1b8fe9),(0x3f8fea69,0x3f8fea69,0));
        assert_eq!((*g.word_1b8fd0,*g.word_1b8fdc,*g.word_1b8fe0,*g.word_1b8fe4),(11,14,15,16));
        assert!(f32::from_bits(observed.unwrap())>0.6);
        // A host which suspended after mutating context must not have its
        // writes overwritten by another source prefix on the retry.
        *g.word_1b8fd4=99;*g.tangent_1bd2f0=77;
        assert!(p.resume(&mut g,|v|{assert_eq!(Some(v),observed);Ok(true)}).unwrap());
        assert_eq!((*g.word_1b8fd4,*g.tangent_1bd2f0),(99,77));
        assert_eq!((*g.word_1b8fd0,*g.word_1b8fdc,*g.word_1b8fe0,*g.word_1b8fe4),(0,0x3ba3d70a,0x3e4ccccd,0));
        assert!(p.resume(&mut g,|_|panic!("completed host must not replay")).unwrap());
    }
    #[test]
    fn fov_requires_pending_projection_and_host_errors_do_not_replay_prefix() {
        let mut words=[0;7];let mut byte=3;
        let [d0,d4,d8,dc,e0,e4,tangent]=&mut words;
        let mut g=FovGlobals {word_1b8fd0:d0,word_1b8fd4:d4,word_1b8fd8:d8,
            word_1b8fdc:dc,word_1b8fe0:e0,word_1b8fe4:e4,byte_1b8fe9:&mut byte,tangent_1bd2f0:tangent};
        let mut records=[metadata();48];records[0].class_86=0;
        let(mut current,mut half,mut word)=(None,0,0);
        let mut runtime=Runtime {records:&mut records,current:&mut current,half_1b8e90:&mut half,word_1b9004:&mut word};
        let mut reset=Reset::begin(&mut runtime).unwrap();
        assert!(reset.resume_projection(&mut g,|_|panic!("wrong stage")).is_err());
        assert_eq!((*g.word_1b8fd8,*g.byte_1b8fe9),(0,3));
        let types=Types {init:[0;9],classes:[0;9]};
        while let Some(call)=reset.next(&mut runtime,&types).unwrap() {
            if matches!(call,Call::Projection {..}) {break;}
            reset.complete(&mut runtime,call).unwrap();
        }
        assert!(reset.resume_projection(&mut g,|_|invalid("test host failed")).is_err());
        *g.word_1b8fd8=42;
        assert!(reset.resume_projection(&mut g,|_|Ok(true)).unwrap());
        assert_eq!(*g.word_1b8fd8,42);
        assert!(matches!(reset.pending(),Some(Call::Projection {..})),"host contract completion is not Reset acknowledgement or GPU proof");
    }
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
    #[test]
    fn context_scalars_keep_independent_viewport_and_fog_domains() {
        let mut i=ProjectionContextInputs {projection:ProjectionInputs {horizontal_tangent:0.5f32.to_bits(),
            near:32f32.to_bits(),far:99999f32.to_bits(),fog_near:2048f32.to_bits(),fog_far:4096f32.to_bits(),
            word_1a7b98:0,byte_1a7bb9:0},source_200_20c:[640f32,448.,640.,448.].map(f32::to_bits),
            source_228:0f32.to_bits(),source_22c:1f32.to_bits(),word_1b8f00:0};
        let c=native_projection_context_scalars(i);
        assert_eq!(c.ratio_a8_ac,[1f32.to_bits();2]);
        assert_eq!(c.frustum_scale_1e8,1f32.to_bits());
        assert_eq!(c.fog_210_214_220_224.map(f32::from_bits),[0.015625,-1.,0.5,-1.]);
        // These coefficients reproduce the original low/high endpoints in
        // fog distance space, independently of the near-clip coefficient.
        let [_,_,slope,intercept]=c.fog_210_214_220_224.map(f32::from_bits);
        assert_eq!(2048.*(slope/1024.)+intercept,0.);
        assert_eq!(4096.*(slope/1024.)+intercept,1.);
        i.word_1b8f00=2;
        assert_eq!(native_projection_context_scalars(i).frustum_scale_1e8,0x3f93_3333);
        i.source_200_20c[2]=320f32.to_bits();
        let cropped=native_projection_context_scalars(i);
        assert_eq!(cropped.ratio_a8_ac,[0.5f32.to_bits(),1f32.to_bits()]);
        assert!(f32::from_bits(cropped.frustum_scale_1e8)>1.15);
    }
    #[test]
    fn lod_bridge_reuses_original_block_and_source_clamp_force_order() {
        let block=rc_formats::tfrag::TfragBlockHeader {table_offset:16,tfrag_count:1,unknown_8:12.,unknown_c:0};
        assert_eq!(initial_lod_profile(&block).distance_bits.map(f32::from_bits),[72.,48.,24.]);
        assert_eq!(rescaled_lod_profile(&block,0.31f32.to_bits(),0).distance_bits.map(f32::from_bits),[144.,96.,48.]);
        assert_eq!(rescaled_lod_profile(&block,0.1f32.to_bits(),0).distance_bits.map(f32::from_bits),[288.,192.,96.]);
        assert_eq!(rescaled_lod_profile(&block,1f32.to_bits(),0).distance_bits.map(f32::from_bits),[72.,48.,24.]);
        assert_eq!(rescaled_lod_profile(&block,0.1f32.to_bits(),-1).distance_bits.map(f32::from_bits),[72.,48.,24.]);
    }
    #[test]
    fn threshold_conversion_accepts_only_rounding_independent_operands() {
        let p=LodProfile {distance_bits:[72f32.to_bits(),48f32.to_bits(),24f32.to_bits()],scale_bits:1f32.to_bits()};
        assert_eq!(p.exact_raw_thresholds(),Some([73728,49152,24576]));
        let mut p=p;p.distance_bits[0]=(0.5f32/1024.).to_bits();assert_eq!(p.exact_raw_thresholds(),None);
        p.distance_bits[0]=2097152f32.to_bits();assert_eq!(p.exact_raw_thresholds(),None);
        p.distance_bits[0]=(-2097152f32).to_bits();assert_eq!(p.exact_raw_thresholds().unwrap()[0],i32::MIN);
        p.distance_bits[0]=f32::NAN.to_bits();assert_eq!(p.exact_raw_thresholds(),None);
    }
    fn metadata()->Metadata {Metadata {class_86:7,type_8c:0,active_8e:6,byte_7d:9,half_7e:12,data_70:DataBinding::Original(0x1234)}}
    #[test]
    fn source_order_preserves_metadata_and_requires_every_host_service() {
        let mut records=[metadata();48];records[4].class_86=0;records[9].class_86=0;
        let(mut current,mut half,mut word)=(Some(2),17,0xabcd);
        let mut runtime=Runtime {records:&mut records,current:&mut current,half_1b8e90:&mut half,word_1b9004:&mut word};
        let types=Types {init:[0x424908;9],classes:[0,4,5,6,7,19,23,27,-1]};let mut reset=Reset::begin(&mut runtime).unwrap();
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
        let mut reset=Reset::begin(&mut runtime).unwrap();assert!(reset.next(&mut runtime,&Types {init:[0;9],classes:[0;9]}).is_err());
        assert_eq!((*runtime.current,*runtime.half_1b8e90,*runtime.word_1b9004),(None,17,99));
        *runtime.current=Some(48);assert!(Reset::begin(&mut runtime).is_err());
    }
}
