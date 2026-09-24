//! Application state shared by all components, and the actions that change it.
//!
//! Components read the state through `AppContext` (`use_context::<AppContext>()`); user
//! actions and incoming data change it through the functions of these files:
//!
//! * `app_context`          – the shared signals and small enums describing the UI state
//! * `sample_fight`         – built-in sample data for the settings previews and demo
//! * `data_ingestion`       – what happens when new combat data arrives
//! * `encounter_history`    – the list of finished encounters
//! * `screen_navigation`    – switching between main, history and settings screens
//! * `toast_notifications`  – short messages that slide in
//! * `standby_mode`         – hiding the tables after a period of inactivity
//! * `settings_maintenance` – reset, backup, restore, fullscreen
//! * `settings_saving`      – delayed saving of the settings

mod app_context;
mod data_ingestion;
mod encounter_history;
mod sample_fight;
mod screen_navigation;
mod settings_maintenance;
mod settings_saving;
mod standby_mode;
mod toast_notifications;

pub use app_context::{AppContext, Dropdown, Screen, SettingsLocation, ToastState};
pub use data_ingestion::handle_combat_data_received;
pub use encounter_history::{close_history_screen, open_history_screen, show_history_entry, HistoryEntry};
pub use sample_fight::sample_combat_message;
pub use screen_navigation::{
    go_back_in_settings, open_settings_page, open_settings_screen, return_to_main_screen, select_settings_tab,
    PAGES_WITH_LIVE_PREVIEW, PAGES_WITH_TABS,
};
pub use settings_maintenance::{back_up_settings, reset_settings_to_defaults, restore_settings_from_backup, toggle_fullscreen_mode};
pub use settings_saving::{register_save_on_page_hide, schedule_settings_save};
pub use toast_notifications::{dismiss_toast_message, show_toast_message};
