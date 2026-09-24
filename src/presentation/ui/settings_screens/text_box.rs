//! Text boxes whose content is kept in the page's `TypedTexts`.

use super::row_context::TypedTexts;
use dioxus::prelude::*;

/// Text box whose content lives in `inputs[id]`. `on_enter` is called with the text on Enter.
pub(super) fn text_box(inputs: TypedTexts, id: &str, placeholder: &str, on_enter: impl Fn(String) + 'static) -> Element {
    let text = inputs.read().get(id).cloned().unwrap_or_default();
    let (for_input, for_key) = (id.to_string(), id.to_string());
    rsx! {
        div { class: "inputBox",
            input {
                class: "inputEff",
                r#type: "text",
                id: "{id}",
                placeholder: "{placeholder}",
                value: "{text}",
                oninput: move |event| {
                    let mut all = inputs;
                    all.write().insert(for_input.clone(), event.value());
                },
                onkeydown: move |event| {
                    if event.key() == Key::Enter {
                        let typed = inputs.peek().get(&for_key).cloned().unwrap_or_default();
                        on_enter(typed);
                    }
                },
            }
            span { class: "focus-border" }
        }
    }
}

/// Current text of a box (empty if untouched).
pub(super) fn typed_text(inputs: TypedTexts, id: &str) -> String {
    inputs.peek().get(id).cloned().unwrap_or_default()
}


pub(super) fn clear_typed_text(inputs: TypedTexts, id: &str) {
    let mut all = inputs;
    all.write().remove(id);
}
