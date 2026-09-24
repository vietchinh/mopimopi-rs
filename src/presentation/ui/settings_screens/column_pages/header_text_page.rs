//! Custom header titles for each column.

use super::column_titles::{active_columns};
use crate::presentation::ui::settings_screens::form_actions::submit_text;
use crate::presentation::ui::settings_screens::text_box::{text_box, typed_text};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::domain::translations::{translate, translations};
use crate::presentation::ui::shared::switch_and_icon::RowIcon;
use dioxus::prelude::*;

/// Custom header titles: a note, then (current title + text box) per column.
pub(super) fn header_text_page(page_context: &RowContext) -> Element {
    let (context, inputs) = (page_context.context, page_context.typed_texts);
    let d = &translations().dictionary;
    let placeholder = translate(&d["headerText"], page_context.language_code);

    let boxes = active_columns(page_context).into_iter().map(|col| {
        let label = translate(&d[col.as_str()]["tt"], page_context.language_code);
        let current = page_context.settings.column_text(&col, "tt");
        let box_id = format!("headerText_{col}");
        let on_enter = {
            let box_id = box_id.clone();
            move |text: String| submit_text(context, inputs, &box_id, text)
        };
        let input = text_box(inputs, &box_id, &placeholder, on_enter);
        let send_id = box_id.clone();
        rsx! {
            ul { key: "{col}", class: "group shadow",
                li { class: "li_box", {settings_row("arrow_right", &label, Some(("ac", &current)), None)} }
                li { class: "li_text", style: "border:0",
                    table { tbody { tr {
                        td { class: "gIcon", RowIcon { icon: "text_fields".to_string() } }
                        td { style: "width:100%", {input} }
                        td { class: "gIcon ft sendBtn",
                            onclick: move |_| submit_text(context, inputs, &send_id, typed_text(inputs, &send_id)),
                            RowIcon { icon: "send".to_string() }
                        }
                    } } }
                }
            }
        }
    });
    rsx! {
        {boxes}
    }
}
