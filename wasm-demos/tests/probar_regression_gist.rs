#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — regression tests pinning the 3 bugs from
//! https://gist.github.com/noahgift/630218c2e66bc4b4b5e1c54cec8f5610
//!
//! Each test asserts on the *demo's* `paint_ops()` output — the pure
//! data the wasm `paint()` wrapper iterates. If a future edit regresses
//! the fix (e.g. removes the special-case `match` arm), these tests
//! fail because the expected FillText op will no longer be in the list.

use m2_elm_wasm::{init, update, Msg};
use m5_dash::Dashboard;
use wasm_demos::logic::counter::paint_ops as counter_paint_ops;
use wasm_demos::logic::process_table::{
    fixture, initial_paint_order, sort_procs, SortKey, INITIAL_SORT,
};
use wasm_demos::logic::wasm_dash::paint_ops as dash_paint_ops;
use wasm_demos::logic::DrawOp;

// ============================================================
// BUG #1 — counter value invisible
// ============================================================

#[test]
fn regression_bug1_counter_state0_paints_count_value_as_text() {
    // The pre-fix paint() filtered for `tag == "count"` — but view()
    // never emits such a tag. So the count value was never painted.
    // After fix, paint_ops() MUST contain a FillText op carrying the
    // current count.
    let state = init();
    let ops = counter_paint_ops(state, 1000.0, 480.0);
    let big_count_text = format!("{}", state.count);
    let painted = ops
        .iter()
        .any(|op| matches!(op, DrawOp::FillText(t, _, _) if *t == big_count_text));
    assert!(
        painted,
        "BUG #1 regressed: paint_ops() never emits the count text {:?}",
        big_count_text
    );
}

#[test]
fn regression_bug1_counter_updates_on_increment() {
    // Click [+] three times → paint_ops must emit "3" as the big text.
    let state = (0..3).fold(init(), |s, _| update(s, Msg::Increment));
    assert_eq!(state.count, 3);
    let ops = counter_paint_ops(state, 1000.0, 480.0);
    let three_painted = ops
        .iter()
        .any(|op| matches!(op, DrawOp::FillText(t, _, _) if t == "3"));
    assert!(
        three_painted,
        "After 3× Increment, paint_ops() must paint '3' as the count value"
    );
}

#[test]
fn regression_bug1_counter_paints_view_node_dump() {
    // Fix dumps the VDOM nodes above the big count for debug visibility —
    // each of "h1", "div", "small" should appear in some FillText.
    let ops = counter_paint_ops(init(), 1000.0, 480.0);
    let all_text: String = ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::FillText(t, _, _) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    for expected in ["h1", "div", "small"] {
        assert!(
            all_text.contains(expected),
            "VDOM dump missing tag {expected:?}"
        );
    }
}

// ============================================================
// BUG #2 — wasm-dash header panels empty
// ============================================================

#[test]
fn regression_bug2_dash_header_panels_paint_their_content() {
    // Pre-fix: header cmds (title/event-pulse/mem-gauge) emitted with
    // fill=0.0; default branch drew only the label + "0%". Fix is a
    // 3-arm match emitting rich content. We assert each arm fired.
    let dash = Dashboard::fixture();
    let ops = dash_paint_ops(&dash, 1280.0, 720.0);
    let all_text: String = ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::FillText(t, _, _) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        all_text.contains("wasm-from-zero · capstone"),
        "BUG #2.title regressed: title arm should paint 'wasm-from-zero · capstone'"
    );
    assert!(
        all_text.contains(&format!("⚡ {}", dash.event_count)),
        "BUG #2.event-pulse regressed: must paint '⚡ {}'",
        dash.event_count
    );
    assert!(
        all_text.contains("events"),
        "event-pulse footer label missing"
    );
    assert!(
        all_text.contains(&format!(
            "{:.1}/{:.1} GB",
            dash.mem_used_gb, dash.mem_total_gb
        )),
        "BUG #2.mem-gauge regressed: must paint GB ratio"
    );
}

#[test]
fn regression_bug2_dash_header_does_not_just_say_zero_percent() {
    // The pre-fix bug signature was a literal "0%" + empty body in each
    // header cell. Assert no FillText reads exactly "0%".
    let dash = Dashboard::fixture();
    let ops = dash_paint_ops(&dash, 1280.0, 720.0);
    let zero_pct = ops
        .iter()
        .filter(|op| matches!(op, DrawOp::FillText(t, _, _) if t == "0%"))
        .count();
    assert_eq!(
        zero_pct, 0,
        "BUG #2 regressed: paint_ops() emitted '0%' — the bug-era placeholder shape"
    );
}

