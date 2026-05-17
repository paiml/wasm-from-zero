//! Probar snapshot test for m5-dash — the capstone.

use m4_tests::{diff_snapshot, snapshot, VNode};
use m5_dash::{build_paint_list, Dashboard};

#[test]
fn probar_snapshot_dashboard_paint_count() {
    let dash = Dashboard::fixture();
    let cmds = build_paint_list(&dash);

    // sanity checks: every cmd is inside the viewport
    for c in &cmds {
        assert!(c.bounds.x >= dash.viewport.x);
        assert!(c.bounds.y >= dash.viewport.y);
        assert!(c.bounds.x + c.bounds.w <= dash.viewport.x + dash.viewport.w + 0.001);
        assert!(c.bounds.y + c.bounds.h <= dash.viewport.y + dash.viewport.h + 0.001);
    }

    // determinism via snapshot diff
    let render = |cmds: &[m5_dash::PaintCmd]| VNode {
        tag: "dashboard".into(),
        text: String::new(),
        children: cmds
            .iter()
            .map(|c| VNode {
                tag: c.label.clone(),
                text: format!("fill={:.2}", c.fill),
                children: vec![],
            })
            .collect(),
    };
    let a = render(&cmds);
    let b = render(&build_paint_list(&dash));
    let golden = snapshot(&a);
    assert!(
        diff_snapshot(&b, &golden).is_empty(),
        "fixture render is non-deterministic"
    );
}
