//! Working demo of `aprender-graph` (re-exported as `trueno_graph`).
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, lesson 1.1.4 (aprender-graph quickstart).
//!
//! Constructs a 5-node directed graph, runs three algorithms via the
//! real `aprender-graph 0.31.2` crate, and asserts two structural
//! contracts at runtime:
//!
//! - [`assert_pagerank_normalized`] — scores sum to 1.0 ± tol
//! - [`assert_components_partition`] — Kosaraju output partitions V
//!
//! Outputs flow through [`DemoReport`] so tests can compare against
//! known values; the values shown in the companion SVG and any
//! marketing copy must match what [`run_demo`] returns.

#![deny(missing_docs)]

use anyhow::{Context, Result};
use trueno_graph::{CsrGraph, NodeId};

/// Whatever the demo run produced. Every field is observable so the
/// SVG, the lesson narration, and any external copy can be falsified
/// against the live numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoReport {
    /// Number of nodes in the demo graph.
    pub num_nodes: usize,
    /// BFS pre-order visit sequence from node 0.
    pub bfs_order: Vec<u32>,
    /// PageRank scores, indexed by node id.
    pub pagerank: Vec<f32>,
    /// Sum of all PageRank scores. Should be 1.0 ± 1e-3.
    pub pagerank_sum: f32,
    /// Strongly-connected components, each inner Vec is one SCC.
    pub sccs: Vec<Vec<NodeId>>,
    /// Index of the highest-PageRank node (the "winner").
    pub winner: usize,
}

/// Build the canonical 5-node PageRank demo graph used in lesson 1.1.4.
///
/// Edges: 0→2, 1→2, 2→3, 2→4, 3→2, 4→2.
/// Node 2 is the hub — every other node points at it directly or
/// reciprocally — and PageRank should concentrate there.
pub fn build_demo_graph() -> Result<CsrGraph> {
    let edges = [
        (NodeId(0), NodeId(2), 1.0_f32),
        (NodeId(1), NodeId(2), 1.0_f32),
        (NodeId(2), NodeId(3), 1.0_f32),
        (NodeId(2), NodeId(4), 1.0_f32),
        (NodeId(3), NodeId(2), 1.0_f32),
        (NodeId(4), NodeId(2), 1.0_f32),
    ];
    CsrGraph::from_edge_list(&edges).context("CsrGraph::from_edge_list")
}

/// Run the full demo: construct the graph, run BFS, PageRank, and
/// Kosaraju SCC, assert two structural contracts, return a report.
pub fn run_demo() -> Result<DemoReport> {
    let g = build_demo_graph()?;
    let num_nodes = g.num_nodes();

    let bfs_order = trueno_graph::bfs(&g, NodeId(0)).context("bfs")?;
    let pagerank = trueno_graph::pagerank(&g, 200, 1e-9).context("pagerank")?;
    let sccs = trueno_graph::kosaraju_scc(&g);

    let pagerank_sum: f32 = pagerank.iter().sum();
    let winner = pick_winner(&pagerank).context("pagerank produced no scores")?;

    let report = DemoReport {
        num_nodes,
        bfs_order,
        pagerank,
        pagerank_sum,
        sccs,
        winner,
    };

    // Runtime contracts — any drift trips a panic and aborts the demo.
    assert_pagerank_normalized(&report.pagerank, 1e-3);
    assert_components_partition(report.num_nodes, &report.sccs);

    Ok(report)
}

fn pick_winner(scores: &[f32]) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, &s) in scores.iter().enumerate() {
        match best {
            None => best = Some((i, s)),
            Some((_, b)) if s > b => best = Some((i, s)),
            _ => {}
        }
    }
    best.map(|(i, _)| i)
}

/// Provable contract — referenced from
/// `contracts/aprender-demo-v1.yaml`. Sum of PageRank scores is
/// approximately 1.0.
pub fn assert_pagerank_normalized(scores: &[f32], tol: f32) {
    let sum: f32 = scores.iter().sum();
    assert!(
        (sum - 1.0).abs() < tol,
        "PageRank sum {} drifted from 1.0 by more than {}",
        sum,
        tol
    );
}

