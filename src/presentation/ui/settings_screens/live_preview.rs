//! Sample bar and tables above the settings so changes are visible while editing.

use crate::application::app_state::AppContext;
use crate::domain::translations::{translate, translations};
use crate::presentation::ui::combat_tables::CombatTables;
use crate::presentation::ui::navigation_bar::NavigationBar;
use crate::presentation::ui::shared::switch_and_icon::SwitchToggle;
use dioxus::prelude::*;

/// Sample nav bar + tables so changes are visible while editing.
#[component]
pub(super) fn LivePreview() -> Element {
    let context = use_context::<AppContext>();
    let settings = context.settings.read();
    let raid_mode = *context.settings_preview_raid_mode.read();
    let label = translate(&translations().ui_schema["raid"]["tab_general"]["inner"]["view24_Number"]["tt"], &settings.language_code());
    let color = if raid_mode { format!("color:#{};font-size:1.2rem", settings.color_hex("accent")) } else { "color:rgba(189,189,189,.5);font-size:1.2rem".into() };
    rsx! {
        div {
            NavigationBar { is_settings_preview: true }
            div { "name": "main_P", CombatTables { is_settings_preview: true } }
            // switch to see how the sample looks in raid mode
            table {
                id: "preview24",
                onclick: move |_| {
                    let mut raid_preview = context.settings_preview_raid_mode;
                    let was_on = *raid_preview.peek();
                    raid_preview.set(!was_on);
                },
                tbody { tr { td { style: "{color}", "{label}" } td { SwitchToggle { is_on: raid_mode } } } }
            }
        }
    }
}
