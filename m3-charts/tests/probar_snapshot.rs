//! Probar snapshot test for m3-charts.

use m3_charts::{gauge_fraction, position};
use m4_tests::{diff_snapshot, snapshot, VNode};

#[test]
fn probar_snapshot_chart_math() {
    let root = VNode {
        tag: "charts".into(),
        text: String::new(),
        children: vec![
            VNode {
                tag: "gauge-half".into(),
                text: format!("{:.3}", gauge_fraction(50.0, 100.0)),
                children: vec![],
            },
            VNode {
                tag: "gauge-overflow".into(),
                text: format!("{:.3}", gauge_fraction(200.0, 100.0)),
                children: vec![],
            },
            VNode {
                tag: "gauge-nan".into(),
                text: format!("{:.3}", gauge_fraction(f64::NAN, 100.0)),
                children: vec![],
            },
            VNode {
                tag: "pos-0".into(),
                text: format!("{:.1}", position(0.0, 0.0, 100.0, 0.0, 800.0)),
                children: vec![],
            },
            VNode {
                tag: "pos-50".into(),
                text: format!("{:.1}", position(50.0, 0.0, 100.0, 0.0, 800.0)),
                children: vec![],
            },
            VNode {
                tag: "pos-100".into(),
                text: format!("{:.1}", position(100.0, 0.0, 100.0, 0.0, 800.0)),
                children: vec![],
            },
            VNode {
                tag: "pos-overflow".into(),
                text: format!("{:.1}", position(1e30, 0.0, 100.0, 0.0, 800.0)),
                children: vec![],
            },
        ],
    };
    let golden = "<charts>\n  <gauge-half>0.500\n  <gauge-overflow>1.000\n  <gauge-nan>0.000\n  <pos-0>0.0\n  <pos-50>400.0\n  <pos-100>800.0\n  <pos-overflow>800.0\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}
