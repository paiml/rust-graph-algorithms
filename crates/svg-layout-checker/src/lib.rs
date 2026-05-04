//! Falsifier for the c12 SVG layout contract.
//!
//! Companion code for Coursera **Rust for Data Engineering c12 — Graph
//! Algorithms with Rust**, lesson 5.1.5 (verifying graph code with
//! pmat query + aprender-contracts).
//!
//! Five structural floors are checked against an SVG source string:
//!
//! - [`Check::CanvasSize`] — viewBox is exactly `"0 0 1920 1080"`
//! - [`Check::ShapeFloor`] — at least 40 visual-shape elements
//! - [`Check::NamedGroupBounds`] — 5..=20 named `<g id="...">` groups
//! - [`Check::FillDiversity`] — at least 8 distinct hex fills
//! - [`Check::FontSizeFloor`] — minimum `font-size` attribute is 18+
//!
//! Spec at `contracts/svg-layout-v1.yaml`. Each [`Check`] maps to one
//! YAML obligation; the falsification-test ids on the right of the
//! [`FALSIFY_IDS`] table are the formal binding.

#![deny(missing_docs)]

use std::collections::BTreeSet;
use thiserror::Error;

/// One structural rule from the layout contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Check {
    /// `viewBox="0 0 1920 1080"` exact match.
    CanvasSize,
    /// At least 40 visual-shape elements.
    ShapeFloor,
    /// Named-group count in `[5, 20]`.
    NamedGroupBounds,
    /// At least 8 distinct hex fills.
    FillDiversity,
    /// Minimum `font-size` attribute is 18 or more.
    FontSizeFloor,
}

/// Falsification-test ids per [`Check`] — keep in sync with
/// `contracts/svg-layout-v1.yaml`.
pub const FALSIFY_IDS: &[(Check, &str)] = &[
    (Check::CanvasSize, "FALSIFY-SVG-001"),
    (Check::ShapeFloor, "FALSIFY-SVG-002"),
    (Check::NamedGroupBounds, "FALSIFY-SVG-003"),
    (Check::FillDiversity, "FALSIFY-SVG-004"),
    (Check::FontSizeFloor, "FALSIFY-SVG-005"),
];

/// Reasons a check can fail. Carries the offending value so callers can
/// surface a precise diagnostic.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum LayoutFailure {
    /// `viewBox` was not the canonical `"0 0 1920 1080"`.
    #[error("canvas size: expected viewBox \"0 0 1920 1080\", saw {0:?}")]
    CanvasMismatch(String),
    /// Visual-shape count fell below the floor.
    #[error("shape floor: counted {0} shapes, need >= 40")]
    ShapeBelowFloor(usize),
    /// Named-group count outside `[5, 20]`.
    #[error("named-group bounds: counted {0} groups, need in [5, 20]")]
    NamedGroupOutOfBounds(usize),
    /// Distinct hex-fill count fell below the floor.
    #[error("fill diversity: counted {0} distinct hex fills, need >= 8")]
    FillBelowFloor(usize),
    /// At least one font-size attribute is below 18.
    #[error("font-size floor: smallest font-size is {0}, need >= 18")]
    FontBelowFloor(u32),
}

/// One [`Check`]'s outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Rule was satisfied; the carried value is the measured datum
    /// (shape count, fill count, etc.) for diagnostic logging.
    Pass {
        /// Which check this is.
        check: Check,
        /// Measured value.
        measured: usize,
    },
    /// Rule failed — carries the structured failure reason.
    Fail(LayoutFailure),
}

impl CheckResult {
    /// `true` if this is a passing result.
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass { .. })
    }
}

/// Aggregate report across all five checks for a single SVG source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutReport {
    /// One result per check, in [`Check`] enum order.
    pub results: Vec<CheckResult>,
}

impl LayoutReport {
    /// `true` iff every check passed.
    pub fn all_pass(&self) -> bool {
        self.results.iter().all(CheckResult::is_pass)
    }

    /// Failed checks only.
    pub fn failures(&self) -> Vec<&LayoutFailure> {
        self.results
            .iter()
            .filter_map(|r| match r {
                CheckResult::Fail(f) => Some(f),
                CheckResult::Pass { .. } => None,
            })
            .collect()
    }
}

