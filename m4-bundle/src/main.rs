#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m4_bundle::{build_pipeline, contract_marker};

fn main() {
    println!("M4.1 · WASM build pipeline (Rust → wasm32 → bindgen → bundle → deploy)\n");
    for (i, step) in build_pipeline().iter().enumerate() {
        println!("  step {} · {}", i + 1, step.name);
        println!("    $ {}", step.command);
        println!("    -> {}\n", step.artifact);
    }
    eprintln!("{}", contract_marker());
}
