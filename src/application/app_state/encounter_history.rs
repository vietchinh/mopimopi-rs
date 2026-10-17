//! The list of finished encounters (History screen).

use super::app_actions::AppActions;
use super::contexts::Screen;
use super::standby_mode::restart_standby_timer;
use crate::domain::combat::EncounterKey;
use dioxus::prelude::*;

pub fn open_history_screen(actions: AppActions) {
    let mut screen = actions.screen.current_screen;
    screen.set(Screen::History);
    let mut dropdown = actions.dropdown.open_dropdown;
    dropdown.set(None);
}

pub fn close_history_screen(actions: AppActions) {
    let mut screen = actions.screen.current_screen;
    screen.set(Screen::Main);
    restart_standby_timer(actions);
}

pub fn show_history_entry(actions: AppActions, encounter_key: &EncounterKey) {
    let picked = actions.history.encounter_history.peek().find(encounter_key).map(|entry| entry.encounter().clone());
    let Some(combat_data) = picked else { return };
    let mut displayed = actions.tables.displayed_combat_data;
    displayed.set(Some(combat_data));
    let mut viewed = actions.history.viewed_history_key;
    viewed.set(Some(encounter_key.clone()));
    let mut has_received_data = actions.tables.has_received_data;
    has_received_data.set(true);
    close_history_screen(actions);
}