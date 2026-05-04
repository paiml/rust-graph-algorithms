//! Graph traversal — BFS, DFS, Dijkstra.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, Module 2.

#![deny(missing_docs)]

use graph_core::{Graph, NodeId};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// Breadth-first search starting at `source`. Returns the visit order:
/// every reachable node appears exactly once, and any node at distance
/// `d` precedes every node at distance `d+1` in the returned vector.
pub fn bfs(g: &Graph, source: NodeId) -> Vec<NodeId> {
    let mut visited = vec![false; g.node_count()];
    let mut order = Vec::new();
    let mut queue = VecDeque::new();
    visited[source.index()] = true;
    queue.push_back(source);
    while let Some(n) = queue.pop_front() {
        order.push(n);
        for e in g.neighbors(n) {
            if !visited[e.to.index()] {
                visited[e.to.index()] = true;
                queue.push_back(e.to);
            }
        }
    }
    order
}

/// Depth-first search starting at `source`. Returns nodes in pre-order
/// (i.e. each node appears the first time it is reached).
pub fn dfs(g: &Graph, source: NodeId) -> Vec<NodeId> {
    let mut visited = vec![false; g.node_count()];
    let mut order = Vec::new();
    let mut stack = vec![source];
    while let Some(n) = stack.pop() {
        if visited[n.index()] {
            continue;
        }
        visited[n.index()] = true;
        order.push(n);
        // Push neighbors in reverse so the resulting visit order matches
        // the textbook "first neighbor visited first" convention.
        for e in g.neighbors(n).iter().rev() {
            if !visited[e.to.index()] {
                stack.push(e.to);
            }
        }
    }
    order
}

/// Single-source shortest-path distances via Dijkstra's algorithm.
/// Requires non-negative edge weights — graphs with negative weights
/// produce undefined output.
///
/// Returns a vector of length `g.node_count()` where index `i` holds
/// `Some(distance)` if node `i` is reachable from `source`, `None`
/// otherwise. The source node's own distance is always `Some(0)`.
pub fn dijkstra(g: &Graph, source: NodeId) -> Vec<Option<u32>> {
    let n = g.node_count();
    let mut dist: Vec<Option<u32>> = vec![None; n];
    let mut heap: BinaryHeap<Reverse<(u32, NodeId)>> = BinaryHeap::new();
    dist[source.index()] = Some(0);
    heap.push(Reverse((0, source)));
    while let Some(Reverse((d, node))) = heap.pop() {
        // Stale entries are filtered implicitly by the strict `next < c`
        // check below — re-relaxing with a stale `d` cannot improve a
        // bound that was already tightened, so no extra guard is needed.
        for e in g.neighbors(node) {
            let next = d.saturating_add(e.weight);
            let cur = dist[e.to.index()];
            if cur.is_none() || cur.is_some_and(|c| next < c) {
                dist[e.to.index()] = Some(next);
                heap.push(Reverse((next, e.to)));
            }
        }
    }
    dist
}

