//! M3.2 — Gauge + progress bar + line-chart cursor math. The chart-
//! cursor-monotonic + chart-cursor-clamped obligations from
//! wasm-panels-v1 are proved as pure numerics here.

/// Compute the cursor position (in pixel coords) of a value `v` on a
/// linear chart whose data range is `[v_min, v_max]` and pixel range
/// is `[px_start, px_end]`. Total: NaN/Inf inputs return `px_start`
/// (no panic, clamped to leftmost).
#[must_use]
pub fn position(v: f64, v_min: f64, v_max: f64, px_start: f64, px_end: f64) -> f64 {
    if !v.is_finite() || !v_min.is_finite() || !v_max.is_finite() {
        return px_start;
    }
    if v_max <= v_min {
        return px_start;
    }
    let t = ((v - v_min) / (v_max - v_min)).clamp(0.0, 1.0);
    px_start + t * (px_end - px_start)
}

/// Gauge fill fraction in `[0.0, 1.0]`. Out-of-range → clamped.
#[must_use]
pub fn gauge_fraction(value: f64, max: f64) -> f64 {
    if !value.is_finite() || !max.is_finite() || max <= 0.0 {
        return 0.0;
    }
    (value / max).clamp(0.0, 1.0)
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-panels-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn position_monotonic_on_sorted_input() {
        let prev = position(0.0, 0.0, 100.0, 0.0, 800.0);
        let curr = position(50.0, 0.0, 100.0, 0.0, 800.0);
        let next = position(100.0, 0.0, 100.0, 0.0, 800.0);
        assert!(prev <= curr);
        assert!(curr <= next);
    }

    #[test]
    fn position_clamped_to_chart_pixel_bounds() {
        let p = position(1e30, 0.0, 100.0, 0.0, 800.0);
        assert!((0.0..=800.0).contains(&p));
        let n = position(-1e30, 0.0, 100.0, 0.0, 800.0);
        assert!((0.0..=800.0).contains(&n));
    }

    #[test]
    fn position_nan_returns_start() {
        assert_eq!(position(f64::NAN, 0.0, 100.0, 50.0, 800.0), 50.0);
        assert_eq!(position(f64::INFINITY, 0.0, 100.0, 50.0, 800.0), 50.0);
    }

    #[test]
    fn position_degenerate_range_returns_start() {
        assert_eq!(position(5.0, 10.0, 10.0, 0.0, 800.0), 0.0);
        assert_eq!(position(5.0, 100.0, 0.0, 0.0, 800.0), 0.0);
    }

    #[test]
    fn gauge_clamps_above_and_below() {
        assert_eq!(gauge_fraction(50.0, 100.0), 0.5);
        assert_eq!(gauge_fraction(150.0, 100.0), 1.0);
        assert_eq!(gauge_fraction(-50.0, 100.0), 0.0);
        assert_eq!(gauge_fraction(f64::NAN, 100.0), 0.0);
        assert_eq!(gauge_fraction(50.0, 0.0), 0.0);
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
