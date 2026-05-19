#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — canvas demo: m1-canvas::clip survives 8 adversarial inputs.

use m1_canvas::CanvasDims;
use wasm_demos::logic::canvas::{adversarial_plans, clip_results};

#[test]
fn probar_canvas_8_plans_match_promised_survival() {
    let canvas = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let results = clip_results(canvas);
    assert_eq!(results.len(), 8);
    // Each adversarial input maps deterministically to Some|None:
    let expected: &[(&str, bool)] = &[
        ("clean", true),
        ("overflow-right", true),  // partial → clipped to canvas
        ("overflow-bottom", true), // partial → clipped
        ("negative-w", false),
        ("nan-x", false),
        ("inf-y", false),
        ("off-screen", false),
        ("zero-w", false),
    ];
    for ((label, got), (exp_label, exp_survives)) in results.iter().zip(expected.iter()) {
        assert_eq!(label, exp_label, "ordering drift");
        assert_eq!(
            got.is_some(),
            *exp_survives,
            "plan {label}: expected survival={exp_survives}, got Some={}",
            got.is_some()
        );
    }
}

#[test]
fn probar_canvas_clean_input_preserved_exactly() {
    let canvas = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let results = clip_results(canvas);
    let clean = results
        .iter()
        .find(|(l, _)| *l == "clean")
        .unwrap()
        .1
        .unwrap();
    let plan = adversarial_plans()
        .into_iter()
        .find(|p| p.label == "clean")
        .unwrap()
        .rect;
    assert_eq!(clean.x, plan.x, "clean input x preserved");
    assert_eq!(clean.y, plan.y, "clean input y preserved");
    assert_eq!(clean.w, plan.w, "clean input w preserved");
    assert_eq!(clean.h, plan.h, "clean input h preserved");
}

#[test]
fn probar_canvas_overflow_clipped_inside_bounds() {
    let canvas = CanvasDims {
        width: 1280.0,
        height: 720.0,
    };
    let results = clip_results(canvas);
    let overflow_right = results
        .iter()
        .find(|(l, _)| *l == "overflow-right")
        .unwrap()
        .1
        .unwrap();
    assert!(
        overflow_right.x + overflow_right.w <= canvas.width + 0.001,
        "overflow-right not clipped: {:.1} + {:.1} > {:.1}",
        overflow_right.x,
        overflow_right.w,
        canvas.width
    );
}
