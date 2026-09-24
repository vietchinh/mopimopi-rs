//! What happens when ACT sends new combat data.

use dioxus::prelude::*;
use super::app_context::{AppContext, Screen};
use super::encounter_history::HistoryEntry;
use super::standby_mode::restart_standby_timer;
use crate::models::act_data::CombatDataMessage;
use crate::domain::combat::build_rankings;
use std::rc::Rc;

/// Called for every combat data message. The table is redrawn while a fight runs and once when
/// it ends (then the encounter is also added to the history). While the settings screen is open
/// the display is left alone so the user can edit undisturbed.
pub fn handle_combat_data_received(context: AppContext, message: CombatDataMessage) {
    let message = Rc::new(message);
    let mut latest = context.latest_combat_data;
    latest.set(Some(message.clone()));

    let settings_are_open = *context.current_screen.peek() == Screen::Settings;
    let previous_message_was_active = *context.encounter_was_active.peek();
    let mut encounter_was_active = context.encounter_was_active;

    if message.is_encounter_active {
        if !settings_are_open {
            show_message(context, message);
        }
        encounter_was_active.set(true);
    } else if previous_message_was_active {
        if !settings_are_open {
            record_finished_encounter(context, &message);
            show_message(context, message);
        }
        encounter_was_active.set(false);
    }
}

fn show_message(context: AppContext, message: Rc<CombatDataMessage>) {
    let mut has_received_data = context.has_received_data;
    if !*has_received_data.peek() {
        has_received_data.set(true);
        let mut screen = context.current_screen;
        screen.set(Screen::Main);
    }
    let mut displayed = context.displayed_combat_data;
    displayed.set(Some(message));
    restart_standby_timer(context);
}

/// Adds a finished encounter to the history (skipping duplicates of the previous entry).
fn record_finished_encounter(context: AppContext, message: &Rc<CombatDataMessage>) {
    let merge_pets = context.settings.peek().option_enabled("pets");
    let local_player_name = context.local_player_name.peek().clone();
    let rankings = build_rankings(message, merge_pets, &local_player_name);
    let encounter_key = rankings.by_damage.encounter_key.clone();

    let mut history = context.encounter_history;
    if history.peek().first().map(|entry| entry.encounter_key == encounter_key).unwrap_or(false) {
        return;
    }
    let encounter_number_in_zone = next_encounter_number_in_zone(context, &rankings.by_damage.encounter.zone_name);
    let encounter = &rankings.by_damage.encounter;
    let entry = HistoryEntry {
        encounter_key,
        title: if encounter.title != "Encounter" { encounter.title.clone() } else { "No Data".into() },
        zone_name: encounter.zone_name.clone(),
        duration_text: encounter.duration_text.clone(),
        encounter_dps: encounter.damage_per_second,
        encounter_hps: rankings.by_healing.encounter.heal_per_second,
        local_player_dps: rankings.by_damage.local_player().map(|player| player.rates.encounter_damage_per_second.floor()),
        local_player_hps: rankings.by_healing.local_player().map(|player| player.rates.encounter_heal_per_second.floor()),
        encounter_number_in_zone,
        combat_data: message.clone(),
    };
    history.write().insert(0, entry);
}

/// 1 for the first encounter or a new zone, otherwise one more than the previous entry's count.
fn next_encounter_number_in_zone(context: AppContext, zone_name: &str) -> usize {
    let mut counter = context.encounters_in_current_zone;
    let previous_zone = context.encounter_history.peek().first().map(|entry| entry.zone_name.clone());
    let next_number = match previous_zone {
        Some(previous) if previous == zone_name => *counter.peek() + 1,
        _ => 1,
    };
    counter.set(next_number);
    next_number
}
