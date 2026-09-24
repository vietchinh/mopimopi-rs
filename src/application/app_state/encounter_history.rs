//! The list of finished encounters (History screen).

use dioxus::prelude::*;
use super::app_context::{AppContext, Screen};
use super::standby_mode::restart_standby_timer;
use crate::models::act_data::CombatDataMessage;
use std::rc::Rc;

/// One finished encounter as listed on the History screen.
#[derive(Clone)]
pub struct HistoryEntry {
    /// Unique per encounter (see `EncounterRanking::encounter_key`).
    pub encounter_key: String,
    pub title: String,
    pub zone_name: String,
    pub duration_text: String,
    pub encounter_dps: f64,
    pub encounter_hps: f64,
    pub local_player_dps: Option<f64>,
    pub local_player_hps: Option<f64>,
    /// Which encounter in a row this is within the same zone.
    pub encounter_number_in_zone: usize,
    /// The data to show again when the entry is picked.
    pub combat_data: Rc<CombatDataMessage>,
}

pub fn open_history_screen(context: AppContext) {
    let mut screen = context.current_screen;
    screen.set(Screen::History);
    let mut dropdown = context.open_dropdown;
    dropdown.set(None);
}

pub fn close_history_screen(context: AppContext) {
    let mut screen = context.current_screen;
    screen.set(Screen::Main);
    restart_standby_timer(context);
}

/// Shows a past encounter on the main screen.
pub fn show_history_entry(context: AppContext, encounter_key: &str) {
    let picked = context
        .encounter_history
        .peek()
        .iter()
        .find(|entry| entry.encounter_key == encounter_key)
        .map(|entry| entry.combat_data.clone());
    let Some(combat_data) = picked else { return };
    let mut displayed = context.displayed_combat_data;
    displayed.set(Some(combat_data));
    let mut viewed = context.viewed_history_key;
    viewed.set(Some(encounter_key.to_string()));
    let mut has_received_data = context.has_received_data;
    has_received_data.set(true);
    close_history_screen(context);
}
