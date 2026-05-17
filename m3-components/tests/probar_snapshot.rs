//! Probar snapshot test for m3-components.

use m3_components::{layout, Container, Direction, Rect};
use m4_tests::{diff_snapshot, snapshot, VNode};
use presentar_core::Color;

#[test]
fn probar_snapshot_row_layout() {
    let header = Container {
        direction: Direction::Row,
        children: vec![
            ("nav".into(), Color::BLUE),
            ("title".into(), Color::WHITE),
            ("avatar".into(), Color::GREEN),
        ],
    };
    let parent = Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 60.0,
    };
    let nodes = layout(&header, parent);
    let root = VNode {
        tag: "row".into(),
        text: String::new(),
        children: nodes
            .into_iter()
            .map(|n| VNode {
                tag: n.tag,
                text: format!(
                    "({:.0}, {:.0}, {:.0}, {:.0})",
                    n.bounds.x, n.bounds.y, n.bounds.w, n.bounds.h
                ),
                children: vec![],
            })
            .collect(),
    };
    let golden =
        "<row>\n  <nav>(0, 0, 400, 60)\n  <title>(400, 0, 400, 60)\n  <avatar>(800, 0, 400, 60)\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}
