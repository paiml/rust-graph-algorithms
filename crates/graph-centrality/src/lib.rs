//! Centrality scores — out-degree and PageRank.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, Module 3.

#![deny(missing_docs)]

use graph_core::Graph;

/// Out-degree centrality. Returns one score per node, normalized to
/// `[0, 1]` by dividing the out-degree by `n - 1`. The single-node graph
/// returns `vec![0.0]`.
pub fn out_degree_centrality(g: &Graph) -> Vec<f64> {
    let n = g.node_count();
    let denom = if n > 1 { (n - 1) as f64 } else { 1.0 };
    g.nodes()
        .map(|node| g.neighbors(node).len() as f64 / denom)
        .collect()
}

/// PageRank via power iteration. Returns one score per node summing to
/// approximately `1.0`.
///
/// `damping` is the standard 0.85 in the original paper. `eps` is the
/// L1 norm threshold below which iteration halts. `max_iter` caps the
/// loop in case the graph is pathological.
///
/// Empty graphs return an empty vector.
pub fn pagerank(g: &Graph, damping: f64, eps: f64, max_iter: usize) -> Vec<f64> {
    let n = g.node_count();
    if n == 0 {
        return Vec::new();
    }
    let inv_n = 1.0 / n as f64;
    let mut score = vec![inv_n; n];
    let mut next = vec![0.0; n];
    let teleport = (1.0 - damping) * inv_n;

    // Pre-compute out-degree to avoid recomputing per iteration.
    let out_deg: Vec<usize> = g.nodes().map(|node| g.neighbors(node).len()).collect();

    for _ in 0..max_iter {
        // Reset next to teleport mass plus dangling-node redistribution.
        let dangling: f64 = g
            .nodes()
            .filter(|node| out_deg[node.index()] == 0)
            .map(|node| score[node.index()])
            .sum();
        let dangling_share = damping * dangling * inv_n;
        for x in next.iter_mut() {
            *x = teleport + dangling_share;
        }

        // Push contributions across edges.
        for src in g.nodes() {
            let d = out_deg[src.index()];
            if d == 0 {
                continue;
            }
            let share = damping * score[src.index()] / d as f64;
            for e in g.neighbors(src) {
                next[e.to.index()] += share;
            }
        }

        // L1 delta and swap.
        let delta: f64 = score
            .iter()
            .zip(next.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        std::mem::swap(&mut score, &mut next);
        if delta < eps {
            break;
        }
    }
    score
}

/// Provable contract — referenced from `contracts/graph-algorithms-v1.yaml`.
///
/// PageRank scores must sum to approximately `1.0` (within `tol`).
pub fn assert_pagerank_normalized(scores: &[f64], tol: f64) {
    let sum: f64 = scores.iter().sum();
    assert!(
        (sum - 1.0).abs() < tol,
        "PageRank scores sum to {sum}, expected 1.0 ± {tol}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::{Edge, NodeId};

    #[test]
    fn out_degree_single_node() {
        let g = Graph::with_capacity(1);
        let c = out_degree_centrality(&g);
        assert_eq!(c, vec![0.0]);
    }

    #[test]
    fn out_degree_multi_node() {
        // 0 → {1, 2}, 1 → 2. n=3, denom=2. Scores: [2/2, 1/2, 0/2].
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        let c = out_degree_centrality(&g);
        assert_eq!(c, vec![1.0, 0.5, 0.0]);
    }

    #[test]
    fn pagerank_empty_graph() {
        let g = Graph::with_capacity(0);
        assert!(pagerank(&g, 0.85, 1e-9, 100).is_empty());
    }

    #[test]
    fn pagerank_uniform_two_node() {
        // Symmetric undirected pair — equal scores summing to 1.
        let mut g = Graph::with_capacity(2);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 1))
            .unwrap();
        let pr = pagerank(&g, 0.85, 1e-9, 100);
        assert!((pr[0] - 0.5).abs() < 1e-6);
        assert!((pr[1] - 0.5).abs() < 1e-6);
        assert_pagerank_normalized(&pr, 1e-6);
    }

    #[test]
    fn pagerank_directed_chain_skew() {
        // 0 → 1 → 2; the sink (2) accumulates rank.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        let pr = pagerank(&g, 0.85, 1e-9, 200);
        assert!(pr[2] > pr[1]);
        assert!(pr[1] > pr[0]);
        assert_pagerank_normalized(&pr, 1e-6);
    }

    #[test]
    fn pagerank_isolated_nodes_terminate_via_max_iter() {
        // No edges — every iteration is identical, delta = 0 immediately.
        let g = Graph::with_capacity(4);
        let pr = pagerank(&g, 0.85, 1e-9, 5);
        for s in &pr {
            assert!((s - 0.25).abs() < 1e-9);
        }
    }

    #[test]
    fn pagerank_max_iter_halts() {
        // eps so tight it can't converge — max_iter must end the loop.
        let mut g = Graph::with_capacity(2);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        let pr = pagerank(&g, 0.85, 0.0, 3);
        assert_eq!(pr.len(), 2);
    }

    #[test]
    #[should_panic(expected = "PageRank scores sum to")]
    fn assert_pagerank_normalized_catches_drift() {
        let bad = vec![0.4, 0.4];
        assert_pagerank_normalized(&bad, 1e-3);
    }
}
