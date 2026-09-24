//! Values every settings row builder needs.

use crate::application::app_state::AppContext;
use crate::domain::settings::Settings;
use dioxus::prelude::*;
use std::collections::HashMap;

/// Text typed into the page's text boxes, keyed by the box id (`in_fTime`, `headerText_dps`, ...).
pub(super) type TypedTexts = Signal<HashMap<String, String>>;

/// Shared context passed to every row builder.
pub(super) struct RowContext<'a> {
    pub(super) context: AppContext,
    pub(super) settings: &'a Settings,
    pub(super) language_code: &'a str,
    pub(super) typed_texts: TypedTexts,
}
