#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — showcase demo: deterministic ticking + softmax invariants.

use wasm_demos::logic::showcase::{softmax, State};

#[test]
fn probar_showcase_softmax_sums_to_one() {
    // Property: softmax output sums to 1.0 (±ε) for any finite input.
    for input in [
        vec![0.0; 16],
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![-100.0, 0.0, 100.0],
        (0..16).map(|i| (i as f64).sin()).collect::<Vec<_>>(),
    ] {
        let out = softmax(&input);
        let sum: f64 = out.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "softmax({:?}) sum = {} ≠ 1.0",
            input,
            sum
        );
        for p in out.iter() {
            assert!(*p >= 0.0 && *p <= 1.0, "softmax output out of [0,1]: {p}");
        }
    }
}

#[test]
fn probar_showcase_tick_increments_frame() {
    let mut s = State::new();
    let frame_0 = s.frame;
    s.tick();
    assert_eq!(s.frame, frame_0 + 1);
    for _ in 0..99 {
        s.tick();
    }
    assert_eq!(s.frame, frame_0 + 100);
}

#[test]
fn probar_showcase_lcg_is_deterministic() {
    let mut a = State::new();
    let mut b = State::new();
    // Two independently-seeded states with the same seed → same lcg() stream
    for i in 0..50 {
        assert_eq!(a.lcg(), b.lcg(), "LCG diverged at step {i}");
    }
}

#[test]
fn probar_showcase_tick_bounded_after_100_frames() {
    // After 100 ticks, all bars ∈ [0,1] (clamp is by-construction in tween).
    let mut s = State::new();
    for _ in 0..100 {
        s.tick();
    }
    for (i, &b) in s.bars.iter().enumerate() {
        assert!(b >= 0.0 && b <= 1.0, "bar[{i}] = {b} out of [0,1]");
    }
    assert!(s.particles.len() <= 50, "particle cap of 50 violated");
}

#[test]
fn probar_showcase_softmax_input_evolves() {
    let mut s = State::new();
    let in_before = s.softmax_in;
    s.tick();
    let in_after = s.softmax_in;
    assert_ne!(in_before, in_after, "softmax_in should change after tick()");
}
