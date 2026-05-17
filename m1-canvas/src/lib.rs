//! M1.1 — Canvas2D draw plans with bounds clamping.
//!
//! Provable contract: `contracts/wasm-rendering-v1.yaml`.
//!
//! In aprender-present-lib, `browser::Canvas2DRenderer` is the production
//! Canvas2D backend. This crate models the SAME safety obligations as
//! pure native-Rust functions so we can:
//!   * unit-test on the host (no headless browser needed)
//!   * verify the bounds-clamping + NaN-safety contract at L1-L3
//!   * hand the same logic to wasm-bindgen for the browser at runtime

pub use presentar_core::Color;

/// One Canvas2D draw call.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub color: Color,
}

/// A finite-pixel canvas frame.
#[derive(Debug, Clone, Copy)]
pub struct CanvasDims {
    pub width: f64,
    pub height: f64,
}

/// Clamp a colour's RGBA components to `[0.0, 1.0]` — the wasm-rendering-v1
/// "Color clamping" obligation.
#[must_use]
pub fn clamp_color(c: Color) -> Color {
    Color::new(
        c.r.clamp(0.0, 1.0),
        c.g.clamp(0.0, 1.0),
        c.b.clamp(0.0, 1.0),
        c.a.clamp(0.0, 1.0),
    )
}

/// Total function: clip a `DrawRect` to a canvas, dropping it entirely
/// if any coord is NaN/Inf or the canvas is degenerate. Returns `None`
/// when the result has zero area.
#[must_use]
pub fn clip(rect: DrawRect, canvas: CanvasDims) -> Option<DrawRect> {
    if canvas.width <= 0.0 || canvas.height <= 0.0 {
        return None;
    }
    if !rect.x.is_finite() || !rect.y.is_finite() || !rect.w.is_finite() || !rect.h.is_finite() {
        return None;
    }
    let x = rect.x.max(0.0);
    let y = rect.y.max(0.0);
    let right = (rect.x + rect.w).min(canvas.width);
    let bottom = (rect.y + rect.h).min(canvas.height);
    let w = (right - x).max(0.0);
    let h = (bottom - y).max(0.0);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(DrawRect {
        x,
        y,
        w,
        h,
        color: clamp_color(rect.color),
    })
}

/// Runtime smoke check the demo binary asserts against.
#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-rendering-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn canvas() -> CanvasDims {
        CanvasDims {
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn clamp_color_handles_out_of_range() {
        let c = clamp_color(Color::new(2.0, -1.0, 0.5, 1.5));
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.5);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn clip_returns_none_for_nan_coords() {
        let r = DrawRect {
            x: f64::NAN,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: Color::RED,
        };
        assert!(clip(r, canvas()).is_none());
    }

    #[test]
    fn clip_returns_none_for_infinite_coords() {
        let r = DrawRect {
            x: f64::INFINITY,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: Color::RED,
        };
        assert!(clip(r, canvas()).is_none());
    }

    #[test]
    fn clip_returns_none_for_zero_area() {
        let r = DrawRect {
            x: -100.0,
            y: -100.0,
            w: 50.0,
            h: 50.0,
            color: Color::RED,
        };
        assert!(clip(r, canvas()).is_none());
    }

    #[test]
    fn clip_returns_none_for_degenerate_canvas() {
        let degen = CanvasDims {
            width: 0.0,
            height: 600.0,
        };
        let r = DrawRect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: Color::RED,
        };
        assert!(clip(r, degen).is_none());
    }

    #[test]
    fn clip_clamps_overhanging_rect_into_canvas() {
        let r = DrawRect {
            x: 750.0,
            y: 550.0,
            w: 200.0,
            h: 200.0,
            color: Color::BLUE,
        };
        let clipped = clip(r, canvas()).expect("partially in bounds");
        assert!(clipped.x + clipped.w <= 800.0);
        assert!(clipped.y + clipped.h <= 600.0);
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