// ============================================================
// BUG #3 — process-table first-click is no-op
// ============================================================

#[test]
fn regression_bug3_initial_sort_is_not_cpu() {
    // Initial sort = Cpu + fixture is roughly CPU-ordered →
    // first CPU click did nothing visible. Fix: seed with Pid.
    assert_ne!(
        INITIAL_SORT,
        SortKey::Cpu,
        "BUG #3 regressed: demo initial sort is Cpu, which is the natural fixture order"
    );
}

#[test]
fn regression_bug3_initial_paint_order_differs_from_cpu_order() {
    let mut by_cpu = fixture();
    sort_procs(&mut by_cpu, SortKey::Cpu);
    let cpu_pids: Vec<u32> = by_cpu.into_iter().map(|p| p.pid).collect();
    let initial = initial_paint_order();
    assert_ne!(
        initial, cpu_pids,
        "BUG #3 regressed: initial paint order equals CPU-sort order"
    );
}

#[test]
fn regression_bug3_initial_paint_order_differs_from_mem_order() {
    let mut by_mem = fixture();
    sort_procs(&mut by_mem, SortKey::Mem);
    let mem_pids: Vec<u32> = by_mem.into_iter().map(|p| p.pid).collect();
    let initial = initial_paint_order();
    assert_ne!(
        initial, mem_pids,
        "BUG #3 regressed: initial paint order equals MEM-sort order"
    );
}

// ============================================================
// BUG #4 (gist round 1, issue 3a) — COMMAND column did not sort
// ============================================================

#[test]
fn regression_bug4_command_column_hit_routes_to_sortkey_name() {
    use wasm_demos::logic::process_table::{header_hit, COL_NAME, HEADER_Y};
    let cx = COL_NAME.0 + COL_NAME.1 / 2.0;
    let cy = HEADER_Y - 10.0;
    let hit = header_hit(cx, cy);
    assert_eq!(
        hit,
        Some(SortKey::Name),
        "BUG #4 regressed: clicking COMMAND header didn't route to SortKey::Name"
    );
}

#[test]
fn regression_bug4_sort_by_name_orders_alphabetically() {
    use wasm_demos::logic::process_table::{fixture, sort_procs_dir, SortDir};
    let mut procs = fixture();
    sort_procs_dir(&mut procs, SortKey::Name, SortDir::Asc);
    let names: Vec<&str> = procs.iter().map(|p| p.name).collect();
    // BUG #9 (round 2) made COMMAND sort case-insensitive; expected
    // order is the lower-cased sort, not the raw `str::cmp` sort.
    let mut expected = names.clone();
    expected.sort_by_key(|n| n.to_ascii_lowercase());
    assert_eq!(
        names, expected,
        "BUG #4 regressed: SortKey::Name didn't produce alphabetical order"
    );
    // Spot-check: "bash" alphabetically before "cargo" before "claude-code"
    let bash_idx = names.iter().position(|&n| n == "bash").unwrap();
    let cargo_idx = names.iter().position(|&n| n == "cargo").unwrap();
    let claude_idx = names.iter().position(|&n| n == "claude-code").unwrap();
    assert!(bash_idx < cargo_idx && cargo_idx < claude_idx);
}

// ============================================================
// BUG #5 (gist round 1, issue 3b) — repeated clicks didn't toggle direction
// ============================================================

#[test]
fn regression_bug5_repeated_click_flips_direction() {
    use wasm_demos::logic::process_table::{default_dir, next_sort_state, SortDir};
    // Start with PID-asc, click PID again → must flip to PID-desc.
    let s0 = (SortKey::Pid, default_dir(SortKey::Pid));
    assert_eq!(s0.1, SortDir::Asc);
    let s1 = next_sort_state(s0, SortKey::Pid);
    assert_eq!(
        s1,
        (SortKey::Pid, SortDir::Desc),
        "BUG #5 regressed: second click on same column didn't flip direction"
    );
    // Third click → back to asc
    let s2 = next_sort_state(s1, SortKey::Pid);
    assert_eq!(s2, (SortKey::Pid, SortDir::Asc));
}

