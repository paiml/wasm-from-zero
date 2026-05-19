#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — process-table demo: sort_procs is deterministic + monotone;
//! fixture has 12 procs; header_hit returns the right column.

use wasm_demos::logic::process_table::{
    fixture, header_hit, sort_procs, SortKey, COL_CPU, COL_MEM, COL_PID, HEADER_Y,
};

#[test]
fn probar_table_fixture_size() {
    let procs = fixture();
    assert_eq!(procs.len(), 12, "fixture has 12 processes");
    // History length is fixed (the spark column expects exactly 32)
    for p in &procs {
        assert_eq!(p.history.len(), 32, "{} history length", p.name);
    }
}

#[test]
fn probar_table_sort_by_cpu_is_descending() {
    let mut procs = fixture();
    sort_procs(&mut procs, SortKey::Cpu);
    for w in procs.windows(2) {
        assert!(
            w[0].cpu >= w[1].cpu,
            "CPU sort not descending: {} > {}",
            w[1].cpu,
            w[0].cpu
        );
    }
    // Top row should be rustc (0.81)
    assert_eq!(procs[0].name, "rustc");
}

#[test]
fn probar_table_sort_by_mem_is_descending() {
    let mut procs = fixture();
    sort_procs(&mut procs, SortKey::Mem);
    for w in procs.windows(2) {
        assert!(w[0].mem >= w[1].mem, "MEM sort not descending");
    }
    // Top row should be firefox (0.72)
    assert_eq!(procs[0].name, "firefox");
}

#[test]
fn probar_table_sort_by_pid_is_ascending() {
    let mut procs = fixture();
    sort_procs(&mut procs, SortKey::Pid);
    for w in procs.windows(2) {
        assert!(w[0].pid <= w[1].pid, "PID sort not ascending");
    }
}

#[test]
fn probar_table_header_hit_centers() {
    // Click center of each header
    let pid_center = (COL_PID.0 + COL_PID.1 / 2.0, HEADER_Y - 10.0);
    let cpu_center = (COL_CPU.0 + COL_CPU.1 / 2.0, HEADER_Y - 10.0);
    let mem_center = (COL_MEM.0 + COL_MEM.1 / 2.0, HEADER_Y - 10.0);
    assert_eq!(header_hit(pid_center.0, pid_center.1), Some(SortKey::Pid));
    assert_eq!(header_hit(cpu_center.0, cpu_center.1), Some(SortKey::Cpu));
    assert_eq!(header_hit(mem_center.0, mem_center.1), Some(SortKey::Mem));
}

#[test]
fn probar_table_header_hit_misses() {
    assert_eq!(header_hit(0.0, 0.0), None, "off-canvas");
    assert_eq!(header_hit(400.0, 200.0), None, "below header band");
    // BUG #4 fix added SortKey::Name spanning x=160..360, so the old gap
    // assertion at x=300 is no longer a miss. The new gap lives between
    // the end of MEM (x=560) and start of HISTORY (x=580).
    assert_eq!(
        header_hit(565.0, 80.0),
        None,
        "between MEM + HISTORY columns"
    );
}

#[test]
fn probar_table_sort_is_idempotent() {
    let mut procs = fixture();
    sort_procs(&mut procs, SortKey::Cpu);
    let snap = procs.clone();
    sort_procs(&mut procs, SortKey::Cpu);
    assert_eq!(procs, snap, "sorting twice gives the same result");
}
