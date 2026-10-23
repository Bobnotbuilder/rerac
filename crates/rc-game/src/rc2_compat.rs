//! Controlled GC data compatibility path through existing native engine types.
//! Does not construct `tick::Game`, call RAC1 initialization/update functions,
//! access saves, or permit RC2 Play. Metadata compatibility is not gameplay parity.
use crate::moby_runtime::{mode, Moby, MobyTable};
use rc_formats::{buf::{invalid, Result}, collision::{self, Collision}, moby::{self, LevelMobyClass}, rc2};

pub struct Scene {
    pub core: rc2::Core,
    pub classes: Vec<LevelMobyClass>,
    pub objects: Vec<rc2::Object>,
    pub collision: Collision,
    /// Existing native storage, metadata only; no RAC1 load pass or scheduler.
    pub metadata_table: MobyTable,
}

impl Scene {
    pub fn read(index: &[u8], data: &[u8], gameplay: &[u8]) -> Result<Self> {
        let core=rc2::core(index,data.len())?;
        let bounds=rc2::core_block_bounds(index,&core)?;
        let collision=collision::parse_collision_block(&data[rc2::core_block_range(&bounds,core.collision)?])?;
        let mut classes=Vec::new();
        for entry in &core.mobys {
            let at=entry.offset_in_asset_wad as usize;
            if at==0 { continue; }
            let class=moby::parse_gc_moby_class(&data[rc2::core_block_range(&bounds,at)?])?;
            classes.push(LevelMobyClass { o_class:entry.o_class,entry:*entry,class });
        }
        let objects=rc2::objects(gameplay)?;
        let metadata_table=metadata_table(&objects,&core)?;
        Ok(Self {core,classes,objects,collision,metadata_table})
    }
}

fn metadata_table(objects: &[rc2::Object],core: &rc2::Core) -> Result<MobyTable> {
    let mut records=Vec::new();
    for (index,o) in objects.iter().enumerate() {
        let Some(slot)=core.mobys.iter().position(|v|v.o_class==o.class) else { return invalid("GC_COMPAT_OBJECT_CLASS_MISSING"); };
        let mut m=Moby::zeroed();
        m.o_class=i16::try_from(o.class).map_err(|_|rc_formats::FormatError::Invalid("GC_COMPAT_CLASS_WIDTH".into()))?;
        m.class_slot=u8::try_from(slot).map_err(|_|rc_formats::FormatError::Invalid("GC_COMPAT_SLOT_WIDTH".into()))?;
        m.position=[o.position[0],o.position[1],o.position[2],1.];
        m.rotation=[o.rotation[0],o.rotation[1],o.rotation[2],0.];
        m.scale=o.scale; // raw instance scale, NOT RAC1's class-scale initialization.
        m.index=index as u32;
        m.has_class=core.mobys[slot].offset_in_asset_wad>0;
        m.mode=mode::NO_UPDATE|mode::NO_ANIM;
        m.anim.skip_advance=true;
        // Mission/pvar indices stay in `objects`; do not coerce their GC meaning
        // into RAC1's mission bytes, pvar layouts or update addresses.
        records.push(m);
    }
    Ok(MobyTable::new(records,0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_gc_data_cannot_enter_native_gameplay() {
        assert!(Scene::read(&[],&[],&[]).is_err());
        assert!(Scene::read(&[0;188],&[0;1024],&[0;160]).is_err());
    }
}
