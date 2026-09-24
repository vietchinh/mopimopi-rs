//! Which attributes are allowed, and what their values may contain.
//!
//! Everything not listed here is dropped: `onclick` and other event handlers, `id`, inline
//! styles other than a text colour, `srcset`, and so on.

use super::markup_tree::{MarkupTag, SafeAttributes};

/// Keeps the allowed attributes of `tag` from the raw `(name, value)` pairs.
pub(super) fn sanitize_attributes(tag: MarkupTag, raw_attributes: &[(String, String)]) -> SafeAttributes {
    let mut safe = SafeAttributes::default();
    for (name, value) in raw_attributes {
        match (name.to_ascii_lowercase().as_str(), tag) {
            ("class", _) if is_identifier_list(value) => safe.class = Some(value.clone()),
            ("name", MarkupTag::Division) if is_identifier_list(value) => safe.name = Some(value.clone()),
            ("style", MarkupTag::Bold | MarkupTag::Font | MarkupTag::Span) => safe.text_color = parse_text_color(value),
            ("href", MarkupTag::Anchor) if is_safe_link(value) => safe.link_target = Some(value.clone()),
            ("target", MarkupTag::Anchor) => safe.opens_in_new_tab = value == "_blank",
            ("src", MarkupTag::Image) => safe.image_source = normalize_image_source(value),
            _ => {}
        }
    }
    safe
}

/// Class and name values: letters, digits, `_`, `-` and spaces between class names.
fn is_identifier_list(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ' '))
}

/// `color:#ffea00` or `color:cyan` (optionally with spaces and a trailing `;`). Returns the value.
fn parse_text_color(style: &str) -> Option<String> {
    let declaration = style.trim().trim_end_matches(';');
    let (property, value) = declaration.split_once(':')?;
    if !property.trim().eq_ignore_ascii_case("color") {
        return None;
    }
    let value = value.trim();
    let is_hex = value.strip_prefix('#').is_some_and(|digits| matches!(digits.len(), 3 | 4 | 6 | 8) && digits.chars().all(|c| c.is_ascii_hexdigit()));
    let is_named = !value.is_empty() && value.len() <= 20 && value.chars().all(|c| c.is_ascii_alphabetic());
    (is_hex || is_named).then(|| value.to_string())
}

/// Only in-page anchors and web links: never `javascript:`, `data:` or other schemes.
fn is_safe_link(value: &str) -> bool {
    let has_only_plain_characters = value.chars().all(|c| !c.is_whitespace() && !c.is_control() && !matches!(c, '"' | '\'' | '<' | '>'));
    has_only_plain_characters && (value == "#" || value.starts_with("https://") || value.starts_with("http://"))
}

/// Images must come from the overlay's own `images/` folder. `./images/a.png` becomes `images/a.png`.
fn normalize_image_source(value: &str) -> Option<String> {
    let path = value.strip_prefix("./").unwrap_or(value);
    let is_plain_path = path.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'));
    (path.starts_with("images/") && is_plain_path && !path.contains("..")).then(|| path.to_string())
}
