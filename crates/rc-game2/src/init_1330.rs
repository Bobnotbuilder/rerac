//! Original Aranos class1330 initialization checkpoints. A completed prefix is
//! NOT a completed callback: the native world retains its object cursor until
//! every required service, including HeroTeleport, has actually executed.
use rc_formats::{buf::{invalid,Buf,Result},rac2::gameplay,volumes::Shape};
use rc_game::moby_runtime::MobyId;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct TeleportRequest {
    pub owner:MobyId,
    pub cuboid:usize,
    pub position:[f32;4],
    pub euler:[f32;4],
    pub state:i32,
    pub reset_camera:bool,
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Pending1330 {
    RuntimeState,
    Teleport(TeleportRequest),
    ActorAndRemainingServices,
    InvalidOwnedData,
}

/// GC loader303210..3032a4 copies header+68 records to1b1f0c, stride80.
/// This is the original cuboid table, NOT the camera table. The existing
/// native Shape layout and bounded POD reader are compatible unchanged.
pub fn cuboids(gameplay:&[u8])->Result<Vec<Shape>> {
    let Some(section)=gameplay::section(gameplay,"cuboids")? else {return Ok(Vec::new());};
    let b=Buf(section);let count=b.i32(0)?;
    if !(0..=0x10000).contains(&count) {return invalid("GC cuboid count invalid");}
    b.pod_slice::<Shape>(16,count as usize,"GC cuboid records")
}

pub fn teleport_request(owner:MobyId,pvars:&[u8],cuboids:&[Shape])->Result<Option<TeleportRequest>> {
    let b=Buf(pvars);
    // The complete state0 branch also reads/writes through +20.
    b.sub(0,0x24,"GC class1330 owned pvars")?;
    let index=b.i32(4)?;
    if index == -1 {return Ok(None);}
    let Some(shape)=usize::try_from(index).ok().and_then(|i|cuboids.get(i)) else {
        return invalid("GC class1330 cuboid selector out of range");
    };
    Ok(Some(TeleportRequest {owner,cuboid:index as usize,position:shape.matrix[3],
        euler:[shape.euler[0],shape.euler[1],shape.euler[2],shape.unused_7c],state:0,reset_camera:true}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_selector_is_exact_minus_one_and_all_other_selectors_are_bounded() {
        let mut pvars=vec![0;0x24];
        pvars[4..8].copy_from_slice(&(-1i32).to_le_bytes());
        assert_eq!(teleport_request(24,&pvars,&[]).unwrap(),None);
        pvars[4..8].copy_from_slice(&(-2i32).to_le_bytes());
        assert!(teleport_request(24,&pvars,&[]).is_err());
        pvars[4..8].copy_from_slice(&0i32.to_le_bytes());
        assert!(teleport_request(24,&pvars,&[]).is_err());
        assert!(teleport_request(24,&pvars[..0x20],&[Shape::default()]).is_err());
        let r=teleport_request(24,&pvars,&[Shape::default()]).unwrap().unwrap();
        assert_eq!((r.owner,r.cuboid,r.state,r.reset_camera),(24,0,0,true));
    }
}
