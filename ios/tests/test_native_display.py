"""Source/package contracts complement actual compiled Rust maths and iOS probe tests."""
from pathlib import Path
import plistlib
import unittest

ROOT=Path(__file__).resolve().parents[2]

class NativeDisplay(unittest.TestCase):
    def source(self,name):
        return (ROOT/'crates/rc-engine/src'/name).read_text()

    def test_ios_defaults_are_separate_and_non_destructive(self):
        s=self.source('display.rs')
        for token in ('ios_aspect','ios_resolution','resolution: Resolution::Height(720)',
                      'Aspect::Native { width: ws.x, height: ws.y }'):
            self.assertIn(token,s)
        self.assertIn('aspect: Aspect::Original, resolution: Resolution::Window, fullscreen: false',s)
        self.assertNotIn('remove_file',s)

    def test_final_ui_is_independent_of_world_resolution(self):
        for name in ('hud_render.rs','menu_render.rs','movie_render.rs'):
            s=self.source(name)
            self.assertIn('With<crate::display::PresentCamera>',s)
            self.assertIn('crate::display::PresentationUi',s)
        self.assertIn('fn place_presentation_ui',self.source('display.rs'))

    def test_safe_area_does_not_move_world_target_markers(self):
        s=self.source('hud_render.rs')
        self.assertLess(s.index('marker_render::prims(p)'),s.index('let hud_start='))
        self.assertIn('hud_start..hud_end,device.ui_scale',s)
        self.assertIn('hud.contains(&i)||p.boxed',s)
        self.assertIn('!(lo<=0 && hi>=W)',s)
        self.assertIn('assert_eq!(h.prims[0],old[0])',s)
        self.assertIn('assert_eq!(h.prims[2],old[2])',s)

    def test_movie_and_preview_aspects_remain_explicit(self):
        self.assertIn('crate::display::UiBoxed',self.source('movie_render.rs'))
        s=self.source('screen_canvas.rs')
        self.assertIn('s.composite==Composite::OverHud',s)
        self.assertIn('focal:[v.focal[0]*scale,v.focal[1]*scale]',s)
        self.assertIn('crate::display::BoxedView',self.source('menu_render.rs'))

    def test_projection_uses_hor_plus_not_image_stretch(self):
        s=self.source('display.rs')
        self.assertIn('game_tan * hor_scale(aspect)',s)
        self.assertIn('game_tan * crate::game_camera::NTSC_Y_RATIO',s)
        self.assertIn('widened.get_frustum_corners',s)
        self.assertIn('(widened.near,widened.far),(original.near,original.far)',s)

    def test_ios_wait_has_no_spin_and_ticks_remain_sixty(self):
        s=self.source('frame_pace.rs')
        self.assertIn('Duration::from_secs_f64(TICK)',s)
        self.assertIn('if deadline > now { std::thread::sleep(deadline - now); }',s)
        self.assertIn('FixedTimesteps(0)',s)
        self.assertIn('crate::determinism::TICK_HZ',s)

    def test_local_diagnostics_label_measurement_limits(self):
        s=self.source('ios_runtime.rs')
        for token in ('NonSendMut<MainThread>','Duration::from_secs(30)',
                      'gpu_ms=UNMEASURED','presented_fps=UNMEASURED',
                      'footprint_bytes=','battery_fraction=','< 2048'):
            self.assertIn(token,s)
        info=plistlib.loads((ROOT/'ios/Info.plist').read_bytes())
        self.assertNotIn('UIBackgroundModes',info)
        self.assertEqual(info['CFBundleVersion'],'9')

    def test_sandbox_saves_and_input_contract_preserved(self):
        s=self.source('ios.rs')
        self.assertIn('support.join("memcard")',s)
        self.assertIn('documents.join("ReRACData")',s)
        self.assertIn('ios_gamepad::IOSGameControllerPlugin',s)

    def test_ci_compiles_real_math_and_preserves_startup_failure_evidence(self):
        workflow=(ROOT/'.github/workflows/build-ios-arm64.yml').read_text()
        self.assertIn('rustc --edition 2021 --test crates/rc-engine/src/ios_display_math.rs',workflow)
        self.assertIn('cargo xtask test-job engine ui --cargo-test',workflow)
        smoke=(ROOT/'ios/simulator_startup_smoke.py').read_text()
        self.assertIn('for attempt in range(2)',smoke)
        self.assertIn("if 'startup' in locals() and startup.exists():",smoke)

    def test_render_fixture_is_simulator_only(self):
        s=self.source('ios_runtime.rs')
        self.assertIn('#[cfg(target_abi = "sim")]\npub(crate) fn install_simulator_probe',s)
        self.assertIn('IOS_RENDER_RESIZE_PROBE_PASS',s)
        self.assertIn('physical_target_size()',s)
        smoke=(ROOT/'ios/simulator_startup_smoke.py').read_text()
        self.assertIn("'IOS_RENDER_RESIZE_PROBE_PASS' in text",smoke)

    def test_missing_key_window_does_not_gate_app_lifecycle(self):
        native=self.source('ios_device.m')
        rust=self.source('ios_runtime.rs')
        self.assertIn('return layout ? 1 : 2;',native)
        self.assertNotIn('return 0;',native)
        self.assertIn('if status == 1',rust)
        self.assertIn('let active = info.active != 0',rust)

if __name__=='__main__': unittest.main()
