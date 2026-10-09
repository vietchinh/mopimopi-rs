//! Helpers for the pages that list the active columns.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::application::i18n::translate;
use crate::domain::translations::translations;

/// Columns that are switched on in at least one table.
pub(super) fn active_columns(page_context: &RowContext) -> Vec<String> {
    page_context.settings.columns_enabled_anywhere()
}

/// "Job  ❙ description of the column" – title used on all four column pages.
pub(super) fn column_title(page_context: &RowContext, col: &str, suffix: &str) -> String {
    let explanation = translate(&translations().dictionary[col]["tt"]);
    format!("{}<font class=\"ex\">　❙ {explanation}{suffix}</font>", page_context.settings.column_text(col, "tt"))
}
