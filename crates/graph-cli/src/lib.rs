//! `graph` CLI logic.
//!
//! All real work happens inside [`run`] — the binary entry-point in
//! `main.rs` is a thin shell that parses [`Cli`] and exits on error.
//! Algorithms live in the sibling crates (`graph-traversal`, `-centrality`,
//! `-community`, `-viz`); this crate orchestrates them.
//!
//! Provable contracts asserted at runtime after every successful command:
//! - [`graph_core::assert_edge_count_consistent`]
//! - [`graph_centrality::assert_pagerank_normalized`] (when running `pagerank`)
//! - [`graph_community::assert_components_partition`] (when running `scc` or
//!   `components`)

#![deny(missing_docs)]

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use graph_core::{assert_edge_count_consistent, Edge, Graph, NodeId};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Top-level CLI.
#[derive(Debug, Parser)]
#[command(name = "graph", version, about = "Graph algorithms in Rust")]
pub struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    pub cmd: Cmd,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// BFS visit order from `source`.
    Bfs {
        /// Starting node.
        #[arg(long, default_value_t = 0)]
        source: u32,
    },
    /// DFS visit order from `source`.
    Dfs {
        /// Starting node.
        #[arg(long, default_value_t = 0)]
        source: u32,
    },
    /// Single-source shortest paths (Dijkstra).
    Dijkstra {
        /// Starting node.
        #[arg(long, default_value_t = 0)]
        source: u32,
    },
    /// PageRank scores per node.
    Pagerank {
        /// Damping factor (default 0.85).
        #[arg(long, default_value_t = 0.85)]
        damping: f64,
        /// L1-norm convergence threshold.
        #[arg(long, default_value_t = 1e-6)]
        eps: f64,
        /// Maximum iteration count.
        #[arg(long, default_value_t = 200)]
        max_iter: usize,
    },
    /// Weakly-connected components.
    Components,
    /// Strongly-connected components (Kosaraju).
    Scc,
    /// Emit Graphviz DOT.
    Dot,
}

/// Input format on stdin: number of nodes plus a list of directed edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Input {
    /// Total node count.
    pub nodes: usize,
    /// Edges. Use `Edge::new` to build them.
    pub edges: Vec<Edge>,
}

/// Result envelope written to `out`. One variant per subcommand keeps the
/// downstream parsing uniform across all calls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Output {
    /// BFS visit order.
    Bfs {
        /// The visit order.
        order: Vec<u32>,
    },
    /// DFS visit order.
    Dfs {
        /// The visit order.
        order: Vec<u32>,
    },
    /// Dijkstra distances. Index `i` = distance from source to node `i`,
    /// or `null` if unreachable.
    Dijkstra {
        /// Distances.
        distances: Vec<Option<u32>>,
    },
    /// PageRank scores per node, summing to ~1.0.
    Pagerank {
        /// Scores.
        scores: Vec<f64>,
    },
    /// Weakly-connected components.
    Components {
        /// One inner Vec per component.
        components: Vec<Vec<u32>>,
    },
    /// Strongly-connected components.
    Scc {
        /// One inner Vec per SCC.
        components: Vec<Vec<u32>>,
    },
    /// DOT source.
    Dot {
        /// The Graphviz DOT source code.
        dot: String,
    },
}

/// Read JSON from `r`, dispatch on `cli.cmd`, write JSON to `w`.
pub fn run(cli: Cli, mut r: impl Read, mut w: impl Write) -> Result<()> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    let input: Input = serde_json::from_str(&buf)?;
    let g = Graph::from_edges(input.nodes, &input.edges)?;
    assert_edge_count_consistent(&g);

    let out = match cli.cmd {
        Cmd::Bfs { source } => {
            check_source(&g, source)?;
            let order = graph_traversal::bfs(&g, NodeId(source));
            Output::Bfs {
                order: order.into_iter().map(|n| n.0).collect(),
            }
        }
        Cmd::Dfs { source } => {
            check_source(&g, source)?;
            let order = graph_traversal::dfs(&g, NodeId(source));
            Output::Dfs {
                order: order.into_iter().map(|n| n.0).collect(),
            }
        }
        Cmd::Dijkstra { source } => {
            check_source(&g, source)?;
            let distances = graph_traversal::dijkstra(&g, NodeId(source));
            Output::Dijkstra { distances }
        }
        Cmd::Pagerank {
            damping,
            eps,
            max_iter,
        } => {
            let scores = graph_centrality::pagerank(&g, damping, eps, max_iter);
            if !scores.is_empty() {
                graph_centrality::assert_pagerank_normalized(&scores, 1e-3);
            }
            Output::Pagerank { scores }
        }
        Cmd::Components => {
            let comps = graph_community::connected_components(&g);
            graph_community::assert_components_partition(&g, &comps);
            Output::Components {
                components: comps_to_u32(comps),
            }
        }
        Cmd::Scc => {
            let comps = graph_community::kosaraju(&g);
            graph_community::assert_components_partition(&g, &comps);
            Output::Scc {
                components: comps_to_u32(comps),
            }
        }
        Cmd::Dot => {
            let dot = graph_viz::to_dot(&g);
            Output::Dot { dot }
        }
    };

    let json = serde_json::to_string(&out)?;
    writeln!(w, "{json}")?;
    Ok(())
}

