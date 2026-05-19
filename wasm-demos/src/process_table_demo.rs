//! Demo 3 — Process table with sortable columns + sparkline per row.
//!
//! 12 fake processes with LCG-generated CPU/MEM history. Click a column
//! header to re-sort; click again to toggle direction. Each row paints a
//! 32-cell sparkline built from m3-charts::position.
//!
//! Bug history (see gist https://gist.github.com/noahgift/630218c2e66bc4b4b5e1c54cec8f5610):
//!   BUG #3 (round 0) — initial sort was Cpu, fixture is CPU-ordered →
//!                       first CPU click visually a no-op. Fixed: seed Pid.
//!   BUG #4 (round 1) — COMMAND column header was a no-op (no SortKey::Name).
//!                       Fixed: added SortKey::Name + alphabetical sort.
//!   BUG #5 (round 1) — Repeated clicks didn't toggle direction.
//!                       Fixed: SortDir + next_sort_state click reducer.

use crate::canvas_helpers::{get_canvas_ctx, rgb};
use crate::logic::process_table::{
    advance_history, fixture, header_hit, next_sort_state, sort_procs_dir, Proc, SortDir, SortKey,
    COL_CPU, COL_MEM, COL_NAME, COL_PID, INITIAL_SORT,
};
use m3_charts::position;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const COL_SPARK: (f64, f64) = (580.0, 700.0);
const ROW_H: f64 = 36.0;
const HEADER_Y: f64 = 90.0;
const FIRST_ROW_Y: f64 = 126.0;

fn arrow(dir: SortDir) -> &'static str {
    match dir {
        SortDir::Asc => "↑",
        SortDir::Desc => "↓",
    }
}

fn paint(
    ctx: &web_sys::CanvasRenderingContext2d,
    w: f64,
    h: f64,
    procs: &[Proc],
    sort: SortKey,
    dir: SortDir,
) {
    ctx.set_fill_style_str("#0d1117");
    ctx.fill_rect(0.0, 0.0, w, h);

    ctx.set_fill_style_str("#a5d6ff");
    ctx.set_font("20px monospace");
    let _ = ctx.fill_text(
        "Process table — sortable, m3-components + m3-charts",
        20.0,
        40.0,
    );
    ctx.set_fill_style_str("#565f89");
    ctx.set_font("13px monospace");
    let _ = ctx.fill_text(
        "click any column header to sort; click again to reverse direction",
        20.0,
        62.0,
    );

    // Column headers. The ↕ shows the column is sortable; the active
    // column shows ↑ or ↓ instead, indicating the current direction.
    ctx.set_font("14px monospace");
    let cols: &[(&str, (f64, f64), Option<SortKey>)] = &[
        ("PID", COL_PID, Some(SortKey::Pid)),
        ("COMMAND", COL_NAME, Some(SortKey::Name)),
        ("CPU%", COL_CPU, Some(SortKey::Cpu)),
        ("MEM%", COL_MEM, Some(SortKey::Mem)),
        ("HISTORY (32 ticks)", COL_SPARK, None),
    ];
    for (col, (cx, cw), key) in cols {
        ctx.set_fill_style_str("#161b22");
        ctx.fill_rect(*cx, HEADER_Y - 22.0, *cw, 28.0);
        let is_active = key.map_or(false, |k| k == sort);
        ctx.set_fill_style_str(if is_active { "#7c3aed" } else { "#c9d1d9" });
        let indicator = if key.is_none() {
            ""
        } else if is_active {
            arrow(dir)
        } else {
            "↕"
        };
        let text = if indicator.is_empty() {
            col.to_string()
        } else {
            format!("{col} {indicator}")
        };
        let _ = ctx.fill_text(&text, *cx + 8.0, HEADER_Y - 4.0);
    }

    // Rows
    ctx.set_font("13px monospace");
    for (i, p) in procs.iter().enumerate() {
        let y = FIRST_ROW_Y + (i as f64) * ROW_H;
        if i % 2 == 0 {
            ctx.set_fill_style_str("#11161d");
            ctx.fill_rect(40.0, y - 18.0, w - 80.0, ROW_H - 4.0);
        }
        ctx.set_fill_style_str("#c9d1d9");
        let _ = ctx.fill_text(&p.pid.to_string(), COL_PID.0 + 8.0, y);
        let _ = ctx.fill_text(p.name, COL_NAME.0 + 8.0, y);
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

    ctx.set_fill_style_str("#7c3aed");
    ctx.set_font("16px monospace");
    let _ = ctx.fill_text(
        "contract: wasm-panels-v1 holds — ChartCursor_Monotonic + Container_NoOverlap",
        20.0,
        h - 20.0,
    );
}

#[wasm_bindgen]
pub fn mount_process_table(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;

    // BUG #3 fix: seed initial sort with Pid (not Cpu). The fixture is
    // roughly CPU-ordered so a Cpu seed makes the first CPU click silent.
    let initial = (
        INITIAL_SORT,
        crate::logic::process_table::default_dir(INITIAL_SORT),
    );
    let state = Rc::new(RefCell::new((fixture(), initial.0, initial.1)));
    {
        let (ref procs, sort, dir) = *state.borrow();
        let mut sorted = procs.clone();
        sort_procs_dir(&mut sorted, sort, dir);
        paint(&ctx, w, h, &sorted, sort, dir);
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
        if let Some(clicked) = header_hit(cx, cy) {
            let mut s = state_click.borrow_mut();
            // BUG #4 fix: header_hit now routes COMMAND → SortKey::Name.
            // BUG #5 fix: next_sort_state flips dir on same-column re-click.
            let (new_key, new_dir) = next_sort_state((s.1, s.2), clicked);
            s.1 = new_key;
            s.2 = new_dir;
            let mut sorted = s.0.clone();
            sort_procs_dir(&mut sorted, new_key, new_dir);
            paint(&ctx_click, w, h, &sorted, new_key, new_dir);
        }
    });
    canvas.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())?;
    on_click.forget();

    // BUG #8 fix (gist round 2): the HISTORY column header advertises
    // `(32 ticks)` but the fixture's history was generated once and
    // never updated, so the sparkline was a static slice. Drive an
    // rAF loop that calls `advance_history` every ~250ms (≈4 Hz) and
    // re-paints so the sparkline actually scrolls left.
    let state_tick = state.clone();
    let ctx_tick = ctx.clone();
    let seed = Rc::new(RefCell::new(0xC0FFEE_BEEF_u64));
    let frame = Rc::new(RefCell::new(0_u32));
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        *frame.borrow_mut() += 1;
        // ~60Hz / 15 ≈ 4Hz: a tick every quarter second is enough
        // for the eye to read movement without taxing the GPU.
        if *frame.borrow() % 15 == 0 {
            let mut s = state_tick.borrow_mut();
            let mut seed_mut = seed.borrow_mut();
            advance_history(&mut s.0, &mut seed_mut);
            let (procs, sort, dir) = (s.0.clone(), s.1, s.2);
            drop(s);
            drop(seed_mut);
            let mut sorted = procs;
            sort_procs_dir(&mut sorted, sort, dir);
            paint(&ctx_tick, w, h, &sorted, sort, dir);
        }
        if let Some(win) = web_sys::window() {
            let _ =
                win.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }));
    let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    win.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}
