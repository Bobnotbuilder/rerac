//! Original Aranos 0x360d28 allocation selection, used by class 0x1330.
//! GAME_SPECIFIC. This does not execute attachment lifetime/pose services.
//! Existing RAC1 MobyTable, animation rows and item attachment math are reused
//! by the caller; no second object owner or guest-memory runtime is introduced.
use rc_formats::{buf::{invalid,Result},font::{parse_overlay_sections,read_overlay}};

#[derive(Clone,Debug)]
pub struct Actor1330Data { tables:[[i16;4];7] }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct AttachmentAllocation {
    /// Signed original class argument to 0x311320; retained without interpretation.
    pub class:i16,
    /// The source ORs bit 0x8000 into this allocated object's +0x34.
    pub flag_8000:bool,
}
impl Actor1330Data {
    pub fn parse(level:u32,overlay:&[u8])->Result<Self> {
        if level!=0 {return invalid("class1330 attachment tables are Aranos-only");}
        let sections=parse_overlay_sections(overlay)?;
        for (pc,word) in [(0x360d80,0x24550d50u32),(0x360d88,0x8c427af8),
                          (0x360dcc,0x0c0c44c8),(0x360f94,0x84440000)] {
            if read_overlay(&sections,pc,4)!=Some(word.to_le_bytes().as_slice()) {
                return invalid("class1330 attachment source signature mismatch");
            }
        }
        let mut tables=[[0i16;4];7];
        for (i,t) in tables.iter_mut().enumerate() {
            let address=0x1a8f78+8*i as u32;
            // These must be initialized bytes, never a synthesized NOBITS bank.
            let Some(section)=sections.iter().find(|s|s.kind==1 && address.checked_sub(s.dest)
                .is_some_and(|offset|(offset as usize)+8<=s.data.len())) else {
                return invalid("class1330 initialized attachment table absent");
            };
            let off=(address-section.dest) as usize;
            for j in 0..4 {t[j]=i16::from_le_bytes(section.data[off+2*j..off+2*j+2].try_into().unwrap());}
        }
        Ok(Self {tables})
    }
    /// Exact allocation order of 0x360db8..0x360fa0. `source_word` is the
    /// explicit original 0x1a7af8 input, whose gameplay interpretation is unverified.
    /// 0x360cd0 must delete/clear the previous 20 entries before applying this
    /// plan to the existing World allocator. There are no RNG calls here.
    pub fn attachment_plan(&self,source_word:u32)->Vec<AttachmentAllocation> {
        let mut plan=Vec::with_capacity(14);
        let mut add=|class,flag_8000|plan.push(AttachmentAllocation {class,flag_8000});
        let pair=self.tables[0][((source_word>>2)&3) as usize];
        add(pair,false);add(pair,true);
        add(self.tables[1][((source_word>>4)&1) as usize],false);
        let pair=self.tables[2][((source_word>>6)&1) as usize];
        add(pair,false);add(pair,true);
        let pair=self.tables[3][((source_word>>7)&3) as usize];
        if pair!=0 {add(pair,false);add(pair,true);}
        let pair=self.tables[4][((source_word>>9)&1) as usize];
        add(pair,false);add(pair,true);
        let pair=self.tables[5][((source_word>>10)&3) as usize];
        add(pair,false);add(pair,true);
        if source_word&(1<<12)!=0 {add(0x10e2,false);}
        if source_word&(1<<13)!=0 {add(0x10e4,false);}
        let index=((source_word>>14)&3) as usize;
        if index!=0 {add(self.tables[6][index],false);}
        plan
    }
    pub fn tables(&self)->&[[i16;4];7] {&self.tables}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_selection_preserves_signed_classes_order_pairs_and_optional_branches() {
        let data=Actor1330Data {tables:[[10,11,12,13],[20,21,22,23],[30,31,32,33],
            [0,41,42,43],[50,51,52,53],[60,61,62,63],[70,71,72,-73]]};
        assert_eq!(data.attachment_plan(0).iter().map(|x|(x.class,x.flag_8000)).collect::<Vec<_>>(),
            vec![(10,false),(10,true),(20,false),(30,false),(30,true),(50,false),(50,true),(60,false),(60,true)]);
        let word=(3<<2)|(1<<4)|(1<<6)|(2<<7)|(1<<9)|(3<<10)|(1<<12)|(1<<13)|(3<<14);
        assert_eq!(data.attachment_plan(word).iter().map(|x|(x.class,x.flag_8000)).collect::<Vec<_>>(),
            vec![(13,false),(13,true),(21,false),(31,false),(31,true),(42,false),(42,true),
                (51,false),(51,true),(63,false),(63,true),(0x10e2,false),(0x10e4,false),(-73,false)]);
        assert!(data.attachment_plan(u32::MAX).len()<=14);
    }
}
