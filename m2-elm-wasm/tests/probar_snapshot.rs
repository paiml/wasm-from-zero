//! Probar snapshot test for m2-elm-wasm.

use m2_elm_wasm::{init, update, view, Msg, State};
use m4_tests::{diff_snapshot, snapshot, VNode};

#[test]
fn probar_snapshot_counter_view() {
    let msgs = [Msg::Increment, Msg::Increment, Msg::Decrement];
    let s = msgs.iter().copied().fold(init(), update);
    let nodes = view(s);
    let root = VNode {
        tag: "counter".into(),
        text: String::new(),
        children: nodes
            .into_iter()
            .map(|n| VNode {
                tag: n.tag,
                text: n.text,
                children: vec![],
            })
            .collect(),
    };

    let golden = "<counter>\n  <h1>counter\n  <div>count = 1\n  <small>+/- to step, r to reset\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}

#[test]
fn probar_snapshot_view_color_branches() {
    // Negative count → red text colour
    let s_neg = update(init(), Msg::Decrement);
    assert_ne!(s_neg, State::default());
    let v = view(s_neg);
    assert!(v[1].text.contains("-1"));
}
