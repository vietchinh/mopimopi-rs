//! Everything the table drawing functions need, bundled so they don't take many arguments each.

use crate::application::app_state::AppContext;
use dioxus::prelude::*;
use crate::domain::formatting::CellContext;
use crate::domain::settings::Settings;
use crate::domain::translations::translations;
use std::collections::HashSet;

pub(super) struct TableEnvironment<'a> {
    pub context: AppContext,
    pub settings: &'a Settings,
    /// "" for the real tables, "_P" for the settings preview (element ids must differ).
    pub element_id_suffix: &'a str,
    pub cell_context: CellContext<'a>,
    /// Rows whose name the user blurred by clicking the job icon.
    pub blurred_rows: &'a HashSet<String>,
    /// Blurring names is only allowed while no fight is running.
    pub can_blur_names: bool,
    pub animate_bars: bool,
}

impl<'a> TableEnvironment<'a> {
    pub fn new(
        context: AppContext,
        settings: &'a Settings,
        local_player_name: &'a str,
        blurred_rows: &'a HashSet<String>,
        is_settings_preview: bool,
    ) -> TableEnvironment<'a> {
        let fight_is_running = context.latest_combat_data.read().as_ref().map(|data| data.is_encounter_active).unwrap_or(false);
        TableEnvironment {
            context,
            settings,
            element_id_suffix: if is_settings_preview { "_P" } else { "" },
            cell_context: CellContext::new(settings, translations(), local_player_name),
            blurred_rows,
            can_blur_names: !is_settings_preview && !fight_is_running,
            animate_bars: settings.option_enabled("ani") && !is_settings_preview,
        }
    }
}
