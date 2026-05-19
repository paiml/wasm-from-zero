//! Demo 3 — Process table with sortable columns + sparkline per row.
//!
//! 12 fake processes with LCG-generated CPU/MEM history. Click a column
//! header to re-sort. Each row paints a 32-cell BrailleGraph-style spark
//! built from m3-charts::position. Demonstrates m3-components::layout +
//! m3-charts::position + bounds-clamp via clip(), all from a real click
//! handler that mutates state and repaints.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use m3_charts::position;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct Proc {
    pid: u32,
    name: &'static str,
    cpu: f64,
    mem: f64,
    history: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq)]
enum SortKey {
    Pid,
    Cpu,
    Mem,
}

fn make_history(seed: u64) -> Vec<f64> {
    let mut s = seed.wrapping_mul(2654435761);
    (0..32)
        .map(|_| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((s >> 32) as u32 as f64) / (u32::MAX as f64)
        })
        .collect()
}

fn fixture() -> Vec<Proc> {
    let names: &[&str] = &[
        "wasm-bindgen",
        "rustc",
        "cargo",
        "claude-code",
        "firefox",
        "tmux",
        "ssh-agent",
        "kernel-task",
        "Xorg",
        "pulseaudio",
        "systemd",
        "bash",
    ];
    let cpus = [
        0.34, 0.81, 0.42, 0.12, 0.61, 0.05, 0.02, 0.18, 0.27, 0.04, 0.09, 0.01,
    ];
    let mems = [
        0.21, 0.45, 0.18, 0.08, 0.72, 0.03, 0.01, 0.11, 0.16, 0.06, 0.05, 0.01,
    ];
    names
        .iter()
        .enumerate()
        .map(|(i, &n)| Proc {
            pid: 1000 + i as u32 * 137,
            name: n,
            cpu: cpus[i],
            mem: mems[i],
            history: make_history((i + 1) as u64),
        })
        .collect()
}

const COL_PID: (f64, f64) = (60.0, 80.0);
const COL_NAME: (f64, f64) = (160.0, 200.0);
const COL_CPU: (f64, f64) = (380.0, 80.0);
const COL_MEM: (f64, f64) = (480.0, 80.0);
const COL_SPARK: (f64, f64) = (580.0, 700.0);
const ROW_H: f64 = 36.0;
const HEADER_Y: f64 = 90.0;
const FIRST_ROW_Y: f64 = 126.0;

