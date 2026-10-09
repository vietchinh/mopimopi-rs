use super::app_actions::AppActions;
use super::contexts::Screen;
use super::encounter_history::HistoryEntry;
use super::standby_mode::restart_standby_timer;
use crate::infrastructure::act::data::CombatDataMessage;
use dioxus::prelude::*;
use std::rc::Rc;

/// Called whenever `OverlayPlugin`'s combat-data signal changes. The table is redrawn while a
/// fight runs and once when it ends (then the encounter is also added to the history). While the
/// settings screen is open the display is left alone so the user can edit undisturbed.
pub fn on_combat_data_changed(actions: AppActions, message: Rc<CombatDataMessage>) {
    let settings_are_open = *actions.screen.current_screen.peek() == Screen::Settings;
    let previous_message_was_active = *actions.tables.encounter_was_active.peek();
    let mut encounter_was_active = actions.tables.encounter_was_active;

    if message.is_encounter_active {
        if !settings_are_open {
            show_message(actions, message);
        }
        encounter_was_active.set(true);
    } else if previous_message_was_active {
        if !settings_are_open {
            record_finished_encounter(actions, &message);
            show_message(actions, message);
        }
        encounter_was_active.set(false);
    } else if !*actions.tables.has_received_data.peek() && !settings_are_open {
        // The very first message, and nothing is happening: ACT is connected and idle. The start screen gives way to
        // the (empty) tables, as in the original, and the standby timer starts.
        show_message(actions, message);
    }
}

fn show_message(actions: AppActions, message: Rc<CombatDataMessage>) {
    let mut has_received_data = actions.tables.has_received_data;
    if !*has_received_data.peek() {
        has_received_data.set(true);
        let mut screen = actions.screen.current_screen;
        screen.set(Screen::Main);
    }
    let mut displayed = actions.tables.displayed_combat_data;
    displayed.set(Some(message));
    restart_standby_timer(actions);
}

/// Adds a finished encounter to the history (skipping duplicates of the previous entry).
fn record_finished_encounter(actions: AppActions, message: &Rc<CombatDataMessage>) {
    let encounter = &message.encounter;
    // Identifies the encounter (title + totals) well enough to detect duplicates.
    let encounter_key = format!("{}{}{}", encounter.title, encounter.total_damage, encounter.total_healed);

    let mut history = actions.history.encounter_history;
    if history.peek().first().is_some_and(|entry| entry.encounter_key == encounter_key) {
        return;
    }
    let encounter_number_in_zone = next_encounter_number_in_zone(actions, &encounter.zone_name);
    let entry = HistoryEntry { encounter_key: encounter_key.clone(), encounter_number_in_zone, combat_data: message.clone() };
    history.write().insert(0, entry);

    let mut viewed = actions.history.viewed_history_key;
    viewed.set(Some(encounter_key));
}

/// 1 for the first encounter or a new zone, otherwise one more than the previous entry's count.
fn next_encounter_number_in_zone(actions: AppActions, zone_name: &str) -> usize {
    let mut counter = actions.history.encounters_in_current_zone;
    let previous_zone = actions.history.encounter_history.peek().first().map(|entry| entry.zone_name().to_string());
    let next_number = match previous_zone {
        Some(previous) if previous == zone_name => *counter.peek() + 1,
        _ => 1,
    };
    counter.set(next_number);
    next_number
}