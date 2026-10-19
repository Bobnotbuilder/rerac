"""Source contracts complement compiled policy and real simulator GPU probes."""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
def source(name): return (ROOT/'crates/rc-engine/src'/name).read_text()

class RenderEfficiency(unittest.TestCase):
    def test_render_startup_invalidates_owned_device_resources(self):
        self.assertIn('remove::<PassBindGroups<T>>()',source('gs_post.rs'))
        shadow=source('shadow_render.rs')
        self.assertIn('commands.insert_resource(ShadowGpu::default())',shadow)
        self.assertIn('remove::<(ShadowBindings,ShadowCount)>()',shadow)

    def test_batch_preserves_order_and_pipeline_readiness(self):
        s=source('aa_blit.rs')
        calls=['encode(t,ai,ag,ap','encode(t,bi,bg,bp','encode(t,ci,cg,cp','encode(t,di,dg,dp']
        self.assertEqual(sorted(s.index(x) for x in calls),[s.index(x) for x in calls])
        self.assertLess(s.index('ready(ap'),s.index(calls[0]))
        self.assertEqual(s.count('::prepare_only()'),4)
        self.assertIn('ios_batch_aa',s)
        self.assertIn('#[cfg(target_os="ios")]',s)
        self.assertNotIn('queue.submit',s)

    def test_aa_shaders_and_filter_parameters_remain_original(self):
        s=source('aa_blit.rs')
        for spec in ('BlitAcross { k: 1.0, a: 0.5','BlitDown { k: 416.0 / 415.0, a: 0.5',
                     'CopyAcross { k: 1.0, a: 0.5','CopyDown { k: 1.0, a: 0.0'):
            self.assertIn(spec,s)
        self.assertIn('shaders/aa_blit.wgsl',s)

    def test_post_cache_uses_unordered_textures_and_uniform_identity(self):
        s=source('gs_post.rs')
        self.assertIn('render_policy::same_pair',s)
        self.assertIn('g.uniform != uniform',s)
        self.assertIn('std::any::type_name::<T>()',s)
        self.assertIn('target.post_process_write()',s)

    def test_shadow_allocation_and_groups_outside_render_pass(self):
        s=source('shadow_render.rs')
        render=s[s.index('fn shadow_pass('):s.index('#[cfg(test)]')]
        self.assertNotIn('create_buffer',render)
        self.assertNotIn('create_bind_group',render)
        self.assertIn('render_policy::capacity',s)
        self.assertIn('b.count_id!=count.default_view.id()||b.samples!=samples',s)
        self.assertIn('b.uniform.get()!=&clip',s)
        self.assertIn('BufferUsages::VERTEX|BufferUsages::COPY_DST',s)
        self.assertIn('gpu.vertices==0',s)

    def test_cpu_wall_and_buffer_count_limits_are_explicit(self):
        s=source('render_work.rs')
        for key in ('pending.len()', 'metal_total_command_buffers=UNMEASURED',
                    'gpu_ms=UNMEASURED','scopes_overlap=YES','prepare_to_submit_wall_ms'):
            self.assertIn(key,s)
        self.assertNotIn('queue.submit',s)
        self.assertNotIn('device.poll',s)
        runtime=source('ios_runtime.rs')
        self.assertIn('IOS_RENDER_WORK',runtime)
        self.assertIn('Duration::from_secs(30)',runtime)

    def test_resolution_rows_match_ios_choices_and_fixed_mode(self):
        s=source('display.rs')
        for token in ('Self::Height(416)','Self::Height(540)','Self::Height(576)','Self::Fixed720',
                      '720-fixed','Resolution::Height(720) => Resolution::Height(device.render_height'):
            self.assertIn(token,s)
        menu=(ROOT/'crates/rc-game/src/menus/pause/port.rs').read_text()
        for label in ('720p auto','720p fixed','540p','576p'):
            self.assertIn(label,menu)

    def test_ci_exercises_batched_aa_shadow_and_all_resizes(self):
        runtime=source('ios_runtime.rs')
        self.assertIn('shadow_render::simulator_install(app)',runtime)
        self.assertIn('Resolution::Fixed720',runtime)
        self.assertIn('Resolution::Height(540)',runtime)
        smoke=(ROOT/'ios/simulator_startup_smoke.py').read_text()
        for marker in ('IOS_AA_BATCH_PROBE_PASS','IOS_SHADOW_REUSE_PROBE_PASS'):
            self.assertIn(marker,smoke)
        workflow=(ROOT/'.github/workflows/build-ios-arm64.yml').read_text()
        self.assertIn('--test crates/rc-engine/src/render_policy.rs',workflow)

if __name__=='__main__': unittest.main()
