//! Unit tests for `ui::settings_screens::base64_encoding`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

#[test]
fn base64_matches_reference() {
    assert_eq!(super::base64_encode(b"Man"), "TWFu");
    assert_eq!(super::base64_encode(b"Ma"), "TWE=");
    assert_eq!(super::base64_encode(b"M"), "TQ==");
}
