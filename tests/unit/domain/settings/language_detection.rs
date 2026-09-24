//! Unit tests for `settings::language_detection`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn maps_browser_languages() {
    assert_eq!(language_code_for_browser_language("ko-kr"), "KR");
    assert_eq!(language_code_for_browser_language("zh-tw"), "CN");
    assert_eq!(language_code_for_browser_language("nl-nl"), "EN");
}
