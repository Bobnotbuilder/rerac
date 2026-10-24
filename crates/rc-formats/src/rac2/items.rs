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
        Ok(Self {definition_index:index,class,byte_1c:b.u8(0x1c)?,ammo_flag:b.u16(0x8e)?})
    }
}
