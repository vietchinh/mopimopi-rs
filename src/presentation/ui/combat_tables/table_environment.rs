//! Everything the table drawing needs, bundled so that rows can take it as one prop.
//!
//! It is owned (no borrows) and compares by value, so a row component can be skipped when neither its own data nor this
//! changed. `CombatTables` builds it inside a memo and hands out `SharedEnvironment`s: while it stays the same, comparing
//! two of them is a pointer check.

use crate::application::i18n::dictionary_title;
use crate::domain::formatting::CellContext;
use crate::domain::formatting::LIMIT_BREAK_JOB_CODE;
use crate::domain::settings::Settings;
use crate::presentation::ui::areas::{BarSettings, ColumnSettings, RaidSettings, TableSettings};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TableEnvironment {
    pub table: TableSettings,
    pub columns: ColumnSettings,
    pub bars: BarSettings,
    pub raid: RaidSettings,
    /// "" for the real tables, "_P" for the settings preview (element ids must differ).
    pub element_id_suffix: &'static str,
    /// What the cells' text is worked out from (number format, name options, ...).
    settings: Settings,
    limit_break_name: String,
    local_player_name: String,
    /// Blurring names is only allowed while no fight is running.
    pub can_blur_names: bool,
    pub animate_bars: bool,
}

impl TableEnvironment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fight_is_running: bool,
        settings: &Settings,
        table: &TableSettings,
        columns: &ColumnSettings,
        bars: &BarSettings,
        raid: &RaidSettings,
        local_player_name: &str,
        is_settings_preview: bool,
    ) -> TableEnvironment {
        TableEnvironment {
            table: table.clone(),
            columns: columns.clone(),
            bars: bars.clone(),
            raid: raid.clone(),
            element_id_suffix: if is_settings_preview { "_P" } else { "" },
            settings: settings.clone(),
            limit_break_name: dictionary_title(LIMIT_BREAK_JOB_CODE),
            local_player_name: local_player_name.to_string(),
            can_blur_names: !is_settings_preview && !fight_is_running,
            animate_bars: bars.animate && !is_settings_preview,
        }
    }

    /// The text rules for the cells. Cheap, but not free, so a row builds it when it draws, not when it is compared.
    pub fn cell_context(&self) -> CellContext<'_> {
        CellContext::new(&self.settings, self.limit_break_name.clone(), &self.local_player_name)
    }
}

/// A `TableEnvironment` shared by every row. Two of them are equal when they are the same allocation (the usual case) or
/// hold equal values.
#[derive(Clone, Debug)]
pub(super) struct SharedEnvironment(Rc<TableEnvironment>);

impl SharedEnvironment {
    pub fn new(environment: TableEnvironment) -> Self {
        Self(Rc::new(environment))
    }
}

impl PartialEq for SharedEnvironment {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0) || *self.0 == *other.0
    }
}

impl Deref for SharedEnvironment {
    type Target = TableEnvironment;
    fn deref(&self) -> &TableEnvironment {
        &self.0
    }
}
