//! Cell text and job icons as DOM.

use crate::domain::combat::Player;
use crate::domain::formatting::TextFragment;
use crate::domain::settings::Settings;
use dioxus::prelude::*;

/// Fragments as DOM: plain text, and dimmed text in a `span.ex`.
pub fn text_fragments_view(fragments: Vec<TextFragment>) -> Element {
    rsx! {
        for fragment in fragments {
            match fragment {
                TextFragment::Plain(text) => rsx! { "{text}" },
                TextFragment::Dimmed(text) => rsx! { span { class: "ex", "{text}" } },
            }
        }
    }
}

/// Job icon image for a player (`images/icon/<icon set>/<JOB>.png`).
pub fn job_icon_view(settings: &Settings, player: &Player) -> Element {
    let source = format!("images/icon/{}/{}.png", settings.option_text("iconSet"), player.job_code.to_uppercase());
    rsx! { img { src: "{source}" } }
}
