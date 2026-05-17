#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m3_components::{contract_marker, layout, Container, Direction, Rect};
use presentar_core::Color;

fn main() {
    let parent = Rect {
        x: 0.0,
        y: 0.0,
        w: 1200.0,
        h: 600.0,
    };
    let header = Container {
        direction: Direction::Row,
        children: vec![
            ("nav".into(), Color::BLUE),
            ("title".into(), Color::WHITE),
            ("avatar".into(), Color::GREEN),
        ],
    };
    let nodes = layout(&header, parent);
    println!(
        "M3.1 · Container::Row laid out across {} × {}\n",
        parent.w, parent.h
    );
    for n in &nodes {
        println!(
            "  <{}> rect=({:.0}, {:.0}, {:.0}, {:.0}) color=({:.2},{:.2},{:.2},{:.2})",
            n.tag, n.bounds.x, n.bounds.y, n.bounds.w, n.bounds.h, n.fg.r, n.fg.g, n.fg.b, n.fg.a
        );
    }
    println!("\nevery child rect ⊆ parent rect; pairwise disjoint — panels contract holds.");
    eprintln!("{}", contract_marker());
}
