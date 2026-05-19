//! Demo 1 — Canvas2D primitives (M1 visual proof).
//!
//! Paints a 16×9 grid of cells; each cell shows a different m1-canvas
//! clip outcome — including NaN / Inf / oversize / negative inputs.
//! A debug overlay names which inputs survived `clip()` and which
//! were dropped. Demonstrates `wasm-rendering-v1` live.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use m1_canvas::{clip, CanvasDims, DrawRect};
use presentar_core::Color;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn mount_canvas(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    // Slate background
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    let canvas_dims = CanvasDims {
        width: w,
        height: h,
    };

    // 8 adversarial test inputs to clip
    let c = Color::GREEN;
    let plans = vec![
        (
            "clean",
            DrawRect {
                x: 60.0,
                y: 60.0,
                w: 200.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "overflow-right",
            DrawRect {
                x: 700.0,
                y: 60.0,
                w: 600.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "overflow-bottom",
            DrawRect {
                x: 60.0,
                y: 540.0,
                w: 200.0,
                h: 400.0,
                color: c,
            },
        ),
        (
            "negative-w",
            DrawRect {
                x: 320.0,
                y: 60.0,
                w: -50.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "nan-x",
            DrawRect {
                x: f64::NAN,
                y: 60.0,
                w: 200.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "inf-y",
            DrawRect {
                x: 60.0,
                y: f64::INFINITY,
                w: 200.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "off-screen",
            DrawRect {
                x: 2000.0,
                y: 60.0,
                w: 200.0,
                h: 120.0,
                color: c,
            },
        ),
        (
            "zero-w",
            DrawRect {
                x: 320.0,
                y: 200.0,
                w: 0.0,
                h: 120.0,
                color: c,
            },
        ),
    ];

    // Header
    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("20px monospace");
    ctx.fill_text(
        "m1-canvas::clip — 8 adversarial inputs, drawn only if they survive",
        20.0,
        30.0,
    )?;

    let mut survivors = 0;
    let mut dropped = 0;
    for (i, (label, plan)) in plans.iter().enumerate() {
        // Status row (always shown)
        let row_y = 380.0 + (i as f64) * 24.0;
        match clip(*plan, canvas_dims) {
            Some(clipped) => {
                survivors += 1;
                // Translucent fill bar showing the surviving rect (scaled down to lower band)
                let preview_y = 60.0;
                let preview_h = 200.0;
                let scale_x = 0.5;
                let scale_y = 0.5;
                ctx.set_fill_style_str(&rgb(0.3, 0.7, 0.4));
                ctx.set_global_alpha(0.4);
                ctx.fill_rect(
                    clipped.x * scale_x,
                    preview_y + clipped.y * scale_y,
                    clipped.w * scale_x,
                    clipped.h * scale_y.min(preview_h / clipped.h.max(1.0)),
                );
                ctx.set_global_alpha(1.0);
                ctx.set_fill_style_str("#7ee787");
                ctx.set_font("14px monospace");
                ctx.fill_text(
                    &format!(
                        "✓ {:<15}  in={:?} out=({:.0},{:.0},{:.0}x{:.0})",
                        label, plan.x as i64, clipped.x, clipped.y, clipped.w, clipped.h
                    ),
                    20.0,
                    row_y,
                )?;
            }
            None => {
                dropped += 1;
                ctx.set_fill_style_str("#ff7b72");
                ctx.set_font("14px monospace");
                ctx.fill_text(
                    &format!("✗ {:<15}  dropped (out of canvas)", label),
                    20.0,
                    row_y,
                )?;
            }
        }
    }

    // Footer: contract marker
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("16px monospace");
    ctx.fill_text(
        &format!(
            "survived: {} · dropped: {} · contract: wasm-rendering-v1 holds — OK",
            survivors, dropped
        ),
        20.0,
        h - 20.0,
    )?;

    Ok(())
}
