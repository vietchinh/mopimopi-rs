//! One line of a pop-up menu.

use crate::presentation::ui::shared::safe_markup::markup_view;
use crate::presentation::ui::shared::switch_and_icon::SwitchToggle;
use dioxus::prelude::*;

/// A clickable line with an HTML label. `switch_state` = `Some(..)` draws an on/off switch.
pub(super) fn menu_item(item_id: &str, label_html: &str, switch_state: Option<bool>, on_click: impl FnMut(()) + 'static) -> Element {
    let mut on_click = on_click;
    rsx! {
        li { key: "{item_id}", id: "{item_id}", onclick: move |_| on_click(()),
            span { {markup_view(label_html)} }
            if let Some(is_on) = switch_state { SwitchToggle { is_on } }
        }
    }
}
