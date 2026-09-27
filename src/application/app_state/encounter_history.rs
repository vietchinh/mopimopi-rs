//! The list of finished encounters (History screen).

use super::app_context::{AppContext, Screen};
use super::standby_mode::restart_standby_timer;
use crate::infrastructure::act::data::CombatDataMessage;
use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct HistoryEntry {
    /// Unique per encounter (title + totals; see `data_ingestion::record_finished_encounter`).
    pub encounter_key: String,
    /// Which encounter in a row this is within the same zone.
    pub encounter_number_in_zone: usize,
    pub combat_data: Rc<CombatDataMessage>,
}

impl HistoryEntry {
    pub fn title(&self) -> &str {
        let title = &self.combat_data.encounter.title;
        if title == "Encounter" { "No Data" } else { title }
    }

    pub fn zone_name(&self) -> &str {
        &self.combat_data.encounter.zone_name
    }

    pub fn duration_text(&self) -> &str {
        &self.combat_data.encounter.duration_text
    }

    pub fn encounter_dps(&self) -> f64 {
        self.combat_data.encounter.damage_per_second
    }

    pub fn encounter_hps(&self) -> f64 {
        self.combat_data.encounter.heal_per_second
    }

    pub fn local_player_dps(&self) -> Option<f64> {
        let duration = self.combat_data.encounter.duration_seconds;
        self.combat_data.local_player().map(|player| (player.damage / duration).floor())
    }

    pub fn local_player_hps(&self) -> Option<f64> {
        let duration = self.combat_data.encounter.duration_seconds;
        self.combat_data.local_player().map(|player| (player.healed / duration).floor())
    }
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