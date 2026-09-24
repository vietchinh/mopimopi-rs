//! Element ids of table rows. The stylesheet targets `#YOU` (the local player's row).

/// "Eos (Some One)" -> "Eos_SomeOne"-like id: spaces and the first parentheses removed,
/// apostrophes replaced (same rule as the original overlay).
pub fn row_element_id(player_name: &str) -> String {
    player_name.replace(' ', "").replacen('(', "", 1).replacen(')', "", 1).replace('\'', "_")
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/shared/row_identity.rs"]
mod tests;
