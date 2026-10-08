//! The actions that change more than one section of the page.
//!
//! `AppActions` holds the contexts these actions change, but its fields are private to `app_state`: a component can start
//! "open the settings" or "a new fight arrived" with it, and cannot read any state through it (it reads its own
//! section's context for that). It is `Copy`, so a timer can keep one.

use super::contexts::*;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct AppActions {
    pub(super) settings: SettingsContext,
    pub(super) screen: ScreenContext,
    pub(super) tables: TablesContext,
    pub(super) history: HistoryContext,
    pub(super) settings_screen: SettingsScreenContext,
    pub(super) dropdown: DropdownContext,
    pub(super) notices: NoticesContext,
}

impl AppActions {
    /// Creates the contexts the actions change, and provides each one and the actions to everything below. (The top bar's
    /// own `NavigationBarContext` is no action's business: the root provides it.) A hook: call it
    /// once, from the root component.
    pub fn provide() -> Self {
        let actions = Self {
            settings: SettingsContext::provide(),
            screen: ScreenContext::provide(),
            tables: TablesContext::provide(),
            history: HistoryContext::provide(),
            settings_screen: SettingsScreenContext::provide(),
            dropdown: DropdownContext::provide(),
            notices: NoticesContext::provide(),
        };
        use_context_provider(|| actions)
    }
}
