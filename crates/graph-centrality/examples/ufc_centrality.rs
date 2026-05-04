//! UFC graph-centrality demo (Coursera RDE c12, Lesson 3.1.6).
//!
//! Walks the same problem as the Duke `graph-centrality-ufc` recording:
//! model UFC fighters as nodes, individual fights as undirected edges,
//! and rank fighters by degree centrality. Uses `petgraph` to mirror
//! what the lesson video shows on screen — including the `Fighter`
//! struct with a `Display` impl that the SRT calls out by name.
//!
//! Workspace-crate equivalent: `graph_centrality::out_degree_centrality`
//! over a `graph_core::Graph` produces a directed-edge analog; degree
//! centrality on undirected graphs is just the neighbor count.
//!
//! Run:  cargo run -p graph-centrality --example ufc_centrality

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::Direction;
use std::fmt;

#[derive(Debug, Clone)]
struct Fighter {
    name: &'static str,
}

impl fmt::Display for Fighter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn main() {
    let mut g: UnGraph<Fighter, ()> = UnGraph::new_undirected();

    // Five well-known UFC fighters and the seven bouts the Duke lesson
    // walks through on screen.
    let poirier = g.add_node(Fighter { name: "Dustin Poirier" });
    let khabib = g.add_node(Fighter { name: "Khabib Nurmagomedov" });
    let mcgregor = g.add_node(Fighter { name: "Conor McGregor" });
    let aldo = g.add_node(Fighter { name: "Jose Aldo" });
    let diaz = g.add_node(Fighter { name: "Nate Diaz" });

    let fights: &[(NodeIndex, NodeIndex)] = &[
        (poirier, khabib),
        (khabib, mcgregor),
        (mcgregor, poirier),
        (mcgregor, aldo),
        (mcgregor, diaz),
        (poirier, diaz),
        (aldo, diaz),
    ];
    for &(a, b) in fights {
        g.add_edge(a, b, ());
    }

    // Degree centrality on an undirected graph = neighbor count.
    let mut ranked: Vec<(&Fighter, usize)> = g
        .node_indices()
        .map(|i| {
            let degree = g.neighbors_directed(i, Direction::Outgoing).count();
            (g.node_weight(i).expect("node weight present"), degree)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.name.cmp(b.0.name)));

    println!("UFC degree centrality — most-connected fighters");
    for (fighter, degree) in &ranked {
        println!("  {fighter:24} {degree}");
    }

    // Provable contract: McGregor is in 4 of the 7 bouts, so his degree
    // is 4 — the highest in the graph. Drift means the edge list moved.
    let mcgregor_degree = g.neighbors_directed(mcgregor, Direction::Outgoing).count();
    assert!(
        mcgregor_degree == 4,
        "UFC centrality contract: McGregor must have degree 4. \
         Got {mcgregor_degree}. Edge list drifted from the lesson script."
    );
}
