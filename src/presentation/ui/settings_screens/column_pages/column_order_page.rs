//! Column order page: up / down buttons per column.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use dioxus::prelude::*;

/// Up / down buttons to reorder the columns that are switched on for a table.
pub(in crate::presentation::ui::settings_screens) fn column_order_page(page_context: &RowContext, table: &str) -> Element {
    let context = page_context.context;
    let columns: Vec<String> = page_context.settings.column_order(table).into_iter().filter(|c| page_context.settings.column_enabled_in_table(c, table)).collect();
    let rows = columns.into_iter().map(|col| {
        let name = page_context.settings.column_text(&col, "tt");
        let (up_col, down_col) = (col.clone(), col.clone());
        let (up_table, down_table) = (table.to_string(), table.to_string());
        let buttons = rsx! {
            span { class: "UBtn", onclick: move |_| context.edit_settings(|settings| settings.move_column(&up_col, &up_table, true)),
                i { class: "material-icons", "arrow_upward" }
            }
            span { style: "padding:0 1.4rem" }
            span { class: "DBtn", onclick: move |_| context.edit_settings(|settings| settings.move_column(&down_col, &down_table, false)),
                i { class: "material-icons", "arrow_downward" }
            }
        };
        rsx! { li { key: "{col}", class: "listBox", style: "cursor:default", {settings_row("arrow_right", &name, None, Some(buttons))} } }
    });
    rsx! { ul { class: "group shadow", {rows} } }
}
