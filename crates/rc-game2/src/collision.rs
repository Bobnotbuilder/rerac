//! GC placement collision through the existing native collision kernels.
//!
//! The initial RC2 port supplied only the baked world mesh to hero/camera
//! queries. Separate original class collision meshes therefore never took part.
//! This adapter registers unconditional, unrooted rigid triangle placements.
//! Spawn scripts, posed primitives, class initialization and moving-platform
//! carry remain separate missing capabilities; no render mesh becomes a collider.
use rc_formats::{buf::{invalid, Buf, Result}, gameplay::MobyInstance, level::LevelCore,
    moby::MobyClassHeader, moby_collision::{self, MobyCollision}};
use rc_game::{collision_query::{CollMoby, MobyGrid, MobyScene, MobySource, PoseCache},
    moby_runtime::Moby, moby_update::scheduler::rebuild_matrix, ps2v::Pf};
use rc_game::collision_query::mobys::GRID_NONE;
use std::{collections::HashMap, sync::Mutex};

pub struct PlacementCollision {
    records: Vec<Option<CollMoby>>,
    classes: HashMap<i16, MobyCollision>,
    grid: MobyGrid,
    cache: Mutex<PoseCache>,
    pub registered: usize,
    pub unsupported: usize,
}

impl PlacementCollision {
    pub fn load(core: &LevelCore, data: &[u8], instances: &[MobyInstance]) -> Result<Self> {
        if core.game != rc_formats::Game::Rac2 { return invalid("GC placement collision requires an explicitly tagged GC core"); }
        let mut classes = HashMap::new();
        let mut headers = HashMap::new();
        for (id, blob) in moby_collision::parse_level(core, data)? {
            // Joint poses and primitive initialization must come from native GC
            // class behavior, never zeros or RAC1 class handlers.
            if blob.faces.is_empty() || !blob.prims.is_empty() || blob.joint_counts != [0, 0] { continue; }
            let raw = core.block(data, &format!("moby_class/{id:04}")).expect("parsed class block");
            let header: MobyClassHeader = Buf(raw).pod(0, "GC collision class header")?;
            headers.insert(id, header);
            classes.insert(id as i16, blob);
        }
        Ok(Self::build(instances, headers, classes))
    }

    fn build(instances: &[MobyInstance], headers: HashMap<i32, MobyClassHeader>, classes: HashMap<i16, MobyCollision>) -> Self {
        let mut out = Self { records: vec![None; instances.len()], classes, grid: MobyGrid::new(),
            cache: Mutex::new(PoseCache::default()), registered: 0, unsupported: 0 };
        for (id, i) in instances.iter().enumerate() {
            let Some(h) = headers.get(&i.o_class) else { out.unsupported += 1; continue; };
            let mode = i.mode() | h.mode_bits as u16;
            if id > u16::MAX as usize || i.spawn_flags != 0 || i.is_rooted != 0 || mode & 0x100 != 0
                || !h.scale.is_finite() || h.scale <= 0.0 || !i.scale.is_finite() || i.scale <= 0.0
                || !i.position.iter().chain(&i.rotation).chain(&h.bsphere).all(|v| v.is_finite()) {
                out.unsupported += 1; continue;
            }
            // Only the shared transform/bounds builder is reused. No RAC1
            // loader, spawn tests, class initialization or scheduler runs.
            let mut m = Moby::zeroed();
            m.index = id as u32; m.o_class = i.o_class as i16;
            m.position[..3].copy_from_slice(&i.position);
            m.rotation[..3].copy_from_slice(&i.rotation);
            m.scale = (Pf::f(h.scale) * Pf::f(i.scale)).to_f32();
            m.mode = mode; m.rows[3] = h.bsphere;
            m.has_collision = true; m.ba0 = GRID_NONE.to_le_bytes();
            m.uid_hi = (id as u32) << 16;
            rebuild_matrix(&mut m, None);
            out.grid.register(&mut m);
            out.records[id] = Some(CollMoby::of(&m));
            out.registered += 1;
        }
        out
    }

    pub fn scene(&self) -> MobyScene<'_> {
        MobyScene { mobys: self, grid: &self.grid, classes: &self.classes, cache: &self.cache }
    }

    /// Original placement indices tested by the spatial grid at a point.
    pub fn candidates(&self, p: [f32; 3]) -> &[u16] {
        self.grid.cell((p[0] / 16.0).floor() as u32, (p[1] / 16.0).floor() as u32)
    }
}

impl MobySource for PlacementCollision {
    fn moby(&self, id: usize) -> Option<CollMoby> { self.records.get(id).copied().flatten() }
    fn joints(&self, _: usize, count: usize) -> Vec<[u32; 4]> {
        assert_eq!(count, 0, "GC placement collision must not request fabricated joint poses");
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_game::collision_query::{coll_line_m, QueryFlags};

    fn parts() -> (HashMap<i32, MobyClassHeader>, HashMap<i16, MobyCollision>) {
        let header=MobyClassHeader {scale:0.5,bsphere:[0.,0.,0.,4096.],..Default::default()};
        let blob=MobyCollision {joint_counts:[0,0],prims:vec![],vertices:vec![[-1024,-1024,0,0],[1024,1024,0,0],[1024,-1024,0,0],[-1024,1024,0,0]],faces:vec![[0,1,2,0x69],[0,3,1,0x69]]};
        ([(7,header)].into_iter().collect(),[(7,blob)].into_iter().collect())
    }

    #[test]
    fn rigid_placements_use_original_transform_scale_grid_and_ignore() {
        let (h,c)=parts();
        let i=MobyInstance {o_class:7,scale:2.,position:[32.,48.,5.],rotation:[0.,0.,1.5707964],..Default::default()};
        let rigid=PlacementCollision::build(&[i],h,c);let scene=rigid.scene();
        assert_eq!(rigid.registered,1);assert!(rigid.candidates(i.position).contains(&0));
        let empty=rc_formats::collision::Collision::default();
        let (a,b)=([32.25,48.,7.],[32.25,48.,3.]);
        let hit=coll_line_m(&empty,Some(&scene),a,b,QueryFlags(2),None).unwrap();
        assert_eq!(hit.moby,Some(0));assert!((hit.point[2]-5.).abs()<0.001);
        assert!(coll_line_m(&empty,Some(&scene),a,b,QueryFlags(2),Some(0)).is_none());
    }

    #[test]
    fn conditional_rooted_kept_matrix_or_invalid_placements_are_not_fabricated() {
        let (h,c)=parts();let i=MobyInstance{o_class:7,scale:1.,position:[32.,48.,5.],..Default::default()};
        let inputs=[MobyInstance{spawn_flags:1,..i},MobyInstance{is_rooted:1,..i},MobyInstance{mode_bits:0x100,..i},MobyInstance{scale:f32::NAN,..i}];
        let rigid=PlacementCollision::build(&inputs,h,c);
        assert_eq!(rigid.registered,0);assert_eq!(rigid.unsupported,4);
        assert!(rigid.candidates(i.position).is_empty());
    }
}
