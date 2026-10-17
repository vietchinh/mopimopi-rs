use super::app_actions::AppActions;
use super::contexts::Screen;
use super::standby_mode::restart_standby_timer;

use crate::domain::combat::{Encounter, EncounterEnded};
use dioxus::prelude::*;
use std::rc::Rc;

/// Called whenever `OverlayPlugin`'s combat-data signal changes. The table is redrawn while a
/// fight runs and once when it ends (then the encounter is also added to the history). While the
/// settings screen is open the display is left alone so the user can edit undisturbed.
pub fn on_combat_data_changed(actions: AppActions, message: Rc<Encounter>) {
    let settings_are_open = *actions.screen.current_screen.peek() == Screen::Settings;
    let previous_message_was_active = *actions.tables.encounter_was_active.peek();
    let mut encounter_was_active = actions.tables.encounter_was_active;

    if message.is_active() {
        if !settings_are_open {
            show_message(actions, message);
        }
        encounter_was_active.set(true);
    } else if let Some(EncounterEnded { .. }) = message.ended_since(previous_message_was_active) {
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

fn show_message(actions: AppActions, message: Rc<Encounter>) {
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

/// Adds a finished encounter to the history. The rules (skip a repeat, number within the zone) are `History`'s.
fn record_finished_encounter(actions: AppActions, message: &Rc<Encounter>) {
    let mut history = actions.history.encounter_history;
    if history.peek().is_repeat(message) {
        return;
    }
    let recorded_key = history.write().record(message.clone()).map(|entry| entry.key().clone());
    if let Some(key) = recorded_key {
        let mut viewed = actions.history.viewed_history_key;
        viewed.set(Some(key));
    }
}
