#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m4_tests::{contract_marker, diff_snapshot, snapshot, VNode};

fn build_tree() -> VNode {
    VNode {
        tag: "main".into(),
        text: String::new(),
        children: vec![
            VNode {
                tag: "h1".into(),
                text: "wasm-from-zero".into(),
                children: vec![],
            },
            VNode {
                tag: "p".into(),
                text: "snapshot harness — probar pattern".into(),
                children: vec![],
            },
        ],
    }
}

const GOLDEN: &str = "<main>\n  <h1>wasm-from-zero\n  <p>snapshot harness — probar pattern\n";

fn main() {
    let tree = build_tree();
    println!("M4.2 · VDOM snapshot harness (probar-style)\n");
    println!("snapshot:\n{}", snapshot(&tree));
    let mismatches = diff_snapshot(&tree, GOLDEN);
    println!("diff vs golden: {} mismatch(es)", mismatches.len());
    if mismatches.is_empty() {
        println!("✓ snapshot matches golden — TUI/VDOM tests without a browser.");
    } else {
        println!("✘ mismatches: {mismatches:?}");
    }
    eprintln!("{}", contract_marker());
}
