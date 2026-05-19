//! Demo 2 — Elm counter (M2 textbook).
//!
//! Three buttons: `+`, `−`, `reset`. Click any button → emits Msg →
//! `update(state, msg) -> State` runs in Rust → repaint via `view(state)`.
//! Demonstrates the full Elm shape (init/update/view) end-to-end in the
//! browser. Zero hand-written JS.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use m2_elm_wasm::{init, update, view, Msg, State};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Button hit-zones (x, y, w, h, label)
const BUTTONS: &[(f64, f64, f64, f64, &str)] = &[
    (80.0, 320.0, 120.0, 80.0, "−"),
    (340.0, 320.0, 120.0, 80.0, "+"),
    (600.0, 320.0, 200.0, 80.0, "reset"),
];

fn paint(ctx: &web_sys::CanvasRenderingContext2d, w: f64, h: f64, state: State) {
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Header
    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("20px monospace");
    let _ = ctx.fill_text(
        "Elm counter — init / update / view, painted via Rust → WASM",
        20.0,
        30.0,
    );

    // Render via the same view() the contract proves pure
    let nodes = view(state);
    for n in &nodes {
        if n.tag == "count" {
            ctx.set_fill_style_str(&rgb(n.fg.r as f64, n.fg.g as f64, n.fg.b as f64));
            ctx.set_font("96px monospace");
            let _ = ctx.fill_text(&n.text, 380.0, 220.0);
            ctx.set_fill_style_str("#565f89");
            ctx.set_font("16px monospace");
            let _ = ctx.fill_text("count", 380.0, 250.0);
        }
    }

    // Buttons
    for (bx, by, bw, bh, label) in BUTTONS {
        ctx.set_fill_style_str("#161b22");
        ctx.fill_rect(*bx, *by, *bw, *bh);
        ctx.set_stroke_style_str("#7c3aed");
        ctx.set_line_width(3.0);
        ctx.stroke_rect(*bx, *by, *bw, *bh);
        ctx.set_fill_style_str("#c9d1d9");
        ctx.set_font("48px monospace");
        // Center label
        let m = ctx.measure_text(label).ok();
        let tx = bx + (bw - m.as_ref().map(|m| m.width()).unwrap_or(20.0)) / 2.0;
        let _ = ctx.fill_text(label, tx, by + bh * 0.65);
    }

    // Footer
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("16px monospace");
    let _ = ctx.fill_text(
        "contract: wasm-lifecycle-v1 holds — View_Pure, EventReplay_Deterministic",
        20.0,
        h - 20.0,
    );
}

fn hit_test(x: f64, y: f64) -> Option<Msg> {
    for (bx, by, bw, bh, label) in BUTTONS {
        if x >= *bx && x <= bx + bw && y >= *by && y <= by + bh {
            return Some(match *label {
                "+" => Msg::Increment,
                "−" => Msg::Decrement,
                "reset" => Msg::Reset,
                _ => return None,
            });
        }
    }
    None
}

#[wasm_bindgen]
pub fn mount_counter(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let state = Rc::new(RefCell::new(init()));
    paint(&ctx, w, h, *state.borrow());

    // Click handler
    let state_click = state.clone();
    let canvas_for_hit = canvas.clone();
    let ctx_click = ctx.clone();
    let on_click = Closure::<dyn FnMut(_)>::new(move |evt: web_sys::MouseEvent| {
        let rect = canvas_for_hit.get_bounding_client_rect();
        let scale_x = canvas_for_hit.width() as f64 / rect.width();
        let scale_y = canvas_for_hit.height() as f64 / rect.height();
        let cx = (evt.client_x() as f64 - rect.left()) * scale_x;
        let cy = (evt.client_y() as f64 - rect.top()) * scale_y;
        if let Some(msg) = hit_test(cx, cy) {
            let new_state = update(*state_click.borrow(), msg);
            *state_click.borrow_mut() = new_state;
            paint(&ctx_click, w, h, new_state);
        }
    });
    canvas
        .add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())
        .map_err(|e| e)?;
    on_click.forget(); // Leak the closure for the lifetime of the page

    Ok(())
}
