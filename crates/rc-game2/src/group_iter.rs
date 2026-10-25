//! Original GC31c9d8/31cac0 over the existing compacted World group bank.
//! GP+3008 IS1b1ff8 andGP+3010 IS1b2000: one cursor/current owner.
//! No guest pointer arithmetic, cloned object bank or independent scheduler.
use crate::objects::Objects;
use rc_game::moby_runtime::MobyId;
use rc_formats::{buf::{invalid,Buf,Result},rac2::gameplay};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct GroupIterator {
    /// Loader1b1e38, NOT the serialized header count. None remains unknown
    /// when no serialized offset caused the original loader to write it.
    pub limit:Option<usize>,
    /// OuterNone unknown; Some(None) is a known null original pointer.
    pub cursor:Option<Option<(usize,usize)>>,
    pub current:Option<Option<MobyId>>,
    pub last_index:Option<u16>,
}
impl GroupIterator {
    pub fn new(limit:Option<usize>)->Self {Self {limit,cursor:None,current:None,last_index:None}}
    pub fn from_gameplay(bytes:&[u8])->Result<Self> {
        let Some(section)=gameplay::section(bytes,"moby_groups")? else {return Ok(Self::new(None));};
        let b=Buf(section);let n=b.i32(0)?;
        if !(0..=240).contains(&n) {return invalid("GC group count outside240-entry original bank");}
        let mut limit=None;
        for i in 0..n as usize {if b.i32(16+i*4)? >=0 {limit=Some(i+1);}}
        Ok(Self::new(limit))
    }
    fn valid_group(&self,g:i32)->Result<bool> {
        let Some(limit)=self.limit else {return invalid("GC original group limit is unbound");};
        if g<0 || limit<g as usize {return Ok(false);}
        if g as usize>=240 {return invalid("GC original one-past240 group table read is unbound");}
        Ok(true)
    }
    fn accepts(objects:&Objects,id:MobyId,a2:i32,a3:i32)->Result<bool> {
        let m=objects.get(id).ok_or_else(||rc_formats::FormatError::Invalid("GC group member outside actual object bank".into()))?;
        let negative=m.state&0x80!=0;
        Ok(match (a2!=0,a3!=0) {(false,false)=>!negative,(false,true)=>false,(true,false)=>true,(true,true)=>negative})
    }
    fn at(&mut self,groups:&[Vec<MobyId>],objects:&Objects,g:usize,i:usize)->Result<MobyId> {
        let id=*groups.get(g).and_then(|m|m.get(i)).ok_or_else(||rc_formats::FormatError::Invalid("GC iterator cursor outside original compacted list".into()))?;
        if id>0x7fff || objects.get(id).is_none() {return invalid("GC group native member index invalid");}
        self.cursor=Some(Some((g,i)));self.last_index=Some(id as u16);self.current=Some(Some(id));Ok(id)
    }
    /// Exact first-owner call. Invalid group leaves previous global iterator
    /// state intact; empty group clears current/cursor but preserves index.
    pub fn begin(&mut self,groups:&[Vec<MobyId>],objects:&Objects,g:i32,a2:i32,a3:i32)->Result<Option<MobyId>> {
        if !self.valid_group(g)? {return Ok(None);}
        let g=g as usize;self.current=Some(None);
        if groups.get(g).is_none_or(Vec::is_empty) {self.cursor=Some(None);return Ok(None);}
        let first=self.at(groups,objects,g,0)?;
        if Self::accepts(objects,first,a2,a3)? {return Ok(Some(first));}
        self.next(groups,objects,first,a2,a3)
    }
    /// Exact next-owner call, including seek when caller is not the global
    /// current owner. The END bit is represented by the compacted last index;
    /// exhausted calls retain last cursor/index/current while returning None.
    pub fn next(&mut self,groups:&[Vec<MobyId>],objects:&Objects,from:MobyId,a2:i32,a3:i32)->Result<Option<MobyId>> {
        let current=self.current.ok_or_else(||rc_formats::FormatError::Invalid("GC group current pointer unbound".into()))?;
        let (g,mut i)=if current==Some(from) {
            self.cursor.flatten().ok_or_else(||rc_formats::FormatError::Invalid("GC current member has no group cursor".into()))?
        }else{
            let m=objects.get(from).ok_or_else(||rc_formats::FormatError::Invalid("GC group seek owner absent".into()))?;
            let g=m.group as u8 as i32;
            if !self.valid_group(g)? {return Ok(None);}
            let g=g as usize;self.current=Some(None);
            let Some(members)=groups.get(g).filter(|m|!m.is_empty()) else {self.cursor=Some(None);return Ok(None);};
            let mut found=None;
            for j in 0..members.len() {
                let id=self.at(groups,objects,g,j)?;
                //31cb64 tests END BEFORE matching the provided owner.
                if j+1==members.len() {return Ok(None);}
                if id==from {found=Some(j);break;}
            }
            let Some(i)=found else {return Ok(None);};(g,i)
        };
        loop {
            let members=groups.get(g).ok_or_else(||rc_formats::FormatError::Invalid("GC retained group absent".into()))?;
            if i+1>=members.len() {return Ok(None);}
            i+=1;let id=self.at(groups,objects,g,i)?;
            if Self::accepts(objects,id,a2,a3)? {return Ok(Some(id));}
        }
    }
}