/// Run all five structural checks against an SVG source string. Returns
/// a [`LayoutReport`] regardless of pass/fail — the caller decides how
/// to react.
pub fn check_layout(svg: &str) -> LayoutReport {
    LayoutReport {
        results: vec![
            check_canvas_size(svg),
            check_shape_floor(svg),
            check_named_group_bounds(svg),
            check_fill_diversity(svg),
            check_font_size_floor(svg),
        ],
    }
}

/// Provable-contract check: viewBox is exactly `"0 0 1920 1080"`.
pub fn check_canvas_size(svg: &str) -> CheckResult {
    let actual = extract_viewbox(svg).unwrap_or_default();
    if actual == "0 0 1920 1080" {
        CheckResult::Pass {
            check: Check::CanvasSize,
            measured: 0,
        }
    } else {
        CheckResult::Fail(LayoutFailure::CanvasMismatch(actual))
    }
}

/// Provable-contract check: at least 40 visual-shape elements.
pub fn check_shape_floor(svg: &str) -> CheckResult {
    let n = count_shapes(svg);
    if n >= 40 {
        CheckResult::Pass {
            check: Check::ShapeFloor,
            measured: n,
        }
    } else {
        CheckResult::Fail(LayoutFailure::ShapeBelowFloor(n))
    }
}

/// Provable-contract check: named-group count in `[5, 20]`.
pub fn check_named_group_bounds(svg: &str) -> CheckResult {
    let n = count_named_groups(svg);
    if (5..=20).contains(&n) {
        CheckResult::Pass {
            check: Check::NamedGroupBounds,
            measured: n,
        }
    } else {
        CheckResult::Fail(LayoutFailure::NamedGroupOutOfBounds(n))
    }
}

/// Provable-contract check: at least 8 distinct hex fills.
pub fn check_fill_diversity(svg: &str) -> CheckResult {
    let n = count_distinct_hex_fills(svg);
    if n >= 8 {
        CheckResult::Pass {
            check: Check::FillDiversity,
            measured: n,
        }
    } else {
        CheckResult::Fail(LayoutFailure::FillBelowFloor(n))
    }
}

/// Provable-contract check: minimum font-size is 18+.
pub fn check_font_size_floor(svg: &str) -> CheckResult {
    let min = min_font_size(svg);
    if min >= 18 {
        CheckResult::Pass {
            check: Check::FontSizeFloor,
            measured: min as usize,
        }
    } else {
        CheckResult::Fail(LayoutFailure::FontBelowFloor(min))
    }
}

// ─── helpers ───────────────────────────────────────────────────────────

const SHAPE_TAGS: &[&str] = &[
    "<rect", "<circle", "<line", "<path", "<polygon", "<ellipse", "<text",
];

