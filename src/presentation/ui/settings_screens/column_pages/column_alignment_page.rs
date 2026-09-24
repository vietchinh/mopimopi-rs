//! Left / center / right alignment of each column's header and body.

use super::column_titles::{active_columns, column_title};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::application::app_state::*;
use crate::domain::translations::{translate, translations};
use dioxus::prelude::*;

/// Left / center / right for each column's header and body.
pub(super) fn column_alignment_page(page_context: &RowContext) -> Element {
    let context = page_context.context;
    let d = &translations().dictionary;
    let boxes = active_columns(page_context).into_iter().map(|col| {
        let rows = [("alignHeader", "header"), ("alignBody", "body")].into_iter().map(|(field, label_key)| {
            let title = column_title(page_context, &col, &format!(" ❯ {}", translate(&d[label_key], page_context.language_code)));
            let current = page_context.settings.column_text(&col, field);
            let current_label = translate(&d["direction"][current.as_str()], page_context.language_code);
            let target = col.clone();
            rsx! {
                li { key: "{field}", id: "{field}_{col}", class: "radio",
                    onclick: move |_| {
                        let mut open_dropdown = context.open_dropdown;
                        open_dropdown.set(Some(Dropdown::ChooseColumnAlignment { column: target.clone(), field: field.to_string() }));
                    },
                    {settings_row("arrow_right", &title, Some(("ac", &current_label)), None)}
                }
            }
        });
        rsx! { ul { key: "{col}", class: "group shadow", {rows} } }
    });
    rsx! { {boxes} }
}
