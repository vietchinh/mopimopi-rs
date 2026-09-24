//! Unit tests for `network::act_endpoint`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

#[test]
fn host_port_becomes_a_mini_parse_url() {
    assert_eq!(mini_parse_url_from_host_port("127.0.0.1:10501/"), "ws://127.0.0.1:10501/MiniParse");
    assert_eq!(mini_parse_url_from_host_port("ws://10.0.0.2:10501"), "ws://10.0.0.2:10501/MiniParse");
}

#[test]
fn user_text_selects_the_protocol() {
    let overlay = |url: &str| ActEndpoint::OverlayPlugin(url.to_string());
    assert_eq!(ActEndpoint::from_user_text("ws://127.0.0.1:10501/ws"), overlay("ws://127.0.0.1:10501/ws"));
    assert_eq!(ActEndpoint::from_user_text("127.0.0.1:10501"), overlay("ws://127.0.0.1:10501/ws"));
    assert_eq!(
        ActEndpoint::from_user_text("ws://h:1/MiniParse"),
        ActEndpoint::MiniParse("ws://h:1/MiniParse".into())
    );
}
