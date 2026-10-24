//! Aranos class 0x132d: only the initialization branch of 0x383eb8 and
//! 0x3840b8. Query 0x383fc0 and queued draw 0x3842f8 remain unsupported.
//! Field names describe storage, not an inferred gameplay purpose.
use rc_formats::{buf::{invalid,Result},font::{parse_overlay_sections,read_overlay}};
use rc_game::{hero::physics::fast_cos,moby_runtime::MobyId,ps2v::Pf,rng::Rng};

#[derive(Clone,Debug)]
pub struct AranosControllerData { descriptor:[u8;16] }
impl AranosControllerData {
    pub fn parse(level:u32,overlay:&[u8])->Result<Self> {
        if level!=0 {return invalid("class132d descriptor contract is Aranos-only");}
        let sections=parse_overlay_sections(overlay)?;
        // GP=0x1aeff0 is the matching boot ELF's .reginfo value. Validate the
        // actual selector, including GP-0x380, before accepting its data.
        for (pc,word) in [(0x383f6c,0x2784fc80u32),(0x383f7c,0x0c0e102e),(0x384128,0x0c0c6e68)] {
            if read_overlay(&sections,pc,4)!=Some(word.to_le_bytes().as_slice()) {
                return invalid("class132d source signature mismatch");
            }
        }
        let Some(s)=sections.iter().find(|s|s.kind==1 && 0x1aec70u32.checked_sub(s.dest)
            .is_some_and(|i|(i as usize)+16<=s.data.len())) else {return invalid("class132d initialized descriptor absent");};
        let off=(0x1aec70-s.dest) as usize;
        Ok(Self {descriptor:s.data[off..off+16].try_into().unwrap()})
    }
    pub fn descriptor(&self)->&[u8;16] {&self.descriptor}
}

/// Native ownership for original global block 0x1b9040. No guest pointers or
/// GPU query results are fabricated. Each state0 callback resets/regenerates it.
#[derive(Clone,Debug)]
pub struct Controller132d {
    pub owner:MobyId,
    /// Source global 0x1b26bc receives 0x49 on Aranos; purpose unverified.
    pub source_value:u32,
    pub values_10:[i16;14],
    pub values_30:[i16;14],
    pub colours_50:[u32;14],
    pub phases_90:[Pf;14],
    pub weights_d0:[Pf;14],
}

impl Controller132d {
    pub fn initialize(owner:MobyId,rng:&mut Rng)->Self {
        let mut s=Self {owner,source_value:0x49,values_10:[0;14],values_30:[0;14],
            colours_50:[0;14],phases_90:[Pf::ZERO;14],weights_d0:[Pf::ZERO;14]};
        // Source 0x384128..0x38428c, i=1..13 (not 0..13).
        for i in 1..14 {
            let n=(rng.randi(63)+1) as u32;
            s.colours_50[i]=match rng.randi(6) {
                0=>n.wrapping_add(0x202080).wrapping_sub(n<<16),
                1=> (n<<8).wrapping_add(n.wrapping_add(0x202080)),
                2=> {let v=rng.randi(n as i32) as u32;(v<<8).wrapping_add(n.wrapping_add(0x202080))},
                3=> {let a=rng.randi(n as i32) as u32;let b=rng.randi(n as i32) as u32;
                    (a<<16).wrapping_add((b<<8).wrapping_add(0x202080)).wrapping_add(n)},
                _=>0x202080,
            };
            s.values_30[i]=(rng.randi(30)/10+1) as i16;
            let angle=Pf(rng.randf_bits(0xbf733333,0x4019999a));
            s.phases_90[i]=angle;
            // GC FastModRot 0x2efef0 then VU0 cosine 0x2ef448 and abs.
            let x=(angle+Pf::ONE)*Pf::b(0x3e22f983)+Pf::b(0x3f000000);
            let whole=x.to_i32().wrapping_add((x.0 as i32)>>31);
            let wrapped=((x-Pf::from_i32(whole))-Pf::b(0x3f000000))*Pf::b(0x40c90fdb);
            let c=fast_cos(wrapped).abs();
            s.weights_d0[i]=((c*c)*c)*Pf::b(0x3fb33333);
            s.values_10[i]=32;
        }
        s.values_10[0]=127;s.values_30[0]=0;s.colours_50[0]=0x808080;
        s.phases_90[0]=-Pf::ONE;s.weights_d0[0]=Pf::f(2.);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_init_preserves_rng_branch_order_and_existing_ps2_math_model() {
        // Independent scalar transcription of source 0x384128..0x38428c:
        // seed1234 takes62 draws, including conditional colour draws.
        // Phase vectors include the existing native adder guard-bit rule;
        // they are model checks, not captured PS2 hardware float outputs.
        let mut rng=Rng {state:1234};let c=Controller132d::initialize(23,&mut rng);
        assert_eq!(rng.state,0x1377e9f4);
        assert_eq!(c.colours_50,[0x808080,0x202080,0x292593,0x3137a3,0xffee20b2,0x202080,
            0x202083,0x202b9b,0x2e3aaf,0x203d9d,0x202080,0x202080,0x26258f,0x202080]);
        assert_eq!(c.values_30,[0,2,3,1,2,2,1,3,1,3,2,2,3,1]);
        assert_eq!(c.phases_90.map(|v|v.0),[0xbf800000,0x3eb3ac96,0x3f015499,0xbf53d1e6,
            0xbe42079c,0xbef06fcd,0x3f0e7eb1,0x3fbcb84b,0x401190b2,0xbf4d5b00,
            0x3ec9d0ca,0x3e0d6394,0x400d74cc,0xbee19234]);
        assert_eq!(c.values_10,[127,32,32,32,32,32,32,32,32,32,32,32,32,32]);
        assert_eq!(c.weights_d0[0],Pf::f(2.));
        assert!(c.weights_d0[1..].iter().all(|v|Pf::ZERO<=*v && *v<=Pf::b(0x3fb33333)));
        assert_eq!(c.owner,23);
    }
}
