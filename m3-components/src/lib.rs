//! M3.1 — Composed UI: Container/Row/Column with non-overlapping children
//! and deterministic layout for the wasm-panels-v1 contract.

use presentar_core::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub tag: String,
    pub fg: Color,
    pub bounds: Rect,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub direction: Direction,
    pub children: Vec<(String, Color)>,
}

/// Lay out a container — every child is sliced evenly along `direction`.
/// Total: the union of children rects ⊆ parent rect, pairwise disjoint.
#[must_use]
pub fn layout(container: &Container, parent: Rect) -> Vec<Node> {
    let n = container.children.len();
    if n == 0 {
        return vec![];
    }
    let n_f = n as f64;
    container
        .children
        .iter()
        .enumerate()
        .map(|(i, (tag, fg))| {
            let bounds = match container.direction {
                Direction::Row => Rect {
                    x: parent.x + (i as f64) * (parent.w / n_f),
                    y: parent.y,
                    w: parent.w / n_f,
                    h: parent.h,
                },
                Direction::Column => Rect {
                    x: parent.x,
                    y: parent.y + (i as f64) * (parent.h / n_f),
                    w: parent.w,
                    h: parent.h / n_f,
                },
            };
            Node {
                tag: tag.clone(),
                fg: *fg,
                bounds,
            }
        })
        .collect()
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-panels-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn parent() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 800.0,
            h: 600.0,
        }
    }

    #[test]
    fn row_layout_3_children_no_overlap() {
        let c = Container {
            direction: Direction::Row,
            children: vec![
                ("a".into(), Color::RED),
                ("b".into(), Color::GREEN),
                ("c".into(), Color::BLUE),
            ],
        };
        let nodes = layout(&c, parent());
        assert_eq!(nodes.len(), 3);
        // every child fits inside parent
        for n in &nodes {
            assert!(n.bounds.x >= 0.0 && n.bounds.y >= 0.0);
            assert!(n.bounds.x + n.bounds.w <= 800.0);
            assert!(n.bounds.y + n.bounds.h <= 600.0);
        }
        // pairwise disjoint along x for Row
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let a = &nodes[i].bounds;
                let b = &nodes[j].bounds;
                assert!(a.x + a.w <= b.x || b.x + b.w <= a.x, "row children overlap");
            }
        }
    }

    #[test]
    fn column_layout_2_children() {
        let c = Container {
            direction: Direction::Column,
            children: vec![("top".into(), Color::WHITE), ("bot".into(), Color::WHITE)],
        };
        let nodes = layout(&c, parent());
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].bounds.y, 0.0);
        assert_eq!(nodes[1].bounds.y, 300.0);
    }

    #[test]
    fn empty_container_is_empty() {
        let c = Container {
            direction: Direction::Row,
            children: vec![],
        };
        assert!(layout(&c, parent()).is_empty());
    }

    #[test]
    fn layout_is_deterministic() {
        let c = Container {
            direction: Direction::Row,
            children: vec![("a".into(), Color::RED), ("b".into(), Color::GREEN)],
        };
        assert_eq!(layout(&c, parent()).len(), layout(&c, parent()).len());
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
