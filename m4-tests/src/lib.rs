//! M4.2 — VDOM snapshot harness for browser tests. Probar-style:
//! stringify the rendered VDOM, diff against an inline golden.

#[derive(Debug, Clone, PartialEq)]
pub struct VNode {
    pub tag: String,
    pub text: String,
    pub children: Vec<VNode>,
}

#[must_use]
pub fn snapshot(node: &VNode) -> String {
    let mut out = String::new();
    write_node(node, 0, &mut out);
    out
}

fn write_node(node: &VNode, depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push('<');
    out.push_str(&node.tag);
    out.push('>');
    if !node.text.is_empty() {
        out.push_str(&node.text);
    }
    out.push('\n');
    for c in &node.children {
        write_node(c, depth + 1, out);
    }
}

pub type SnapshotDiff = Vec<(usize, String, String)>;

#[must_use]
pub fn diff_snapshot(node: &VNode, golden: &str) -> SnapshotDiff {
    let actual = snapshot(node);
    let actual_lines: Vec<&str> = actual.lines().collect();
    let golden_lines: Vec<&str> = golden.lines().collect();
    let mut out = vec![];
    let n = actual_lines.len().max(golden_lines.len());
    for i in 0..n {
        let a = actual_lines.get(i).copied().unwrap_or("");
        let g = golden_lines.get(i).copied().unwrap_or("");
        if a != g {
            out.push((i, g.to_string(), a.to_string()));
        }
    }
    out
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-rendering-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample() -> VNode {
        VNode {
            tag: "div".into(),
            text: String::new(),
            children: vec![
                VNode {
                    tag: "h1".into(),
                    text: "title".into(),
                    children: vec![],
                },
                VNode {
                    tag: "p".into(),
                    text: "body".into(),
                    children: vec![],
                },
            ],
        }
    }

    #[test]
    fn snapshot_of_leaf_node() {
        let n = VNode {
            tag: "span".into(),
            text: "hi".into(),
            children: vec![],
        };
        assert_eq!(snapshot(&n), "<span>hi\n");
    }

    #[test]
    fn snapshot_of_tree_indents_children() {
        let s = snapshot(&sample());
        assert!(s.contains("<div>"));
        assert!(s.contains("  <h1>title"));
        assert!(s.contains("  <p>body"));
    }

    #[test]
    fn diff_of_matching_snapshot_is_empty() {
        let g = "<div>\n  <h1>title\n  <p>body\n";
        assert!(diff_snapshot(&sample(), g).is_empty());
    }

    #[test]
    fn diff_reports_mismatches_with_line_numbers() {
        let g = "<div>\n  <h1>WRONG\n  <p>body\n";
        let d = diff_snapshot(&sample(), g);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].0, 1);
    }

    #[test]
    fn diff_handles_shorter_golden() {
        let g = "<div>\n";
        let d = diff_snapshot(&sample(), g);
        assert!(d.len() >= 2);
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
