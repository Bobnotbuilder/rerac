//! GC native loader -> owned object records -> initialization -> live scheduling.
//! All class calls require an explicit native callback. Unknown source identities
//! return a blocker, never fall through to a RAC1 handler or a pretend no-op.
use crate::{objects::{LoadedClass,Objects},pvars::PvarPlan,spawn::SpawnState};
use rc_formats::{buf::{invalid,Buf,Result},rac2::gameplay};
use rc_game::{moby_runtime::{Moby,MobyId,mode},moby_update::scheduler,ps2v::Pf};

pub struct World {
    pub objects:Objects,
    pub instance_to_moby:Vec<Option<MobyId>>,
    pub moby_to_instance:Vec<usize>,
    /// Relocation metadata/shared blob. Each block's bytes have been MOVED
    /// into its unique moby owner; no disconnected copy is updated instead.
    pub pvars:PvarPlan,
    pvar_indices:Vec<i32>,
    pub groups:Vec<Vec<MobyId>>,
    pub gc:Vec<GcState>,
    pub preceding_owners:Vec<PrecedingOwner>,
    pub grid:rc_game::collision_query::MobyGrid,
    pub effects:Option<crate::effects::Effects>,
    hits:crate::hits::HitLog,
    init_list:Vec<MobyId>,
    init_cursor:usize,
    initialized:bool,
}

