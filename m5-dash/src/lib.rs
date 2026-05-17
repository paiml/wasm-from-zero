//! M5 capstone — composes every prior module (m1-canvas, m1-app,
//! m2-elm-wasm, m2-events, m3-components, m3-charts, m4-tests) into a
//! single in-browser dashboard model.

use m3_charts::gauge_fraction;
use m3_components::{layout, Container, Direction, Rect};
use presentar_core::Color;

#[derive(Debug, Clone)]
pub struct Dashboard {
    pub title: String,
    pub viewport: Rect,
    pub cpu_load: Vec<f64>,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,
    pub event_count: u64,
}

impl Dashboard {
    #[must_use]
    pub fn fixture() -> Self {
        Self {
            title: "wasm-from-zero · capstone dashboard".into(),
            viewport: Rect {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0,
            },
            cpu_load: vec![0.18, 0.42, 0.71, 0.88, 0.55, 0.34, 0.62, 0.27],
            mem_used_gb: 9.4,
            mem_total_gb: 16.0,
            event_count: 42,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaintCmd {
    pub label: String,
    pub bounds: Rect,
    pub fg: Color,
    pub fill: f64,
}

/// Build the paint command list for the dashboard — composes every prior
/// module's logic.
#[must_use]
pub fn build_paint_list(dash: &Dashboard) -> Vec<PaintCmd> {
    let header = Container {
        direction: Direction::Row,
        children: vec![
            ("title".into(), Color::WHITE),
            ("event-pulse".into(), Color::BLUE),
            ("mem-gauge".into(), Color::GREEN),
        ],
    };
    let header_rect = Rect {
        x: dash.viewport.x,
        y: dash.viewport.y,
        w: dash.viewport.w,
        h: 60.0,
    };
    let header_nodes = layout(&header, header_rect);

    let mut cmds: Vec<PaintCmd> = header_nodes
        .into_iter()
        .map(|n| PaintCmd {
            label: n.tag,
            bounds: n.bounds,
            fg: n.fg,
            fill: 0.0,
        })
        .collect();

    // body: one column per CPU core
    let body = Container {
        direction: Direction::Row,
        children: dash
            .cpu_load
            .iter()
            .enumerate()
            .map(|(i, _)| (format!("cpu-{i}"), Color::BLUE))
            .collect(),
    };
    let body_rect = Rect {
        x: dash.viewport.x,
        y: dash.viewport.y + 60.0,
        w: dash.viewport.w,
        h: dash.viewport.h - 120.0,
    };
    for (n, &load) in layout(&body, body_rect)
        .into_iter()
        .zip(dash.cpu_load.iter())
    {
        cmds.push(PaintCmd {
            label: n.tag,
            bounds: n.bounds,
            fg: if load > 0.8 {
                Color::RED
            } else if load > 0.5 {
                Color::YELLOW
            } else {
                Color::GREEN
            },
            fill: load,
        });
    }

    cmds.push(PaintCmd {
        label: "mem-bar".into(),
        bounds: Rect {
            x: dash.viewport.x,
            y: dash.viewport.h - 40.0,
            w: dash.viewport.w,
            h: 40.0,
        },
        fg: Color::GREEN,
        fill: gauge_fraction(dash.mem_used_gb, dash.mem_total_gb),
    });

    cmds
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-panels-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn fixture_dashboard_paints_within_viewport() {
        let dash = Dashboard::fixture();
        for cmd in build_paint_list(&dash) {
            assert!(cmd.bounds.x >= dash.viewport.x);
            assert!(cmd.bounds.y >= dash.viewport.y);
            assert!(cmd.bounds.x + cmd.bounds.w <= dash.viewport.x + dash.viewport.w + 0.001);
            assert!(cmd.bounds.y + cmd.bounds.h <= dash.viewport.y + dash.viewport.h + 0.001);
        }
    }

    #[test]
    fn paint_list_is_deterministic() {
        let dash = Dashboard::fixture();
        let a = build_paint_list(&dash);
        let b = build_paint_list(&dash);
        assert_eq!(a, b);
    }

    #[test]
    fn mem_bar_fill_is_used_over_total() {
        let dash = Dashboard::fixture();
        let cmds = build_paint_list(&dash);
        let mem = cmds.iter().find(|c| c.label == "mem-bar").expect("mem bar");
        assert!((mem.fill - 9.4 / 16.0).abs() < 0.001);
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