#[test]
fn regression_bug5_different_column_uses_default_dir_not_flip() {
    use wasm_demos::logic::process_table::{default_dir, next_sort_state, SortDir};
    // Start PID-desc; clicking CPU should give CPU-DESC (CPU default),
    // not CPU-asc (which would be a flip of PID-desc).
    let s0 = (SortKey::Pid, SortDir::Desc);
    let s1 = next_sort_state(s0, SortKey::Cpu);
    assert_eq!(s1, (SortKey::Cpu, default_dir(SortKey::Cpu)));
    assert_eq!(s1.1, SortDir::Desc); // CPU's default
}

#[test]
fn regression_bug5_pid_asc_then_pid_desc_yield_reversed_lists() {
    use wasm_demos::logic::process_table::{fixture, sort_procs_dir, SortDir};
    let mut asc = fixture();
    sort_procs_dir(&mut asc, SortKey::Pid, SortDir::Asc);
    let mut desc = fixture();
    sort_procs_dir(&mut desc, SortKey::Pid, SortDir::Desc);
    let asc_pids: Vec<u32> = asc.iter().map(|p| p.pid).collect();
    let mut desc_expected = asc_pids.clone();
    desc_expected.reverse();
    let desc_pids: Vec<u32> = desc.iter().map(|p| p.pid).collect();
    assert_eq!(
        desc_pids, desc_expected,
        "BUG #5 regressed: Desc didn't reverse the Asc list"
    );
}

// ============================================================
// BUG #6 (gist round 2) — canvas contract assertion unsound:
// footer claimed `survived: 3` but only 2 rectangles painted.
// ============================================================

#[test]
fn regression_bug6_canvas_paints_one_rect_per_survivor() {
    use m1_canvas::CanvasDims;
    use wasm_demos::logic::canvas::{count_survivor_rects, paint_ops};
    let dims = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let ops = paint_ops(dims, 1280.0, 720.0);
    let rects = count_survivor_rects(&ops);
    // 3 survivors → 3 outline StrokeRects at literal clipped coords.
    assert_eq!(
        rects, 3,
        "BUG #6 regressed: canvas paint_ops painted {rects} outline rects, not 3"
    );
}

#[test]
fn regression_bug6_canvas_paints_at_literal_clipped_positions() {
    use m1_canvas::CanvasDims;
    use wasm_demos::logic::canvas::paint_ops;
    let dims = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let ops = paint_ops(dims, 1280.0, 720.0);
    // The round-3 QA explicitly checked pixels at the stated `out=`
    // positions. Each survivor's StrokeRect MUST be at the clip()'d
    // (x, y, w, h) — not in some preview band or legend swatch.
    let stroke_rects: Vec<(f64, f64, f64, f64)> = ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::StrokeRect(x, y, w, h) if *w > 20.0 && *h > 20.0 => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .collect();
    // clean → (60, 60, 200, 120)
    assert!(
        stroke_rects.contains(&(60.0, 60.0, 200.0, 120.0)),
        "BUG #6 regressed: `clean` rect not painted at (60,60,200×120). Found: {stroke_rects:?}"
    );
    // overflow-right clips x=700..1280 → (700, 60, 580, 120)
    assert!(
        stroke_rects.contains(&(700.0, 60.0, 580.0, 120.0)),
        "BUG #6 regressed: `overflow-right` rect not painted at (700,60,580×120). Found: {stroke_rects:?}"
    );
    // overflow-bottom clips y=540..720 → (60, 540, 200, 180)
    assert!(
        stroke_rects.contains(&(60.0, 540.0, 200.0, 180.0)),
        "BUG #6 regressed: `overflow-bottom` rect not painted at (60,540,200×180). Found: {stroke_rects:?}"
    );
}

#[test]
fn regression_bug6_canvas_footer_count_matches_rect_count() {
    use m1_canvas::CanvasDims;
    use wasm_demos::logic::canvas::{count_survivor_rects, paint_ops};
    let dims = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let ops = paint_ops(dims, 1280.0, 720.0);
    let rects = count_survivor_rects(&ops);
    let footer = ops
        .iter()
        .find_map(|op| match op {
            DrawOp::FillText(t, _, _) if t.starts_with("survived: ") => Some(t.clone()),
            _ => None,
        })
        .expect("paint_ops should emit a `survived: N …` footer");
    let n: usize = footer
        .trim_start_matches("survived: ")
        .split(' ')
        .next()
        .and_then(|s| s.parse().ok())
        .expect("footer should start with `survived: <int>`");
    assert_eq!(
        n, rects,
        "BUG #6 regressed: footer claims {n} survived but paint_ops drew {rects} outline rects"
    );
}

// ============================================================
// BUG #7 (gist round 2) — counter keyboard shortcuts unwired.
// ============================================================

