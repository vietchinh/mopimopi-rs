//! Tab buttons of a settings page.

use super::page_content::{current_tab, tabs_of};
use super::row_context::RowContext;
use crate::application::app_state::*;
use crate::domain::translations::translate;
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;

pub(super) fn tab_bar(page_context: &RowContext, nav: &SettingsLocation) -> Element {
    let context = page_context.context;
    let selected = current_tab(&nav.page, &nav.tab);
    let buttons = tabs_of(&nav.page).into_iter().map(|(id, def)| {
        let on = selected.as_deref() == Some(id.as_str());
        let width = def["w"].as_f64().unwrap_or(25.0);
        let title = translate(&def["tt"], page_context.language_code);
        let target = id.clone();
        rsx! {
            div { key: "{id}", "name": "{id}", class: "tab_box", style: "width:{width}%", onclick: move |_| select_settings_tab(context, &target),
                div { class: if on { "tab_title on" } else { "tab_title" }, {markup_view(&title)} }
                div { class: if on { "tab_underBar on_bar" } else { "tab_underBar" } }
            }
        }
    });
    rsx! { div { class: "tabArea", {buttons} } }
}
