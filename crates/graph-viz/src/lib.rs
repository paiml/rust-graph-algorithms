//! Graphviz DOT export.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, Module 5.

#![deny(missing_docs)]

use graph_core::Graph;
use std::fmt::Write;

/// Render a [`Graph`] as a Graphviz DOT directed graph. Edge weights are
/// emitted as `label="weight"` attributes. The output ends with a single
/// trailing newline.
pub fn to_dot(g: &Graph) -> String {
    let mut s = String::from("digraph G {\n");
    for n in g.nodes() {
        writeln!(s, "  {} [label=\"{}\"];", n.0, n.0).unwrap();
    }
    for src in g.nodes() {
        for e in g.neighbors(src) {
            writeln!(s, "  {} -> {} [label=\"{}\"];", e.from.0, e.to.0, e.weight).unwrap();
        }
    }
    s.push_str("}\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::{Edge, NodeId};

    #[test]
    fn empty_graph_emits_only_braces() {
        let g = Graph::with_capacity(0);
        let dot = to_dot(&g);
        assert_eq!(dot, "digraph G {\n}\n");
    }

    #[test]
    fn single_node_no_edges() {
        let g = Graph::with_capacity(1);
        let dot = to_dot(&g);
        assert!(dot.contains("0 [label=\"0\"];"));
        assert!(!dot.contains("->"));
    }

    #[test]
    fn two_nodes_one_edge() {
        let mut g = Graph::with_capacity(2);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 5)).unwrap();
        let dot = to_dot(&g);
        assert!(dot.contains("0 -> 1 [label=\"5\"];"));
        assert!(dot.contains("0 [label=\"0\"];"));
        assert!(dot.contains("1 [label=\"1\"];"));
    }

    #[test]
    fn dot_output_ends_with_newline() {
        let g = Graph::with_capacity(1);
        let dot = to_dot(&g);
        assert!(dot.ends_with("}\n"));
    }
}
