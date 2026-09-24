//! The summary text in the top bar.

use crate::application::app_state::AppContext;
use crate::domain::combat::EncounterRankings;
use crate::domain::formatting::{cell_plain_text, CellContext, NumberFormat};
use crate::domain::settings::Settings;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;

/// Shows only the parts the user enabled ("Display Type of Combatant Data" settings). Without a
/// local player row it shows the "Please start the combat." hint.
pub(super) fn summary_line(
    context: AppContext,
    settings: &Settings,
    language: &str,
    rankings: Option<&EncounterRankings>,
    is_settings_preview: bool,
) -> Element {
    let waiting_hint = translate(&translations().ui_schema["NAV"]["main"]["tt"]["rps"], language);
    let Some(rankings) = rankings else { return rsx! { "{waiting_hint}" } };
    let (Some(local_damage), Some(local_healing)) = (rankings.by_damage.local_player(), rankings.by_healing.local_player()) else {
        return rsx! { "{waiting_hint}" };
    };

    let number_format = NumberFormat::from_settings(settings);
    let rate = |value: f64| number_format.format_number(value, 1.0, number_format.rate_decimals);
    let local_player_name = context.local_player_name.read().clone();
    let cell_context = CellContext::new(settings, translations(), &local_player_name);

    let mut summary = String::new();
    if settings.option_enabled("act_rd") {
        summary += &format!("Total DPS {}　", rate(rankings.by_damage.encounter.damage_per_second));
    }
    if settings.option_enabled("act_rh") {
        summary += &format!("Total HPS {}　", rate(rankings.by_healing.encounter.heal_per_second));
    }
    if settings.option_enabled("act_md") {
        summary += &format!("My DPS {}　", rate(local_damage.rates.encounter_damage_per_second));
    }
    if settings.option_enabled("act_mh") {
        summary += &format!("My HPS {}　", rate(local_healing.rates.encounter_heal_per_second));
    }
    if settings.option_enabled("act_rank") {
        summary += &format!("Rank {}/{}/{}　", local_damage.rank + 1, local_healing.rank + 1, rankings.by_damage.party_size);
    }

    // Clicking the "MaxHit" label switches it to "MaxHeal" and back.
    let shows_strongest_heal = settings.option_enabled("swap");
    let (label, strongest_action_text) = if shows_strongest_heal {
        ("MaxHeal ", cell_plain_text("maxheal", local_healing, &rankings.by_healing, &cell_context))
    } else {
        ("MaxHit ", cell_plain_text("maxhit", local_damage, &rankings.by_damage, &cell_context))
    };
    let toggle_strongest_action_kind = move |_| {
        if !is_settings_preview {
            context.edit_settings(|settings| {
                let was_showing_heal = settings.option_enabled("swap");
                settings.set_option_enabled("swap", !was_showing_heal);
            });
        }
    };
    rsx! {
        "{summary}"
        if settings.option_enabled("act_max") {
            span { "name": "swapBtn", onclick: toggle_strongest_action_kind, "{label}" }
            "{strongest_action_text}"
        }
    }
}
