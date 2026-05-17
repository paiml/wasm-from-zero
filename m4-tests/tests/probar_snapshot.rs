//! Probar snapshot test for m4-tests — the snapshot harness itself.

use m4_tests::{diff_snapshot, snapshot, VNode};

#[test]
fn probar_snapshot_self_roundtrip() {
    let tree = VNode {
        tag: "main".into(),
        text: String::new(),
        children: vec![
            VNode {
                tag: "h1".into(),
                text: "title".into(),
                children: vec![],
            },
            VNode {
                tag: "p".into(),
                text: "body".into(),
                children: vec![],
            },
        ],
    };
    let s = snapshot(&tree);
    assert!(
        diff_snapshot(&tree, &s).is_empty(),
        "self-diff must be empty"
    );
    assert_eq!(s, "<main>\n  <h1>title\n  <p>body\n");
}
