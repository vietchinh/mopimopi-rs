//! Everything the table drawing functions need, bundled so they don't take many arguments each.

use crate::domain::formatting::CellContext;
use crate::domain::settings::Settings;
use crate::presentation::ui::areas::{BarSettings, ColumnSettings, RaidSettings, TableSettings};
use crate::application::i18n::dictionary_title;
use crate::domain::formatting::LIMIT_BREAK_JOB_CODE;
use std::collections::HashSet;

pub(super) struct TableEnvironment<'a> {
    pub table: &'a TableSettings,
    pub columns: &'a ColumnSettings,
    pub bars: &'a BarSettings,
    pub raid: &'a RaidSettings,
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
        fight_is_running: bool,
        settings: &'a Settings,
        table: &'a TableSettings,
        columns: &'a ColumnSettings,
        bars: &'a BarSettings,
        raid: &'a RaidSettings,
        local_player_name: &'a str,
        blurred_rows: &'a HashSet<String>,
        is_settings_preview: bool,
    ) -> TableEnvironment<'a> {
        TableEnvironment {
            table,
            columns,
            bars,
            raid,
            element_id_suffix: if is_settings_preview { "_P" } else { "" },
            cell_context: CellContext::new(settings, dictionary_title(LIMIT_BREAK_JOB_CODE), local_player_name),
            blurred_rows,
            can_blur_names: !is_settings_preview && !fight_is_running,
            animate_bars: bars.animate && !is_settings_preview,
        }
    }
}