pub struct PrecedingOwner {pub section:&'static str,pub record:usize,pub pvar_index:i32,pub pvars:Vec<u8>}

/// GC's state bookkeeping has no matching raw RAC1 moby fields. Keep it
/// beside the shared semantic record rather than reinterpreting its layout.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct GcState {pub previous_94:u8,pub substate_95:u8,pub timer_96:u16,pub flags_be:u8}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MissingUpdate {pub moby:MobyId,pub class:i16,pub source:u32,pub state:u8,pub initializing:bool}

#[derive(Debug,PartialEq,Eq)]
pub enum TickError {NotInitialized,Missing(MissingUpdate),InvalidClass(i16),Effect(String)}

#[derive(Debug,PartialEq,Eq)]
pub struct Active {pub updates:Vec<MobyId>,pub targets:Vec<MobyId>}

impl rc_game::collision_query::MobySource for World {
    fn moby(&self,id:usize)->Option<rc_game::collision_query::CollMoby> {
        self.objects.get(id).map(rc_game::collision_query::CollMoby::of)
    }
    fn joints(&self,_id:usize,count:usize)->Vec<[u32;4]> {
        assert_eq!(count,0,"GC live rigid scene must not request invented joint poses");Vec::new()
    }
}

impl World {
    /// Static owner portion of the original loader. Non-moby owners must be
    /// supplied in original service/camera order; ambiguous aliases fail closed
    /// because the original table is overwritten with each allocation address.
    /// `ground` implements original 0x31ce20; it is required for rooted records.
    pub fn load(gameplay:&[u8],classes:&mut [LoadedClass<'_>],state:&mut SpawnState,
        mut ground:impl FnMut([f32;3])->Result<f32>)->Result<Self> {
        let instances=gameplay::moby_instances(gameplay)?;
        let section=gameplay::section(gameplay,"moby_instances")?.ok_or_else(||rc_formats::FormatError::Invalid("GC static section absent".into()))?;
        let spare=Buf(section).i32(4)?;
        if !(2..=32767).contains(&spare) {return invalid("GC dynamic capacity must include reserved moby and end marker");}
        //0x302af4..0x302b90 reserves one static class0 record and one end
        // marker OUTSIDE the allocatable dynamic interval [1e20,1e24).
        let dynamic_capacity=spare as usize-2;
        let mut next_state=state.clone();
        let decisions=instances.iter().map(|i|next_state.decide(i)).collect::<Result<Vec<_>>>()?;
        let mut instance_to_moby=vec![None;instances.len()];let mut moby_to_instance=Vec::new();
        for (i,d) in decisions.iter().enumerate() {if d.spawn {
            if moby_to_instance.len()>i16::MAX as usize {return invalid("GC spawned ids exceed signed instance map");}
            instance_to_moby[i]=Some(moby_to_instance.len());moby_to_instance.push(i);
        }}
        // These are cameras (header08, stride20, pvar1c), then sound
        // instances (header0c, stride90, pvar08), not a RAC1 pvar loader.
        let mut preceding_owners=Vec::new();
        for (section,stride,field) in [("cameras",0x20,0x1c),("sound_instances",0x90,8)] {
            let Some(bytes)=gameplay::section(gameplay,section)? else {continue;};
            let b=Buf(bytes);let count=b.i32(0)?;
            if !(0..=10000).contains(&count) {return invalid("GC non-moby pvar owner count is invalid");}
            for record in 0..count as usize {
                let r=b.sub(16+record*stride,stride,"GC preceding pvar owner")?;
                preceding_owners.push(PrecedingOwner {section,record,pvar_index:r.i32(field)?,pvars:Vec::new()});
            }
        }
        let mut copied=Vec::new();
        for &index in preceding_owners.iter().map(|o|&o.pvar_index).chain(moby_to_instance.iter().filter_map(|&i| {
            (instances[i].base.o_class!=0).then_some(&instances[i].base.pvar_index)
        })) {
            if index<0 {continue;}
            if copied.contains(&index) {return invalid("GC pvar owner aliases an already allocated source entry; alias semantics require an adapter");}
            copied.push(index);
        }
        let mut pvars=PvarPlan::copy_and_link(gameplay,&copied,&instance_to_moby)?;
        for owner in &mut preceding_owners {if owner.pvar_index>=0 {
            owner.pvars=pvars.blocks[owner.pvar_index as usize].take().ok_or_else(||rc_formats::FormatError::Invalid("GC preceding owner lost pvar allocation".into()))?;
        }}
        let mut records=Vec::with_capacity(moby_to_instance.len());
        let mut grid=rc_game::collision_query::MobyGrid::new();
        // Class mutation is committed after all input checks; a failed load
        // must not mutate the shared class bank or caller's save inputs.
        let mut modes=classes.iter().map(|c|c.info.mode_bits).collect::<Vec<_>>();
        for &i in &moby_to_instance {
            let inst=&instances[i];let b=&inst.base;let d=decisions[i];let f=inst.spawn_fields();
            let slot=classes.iter().position(|c|c.class as i32==b.o_class)
                .ok_or_else(||rc_formats::FormatError::Invalid(format!("GC static class {:#x} has no validated loaded header",b.o_class)))?;
            let class=&classes[slot];let mut info=class.info;info.mode_bits=modes[slot];
            let id=records.len();let mut m=Moby::init_instance(id as u32,class.class,Some(&info));
            rc_game::moby_update::anim_sound::init(&mut m,Some(class.animation));
            m.mission=f.mission as u8;m.spawn_flag=d.b1;m.spawn_id=f.id as i16;m.b4=d.b4;m.b6=d.b6;
            if !info.no_header {m.scale=(Pf::f(info.scale)*Pf::f(b.scale)).to_f32();}
            m.draw_dist=b.draw_distance as i16;m.update_dist=b.update_distance as u8;
            m.position=[b.position[0],b.position[1],b.position[2],0.];
            m.rotation=[b.rotation[0],b.rotation[1],b.rotation[2],0.];m.group=b.group as i8;
            if b.is_rooted!=0 {let z=ground(b.position)?;if 0.<z {m.position[2]=(Pf::f(z)+Pf::f(b.rooted_distance)).to_f32();}}
            m.occlusion=if b.occlusion==0 {0}else{0x7f80};
            if b.o_class!=0 && b.pvar_index>=0 {m.pvars=pvars.blocks[b.pvar_index as usize].take().ok_or_else(||rc_formats::FormatError::Invalid("GC moby lost its owned pvar allocation".into()))?;}
            if !info.no_header {modes[slot]|=b.mode_bits as u16;}
            m.mode|=b.mode_bits as u16;
            if m.has_class {scheduler::rebuild_matrix(&mut m,Some(class.animation));grid.register(&mut m);}
            let rgb=b.ambient_rgb();m.ambient=[rgb[0],rgb[1],rgb[2],0];m.light=b.light_word();
            records.push(m);
        }
        let groups=parse_groups(gameplay,&instance_to_moby)?;
        // Talk-slot hook 0x309e90 and non-moby owner storage are not silently
        // marked initialized. Their native ports run before class callbacks.
        if instances.iter().zip(&instance_to_moby).any(|(i,m)|m.is_some() && i.base.unknown_74!=-1) {
            return invalid("GC static talk-slot registration 0x309e90 is not ported");
        }
        let hero=classes.iter().find(|c|c.class==0).ok_or_else(||rc_formats::FormatError::Invalid("GC reserved record requires class0 resource".into()))?;
        let hero_slot=classes.iter().position(|c|c.class==0).unwrap();let mut info=hero.info;info.mode_bits=modes[hero_slot];
        let mut reserved=Moby::init_instance(records.len() as u32,0,Some(&info));
        rc_game::moby_update::anim_sound::init(&mut reserved,Some(hero.animation));reserved.state=0xfd;records.push(reserved);
        let init_list=records.iter().enumerate().filter_map(|(id,m)|(m.state&0x80==0 && m.mode&2==0).then_some(id)).collect();
        let count=records.len()+dynamic_capacity;
        for (c,mode) in classes.iter_mut().zip(modes) {c.info.mode_bits=mode;}
        *state=next_state;
        let mut pvar_indices=moby_to_instance.iter().map(|&i|if instances[i].base.o_class==0 {-1}else{instances[i].base.pvar_index}).collect::<Vec<_>>();
        pvar_indices.push(-1);
        Ok(Self {objects:Objects::new(records,dynamic_capacity),instance_to_moby,moby_to_instance,pvars,pvar_indices,groups,
            gc:vec![GcState::default();count],preceding_owners,grid,effects:None,hits:Default::default(),init_list,init_cursor:0,initialized:false})
    }

    pub fn local_pvar_bytes(&self,id:MobyId,field:usize,len:usize)->Result<&[u8]> {
        let Some(&index)=self.pvar_indices.get(id).filter(|&&i|i>=0) else {return invalid("GC moby has no static pvar source");};
        let Some(m)=self.objects.get(id) else {return invalid("GC pvar owner absent");};
        self.pvars.local_bytes_in(index as usize,field,&m.pvars,len)
    }

    pub fn shared_pvar_bytes(&self,id:MobyId,field:usize,len:usize)->Result<&[u8]> {
        let Some(&index)=self.pvar_indices.get(id).filter(|&&i|i>=0) else {return invalid("GC moby has no static pvar source");};
        let Some(m)=self.objects.get(id) else {return invalid("GC pvar owner absent");};
        self.pvars.shared_bytes_in(index as usize,field,&m.pvars,len)
    }

    /// Original0x31b8d0. Substate=None represents source argument -1; it
    /// preserves+95 and flagbit1 while every transition clears flagbit0.
    pub fn set_state(&mut self,id:MobyId,state:u8,substate:Option<u8>) {
        let m=self.objects.get_mut(id).expect("live GC state owner");let gc=&mut self.gc[id];
        gc.previous_94=m.state;m.state=state;gc.timer_96=0;gc.flags_be&=!1;
        if let Some(substate)=substate {gc.substate_95=substate;gc.flags_be&=!2;}
    }

    /// Hit delivery into the ACTUAL owner's GC hit slot, not a detached test
    /// variable or RAC1 hit record. Applying health/death remains the receiver.
    #[allow(clippy::too_many_arguments)]
    pub fn deliver_hit(&mut self,target:MobyId,damage:Pf,attacker:Option<MobyId>,flags:u32,
        point:[Pf;4],direction:[Pf;4])->Result<()> {
        let Some(m)=self.objects.get_mut(target).filter(|m|m.state<0x80) else {return invalid("GC hit target is not a live owner");};
        if attacker.is_some_and(|id|id>u32::MAX as usize) {return invalid("GC hit attacker id overflow");}
        let mut slot=crate::hits::HitSlot::from_raw(m.hit_slot);
        self.hits.deliver(crate::hits::MobyId(target as u32),&mut slot,damage,
            attacker.map(|i|crate::hits::MobyId(i as u32)),flags,point,direction);
        m.hit_slot=slot.raw();Ok(())
    }

    /// Source-bound native dispatch. 0x383b90 is completely ported.
    /// 0x383bb0 supports its original0/1 link-wait states; state2's effect
    /// and audio calls remain an explicit blocker, not a fake completion.
    pub fn native_update(&mut self,id:MobyId,source:u32,counter:u32)->bool {
        if !self.objects.get(id).is_some_and(|m|m.update_fn==Some(source)) {return false;}
        match source {
            0x383b90=>{
                let grid=&mut self.grid;
                self.objects.delete(id,counter,|m|grid.remove(m)).expect("scheduler supplied live GC moby");true
            },
            0x383bb0=>{
                let m=self.objects.get(id).expect("live GC link-wait owner");
                match m.state {
                    0=>{self.set_state(id,1,None);true},
                    1=>{
                        let b=Buf(&m.pvars);let (Ok(a),Ok(c))=(b.i32(4),b.i32(8)) else {return false;};
                        // The source intentionally requires BOTH ids >0;
                        // index0 (Ratchet) is not a null-host-pointer shortcut.
                        if a<=0 || c<=0 {return true;}
                        let Some(a)=self.objects.get(a as usize) else {return false;};
                        let Some(c)=self.objects.get(c as usize) else {return false;};
                        if matches!(a.state,0xfd|0xfe) || matches!(c.state,0xfd|0xfe) {self.set_state(id,2,None);}
                        true
                    },
                    2=>false,
                    _=>true, // Verified original switch default, no calls.
                }
            },
            0x383ce0=>{
                let m=self.objects.get(id).expect("GC sprite receiver owner");
                match m.state {
                    0=>{
                        let Some(effects)=self.effects.as_mut() else {return false;};
                        if effects.spawn_aranos_12fa(id,&self.objects).is_err() {return false;}
                        self.set_state(id,1,None);true
                    },
                    1=>{
                        let mut slot=crate::hits::HitSlot::from_raw(m.hit_slot);
                        let hit=self.hits.find(crate::hits::MobyId(id as u32),&mut slot,0x04010000,false).is_some();
                        let Ok(gate)=Buf(&m.pvars).i32(0) else {return false;};
                        if hit && gate==0 {self.set_state(id,2,None);}
                        self.objects.get_mut(id).unwrap().hit_slot=255;true
                    },
                    // Original debris/impact/audio calls are mandatory.
                    2=>false,
                    _=>true,
                }
            },
            _=>false,
        }
    }

    /// Original load-pass order, 0x32e290. Preflight a missing callback before
    /// advancing its animation so retries resume at exactly the same object.
    /// The callback may mutate the live world and allocate dynamic objects.
    pub fn initialize(&mut self,classes:&[LoadedClass<'_>],
        mut callback:impl FnMut(&mut Self,MobyId,u32)->bool)->std::result::Result<(),TickError> {
        while let Some(&id)=self.init_list.get(self.init_cursor) {
            self.update_one(classes,id,true,&mut callback)?;self.init_cursor+=1;
        }
        self.initialized=true;Ok(())
    }

    fn update_one(&mut self,classes:&[LoadedClass<'_>],id:MobyId,initializing:bool,
        callback:&mut impl FnMut(&mut Self,MobyId,u32)->bool)->std::result::Result<(),TickError> {
        let m=self.objects.get(id).ok_or(TickError::InvalidClass(-1))?;
        if m.state>=0x80 {return Ok(());}
        let class=classes.iter().find(|c|c.class==m.o_class).ok_or(TickError::InvalidClass(m.o_class))?;
        // Callback availability is explicit. Returning false means no update
        // was executed and animation must not advance. A successful port owns
        // its source identity and is responsible for the complete callback.
        let before=m.anim;
        if m.mode&mode::NO_ANIM==0 {rc_formats::moby_anim::advance(&mut self.objects.get_mut(id).unwrap().anim,class.animation);}
        if let Some(source)=self.objects.get(id).unwrap().update_fn {
            if !callback(self,id,source) {
                self.objects.get_mut(id).unwrap().anim=before;
                return Err(TickError::Missing(MissingUpdate {moby:id,class:class.class,source,
                    state:self.objects.get(id).unwrap().state,initializing}));
            }
        }
        let m=self.objects.get_mut(id).unwrap();
        if m.mode&mode::KEEP_MATRIX==0 {scheduler::rebuild_matrix(m,Some(class.animation));self.grid.register(m);}
        Ok(())
    }

    /// GC 0x313718 (240 class slots / 240 group flags), NOT RAC1's 224/112
    /// limits. Reuses its compatible pure distance arithmetic only.
    pub fn active(&self,camera:[f32;4])->std::result::Result<Active,TickError> {
        let mut slots=vec![Vec::new();240];let mut flagged=vec![false;self.groups.len()];let mut targets=Vec::new();
        let mut add=|id:usize,m:&Moby|->std::result::Result<(),TickError> {
            let Some(slot)=slots.get_mut(m.class_slot as usize) else {return Err(TickError::InvalidClass(m.o_class));};
            slot.push(id);if m.mode&mode::TARGETABLE!=0 {targets.push(id);}Ok(())
        };
        for (id,m) in self.objects.records().iter().enumerate() {
            if m.state==255 {break;}
            if m.state>=0x80 || m.mode&mode::NO_UPDATE!=0 || !scheduler::is_active(m,camera.map(Pf::f)) {continue;}
            if m.group>=0 {
                let Some(flag)=flagged.get_mut(m.group as usize) else {return Err(TickError::InvalidClass(m.o_class));};*flag=true;
            } else {add(id,m)?;}
        }
        for (g,flag) in flagged.iter().enumerate() {if *flag {for &id in &self.groups[g] {
            let Some(m)=self.objects.get(id) else {return Err(TickError::InvalidClass(-1));};
            if m.state<0x80 && m.mode&mode::NO_UPDATE==0 {add(id,m)?;}
        }}}
        Ok(Active {updates:slots.into_iter().flatten().collect(),targets})
    }

    pub fn tick(&mut self,counter:u32,camera:[f32;4],classes:&[LoadedClass<'_>],
        mut callback:impl FnMut(&mut Self,MobyId,u32)->bool)->std::result::Result<Active,TickError> {
        if !self.initialized {return Err(TickError::NotInitialized);}
        self.objects.free_slot_pass(counter);
        let list=self.active(camera)?;
        for &id in &list.updates {self.update_one(classes,id,false,&mut callback)?;}
        if let Some(effects)=self.effects.as_mut() {effects.update(&self.objects,camera).map_err(|e|TickError::Effect(e.to_string()))?;}
        Ok(list)
    }
}

fn parse_groups(gameplay:&[u8],map:&[Option<usize>])->Result<Vec<Vec<usize>>> {
    let Some(bytes)=gameplay::section(gameplay,"moby_groups")? else {return Ok(Vec::new());};
    let b=Buf(bytes);let count=b.i32(0)?;let size=b.i32(4)?;
    if !(0..=240).contains(&count) || size<0 {return invalid("GC group extent is unsupported");}
    let data=b.sub(16+count as usize*4,size as usize,"GC group data")?;let mut groups=Vec::new();
    for i in 0..count as usize {
        let offset=b.i32(16+i*4)?;let mut group=Vec::new();
        if offset>=0 {
            let mut p=offset as usize;
            loop {
                let entry=data.u16(p)?;let Some(mapped)=map.get((entry&0x7fff) as usize) else {return invalid("GC group references absent placement");};
                if let Some(id)=mapped {group.push(*id);}
                if entry&0x8000!=0 {break;}p+=2;
            }
        }
        groups.push(group);
    }
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(records:Vec<Moby>)->World {
        let n=records.len();World {objects:Objects::new(records,0),instance_to_moby:(0..n).map(Some).collect(),
            moby_to_instance:(0..n).collect(),pvars:PvarPlan {blocks:Vec::new(),pointer_fields:Vec::new(),shared_data:Vec::new(),shared_fields:Vec::new()},
            pvar_indices:vec![-1;n],groups:vec![Vec::new();2],gc:vec![GcState::default();n],preceding_owners:Vec::new(),
            grid:rc_game::collision_query::MobyGrid::new(),effects:None,hits:Default::default(),init_list:Vec::new(),init_cursor:0,initialized:false}
    }
    fn object(id:u32,slot:u8)->Moby {
        let mut m=Moby::init_instance(id,slot as i16,None);m.class_slot=slot;m.mode=mode::TARGETABLE;m.visible=1;m
    }
    #[test]
    fn gc_active_pass_retains_target_order_groups_and_slot239() {
        let mut a=object(0,239);a.group=1;a.visible=0;a.position[0]=1000.;a.update_dist=1;
        let b=object(1,0);let c=object(2,239);let mut d=object(3,10);d.group=1;d.mode=0;
        let mut excluded=object(4,0);excluded.mode|=2;let mut dead=object(5,0);dead.state=0xfd;
        let mut w=fixture(vec![a,b,c,d,excluded,dead]);w.groups[1]=vec![3,0];
        let list=w.active([0.;4]).unwrap();assert_eq!(list.updates,vec![1,3,2,0]);assert_eq!(list.targets,vec![1,2,0]);
        w.objects.get_mut(2).unwrap().class_slot=240;assert_eq!(w.active([0.;4]),Err(TickError::InvalidClass(239)));
    }
    #[test]
    fn gc_state_helper_keeps_substate_on_minus_one_and_preserves_other_flags() {
        let mut w=fixture(vec![object(0,1)]);w.objects.get_mut(0).unwrap().state=7;
        w.gc[0]=GcState {previous_94:3,substate_95:9,timer_96:27,flags_be:255};
        w.set_state(0,2,None);assert_eq!(w.objects.get(0).unwrap().state,2);
        assert_eq!(w.gc[0],GcState {previous_94:7,substate_95:9,timer_96:0,flags_be:254});
        w.set_state(0,4,Some(6));assert_eq!(w.gc[0],GcState {previous_94:2,substate_95:6,timer_96:0,flags_be:252});
    }
    #[test]
    fn native_world_never_ticks_before_all_initializers_finish() {
        let mut w=fixture(vec![object(0,1)]);
        assert_eq!(w.tick(1,[0.;4],&[],|_,_,_|panic!("must not update")),Err(TickError::NotInitialized));
    }
}
