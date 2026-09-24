//! Grouping rows into boxes.

use super::page_content::SchemaEntry;
use super::row_context::RowContext;
use super::schema_rows::render_schema_row;
use dioxus::prelude::*;

/// Rows are grouped into boxes: a new box starts at every entry with `"ul": 1`.
pub(super) fn grouped_rows(page_context: &RowContext, items: &[SchemaEntry]) -> Element {
    let mut boxes: Vec<Vec<&SchemaEntry>> = vec![];
    for item in items {
        if item.definition["ul"] == 1 || boxes.is_empty() {
            boxes.push(vec![]);
        }
        boxes.last_mut().unwrap().push(item);
    }
    let rendered = boxes.into_iter().enumerate().map(|(i, group)| {
        let rows = group.into_iter().map(|item| render_schema_row(page_context, item));
        rsx! { ul { key: "{i}", class: "group shadow", {rows} } }
    });
    rsx! { {rendered} }
}

