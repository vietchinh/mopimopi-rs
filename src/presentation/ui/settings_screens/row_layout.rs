//! The standard layout of one settings row.

use crate::presentation::ui::shared::switch_and_icon::RowIcon;
use crate::presentation::ui::shared::safe_markup::markup_view;
use dioxus::prelude::*;

/// Standard row: icon on the left, title, optional grey second line, optional control on the right
/// (in a cell with the original's `padding:0 1.4rem`).
/// `sub` = (css class of the second line, its html text).
pub(super) fn settings_row(icon: &str, title: &str, sub: Option<(&str, &str)>, right: Option<Element>) -> Element {
    row(icon, title, sub, right, None)
}

/// Like `settings_row`, but the right-hand cell is an icon cell of its own class (`gIcon` for a link's
/// arrow, `gIcon removeBtn` for a remove button) instead of the padded control cell -- what the original
/// builds for those two row kinds.
pub(super) fn settings_row_with_icon_cell(icon: &str, title: &str, sub: Option<(&str, &str)>, right: Element, right_class: &str) -> Element {
    row(icon, title, sub, Some(right), Some(right_class))
}

fn row(icon: &str, title: &str, sub: Option<(&str, &str)>, right: Option<Element>, right_class: Option<&str>) -> Element {
    // The original only writes `rowspan` (as 2) on the cells of a two-line row.
    let span = sub.is_some().then_some("2");
    let has_right = right.is_some();
    rsx! {
        table {
            tbody {
                tr {
                    td { class: "gIcon", rowspan: span, RowIcon { icon: icon.to_string() } }
                    td { class: "gTitle", {markup_view(title)} }
                    if has_right {
                        if let Some(class) = right_class {
                            td { class: "{class}", rowspan: span, {right} }
                        } else {
                            td { rowspan: span, style: "padding:0 1.4rem", {right} }
                        }
                    }
                }
                if let Some((class, text)) = sub {
                    tr { td { class: "gVal {class}", {markup_view(text)} } }
                }
            }
        }
    }
}