impl crate::world::World {
    pub fn group_begin(&mut self,g:i32,a2:i32,a3:i32)->Result<Option<MobyId>> {
        self.group_iterator.begin(&self.groups,&self.objects,g,a2,a3)
    }
    pub fn group_next(&mut self,from:MobyId,a2:i32,a3:i32)->Result<Option<MobyId>> {
        self.group_iterator.next(&self.groups,&self.objects,from,a2,a3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::moby_runtime::Moby;
    fn objects()->Objects {
        Objects::new((0..4).map(|i| {let mut m=Moby::init_instance(i,0,None);m.group=0;m.state=if i%2==0 {0}else{0x80};m}).collect(),0)
    }
    #[test]
    fn filters_and_exhaustion_preserve_one_original_global_owner() {
        let o=objects();let groups=vec![vec![0,1,2,3]];
        for (a2,a3,expected) in [(0,0,vec![0,2]),(0,1,vec![]),(1,0,vec![0,1,2,3]),(1,1,vec![1,3])] {
            let mut it=GroupIterator::new(Some(1));let mut got=vec![];
            let mut current=it.begin(&groups,&o,0,a2,a3).unwrap();
            while let Some(id)=current {got.push(id);current=it.next(&groups,&o,id,a2,a3).unwrap();}
            assert_eq!(got,expected);assert_eq!(it.current,Some(Some(3)));assert_eq!(it.cursor,Some(Some((0,3))));assert_eq!(it.last_index,Some(3));
            let before=it.clone();assert_eq!(it.begin(&groups,&o,2,a2,a3).unwrap(),None);assert_eq!(it,before);
            assert_eq!(it.begin(&groups,&o,1,a2,a3).unwrap(),None);assert_eq!(it.current,Some(None));assert_eq!(it.cursor,Some(None));assert_eq!(it.last_index,Some(3));
        }
    }
    #[test]
    fn nonsequential_seek_writes_visited_members_and_end_before_identity() {
        let o=objects();let groups=vec![vec![0,1,2,3]];let mut it=GroupIterator::new(Some(1));
        it.begin(&groups,&o,0,1,0).unwrap();
        assert_eq!(it.next(&groups,&o,2,1,0).unwrap(),Some(3));
        it.begin(&groups,&o,0,1,0).unwrap();
        assert_eq!(it.next(&groups,&o,3,1,0).unwrap(),None);
        assert_eq!((it.cursor,it.current,it.last_index),(Some(Some((0,3))),Some(Some(3)),Some(3)));
        assert!(GroupIterator::new(None).begin(&groups,&o,0,0,0).is_err());
        assert!(GroupIterator::new(Some(240)).begin(&groups,&o,240,0,0).is_err());
    }
    #[test]
    fn seek_reads_unsigned_group_byte_even_when_shared_native_storage_is_negative() {
        let mut o=objects();for id in 0..4 {o.get_mut(id).unwrap().group=200u8 as i8;}
        let mut groups=vec![vec![];201];groups[200]=vec![0,1,2,3];
        let mut it=GroupIterator::new(Some(201));assert_eq!(it.begin(&groups,&o,0,1,0).unwrap(),None);
        assert_eq!(it.next(&groups,&o,0,1,0).unwrap(),Some(1));
        assert_eq!((it.current,it.cursor),(Some(Some(1)),Some(Some((200,1)))));
    }

}
