//! Markup text -> `MarkupNode` tree, tolerant like a browser: bad nesting is repaired, unknown
//! tags are shown as text, and nothing outside the allowlist is kept.

use super::attribute_rules::sanitize_attributes;
use super::entities::decode_entities;
use super::markup_tree::{MarkupElement, MarkupNode, MarkupTag};

/// One tag as written in the text.
enum Tag {
    Open { tag: OpenTag, attributes: Vec<(String, String)>, self_closing: bool },
    Close(OpenTag),
}

#[derive(Clone, Copy, PartialEq)]
enum OpenTag {
    LineBreak,
    Element(MarkupTag),
}

/// An element whose closing tag has not been seen yet.
struct OpenElement {
    tag: MarkupTag,
    attributes: Vec<(String, String)>,
    children: Vec<MarkupNode>,
}

pub(super) fn parse_markup(markup: &str) -> Vec<MarkupNode> {
    let mut builder = TreeBuilder::default();
    let mut text = String::new();
    let mut position = 0;
    while position < markup.len() {
        let rest = &markup[position..];
        if rest.starts_with('<') {
            if let Some((tag, length)) = parse_tag(rest) {
                builder.add_text(std::mem::take(&mut text));
                builder.add_tag(tag);
                position += length;
                continue;
            }
        }
        let character = rest.chars().next().expect("position is inside the text");
        text.push(character);
        position += character.len_utf8();
    }
    builder.add_text(text);
    builder.finish()
}

#[derive(Default)]
struct TreeBuilder {
    open_elements: Vec<OpenElement>,
    top_level: Vec<MarkupNode>,
}

impl TreeBuilder {
    fn current_children(&mut self) -> &mut Vec<MarkupNode> {
        match self.open_elements.last_mut() {
            Some(open) => &mut open.children,
            None => &mut self.top_level,
        }
    }

    fn add_text(&mut self, text: String) {
        if !text.is_empty() {
            self.current_children().push(MarkupNode::Text(decode_entities(&text)));
        }
    }

    fn add_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Open { tag: OpenTag::LineBreak, .. } => self.current_children().push(MarkupNode::LineBreak),
            Tag::Open { tag: OpenTag::Element(tag), attributes, self_closing } => {
                if tag.is_void() || self_closing {
                    let element = close_element(OpenElement { tag, attributes, children: Vec::new() });
                    self.current_children().push(element);
                } else {
                    self.open_elements.push(OpenElement { tag, attributes, children: Vec::new() });
                }
            }
            Tag::Close(OpenTag::LineBreak) => {}
            Tag::Close(OpenTag::Element(tag)) => {
                // close the nearest open element of that kind (and anything left open inside it)
                if let Some(depth) = self.open_elements.iter().rposition(|open| open.tag == tag) {
                    while self.open_elements.len() > depth {
                        self.close_last_element();
                    }
                }
            }
        }
    }

    fn close_last_element(&mut self) {
        if let Some(open) = self.open_elements.pop() {
            let element = close_element(open);
            self.current_children().push(element);
        }
    }

    fn finish(mut self) -> Vec<MarkupNode> {
        while !self.open_elements.is_empty() {
            self.close_last_element();
        }
        self.top_level
    }
}

fn close_element(open: OpenElement) -> MarkupNode {
    MarkupNode::Element(MarkupElement {
        tag: open.tag,
        attributes: sanitize_attributes(open.tag, &open.attributes),
        children: open.children,
    })
}

/// Reads one tag at the start of `text` (which begins with `<`). Returns it and its length in bytes,
/// or `None` when this is not an allowlisted tag (the `<` is then ordinary text).
fn parse_tag(text: &str) -> Option<(Tag, usize)> {
    let end = find_tag_end(text)?;
    let inner = &text[1..end];
    let length = end + 1;
    if let Some(closing_name) = inner.strip_prefix('/') {
        return Some((Tag::Close(tag_from_name(closing_name.trim())?), length));
    }
    let self_closing = inner.trim_end().ends_with('/');
    let inner = inner.trim_end().trim_end_matches('/');
    let name_length = inner.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(inner.len());
    let tag = tag_from_name(&inner[..name_length])?;
    let attributes = parse_attributes(&inner[name_length..]);
    Some((Tag::Open { tag, attributes, self_closing }, length))
}

fn tag_from_name(name: &str) -> Option<OpenTag> {
    if name.eq_ignore_ascii_case("br") {
        Some(OpenTag::LineBreak)
    } else {
        MarkupTag::from_name(name).map(OpenTag::Element)
    }
}

/// Byte index of the `>` that ends the tag, ignoring `>` inside quoted attribute values.
fn find_tag_end(text: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (index, character) in text.char_indices().skip(1) {
        match (quote, character) {
            (None, '"' | '\'') => quote = Some(character),
            (Some(open_quote), c) if c == open_quote => quote = None,
            (None, '>') => return Some(index),
            (None, '<') => return None, // a new tag starts first: this `<` was just text
            _ => {}
        }
    }
    None
}

/// `name="value" other='value' bare=value flag` -> pairs. Attributes without a value are ignored.
fn parse_attributes(text: &str) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    let mut rest = text.trim_start();
    while !rest.is_empty() {
        let name_length = rest.find(|c: char| c.is_whitespace() || c == '=').unwrap_or(rest.len());
        let name = rest[..name_length].to_string();
        rest = rest[name_length..].trim_start();
        let Some(after_equals) = rest.strip_prefix('=') else {
            rest = rest.trim_start();
            continue; // a flag without a value
        };
        let after_equals = after_equals.trim_start();
        let (value, remaining) = match after_equals.chars().next() {
            Some(quote @ ('"' | '\'')) => {
                let body = &after_equals[1..];
                match body.find(quote) {
                    Some(close) => (&body[..close], &body[close + 1..]),
                    None => (body, ""),
                }
            }
            _ => {
                let end = after_equals.find(char::is_whitespace).unwrap_or(after_equals.len());
                (&after_equals[..end], &after_equals[end..])
            }
        };
        if !name.is_empty() {
            attributes.push((name, decode_entities(value)));
        }
        rest = remaining.trim_start();
    }
    attributes
}

#[cfg(test)]
#[path = "../../../../../tests/unit/presentation/ui/shared/safe_markup/parser.rs"]
mod tests;
