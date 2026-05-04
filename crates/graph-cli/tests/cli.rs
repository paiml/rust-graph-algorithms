//! End-to-end CLI tests.
//!
//! Exercises the binary via [`assert_cmd`] so `main.rs` shows up in
//! workspace coverage. Per-subcommand correctness is unit-tested in
//! `lib.rs`; this file only covers the binary shell — successful exit,
//! error exit, and stdin/stdout plumbing.

use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    Command::cargo_bin("graph").unwrap()
}

const TRIANGLE: &str = r#"{"nodes":3,"edges":[
    {"from":0,"to":1,"weight":1},
    {"from":1,"to":2,"weight":1},
    {"from":2,"to":0,"weight":1}
]}"#;

#[test]
fn bfs_succeeds() {
    cmd()
        .args(["bfs", "--source", "0"])
        .write_stdin(TRIANGLE)
        .assert()
        .success()
        .stdout(contains("\"bfs\""));
}

#[test]
fn malformed_input_fails() {
    cmd()
        .args(["bfs"])
        .write_stdin("not json")
        .assert()
        .failure()
        .stderr(contains("graph:"));
}
