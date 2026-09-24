//! The on/off switch and row icons.

use super::safe_markup::markup_view;
use dioxus::prelude::*;

/// The sliding on/off switch drawn at the right of a row or menu item.
#[component]
pub fn SwitchToggle(is_on: bool) -> Element {
    rsx! {
        div { class: if is_on { "switch hover" } else { "switch" }, div { class: "toggle" } }
    }
}

/// A Material icon by name, or an inline `<img>` when the schema gives HTML (job icons on the colour pages).
#[component]
pub fn RowIcon(icon: String) -> Element {
    if icon.starts_with('<') {
        // schema-provided `<img src='./images/...'/>`; only allowlisted markup is rendered
        rsx! { span { {markup_view(&icon)} } }
    } else {
        rsx! { i { class: "material-icons", "{icon}" } }
    }
}
