//! Percentages the way ACT shows them, but unrounded.

/// `part` as a percentage of `whole`, or `None` when `whole` is 0 (ACT would show NaN).
pub fn percent_of(part: f64, whole: f64) -> Option<f64> {
    (whole != 0.0).then(|| part / whole * 100.0)
}
