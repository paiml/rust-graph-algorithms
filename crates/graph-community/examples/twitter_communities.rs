//! Twitter community detection (Coursera RDE c12, Lesson 4.1.3).
//!
//! Walks the same problem as the Duke `community-detection` recording:
//! model a small directed follower graph mixing real troll accounts
//! (Neo4j/NBC News dataset) with a hypothetical journalist clique, then
//! run Kosaraju to surface strongly-connected communities. Uses
//! `petgraph` to mirror what the lesson video shows on screen.
//!
//! Workspace-crate equivalent: `graph_community::kosaraju` over a
//! `graph_core::Graph` produces the same SCC partition via the
//! two-DFS-pass algorithm taught in lesson 4.1.2.
//!
//! Run:  cargo run -p graph-community --example twitter_communities

use petgraph::algo::kosaraju_scc;
use petgraph::graph::DiGraph;

fn main() {
    let mut g: DiGraph<&str, ()> = DiGraph::new();

    // Hypothetical journalist clique — three accounts that all follow
    // one another (mutual follows form a strongly-connected triangle).
    let journalist1 = g.add_node("journalist1");
    let journalist2 = g.add_node("journalist2");
    let journalist3 = g.add_node("journalist3");

    // Sample handles standing in for the Neo4j troll-account set the
    // lesson references; mutual follows form the second SCC.
    let troll1 = g.add_node("troll_alpha");
    let troll2 = g.add_node("troll_bravo");
    let troll3 = g.add_node("troll_charlie");

    // A handful of one-way follows from outside the cliques — these stay
    // their own singleton SCCs and prove that Kosaraju is not just
    // counting weakly-connected components.
    let lurker = g.add_node("news_lurker");
    let bot = g.add_node("retweet_bot");

    let follows = [
        // Journalist clique (3 mutual follows = SCC of 3).
        (journalist1, journalist2),
        (journalist2, journalist3),
        (journalist3, journalist1),
        // Troll clique (3 mutual follows = SCC of 3).
        (troll1, troll2),
        (troll2, troll3),
        (troll3, troll1),
        // One-way bridges that link cliques without merging them.
        (lurker, journalist1),
        (lurker, troll1),
        (bot, troll2),
    ];
    for (src, dst) in follows {
        g.add_edge(src, dst, ());
    }

    let sccs = kosaraju_scc(&g);

    println!("Kosaraju strongly-connected components on the follower graph");
    for (i, scc) in sccs.iter().enumerate() {
        let names: Vec<&str> = scc
            .iter()
            .map(|&i| *g.node_weight(i).expect("node weight present"))
            .collect();
        println!(
            "  SCC {} ({} nodes): {}",
            i + 1,
            names.len(),
            names.join(", ")
        );
    }

    // Provable contract: exactly two non-trivial communities (size ≥ 2).
    let big = sccs.iter().filter(|c| c.len() >= 2).count();
    assert!(
        big == 2,
        "Community-detection contract: expected exactly 2 mutual-follow \
         communities (journalists + trolls). Got {big}."
    );
    let three_node = sccs.iter().filter(|c| c.len() == 3).count();
    assert!(
        three_node == 2,
        "Community-detection contract: both communities must have 3 \
         members. Got {three_node} three-node SCCs."
    );
}
