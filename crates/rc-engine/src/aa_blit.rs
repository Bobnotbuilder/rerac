//! **Anti-aliasing Original** (crate::graphics): the game's own softening of the picture, two full-screen bilinear
//! copies, drawn at the frame's resolution in game-pixel units (`assets/shaders/aa_blit.wgsl`).
//!
//! The game (level01 addresses, level07's in brackets; `SetupFS_AA_buffer__Fiiiiii` 0x2224f8 [0x228450] builds both
//! packets, NTSC: draw 512×416, display 512×448):
//! * **A, the AA blit** (`DrawWorld` 0x21a1b8 [0x2203a0] → `PutAABlitPacket_A` 0x223200 [0x229158], packet 0x151900,
//!   38 qwords; its toggle 0x16a0b8 is always 1: [0x21b420] sets every render toggle): after the particles, before the HUD, the draw buffer is copied onto itself as 16
//!   sprites 32 pixels wide (PRIM 0x116: sprite, textured, UV; no blending; TEST_1 0x30000; CLAMP_1 5 = clamp;
//!   TEX1_1 0x100000261 = bilinear, level 0; TEX0 = the draw buffer, DECAL). X runs 0..512 with U = X (texel units):
//!   pixel x reads texels x − 1 and x half each. Y runs 0..415 with V 0..416: row y reads at V = y·416/415, and row 415
//!   is not drawn.
//! * **B, the display copy** (the frame loop [0x26d280] → `PutAABlitPacket_B` 0x223260 [0x2291b8] between `PutDrawBufferSmall` and `PutDrawBufferLarge`,
//!   packet 0x151b60, 41 qwords): the finished draw buffer, HUD included, copied to the 512×448 display buffer the same
//!   way: U = X again, and Y −0.5..447.5 with V 0..416 (the 416 lines stretched to 448, centred).
//!
//! On a TV both soften the picture: about half a pixel across, twice, and the vertical resampling. Here each copy is
//! drawn at the frame's resolution: every frame pixel reads the box of one game pixel a GS bilinear tap is, at the place
//! the copy maps it to (`assets/shaders/aa_blit.wgsl`), so at 416 lines the vertical is the GS's result exactly. B's
//! vertical stretch is the frame's own scale (416 game lines → the frame's height): its box is centred. Each copy is
//! separable: one draw across, one down, A's then B's ([`BlitAcross`], [`BlitDown`], [`CopyAcross`], [`CopyDown`]), after the 3D passes and the screen tints and before
//! the UI pass. The HUD, drawn between the two on the PS2, gets B only (crate::hud_render, the HUD option).
//!
//! [L] The blit reads the buffer it writes, a strip at a time; whether a strip's left texel comes from the GS texture
//! cache before or after the strip to its left was written is not modelled (the unfiltered frame is read throughout).
//! The interlace offsets (XYOFFSET) are not part of it: on a progressive screen they would only show as jitter.

use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

use crate::fly_cam::FlyCam;
use crate::graphics::{AntiAliasing, GraphicsSettings};

