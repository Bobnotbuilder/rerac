//! GC loaded-class and dynamic-slot adapters over the existing native storage.
//! The common loaded-header initialization matches 0x311418..0x311684.
//! No RAC1 spawn tests, class registry, scheduler or gameplay callbacks run.
use rc_formats::{buf::{invalid,Result},level::LevelCore,moby::MobyClass,moby_anim::MobyAnimClass,
    rac2::classes::AranosClassUpdates,Game};
use rc_game::{moby_runtime::{ClassInfo,Moby,MobyId,MobyTable,Seq0Info},moby_update::scheduler};

pub struct LoadedClass<'a> {class:i16,info:ClassInfo,animation:&'a MobyAnimClass}

impl<'a> LoadedClass<'a> {
    /// The core's loaded class order provides the native slot. The GC source
    /// callback identity comes only from its original registry, not RAC1's.
    pub fn from_core(level:u32,core:&LevelCore,slot:usize,class:&MobyClass,anim:&'a MobyAnimClass,
        updates:&AranosClassUpdates)->Result<Self> {
        if level!=0 {return invalid("GC source update binding is validated only for Aranos level 0");}
        if core.game!=Game::Rac2 {return invalid("GC class adapter requires explicitly tagged GC core");}
        let Some(entry)=core.moby_classes.get(slot) else {return invalid("GC loaded class slot absent");};
        if slot>=255 || entry.offset_in_asset_wad<=0 || !(0..=0x7fff).contains(&entry.o_class) {
            return invalid("GC loaded class slot has no supported header");
        }
        let update=updates.source_update(entry.o_class);
        // This is a pure header view, not scheduler execution. Common native
        // fields have the same semantics despite changed raw GC moby offsets:
        // class +aa, hit +a8, update +64, pvar +68 and animation +40..+60.
        let mut info=scheduler::class_info(class,slot as u8,(update!=0).then_some(update));
        info.seq0=anim.sequence(0).map(|q|Seq0Info {frame_count:q.header.frame_count,
            loop_sound_bit7:q.header.loop_sound&0x80!=0});
        Ok(Self {class:entry.o_class as i16,info,animation:anim})
    }

    pub fn class(&self)->i16 {self.class}
    pub fn source_update(&self)->Option<u32> {self.info.update_fn}
}

pub struct Objects {table:MobyTable}

impl Objects {
    /// Static records must come from the GC loader. This does not apply spawn
    /// conditions or pvar fixups to original placements.
    pub fn new(static_records:Vec<Moby>,dynamic_capacity:usize)->Self {
        Self {table:MobyTable::new(static_records,dynamic_capacity)}
    }
    pub fn get(&self,id:MobyId)->Option<&Moby> {self.table.mobys.get(id)}
    pub fn get_mut(&mut self,id:MobyId)->Option<&mut Moby> {self.table.mobys.get_mut(id)}

    /// GC 0x311320: same first-reusable-slot allocation and zeroed 0x80-byte
    /// pvar block. Reuse the existing native allocator and semantic initializer.
    /// A source_update identity does not grant a native class implementation.
    pub fn create(&mut self,class:&LoadedClass<'_>,counter:u32)->Option<MobyId> {
        let id=self.table.create(class.class,Some(&class.info),counter as u64)?;
        // GC 0x311600 -> 0x3116e0, shared sequence-header bookkeeping only.
        // No RAC1 sound lookup or class update executes in this helper.
        rc_game::moby_update::anim_sound::init(&mut self.table.mobys[id],Some(class.animation));
        Some(id)
    }

    /// Slot bookkeeping of GC 0x311288; no class behavior executes here.
    pub fn free_slot_pass(&mut self,counter:u32) {self.table.free_slot_pass(counter as u64);}

    /// GC 0x311688: shared state markers, with GC's u32 deletion tick +a0.
    /// Spatial removal (0x313ae0) is supplied by the native GC world owner.
    pub fn delete(&mut self,id:MobyId,counter:u32,mut remove_from_grid:impl FnMut(&mut Moby))->Result<()> {
        if id>=self.table.mobys.len() {return invalid("GC delete slot out of range");}
        self.table.delete(id,counter as u64);
        self.table.mobys[id].delete_tick=counter.wrapping_add(2) as u64;
        remove_from_grid(&mut self.table.mobys[id]);
        Ok(())
    }

    pub fn free_slots(&self)->i32 {self.table.free_slots}
}