#[test]
fn regression_bug7_key_plus_dispatches_increment() {
    use wasm_demos::logic::counter::key_to_msg;
    assert_eq!(key_to_msg("+"), Some(Msg::Increment));
    // `=` is the same physical key as `+` on a US keyboard without Shift —
    // hint says `+/- to step` so both shapes must work.
    assert_eq!(key_to_msg("="), Some(Msg::Increment));
}

#[test]
fn regression_bug7_key_minus_dispatches_decrement() {
    use wasm_demos::logic::counter::key_to_msg;
    assert_eq!(key_to_msg("-"), Some(Msg::Decrement));
    assert_eq!(key_to_msg("_"), Some(Msg::Decrement));
}

#[test]
fn regression_bug7_key_r_dispatches_reset() {
    use wasm_demos::logic::counter::key_to_msg;
    assert_eq!(key_to_msg("r"), Some(Msg::Reset));
    assert_eq!(key_to_msg("R"), Some(Msg::Reset));
}

#[test]
fn regression_bug7_unrelated_keys_do_not_dispatch() {
    use wasm_demos::logic::counter::key_to_msg;
    // Modifier keys arrive as named keys; mustn't map to a Msg.
    for k in ["a", "Shift", "Tab", "ArrowLeft", "Enter", "0"] {
        assert!(
            key_to_msg(k).is_none(),
            "key_to_msg({k:?}) should be None — only +/=, -/_, r/R map"
        );
    }
}

// Layout fix: count text centering for multi-char values (gist round 2).

#[test]
fn regression_bug7_reset_button_label_fits_inside_button() {
    use wasm_demos::logic::counter::{paint_ops as counter_paint_ops, BUTTONS};
    // Layout regression caught by headless QA round 2:
    // the previous hard-coded `bx + bw/2 - 12.0` centred for single-char
    // labels only, so the 5-char `reset` text spilled past the right edge
    // of its 200×80 button. Centring on actual char count keeps it inside.
    let ops = counter_paint_ops(init(), 1000.0, 480.0);
    let reset_button = BUTTONS.iter().find(|b| b.4 == "reset").unwrap();
    let (bx, _, bw, _, _) = *reset_button;
    let reset_x = ops
        .iter()
        .find_map(|op| match op {
            DrawOp::FillText(t, x, _) if t == "reset" => Some(*x),
            _ => None,
        })
        .expect("paint_ops should emit `reset` button label");
    // 5 chars × 28px = 140px wide. Must start ≥ bx and end ≤ bx+bw.
    let label_w = 5.0 * 28.0;
    assert!(
        reset_x >= bx,
        "reset label x={reset_x} starts before button left edge ({bx})"
    );
    assert!(
        reset_x + label_w <= bx + bw + 0.001,
        "reset label spans x={reset_x}..{} but button ends at {} — overflows right edge",
        reset_x + label_w,
        bx + bw
    );
}

#[test]
fn regression_bug7_negative_count_centers_on_string_width_not_single_char() {
    use wasm_demos::logic::counter::paint_ops as counter_paint_ops;
    // After 3× Decrement → count = -3, two chars. The previous demo
    // used a hard-coded `w/2 - 32` so the minus sat to the left of
    // center. The fix uses the full string width.
    let state = (0..3).fold(init(), |s, _| update(s, Msg::Decrement));
    assert_eq!(state.count, -3);
    let w = 1000.0;
    let ops = counter_paint_ops(state, w, 480.0);
    let big_x = ops
        .iter()
        .find_map(|op| match op {
            DrawOp::FillText(t, x, _) if t == "-3" => Some(*x),
            _ => None,
        })
        .expect("paint_ops should emit `-3` as the big count");
    // 128px monospace → ~64px char width. `-3` is 2 chars so the
    // centred x should be w/2 - 64 = 436, NOT w/2 - 32 = 468.
    let expected = w / 2.0 - 2.0 * 64.0 / 2.0;
    assert!(
        (big_x - expected).abs() < 1.0,
        "BUG #7 regressed: `-3` painted at x={big_x}, expected ~{expected} (centred on 2-char width)"
    );
}

// ============================================================
// BUG #8 (gist round 2) — process-table sparkline static.
// ============================================================

