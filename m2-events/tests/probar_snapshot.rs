//! Probar snapshot test for m2-events.

use m2_events::{dispatch, AppMsg, BrowserEvent};
use m4_tests::{diff_snapshot, snapshot, VNode};

fn entry(label: &str, ev: BrowserEvent) -> VNode {
    let msg = dispatch(&ev);
    VNode {
        tag: label.into(),
        text: format!("{msg:?}"),
        children: vec![],
    }
}

#[test]
fn probar_snapshot_dispatch_table() {
    let root = VNode {
        tag: "dispatch".into(),
        text: String::new(),
        children: vec![
            entry("click-inc", BrowserEvent::Click { id: "inc".into() }),
            entry("click-dec", BrowserEvent::Click { id: "dec".into() }),
            entry("click-noop", BrowserEvent::Click { id: "noop".into() }),
            entry(
                "key-arrowup",
                BrowserEvent::KeyPress {
                    key: "ArrowUp".into(),
                    ctrl: false,
                },
            ),
            entry(
                "ctrl-c",
                BrowserEvent::KeyPress {
                    key: "c".into(),
                    ctrl: true,
                },
            ),
            entry(
                "resize",
                BrowserEvent::Resize {
                    width: 1920,
                    height: 1080,
                },
            ),
            entry("beforeunload", BrowserEvent::BeforeUnload),
        ],
    };
    let golden = "<dispatch>\n  <click-inc>Some(Bump(1))\n  <click-dec>Some(Bump(-1))\n  <click-noop>None\n  <key-arrowup>Some(Bump(1))\n  <ctrl-c>Some(Quit)\n  <resize>Some(Resized(1920, 1080))\n  <beforeunload>Some(Quit)\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
    // assert: returned variants match
    assert_eq!(
        dispatch(&BrowserEvent::Click { id: "inc".into() }),
        Some(AppMsg::Bump(1))
    );
}
