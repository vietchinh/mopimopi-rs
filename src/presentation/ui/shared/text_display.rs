//! Cell text and job icons as DOM.

use crate::domain::formatting::TextFragment;
use crate::domain::settings::Settings;
use dioxus::prelude::*;
use crate::infrastructure::act::data::CombatantRecord;

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

/// Job icon image for a player (`images/icon/<icon set>/<JOB TEXT>.png`, the raw ACT job text
/// uppercased). Pets, chocobos and Limit Break have no real job text, so their icon file name is
/// whatever ACT sent (often blank); that is a known, accepted rough edge for now.
pub fn job_icon_view(settings: &Settings, combatant: &CombatantRecord) -> Element {
    let source = format!("images/icon/{}/{}.png", settings.option_text("iconSet"), combatant.job_text.to_uppercase());
    rsx! { img { src: "{source}" } }
}
