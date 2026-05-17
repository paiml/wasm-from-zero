#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
//! M1.1 demo: clip a few draw plans against a 800×600 canvas, including
//! NaN, Inf, and overhanging rects. Prints what survives + the contract
//! marker.

use m1_canvas::{clip, contract_marker, CanvasDims, Color, DrawRect};

fn main() {
    let canvas = CanvasDims {
        width: 800.0,
        height: 600.0,
    };
    let probes = [
        (
            "in-bounds rect",
            DrawRect {
                x: 100.0,
                y: 100.0,
                w: 50.0,
                h: 50.0,
                color: Color::RED,
            },
        ),
        (
            "overhang right/bottom",
            DrawRect {
                x: 750.0,
                y: 550.0,
                w: 200.0,
                h: 200.0,
                color: Color::BLUE,
            },
        ),
        (
            "NaN coord",
            DrawRect {
                x: f64::NAN,
                y: 0.0,
                w: 10.0,
                h: 10.0,
                color: Color::GREEN,
            },
        ),
        (
            "fully off-canvas",
            DrawRect {
                x: -200.0,
                y: -200.0,
                w: 50.0,
                h: 50.0,
                color: Color::YELLOW,
            },
        ),
        (
            "out-of-range color",
            DrawRect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
                color: Color::new(2.0, -1.0, 0.5, 1.5),
            },
        ),
    ];
    println!("M1.1 · Canvas2D clip + clamp (presentar_core::Color)\n");
    println!("canvas: {}×{}\n", canvas.width, canvas.height);
    for (label, rect) in &probes {
        match clip(*rect, canvas) {
            Some(c) => println!(
                "  {label:<22} -> clipped to ({:.0}, {:.0}, {:.0}, {:.0}) color=({:.2},{:.2},{:.2},{:.2})",
                c.x, c.y, c.w, c.h, c.color.r, c.color.g, c.color.b, c.color.a,
            ),
            None => println!("  {label:<22} -> dropped (NaN / off-canvas / zero-area)"),
        }
    }
    eprintln!("{}", contract_marker());
}
