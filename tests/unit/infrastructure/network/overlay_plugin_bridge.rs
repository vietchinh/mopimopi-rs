//! Unit tests for `network::overlay_plugin_bridge`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn finds_guid_inside_user_agent() {
    let agent = "Mozilla/5.0 Chrome/1 0f8fad5b-d9cb-469f-a165-70867728950e Safari";
    assert_eq!(find_guid(agent).as_deref(), Some("0f8fad5b-d9cb-469f-a165-70867728950e"));
    assert_eq!(find_guid("no guid here"), None);
}
