//! GC crate initializer through the existing World/Objects/HitLog/RNG/grid.
//! This adapter does not bypass an earlier initializer or claim state1 gameplay.
use crate::{world::World,objects::Objects,crate_init::{CrateServices,StackServices,InitResult}};
use rc_formats::{buf::{invalid,Result},collision::Collision};
use rc_game::{moby_runtime::MobyId,collision_query::{CollOutput,QueryFlags,coll_line_m}};

struct Services<'a> {
    world:&'a mut World,collision:&'a Collision,rigid:&'a crate::collision::LiveRigidCollision,
    counter:u32,missions:Option<&'a[i32]>,pending:Option<u32>,
}
impl CrateServices for Services<'_> {
    fn objects(&self)->&Objects {&self.world.objects}
    fn objects_mut(&mut self)->&mut Objects {&mut self.world.objects}
    fn counter(&self)->u32 {self.counter}
    fn physics_39dfd8(&mut self,_id:MobyId)->Result<bool> {self.pending=Some(0x39dfd8);Ok(false)}
    fn hit_31dce8(&mut self,id:MobyId,mask:u32,keep:bool)->Result<()> {
        self.world.find_hit(id,mask,keep)?;Ok(())
    }
    fn resolve_stack_link(&mut self,id:MobyId,field:usize)->Result<Option<Option<MobyId>>> {
        self.world.objects.moby_reference(id,field)
    }
    fn write_stack_link(&mut self,id:MobyId,field:usize,target:Option<MobyId>)->Result<bool> {
        self.world.objects.write_moby_reference(id,field,target)?;Ok(true)
    }
    fn stack_39d0c0(&mut self,id:MobyId)->Result<bool> {
        let key=(id,self.world.objects.generation(id).unwrap());
        let mut progress=self.world.crate_stacks.remove(&key).unwrap_or_default();
        let result=progress.resume(id,self);self.world.crate_stacks.insert(key,progress);
        match result? {InitResult::Complete=>Ok(true),InitResult::Pending(source)=>{self.pending=Some(source);Ok(false)}}
    }
    fn icon_39ff50(&mut self,_id:MobyId,_mode:i32)->Result<Option<u32>> {self.pending=Some(0x39ff50);Ok(None)}
}
impl StackServices for Services<'_> {
    fn rng(&mut self)->&mut rc_game::rng::Rng {&mut self.world.rng}
    fn line(&mut self,a:[f32;3],b:[f32;3],flags:u32,ignore:Option<MobyId>)->Result<Option<CollOutput>> {
        Ok(coll_line_m(self.collision,Some(&self.rigid.scene(self.world)),a,b,QueryFlags(flags),ignore))
    }
    fn mission_word(&mut self,id:u8)->Result<Option<i32>> {Ok(self.missions.and_then(|m|m.get(id as usize)).copied())}
    fn delete_311688(&mut self,id:MobyId)->Result<()> {
        let grid=&mut self.world.grid;self.world.objects.delete(id,self.counter,|m|grid.remove(m))
    }
}
impl World {
    /// Caller must retain original initialization order. Completing one owner
    /// here does not advance World.init_cursor; only initialize() owns that.
    /// Missing save/mission, grouped stack, carrier and icon services suspend.
    pub fn resume_crate_initializer(&mut self,id:MobyId,collision:&Collision,
        rigid:&crate::collision::LiveRigidCollision,counter:u32,mission_words:Option<&[i32]>)->Result<InitResult> {
        let Some(owner)=self.objects.get(id) else {return invalid("GC crate owner absent");};
        if owner.update_fn!=Some(crate::crate_init::UPDATE) {return invalid("GC crate callback identity differs");}
        let key=(id,self.objects.generation(id).unwrap());
        let mut progress=self.crate_initializers.remove(&key).unwrap_or_default();
        let mut services=Services {world:self,collision,rigid,counter,missions:mission_words,pending:None};
        let result=progress.resume(id,&mut services);
        let pending=services.pending;
        self.crate_initializers.insert(key,progress);
        let result=result?;
        self.pending_crate=match result {InitResult::Complete=>None,InitResult::Pending(source)=>Some((id,pending.unwrap_or(source)))};
        Ok(result)
    }
}
