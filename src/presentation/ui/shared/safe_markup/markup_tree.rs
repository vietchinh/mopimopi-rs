//! The parsed form of a markup fragment.

/// Tags that may appear. `<font>` is drawn as a `span` (the stylesheet only uses its classes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MarkupTag {
    Bold,
    Font,
    Anchor,
    Division,
    Span,
    Image,
}

impl MarkupTag {
    /// The tag for a (case-insensitive) name, if it is on the allowlist. `br` is handled separately.
    pub(super) fn from_name(name: &str) -> Option<MarkupTag> {
        match name.to_ascii_lowercase().as_str() {
            "b" => Some(MarkupTag::Bold),
            "font" => Some(MarkupTag::Font),
            "a" => Some(MarkupTag::Anchor),
            "div" => Some(MarkupTag::Division),
            "span" => Some(MarkupTag::Span),
            "img" => Some(MarkupTag::Image),
            _ => None,
        }
    }

    /// Tags that never have children or a closing tag.
    pub(super) fn is_void(self) -> bool {
        self == MarkupTag::Image
    }
}

/// Attributes that survived the allowlist. Everything else is dropped while parsing.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SafeAttributes {
    pub class: Option<String>,
    /// The `name` attribute of a `div` (the stylesheet selects `[name=row]`).
    pub name: Option<String>,
    /// From `style="color:..."`; the only style allowed.
    pub text_color: Option<String>,
    /// `#`, `http://` or `https://` links only.
    pub link_target: Option<String>,
    pub opens_in_new_tab: bool,
    /// Relative path below `images/`.
    pub image_source: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MarkupElement {
    pub tag: MarkupTag,
    pub attributes: SafeAttributes,
    pub children: Vec<MarkupNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum MarkupNode {
    Text(String),
    LineBreak,
    Element(MarkupElement),
}
