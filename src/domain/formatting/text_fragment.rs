//! A piece of cell text: normal, or dimmed (units and action names are drawn dimmer).

#[derive(Clone, Debug, PartialEq)]
pub enum TextFragment {
    Plain(String),
    Dimmed(String),
}

impl TextFragment {
    pub fn text(&self) -> &str {
        match self {
            TextFragment::Plain(text) | TextFragment::Dimmed(text) => text,
        }
    }
}

/// Joins fragments into one string, dropping the dimming.
pub fn join_plain_text(fragments: &[TextFragment]) -> String {
    fragments.iter().map(TextFragment::text).collect()
}
