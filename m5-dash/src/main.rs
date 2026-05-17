#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m4_tests::{snapshot, VNode};
use m5_dash::{build_paint_list, contract_marker, Dashboard};

fn main() {
    let dash = Dashboard::fixture();
    let cmds = build_paint_list(&dash);
    println!("M5 · capstone dashboard — composes every prior module");
    println!("title:    {}", dash.title);
    println!("viewport: {}×{}", dash.viewport.w, dash.viewport.h);
    println!("cores:    {}", dash.cpu_load.len());
    println!(
        "mem:      {:.1} / {:.1} GB",
        dash.mem_used_gb, dash.mem_total_gb
    );
    println!("events:   {}", dash.event_count);
    println!();
    println!("paint commands ({}):", cmds.len());
    for c in &cmds {
        println!(
            "  {:<14} bounds=({:>5.0}, {:>5.0}, {:>5.0}, {:>5.0}) fill={:.2} fg=({:.2},{:.2},{:.2},{:.2})",
            c.label, c.bounds.x, c.bounds.y, c.bounds.w, c.bounds.h, c.fill,
            c.fg.r, c.fg.g, c.fg.b, c.fg.a,
        );
    }
    // VDom snapshot via m4-tests (probar harness in action)
    let vdom = VNode {
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
    println!("\nVDom snapshot (probar pattern):\n{}", snapshot(&vdom));
    eprintln!("{}", contract_marker());
}
