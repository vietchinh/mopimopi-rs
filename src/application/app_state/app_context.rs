//! The shared signals.

use crate::models::act_data::CombatDataMessage;
use crate::application::app_state::encounter_history::HistoryEntry;
use crate::domain::combat::EncounterRankings;
use crate::infrastructure::network::ConnectionStatus;
use crate::domain::settings::Settings;
use dioxus::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::rc::Rc;

/// Which full screen is shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Main,
    History,
    Settings,
}

/// The pop-up menu that is currently open.
#[derive(Clone, PartialEq, Debug)]
pub enum Dropdown {
    /// The ⋮ menu; its entries depend on the current screen.
    Navigation,
    /// Pick one value of a setting: `setting_key` and the schema's value -> label object.
    ChooseOption { setting_key: String, choices: Value },
    /// Several on/off settings as (setting key, label) pairs.
    ToggleOptions { options: Vec<(String, String)> },
    /// Left / center / right for a table column (`field` is "alignHeader" or "alignBody").
    ChooseColumnAlignment { column: String, field: String },
}

/// Which settings page and tab are open.
#[derive(Clone, PartialEq, Debug)]
pub struct SettingsLocation {
    pub page: String,
    pub tab: Option<String>,
}

impl SettingsLocation {
    pub fn top_level() -> Self {
        SettingsLocation { page: "Settings".into(), tab: None }
    }
}

/// The message currently sliding in or out.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ToastState {
    pub text: String,
    pub is_visible: bool,
    /// True while the message is slid into view (drives the CSS transition).
    pub is_slid_in: bool,
}

/// Every piece of shared state. All fields are signals (cheap `Copy` handles).
#[derive(Clone, Copy)]
pub struct AppContext {
    pub settings: Signal<Settings>,
    pub current_screen: Signal<Screen>,

    /// Newest data received from ACT.
    pub latest_combat_data: Signal<Option<Rc<CombatDataMessage>>>,
    /// Data currently drawn; differs from the latest while browsing history or settings.
    pub displayed_combat_data: Signal<Option<Rc<CombatDataMessage>>>,
    /// `displayed_combat_data` turned into sorted tables for the current settings.
    pub rankings: Memo<Option<Rc<EncounterRankings>>>,
    /// Rankings of the built-in sample fight (settings page previews).
    pub sample_rankings: Memo<Rc<EncounterRankings>>,

    /// Name of the local character, reported by ACT.
    pub local_player_name: Signal<String>,
    pub connection_status: Signal<ConnectionStatus>,
    /// False until the first data arrives (the start screen is shown until then).
    pub has_received_data: Signal<bool>,
    /// True while the previous message was from a running fight (used to detect fight ends).
    pub encounter_was_active: Signal<bool>,

    pub encounter_history: Signal<Vec<HistoryEntry>>,
    /// Number of encounters recorded in a row in the same zone.
    pub encounters_in_current_zone: Signal<usize>,
    /// History entry currently displayed, if any.
    pub viewed_history_key: Signal<Option<String>>,

    pub settings_location: Signal<SettingsLocation>,
    pub open_dropdown: Signal<Option<Dropdown>>,
    /// Sample tables on settings pages.
    pub settings_preview_enabled: Signal<bool>,
    pub settings_preview_raid_mode: Signal<bool>,

    pub toast_message: Signal<ToastState>,
    /// Incremented for each new message so old timers can tell they are outdated. No component
    /// reads it, so changing it never redraws anything.
    pub toast_generation: Signal<u32>,
    pub tooltip_html: Signal<Option<String>>,
    /// Standby mode: tables hidden after inactivity.
    pub is_standby_hidden: Signal<bool>,
    /// Rows whose name the user blurred by clicking the job icon.
    pub blurred_player_rows: Signal<HashSet<String>>,
    /// The hidden Capture / History / End buttons are shown (mouse over the ⋮ button).
    pub nav_buttons_expanded: Signal<bool>,
    /// The Capture button icon is blinking.
    pub capture_flash_active: Signal<bool>,
}

impl AppContext {
    pub fn language_code(&self) -> String {
        self.settings.read().language_code()
    }

    /// Applies a change to the settings (they are saved automatically).
    pub fn edit_settings(&self, change: impl FnOnce(&mut Settings)) {
        let mut settings = self.settings;
        change(&mut settings.write());
    }
}
