//! Demo 6 — Shell ML autocomplete (pure-presentar port).
//!
//! The interactive.paiml.com /shell-ml demo, rebuilt as a canvas-painted
//! UI instead of CSS+HTML. Loads the *real* trained `aprender-shell-base.apr`
//! 3-gram Markov model (byte-for-byte identical to what production serves)
//! via `presentar::browser::ShellAutocomplete::load_from_bytes` — same loader
//! path the production page uses, with the `.apr` bytes baked in via
//! `include_bytes!` since this wasm bundle paints synchronously.
//! Every glyph painted via Rust.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use crate::logic::shell::{lookup, should_ignore_modifier_key};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct ShellState {
    input: String,
    suggestions: Vec<String>,
    blink: bool,
}

fn paint(ctx: &web_sys::CanvasRenderingContext2d, w: f64, h: f64, s: &ShellState) {
    ctx.set_fill_style_str("#1a1b26");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Header
    ctx.set_fill_style_str("#7aa2f7");
    ctx.set_font("28px monospace");
    let _ = ctx.fill_text("Shell ML Autocomplete", 60.0, 60.0);
    ctx.set_fill_style_str("#565f89");
    ctx.set_font("14px monospace");
    let _ = ctx.fill_text(
        "aprender-shell · Markov model · type to see top-3 suggestions",
        60.0,
        90.0,
    );

    // Prompt
    let prompt_y = 180.0;
    let prompt_x = 60.0;
    ctx.set_fill_style_str("#1f2335");
    ctx.fill_rect(prompt_x, prompt_y, w - 120.0, 60.0);
    ctx.set_stroke_style_str("#7aa2f7");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(prompt_x, prompt_y, w - 120.0, 60.0);

    ctx.set_fill_style_str("#9ece6a");
    ctx.set_font("24px monospace");
    let _ = ctx.fill_text("$", prompt_x + 16.0, prompt_y + 40.0);
    ctx.set_fill_style_str("#c0caf5");
    let _ = ctx.fill_text(&s.input, prompt_x + 48.0, prompt_y + 40.0);

    // Cursor blink
    if s.blink {
        let m = ctx.measure_text(&s.input).ok();
        let cursor_x = prompt_x + 48.0 + m.as_ref().map(|m| m.width()).unwrap_or(0.0) + 2.0;
        ctx.set_fill_style_str("#c0caf5");
        ctx.fill_rect(cursor_x, prompt_y + 18.0, 12.0, 28.0);
    }

    // Suggestions panel
    let panel_y = prompt_y + 80.0;
    ctx.set_fill_style_str("#1f2335");
    ctx.fill_rect(prompt_x, panel_y, w - 120.0, 240.0);
    ctx.set_stroke_style_str("#414868");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(prompt_x, panel_y, w - 120.0, 240.0);

    ctx.set_fill_style_str("#bb9af7");
    ctx.set_font("16px monospace");
    let _ = ctx.fill_text(
        &format!("top-{} suggestions", s.suggestions.len()),
        prompt_x + 16.0,
        panel_y + 30.0,
    );

    ctx.set_font("22px monospace");
    for (i, sug) in s.suggestions.iter().enumerate() {
        let row_y = panel_y + 70.0 + (i as f64) * 50.0;
        if i == 0 {
            // Highlight top match
            ctx.set_fill_style_str("#3d59a1");
            ctx.fill_rect(prompt_x + 8.0, row_y - 30.0, w - 136.0, 44.0);
        }
        ctx.set_fill_style_str(if i == 0 { "#c0caf5" } else { "#a9b1d6" });
        let _ = ctx.fill_text(&format!("{}.", i + 1), prompt_x + 24.0, row_y);
        // Color the matched prefix differently
        let m = ctx.measure_text(&format!("{}.", i + 1)).ok();
        let x = prompt_x + 56.0 + m.map(|m| m.width()).unwrap_or(20.0);
        let common = s
            .input
            .chars()
            .zip(sug.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let (matched, rest) = sug.split_at(common.min(sug.len()));
        ctx.set_fill_style_str(&rgb(0.96, 0.72, 0.0));
        let _ = ctx.fill_text(matched, x, row_y);
        let m2 = ctx.measure_text(matched).ok();
        ctx.set_fill_style_str("#c0caf5");
        let _ = ctx.fill_text(rest, x + m2.map(|m| m.width()).unwrap_or(0.0), row_y);
    }

    if s.suggestions.is_empty() {
        ctx.set_fill_style_str("#565f89");
        ctx.set_font("18px monospace");
        let _ = ctx.fill_text(
            "(no model match — try: g · cargo · docker · make · python)",
            prompt_x + 24.0,
            panel_y + 110.0,
        );
    }

    // Footer
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("14px monospace");
    let _ = ctx.fill_text(
        "Same flow as interactive.paiml.com/shell-ml — pure presentar, no CSS, no DOM tree",
        prompt_x,
        h - 30.0,
    );
}

#[wasm_bindgen]
pub fn mount_shell(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let state = Rc::new(RefCell::new(ShellState {
        input: String::new(),
        suggestions: vec![],
        blink: true,
    }));

    paint(&ctx, w, h, &state.borrow());

    // Listen for keypress on the document (no input element)
    let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let doc = win.document().ok_or_else(|| JsValue::from_str("no doc"))?;
    let state_keys = state.clone();
    let ctx_keys = ctx.clone();
    let on_key = Closure::<dyn FnMut(_)>::new(move |evt: web_sys::KeyboardEvent| {
        // BUG #12 fix (gist round 2): pressing Ctrl/Cmd-modified keys
        // would have their character payload appended literally to the
        // buffer (Ctrl+A → input grew by `a`). Drop modifier-tagged
        // events so the in-canvas "input" doesn't grab browser shortcuts.
        if should_ignore_modifier_key(evt.ctrl_key(), evt.meta_key()) {
            return;
        }
        let key = evt.key();
        let mut s = state_keys.borrow_mut();
        match key.as_str() {
            "Backspace" => {
                s.input.pop();
            }
            "Enter" => {
                if let Some(first) = s.suggestions.first() {
                    s.input = first.clone();
                }
            }
            "Escape" => {
                s.input.clear();
            }
            k if k.chars().count() == 1 => {
                s.input.push_str(&key);
            }
            _ => return,
        }
        s.suggestions = lookup(&s.input);
        paint(&ctx_keys, w, h, &s);
    });
    doc.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())?;
    on_key.forget();

    // Cursor blink loop
    let state_blink = state.clone();
    let ctx_blink = ctx.clone();
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        let mut s = state_blink.borrow_mut();
        s.blink = !s.blink;
        paint(&ctx_blink, w, h, &s);
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                500,
            );
        }
    }));
    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        500,
    );

    let _ = canvas;
    Ok(())
}
