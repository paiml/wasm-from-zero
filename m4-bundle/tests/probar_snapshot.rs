//! Probar snapshot test for m4-bundle.

use m4_bundle::build_pipeline;
use m4_tests::{diff_snapshot, snapshot, VNode};

#[test]
fn probar_snapshot_pipeline_steps() {
    let pipeline = build_pipeline();
    let root = VNode {
        tag: "pipeline".into(),
        text: String::new(),
        children: pipeline
            .iter()
            .map(|s| VNode {
                tag: s.name.to_string(),
                text: format!("$ {}", s.command),
                children: vec![],
            })
            .collect(),
    };
    let golden = "<pipeline>\n  <compile rust → wasm>$ cargo build --release --target wasm32-unknown-unknown\n  <generate JS bindings>$ wasm-bindgen <wasm> --out-dir pkg --target web\n  <optimize wasm size>$ wasm-opt -Oz <wasm> -o <wasm>\n  <bundle for browser>$ presentar-cli serve --pkg pkg --out dist\n  <deploy>$ rsync -avz dist/ <host>:/var/www/<crate>/\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}
