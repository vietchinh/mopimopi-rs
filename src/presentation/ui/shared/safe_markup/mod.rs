//! Safe rendering of the small HTML fragments that the translation files contain
//! (`<b>`, `<br>`, `<font class="ex">`, `<a href>`, `<img src>` ...).
//!
//! The text is **never** given to the browser as HTML (`dangerous_inner_html`). Instead:
//! 1. `parser` turns it into a tree of `MarkupNode`s and keeps only allowlisted tags and attributes
//!    (`attribute_rules` decides what each attribute may contain; event handlers, `javascript:`
//!    links, unknown tags and scripts are dropped or shown as plain text),
//! 2. `render` builds ordinary Dioxus elements from that tree.
//!
//! Anything not on the allowlist can therefore not reach the DOM, whatever the text contains.
//!
//! * `markup_tree`      – the node types
//! * `attribute_rules`  – which attributes are allowed and how their values are checked
//! * `entities`         – `&amp;` style character references
//! * `parser`           – text -> `MarkupNode`s
//! * `render`           – `MarkupNode`s -> Dioxus elements

mod attribute_rules;
mod entities;
mod markup_tree;
mod parser;
mod render;

use dioxus::prelude::*;

/// Renders markup text as safe elements. Use it wherever text from the translation files is shown.
pub fn markup_view(markup: &str) -> Element {
    render::render_nodes(&parser::parse_markup(markup))
}
