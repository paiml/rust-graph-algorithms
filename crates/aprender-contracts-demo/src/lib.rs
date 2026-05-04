//! Working demo of `aprender-contracts` (lib name `provable_contracts`).
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, lesson 5.1.5 (verifying graph code with
//! pmat query + aprender-contracts).
//!
//! Three pure functions drive the workflow:
//!
//! - [`parse_demo_contract`] — read the bundled YAML and parse via
//!   `provable_contracts::schema::parse_contract_str`
//! - [`extract_query_terms`] — pull the keyword most likely to bind a
//!   contract obligation to a function in the codebase (the leading
//!   noun phrase of the obligation `property` field)
//! - [`discover_bindings_with`] — given a query runner closure, ask it
//!   for matching functions per term and return a [`BindingReport`]
//!
//! The shell-out to `pmat query` is in the companion example
//! ([`examples/discover.rs`](../examples/discover.rs)) so this crate's
//! library code stays pure and 100% line-coverable.

#![deny(missing_docs)]

use anyhow::{Context, Result};
use provable_contracts::schema::{parse_contract_str, Contract};

/// One discovered binding — links one contract obligation term to the
/// list of function names a query runner returned for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// The obligation property text (or its extracted keyword).
    pub term: String,
    /// Function names returned by the query runner for `term`.
    pub matches: Vec<String>,
}

/// Outcome of running a [`QueryRunner`]-style closure across every
/// obligation in a parsed contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingReport {
    /// Contract version (from `metadata.version`).
    pub contract_version: String,
    /// Number of obligations the contract declared.
    pub obligation_count: usize,
    /// One [`Binding`] per query term.
    pub bindings: Vec<Binding>,
}

/// Bundled YAML loaded at compile time from
/// `contracts/aprender-demo-v1.yaml` (relative to the workspace root).
const BUNDLED_CONTRACT_YAML: &str = include_str!("../../../contracts/aprender-demo-v1.yaml");

/// Parse the bundled aprender-demo-v1.yaml via the real
/// `provable_contracts::schema::parse_contract_str` API.
pub fn parse_demo_contract() -> Result<Contract> {
    parse_contract_yaml(BUNDLED_CONTRACT_YAML)
}

/// Parse any contract YAML string. Wrapper around
/// `provable_contracts::schema::parse_contract_str` that converts the
/// crate-specific error into `anyhow::Error` and adds context.
pub fn parse_contract_yaml(yaml: &str) -> Result<Contract> {
    parse_contract_str(yaml)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("parse contract YAML")
}

/// Extract one query term per proof obligation. We use the obligation's
/// `property` field as-is; the runner is responsible for tokenizing /
/// keyword-extracting if it wants narrower matches.
///
/// Empty properties are skipped — they carry no information.
pub fn extract_query_terms(contract: &Contract) -> Vec<String> {
    contract
        .proof_obligations
        .iter()
        .filter_map(|o| {
            let p = o.property.trim();
            if p.is_empty() {
                None
            } else {
                Some(p.to_string())
            }
        })
        .collect()
}

/// Run the query closure across every term and assemble a [`BindingReport`].
///
/// The closure is the pluggable boundary: production code passes a
/// closure that shells out to `pmat query`; tests pass a closure that
/// returns canned data.
pub fn discover_bindings_with<F>(contract: &Contract, mut f: F) -> Result<BindingReport>
where
    F: FnMut(&str) -> Result<Vec<String>>,
{
    let terms = extract_query_terms(contract);
    let mut bindings = Vec::with_capacity(terms.len());
    for term in &terms {
        let matches = f(term).with_context(|| format!("query for {term:?}"))?;
        bindings.push(Binding {
            term: term.clone(),
            matches,
        });
    }
    Ok(BindingReport {
        contract_version: contract.metadata.version.clone(),
        obligation_count: contract.proof_obligations.len(),
        bindings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_contract_parses_cleanly() {
        let c = parse_demo_contract().unwrap();
        assert_eq!(c.metadata.version, "1.0.0");
        // The aprender-demo contract declares three obligations.
        assert_eq!(c.proof_obligations.len(), 3);
    }

    #[test]
    fn extract_query_terms_returns_one_per_obligation() {
        let c = parse_demo_contract().unwrap();
        let terms = extract_query_terms(&c);
        assert_eq!(terms.len(), 3);
        // The obligations include a PageRank claim — its property text
        // mentions "PageRank" verbatim, which is what binds to the
        // matching function in graph-centrality.
        assert!(terms.iter().any(|t| t.contains("PageRank")));
    }

    #[test]
    fn extract_query_terms_skips_empty_properties() {
        // Build a contract YAML with one empty-property obligation
        // alongside two real ones, and confirm the empty one is dropped.
        let yaml = r#"
metadata:
  version: '1.0.0'
  description: 'test'
equations: {}
proof_obligations:
  - type: invariant
    property: 'real claim'
    formal: 'x = 1'
    applies_to: all
  - type: invariant
    property: ''
    formal: 'y = 2'
    applies_to: all
  - type: postcondition
    property: '   '
    formal: 'z = 3'
    applies_to: all
enforcement: {}
"#;
        let c = parse_contract_str(yaml).unwrap();
        let terms = extract_query_terms(&c);
        assert_eq!(terms, vec!["real claim".to_string()]);
    }

    #[test]
    fn discover_bindings_calls_runner_per_term() {
        let c = parse_demo_contract().unwrap();
        let mut calls = 0;
        let report = discover_bindings_with(&c, |t| {
            calls += 1;
            Ok(vec![format!(
                "fn_for_{}",
                t.split_whitespace().next().unwrap_or("x")
            )])
        })
        .unwrap();
        assert_eq!(calls, 3);
        assert_eq!(report.obligation_count, 3);
        assert_eq!(report.bindings.len(), 3);
        assert_eq!(report.contract_version, "1.0.0");
    }

    #[test]
    fn discover_bindings_propagates_runner_error() {
        let c = parse_demo_contract().unwrap();
        let err = discover_bindings_with(&c, |_| -> Result<Vec<String>> {
            Err(anyhow::anyhow!("pmat not on PATH"))
        })
        .unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("query for"));
        assert!(msg.contains("pmat not on PATH"));
    }

    #[test]
    fn discover_bindings_preserves_term_text_verbatim() {
        let c = parse_demo_contract().unwrap();
        let terms_seen = std::cell::RefCell::new(Vec::<String>::new());
        let _ = discover_bindings_with(&c, |t| {
            terms_seen.borrow_mut().push(t.to_string());
            Ok(Vec::new())
        })
        .unwrap();
        let expected = extract_query_terms(&c);
        assert_eq!(*terms_seen.borrow(), expected);
    }

    #[test]
    fn binding_clone_debug_eq() {
        let b = Binding {
            term: "x".into(),
            matches: vec!["fn_a".into()],
        };
        let c = b.clone();
        assert_eq!(b, c);
        let _ = format!("{b:?}");
    }

    #[test]
    fn report_clone_debug_eq() {
        let r = BindingReport {
            contract_version: "1.0.0".into(),
            obligation_count: 0,
            bindings: Vec::new(),
        };
        let c = r.clone();
        assert_eq!(r, c);
        let _ = format!("{r:?}");
    }

    #[test]
    fn parse_contract_yaml_errors_on_malformed_input() {
        // Exercises the .map_err + .context error path of
        // `parse_contract_yaml`. Without this, those branches stay
        // uncovered (the bundled YAML always parses cleanly).
        let bad = "::: not yaml :::\n    -- broken --\n";
        let err = parse_contract_yaml(bad).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("parse contract YAML"));
    }
}
