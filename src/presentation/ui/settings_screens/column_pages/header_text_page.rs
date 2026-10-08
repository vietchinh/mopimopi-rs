//! Custom header titles for each column.

use super::column_titles::{active_columns};
use crate::presentation::ui::settings_screens::form_actions::submit_text;
use crate::presentation::ui::settings_screens::text_box::{text_box, typed_text};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::settings_screens::row_layout::settings_row;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::shared::safe_markup::markup_view;
use crate::presentation::ui::shared::switch_and_icon::RowIcon;
use dioxus::prelude::*;

/// Custom header titles: a note, then (current title + text box) per column.
pub(super) fn header_text_page(page_context: &RowContext) -> Element {
    let (settings_context, actions, inputs) = (page_context.settings_context, page_context.actions, page_context.typed_texts);
    let d = &translations().dictionary;
    let placeholder = translate(&d["headerText"]);

    let columns = active_columns(page_context);
    // The original puts a "Read Me First" box (typing needs the overlay to take keyboard focus) in a list of its
    // own above the first column's, and only when some column is shown (`createCellPageDOM`, ui.js).
    let caution = (!columns.is_empty()).then(|| {
        let title = translate(&d["caution_tt"]);
        let note = translate(&d["caution_m1"]);
        rsx! {
            ul { class: "group shadow",
                li { class: "li_box",
                    table { tbody {
                        tr {
                            td { class: "gIcon", rowspan: "2", RowIcon { icon: "priority_high".to_string() } }
                            td { class: "gTitle", {markup_view(&title)} }
                        }
                        tr { td { class: "gVal ex", style: "padding-right:1.4rem; text-align:justify", {markup_view(&note)} } }
                    } }
                }
            }
        }
    });
    let boxes = columns.into_iter().map(|col| {
        let label = translate(&d[col.as_str()]["tt"]);
        let current = page_context.settings.column_text(&col, "tt");
        let box_id = format!("headerText_{col}");
        let on_enter = {
            let box_id = box_id.clone();
            move |text: String| submit_text(settings_context, actions, inputs, &box_id, text.as_str())
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
                            onclick: move |_| submit_text(settings_context, actions, inputs, &send_id, typed_text(inputs, &send_id).as_str()),
                            RowIcon { icon: "send".to_string() }
                        }
                    } } }
                }
            }
        }
    });
    rsx! {
        {caution}
        {boxes}
    }
}
