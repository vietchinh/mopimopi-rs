//! The standard layout of one settings row.

use crate::presentation::ui::shared::switch_and_icon::RowIcon;
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;

/// Standard row: icon on the left, title, optional grey second line, optional control on the right.
/// `sub` = (css class of the second line, its html text).
pub(super) fn settings_row(icon: &str, title: &str, sub: Option<(&str, &str)>, right: Option<Element>) -> Element {
    let span = if sub.is_some() { "2" } else { "1" };
    let has_right = right.is_some();
    rsx! {
        table {
            tbody {
                tr {
                    td { class: "gIcon", rowspan: "{span}", RowIcon { icon: icon.to_string() } }
                    td { class: "gTitle", {markup_view(title)} }
                    if has_right {
                        td { rowspan: "{span}", style: "padding:0 1.4rem", {right} }
                    }
                }
                if let Some((class, text)) = sub {
                    tr { td { class: "gVal {class}", {markup_view(text)} } }
                }
            }
        }
    }
}
