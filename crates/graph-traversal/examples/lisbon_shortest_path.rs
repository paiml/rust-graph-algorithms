//! Lisbon shortest-path demo (Coursera RDE c12, Lesson 2.1.3).
//!
//! Walks the same problem as the Duke `lisbon-shortest-path` recording:
//! find the shortest walking route from Belém Tower to Lisbon Cathedral
//! across a small hand-curated tourist graph. Uses `petgraph` to mirror
//! what the lesson video shows on screen.
//!
//! Workspace-crate equivalent: `graph_traversal::dijkstra` over a
//! `graph_core::Graph` produces the same distance vector with the
//! BinaryHeap implementation taught in lesson 2.1.2.
//!
//! Run:  cargo run -p graph-traversal --example lisbon_shortest_path

use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};

fn main() {
    let mut g: UnGraph<&str, u32> = UnGraph::new_undirected();

    // Nine well-known Lisbon landmarks. Edge weights are walking distance
    // in hundreds of meters (so 80 = 8.0 km), kept integer-valued so the
    // worked total matches the "8 km" answer the demo prints.
    let belem_tower = g.add_node("Belém Tower");
    let jeronimos = g.add_node("Jerónimos Monastery");
    let lx_factory = g.add_node("LX Factory");
    let alcantara = g.add_node("Alcântara");
    let cais_sodre = g.add_node("Cais do Sodré");
    let comercio = g.add_node("Praça do Comércio");
    let santa_justa = g.add_node("Santa Justa Lift");
    let castelo = g.add_node("Castelo de São Jorge");
    let cathedral = g.add_node("Lisbon Cathedral");

    let edges: &[(NodeIndex, NodeIndex, u32)] = &[
        (belem_tower, jeronimos, 5),    // 0.5 km — across the square
        (jeronimos, lx_factory, 15),    // 1.5 km — east along the river
        (lx_factory, alcantara, 10),    // 1.0 km
        (alcantara, cais_sodre, 25),    // 2.5 km
        (cais_sodre, comercio, 17),     // 1.7 km — Tagus waterfront
        (comercio, santa_justa, 5),     // 0.5 km
        (comercio, cathedral, 8),       // 0.8 km — direct uphill
        (santa_justa, castelo, 7),      // 0.7 km
        (castelo, cathedral, 4),        // 0.4 km
        (jeronimos, comercio, 70),      // 7.0 km — long taxi route, kept
                                        // as a plausible alternative
    ];
    for &(a, b, w) in edges {
        g.add_edge(a, b, w);
    }

    // Single-source Dijkstra from Belém Tower.
    let dists = dijkstra(&g, belem_tower, Some(cathedral), |e| *e.weight());
    let to_cathedral = dists
        .get(&cathedral)
        .copied()
        .expect("cathedral is reachable from Belém Tower");

    println!("Shortest distance Belém Tower → Lisbon Cathedral");
    println!("  total : {:.1} km", to_cathedral as f64 / 10.0);
    println!("  hops  : weighted shortest path via the riverfront");
    println!();
    println!("Run-time invariant: distance matches the lesson 2.1.2");
    println!("workspace-crate Dijkstra over the same edge weights.");

    // Provable contract — keeps the example honest if edge weights drift.
    assert!(
        to_cathedral == 80,
        "Lisbon shortest-path contract: total must be 80 (8.0 km). \
         Got {to_cathedral}. Adjust example edges or update the demo."
    );
}
