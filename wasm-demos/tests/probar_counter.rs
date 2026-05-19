#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — counter demo: hit_test() returns the right Msg per pixel;
//! init/update/view is referentially transparent (m2-elm-wasm already
//! tests that; here we test the hit grid is correct).

use m2_elm_wasm::{init, update, Msg, State};
use wasm_demos::logic::counter::{hit_test, BUTTONS};

#[test]
fn probar_counter_buttons_distinct_msgs() {
    // Click center of each button → expected Msg
    let cases = [
        (140.0, 360.0, Some(Msg::Decrement)), // [-] center
        (400.0, 360.0, Some(Msg::Increment)), // [+] center
        (700.0, 360.0, Some(Msg::Reset)),     // [reset] center
    ];
    for (x, y, expected) in cases {
        assert_eq!(hit_test(x, y), expected, "hit_test({x}, {y})");
    }
}

#[test]
fn probar_counter_buttons_do_not_overlap() {
    // Any point in button N should NOT be in button M for M ≠ N.
    for (i, (bx, by, bw, bh, _)) in BUTTONS.iter().enumerate() {
        let cx = bx + bw / 2.0;
        let cy = by + bh / 2.0;
        let msg_i = hit_test(cx, cy);
        for (j, _) in BUTTONS.iter().enumerate() {
            if i == j {
                continue;
            }
            // (cx, cy) is inside button i — should not also be inside button j
            let (jx, jy, jw, jh, _) = BUTTONS[j];
            let in_j = cx >= jx && cx <= jx + jw && cy >= jy && cy <= jy + jh;
            assert!(!in_j, "buttons {i} and {j} overlap at ({cx}, {cy})");
        }
        assert!(msg_i.is_some(), "center of button {i} should hit");
    }
}

#[test]
fn probar_counter_outside_returns_none() {
    assert_eq!(hit_test(0.0, 0.0), None);
    assert_eq!(hit_test(1000.0, 100.0), None);
    assert_eq!(hit_test(400.0, 100.0), None, "above buttons");
    assert_eq!(hit_test(400.0, 500.0), None, "below buttons");
}

#[test]
fn probar_counter_fold_determinism() {
    // Snapshot: same Msg sequence → same final state, always.
    let msgs = [
        Msg::Increment,
        Msg::Increment,
        Msg::Increment,
        Msg::Decrement,
        Msg::Reset,
        Msg::Increment,
        Msg::Increment,
    ];
    let final_a = msgs.iter().fold(init(), |s, &m| update(s, m));
    let final_b = msgs.iter().fold(init(), |s, &m| update(s, m));
    assert_eq!(
        final_a, final_b,
        "fold(msgs, init, update) is deterministic"
    );
    assert_eq!(final_a, State { count: 2 });
}
