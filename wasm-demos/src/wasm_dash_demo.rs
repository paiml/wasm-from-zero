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

    // Walk the same paint list build_paint_list produces and render.
    // BUG FIX (gist): build_paint_list emits 3 HEADER nodes (`title`,
    // `event-pulse`, `mem-gauge`) at the top with fill=0.0 — those are
    // placeholders; we special-case them to show the actual title /
    // event-count / mem percentage so the dashboard reads as a
    // dashboard, not as three empty cells stamped "0%".
    for cmd in build_paint_list(dash) {
        let r = cmd.fg.r.clamp(0.0, 1.0) as f64;
        let g = cmd.fg.g.clamp(0.0, 1.0) as f64;
        let b = cmd.fg.b.clamp(0.0, 1.0) as f64;
        // background
        ctx.set_fill_style_str("#161b22");
        ctx.fill_rect(cmd.bounds.x, cmd.bounds.y, cmd.bounds.w, cmd.bounds.h);

        match cmd.label.as_str() {
            "title" => {
                ctx.set_fill_style_str("#a5d6ff");
                ctx.set_font("22px monospace");
                let _ = ctx.fill_text(
                    "wasm-from-zero · capstone",
                    cmd.bounds.x + 10.0,
                    cmd.bounds.y + 36.0,
                );
            }
            "event-pulse" => {
                ctx.set_fill_style_str(&rgb(0.5, 0.6, 1.0));
                ctx.set_font("28px monospace");
                let _ = ctx.fill_text(
                    &format!("⚡ {}", dash.event_count),
                    cmd.bounds.x + 10.0,
                    cmd.bounds.y + 40.0,
                );
                ctx.set_fill_style_str("#7c3aed");
                ctx.set_font("11px monospace");
                let _ = ctx.fill_text("events", cmd.bounds.x + 10.0, cmd.bounds.y + 54.0);
            }
            "mem-gauge" => {
                let pct = dash.mem_used_gb / dash.mem_total_gb;
                ctx.set_fill_style_str(&rgb(0.2, 0.85, 0.5));
                ctx.set_font("28px monospace");
                let _ = ctx.fill_text(
                    &format!("{:.0}%", pct * 100.0),
                    cmd.bounds.x + 10.0,
                    cmd.bounds.y + 40.0,
                );
                ctx.set_fill_style_str("#7c3aed");
                ctx.set_font("11px monospace");
                let _ = ctx.fill_text(
                    &format!("mem · {:.1}/{:.1} GB", dash.mem_used_gb, dash.mem_total_gb),
                    cmd.bounds.x + 10.0,
                    cmd.bounds.y + 54.0,
                );
            }
            "mem-bar" => {
                // BUG #10 fix (gist round 2): mem-bar is a wide
                // horizontal strip (w=viewport.w, h=40); previously
                // it fell through to the vertical-fill branch so the
                // green fill always painted at 100% width regardless
                // of the gauge percentage. Fill width by gauge.
                let fill_w = cmd.bounds.w * cmd.fill.clamp(0.0, 1.0);
                ctx.set_fill_style_str(&rgb(r, g, b));
                ctx.fill_rect(cmd.bounds.x, cmd.bounds.y, fill_w, cmd.bounds.h);
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
            _ => {
                // CPU bars — fill is the gauge, painted vertically.
                let fill_h = cmd.bounds.h * cmd.fill.clamp(0.0, 1.0);
                ctx.set_fill_style_str(&rgb(r, g, b));
                ctx.fill_rect(
                    cmd.bounds.x,
                    cmd.bounds.y + cmd.bounds.h - fill_h,
                    cmd.bounds.w,
                    fill_h,
                );
                ctx.set_fill_style_str("#c9d1d9");
                // BUG #11 (gist round 2/3): users reported reading
                // `cpu-0` as `cpu-8` at 13px. The hyphen-minus at small
                // font sizes can blend with the `0` glyph's loop.
                // Rename "cpu-N" → "CPU N" (space separator + caps) +
                // bump to 14px ui-monospace which has unambiguous digit
                // glyphs across OS font stacks.
                let display_label = if let Some(n) = cmd.label.strip_prefix("cpu-") {
                    format!("CPU {n}")
                } else {
                    cmd.label.clone()
                };
                ctx.set_font("14px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace");
                let _ = ctx.fill_text(&display_label, cmd.bounds.x + 6.0, cmd.bounds.y + 18.0);
                ctx.set_font("11px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace");
                let _ = ctx.fill_text(
                    &format!("{:.0}%", cmd.fill * 100.0),
                    cmd.bounds.x + 6.0,
                    cmd.bounds.y + cmd.bounds.h - 6.0,
                );
            }
        }
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
