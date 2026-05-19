//! Demo 2 — Elm counter (M2 textbook).
//!
//! Three buttons: `+`, `−`, `reset`. Click any button → emits Msg →
//! `update(state, msg) -> State` runs in Rust → repaint via `view(state)`.
//! Demonstrates the full Elm shape (init/update/view) end-to-end in the
//! browser. Zero hand-written JS.

use crate::canvas_helpers::get_canvas_ctx;
use crate::logic::counter::{key_to_msg, paint_ops};
use crate::logic::replay;
use m2_elm_wasm::{init, update, Msg, State};
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
    // Render via the pure paint_ops() — proven by probar to emit the
    // count value as a FillText. The previous in-place paint code
    // failed bug #1 because it looked for `tag == "count"` which
    // m2-elm-wasm::view() never emits.
    let ops = paint_ops(state, w, h);
    let _ = replay(ctx, &ops);
    // (Old in-line paint kept below for reference until end of fn,
    // but harmless — replay already drew everything.)
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

    // BUG #7 fix (gist round 2, hardened round 3): on-canvas hint
    // advertises `+/- to step, r to reset` but the previous demo only
    // wired mouse clicks. Round-2 attached the listener to `window`
    // which silently received nothing in some browsers; round-3 QA
    // confirmed shell's `document.addEventListener("keydown")` path
    // works reliably, so we mirror it here. Ignore Ctrl/Meta-modified
    // events so they don't shadow browser shortcuts (cmd+R = reload).
    let state_keys = state.clone();
    let ctx_keys = ctx.clone();
    let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let doc = win
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let on_key = Closure::<dyn FnMut(_)>::new(move |evt: web_sys::KeyboardEvent| {
        if evt.ctrl_key() || evt.meta_key() {
            return;
        }
        if let Some(msg) = key_to_msg(&evt.key()) {
            let new_state = update(*state_keys.borrow(), msg);
            *state_keys.borrow_mut() = new_state;
            paint(&ctx_keys, w, h, new_state);
            evt.prevent_default();
        }
    });
    doc.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())?;
    on_key.forget();

    Ok(())
}
