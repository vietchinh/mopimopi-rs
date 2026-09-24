//! Unit tests for `ui::shared::safe_markup::entities`.

use super::*;

#[test]
fn decodes_named_and_numeric_references() {
    assert_eq!(decode_entities("Tom &amp; Jerry &quot;x&quot;"), "Tom & Jerry \"x\"");
    assert_eq!(decode_entities("&#39;&#x27;&nbsp;"), "''\u{a0}");
}

#[test]
fn leaves_unknown_or_unfinished_references_alone() {
    assert_eq!(decode_entities("a & b &unknown; c &amp"), "a & b &unknown; c &amp");
    assert_eq!(decode_entities("no references"), "no references");
}
