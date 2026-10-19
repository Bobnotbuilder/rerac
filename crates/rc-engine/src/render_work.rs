//! Bounded counters, not GPU timings or CPU utilisation. No queue waits/readbacks.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

pub const POST_PASS: usize = 0;
pub const POST_GROUP: usize = 1;
pub const SHADOW_ALLOC: usize = 2;
pub const SHADOW_GROUP: usize = 3;
pub const SHADOW_UPLOAD: usize = 4;
pub const SHADOW_PASS: usize = 5;
pub const AA_BATCH: usize = 6;
static COUNTERS: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];
static TIMES: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];
static SAMPLES: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];
static GRAPH_FRAMES: AtomicU64 = AtomicU64::new(0);
static GRAPH_BUFFERS: AtomicU64 = AtomicU64::new(0);
pub fn count(kind: usize, n: u64) {
    if cfg!(target_os = "ios") {
        COUNTERS[kind].fetch_add(n, Relaxed);
    }
}
pub struct Scope(usize, Option<Instant>);
impl Scope {
    pub fn new(kind: usize) -> Self {
        Self(
            kind,
            if cfg!(target_os = "ios") {
                Some(Instant::now())
            } else {
                None
            },
        )
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(start) = self.1 {
            TIMES[self.0].fetch_add(start.elapsed().as_nanos() as u64, Relaxed);
            SAMPLES[self.0].fetch_add(1, Relaxed);
        }
    }
}
pub fn take_report() -> String {
    let c = COUNTERS.each_ref().map(|a| a.swap(0, Relaxed));
    let mean = |i: usize| {
        let n = SAMPLES[i].swap(0, Relaxed);
        let t = TIMES[i].swap(0, Relaxed);
        if n == 0 {
            0.0
        } else {
            t as f64 / n as f64 / 1e6
        }
    };
    let frames = GRAPH_FRAMES.swap(0, Relaxed);
    let buffers = GRAPH_BUFFERS.swap(0, Relaxed);
    format!("graph_frames={frames} graph_pending_buffers={buffers} graph_buffers_per_frame={:.2} post_passes={} post_bind_groups={} shadow_vertex_allocations={} shadow_bind_groups={} shadow_vertex_upload_bytes={} shadow_passes={} aa_batches={} post_encode_wall_ms={:.3} shadow_prepare_wall_ms={:.3} shadow_encode_wall_ms={:.3} prepare_to_submit_wall_ms={:.3} metal_total_command_buffers=UNMEASURED gpu_ms=UNMEASURED scopes_overlap=YES",
        if frames==0 {0.0} else {buffers as f64/frames as f64},c[0],c[1],c[2],c[3],c[4],c[5],c[6],mean(0),mean(1),mean(2),mean(3))
}

#[cfg(target_os = "ios")]
pub struct RenderWorkPlugin;
#[cfg(target_os = "ios")]
impl bevy::prelude::Plugin for RenderWorkPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use bevy::prelude::*;
        use bevy::render::{
            renderer::{PendingCommandBuffers, RenderGraph, RenderGraphSystems},
            Render, RenderApp, RenderSystems,
        };
        if let Some(r) = app.get_sub_app_mut(RenderApp) {
            #[derive(Resource)]
            struct Start(Instant);
            r.insert_resource(Start(Instant::now()));
            r.add_systems(
                Render,
                (|mut start: ResMut<Start>| start.0 = Instant::now())
                    .before(RenderSystems::PrepareAssets),
            );
            r.add_systems(
                RenderGraph,
                (|pending: Res<PendingCommandBuffers>, start: Res<Start>| {
                    GRAPH_FRAMES.fetch_add(1, Relaxed);
                    GRAPH_BUFFERS.fetch_add(pending.len() as u64, Relaxed);
                    TIMES[3].fetch_add(start.0.elapsed().as_nanos() as u64, Relaxed);
                    SAMPLES[3].fetch_add(1, Relaxed);
                })
                .after(RenderGraphSystems::Render)
                .before(RenderGraphSystems::Submit),
            );
        }
    }
}
