//! Original GC class-to-update binding, not a RAC1 class registry.
//! Aranos 0x32e1f8 scans 12-byte records at 0x2a2e80; 0x3002e4
//! registers the selected update in the loaded class slot. Addresses identify
//! source routines for independent native ports; they are never executed.
use crate::{buf::{invalid,Buf,Result},font::{parse_overlay_sections,OverlaySection}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ClassUpdate {
    pub class:i32,
    pub source_update:u32,
    /// Third word is retained as opaque source metadata, not called as init.
    pub source_metadata:u32,
}

#[derive(Clone,Debug)]
pub struct AranosClassUpdates { entries:Vec<ClassUpdate>,fallback:u32 }

fn read(sections:&[OverlaySection],address:u32,len:usize)->Result<&[u8]> {
    let mut found=None;
    for s in sections.iter().filter(|s|s.kind!=8) {
        if let Some(data)=address.checked_sub(s.dest).and_then(|o|s.data.get(o as usize..)).and_then(|b|b.get(..len)) {
            if found.replace(data).is_some() {return invalid("ambiguous GC class registry mapping");}
        }
    }
    found.ok_or_else(||crate::FormatError::Invalid(format!("unmapped GC class registry {address:#x}")))
}

impl AranosClassUpdates {
    pub fn parse(level:u32,overlay:&[u8])->Result<Self> {
        if level!=0 {return invalid("GC class registry address contract is validated only for Aranos level 0");}
        let sections=parse_overlay_sections(overlay)?;
        for (at,word) in [(0x32e1f8,0x3c02002a),(0x32e200,0x8c432e80),(0x32e280,0x8ca20004)] {
            if Buf(read(&sections,at,4)?).u32(0)?!=word {return invalid("GC class registry selector signature mismatch");}
        }
        let mut entries=Vec::new();
        for i in 0..512 {
            let b=Buf(read(&sections,0x2a2e80+i*12,12)?);
            let class=b.i32(0)?;let update=b.u32(4)?;
            if update!=0 && (update&3!=0 || !sections.iter().any(|s|s.kind==1 && update.checked_sub(s.dest).is_some_and(|o|(o as usize)<s.data.len()))) {
                return invalid("GC class update is not aligned mapped overlay code");
            }
            if class == -1 {return Ok(Self {entries,fallback:update});}
            if !(0..=0x7fff).contains(&class) {return invalid("invalid GC class registry id");}
            entries.push(ClassUpdate {class,source_update:update,source_metadata:b.u32(8)?});
        }
        invalid("GC class registry lacks bounded sentinel")
    }

    pub fn entries(&self)->&[ClassUpdate] {&self.entries}

    /// Preserve the original first-match/sentinel lookup, including a null
    /// update. Finding a binding does not mean its native port exists.
    pub fn source_update(&self,class:i32)->u32 {
        self.entries.iter().find(|e|e.class==class).map_or(self.fallback,|e|e.source_update)
    }
}