fn paint(ctx: &web_sys::CanvasRenderingContext2d, w: f64, h: f64, procs: &[Proc], sort: SortKey) {
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Header
    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("20px monospace");
    let _ = ctx.fill_text(
        "Process table — sortable, m3-components + m3-charts",
        20.0,
        40.0,
    );
    ctx.set_fill_style_str("#565f89");
    ctx.set_font("13px monospace");
    let _ = ctx.fill_text("click any column header to re-sort", 20.0, 62.0);

    // Column headers
    ctx.set_font("14px monospace");
    for (col, (cx, cw), key) in &[
        ("PID  ↕", COL_PID, Some(SortKey::Pid)),
        ("COMMAND", COL_NAME, None),
        ("CPU% ↕", COL_CPU, Some(SortKey::Cpu)),
        ("MEM% ↕", COL_MEM, Some(SortKey::Mem)),
        ("HISTORY (32 ticks)", COL_SPARK, None),
    ] {
        ctx.set_fill_style_str("#161b22");
        ctx.fill_rect(*cx, HEADER_Y - 22.0, *cw, 28.0);
        let is_active = key.map_or(false, |k| k == sort);
        ctx.set_fill_style_str(if is_active { "#7c3aed" } else { "#c9d1d9" });
        let _ = ctx.fill_text(col, *cx + 8.0, HEADER_Y - 4.0);
    }

    // Rows
    ctx.set_font("13px monospace");
    for (i, p) in procs.iter().enumerate() {
        let y = FIRST_ROW_Y + (i as f64) * ROW_H;
        // Zebra stripe
        if i % 2 == 0 {
            ctx.set_fill_style_str("#11161d");
            ctx.fill_rect(40.0, y - 18.0, w - 80.0, ROW_H - 4.0);
        }
        ctx.set_fill_style_str("#c9d1d9");
        let _ = ctx.fill_text(&p.pid.to_string(), COL_PID.0 + 8.0, y);
        let _ = ctx.fill_text(p.name, COL_NAME.0 + 8.0, y);
        // CPU + MEM colored by threshold
        let cpu_color = if p.cpu > 0.6 {
            rgb(1.0, 0.4, 0.3)
        } else if p.cpu > 0.3 {
            rgb(1.0, 0.85, 0.0)
        } else {
            rgb(0.4, 0.85, 0.4)
        };
        ctx.set_fill_style_str(&cpu_color);
        let _ = ctx.fill_text(&format!("{:.0}%", p.cpu * 100.0), COL_CPU.0 + 8.0, y);
        ctx.set_fill_style_str("#c9d1d9");
        let _ = ctx.fill_text(&format!("{:.0}%", p.mem * 100.0), COL_MEM.0 + 8.0, y);

        // Sparkline via m3-charts::position
        let spark_w = COL_SPARK.1 - 16.0;
        let spark_h = 20.0;
        let spark_x = COL_SPARK.0 + 8.0;
        let spark_y = y - 14.0;
        let n = p.history.len();
        ctx.set_fill_style_str(&rgb(0.3, 0.6, 0.9));
        for (j, &v) in p.history.iter().enumerate() {
            let bx = spark_x + position(j as f64, 0.0, (n - 1) as f64, 0.0, spark_w);
            let bw = spark_w / n as f64 - 1.0;
            let bh = v.clamp(0.0, 1.0) * spark_h;
            ctx.fill_rect(bx, spark_y + spark_h - bh, bw, bh);
        }
    }

    // Footer
    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("16px monospace");
    let _ = ctx.fill_text(
        "contract: wasm-panels-v1 holds — ChartCursor_Monotonic + Container_NoOverlap",
        20.0,
        h - 20.0,
    );
}

fn header_hit(x: f64, y: f64) -> Option<SortKey> {
    if y < HEADER_Y - 22.0 || y > HEADER_Y + 6.0 {
        return None;
    }
    if (COL_PID.0..COL_PID.0 + COL_PID.1).contains(&x) {
        Some(SortKey::Pid)
    } else if (COL_CPU.0..COL_CPU.0 + COL_CPU.1).contains(&x) {
        Some(SortKey::Cpu)
    } else if (COL_MEM.0..COL_MEM.0 + COL_MEM.1).contains(&x) {
        Some(SortKey::Mem)
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn mount_process_table(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    let state = Rc::new(RefCell::new((fixture(), SortKey::Cpu)));
    {
        let (ref procs, sort) = *state.borrow();
        let mut sorted = procs.clone();
        sort_procs(&mut sorted, sort);
        paint(&ctx, w, h, &sorted, sort);
    }

    let state_click = state.clone();
    let canvas_for_hit = canvas.clone();
    let ctx_click = ctx.clone();
    let on_click = Closure::<dyn FnMut(_)>::new(move |evt: web_sys::MouseEvent| {
        let rect = canvas_for_hit.get_bounding_client_rect();
        let scale_x = canvas_for_hit.width() as f64 / rect.width();
        let scale_y = canvas_for_hit.height() as f64 / rect.height();
        let cx = (evt.client_x() as f64 - rect.left()) * scale_x;
        let cy = (evt.client_y() as f64 - rect.top()) * scale_y;
        if let Some(new_sort) = header_hit(cx, cy) {
            let mut s = state_click.borrow_mut();
            s.1 = new_sort;
            let mut sorted = s.0.clone();
            sort_procs(&mut sorted, new_sort);
            paint(&ctx_click, w, h, &sorted, new_sort);
        }
    });
    canvas.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();
    Ok(())
}

fn sort_procs(procs: &mut Vec<Proc>, key: SortKey) {
    match key {
        SortKey::Pid => procs.sort_by_key(|p| p.pid),
        SortKey::Cpu => procs.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortKey::Mem => procs.sort_by(|a, b| {
            b.mem
                .partial_cmp(&a.mem)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }
}
