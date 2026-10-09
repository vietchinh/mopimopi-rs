//! The application state, one context per section of the page. Each component reads the context of its own section
//! (`use_context::<TablesContext>()`), so it only knows the state it draws, and is only redrawn for that.
//!
//! * `settings_context`         – the user's settings (every section reads them)
//! * `screen_context`           – which full screen is shown
//! * `tables_context`           – the tables: displayed data, local player, standby, blurred rows
//! * `history_context`          – the History screen: finished encounters
//! * `settings_screen_context`  – the settings screens: page, tab, sample tables
//! * `navigation_bar_context`   – the top bar's buttons
//! * `dropdown_context`         – the open pop-up menu
//! * `notices_context`          – the toast and the tooltip
//!
//! Something that changes several sections at once (opening the settings, a new fight) is an action of
//! `AppActions`, which holds them all but lets nobody read them.

mod dropdown_context;
mod history_context;
mod navigation_bar_context;
mod notices_context;
mod screen_context;
mod settings_context;
mod settings_screen_context;
mod tables_context;

pub use dropdown_context::{Dropdown, DropdownContext};
pub use history_context::HistoryContext;
pub use navigation_bar_context::NavigationBarContext;
pub use notices_context::NoticesContext;
pub use screen_context::{Screen, ScreenContext};
pub use settings_context::SettingsContext;
pub use settings_screen_context::{SettingsLocation, SettingsScreenContext};
pub use tables_context::TablesContext;
