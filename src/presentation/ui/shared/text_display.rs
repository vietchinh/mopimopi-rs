//! Cell text and job icons as DOM.

use crate::domain::formatting::TextFragment;
use dioxus::prelude::*;
use crate::domain::combat::CombatantKind;
use crate::infrastructure::act::data::CombatantRecord;

/// Fragments as DOM: plain text, and dimmed text in a `span.ex`. `sized_by_table` says whether the
/// dimmed span takes the table's dimmed-text size (`chrome-ex`, from `--chrome-ex-size`) -- that
/// never applied to raid-mode cards in the original (a different selector scope), so they pass `false`.
pub fn text_fragments_view(fragments: Vec<TextFragment>, sized_by_table: bool) -> Element {
    rsx! {
        for fragment in fragments {
            match fragment {
                TextFragment::Plain(text) => rsx! { "{text}" },
                TextFragment::Dimmed(text) => rsx! { span { class: "ex", class: if sized_by_table { "text-(length:--chrome-ex-size)" }, "{text}" } },
            }
        }
    }
}

/// The job codes that have an icon file (`public/images/icon/<set>/<CODE>.png`, the same files in every set: a unit test
/// keeps this list and the folders in step). Anything else has no icon, and shows Limit Break's.
const ICON_CODES: [&str; 46] = [
    "ACN", "ALC", "ARC", "ARM", "AST", "AVA", "BLM", "BLU", "BRD", "BSM", "BST", "BTN", "CBO", "CNJ", "CRP", "CUL", "DNC", "DRG", "DRK", "FSH", "GLA",
    "GNB", "GSM", "LMB", "LNC", "LTW", "MCH", "MIN", "MNK", "MRD", "NIN", "PCT", "PGL", "PLD", "RDM", "ROG", "RPR", "SAM", "SCH", "SGE", "SMN", "THM",
    "VPR", "WAR", "WHM", "WVR",
];

/// The icon file to show for a job code: its own when there is one, Limit Break's (`LMB`) for anything else (a job that
/// ACT sends that the overlay has no icon for, an empty one, ...) rather than a broken image.
fn icon_code(job_code: &str) -> &str {
    if ICON_CODES.contains(&job_code) { job_code } else { "LMB" }
}

/// Job icon image for a player (`images/icon/<icon set>/<JOB TEXT>.png`). `sized_by_table` says
/// whether the image takes the table's icon width (`chrome-icon`, from `--chrome-icon-size`) --
/// raid mode's cards size their icon through the parent `.rIcon`'s own nested `& img` rule
/// instead, so they pass `false`.
pub fn job_icon_view(icon_set: &str, combatant: &CombatantRecord, sized_by_table: bool) -> Element {
    let job_code = match combatant.kind() {
        CombatantKind::LimitBreak => "LMB".to_string(),
        CombatantKind::Pet { .. } => "AVA".to_string(),
        CombatantKind::Chocobo { .. } => "CBO".to_string(),
        _ => combatant.job_text.to_uppercase(),
    };

    let source = format!("images/icon/{icon_set}/{}.png", icon_code(&job_code));
    rsx! { img { class: if sized_by_table { "w-(--chrome-icon-size)" }, src: "{source}" } }
}

#[cfg(test)]
#[path = "../../../../tests/unit/presentation/ui/shared/text_display.rs"]
mod tests;
