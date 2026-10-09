//! The summary text in the top bar.

use crate::application::app_state::{SettingsContext, TablesContext};
use crate::common::javascript_compat::p_float;
use crate::domain::formatting::{cell_plain_text, CellContext, NumberFormat};
use crate::domain::settings::Settings;
use crate::presentation::ui::areas::SummaryParts;
use crate::application::i18n::{dictionary_title, translate};
use crate::domain::formatting::LIMIT_BREAK_JOB_CODE;
use crate::domain::translations::translations;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};
use dioxus::prelude::*;
use std::fmt::Write;

/// Shows only the parts the user enabled ("Display Type of Combatant Data" settings). Without a
/// local player row it shows the "Please start the combat." hint.
pub(super) fn summary_line(
    settings_context: SettingsContext,
    tables: TablesContext,
    settings: &Settings,
    parts: &SummaryParts,
    combatants: &[CombatantRecord],
    encounter: &EncounterRecord,
    is_settings_preview: bool,
) -> Element {
    let waiting_hint = translate(&translations().ui_schema["NAV"]["main"]["tt"]["rps"]);

    let Some(damage_rank) = combatants.iter().position(|c| c.name == "YOU") else { return rsx! { "{waiting_hint}" } };
    let player = &combatants[damage_rank];
    let number_format = NumberFormat::from_settings(settings);
    // The top bar always shows whole numbers (the original's `addComma(x)` with no decimals), whatever
    // the DPS/HPS decimals setting says for the tables.
    let whole = |value: f64| number_format.format_number(value, 1.0, 0);
    let local_player_name = tables.local_player_name.read().clone();
    let cell_context = CellContext::new(settings, dictionary_title(LIMIT_BREAK_JOB_CODE), &local_player_name);

    let mut summary = String::new();
    if parts.total_dps {
        let _ = write!(summary, "Total DPS {}　", whole(encounter.damage_per_second_whole));
    }
    if parts.total_hps {
        let _ = write!(summary, "Total HPS {}　", whole(encounter.heal_per_second_whole));
    }
    if parts.max_hit_damage {
        // Recomputed from the raw accumulator and the encounter's own duration, same as the
        // "encdps" table column and for the same reason (see `column_cell::cell_fragments`): the
        // local player's own raw rate field can go stale while they aren't dealing damage, but the
        // encounter's duration keeps advancing every message.
        let _ = write!(summary, "My DPS {}　", whole(p_float(player.damage / encounter.duration_seconds)));
    }
    if parts.max_hit_heal {
        let _ = write!(summary, "My HPS {}　", whole(p_float(player.healed / encounter.duration_seconds)));
    }
    if parts.rank {
        let mut by_healing: Vec<&CombatantRecord> = combatants.iter().collect();
        by_healing.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(std::cmp::Ordering::Equal));
        let healing_rank = by_healing.iter().position(|c| c.name == "YOU").unwrap_or(damage_rank);
        let _ = write!(summary, "Rank {}/{}/{}　", damage_rank + 1, healing_rank + 1, combatants.len());
    }

    // Clicking the "MaxHit" label switches it to "MaxHeal" and back.
    let (label, strongest_action_text) = if parts.shows_strongest_heal {
        ("MaxHeal ", cell_plain_text("maxheal", player, encounter, &cell_context))
    } else {
        ("MaxHit ", cell_plain_text("maxhit", player, encounter, &cell_context))
    };
    let toggle_strongest_action_kind = move |_| {
        if !is_settings_preview {
            settings_context.edit_settings(|settings| {
                let was_showing_heal = settings.option_enabled("swap");
                settings.set_option_enabled("swap", !was_showing_heal);
            });
        }
    };
    rsx! {
        "{summary}"
        if parts.max_hit {
            span { "name": "swapBtn", onclick: toggle_strongest_action_kind, "{label}" }
            "{strongest_action_text}"
        }
    }
}
