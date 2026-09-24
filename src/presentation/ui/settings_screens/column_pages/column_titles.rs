//! Helpers for the pages that list the active columns.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::domain::translations::{translate, translations};
use crate::domain::settings::is_truthy;

/// Columns that are switched on in at least one table.
pub(super) fn active_columns(page_context: &RowContext) -> Vec<String> {
    page_context
        .settings
        .column_definitions()
        .iter()
        .filter(|(_, definition)| is_truthy(&definition["DPS"]) || is_truthy(&definition["HPS"]))
        .map(|(column, _)| column.clone())
        .collect()
}

/// "Job  ❙ description of the column" – title used on all four column pages.
pub(super) fn column_title(page_context: &RowContext, col: &str, suffix: &str) -> String {
    let explanation = translate(&translations().dictionary[col]["tt"], page_context.language_code);
    format!("{}<font class=\"ex\">　❙ {explanation}{suffix}</font>", page_context.settings.column_text(col, "tt"))
}
