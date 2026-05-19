#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m3_charts::{contract_marker, gauge_fraction, position};

fn main() {
    println!("M3.2 · chart math (gauge + line-chart cursor)\n");
    println!(
        "gauge: value=72.5  / max=100  -> fill = {:.3}",
        gauge_fraction(72.5, 100.0)
    );
    println!(
        "gauge: value=-10.0 / max=100  -> fill = {:.3} (clamped)",
        gauge_fraction(-10.0, 100.0)
    );
    println!(
        "gauge: value=NaN   / max=100  -> fill = {:.3} (NaN-safe)",
        gauge_fraction(f64::NAN, 100.0)
    );
    println!();
    println!("line chart 0..100 mapped to pixels 0..800:");
    for v in [0.0, 25.0, 50.0, 75.0, 100.0, 1e30, f64::NAN] {
        let px = position(v, 0.0, 100.0, 0.0, 800.0);
        println!("  v = {v:>5}  -> x = {px:>6.1} px  (cursor stays in [0, 800])");
    }
    eprintln!("{}", contract_marker());
}
