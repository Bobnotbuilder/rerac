//! Aranos item resources, explicitly separate from RAC1's 0x4c-byte definitions.
//! L00 0x2afc68 uses boot map 0x139568, overlay table 0x2624f0, stride 0xe0.
use crate::{buf::{invalid,Buf,Result},font::{parse_overlay_sections,read_overlay}};
use super::boot::BootElf;

pub const WRENCH_ITEM:i32=10;
pub const WRENCH_CLASS:i16=0x47;

#[derive(Clone,Debug)]
pub struct AranosWrenchDefinition {
    pub definition_index:u8,
    pub class:i32,
    /// Definition +0x1c, copied to hero+0x22bb by the slot initialization.
    pub byte_1c:u8,
    /// Definition +0x8e, tested by the original item ammo initialization.
    pub ammo_flag:u16,
    /// +0x70 / +0x74, read by class 0x47's ground / jump hit paths.
    pub ground_damage:f32,
    pub jump_damage:f32,
    /// Original +0x10 selects the nine-list attachment table used by 0x2aa1f0.
    pub attachment_list:usize,
}

impl AranosWrenchDefinition {
    /// Initial boot mapping only. Upgraded mutable item-map semantics are not
    /// inferred from this original static byte or applied to other levels.
    pub fn parse(level:u32,boot:&BootElf,overlay:&[u8])->Result<Self> {
        if level!=0 {return invalid("GC item address contract is validated only for Aranos level 0");}
        let index=boot.read(0x139568+WRENCH_ITEM as u32,1)?[0];
        let sections=parse_overlay_sections(overlay)?;
        let signature=read_overlay(&sections,0x2a0600,4).ok_or_else(||crate::FormatError::Invalid("GC wrench dispatcher unmapped".into()))?;
        if Buf(signature).u32(0)?!=0x2c9790 {return invalid("GC wrench dispatcher signature mismatch");}
        let data=read_overlay(&sections,0x2624f0+index as u32*0xe0,0xe0).ok_or_else(||crate::FormatError::Invalid("GC wrench definition unmapped".into()))?;
        let b=Buf(data);let class=b.i32(0x14)?;
        if class!=WRENCH_CLASS as i32 {return invalid("GC initial wrench definition does not reference original class 0x47");}
        if Buf(read_overlay(&sections,0x2aa260,4).ok_or_else(||crate::FormatError::Invalid("GC hand matrix call unmapped".into()))?).u32(0)?!=0x0c0c4a3c {
            return invalid("GC hand matrix call signature mismatch");
        }
        let attachment=b.i32(0x10)?;
        if !(0..9).contains(&attachment) {return invalid("GC wrench attachment index out of range");}
        let table=match read_overlay(&sections,0x1a7f68,9*4) {Some(b)=>b,None=>boot.read(0x1a7f68,9*4)?};
        let attachment_list=Buf(table).u32(attachment as usize*4)? as usize;
        if attachment_list>255 {return invalid("GC wrench attachment list out of range");}
        let (ground_damage,jump_damage)=(b.f32(0x70)?,b.f32(0x74)?);
        if !ground_damage.is_finite() || !jump_damage.is_finite() || ground_damage<=0. || jump_damage<=0. {
            return invalid("GC wrench damage definition is not positive finite");
        }
        Ok(Self {definition_index:index,class,byte_1c:b.u8(0x1c)?,ammo_flag:b.u16(0x8e)?,ground_damage,jump_damage,attachment_list})
    }
}
