//! The summary text in the top bar.

use crate::application::app_state::AppContext;
use crate::domain::formatting::{cell_plain_text, CellContext, NumberFormat};
use crate::domain::settings::Settings;
use crate::domain::translations::{translate, translations};
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};
use dioxus::prelude::*;

/// Shows only the parts the user enabled ("Display Type of Combatant Data" settings). Without a
/// local player row it shows the "Please start the combat." hint.
pub(super) fn summary_line(
    context: AppContext,
    settings: &Settings,
    language: &str,
    combatants: &[CombatantRecord],
    encounter: &EncounterRecord,
    is_settings_preview: bool,
) -> Element {
    let waiting_hint = translate(&translations().ui_schema["NAV"]["main"]["tt"]["rps"], language);

    let Some(damage_rank) = combatants.iter().position(|c| c.name == "YOU") else { return rsx! { "{waiting_hint}" } };
    let player = &combatants[damage_rank];
    let number_format = NumberFormat::from_settings(settings);
    let rate = |value: f64| number_format.format_number(value, 1.0, number_format.rate_decimals);
    let local_player_name = context.local_player_name.read().clone();
    let cell_context = CellContext::new(settings, translations(), &local_player_name);

    let mut summary = String::new();
    if settings.option_enabled("act_rd") {
        summary += &format!("Total DPS {}　", rate(encounter.damage_per_second));
    }
    if settings.option_enabled("act_rh") {
        summary += &format!("Total HPS {}　", rate(encounter.heal_per_second));
    }
    if settings.option_enabled("act_md") {

        summary += &format!("My DPS {}　", rate(player.damage / encounter.duration_seconds));
    }
    if settings.option_enabled("act_mh") {
        summary += &format!("My HPS {}　", rate(player.healed / encounter.duration_seconds));
    }
    if settings.option_enabled("act_rank") {
        let mut by_healing: Vec<&CombatantRecord> = combatants.iter().collect();
        by_healing.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(std::cmp::Ordering::Equal));
        let healing_rank = by_healing.iter().position(|c| c.name == "YOU").unwrap_or(damage_rank);
        summary += &format!("Rank {}/{}/{}　", damage_rank + 1, healing_rank + 1, combatants.len());
    }

    // Clicking the "MaxHit" label switches it to "MaxHeal" and back.
    let shows_strongest_heal = settings.option_enabled("swap");
    let (label, strongest_action_text) = if shows_strongest_heal {
        ("MaxHeal ", cell_plain_text("maxheal", player, encounter, &cell_context))
    } else {
        ("MaxHit ", cell_plain_text("maxhit", player, encounter, &cell_context))
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
