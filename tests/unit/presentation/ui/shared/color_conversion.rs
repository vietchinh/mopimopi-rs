//! Unit tests for `theme::color_conversion`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn converts_hex() {
    assert_eq!(rgba("03A9F4", 0.5), "rgba(3,169,244,0.5)");
    assert_eq!(rgba("zz", 1.0), "rgba(0,0,0,1)");
}
