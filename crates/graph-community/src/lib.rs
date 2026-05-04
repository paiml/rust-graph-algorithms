//! Community detection — connected components and Kosaraju SCC.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, Module 4.

#![deny(missing_docs)]

use graph_core::{Graph, NodeId};

/// Weakly-connected components on a directed graph (treats every edge
/// as undirected). Returns one inner Vec per component, each sorted by
/// node id. The outer Vec is sorted by smallest node id.
pub fn connected_components(g: &Graph) -> Vec<Vec<NodeId>> {
    let n = g.node_count();
    let mut comp_of = vec![usize::MAX; n];
    let mut comps: Vec<Vec<NodeId>> = Vec::new();
    for seed in g.nodes() {
        if comp_of[seed.index()] != usize::MAX {
            continue;
        }
        let cid = comps.len();
        comps.push(Vec::new());
        let mut stack = vec![seed];
        while let Some(node) = stack.pop() {
            if comp_of[node.index()] != usize::MAX {
                continue;
            }
            comp_of[node.index()] = cid;
            comps[cid].push(node);
            // Forward edges.
            for e in g.neighbors(node) {
                if comp_of[e.to.index()] == usize::MAX {
                    stack.push(e.to);
                }
            }
        }
    }
    // Treat as undirected by union-finding from the transpose too.
    let t = g.transpose();
    let mut parent: Vec<usize> = (0..comps.len()).collect();
    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }
    for src in t.nodes() {
        for e in t.neighbors(src) {
            let a = find(&mut parent, comp_of[src.index()]);
            let b = find(&mut parent, comp_of[e.to.index()]);
            if a != b {
                parent[a] = b;
            }
        }
    }
    // Re-bucket by root.
    let mut out: Vec<Vec<NodeId>> = vec![Vec::new(); comps.len()];
    for node in g.nodes() {
        let r = find(&mut parent, comp_of[node.index()]);
        out[r].push(node);
    }
    let mut nonempty: Vec<Vec<NodeId>> = out.into_iter().filter(|c| !c.is_empty()).collect();
    for c in nonempty.iter_mut() {
        c.sort();
    }
    nonempty.sort_by_key(|c| c[0]);
    nonempty
}

/// Strongly-connected components via Kosaraju's two-pass DFS algorithm.
/// Returns one inner Vec per component. Each component lists its nodes
/// in arbitrary order; the outer Vec is sorted by smallest node id.
pub fn kosaraju(g: &Graph) -> Vec<Vec<NodeId>> {
    let n = g.node_count();
    // Pass 1: DFS on the forward graph, recording finish order.
    let mut visited = vec![false; n];
    let mut order: Vec<NodeId> = Vec::with_capacity(n);
    for start in g.nodes() {
        if visited[start.index()] {
            continue;
        }
        // Iterative DFS recording post-order.
        let mut stack: Vec<(NodeId, usize)> = vec![(start, 0)];
        visited[start.index()] = true;
        while let Some(&(node, idx)) = stack.last() {
            let neighbors = g.neighbors(node);
            if idx < neighbors.len() {
                let next = neighbors[idx].to;
                stack.last_mut().unwrap().1 = idx + 1;
                if !visited[next.index()] {
                    visited[next.index()] = true;
                    stack.push((next, 0));
                }
            } else {
                order.push(node);
                stack.pop();
            }
        }
    }
    // Pass 2: DFS on the transpose in reverse finish order.
    let t = g.transpose();
    let mut comp_of = vec![usize::MAX; n];
    let mut comps: Vec<Vec<NodeId>> = Vec::new();
    while let Some(seed) = order.pop() {
        if comp_of[seed.index()] != usize::MAX {
            continue;
        }
        let cid = comps.len();
        comps.push(Vec::new());
        let mut stack = vec![seed];
        while let Some(node) = stack.pop() {
            if comp_of[node.index()] != usize::MAX {
                continue;
            }
            comp_of[node.index()] = cid;
            comps[cid].push(node);
            for e in t.neighbors(node) {
                if comp_of[e.to.index()] == usize::MAX {
                    stack.push(e.to);
                }
            }
        }
    }
    comps.sort_by_key(|c| c.iter().min().copied().unwrap_or(NodeId(0)));
    comps
}

