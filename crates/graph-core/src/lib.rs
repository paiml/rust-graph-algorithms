//! Shared graph types for the rust-graph-algorithms workspace.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**.
//!
//! Three primitives:
//! - [`NodeId`] — newtype around `u32` so node ids and edge weights cannot
//!   be confused at compile time
//! - [`Edge`] — directed edge `from → to` carrying a `u32` weight
//! - [`Graph`] — adjacency-list graph with directed and undirected edge
//!   constructors and a [`Graph::transpose`] for SCC algorithms

#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A node identifier. Newtyped so it cannot be confused with edge weights
/// or arbitrary indices at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    /// Construct a [`NodeId`] from a `u32`.
    pub const fn new(id: u32) -> Self {
        NodeId(id)
    }

    /// Convert to a `usize` for indexing the adjacency vector.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A directed, weighted edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Source node.
    pub from: NodeId,
    /// Destination node.
    pub to: NodeId,
    /// Non-negative integer weight. Algorithms requiring non-negative
    /// weights (Dijkstra) document the requirement; types alone don't
    /// enforce it.
    pub weight: u32,
}

impl Edge {
    /// Build a new edge.
    pub const fn new(from: NodeId, to: NodeId, weight: u32) -> Self {
        Edge { from, to, weight }
    }
}

/// Adjacency-list directed graph. Use [`Graph::add_undirected`] to record
/// an edge in both directions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    adj: Vec<Vec<Edge>>,
    edge_count: usize,
}

/// Errors produced by graph constructors.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GraphError {
    /// An edge referenced a node id that the graph does not contain.
    #[error("node id {0} out of range (graph has {1} nodes)")]
    NodeOutOfRange(u32, usize),
}

impl Graph {
    /// Allocate a graph with `n` nodes and zero edges.
    pub fn with_capacity(n: usize) -> Self {
        Graph {
            adj: vec![Vec::new(); n],
            edge_count: 0,
        }
    }

    /// Build a directed graph from `n` nodes and a list of edges.
    pub fn from_edges(n: usize, edges: &[Edge]) -> Result<Self, GraphError> {
        let mut g = Self::with_capacity(n);
        for e in edges {
            g.add_directed(*e)?;
        }
        Ok(g)
    }

    /// Add a single directed edge.
    pub fn add_directed(&mut self, e: Edge) -> Result<(), GraphError> {
        let n = self.adj.len();
        if e.from.index() >= n {
            return Err(GraphError::NodeOutOfRange(e.from.0, n));
        }
        if e.to.index() >= n {
            return Err(GraphError::NodeOutOfRange(e.to.0, n));
        }
        self.adj[e.from.index()].push(e);
        self.edge_count += 1;
        Ok(())
    }

    /// Add an undirected edge — internally stored as two directed edges.
    pub fn add_undirected(&mut self, e: Edge) -> Result<(), GraphError> {
        self.add_directed(e)?;
        self.add_directed(Edge::new(e.to, e.from, e.weight))
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.adj.len()
    }

    /// Number of directed edges. An undirected edge counts as two.
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// Neighbors of a node — slice of outgoing edges.
    pub fn neighbors(&self, n: NodeId) -> &[Edge] {
        &self.adj[n.index()]
    }

    /// Iterator over all node ids.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        (0..self.adj.len() as u32).map(NodeId)
    }

    /// Build the transposed graph: every directed edge `u → v` becomes
    /// `v → u`. Required for the second pass of Kosaraju's SCC algorithm.
    pub fn transpose(&self) -> Self {
        let mut g = Self::with_capacity(self.node_count());
        for src in self.nodes() {
            for e in self.neighbors(src) {
                g.adj[e.to.index()].push(Edge::new(e.to, e.from, e.weight));
                g.edge_count += 1;
            }
        }
        g
    }
}

