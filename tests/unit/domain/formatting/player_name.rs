//! Unit tests for `formatting::player_name`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn abbreviation_modes() {
    assert_eq!(abbreviate_name("Eos Fair", NameAbbreviation::FullName), "Eos Fair");
    assert_eq!(abbreviate_name("Eos Fair", NameAbbreviation::AbbreviatedLastName), "Eos F.");
    assert_eq!(abbreviate_name("Eos Fair", NameAbbreviation::AbbreviatedFirstName), "E. Fair");
    assert_eq!(abbreviate_name("Eos Fair", NameAbbreviation::AbbreviatedBothNames), "E. F.");
    assert_eq!(abbreviate_name("Solo", NameAbbreviation::AbbreviatedBothNames), "Solo");
}