/// Provable contract — referenced from
/// `contracts/aprender-demo-v1.yaml`. Components form a partition of V.
pub fn assert_components_partition(num_nodes: usize, comps: &[Vec<NodeId>]) {
    let mut seen = vec![false; num_nodes];
    for c in comps {
        for node in c {
            let i = node.0 as usize;
            assert!(!seen[i], "node {} appears in two components", i);
            seen[i] = true;
        }
    }
    for (i, s) in seen.iter().enumerate() {
        assert!(*s, "node {} missing from any component", i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_graph_has_5_nodes() {
        let g = build_demo_graph().unwrap();
        assert_eq!(g.num_nodes(), 5);
    }

    #[test]
    fn run_demo_returns_consistent_report() {
        let r = run_demo().unwrap();
        assert_eq!(r.num_nodes, 5);
        assert_eq!(r.pagerank.len(), 5);
        // BFS from 0 reaches {0, 2, 3, 4} via 0→2 and 2→{3,4}.
        // Node 1 is unreachable from 0 (1→2 only, no 0→1 edge).
        assert_eq!(r.bfs_order.len(), 4);
        let mut reached: Vec<u32> = r.bfs_order.clone();
        reached.sort();
        assert_eq!(reached, vec![0, 2, 3, 4]);
        // Sum stays normalized.
        assert!((r.pagerank_sum - 1.0).abs() < 1e-3);
    }

    #[test]
    fn pagerank_scores_match_expected() {
        // Verified-against-real-aprender-graph values. Drift here means
        // either the dep version changed or the demo graph changed —
        // either way an SVG/lesson narration claim has been falsified.
        let r = run_demo().unwrap();
        let expected = [0.030_f32, 0.030, 0.476, 0.232, 0.232];
        for (i, (got, want)) in r.pagerank.iter().zip(expected.iter()).enumerate() {
            assert!(
                (got - want).abs() < 0.005,
                "node {} score {} drifted from expected {}",
                i,
                got,
                want
            );
        }
    }

    #[test]
    fn sccs_are_3_components_with_hub_cycle() {
        // Verified: {1}, {0}, {2, 3, 4}. Nodes 0 and 1 are sources
        // (singleton SCCs); 2 ↔ 3 and 2 ↔ 4 form one SCC.
        let r = run_demo().unwrap();
        assert_eq!(r.sccs.len(), 3);
        let big = r
            .sccs
            .iter()
            .max_by_key(|c| c.len())
            .expect("at least one SCC");
        assert_eq!(big.len(), 3);
        let mut big_ids: Vec<u32> = big.iter().map(|n| n.0).collect();
        big_ids.sort();
        assert_eq!(big_ids, vec![2, 3, 4]);
    }

    #[test]
    fn winner_is_node_2() {
        // Node 2 is the hub of the demo graph — PageRank must rank it #1.
        let r = run_demo().unwrap();
        assert_eq!(r.winner, 2);
    }

    #[test]
    fn sccs_partition_total_matches_node_count() {
        let r = run_demo().unwrap();
        let total: usize = r.sccs.iter().map(|c| c.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn ordering_by_score() {
        // PageRank should rank node 2 highest, then {3, 4}, then {0, 1}.
        let r = run_demo().unwrap();
        assert!(r.pagerank[2] > r.pagerank[3]);
        assert!(r.pagerank[2] > r.pagerank[4]);
        assert!(r.pagerank[3] > r.pagerank[0]);
        assert!(r.pagerank[3] > r.pagerank[1]);
        assert!(r.pagerank[4] > r.pagerank[0]);
        assert!(r.pagerank[4] > r.pagerank[1]);
    }

    #[test]
    fn pick_winner_empty_returns_none() {
        assert_eq!(pick_winner(&[]), None);
    }

    #[test]
    fn pick_winner_handles_ties() {
        // Ties: pick_winner returns the FIRST seen max (stable for the
        // demo, since node 2 is strictly maximal here).
        let scores = vec![0.5, 0.5, 0.0];
        assert_eq!(pick_winner(&scores), Some(0));
    }

    #[test]
    fn assert_pagerank_normalized_passes_on_real_run() {
        let r = run_demo().unwrap();
        assert_pagerank_normalized(&r.pagerank, 1e-3);
    }

    #[test]
    #[should_panic(expected = "PageRank sum")]
    fn assert_pagerank_normalized_catches_drift() {
        // Hand-rolled bad scores (sum=0.4, tol=1e-3 → panics).
        assert_pagerank_normalized(&[0.2, 0.2], 1e-3);
    }

    #[test]
    fn assert_components_partition_passes_on_real_sccs() {
        let r = run_demo().unwrap();
        assert_components_partition(r.num_nodes, &r.sccs);
    }

    #[test]
    #[should_panic(expected = "appears in two components")]
    fn assert_components_partition_catches_overlap() {
        let bad = vec![vec![NodeId(0)], vec![NodeId(0), NodeId(1)]];
        assert_components_partition(2, &bad);
    }

    #[test]
    #[should_panic(expected = "missing from any component")]
    fn assert_components_partition_catches_missing() {
        let bad = vec![vec![NodeId(0)]]; // missing node 1
        assert_components_partition(2, &bad);
    }

    #[test]
    fn report_clone_debug_eq() {
        let r = run_demo().unwrap();
        let c = r.clone();
        assert_eq!(r, c);
        let _ = format!("{r:?}");
    }
}