/// Provable contract — referenced from `contracts/graph-algorithms-v1.yaml`.
///
/// Sum of every node's neighbor list equals the recorded edge count.
/// Asserted at runtime in `graph-cli` after every successful command so
/// the invariant is exercised on real inputs and not just unit tests.
pub fn assert_edge_count_consistent(g: &Graph) {
    let total: usize = g.nodes().map(|n| g.neighbors(n).len()).sum();
    assert_eq!(total, g.edge_count());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_constructors() {
        assert_eq!(NodeId::new(7).0, 7);
        assert_eq!(NodeId(7).index(), 7);
    }

    #[test]
    fn node_id_ordering() {
        assert!(NodeId(1) < NodeId(2));
    }

    #[test]
    fn edge_new() {
        let e = Edge::new(NodeId(0), NodeId(1), 5);
        assert_eq!(e.from, NodeId(0));
        assert_eq!(e.to, NodeId(1));
        assert_eq!(e.weight, 5);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_capacity(3);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 0);
        assert!(g.neighbors(NodeId(1)).is_empty());
    }

    #[test]
    fn from_edges_directed() {
        let edges = [
            Edge::new(NodeId(0), NodeId(1), 1),
            Edge::new(NodeId(1), NodeId(2), 2),
        ];
        let g = Graph::from_edges(3, &edges).unwrap();
        assert_eq!(g.edge_count(), 2);
        assert_eq!(g.neighbors(NodeId(0)).len(), 1);
        assert_eq!(g.neighbors(NodeId(1)).len(), 1);
        assert_eq!(g.neighbors(NodeId(2)).len(), 0);
    }

    #[test]
    fn from_edges_to_out_of_range() {
        let edges = [Edge::new(NodeId(0), NodeId(5), 1)];
        let err = Graph::from_edges(3, &edges).unwrap_err();
        assert_eq!(err, GraphError::NodeOutOfRange(5, 3));
    }

    #[test]
    fn add_directed_from_out_of_range() {
        let mut g = Graph::with_capacity(2);
        let err = g
            .add_directed(Edge::new(NodeId(5), NodeId(0), 1))
            .unwrap_err();
        assert_eq!(err, GraphError::NodeOutOfRange(5, 2));
    }

    #[test]
    fn add_undirected_creates_two_edges() {
        let mut g = Graph::with_capacity(2);
        g.add_undirected(Edge::new(NodeId(0), NodeId(1), 3))
            .unwrap();
        assert_eq!(g.edge_count(), 2);
        assert_eq!(g.neighbors(NodeId(0))[0].to, NodeId(1));
        assert_eq!(g.neighbors(NodeId(1))[0].to, NodeId(0));
    }

    #[test]
    fn add_undirected_propagates_error() {
        let mut g = Graph::with_capacity(2);
        let err = g
            .add_undirected(Edge::new(NodeId(0), NodeId(9), 1))
            .unwrap_err();
        assert_eq!(err, GraphError::NodeOutOfRange(9, 2));
    }

    #[test]
    fn nodes_iterates() {
        let g = Graph::with_capacity(4);
        let ns: Vec<NodeId> = g.nodes().collect();
        assert_eq!(ns, vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)]);
    }

    #[test]
    fn transpose_reverses_edges() {
        let edges = [
            Edge::new(NodeId(0), NodeId(1), 1),
            Edge::new(NodeId(1), NodeId(2), 2),
        ];
        let g = Graph::from_edges(3, &edges).unwrap();
        let t = g.transpose();
        assert_eq!(t.edge_count(), 2);
        assert_eq!(t.neighbors(NodeId(1))[0].to, NodeId(0));
        assert_eq!(t.neighbors(NodeId(2))[0].to, NodeId(1));
    }

    #[test]
    fn graph_error_display() {
        let e = GraphError::NodeOutOfRange(7, 3);
        assert_eq!(e.to_string(), "node id 7 out of range (graph has 3 nodes)");
    }

    #[test]
    fn graph_error_clone_debug() {
        let e = GraphError::NodeOutOfRange(7, 3);
        let _ = format!("{e:?}");
        let _ = e.clone();
    }

    #[test]
    fn graph_clone_eq_debug() {
        let g = Graph::with_capacity(3);
        let h = g.clone();
        assert_eq!(g, h);
        let _ = format!("{g:?}");
    }

    #[test]
    fn edge_clone_debug() {
        let e = Edge::new(NodeId(0), NodeId(1), 1);
        let _ = format!("{e:?}");
        assert_eq!(e, e);
    }

    #[test]
    fn node_id_serde_roundtrip() {
        let n = NodeId(42);
        let s = serde_json::to_string(&n).unwrap();
        let back: NodeId = serde_json::from_str(&s).unwrap();
        assert_eq!(n, back);
    }

    #[test]
    fn edge_serde_roundtrip() {
        let e = Edge::new(NodeId(1), NodeId(2), 3);
        let s = serde_json::to_string(&e).unwrap();
        let back: Edge = serde_json::from_str(&s).unwrap();
        assert_eq!(e, back);
    }

    #[test]
    fn graph_serde_roundtrip() {
        let edges = [Edge::new(NodeId(0), NodeId(1), 1)];
        let g = Graph::from_edges(2, &edges).unwrap();
        let s = serde_json::to_string(&g).unwrap();
        let back: Graph = serde_json::from_str(&s).unwrap();
        assert_eq!(g, back);
    }

    #[test]
    fn assert_edge_count_consistent_passes() {
        let edges = [Edge::new(NodeId(0), NodeId(1), 1)];
        let g = Graph::from_edges(2, &edges).unwrap();
        assert_edge_count_consistent(&g);
    }
}
