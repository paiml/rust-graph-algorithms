# rust-graph-algorithms

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange.svg)](rust-toolchain.toml)

Reference Rust workspace for course **c12 — Graph Algorithms with Rust** in the Coursera
[Rust for Data Engineering](https://www.coursera.org/) specialization.

Cache-friendly Rust implementations of classical graph algorithms — BFS/DFS, Dijkstra, Bellman-Ford, betweenness/closeness/PageRank centrality, Kosaraju and Louvain community detection, plus visualization. Anchored on `aprender-graph` and `petgraph`.

## Workspace layout

- [`crates/graph-core`](crates/graph-core) —  NodeId, Edge<W>, Graph, IO
- [`crates/graph-traversal`](crates/graph-traversal) — BFS, DFS, Dijkstra, Bellman-Ford
- [`crates/graph-centrality`](crates/graph-centrality) — Degree, betweenness, closeness, PageRank
- [`crates/graph-community`](crates/graph-community) — Connected components, Kosaraju SCC, Louvain
- [`crates/graph-viz`](crates/graph-viz) — Force-directed layout and DOT/SVG export
- [`crates/graph-cli`](crates/graph-cli) — clap binary exposing each algorithm as a subcommand

## Quick start

```bash
git clone https://github.com/paiml/rust-graph-algorithms
cd rust-graph-algorithms
cargo test --workspace
```

## Status

Scaffold. Lessons land as recordings ship. Track companion config at
[`paiml/course-studio`](https://github.com/paiml/course-studio).

## License

Dual-licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
