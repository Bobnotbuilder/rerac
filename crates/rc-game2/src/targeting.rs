//! Normal-body GC melee target scoring (0x2ae400) and ordered selection
//! (0x2ae900). Reuses native PS2 math, not RAC1's aim-search gameplay rules.
//! The real GC active-target list, class initialization and optional visibility
//! query must come from the world. This module does not invent live targets.
use rc_formats::buf::{invalid,Result};
use rc_game::{hero::physics::{V4,dist3,len2,vsub},pad::{fast_arctan,fast_diff_rots},ps2v::Pf};

#[derive(Clone,Copy,Debug)]
pub struct Target {
    pub id:usize,
    /// Native moby table member, GC class at +0xaa.
    pub class:i16,
    /// Signed short of the original class header at +0x46.
    pub class_type:i16,
    pub mode:u16,
    pub position:V4,
    /// Only needed when the requested class type is 5 (0x31fa20 -> +0).
    pub creature_health:Option<Pf>,
}

#[derive(Clone,Copy,Debug)]
pub struct Query {
    pub hero_position:V4,
    pub hero_state:i32,
    pub ground_z:Pf,
    pub yaw:Pf,
    pub range:Pf,
    pub yaw_cone:Pf,
    pub pitch_cone:Pf,
    pub alternate_body:bool,
}

impl Query {
    fn check(&self)->Result<()> {
        if self.alternate_body || matches!(self.hero_state,1|0x1d) {
            return invalid("GC alternate-body/first-person targeting requires its original camera scoring path");
        }
        Ok(())
    }
}

/// 0x3204e8, for a verified member of the native moby table. The raw source
/// also checks that the pointer belongs to that array before testing class.
pub fn crate_class(class:i16)->bool {(class as u16).wrapping_sub(0x1f4)<0x29}

/// Source computes the scalar even for a rejected target. None here is the
/// original rejected output, which the search discards. Endpoints are inclusive.
pub fn score(query:&Query,target:&Target)->Result<Option<Pf>> {
    query.check()?;
    let mut from=query.hero_position;
    if query.hero_state==0x14 {from[2]=query.ground_z;}
    let distance=dist3(from,target.position);
    let delta=vsub(target.position,from);
    let yaw=fast_diff_rots(fast_arctan(delta[0],delta[1]),query.yaw);
    let pitch=fast_diff_rots(fast_arctan(len2(delta),delta[2]),Pf::ZERO);
    if query.range<distance || Pf::ZERO<query.yaw_cone && query.yaw_cone<yaw
        || Pf::ZERO<query.pitch_cone && query.pitch_cone<pitch {return Ok(None);}
    let mut value=distance+yaw*distance;
    // GC 0x2ae614: penalty is 0.8 * this query's range, unlike RAC1's fixed7.
    if crate_class(target.class) || target.class_type==0x16 {value+=query.range*Pf::b(0x3f4ccccd);}
    Ok(Some(value))
}

/// Wrench 0x2b776c requests type=-1 and no visibility query. Type5 alone
/// requires a target record with nonzero health; do not apply RAC1's global
/// health filter. Input must preserve the original active-list order.
pub fn select<'a>(query:&Query,targets:&'a [Target],class_type:i32,require_visibility:bool,
    mut visible:impl FnMut(&Target)->bool)->Result<Option<&'a Target>> {
    query.check()?;
    let mut best=Pf::b(0x4cbebc20);let mut selected=None;
    for target in targets {
        if class_type != -1 && target.class_type as i32!=class_type {continue;}
        if class_type==5 && target.creature_health.is_none_or(|v|v==Pf::ZERO) {continue;}
        if target.mode&0x1000==0 {continue;}
        let Some(value)=score(query,target)? else {continue;};
        if value<best && (!require_visibility || visible(target)) {best=value;selected=Some(target);}
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn query()->Query {Query {hero_position:[Pf::ZERO;4],hero_state:0,ground_z:Pf::ZERO,yaw:Pf::ZERO,
        range:Pf::f(20.),yaw_cone:Pf::f(-1.),pitch_cone:Pf::f(-1.),alternate_body:false}}
    fn target(id:usize,class:i16)->Target {Target {id,class,class_type:5,mode:0x1000,
        position:[Pf::ONE,Pf::ZERO,Pf::ZERO,Pf::ONE],creature_health:Some(Pf::ONE)}}
    #[test]
    fn gc_crate_penalty_scales_with_range_and_ties_keep_first() {
        let q=query();let crate_=target(1,0x1f4);let enemy=target(2,0x99c);
        let difference=score(&q,&crate_).unwrap().unwrap()-score(&q,&enemy).unwrap().unwrap();
        assert!((difference.to_f32()-16.).abs()<0.001);
        assert_ne!(difference,Pf::f(7.));
        let targets=[crate_,enemy,target(3,0x99c)];
        assert_eq!(select(&q,&targets,-1,false,|_|panic!("unrequested visibility query")).unwrap().unwrap().id,2);
        assert_eq!(select(&q,&targets,-1,true,|t|t.id!=2).unwrap().unwrap().id,3);
        assert!(crate_class(0x21c));assert!(!crate_class(0x21d));
    }
    #[test]
    fn health_filter_is_only_type_five_and_camera_modes_are_explicit() {
        let q=query();let targets=[Target {creature_health:None,..target(1,0x99c)}];
        assert_eq!(select(&q,&targets,-1,false,|_|true).unwrap().unwrap().id,1);
        assert!(select(&q,&targets,5,false,|_|true).unwrap().is_none());
        assert!(score(&Query{hero_state:1,..q},&targets[0]).is_err());
        assert!(score(&Query{alternate_body:true,..q},&targets[0]).is_err());
        let airborne=Target{position:[Pf::ONE,Pf::ZERO,Pf::f(3.),Pf::ONE],..targets[0]};
        let jump=Query{hero_state:0x14,ground_z:Pf::f(3.),..q};
        assert!(score(&jump,&airborne).unwrap().unwrap()<score(&q,&airborne).unwrap().unwrap());
    }
}
