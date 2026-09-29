//! Text boxes, the share code, the abbreviation add button and the background upload.

use super::{icon, note, title};
use crate::presentation::ui::settings_screens::form_actions::{add_abbreviation, submit_text, set_background};
use crate::presentation::ui::settings_screens::text_box::{text_box, typed_text};
use crate::presentation::ui::settings_screens::row_context::RowContext;
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;
use crate::presentation::ui::settings_screens::page_content::SchemaEntry;

/// Read-only box with the code others can paste into "Apply".
pub(super) fn share_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let code = page_context.settings.export_shareable_code();
    let ic = icon(entry);
    rsx! {
        li { key: "{entry.id}", class: "li_text", style: "border:0",
            table { tbody { tr {
                td { class: "gIcon", MaterialIcon { name: "{ic}" } }
                td { style: "width:100%;padding-right:1.4rem",
                    div { class: "inputBox",
                        input { class: "inputEff", id: "in_share", r#type: "text", readonly: true, value: "{code}" }
                        span { class: "focus-border" }
                    }
                }
            } } }
        }
    }
}

/// Text box, optionally with a send button. The abbreviation boxes only submit together.
pub(super) fn text_row(page_context: &RowContext, entry: &SchemaEntry, with_button: bool) -> Element {
    let (context, inputs) = (page_context.context, page_context.typed_texts);
    let box_id = entry.id.clone();
    let on_enter = {
        let box_id = box_id.clone();
        move |text: String| {
            if box_id == "in_abbOld" || box_id == "in_abbNew" {
                let filled = |k: &str| !typed_text(inputs, k).trim().is_empty();
                if filled("in_abbOld") && filled("in_abbNew") {
                    add_abbreviation(context, inputs);
                }
            } else {
                submit_text(context, inputs, &box_id, text.as_str());
            }
        }
    };
    let input = text_box(inputs, &entry.id, &note(entry), on_enter);
    let ic = icon(entry);
    let send_id = entry.id.clone();
    rsx! {
        li { key: "{entry.id}", class: "li_text", style: "border:0",
            table { tbody { tr {
                // With a send button this is the original's `li_text_inbtn`: icon and send cells span two rows and the
                // input cell has no right padding. Without one it is `li_text`.
                td { class: "gIcon", rowspan: if with_button { "2" }, MaterialIcon { name: "{ic}" } }
                td { style: if with_button { "width:100%;" } else { "width:100%;padding-right:1.4rem" }, {input} }
                if with_button {
                    td { class: "gIcon ft sendBtn", rowspan: "2",
                        onclick: move |_| submit_text(context, inputs, &send_id, typed_text(inputs, &send_id).as_str()),
                        MaterialIcon { name: "send" }
                    }
                }
            } } }
        }
    }
}

/// "Add to list" button of the abbreviation form.
pub(super) fn add_button_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let (context, inputs) = (page_context.context, page_context.typed_texts);
    let label = title(entry);
    rsx! {
        li { key: "{entry.id}", class: "gTitle sendBtn", style: "text-align:center;border-top:solid .1rem rgba(255,255,255,.07)",
            onclick: move |_| add_abbreviation(context, inputs),
            {markup_view(&label)}
        }
    }
}

/// Background image upload: a hidden file input and a button that opens it, like the original's `li_file`
/// (the full-width `<button>` look comes from the stylesheet, not from a label).
pub(super) fn file_row(page_context: &RowContext, entry: &SchemaEntry) -> Element {
    let context = page_context.context;
    let label = title(entry);
    rsx! {
        li { key: "{entry.id}", style: "padding:0; text-align:center; border:0",
            input {
                r#type: "file",
                name: "uploadFile",
                accept: "image/*",
                onchange: move |event| async move {
                    if let Some(file) = event.files().into_iter().next() {
                        if let Ok(bytes) = file.read_bytes().await {
                            let mime = file.content_type().unwrap_or_else(|| "image/png".into());
                            set_background(context, &mime, &bytes);
                        }
                    }
                },
            }
            button {
                onclick: move |_| {
                    let input = web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.query_selector("input[name=uploadFile]").ok().flatten())
                        .and_then(|element| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(element).ok());
                    if let Some(input) = input {
                        input.click();
                    }
                },
                "{label}"
            }
        }
    }
}

