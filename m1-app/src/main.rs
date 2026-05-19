#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m1_app::{contract_marker, App, HostDocument, MountError};

fn main() {
    let host = HostDocument::default()
        .with_target("app")
        .with_target("sidebar");
    println!("M1.2 · App lifecycle — mount/unmount via aprender-present-lib::browser::App");
    println!("       (modelled on the host so the totality contract is unit-testable)\n");
    for id in ["app", "missing"] {
        match App::mount(&host, id) {
            Ok(app) => println!(
                "  mount('{id}') -> Ok (theme rgba {:.2},{:.2},{:.2},{:.2})",
                app.theme_color.r, app.theme_color.g, app.theme_color.b, app.theme_color.a
            ),
            Err(MountError::TargetMissing(name)) => {
                println!("  mount('{id}') -> Err: target '{name}' not in document")
            }
            Err(e) => println!("  mount('{id}') -> Err: {e}"),
        }
    }
    eprintln!("{}", contract_marker());
}