#[test]
fn regression_bug8_advance_history_mutates_every_row() {
    use wasm_demos::logic::process_table::{advance_history, fixture};
    let mut procs = fixture();
    let before: Vec<Vec<f64>> = procs.iter().map(|p| p.history.clone()).collect();
    let mut seed: u64 = 0x00C0_FFEE_BEEF;
    advance_history(&mut procs, &mut seed);
    let after: Vec<Vec<f64>> = procs.iter().map(|p| p.history.clone()).collect();
    assert_ne!(
        before, after,
        "BUG #8 regressed: advance_history is a no-op — sparkline can't tick"
    );
    // History length must stay constant — we drop the oldest + push one new.
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(
            b.len(),
            a.len(),
            "advance_history must keep history length constant"
        );
    }
}

#[test]
fn regression_bug8_advance_history_shifts_left() {
    use wasm_demos::logic::process_table::{advance_history, fixture};
    let mut procs = fixture();
    let p0_before = procs[0].history.clone();
    let mut seed: u64 = 0xDEAD_BEEF;
    advance_history(&mut procs, &mut seed);
    let p0_after = &procs[0].history;
    // After advance, what was history[1..] should now be history[..n-1].
    assert_eq!(
        &p0_before[1..],
        &p0_after[..p0_after.len() - 1],
        "BUG #8 regressed: advance_history didn't shift left (drop oldest, append new)"
    );
}

// ============================================================
// BUG #9 (gist round 2) — COMMAND sort case-sensitive.
// ============================================================

#[test]
fn regression_bug9_command_sort_is_case_insensitive() {
    use wasm_demos::logic::process_table::{fixture, sort_procs_dir, SortDir};
    let mut procs = fixture();
    sort_procs_dir(&mut procs, SortKey::Name, SortDir::Asc);
    let names: Vec<&str> = procs.iter().map(|p| p.name).collect();
    let bash_idx = names.iter().position(|&n| n == "bash").unwrap();
    let xorg_idx = names.iter().position(|&n| n == "Xorg").unwrap();
    assert!(
        bash_idx < xorg_idx,
        "BUG #9 regressed: case-sensitive sort put `Xorg` (0x58) before `bash` (0x62). \
         Got order: {names:?}"
    );
}

// ============================================================
// BUG #10 (gist round 2) — wasm-dash mem-bar width non-proportional.
// ============================================================

#[test]
fn regression_bug10_mem_bar_fill_width_scales_with_percentage() {
    let dash = Dashboard::fixture();
    let ops = dash_paint_ops(&dash, 1280.0, 720.0);
    // The mem-bar's data fill is a green FillRect with bounds
    // (x=0, y=h-40, w=fill_w, h=40). Find it.
    // It's the FIRST FillRect emitted right after a non-`#161b22`
    // Fill in the mem-bar arm of the match. Easier: it has h=40 and
    // x=0 (mem-bar's bounds.x) — locate the FillRect that *isn't*
    // the background and whose dimensions match.
    let mem_total = dash.mem_total_gb;
    let mem_used = dash.mem_used_gb;
    let expected_pct = mem_used / mem_total;
    let viewport_w = dash.viewport.w;
    let expected_fill_w = viewport_w * expected_pct;

    // Find a FillRect with height=40, y=720-40=680, width <= viewport.w.
    let mem_fill = ops.iter().find_map(|op| match op {
        DrawOp::FillRect(x, y, w, h)
            if (*h - 40.0).abs() < 0.001
                && (*y - 680.0).abs() < 0.001
                && (*x).abs() < 0.001
                && *w > 0.0
                && *w < viewport_w - 1.0 =>
        {
            Some(*w)
        }
        _ => None,
    });
    assert!(
        mem_fill.is_some(),
        "BUG #10 regressed: mem-bar paints full viewport width regardless of mem_pct. \
         Expected fill width near {expected_fill_w:.0} (= viewport.w {viewport_w} * \
         {expected_pct:.3}), found nothing under viewport.w."
    );
    let w = mem_fill.unwrap();
    assert!(
        (w - expected_fill_w).abs() < 1.0,
        "BUG #10 regressed: mem-bar fill width {w} doesn't match gauge {expected_fill_w:.1}"
    );
}

// ============================================================
// BUG #11 (gist round 2) — first CPU label rendered as `cpu-8`.
// ============================================================

