//! Native GC pvar copy/link plan from 0x302bc0..0x302e10. The caller supplies
//! which blocks were actually copied and the original loader's spawn mapping.
//! No spawn condition, class initializer or host pointer is fabricated here.
//! Per-class pointer interpretation and ownership allocation remain unported;
//! this plan alone is not a completed gameplay/class loader.
use rc_formats::{buf::{invalid,Buf,Result},rac2::gameplay};

pub struct PvarPlan {
    pub blocks:Vec<Option<Vec<u8>>>,
    /// Source adds a block address at these fields. Native code keeps each raw
    /// block-relative offset and this metadata, rather than storing PS2/host pointers.
    pub pointer_fields:Vec<(i32,i32)>,
    pub shared_data:Vec<u8>,
    pub shared_fields:Vec<(usize,usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_relocation_overrides_local_pointer_kind_without_host_addresses() {
        let mut data=vec![0;0xec];
        let word=|data:&mut [u8],at:usize,value:u32|data[at..at+4].copy_from_slice(&value.to_le_bytes());
        for (at,value) in [(19*4,0xa0),(21*4,0xb0),(23*4,0xcc),(24*4,0xd4),(25*4,0xdc),
            (0xb0,4),(0xb4,1),(0xd0,8),(0xd4,4),(0xe4,u32::MAX)] {word(&mut data,at,value);}
        data[0xc0..0xc4].copy_from_slice(&[9,8,7,6]);
        let plan=PvarPlan::copy_and_link(&data,&[0],&[]).unwrap();
        assert!(plan.pointer_fields.is_empty());assert_eq!(plan.shared_fields,vec![(0,0)]);
        assert_eq!(plan.shared_bytes(0,0,4).unwrap(),[9,8,7,6]);
        assert!(plan.local_bytes(0,0,4).is_err());assert!(plan.shared_bytes(0,4,1).is_err());
        assert!(plan.shared_bytes(0,0,5).is_err());
        assert_eq!(Buf(gameplay::unrelocated_pvar(&data,0).unwrap().unwrap()).i32(0).unwrap(),4);
    }
}

impl PvarPlan {
    /// Resolve only a verified original block-relative field. Offset0 means
    /// the block base after the original relocation, not a null pointer.
    /// Cross-block/shared-data pointers need a separate validated adapter.
    pub fn local_bytes(&self,index:usize,field:usize,len:usize)->Result<&[u8]> {
        if index>i32::MAX as usize || field>i32::MAX as usize
            || !self.pointer_fields.contains(&(index as i32,field as i32)) {
            return invalid("GC pvar field is not a verified local pointer");
        }
        let Some(Some(block))=self.blocks.get(index) else {return invalid("GC local pointer block was not copied");};
        let offset=Buf(block).i32(field)?;
        let offset=usize::try_from(offset).map_err(|_|rc_formats::FormatError::Invalid("GC local pointer precedes its block".into()))?;
        Ok(Buf(block).sub(offset,len,"GC local pointer range")?.bytes())
    }

    /// GC 0x302fd8..0x302fe4 writes a pointer into the shared blob, overriding
    /// earlier relocation at that field. Native bytes retain the blob offset.
    pub fn shared_bytes(&self,index:usize,field:usize,len:usize)->Result<&[u8]> {
        if !self.shared_fields.contains(&(index,field)) {return invalid("GC pvar field is not a verified shared pointer");}
        let Some(Some(block))=self.blocks.get(index) else {return invalid("GC shared pointer block was not copied");};
        let offset=usize::try_from(Buf(block).i32(field)?).map_err(|_|rc_formats::FormatError::Invalid("GC shared pointer precedes blob".into()))?;
        Ok(Buf(&self.shared_data).sub(offset,len,"GC shared pointer range")?.bytes())
    }

    pub fn copy_and_link(gameplay:&[u8],copied:&[i32],instance_to_moby:&[Option<usize>])->Result<Self> {
        let instances=gameplay::moby_instances(gameplay)?;
        if instance_to_moby.len()!=instances.len() {return invalid("GC pvar link plan requires an explicit complete instance map");}
        if instance_to_moby.iter().flatten().any(|&id|id>i16::MAX as usize) {return invalid("GC pvar moby id exceeds original signed map");}
        let table=gameplay::section(gameplay,"pvar_table")?.unwrap_or(&[]);
        let mut blocks=vec![None;table.len()/8];
        for &index in copied {
            if index == -1 {continue;}
            if index<0 || index as usize>=blocks.len() {return invalid("GC copied pvar index out of range");}
            blocks[index as usize]=gameplay::unrelocated_pvar(gameplay,index)?.map(<[u8]>::to_vec);
        }
        let fixups=gameplay::pvar_fixups(gameplay)?;
        for &(index,offset) in &fixups.moby_links {
            if index as usize>=blocks.len() {return invalid("GC moby-link pvar index out of range");}
            let Some(Some(block))=blocks.get_mut(index as usize) else {continue;};
            let offset=usize::try_from(offset).map_err(|_|rc_formats::FormatError::Invalid("negative GC moby-link field".into()))?;
            let old=Buf(block).i32(offset)?;
            if old>=0 {
                let Some(mapped)=instance_to_moby.get(old as usize) else {return invalid("GC pvar link references absent placement");};
                let mapped=mapped.map_or(-1,|id|id as i32);
                block[offset..offset+4].copy_from_slice(&mapped.to_le_bytes());
            }
        }
        let mut pointer_fields=Vec::new();
        for (index,offset) in fixups.pointers {
            if index as usize>=blocks.len() {return invalid("GC relative-pointer pvar index out of range");}
            let Some(Some(block))=blocks.get(index as usize) else {continue;};
            let field=usize::try_from(offset).map_err(|_|rc_formats::FormatError::Invalid("negative GC relative-pointer field".into()))?;
            Buf(block).i32(field)?;pointer_fields.push((index,offset));
        }
        let shared=gameplay::shared_pvars(gameplay)?;let mut shared_fields=Vec::new();
        for (index,field,offset) in shared.fields {
            if index as usize>=blocks.len() {return invalid("GC shared pvar index out of range");}
            let Some(Some(block))=blocks.get_mut(index as usize) else {continue;};
            let field=usize::try_from(field).map_err(|_|rc_formats::FormatError::Invalid("GC shared field precedes pvar block".into()))?;
            Buf(block).i32(field)?;block[field..field+4].copy_from_slice(&offset.to_le_bytes());
            pointer_fields.retain(|&(i,o)|i!=index as i32 || o!=field as i32);
            shared_fields.push((index as usize,field));
        }
        Ok(Self {blocks,pointer_fields,shared_data:shared.data,shared_fields})
    }
}
