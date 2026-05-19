//! Demo 5 — wasm-dash live (current m5-dash, animated).
//!
//! The capstone dashboard from the static demo, but driven by an LCG
//! tick at 30Hz: CPU bars jitter, memory bar drifts, sparkline strip
//! scrolls left. Same `m5_dash::build_paint_list` walks every frame.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use m5_dash::{build_paint_list, Dashboard};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*seed >> 32) as u32 as f64) / (u32::MAX as f64)
}

fn jitter(dash: &mut Dashboard, seed: &mut u64) {
    for v in dash.cpu_load.iter_mut() {
        let delta = (lcg(seed) - 0.5) * 0.08;
        *v = (*v + delta).clamp(0.05, 0.99);
    }
    let mem_delta = (lcg(seed) - 0.5) * 0.4;
    dash.mem_used_gb = (dash.mem_used_gb + mem_delta).clamp(0.5, dash.mem_total_gb - 0.5);
    dash.event_count += 1;
}

fn paint(ctx: &web_sys::CanvasRenderingContext2d, w: f64, h: f64, dash: &Dashboard) {
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Header
    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("22px monospace");
    let _ = ctx.fill_text(&dash.title, 20.0, 36.0);
    ctx.set_fill_style_str("#565f89");
    ctx.set_font("13px monospace");
    let _ = ctx.fill_text(
        &format!(
            "frame {}  ·  30 Hz  ·  build_paint_list(Dashboard::fixture()) → live",
            dash.event_count
        ),
        20.0,
        56.0,
    );

    // Walk the same paint list build_paint_list produces and render
    for cmd in build_paint_list(dash) {
        let r = cmd.fg.r.clamp(0.0, 1.0) as f64;
        let g = cmd.fg.g.clamp(0.0, 1.0) as f64;
        let b = cmd.fg.b.clamp(0.0, 1.0) as f64;
        // background
        ctx.set_fill_style_str("#161b22");
        ctx.fill_rect(cmd.bounds.x, cmd.bounds.y, cmd.bounds.w, cmd.bounds.h);
        // fill
        let fill_h = cmd.bounds.h * cmd.fill.clamp(0.0, 1.0);
        ctx.set_fill_style_str(&rgb(r, g, b));
        ctx.fill_rect(
            cmd.bounds.x,
            cmd.bounds.y + cmd.bounds.h - fill_h,
            cmd.bounds.w,
            fill_h,
        );
        // label
        ctx.set_fill_style_str("#c9d1d9");
        ctx.set_font("13px monospace");
        let _ = ctx.fill_text(&cmd.label, cmd.bounds.x + 6.0, cmd.bounds.y + 16.0);
        ctx.set_font("11px monospace");
        let _ = ctx.fill_text(
            &format!("{:.0}%", cmd.fill * 100.0),
            cmd.bounds.x + 6.0,
            cmd.bounds.y + cmd.bounds.h - 6.0,
        );
    }
    let _ = w;
    let _ = h;
}

#[wasm_bindgen]
pub fn mount_wasm_dash(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let dash = Rc::new(RefCell::new(Dashboard::fixture()));
    let seed = Rc::new(RefCell::new(0xC0FFEE_u64));
    let frame = Rc::new(RefCell::new(0_u32));

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let dash_loop = dash.clone();
    let seed_loop = seed.clone();
    let frame_loop = frame.clone();
    let ctx_loop = ctx.clone();

    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        // Throttle to ~30 Hz
        *frame_loop.borrow_mut() += 1;
        if *frame_loop.borrow() % 2 == 0 {
            jitter(&mut dash_loop.borrow_mut(), &mut seed_loop.borrow_mut());
        }
        paint(&ctx_loop, w, h, &dash_loop.borrow());
        if let Some(win) = web_sys::window() {
            let _ =
                win.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }));

    let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    win.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    let _ = canvas;
    Ok(())
}
