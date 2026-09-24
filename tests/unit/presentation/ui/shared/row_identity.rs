//! Unit tests for `ui::shared::row_identity`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn builds_ids() {
    assert_eq!(row_element_id("Eos (YOU)"), "EosYOU");
    assert_eq!(row_element_id("O'Neil Smith"), "O_NeilSmith");
}