#[test]
fn regression_bug11_first_cpu_label_is_zero_not_eight() {
    let dash = Dashboard::fixture();
    let ops = dash_paint_ops(&dash, 1280.0, 720.0);
    // BUG #11 was reported as `cpu-0` rendering visually as `cpu-8`.
    // Round-3 fix: rename label `cpu-N` → `CPU N` (space separator +
    // caps) and switch font to `ui-monospace` stack so the digit
    // glyphs are unambiguous regardless of OS default fallback.
    let first_cpu_label = ops
        .iter()
        .find_map(|op| match op {
            DrawOp::FillText(t, _, _) if t.starts_with("CPU ") => Some(t.clone()),
            _ => None,
        })
        .expect("paint_ops should emit at least one `CPU N` label");
    assert_eq!(
        first_cpu_label, "CPU 0",
        "BUG #11 regressed: first CPU label is {first_cpu_label}, expected `CPU 0`"
    );
    // No `CPU 8` label should ever appear (fixture has 8 cores: 0..=7).
    let cpu8 = ops
        .iter()
        .any(|op| matches!(op, DrawOp::FillText(t, _, _) if t == "CPU 8"));
    assert!(
        !cpu8,
        "BUG #11 regressed: paint_ops emits `CPU 8` (fixture only has cores 0..=7)"
    );
    // Also no raw `cpu-` form should leak — round-3 transformed them all.
    let raw_cpu = ops
        .iter()
        .any(|op| matches!(op, DrawOp::FillText(t, _, _) if t.starts_with("cpu-")));
    assert!(
        !raw_cpu,
        "BUG #11 regressed: paint_ops emits raw `cpu-N` form. \
         Round-3 fix should transform to `CPU N` for unambiguous glyph rendering."
    );
}

#[test]
fn regression_bug11_uses_ui_monospace_font_stack() {
    // Round-3 layout fix: switch from `13px monospace` to a font stack
    // that names `ui-monospace` (and fallbacks) so the OS picks a
    // font whose digit `0` doesn't blend with the hyphen-minus into
    // an `8`-shaped cluster at small sizes.
    let dash = Dashboard::fixture();
    let ops = dash_paint_ops(&dash, 1280.0, 720.0);
    let uses_ui_mono = ops
        .iter()
        .any(|op| matches!(op, DrawOp::Font(f) if f.contains("ui-monospace")));
    assert!(
        uses_ui_mono,
        "BUG #11 regressed: wasm-dash paint_ops dropped the `ui-monospace` font stack — \
         OS-fallback fonts may render `cpu-0` ambiguously."
    );
}

// ============================================================
// BUG #12 (gist round 2) — shell Ctrl-modified keys typed literally.
// ============================================================

#[test]
fn regression_bug12_modifier_keys_are_ignored() {
    use wasm_demos::logic::shell::should_ignore_modifier_key;
    // Plain key: not ignored.
    assert!(!should_ignore_modifier_key(false, false));
    // Ctrl set: ignored (e.g. Ctrl+A).
    assert!(should_ignore_modifier_key(true, false));
    // Meta/Cmd set: ignored (e.g. Cmd+R).
    assert!(should_ignore_modifier_key(false, true));
    // Both: ignored.
    assert!(should_ignore_modifier_key(true, true));
}

// ============================================================
// BUG #13 (gist round 2) — shell suggestions stale on diverged input.
// ============================================================

#[test]
fn regression_bug13_corrupted_input_returns_empty_not_stale_suggestions() {
    use wasm_demos::logic::shell::lookup;
    // Pre-fix: `madocker ps` matched dict prefix `m` and returned
    // the make-* suggestions even though no make command is being
    // typed. Fix filters to suggestions of which the full input
    // is a prefix — `madocker ps` is a prefix of nothing.
    let sugs = lookup("madocker ps");
    assert!(
        sugs.is_empty(),
        "BUG #13 regressed: lookup(`madocker ps`) returned {sugs:?} — should be empty"
    );
}

#[test]
fn regression_bug13_partial_prefix_still_returns_mid_typed_suggestions() {
    use wasm_demos::logic::shell::lookup;
    // `ma` is mid-typing `make demo/serve/wasm` — should still match.
    let sugs = lookup("ma");
    assert_eq!(sugs.len(), 3);
    for s in &sugs {
        assert!(
            s.starts_with("ma"),
            "lookup(`ma`) returned {s:?} which doesn't start with `ma`"
        );
    }
}

#[test]
fn regression_bug13_input_diverging_from_suggestions_empties_panel() {
    use wasm_demos::logic::shell::lookup;
    // `make zzz` starts with `make ` so dict prefix matches, but no
    // make-* suggestion starts with `make zzz` → panel should empty.
    let sugs = lookup("make zzz");
    assert!(
        sugs.is_empty(),
        "BUG #13 regressed: lookup(`make zzz`) returned {sugs:?} — input diverges from all suggestions"
    );
}
