//! Column order page: up / down buttons per column.

use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::shared::safe_markup::markup_view;
use crate::presentation::ui::shared::switch_and_icon::RowIcon;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;

/// Up / down buttons to reorder the columns that are switched on for a table.
pub(in crate::presentation::ui::settings_screens) fn column_order_page(page_context: &RowContext, table: &str) -> Element {
    let context = page_context.context;
    let columns: Vec<String> = page_context.settings.column_order(table).into_iter().filter(|c| page_context.settings.column_enabled_in_table(c, table)).collect();
    let rows = columns.into_iter().map(|col| {
        let name = page_context.settings.column_text(&col, "tt");
        let (up_col, down_col) = (col.clone(), col.clone());
        let (up_table, down_table) = (table.to_string(), table.to_string());
        // The original's `li_2btn` row: icon, title, an up cell, an empty padded spacer cell, a down cell
        // (each a table cell of its own, so their widths and positions come from the table).
        rsx! {
            li { key: "{col}", class: "listBox", style: "cursor:default",
                table {
                    tbody {
                        tr {
                            td { class: "gIcon", RowIcon { icon: "arrow_right".to_string() } }
                            td { class: "gTitle", {markup_view(&name)} }
                            td { class: "UBtn", onclick: move |_| context.edit_settings(|settings| settings.move_column(&up_col, &up_table, true)),
                                MaterialIcon { name: "arrow_upward" }
                            }
                            td { style: "padding:0 1.4rem" }
                            td { class: "DBtn", onclick: move |_| context.edit_settings(|settings| settings.move_column(&down_col, &down_table, false)),
                                MaterialIcon { name: "arrow_downward" }
                            }
                        }
                    }
                }
            }
        }
    });
    rsx! { ul { class: "group shadow", {rows} } }
}
