//! GC loaded-class and dynamic-slot adapters over the existing native storage.
//! The common loaded-header initialization matches 0x311418..0x311684.
//! No RAC1 spawn tests, class registry, scheduler or gameplay callbacks run.
use rc_formats::{buf::{invalid,Result},level::LevelCore,moby::MobyClass,moby_anim::MobyAnimClass,
    rac2::classes::AranosClassUpdates,Game};
use rc_game::{moby_runtime::{ClassInfo,Moby,MobyId,MobyTable,Seq0Info},moby_update::scheduler};

pub struct LoadedClass<'a> {pub(crate) class:i16,pub(crate) info:ClassInfo,pub(crate) animation:&'a MobyAnimClass}

/// Owned identity/header state retained by a host between ticks. Animation
/// storage stays in the existing level resource; no model/pose bank is copied.
#[derive(Clone,Copy)]
pub struct ClassBinding {class:i16,info:ClassInfo}
impl ClassBinding {
    pub fn bind<'a>(&self,animation:&'a MobyAnimClass)->LoadedClass<'a> {
        LoadedClass {class:self.class,info:self.info,animation}
    }
}

impl<'a> LoadedClass<'a> {
    /// The core's loaded class order provides the native slot. The GC source
    /// callback identity comes only from its original registry, not RAC1's.
    pub fn from_core(level:u32,core:&LevelCore,slot:usize,class:&MobyClass,anim:&'a MobyAnimClass,
        updates:&AranosClassUpdates)->Result<Self> {
        if level!=0 {return invalid("GC source update binding is validated only for Aranos level 0");}
        if core.game!=Game::Rac2 {return invalid("GC class adapter requires explicitly tagged GC core");}
        let Some(entry)=core.moby_classes.get(slot) else {return invalid("GC loaded class slot absent");};
        if entry.offset_in_asset_wad<=0 || !(0..=0x7fff).contains(&entry.o_class) {
            return invalid("GC loaded class slot has no supported header");
        }
        // 0x300208 has independent counters for loaded headers (+1e00)
        // and headerless bindings (+1e04). Core-entry index is NOT the slot
        // when a headerless class occurs earlier in the original table.
        let native_slot=core.moby_classes[..slot].iter().filter(|c|c.offset_in_asset_wad>0).count();
        if native_slot>=240 {return invalid("GC loaded class exceeds original slot bank");}
        let update=updates.source_update(entry.o_class);
        // This is a pure header view, not scheduler execution. Common native
        // fields have the same semantics despite changed raw GC moby offsets:
        // class +aa, hit +a8, update +64, pvar +68 and animation +40..+60.
        let mut info=scheduler::class_info(class,native_slot as u8,(update!=0).then_some(update));
        info.seq0=anim.sequence(0).map(|q|Seq0Info {frame_count:q.header.frame_count,
            loop_sound_bit7:q.header.loop_sound&0x80!=0});
        Ok(Self {class:entry.o_class as i16,info,animation:anim})
    }

    pub fn class(&self)->i16 {self.class}
    pub fn source_update(&self)->Option<u32> {self.info.update_fn}
    pub fn binding(&self)->ClassBinding {ClassBinding {class:self.class,info:self.info}}

    /// 0x300234/0x32e244 and InitMobyInstance's mismatch branch0x3114e4.
    /// This is a real GC logic-only class, not a missing external mesh
    /// substituted with fabricated geometry. The empty animation view cannot
    /// render anything and only participates in generic no-op pose bookkeeping.
    pub fn without_header(level:u32,core:&LevelCore,entry:usize,empty:&'a MobyAnimClass,
        updates:&AranosClassUpdates)->Result<Self> {
        if level!=0 || core.game!=Game::Rac2 || empty.joint_count!=0 || !empty.sequences.is_empty() {
            return invalid("GC headerless binding requires Aranos and an empty animation view");
        }
        let Some(c)=core.moby_classes.get(entry) else {return invalid("GC headerless entry absent");};
        if c.offset_in_asset_wad>0 || !(0..=0x7fff).contains(&c.o_class) {return invalid("GC headerless binding has a header or invalid class");}
        let slot=core.moby_classes[..entry].iter().filter(|c|c.offset_in_asset_wad<=0).count();
        if slot>=240 {return invalid("GC headerless class exceeds original slot bank");}
        let update=updates.source_update(c.o_class);
        Ok(Self {class:c.o_class as i16,info:ClassInfo {slot:slot as u8,no_header:true,
            update_fn:(update!=0).then_some(update),..Default::default()},animation:empty})
    }
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
    pub fn records(&self)->&[Moby] {&self.table.mobys}

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
