//! Helpers for the graph bars. Their colours and fade are `areas::BarSettings`.

/// `part` as a whole percentage of `whole`, limited to 0-100.
pub fn percent_of_whole(part: f64, whole: f64) -> i32 {
    if whole <= 0.0 {
        return 0;
    }
    let percent = (part / whole * 100.0).trunc();
    if percent.is_finite() { percent.clamp(0.0, 100.0) as i32 } else { 0 }
}
