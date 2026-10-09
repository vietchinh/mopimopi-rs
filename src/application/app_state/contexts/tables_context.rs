//! The tables section: the data they draw and what decides whether they are shown.

use crate::infrastructure::act::data::CombatDataMessage;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct TablesContext {
    /// Data currently drawn by the tables: mirrors what `OverlayPlugin` last sent, except while browsing history or
    /// editing settings, when it is frozen on a chosen message instead (see `data_ingestion` and `encounter_history`).
    pub displayed_combat_data: Signal<Option<Rc<CombatDataMessage>>>,
    /// Name of the local character, reported by ACT.
    pub local_player_name: Signal<String>,
    /// False until the first data arrives (the start screen is shown until then).
    pub has_received_data: Signal<bool>,
    /// True while the previous message was from a running fight (used to detect fight ends).
    pub encounter_was_active: Signal<bool>,
    /// Rows whose name the user blurred by clicking the job icon.
    pub blurred_player_rows: Signal<HashSet<String>>,
    /// Standby mode: tables hidden after inactivity.
    pub is_standby_hidden: Signal<bool>,
}

impl TablesContext {
    /// A hook: call it once, from the root component.
    pub fn provide() -> Self {
        let context = Self {
            displayed_combat_data: use_signal(|| None),
            local_player_name: use_signal(String::new),
            has_received_data: use_signal(|| false),
            encounter_was_active: use_signal(|| false),
            blurred_player_rows: use_signal(HashSet::<String>::new),
            is_standby_hidden: use_signal(|| false),
        };
        use_context_provider(|| context)
    }
}