fn extract_viewbox(svg: &str) -> Option<String> {
    let key = "viewBox=\"";
    let i = svg.find(key)?;
    let rest = &svg[i + key.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn count_shapes(svg: &str) -> usize {
    SHAPE_TAGS.iter().map(|t| svg.matches(t).count()).sum()
}

fn count_named_groups(svg: &str) -> usize {
    // Match `<g id="..."` (named-group form). Anonymous `<g>` (without
    // id) doesn't count toward structural decomposition.
    svg.match_indices("<g ")
        .filter(|(i, _)| {
            let tail = &svg[*i..];
            // Look ahead within the tag for `id=`.
            tail.find('>')
                .map(|end| tail[..end].contains("id="))
                .unwrap_or(false)
        })
        .count()
}

fn count_distinct_hex_fills(svg: &str) -> usize {
    let mut set = BTreeSet::new();
    let needle = "fill=\"#";
    let mut start = 0usize;
    while let Some(i) = svg[start..].find(needle) {
        let abs = start + i + needle.len();
        if abs + 6 > svg.len() {
            break;
        }
        let hex = &svg[abs..abs + 6];
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            set.insert(hex.to_ascii_lowercase());
        }
        start = abs + 6;
    }
    set.len()
}

fn min_font_size(svg: &str) -> u32 {
    let needle = "font-size=\"";
    let mut start = 0usize;
    let mut min = u32::MAX;
    while let Some(i) = svg[start..].find(needle) {
        let abs = start + i + needle.len();
        let rest = &svg[abs..];
        let end = match rest.find('"') {
            Some(e) => e,
            None => break,
        };
        let raw = &rest[..end];
        // Accept integer or float; truncate float at the dot.
        let int_part = raw.split('.').next().unwrap_or(raw);
        if let Ok(v) = int_part.parse::<u32>() {
            if v < min {
                min = v;
            }
        }
        start = abs + end + 1;
    }
    if min == u32::MAX {
        // No font-size attributes at all — treat as passing (the master
        // simply has no text). Return the floor so the check passes.
        18
    } else {
        min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bundled minimal "good" SVG fixture — exactly clears every floor.
    fn good_svg() -> String {
        let mut shapes = String::new();
        for i in 0..40 {
            shapes.push_str(&format!(
                "<rect x=\"{i}\" y=\"0\" width=\"1\" height=\"1\"/>\n"
            ));
        }
        let groups = (0..6)
            .map(|i| format!("<g id=\"g{i}\"></g>\n"))
            .collect::<String>();
        let fills = (0..8)
            .map(|i| format!("<rect fill=\"#aabbc{i}\"/>\n"))
            .collect::<String>();
        format!(
            "<svg viewBox=\"0 0 1920 1080\">\n\
             <text font-size=\"20\">x</text>\n\
             <text font-size=\"24\">y</text>\n\
             {shapes}{groups}{fills}\n\
             </svg>"
        )
    }

    #[test]
    fn good_svg_passes_all_checks() {
        let report = check_layout(&good_svg());
        for r in &report.results {
            assert!(r.is_pass(), "unexpected fail: {r:?}");
        }
        assert!(report.all_pass());
        assert!(report.failures().is_empty());
    }

    #[test]
    fn canvas_size_failure() {
        let svg = "<svg viewBox=\"0 0 1280 720\"></svg>";
        let r = check_canvas_size(svg);
        assert_eq!(
            r,
            CheckResult::Fail(LayoutFailure::CanvasMismatch("0 0 1280 720".into()))
        );
        assert!(!r.is_pass());
    }

    #[test]
    fn canvas_size_missing_viewbox() {
        let r = check_canvas_size("<svg></svg>");
        assert!(matches!(r, CheckResult::Fail(LayoutFailure::CanvasMismatch(s)) if s.is_empty()));
    }

    #[test]
    fn shape_floor_failure() {
        let svg = "<svg viewBox=\"0 0 1920 1080\"><rect/></svg>";
        let r = check_shape_floor(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::ShapeBelowFloor(1)));
    }

    #[test]
    fn named_group_below_floor() {
        let svg = "<svg viewBox=\"0 0 1920 1080\"><g id=\"a\"/></svg>";
        let r = check_named_group_bounds(svg);
        assert_eq!(
            r,
            CheckResult::Fail(LayoutFailure::NamedGroupOutOfBounds(1))
        );
    }

    #[test]
    fn named_group_above_ceiling() {
        let mut svg = String::from("<svg viewBox=\"0 0 1920 1080\">");
        for i in 0..25 {
            svg.push_str(&format!("<g id=\"g{i}\"></g>"));
        }
        svg.push_str("</svg>");
        let r = check_named_group_bounds(&svg);
        assert_eq!(
            r,
            CheckResult::Fail(LayoutFailure::NamedGroupOutOfBounds(25))
        );
    }

    #[test]
    fn anonymous_groups_not_counted() {
        // `<g>` without id should not contribute to the named count.
        let svg = "<svg viewBox=\"0 0 1920 1080\"><g></g><g></g></svg>";
        let r = check_named_group_bounds(svg);
        assert_eq!(
            r,
            CheckResult::Fail(LayoutFailure::NamedGroupOutOfBounds(0))
        );
    }

    #[test]
    fn fill_diversity_failure() {
        let svg =
            "<svg viewBox=\"0 0 1920 1080\"><rect fill=\"#000000\"/><rect fill=\"#ffffff\"/></svg>";
        let r = check_fill_diversity(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FillBelowFloor(2)));
    }

    #[test]
    fn font_size_floor_failure() {
        let svg = "<svg viewBox=\"0 0 1920 1080\"><text font-size=\"14\">x</text></svg>";
        let r = check_font_size_floor(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FontBelowFloor(14)));
    }

    #[test]
    fn font_size_no_text_passes() {
        // No font-size attributes at all — we treat it as passing
        // (the master might be all shapes, e.g. a non-text decorative).
        let svg = "<svg viewBox=\"0 0 1920 1080\"></svg>";
        let r = check_font_size_floor(svg);
        assert!(r.is_pass());
    }

    #[test]
    fn font_size_with_float_value() {
        // font-size with a fractional value still parses by truncation.
        let svg = "<svg viewBox=\"0 0 1920 1080\"><text font-size=\"21.5\">x</text></svg>";
        let r = check_font_size_floor(svg);
        assert!(r.is_pass());
    }

    #[test]
    fn font_size_unparseable_value_skipped() {
        // A garbage font-size value is treated as absent (does not
        // bring down the minimum).
        let svg = "<svg viewBox=\"0 0 1920 1080\"><text font-size=\"abc\">x</text></svg>";
        let r = check_font_size_floor(svg);
        assert!(r.is_pass());
    }

    #[test]
    fn fill_invalid_hex_skipped() {
        // Non-hex characters in fill should be ignored, not counted.
        let svg = "<svg viewBox=\"0 0 1920 1080\"><rect fill=\"#zzzzzz\"/></svg>";
        let r = check_fill_diversity(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FillBelowFloor(0)));
    }

    #[test]
    fn fill_truncated_hex_skipped() {
        // A fill="#abc" (short hex) should not crash and should not
        // count toward the 6-char-distinct set.
        let svg = "<svg viewBox=\"0 0 1920 1080\"><rect fill=\"#abc\"/></svg>";
        let r = check_fill_diversity(svg);
        // The 3-char hex doesn't match the 6-char extractor, so it
        // contributes nothing.
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FillBelowFloor(0)));
    }

    #[test]
    fn fill_dedupes_case_insensitively() {
        let svg =
            "<svg viewBox=\"0 0 1920 1080\"><rect fill=\"#AABBCC\"/><rect fill=\"#aabbcc\"/></svg>";
        let r = check_fill_diversity(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FillBelowFloor(1)));
    }

    #[test]
    fn falsify_ids_cover_every_check() {
        // Sanity: every Check variant has a FALSIFY-SVG-NNN binding.
        let checks: Vec<Check> = FALSIFY_IDS.iter().map(|(c, _)| *c).collect();
        for variant in [
            Check::CanvasSize,
            Check::ShapeFloor,
            Check::NamedGroupBounds,
            Check::FillDiversity,
            Check::FontSizeFloor,
        ] {
            assert!(checks.contains(&variant), "{variant:?} missing");
        }
    }

    #[test]
    fn report_failures_lists_only_fails() {
        let svg = "<svg viewBox=\"0 0 1280 720\"><rect/></svg>";
        let report = check_layout(svg);
        assert!(!report.all_pass());
        let fails = report.failures();
        // At least canvas + shape + named-group + fill should fail.
        assert!(fails.len() >= 3);
    }

    #[test]
    fn fill_truncated_at_eof_breaks() {
        // fill="# at the very end with fewer than 6 hex chars left
        // exercises the EOF break in count_distinct_hex_fills.
        let svg = "<svg viewBox=\"0 0 1920 1080\">..fill=\"#abc";
        let r = check_fill_diversity(svg);
        assert_eq!(r, CheckResult::Fail(LayoutFailure::FillBelowFloor(0)));
    }

    #[test]
    fn font_size_unclosed_quote_breaks() {
        // font-size=" with no closing quote exercises the early break
        // in min_font_size.
        let svg = "<svg viewBox=\"0 0 1920 1080\"><text font-size=\"18";
        let r = check_font_size_floor(svg);
        // No font-size successfully parsed → treated as no text → pass.
        assert!(r.is_pass());
    }

    #[test]
    fn debug_clone_eq() {
        // Hit derive impls for coverage.
        let r = CheckResult::Pass {
            check: Check::CanvasSize,
            measured: 0,
        };
        let _ = format!("{r:?}");
        let c = r.clone();
        assert_eq!(r, c);

        let f = LayoutFailure::ShapeBelowFloor(3);
        let _ = format!("{f}");
        let _ = format!("{f:?}");
        // PartialEq derive
        assert_eq!(f, LayoutFailure::ShapeBelowFloor(3));

        let report = LayoutReport {
            results: vec![r.clone()],
        };
        let _ = report.clone();
        let _ = format!("{report:?}");
        assert_eq!(report, LayoutReport { results: vec![r] });

        let _ = format!("{:?}", Check::CanvasSize);
        // Hit Copy via assignment.
        let a = Check::CanvasSize;
        let b = a;
        assert_eq!(a, b);
    }
}
