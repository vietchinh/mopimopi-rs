//! `MarkupNode`s -> Dioxus elements.

use super::markup_tree::{MarkupElement, MarkupNode, MarkupTag};
use dioxus::prelude::*;

pub(super) fn render_nodes(nodes: &[MarkupNode]) -> Element {
    rsx! {
        for (index , node) in nodes.iter().enumerate() {
            Fragment { key: "{index}", {render_node(node)} }
        }
    }
}

fn render_node(node: &MarkupNode) -> Element {
    match node {
        MarkupNode::Text(text) => rsx! { "{text}" },
        MarkupNode::LineBreak => rsx! { br {} },
        MarkupNode::Element(element) => render_element(element),
    }
}

fn render_element(element: &MarkupElement) -> Element {
    let attributes = &element.attributes;
    let class = attributes.class.clone();
    let color_style = attributes.text_color.as_ref().map(|color| format!("color:{color}"));
    let children = render_nodes(&element.children);
    match element.tag {
        MarkupTag::Bold => rsx! { b { class, style: color_style, {children} } },
        MarkupTag::Font | MarkupTag::Span => rsx! { span { class, style: color_style, {children} } },
        MarkupTag::Division => {
            let name = attributes.name.clone();
            rsx! { div { class, "name": name, {children} } }
        }
        MarkupTag::Anchor => {
            let href = attributes.link_target.clone();
            let (target, rel) = if attributes.opens_in_new_tab { (Some("_blank"), Some("noopener noreferrer")) } else { (None, None) };
            rsx! { a { class, href, target, rel, {children} } }
        }
        MarkupTag::Image => {
            let source = attributes.image_source.clone();
            rsx! { img { class, src: source } }
        }
    }
}