/// Provable contract — referenced from `contracts/graph-algorithms-v1.yaml`.
///
/// Every node in the graph appears in exactly one component.
pub fn assert_components_partition(g: &Graph, comps: &[Vec<NodeId>]) {
    let n = g.node_count();
    let mut seen = vec![false; n];
    for c in comps {
        for node in c {
            assert!(
                !seen[node.index()],
                "node {:?} appears in two components",
                node
            );
            seen[node.index()] = true;
        }
    }
    for (i, s) in seen.iter().enumerate() {
        assert!(*s, "node {} missing from any component", i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::Edge;

    #[test]
    fn components_single_node() {
        let g = Graph::with_capacity(1);
        assert_eq!(connected_components(&g), vec![vec![NodeId(0)]]);
    }

    #[test]
    fn components_two_islands() {
        // {0,1} and {2,3} as two undirected components.
        let mut g = Graph::with_capacity(4);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(2), NodeId(3), 1))
            .unwrap();
        let c = connected_components(&g);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], vec![NodeId(0), NodeId(1)]);
        assert_eq!(c[1], vec![NodeId(2), NodeId(3)]);
    }

    #[test]
    fn components_directed_link_unifies() {
        // 0 → 1, 2 → 1. Forward DFS finds {0,1} and {2}, but the transpose
        // pass merges them into one weakly-connected component.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(1), 1)).unwrap();
        let c = connected_components(&g);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0], vec![NodeId(0), NodeId(1), NodeId(2)]);
    }

    #[test]
    fn components_revisit_via_stack() {
        // Triangle — the second push of a visited node must short-circuit.
        let mut g = Graph::with_capacity(3);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(1), NodeId(2), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(0), NodeId(2), 1))
            .unwrap();
        let c = connected_components(&g);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].len(), 3);
    }

    #[test]
    fn kosaraju_acyclic_three_components() {
        // 0 → 1 → 2 in a DAG: every node is its own SCC.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        let s = kosaraju(&g);
        assert_eq!(s.len(), 3);
        for c in &s {
            assert_eq!(c.len(), 1);
        }
    }

    #[test]
    fn kosaraju_cycle_one_component() {
        // 0 → 1 → 2 → 0: one SCC.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(0), 1)).unwrap();
        let s = kosaraju(&g);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].len(), 3);
    }

    #[test]
    fn kosaraju_two_sccs_chained() {
        // {0 ↔ 1} → {2 ↔ 3}: two SCCs with a bridge edge.
        let mut g = Graph::with_capacity(4);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(0), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(3), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(3), NodeId(2), 1)).unwrap();
        let s = kosaraju(&g);
        assert_eq!(s.len(), 2);
        assert_components_partition(&g, &s);
    }

    #[test]
    fn kosaraju_triangle_with_chord_one_scc() {
        // 0→1, 0→2, 1→0, 1→2, 2→0 — all one SCC. The transpose has
        // multiple paths to node 1 (from 0 and from 2), so node 1 ends
        // up on the stack twice and the second pop must short-circuit
        // via the assigned-already check.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(0), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(0), 1)).unwrap();
        let s = kosaraju(&g);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].len(), 3);
    }

    #[test]
    fn kosaraju_isolated_nodes() {
        let g = Graph::with_capacity(3);
        let s = kosaraju(&g);
        assert_eq!(s.len(), 3);
        for c in &s {
            assert_eq!(c.len(), 1);
        }
    }

    #[test]
    fn assert_partition_passes_on_kosaraju() {
        let mut g = Graph::with_capacity(2);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(0), 1)).unwrap();
        let s = kosaraju(&g);
        assert_components_partition(&g, &s);
    }

    #[test]
    #[should_panic(expected = "appears in two components")]
    fn assert_partition_catches_duplicate() {
        let mut g = Graph::with_capacity(2);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        let bad = vec![vec![NodeId(0)], vec![NodeId(0), NodeId(1)]];
        assert_components_partition(&g, &bad);
    }

    #[test]
    #[should_panic(expected = "missing from any component")]
    fn assert_partition_catches_missing() {
        let g = Graph::with_capacity(3);
        let bad = vec![vec![NodeId(0)], vec![NodeId(1)]]; // missing 2
        assert_components_partition(&g, &bad);
    }
}
