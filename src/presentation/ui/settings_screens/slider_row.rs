//! Row with a range slider.

use super::row_layout::settings_row;
use crate::common::javascript_compat::number_to_javascript_string;
use dioxus::prelude::*;

/// Unit shown next to a slider value.
pub(super) fn slider_unit(id: &str) -> &'static str {
    match id {
        "sizeDPSTable" | "sizeHPSTable" | "size24TableSlice" => "",
        _ if id.contains("size") => "px",
        "autoHideTime" => "min",
        _ => "%",
    }
}

/// Settings for one slider row.
pub(super) struct SliderSpec<'a> {
    pub(super) id: &'a str,
    pub(super) icon: &'a str,
    pub(super) title: &'a str,
    pub(super) min: f64,
    pub(super) max: f64,
    pub(super) value: f64,
}

/// Row with a range slider. `on_change` receives the new value.
pub(super) fn slider_row(spec: SliderSpec, mut on_change: impl FnMut(f64) + 'static) -> Element {
    let value = number_to_javascript_string(spec.value);
    let shown = format!("{value}{}", slider_unit(spec.id));
    let control = rsx! {
        input {
            class: "shadow",
            r#type: "range",
            min: "{spec.min}",
            max: "{spec.max}",
            value: "{value}",
            oninput: move |event| {
                if let Ok(x) = event.value().parse::<f64>() {
                    on_change(x)
                }
            },
        }
    };
    let id = spec.id;
    rsx! { li { key: "{id}", id: "{id}", class: "li_box", {settings_row(spec.icon, spec.title, Some(("ac", &shown)), Some(control))} } }
}
