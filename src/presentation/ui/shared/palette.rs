//! Helpers for the graph bars. Their colours and fade are `areas::BarSettings`.

use crate::domain::combat::Amount;

/// `part` as a whole percentage of `whole`, limited to 0-100.
pub fn percent_of_whole(part: Amount, whole: Amount) -> i32 {
    let (part, whole) = (part.value(), whole.value());
    if whole <= 0.0 {
        return 0;
    }
    let percent = (part / whole * 100.0).trunc();
    if percent.is_finite() { percent.clamp(0.0, 100.0) as i32 } else { 0 }
}
