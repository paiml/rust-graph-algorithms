# rust-graph-algorithms

[![CI](https://github.com/paiml/rust-graph-algorithms/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/rust-graph-algorithms/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.95-orange.svg)](rust-toolchain.toml)
[![Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen.svg)](Makefile)

Reference Rust workspace for course **c12 — Graph Algorithms with
Rust** in the Coursera Rust for Data Engineering specialization.

Cache-friendly Rust implementations of classical graph algorithms —
BFS, DFS, Dijkstra, degree and PageRank centrality, weakly-connected
components, Kosaraju SCC — plus Graphviz DOT export and a single-binary
CLI (`graph`) that runs each algorithm against JSON input on stdin.

Three runtime contracts are asserted on every successful CLI run:
edge-count consistency, PageRank normalization, and component
partition. Specs in
[`contracts/graph-algorithms-v1.yaml`](contracts/graph-algorithms-v1.yaml),
linted by `pv lint contracts/` in CI.

## Workspace layout

- [`crates/graph-core`](crates/graph-core) — `NodeId`, `Edge`, `Graph`,
  `GraphError`, runtime invariant `assert_edge_count_consistent`
- [`crates/graph-traversal`](crates/graph-traversal) — BFS, DFS,
  Dijkstra, runtime invariant `assert_bfs_distance_monotonic`
- [`crates/graph-centrality`](crates/graph-centrality) — out-degree
  centrality, PageRank power iteration, runtime invariant
  `assert_pagerank_normalized`
- [`crates/graph-community`](crates/graph-community) —
  weakly-connected components and Kosaraju SCC, runtime invariant
  `assert_components_partition`
- [`crates/graph-viz`](crates/graph-viz) — Graphviz DOT export
- [`crates/graph-cli`](crates/graph-cli) — `graph` binary that wires
  all five algorithm crates behind a clap subcommand interface

## Install

```bash
git clone https://github.com/paiml/rust-graph-algorithms
cd rust-graph-algorithms
cargo build --release --workspace
./target/release/graph --help
```

## Quick start

```bash
# Run BFS on a tiny triangle.
echo '{"nodes":3,"edges":[
  {"from":0,"to":1,"weight":1},
  {"from":1,"to":2,"weight":1},
  {"from":2,"to":0,"weight":1}
]}' | cargo run -p graph-cli -q -- bfs --source 0
# {"kind":"bfs","order":[0,1,2]}
```

The same JSON input drives every subcommand — `bfs`, `dfs`,
`dijkstra`, `pagerank`, `components`, `scc`, `dot`. Run `graph --help`
for the full list.

## Local CI gate

Mirror what `gate` runs in CI:

```bash
make ship-ready
```

That runs format-check, clippy, doc, test, doc-test, **100% line
coverage** (`cargo llvm-cov --fail-under-lines 100`), `cargo audit`,
`cargo deny`, contract lint, Makefile lint, and `pmat comply`.

## Provable contracts

Three structural invariants in
[`contracts/graph-algorithms-v1.yaml`](contracts/graph-algorithms-v1.yaml):

| Equation | Statement |
|----------|-----------|
| `edge_count_consistent` | sum of neighbor list lengths equals edge count |
| `pagerank_normalized` | absolute deviation of score sum from 1.0 below 1e-3 |
| `components_partition` | components form a partition of the node set |

All three are asserted at runtime in
[`crates/graph-cli/src/lib.rs`](crates/graph-cli/src/lib.rs) after
every successful CLI invocation, and the YAML is linted by `pv` in CI.

## Status

71 tests / 100% line coverage / 0 clippy warnings / pmat comply
COMPLIANT / pv lint PASS.

## License

Dual-licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
