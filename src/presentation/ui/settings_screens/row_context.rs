//! Values every settings row builder needs.

use crate::application::app_state::{AppActions, DropdownContext, SettingsContext};
use crate::domain::settings::Settings;
use dioxus::prelude::*;
use std::collections::HashMap;

/// Text typed into the page's text boxes, keyed by the box id (`in_fTime`, `headerText_dps`, ...).
pub(super) type TypedTexts = Signal<HashMap<String, String>>;

/// Shared context passed to every row builder.
pub(super) struct RowContext<'a> {
    /// To edit a setting.
    pub(super) settings_context: SettingsContext,
    /// To open a pop-up menu.
    pub(super) dropdown: DropdownContext,
    /// For what changes several sections (opening a page, a toast, backup / reset).
    pub(super) actions: AppActions,
    pub(super) settings: &'a Settings,
    pub(super) typed_texts: TypedTexts,
}
