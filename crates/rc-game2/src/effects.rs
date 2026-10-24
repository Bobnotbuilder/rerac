//! GC attached type0x59 sprite branch of0x33f550/0x33f6e8, required by
//! Aranos class0x12fa. Reuses the existing 2048-record native particle pool,
//! RNG and vector math. No RAC1 particle-type dispatcher executes.
use rc_formats::{buf::{invalid,Result},particle_tex::PartDefs};
use rc_game::{particles::{PartPool,rec},rng::Rng,ps2v::Pf,
    hero::physics::{vadd,vsub,vscale,set_len3}};
use crate::objects::Objects;

pub struct Effects {pub pool:PartPool,defs:PartDefs,pub rng:Rng}

impl Effects {
    /// RNG is the caller's stream at this point in the actual loader/tick.
    /// Seeding here would lose earlier original consumers.
    pub fn new(defs:PartDefs,rng:Rng)->Self {Self {pool:PartPool::new(),defs,rng}}

    /// Complete spawner branch used by0x383d44: attached moby, joint=-1,
    /// kind1, subtype0, lifetime=-1 and draw distance96.
    pub fn spawn_aranos_12fa(&mut self,id:usize,objects:&Objects)->Result<Option<usize>> {
        let Some(owner)=objects.get(id) else {return invalid("GC attached sprite owner absent");};
        let Some(start)=self.defs.start(0x59) else {return invalid("GC particle89 definition absent");};
        let Some(&texture)=self.defs.blob.get(start+1) else {return invalid("GC particle89 kind1 texture absent");};
        let Some(i)=self.pool.create_part(0x59) else {return Ok(None);};
        // GC0x330f20 clears ALL64 bytes. RAC1's allocator intentionally
        // preserves bytes20..3f on reuse; adapt that difference locally.
        let r=&mut self.pool.recs[i];r.fill(0);r[0]=0x59;
        rec::set_u32(r,4,0x32608080);r[3]=0x48;
        rec::set_ff(r,0xc,(Pf::f(owner.scale)*Pf::b(0x4aa037a0)).to_f32());
        r[8]=self.rng.randi(255) as u8;r[9]=0x34;r[2]=texture;
        rec::set_i16(r,10,-1);
        // A stable native moby id replaces the original pointer; +1 keeps
        // id0 distinct from the source null pointer.
        rec::set_u32(r,0x20,id as u32+1);rec::set_u32(r,0x24,u32::MAX);
        rec::set_ff(r,0x28,(Pf::f(owner.scale)*Pf::f(15.)).to_f32());
        let offset=[Pf::ZERO,Pf::ZERO,Pf::f(-23.)*Pf::f(owner.scale),Pf::ZERO];
        rec::set_v4(r,0x30,offset.map(Pf::to_f32));
        let p=vadd(owner.position.map(Pf::f),offset);
        rec::set_v4(r,0x10,p.map(Pf::to_f32));Ok(Some(i))
    }

    /// Only the attached, no-joint, no-alpha/orientation branch above is
    /// accepted. Unknown records are retained with an error, never killed as
    /// a substitute for implementing their update.
    pub fn update(&mut self,objects:&Objects,camera:[f32;4])->Result<()> {
        let end=(self.pool.hw+1).max(0) as usize;
        for i in 0..end {
            let r=&self.pool.recs[i];if r[1]&0x80!=0 {continue;}
            if r[0]!=0x59 || rec::i16(r,0x24)!=-1 || rec::i16(r,0x2c)!=0 || rec::i16(r,0x2e)!=0 || rec::i16(r,10)!=-1 {
                return invalid("GC sprite update requires another original branch");
            }
            let Some(id)=(rec::u32(r,0x20) as usize).checked_sub(1) else {return invalid("GC attached sprite lacks native owner");};
            let Some(m)=objects.get(id) else {return invalid("GC attached sprite owner out of range");};
            if m.state>=0x80 {self.pool.kill_part(i);continue;}
            let offset=rec::v4(r,0x30).map(Pf::f);
            //0x2ef318 is the shared VU row-vector transform, in its original
            // multiply/add order (no translation in this part).
            let rotated=std::array::from_fn(|k|if k==3 {Pf::ZERO}else {
                (Pf::f(m.rows[0][k])*offset[0]+Pf::f(m.rows[1][k])*offset[1])+Pf::f(m.rows[2][k])*offset[2]
            });
            let position=vadd(m.position.map(Pf::f),rotated);
            let toward=set_len3(vsub(camera.map(Pf::f),position),Pf::f(rec::ff(r,0x28)));
            let p=vadd(position,vscale(toward,Pf::ONE));
            rec::set_v4(&mut self.pool.recs[i],0x10,p.map(Pf::to_f32));
        }
        Ok(())
    }
}
