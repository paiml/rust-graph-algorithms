//! PageRank on a sports-site graph (Coursera RDE c12, Lesson 3.1.5).
//!
//! Walks the same problem as the Duke `pagerank` recording: model a small
//! directed graph of sports websites that link to each other, run
//! PageRank with damping 0.85, and print the ranked scores. Uses
//! `petgraph::algo::page_rank` to mirror what the lesson video shows.
//!
//! Workspace-crate equivalent: `graph_centrality::pagerank` over a
//! `graph_core::Graph` produces an equivalent score vector via the
//! power-iteration formulation taught in lesson 3.1.4.
//!
//! Run:  cargo run -p graph-centrality --example pagerank_sports

use petgraph::algo::page_rank;
use petgraph::graph::DiGraph;

fn main() {
    let mut g: DiGraph<&str, ()> = DiGraph::new();

    // Six sports-related sites. The directed edges are "this page links
    // to that page" — the same shape PageRank was originally designed to
    // score. Shape lifted from the Duke `pagerank` lesson script.
    let espn = g.add_node("ESPN");
    let nfl = g.add_node("NFL");
    let nba = g.add_node("NBA");
    let mlb = g.add_node("MLB");
    let ufc = g.add_node("UFC");
    let uspn = g.add_node("USPN");

    let edges = [
        (espn, nfl),
        (espn, nba),
        (espn, ufc),
        (nfl, espn),
        (nfl, mlb),
        (nba, espn),
        (nba, uspn),
        (nba, ufc),
        (mlb, espn),
        (mlb, nfl),
        (ufc, espn),
        (uspn, nba),
    ];
    for (src, dst) in edges {
        g.add_edge(src, dst, ());
    }

    // 0.85 damping and a tight epsilon — the textbook defaults from the
    // Brin/Page paper that the lesson 3.1.4 animation derives.
    let scores = page_rank(&g, 0.85_f64, 100);

    let mut ranked: Vec<(&&str, f64)> = g
        .node_indices()
        .map(|i| (g.node_weight(i).expect("node weight present"), scores[i.index()]))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("scores are finite"));

    println!("PageRank on the sports-site graph (damping = 0.85)");
    for (name, score) in &ranked {
        println!("  {name:8} {score:.4}");
    }

    // Provable contract: PageRank scores must sum to 1.0 (within tol).
    let total: f64 = scores.iter().sum();
    assert!(
        (total - 1.0).abs() < 1e-3,
        "PageRank normalization contract: sum(scores) must be ≈1.0. \
         Got sum = {total:.6}."
    );
}
