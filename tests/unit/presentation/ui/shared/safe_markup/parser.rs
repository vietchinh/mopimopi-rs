//! Unit tests for `ui::shared::safe_markup::parser`. They are compiled as a child module of that
//! file (so they can use its private items) but live here, outside `src/`.

use super::*;
use crate::presentation::ui::shared::safe_markup::markup_tree::{MarkupNode, MarkupTag};
use serde_json::Value;

fn only_element(nodes: &[MarkupNode]) -> &crate::presentation::ui::shared::safe_markup::markup_tree::MarkupElement {
    match nodes {
        [MarkupNode::Element(element)] => element,
        other => panic!("expected one element, got {other:?}"),
    }
}

/// The text a reader would see, without any markup.
fn visible_text(nodes: &[MarkupNode]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            MarkupNode::Text(text) => text.clone(),
            MarkupNode::LineBreak => "\n".to_string(),
            MarkupNode::Element(element) => visible_text(&element.children),
        })
        .collect()
}

#[test]
fn keeps_allowed_tags_and_attributes() {
    let nodes = parse_markup(r#"<b style="color:#ffea00">Hi</b>"#);
    let bold = only_element(&nodes);
    assert_eq!(bold.tag, MarkupTag::Bold);
    assert_eq!(bold.attributes.text_color.as_deref(), Some("#ffea00"));
    assert_eq!(visible_text(&bold.children), "Hi");

    let nodes = parse_markup(r#"<font class="ex">unit</font>"#);
    assert_eq!(only_element(&nodes).attributes.class.as_deref(), Some("ex"));
}

#[test]
fn line_breaks_and_plain_text() {
    let nodes = parse_markup("one<br>two<br/>three");
    assert_eq!(visible_text(&nodes), "one\ntwo\nthree");
}

#[test]
fn event_handlers_and_ids_are_dropped() {
    let nodes = parse_markup(r##"<a href="#" id="KR" onclick="initOverlay(this.id)">x</a>"##);
    let anchor = only_element(&nodes);
    assert_eq!(anchor.attributes.link_target.as_deref(), Some("#"));
    assert_eq!(anchor.attributes.class, None);
    assert_eq!(visible_text(&anchor.children), "x");
}

#[test]
fn dangerous_links_are_dropped() {
    for link in ["javascript:alert(1)", "data:text/html,<b>", "JaVaScRiPt:alert(1)", "//evil.example/x", " https://a b"] {
        let nodes = parse_markup(&format!(r#"<a href="{link}">x</a>"#));
        assert_eq!(only_element(&nodes).attributes.link_target, None, "{link}");
    }
    let nodes = parse_markup(r#"<a href="https://example.com/x" target="_blank">x</a>"#);
    let anchor = only_element(&nodes);
    assert_eq!(anchor.attributes.link_target.as_deref(), Some("https://example.com/x"));
    assert!(anchor.attributes.opens_in_new_tab);
}

#[test]
fn scripts_and_unknown_tags_never_become_elements() {
    let nodes = parse_markup("<script>alert(1)</script><iframe src=x></iframe>ok");
    assert!(nodes.iter().all(|node| matches!(node, MarkupNode::Text(_))));
    assert!(visible_text(&nodes).ends_with("ok"));
}

#[test]
fn images_must_come_from_the_images_folder() {
    let nodes = parse_markup(r#"<img src='./images/icon/frame/PLD.png'/>"#);
    assert_eq!(only_element(&nodes).attributes.image_source.as_deref(), Some("images/icon/frame/PLD.png"));
    for source in ["https://evil.example/x.png", "images/../secret.png", "data:image/png;base64,AAAA", "javascript:x"] {
        let nodes = parse_markup(&format!(r#"<img src="{source}">"#));
        assert_eq!(only_element(&nodes).attributes.image_source, None, "{source}");
    }
}

#[test]
fn only_a_text_colour_is_accepted_as_style() {
    let nodes = parse_markup(r#"<b style="background:url(x);color:red">a</b>"#);
    assert_eq!(only_element(&nodes).attributes.text_color, None);
    let nodes = parse_markup(r#"<font style="color:cyan">a</font>"#);
    assert_eq!(only_element(&nodes).attributes.text_color.as_deref(), Some("cyan"));
    let nodes = parse_markup(r#"<font style="color:red;position:fixed">a</font>"#);
    assert_eq!(only_element(&nodes).attributes.text_color, None);
}

#[test]
fn bad_nesting_is_repaired() {
    let nodes = parse_markup("<b>bold <font>inner</b> after</font> tail");
    assert_eq!(visible_text(&nodes), "bold inner after tail");
    let unclosed = parse_markup("<div><span>open");
    assert_eq!(visible_text(&unclosed), "open");
}

#[test]
fn entities_and_stray_angle_brackets_are_text() {
    assert_eq!(visible_text(&parse_markup("a &lt;b&gt; &amp; &#65;&#x42;")), "a <b> & AB");
    assert_eq!(visible_text(&parse_markup("1 < 2 and 3 > 2")), "1 < 2 and 3 > 2");
}

#[test]
fn every_string_of_the_translation_files_is_handled() {
    fn strings(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::String(text) => found.push(text.clone()),
            Value::Array(items) => items.iter().for_each(|item| strings(item, found)),
            Value::Object(entries) => entries.values().for_each(|item| strings(item, found)),
            _ => {}
        }
    }
    fn check(nodes: &[MarkupNode]) {
        for node in nodes {
            if let MarkupNode::Element(element) = node {
                if let Some(source) = &element.attributes.image_source {
                    assert!(source.starts_with("images/"));
                }
                if let Some(link) = &element.attributes.link_target {
                    assert!(link == "#" || link.starts_with("http"));
                }
                check(&element.children);
            }
        }
    }
    let mut all = Vec::new();
    for file in [
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/l.json")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/d.json")),
    ] {
        strings(&serde_json::from_str(file).unwrap(), &mut all);
    }
    assert!(all.len() > 1000);
    for text in &all {
        check(&parse_markup(text));
    }
}