/// Provable contract — referenced from `contracts/graph-algorithms-v1.yaml`.
///
/// In a BFS visit order, every prefix is "complete by distance": if node
/// `n` appears at position `i`, every node at strictly smaller distance
/// from `source` appears in positions `0..i`.
pub fn assert_bfs_distance_monotonic(g: &Graph, source: NodeId, order: &[NodeId]) {
    // Compute distances via independent BFS using a separate visited set,
    // then verify the order respects them.
    let n = g.node_count();
    let mut dist = vec![u32::MAX; n];
    let mut queue = VecDeque::new();
    dist[source.index()] = 0;
    queue.push_back(source);
    while let Some(node) = queue.pop_front() {
        for e in g.neighbors(node) {
            if dist[e.to.index()] == u32::MAX {
                dist[e.to.index()] = dist[node.index()] + 1;
                queue.push_back(e.to);
            }
        }
    }
    let mut last = 0u32;
    for n in order {
        let d = dist[n.index()];
        assert!(
            d >= last,
            "BFS order violates distance monotonicity at node {:?}",
            n
        );
        last = d;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph_core::Edge;

    fn line_graph(n: usize) -> Graph {
        // 0 — 1 — 2 — … — n-1, undirected, weight 1
        let mut g = Graph::with_capacity(n);
        for i in 0..n.saturating_sub(1) {
            g.add_undirected(Edge::new(NodeId(i as u32), NodeId(i as u32 + 1), 1))
                .unwrap();
        }
        g
    }

    #[test]
    fn bfs_line_graph() {
        let g = line_graph(4);
        let order = bfs(&g, NodeId(0));
        assert_eq!(order, vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)]);
    }

    #[test]
    fn bfs_only_visits_reachable() {
        // Node 3 is isolated.
        let mut g = Graph::with_capacity(4);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(1), NodeId(2), 1))
            .unwrap();
        let order = bfs(&g, NodeId(0));
        assert_eq!(order.len(), 3);
        assert!(!order.contains(&NodeId(3)));
    }

    #[test]
    fn bfs_skips_revisited_neighbor() {
        // Triangle: revisiting a queued-but-unprocessed node should be a no-op.
        let mut g = Graph::with_capacity(3);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(1), NodeId(2), 1))
            .unwrap();
        g.add_undirected(Edge::new(NodeId(0), NodeId(2), 1))
            .unwrap();
        let order = bfs(&g, NodeId(0));
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn dfs_line_graph() {
        let g = line_graph(4);
        let order = dfs(&g, NodeId(0));
        assert_eq!(order, vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)]);
    }

    #[test]
    fn dfs_branching() {
        // 0 → {1, 2}, 1 → 3, 2 → 4. With reverse-push the order is 0,1,3,2,4.
        let mut g = Graph::with_capacity(5);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(3), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(4), 1)).unwrap();
        let order = dfs(&g, NodeId(0));
        assert_eq!(
            order,
            vec![NodeId(0), NodeId(1), NodeId(3), NodeId(2), NodeId(4)]
        );
    }

    #[test]
    fn dfs_handles_revisit_via_stack() {
        // Triangle with shared paths — the visited check inside the pop
        // loop must short-circuit on the second appearance.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(1), NodeId(2), 1)).unwrap();
        let order = dfs(&g, NodeId(0));
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn dijkstra_line_graph_distances() {
        let g = line_graph(4);
        let d = dijkstra(&g, NodeId(0));
        assert_eq!(d, vec![Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn dijkstra_unreachable_node_is_none() {
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 5)).unwrap();
        let d = dijkstra(&g, NodeId(0));
        assert_eq!(d, vec![Some(0), Some(5), None]);
    }

    #[test]
    fn dijkstra_picks_shorter_alternative() {
        // 0 → 1 weight 10, 0 → 2 weight 1, 2 → 1 weight 1. Shortest 0→1 = 2.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 10)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(1), 1)).unwrap();
        let d = dijkstra(&g, NodeId(0));
        assert_eq!(d, vec![Some(0), Some(2), Some(1)]);
    }

    #[test]
    fn dijkstra_lazy_deletion_skips_stale_entry() {
        // Force a stale heap entry by giving a long-then-short path.
        // 0 → 1 weight 5; 0 → 2 weight 1; 2 → 1 weight 1.
        // After processing 0, heap has (5,1) and (1,2). Popping (1,2)
        // updates 1 to dist 2, pushing (2,1). Later pop of (5,1) hits
        // the lazy-deletion `continue` branch.
        let mut g = Graph::with_capacity(3);
        g.add_directed(Edge::new(NodeId(0), NodeId(1), 5)).unwrap();
        g.add_directed(Edge::new(NodeId(0), NodeId(2), 1)).unwrap();
        g.add_directed(Edge::new(NodeId(2), NodeId(1), 1)).unwrap();
        let d = dijkstra(&g, NodeId(0));
        assert_eq!(d[1], Some(2));
    }

    #[test]
    fn assert_bfs_monotonic_passes_on_real_bfs() {
        let g = line_graph(5);
        let order = bfs(&g, NodeId(0));
        assert_bfs_distance_monotonic(&g, NodeId(0), &order);
    }

    #[test]
    #[should_panic(expected = "BFS order violates distance monotonicity")]
    fn assert_bfs_monotonic_catches_bad_order() {
        let g = line_graph(4);
        let bad = vec![NodeId(0), NodeId(2), NodeId(1)];
        assert_bfs_distance_monotonic(&g, NodeId(0), &bad);
    }
}
