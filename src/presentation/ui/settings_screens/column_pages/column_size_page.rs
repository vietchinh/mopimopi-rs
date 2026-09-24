//! Width and margin of each column, as sliders.

use super::column_titles::{active_columns, column_title};
use crate::presentation::ui::settings_screens::slider_row::{slider_row, SliderSpec};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use dioxus::prelude::*;
use serde_json::json;

/// One slider per column for its width or margin.
pub(super) fn column_size_page(page_context: &RowContext, field: &'static str, id_prefix: &str, max: f64) -> Element {
    let context = page_context.context;
    let rows = active_columns(page_context).into_iter().filter(|c| !(field == "width" && c == "name")).map(|col| {
        let title = column_title(page_context, &col, "");
        let target = col.clone();
        slider_row(
            SliderSpec { id: &format!("{id_prefix}{col}"), icon: "arrow_right", title: &title, min: 0.0, max, value: page_context.settings.column_number(&col, field) },
            move |new_value| context.edit_settings(|settings| settings.set_column_field(&target, field, json!(new_value as i64))),
        )
    });
    rsx! { ul { class: "group shadow", {rows} } }
}