macro_rules! aa_copy {
    ($(#[$doc:meta])* $name:ident { k: $k:expr, a: $a:expr, last: $last:expr, down: $down:expr }, $order:path) => {
        $(#[$doc])*
        #[derive(Component, ExtractComponent, Clone, Copy, PartialEq, ShaderType)]
        pub struct $name {
            /// x = k, y = a, z = last (the shader's names).
            pub params: Vec4,
            /// x = 0 across, 1 down.
            pub axis: UVec4,
        }

        impl Default for $name {
            fn default() -> Self { $name { params: Vec4::new($k, $a, $last, 0.0), axis: UVec4::new($down as u32, 0, 0, 0) } }
        }

        impl FullscreenMaterial for $name {
            fn fragment_shader() -> ShaderRef { "shaders/aa_blit.wgsl".into() }

            fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> { $order(system) }
        }
    };
}

// Each draw: frame pixel centre t (game units) reads the box of one game pixel at s = (t − a)·k; pixels at t ≥ last
// are not drawn.
aa_copy!(
    /// A across: U = X, pixel x reads at x (its left edge).
    BlitAcross { k: 1.0, a: 0.5, last: f32::MAX, down: false },
    after_world
);
aa_copy!(
    /// A down: V 0..416 over rows 0..415, row y reads at y·416/415; row 415 is not drawn.
    BlitDown { k: 416.0 / 415.0, a: 0.5, last: 415.0, down: true },
    after_blit_across
);
aa_copy!(
    /// B across: U = X again.
    CopyAcross { k: 1.0, a: 0.5, last: f32::MAX, down: false },
    after_blit_down
);
aa_copy!(
    /// B down: the 416 lines over the frame's height, centred.
    CopyDown { k: 1.0, a: 0.0, last: f32::MAX, down: true },
    after_copy_across
);

/// After the 3D passes and the full-screen world passes (the game draws its tints and its mirror image before the
/// blit), before the UI pass (the HUD).
fn after_world(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    system
        .after(bevy::core_pipeline::Core3dSystems::PostProcess)
        .after(crate::gs_post::pass::<crate::mirror_render::MirrorFlip>)
        .after(crate::gs_post::pass::<crate::screen_tint::ScreenTint>)
        .before(bevy::ui_render::ui_pass)
}

fn after_blit_across(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    system.after(crate::gs_post::pass::<BlitAcross>).before(bevy::ui_render::ui_pass)
}

fn after_blit_down(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    system.after(crate::gs_post::pass::<BlitDown>).before(bevy::ui_render::ui_pass)
}

fn after_copy_across(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
    system.after(crate::gs_post::pass::<CopyAcross>).before(bevy::ui_render::ui_pass)
}

pub struct AaBlitPlugin;

impl Plugin for AaBlitPlugin {
    fn build(&self, app: &mut App) {
        use crate::gs_post::GsPostPlugin;
        #[cfg(target_os="ios")]
        if batching_enabled() {
            app.add_plugins((
                GsPostPlugin::<BlitAcross>::prepare_only(),
                GsPostPlugin::<BlitDown>::prepare_only(),
                GsPostPlugin::<CopyAcross>::prepare_only(),
                GsPostPlugin::<CopyDown>::prepare_only(),
            )).add_systems(PostUpdate,follow_option);
            if let Some(r)=app.get_sub_app_mut(bevy::render::RenderApp) {
                r.add_systems(bevy::core_pipeline::Core3d, after_world(batched.into_configs()));
            }
            return;
        }
        app.add_plugins((
            GsPostPlugin::<BlitAcross>::default(),
            GsPostPlugin::<BlitDown>::default(),
            GsPostPlugin::<CopyAcross>::default(),
            GsPostPlugin::<CopyDown>::default(),
        ))
        .add_systems(PostUpdate, follow_option);
    }
}

#[cfg(target_os="ios")]
fn batching_enabled()->bool {
    #[cfg(target_abi="sim")]
    if std::env::var("RERAC_SIM_AA_MODE").as_deref()==Ok("unbatched") {return false;}
    crate::render_settings::load_key("ios_batch_aa").as_deref()!=Some("off")
}

/// Same four shaders, parameters, ping-pong writes and render passes. Bevy 0.19
/// flushes each RenderContext at system end; one system avoids three command
/// buffers without fusing filters or changing their colour/edge behaviour.
#[cfg(target_os="ios")]
#[allow(clippy::type_complexity)]
fn batched(
    view: bevy::render::renderer::ViewQuery<(
        &bevy::render::view::ViewTarget,
        &bevy::render::extract_component::DynamicUniformIndex<BlitAcross>,
        &crate::gs_post::PassBindGroups<BlitAcross>, &crate::gs_post::PassPipeline<BlitAcross>,
        &bevy::render::extract_component::DynamicUniformIndex<BlitDown>,
        &crate::gs_post::PassBindGroups<BlitDown>, &crate::gs_post::PassPipeline<BlitDown>,
        &bevy::render::extract_component::DynamicUniformIndex<CopyAcross>,
        &crate::gs_post::PassBindGroups<CopyAcross>, &crate::gs_post::PassPipeline<CopyAcross>,
        &bevy::render::extract_component::DynamicUniformIndex<CopyDown>,
        &crate::gs_post::PassBindGroups<CopyDown>, &crate::gs_post::PassPipeline<CopyDown>,
    )>,
    cache: Res<bevy::render::render_resource::PipelineCache>,
    mut ctx: bevy::render::renderer::RenderContext,
) {
    let (t,ai,ag,ap,bi,bg,bp,ci,cg,cp,di,dg,dp)=view.into_inner();
    // Never write a partial filter chain while pipelines warm up.
    if !crate::gs_post::ready(ap,&cache)||!crate::gs_post::ready(bp,&cache)
        ||!crate::gs_post::ready(cp,&cache)||!crate::gs_post::ready(dp,&cache) {return;}
    crate::render_work::count(crate::render_work::AA_BATCH,1);
    #[cfg(target_abi="sim")]
    {
        use std::sync::atomic::{AtomicU64,Ordering};
        static BATCHES:AtomicU64=AtomicU64::new(0);
        if BATCHES.fetch_add(1,Ordering::Relaxed)==29 {
            crate::ios::diagnostic("IOS_AA_BATCH_PROBE_PASS","four_original_shader_passes one_render_context physical_gameplay=NOT_TESTED");
        }
    }
    crate::gs_post::encode(t,ai,ag,ap,&cache,&mut ctx);
    crate::gs_post::encode(t,bi,bg,bp,&cache,&mut ctx);
    crate::gs_post::encode(t,ci,cg,cp,&cache,&mut ctx);
    crate::gs_post::encode(t,di,dg,dp,&cache,&mut ctx);
}

/// The four draws on the main camera while the option is Original.
fn follow_option(mut commands: Commands, g: Res<GraphicsSettings>, cams: Query<(Entity, Has<BlitAcross>), With<FlyCam>>) {
    let on = g.aa == AntiAliasing::Original;
    for (e, have) in &cams {
        match (on, have) {
            (true, false) => { commands.entity(e).insert((BlitAcross::default(), BlitDown::default(), CopyAcross::default(), CopyDown::default())); }
            (false, true) => { commands.entity(e).remove::<(BlitAcross, BlitDown, CopyAcross, CopyDown)>(); }
            _ => {}
        }
    }
}
