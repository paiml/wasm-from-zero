#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — wasm-dash demo: jitter() stays bounded; deterministic LCG.

use m5_dash::Dashboard;
use wasm_demos::logic::wasm_dash::{jitter, lcg};

#[test]
fn probar_wasm_dash_jitter_keeps_cpus_in_bounds() {
    let mut dash = Dashboard::fixture();
    let mut seed = 0xC0FFEE_u64;
    for tick in 0..1000 {
        jitter(&mut dash, &mut seed);
        for (i, &v) in dash.cpu_load.iter().enumerate() {
            assert!(
                (0.05..=0.99).contains(&v),
                "tick {tick}: cpu[{i}] = {v:.3} out of [0.05, 0.99]"
            );
        }
    }
}

#[test]
fn probar_wasm_dash_jitter_keeps_mem_in_bounds() {
    let mut dash = Dashboard::fixture();
    let total = dash.mem_total_gb;
    let mut seed = 0xC0FFEE_u64;
    for tick in 0..1000 {
        jitter(&mut dash, &mut seed);
        assert!(
            dash.mem_used_gb >= 0.5 && dash.mem_used_gb <= total - 0.5,
            "tick {tick}: mem = {:.3} out of [0.5, {:.3}]",
            dash.mem_used_gb,
            total - 0.5
        );
    }
}

#[test]
fn probar_wasm_dash_lcg_deterministic() {
    let mut s1 = 0xDEAD_BEEF_u64;
    let mut s2 = 0xDEAD_BEEF_u64;
    for _ in 0..100 {
        assert_eq!(lcg(&mut s1), lcg(&mut s2));
    }
}

#[test]
fn probar_wasm_dash_event_count_increments() {
    let mut dash = Dashboard::fixture();
    let mut seed = 0xC0FFEE_u64;
    let before = dash.event_count;
    jitter(&mut dash, &mut seed);
    assert_eq!(dash.event_count, before + 1);
    for _ in 0..10 {
        jitter(&mut dash, &mut seed);
    }
    assert_eq!(dash.event_count, before + 11);
}

#[test]
fn probar_wasm_dash_no_panic_on_extreme_seed() {
    // Adversarial seeds shouldn't crash jitter.
    for seed_val in [0_u64, 1, u64::MAX, u64::MAX - 1, 0x5555_5555_5555_5555] {
        let mut dash = Dashboard::fixture();
        let mut s = seed_val;
        for _ in 0..100 {
            jitter(&mut dash, &mut s);
        }
    }
}
