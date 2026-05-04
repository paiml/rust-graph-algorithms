//! Real end-to-end run: parse the bundled YAML and shell out to
//! `pmat query` per obligation. Requires `pmat` on PATH.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example discover -p aprender-contracts-demo
//! ```

use anyhow::{Context, Result};
use aprender_contracts_demo::{discover_bindings_with, parse_demo_contract};
use std::process::Command;

fn pmat_query_runner(term: &str) -> Result<Vec<String>> {
    let out = Command::new("pmat")
        .args(["query", term, "--limit", "5", "--format", "json"])
        .output()
        .context("spawn pmat query — is pmat on PATH?")?;
    if !out.status.success() {
        anyhow::bail!(
            "pmat query failed (exit {}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    parse_pmat_function_names(&stdout)
}

/// Parse `pmat query --format json` output and return the function-name
/// field of every result. The output format may vary across pmat
/// versions, so we accept any JSON shape that has a top-level array
/// (or `{"results":[...]}`) of objects with a `name` or `function` key.
fn parse_pmat_function_names(stdout: &str) -> Result<Vec<String>> {
    let v: serde_json::Value = serde_json::from_str(stdout).context("parse pmat json")?;
    let results = match &v {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(map) => match map.get("results") {
            Some(serde_json::Value::Array(arr)) => arr.clone(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    let mut names = Vec::new();
    for r in results {
        // pmat 3.16 emits `function_name`; older versions used
        // `name`/`function`. Accept any.
        let name = r
            .get("function_name")
            .or_else(|| r.get("name"))
            .or_else(|| r.get("function"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if let Some(n) = name {
            names.push(n);
        }
    }
    Ok(names)
}

fn main() -> Result<()> {
    let contract = parse_demo_contract()?;
    let report = discover_bindings_with(&contract, pmat_query_runner)?;
    println!("contract version  : {}", report.contract_version);
    println!("obligation count  : {}", report.obligation_count);
    println!("bindings          : {}", report.bindings.len());
    for (i, b) in report.bindings.iter().enumerate() {
        println!("  [{}] term: {}", i + 1, b.term);
        if b.matches.is_empty() {
            println!("      matches: (none)");
        } else {
            for m in &b.matches {
                println!("      → {}", m);
            }
        }
    }
    Ok(())
}