fn check_source(g: &Graph, source: u32) -> Result<()> {
    if (source as usize) >= g.node_count() {
        bail!(
            "source {} out of range (graph has {} nodes)",
            source,
            g.node_count()
        );
    }
    Ok(())
}

fn comps_to_u32(comps: Vec<Vec<NodeId>>) -> Vec<Vec<u32>> {
    comps
        .into_iter()
        .map(|c| c.into_iter().map(|n| n.0).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_json() -> &'static str {
        // 0 → 1, 1 → 2, 2 → 0 (one SCC of 3).
        r#"{"nodes":3,"edges":[
            {"from":0,"to":1,"weight":1},
            {"from":1,"to":2,"weight":1},
            {"from":2,"to":0,"weight":1}
        ]}"#
    }

    fn run_cmd(args: &[&str]) -> Output {
        let cli = Cli::parse_from(args);
        let mut out: Vec<u8> = Vec::new();
        run(cli, input_json().as_bytes(), &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        serde_json::from_str(s.trim()).unwrap()
    }

    #[test]
    fn cli_bfs() {
        let out = run_cmd(&["graph", "bfs", "--source", "0"]);
        assert_eq!(
            out,
            Output::Bfs {
                order: vec![0, 1, 2]
            }
        );
    }

    #[test]
    fn cli_dfs() {
        let out = run_cmd(&["graph", "dfs", "--source", "0"]);
        assert_eq!(
            out,
            Output::Dfs {
                order: vec![0, 1, 2]
            }
        );
    }

    #[test]
    fn cli_dijkstra() {
        let out = run_cmd(&["graph", "dijkstra", "--source", "0"]);
        assert_eq!(
            out,
            Output::Dijkstra {
                distances: vec![Some(0), Some(1), Some(2)]
            }
        );
    }

    #[test]
    fn cli_pagerank() {
        let out = run_cmd(&["graph", "pagerank"]);
        // matches! with guards keeps coverage tight: both the type check
        // and the value check live in one expression, no unreachable
        // panic arm.
        assert!(matches!(&out, Output::Pagerank { scores } if scores.len() == 3));
        assert!(matches!(
            &out,
            Output::Pagerank { scores }
                if (scores.iter().sum::<f64>() - 1.0).abs() < 1e-3
        ));
    }

    #[test]
    fn cli_components() {
        let out = run_cmd(&["graph", "components"]);
        assert_eq!(
            out,
            Output::Components {
                components: vec![vec![0, 1, 2]]
            }
        );
    }

    #[test]
    fn cli_scc() {
        let out = run_cmd(&["graph", "scc"]);
        assert!(matches!(
            &out,
            Output::Scc { components }
                if components.len() == 1 && components[0].len() == 3
        ));
    }

    #[test]
    fn cli_dot() {
        let out = run_cmd(&["graph", "dot"]);
        assert!(matches!(
            &out,
            Output::Dot { dot }
                if dot.starts_with("digraph G {") && dot.contains("0 -> 1")
        ));
    }

    #[test]
    fn pagerank_empty_graph_skips_normalization() {
        let cli = Cli::parse_from(["graph", "pagerank"]);
        let input = r#"{"nodes":0,"edges":[]}"#;
        let mut out = Vec::new();
        run(cli, input.as_bytes(), &mut out).unwrap();
        let parsed: Output = serde_json::from_str(String::from_utf8(out).unwrap().trim()).unwrap();
        assert_eq!(parsed, Output::Pagerank { scores: vec![] });
    }

    #[test]
    fn source_out_of_range_errors() {
        let cli = Cli::parse_from(["graph", "bfs", "--source", "9"]);
        let mut out = Vec::new();
        let err = run(cli, input_json().as_bytes(), &mut out).unwrap_err();
        assert!(err.to_string().contains("source 9 out of range"));
    }

    #[test]
    fn malformed_input_errors() {
        let cli = Cli::parse_from(["graph", "bfs"]);
        let mut out = Vec::new();
        let err = run(cli, "not json".as_bytes(), &mut out).unwrap_err();
        let _ = err;
    }

    #[test]
    fn graph_constructor_error_propagates() {
        // Edge references node id 9 in a 2-node graph.
        let cli = Cli::parse_from(["graph", "bfs"]);
        let bad = r#"{"nodes":2,"edges":[{"from":0,"to":9,"weight":1}]}"#;
        let mut out = Vec::new();
        let err = run(cli, bad.as_bytes(), &mut out).unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn input_serde_roundtrip() {
        let i = Input {
            nodes: 2,
            edges: vec![Edge::new(NodeId(0), NodeId(1), 1)],
        };
        let s = serde_json::to_string(&i).unwrap();
        let back: Input = serde_json::from_str(&s).unwrap();
        assert_eq!(i, back);
    }

    #[test]
    fn output_clone_debug() {
        let o = Output::Bfs { order: vec![0, 1] };
        let _ = format!("{o:?}");
        let _ = o.clone();
    }

    #[test]
    fn cli_debug() {
        let c = Cli::parse_from(["graph", "components"]);
        let _ = format!("{c:?}");
    }
}
