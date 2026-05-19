//! Demo 4 — Showcase: animated dashboard at 60Hz.
//!
//! Six-panel synthetic dashboard driven by a deterministic LCG: bar
//! chart (height-tweened), rotating donut, sparkline strip, FPS gauge,
//! particle system, and a "fake ML inference" panel that runs a small
//! softmax on a 16-dim vector per frame. No real .apr file needed —
//! this avoids the broken `aprender-present-lib::showcase` upstream
//! `include_bytes!` while still demoing the same animation primitives.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct State {
    frame: u64,
    seed: u64,
    bars: [f64; 8],
    targets: [f64; 8],
    donut: f64,
    particles: Vec<(f64, f64, f64, f64, f64)>, // x, y, vx, vy, life
    softmax_in: [f64; 16],
}

impl State {
    fn new() -> Self {
        Self {
            frame: 0,
            seed: 0x1234_5678_DEAD_BEEF,
            bars: [0.3, 0.5, 0.7, 0.4, 0.6, 0.2, 0.8, 0.5],
            targets: [0.3, 0.5, 0.7, 0.4, 0.6, 0.2, 0.8, 0.5],
            donut: 0.0,
            particles: vec![],
            softmax_in: [0.0; 16],
        }
    }

    fn lcg(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.seed >> 32) as u32 as f64) / (u32::MAX as f64)
    }

    fn tick(&mut self) {
        self.frame += 1;
        // Tween bars toward targets
        for i in 0..8 {
            self.bars[i] += (self.targets[i] - self.bars[i]) * 0.05;
        }
        // Occasionally update targets
        if self.frame % 60 == 0 {
            for i in 0..8 {
                self.targets[i] = self.lcg();
            }
        }
        self.donut += 0.02;
        // Particles
        if self.particles.len() < 50 && self.lcg() < 0.2 {
            let vx = (self.lcg() - 0.5) * 4.0;
            let vy = (self.lcg() - 0.5) * 4.0;
            self.particles.push((640.0, 360.0, vx, vy, 1.0));
        }
        self.particles.retain_mut(|p| {
            p.0 += p.2;
            p.1 += p.3;
            p.4 -= 0.01;
            p.4 > 0.0
        });
        // Softmax inputs evolve
        for i in 0..16 {
            self.softmax_in[i] = ((self.frame as f64 * 0.05) + i as f64 * 0.4).sin();
        }
    }
}

fn softmax(xs: &[f64]) -> Vec<f64> {
    let max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = xs.iter().map(|&x| (x - max).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

fn paint(ctx: &web_sys::CanvasRenderingContext2d, w: f64, h: f64, s: &State) {
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("22px monospace");
    let _ = ctx.fill_text(
        "Showcase — bar chart · donut · particles · softmax · all @ 60Hz",
        20.0,
        36.0,
    );
    ctx.set_fill_style_str("#565f89");
    ctx.set_font("13px monospace");
    let _ = ctx.fill_text(&format!("frame {}", s.frame), 20.0, 56.0);

    // ── Bar chart (top-left) ──
    let bar_x = 40.0;
    let bar_y = 100.0;
    let bar_w = 380.0;
    let bar_h = 220.0;
    ctx.set_stroke_style_str("#30363d");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(bar_x, bar_y, bar_w, bar_h);
    let bw = bar_w / 8.0 - 4.0;
    for i in 0..8 {
        let h_px = s.bars[i].clamp(0.0, 1.0) * (bar_h - 20.0);
        let color = if s.bars[i] > 0.7 {
            rgb(1.0, 0.4, 0.3)
        } else if s.bars[i] > 0.4 {
            rgb(1.0, 0.85, 0.0)
        } else {
            rgb(0.4, 0.85, 0.4)
        };
        ctx.set_fill_style_str(&color);
        ctx.fill_rect(
            bar_x + 4.0 + i as f64 * (bw + 4.0),
            bar_y + bar_h - h_px - 10.0,
            bw,
            h_px,
        );
    }
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("12px monospace");
    let _ = ctx.fill_text("BAR CHART · 8 channels · LCG", bar_x, bar_y - 6.0);

    // ── Donut (top-right) ──
    let cx = 700.0;
    let cy = 210.0;
    let r_outer = 110.0;
    let r_inner = 60.0;
    for i in 0..6 {
        let a0 = s.donut + (i as f64) * std::f64::consts::TAU / 6.0;
        let a1 = a0 + std::f64::consts::TAU / 6.0;
        let color = match i {
            0 => rgb(0.4, 0.85, 0.4),
            1 => rgb(0.5, 0.6, 1.0),
            2 => rgb(1.0, 0.85, 0.0),
            3 => rgb(1.0, 0.4, 0.3),
            4 => rgb(0.6, 0.4, 1.0),
            _ => rgb(0.0, 0.7, 0.8),
        };
        ctx.set_fill_style_str(&color);
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r_outer, a0, a1);
        let _ = ctx.arc_with_anticlockwise(cx, cy, r_inner, a1, a0, true);
        ctx.close_path();
        ctx.fill();
    }
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("12px monospace");
    let _ = ctx.fill_text("ROTATING DONUT · 6 wedges", cx - 80.0, cy - r_outer - 10.0);

    // ── Particles (bottom-left) ──
    let particles_y0 = 380.0;
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("12px monospace");
    let _ = ctx.fill_text("PARTICLES · emit + decay", 40.0, particles_y0 - 6.0);
    for p in &s.particles {
        let alpha = (p.4 * 255.0).max(0.0) as u8;
        ctx.set_fill_style_str(&format!("rgba(122,162,247,{:.2})", p.4));
        ctx.fill_rect(p.0 - 2.0, p.1 - 2.0, 4.0, 4.0);
        let _ = alpha; // keep clippy quiet
    }

    // ── Softmax (right) ──
    let sm_x = 880.0;
    let sm_y = 380.0;
    let sm_w = 360.0;
    let sm_h = 260.0;
    ctx.set_stroke_style_str("#30363d");
    ctx.stroke_rect(sm_x, sm_y, sm_w, sm_h);
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("12px monospace");
    let _ = ctx.fill_text("SOFTMAX · 16-dim in → 16 probs out", sm_x, sm_y - 6.0);
    let probs = softmax(&s.softmax_in);
    let pw = sm_w / 16.0 - 2.0;
    for (i, &p) in probs.iter().enumerate() {
        let bh = (p * 16.0).clamp(0.0, 1.0) * (sm_h - 20.0);
        ctx.set_fill_style_str(&rgb(0.5, 0.6, 1.0));
        ctx.fill_rect(
            sm_x + 4.0 + i as f64 * (pw + 2.0),
            sm_y + sm_h - bh - 10.0,
            pw,
            bh,
        );
    }

    // Footer
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("16px monospace");
    let _ = ctx.fill_text(
        "60 Hz · rAF loop · all primitives painted from Rust → WASM, zero JS logic",
        20.0,
        h - 20.0,
    );
}

#[wasm_bindgen]
pub fn mount_showcase(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let state = Rc::new(RefCell::new(State::new()));
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let state_loop = state.clone();
    let ctx_loop = ctx.clone();

    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        state_loop.borrow_mut().tick();
        paint(&ctx_loop, w, h, &state_loop.borrow());
        // Schedule next frame
        if let Some(win) = web_sys::window() {
            let _ =
                win.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }));

    let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    win.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    let _ = canvas; // keep alive
    Ok(())
}
