//! The section only this port has, at the bottom of the "Custom UI Data" page.
//!
//! These options are NOT part of MopiMopi's settings: they are kept in a store of their own and have their own export and
//! import, with a code that starts with `MOPIPORT1:`. The "Custom UI Data" code above never contains them, and pasting
//! either kind of code into the other box changes nothing. The section says so on screen, so nobody has to guess.

use super::row_context::RowContext;
use super::row_layout::settings_row;
use super::text_box::{clear_typed_text, text_box, typed_text};
use crate::application::app_state::{show_toast_message, PortPreferencesContext};
use crate::domain::settings::{is_hex_color, PortPreferences};
use crate::presentation::ui::shared::switch_and_icon::SwitchToggle;
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIcon;

const COLOR_BOX: &str = "in_portNpcColor";
const IMPORT_BOX: &str = "in_portApply";

pub(super) fn port_section(page_context: &RowContext) -> Element {
    let (actions, inputs) = (page_context.actions, page_context.typed_texts);
    let context = consume_context::<PortPreferencesContext>(); // not use_context: this runs only on one page, so it must not be a hook
    let preferences = context.preferences.read().clone();
    let code = preferences.to_code();

    let toggled = {
        let preferences = preferences.clone();
        move |_| context.set(PortPreferences { npc_own_look: !preferences.npc_own_look, ..preferences.clone() })
    };
    let set_color = {
        let preferences = preferences.clone();
        move |text: String| {
            let color = text.trim().trim_start_matches('#').to_string();
            if is_hex_color(&color) {
                context.set(PortPreferences { npc_color: color.to_uppercase(), ..preferences.clone() });
                clear_typed_text(inputs, COLOR_BOX);
                show_toast_message(actions, "submit", 500, 3000);
            } else {
                show_toast_message(actions, "notData", 500, 3000);
            }
        }
    };
    let import = move |text: String| {
        match PortPreferences::from_code(&text) {
            Ok(imported) => {
                context.set(imported);
                clear_typed_text(inputs, IMPORT_BOX);
                show_toast_message(actions, "submit", 500, 3000);
            }
            Err(_) => show_toast_message(actions, "notData", 500, 3000),
        }
    };
    let color_box = text_box(inputs, COLOR_BOX, &format!("Colour of NPC rows, 6 hex digits (now {})", preferences.npc_color), set_color);
    let import_box = text_box(inputs, IMPORT_BOX, "Paste a MOPIPORT1: code", import);
    rsx! {
        div { id: "port-only-section",
            li { class: "li_box", style: "border:0",
                {settings_row(
                    "info",
                    "Port-only options",
                    Some(("ac", "Only this port has these. They are not MopiMopi settings, are saved separately, and are never part of the Custom UI Data code above. They have their own export and import below.")),
                    None,
                )}
            }
            li { id: "portNpcLook", onclick: toggled,
                {settings_row(
                    "person",
                    "NPCs look like themselves",
                    Some(("ac", "Off: trust and duty-support NPCs are drawn like the Limit Break, as in MopiMopi. On: they get their own icon and colour.")),
                    Some(rsx! { SwitchToggle { is_on: preferences.npc_own_look } }),
                )}
            }
            li { class: "li_text", style: "border:0",
                table { tbody { tr {
                    td { class: "gIcon", MaterialIcon { name: "palette" } }
                    td { style: "width:100%;padding-right:1.4rem", {color_box} }
                } } }
            }
            li { class: "li_text", style: "border:0",
                table { tbody { tr {
                    td { class: "gIcon", MaterialIcon { name: "share" } }
                    td { style: "width:100%;padding-right:1.4rem",
                        div { class: "inputBox",
                            input { class: "inputEff", id: "in_portShare", r#type: "text", readonly: true, value: "{code}" }
                            span { class: "focus-border" }
                        }
                    }
                } } }
            }
            li { class: "li_text", style: "border:0",
                table { tbody { tr {
                    td { class: "gIcon", rowspan: "2", MaterialIcon { name: "code" } }
                    td { style: "width:100%;", {import_box} }
                    td { class: "gIcon ft sendBtn", rowspan: "2",
                        onclick: move |_| import(typed_text(inputs, IMPORT_BOX)),
                        MaterialIcon { name: "send" }
                    }
                } } }
            }
        }
    }
}
