//! Pure host/display policy; no game state, UIKit, Bevy, or asset dependencies.
//! Also compiled as a standalone Rust test executable by iPhone CI.

pub fn horizontal_scale(width: u32, height: u32) -> f32 {
    if width == 0 || height == 0 {
        return 1.0;
    }
    (width as f32 / height as f32) / (4.0 / 3.0)
}

pub fn pixels(width: u32, height: u32, render_height: u32, limit: u32) -> [u32; 2] {
    let a = width.max(1) as f64 / height.max(1) as f64;
    let h = (render_height.max(1) as f64)
        .min(limit.max(1) as f64)
        .min(limit.max(1) as f64 / a);
    [((h * a).round() as u32).max(1), (h.round() as u32).max(1)]
}

/// Uniform, centre-preserving UI inset. Left/right and top/bottom use the larger
/// opposing inset, so centre aiming does not drift with an asymmetric notch.
/// Inputs are fractions of drawable bounds, in left/top/right/bottom order.
pub fn safe_scale(insets: [f32; 4]) -> f32 {
    let f = |v: f32| {
        if v.is_finite() {
            v.clamp(0.0, 0.45)
        } else {
            0.0
        }
    };
    let x = f(insets[0]).max(f(insets[2]));
    let y = f(insets[1]).max(f(insets[3]));
    (1.0 - 2.0 * x).min(1.0 - 2.0 * y)
}

pub fn ui_point(x: f32, y: f32, scale: f32) -> [f32; 2] {
    [256.0 + (x - 256.0) * scale, 208.0 + (y - 208.0) * scale]
}

/// Nominal 720p is a conservative iOS default, not a measured optimum. Thermal
/// pressure changes only pixel workload, never the 60 Hz simulation clock.
pub fn thermal_height(nominal: u32, thermal: i32) -> u32 {
    match thermal {
        3 => nominal.min(480),
        2 => nominal.min(576),
        _ => nominal,
    }
}

pub struct ResolutionGovernor {
    pub height: u32,
    cool_since_ms: Option<u64>,
}
impl Default for ResolutionGovernor {
    fn default() -> Self {
        Self {
            height: 720,
            cool_since_ms: None,
        }
    }
}
impl ResolutionGovernor {
    pub fn update(&mut self, elapsed_ms: u64, thermal: i32) -> u32 {
        let target = thermal_height(720, thermal);
        if target < self.height {
            self.height = target;
            self.cool_since_ms = None;
        } else if target > self.height {
            let since = *self.cool_since_ms.get_or_insert(elapsed_ms);
            if elapsed_ms.saturating_sub(since) >= 30_000 {
                self.height = target;
                self.cool_since_ms = None;
            }
        } else {
            self.cool_since_ms = None;
        }
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn device_ratios_and_resizing() {
        for [w, h] in [
            [2556, 1179],
            [2796, 1290],
            [2436, 1125],
            [1920, 1080],
            [1024, 768],
        ] {
            let [rw, rh] = pixels(w, h, 720, 8192);
            assert_eq!(rh, 720);
            assert!((rw as f32 / rh as f32 - w as f32 / h as f32).abs() <= 0.5 / 720.0);
            let native = horizontal_scale(w, h);
            // The projected horizontal size * display aspect is invariant (Hor+),
            // vertical FoV/near/far do not change. Not a stretched 4:3 picture.
            assert!(((w as f32 / h as f32) / native - 4.0 / 3.0).abs() < 1e-5);
        }
        assert_ne!(pixels(2556, 1179, 720, 8192), pixels(1179, 2556, 720, 8192));
        assert_eq!(horizontal_scale(1024, 768), 1.0);
        assert!((horizontal_scale(1920, 1080) - 4.0 / 3.0).abs() < 1e-6);
    }
    #[test]
    fn bounded_texture_dimensions() {
        for dims in [[100000, 100], [100, 100000], [0, 0], [2556, 1179]] {
            let size = pixels(dims[0], dims[1], 2160, 2048);
            assert!(size.into_iter().all(|v| (1..=2048).contains(&v)));
        }
    }
    #[test]
    fn safe_controls_never_shift_aim_centre() {
        let insets = [0.07, 0.01, 0.02, 0.06];
        let scale = safe_scale(insets);
        assert_eq!(ui_point(256.0, 208.0, scale), [256.0, 208.0]);
        // Uniform reduction, not x/y squeezing; the expanded HUD rectangle fits
        // both safe edges and is centred after either landscape orientation.
        assert_eq!(safe_scale([0.02, 0.01, 0.07, 0.06]), scale);
        for [x, y] in [[0.0, 0.0], [1.0, 1.0]] {
            let px = 0.5 + (x - 0.5) * scale;
            let py = 0.5 + (y - 0.5) * scale;
            // f32 pixel fractions differ by a few ulps at the exact safe edge.
            assert!(px + 1e-6 >= insets[0] && px <= 1.0 - insets[2] + 1e-6);
            assert!(py + 1e-6 >= insets[1] && py <= 1.0 - insets[3] + 1e-6);
        }
        assert_eq!(safe_scale([0.0; 4]), 1.0);
        assert!(safe_scale([f32::NAN, 1.0, -1.0, f32::INFINITY]) > 0.0);
    }
    #[test]
    fn resolution_changes_do_not_modify_logic_rate() {
        assert_eq!(thermal_height(720, 0), 720);
        assert_eq!(thermal_height(720, 1), 720);
        assert_eq!(thermal_height(720, 2), 576);
        assert_eq!(thermal_height(720, 3), 480);
        assert_eq!(thermal_height(416, 3), 416);
    }

    #[test]
    fn recovery_requires_continuous_cool_period() {
        let mut g = ResolutionGovernor::default();
        assert_eq!(g.update(0, 2), 576);
        assert_eq!(g.update(1000, 0), 576);
        assert_eq!(g.update(29_000, 0), 576);
        assert_eq!(g.update(30_000, 2), 576); // pressure resets recovery clock
        assert_eq!(g.update(31_000, 0), 576);
        assert_eq!(g.update(61_000, 0), 720);
        assert_eq!(g.update(62_000, 3), 480);
        assert_eq!(g.update(63_000, 2), 480);
        assert_eq!(g.update(93_000, 2), 576);
    }
}
