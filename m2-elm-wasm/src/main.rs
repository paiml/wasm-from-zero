#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m2_elm_wasm::{contract_marker, init, update, view, Msg};

fn main() {
    let msgs = [
        Msg::Increment,
        Msg::Increment,
        Msg::Decrement,
        Msg::Increment,
    ];
    let state = msgs.iter().copied().fold(init(), update);
    let again = msgs.iter().copied().fold(init(), update);
    assert_eq!(state, again, "non-deterministic replay");
    println!("M2.1 · Elm-style WASM counter (state machine + VDom)\n");
    println!("messages: {msgs:?}");
    println!("count    = {}", state.count);
    println!("VDom:");
    for node in view(state) {
        println!(
            "  <{}> {} (rgba {:.2},{:.2},{:.2},{:.2})",
            node.tag, node.text, node.fg.r, node.fg.g, node.fg.b, node.fg.a
        );
    }
    println!("\n[determinism] same input → same VDom (replay × 2 verified)");
    eprintln!("{}", contract_marker());
}
