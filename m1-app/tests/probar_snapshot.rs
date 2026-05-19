//! Probar snapshot test for m1-app.

use m1_app::{App, HostDocument, MountError};
use m4_tests::{diff_snapshot, snapshot, VNode};

#[test]
fn probar_snapshot_mount_lifecycle() {
    let host = HostDocument::default().with_target("app");
    let mount_ok = App::mount(&host, "app").map(|_| "ok".to_string());
    let mount_err = App::mount(&host, "missing").map(|_| "ok".to_string());

    let root = VNode {
        tag: "lifecycle".into(),
        text: String::new(),
        children: vec![
            VNode {
                tag: "mount-ok".into(),
                text: mount_ok.unwrap_or_else(|e| format!("err: {e}")),
                children: vec![],
            },
            VNode {
                tag: "mount-missing".into(),
                text: match mount_err {
                    Ok(_) => "unexpected-ok".into(),
                    Err(MountError::TargetMissing(name)) => format!("err: missing '{name}'"),
                    Err(e) => format!("err: {e}"),
                },
                children: vec![],
            },
        ],
    };
    let golden = "<lifecycle>\n  <mount-ok>ok\n  <mount-missing>err: missing 'missing'\n";
    let mismatches = diff_snapshot(&root, golden);
    assert!(
        mismatches.is_empty(),
        "probar snapshot diverged:\n actual:\n{}\nmismatches: {:?}",
        snapshot(&root),
        mismatches
    );
}
