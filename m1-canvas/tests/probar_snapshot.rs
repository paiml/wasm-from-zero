//! Probar snapshot test for m1-canvas.

use m1_canvas::{clip, CanvasDims, Color, DrawRect};
use m4_tests::{diff_snapshot, snapshot, VNode};

fn rect_to_vdom(label: &str, opt: Option<DrawRect>) -> VNode {
    match opt {
        Some(r) => VNode {
            tag: label.into(),
            text: format!(
                "x={:.0} y={:.0} w={:.0} h={:.0} rgba=({:.2},{:.2},{:.2},{:.2})",
                r.x, r.y, r.w, r.h, r.color.r, r.color.g, r.color.b, r.color.a
            ),
            children: vec![],
        },
        None => VNode {
            tag: label.into(),
            text: "dropped".into(),
            children: vec![],
        },
    }
}

#[test]
fn probar_snapshot_clip_table() {
    let canvas = CanvasDims {
        width: 800.0,
        height: 600.0,
    };
    let root = VNode {
        tag: "clip-table".into(),
        text: String::new(),
        children: vec![
            rect_to_vdom(
                "in-bounds",
                clip(
                    DrawRect {
                        x: 100.0,
                        y: 100.0,
                        w: 50.0,
                        h: 50.0,
                        color: Color::RED,
                    },
                    canvas,
                ),
            ),
            rect_to_vdom(
                "nan",
                clip(
                    DrawRect {
                        x: f64::NAN,
                        y: 0.0,
                        w: 10.0,
                        h: 10.0,
                        color: Color::RED,
                    },
                    canvas,
                ),
            ),
            rect_to_vdom(
                "clamp-color",
                clip(
                    DrawRect {
                        x: 0.0,
                        y: 0.0,
                        w: 100.0,
                        h: 100.0,
                        color: Color::new(2.0, -1.0, 0.5, 1.5),
                    },
                    canvas,
                ),
            ),
        ],
    };

    let golden = "<clip-table>\n  <in-bounds>x=100 y=100 w=50 h=50 rgba=(1.00,0.00,0.00,1.00)\n  <nan>dropped\n  <clamp-color>x=0 y=0 w=100 h=100 rgba=(1.00,0.00,0.50,1.00)\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}
