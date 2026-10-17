//! The summary text in the top bar.

use crate::application::app_state::{SettingsContext, TablesContext};
use crate::domain::formatting::{cell_plain_text, CellContext, ColumnKey, NumberFormat};
use crate::domain::settings::Settings;
use crate::presentation::ui::areas::SummaryParts;
use crate::application::i18n::{dictionary_title, translate};
use crate::domain::formatting::LIMIT_BREAK_JOB_CODE;
use crate::domain::translations::translations;
use crate::domain::combat::{Encounter, PetView};
use dioxus::prelude::*;
use std::fmt::Write;

/// Shows only the parts the user enabled ("Display Type of Combatant Data" settings). Without a
/// local player row it shows the "Please start the combat." hint.
pub(super) fn summary_line(
    settings_context: SettingsContext,
    tables: TablesContext,
    settings: &Settings,
    parts: &SummaryParts,
    message: &Encounter,
    pet_view: PetView<'_>,
    is_settings_preview: bool,
) -> Element {
    let waiting_hint = translate(&translations().ui_schema["NAV"]["main"]["tt"]["rps"]);

    let Some(standing) = message.local_standing(pet_view) else { return rsx! { "{waiting_hint}" } };
    let rows = message.rows(pet_view);
    let player = &rows[standing.damage_rank - 1];
    let encounter = message.facts();
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
        let _ = write!(summary, "My DPS {}　", whole(standing.damage_per_second.value()));
    }
    if parts.max_hit_heal {
        let _ = write!(summary, "My HPS {}　", whole(standing.healing_per_second.value()));
    }
    if parts.rank {
        let _ = write!(summary, "Rank {}/{}/{}　", standing.damage_rank, standing.healing_rank, standing.of);
    }

    // Clicking the "MaxHit" label switches it to "MaxHeal" and back.
    let (label, strongest_action_text) = if parts.shows_strongest_heal {
        ("MaxHeal ", cell_plain_text(&ColumnKey::Maxheal, player, encounter, &cell_context))
    } else {
        ("MaxHit ", cell_plain_text(&ColumnKey::Maxhit, player, encounter, &cell_context))
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
