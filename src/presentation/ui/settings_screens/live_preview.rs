//! Sample bar and tables above the settings so changes are visible while editing.

use crate::application::app_state::AppContext;
use crate::application::i18n::translate;
use crate::domain::translations::translations;
use crate::presentation::ui::combat_tables::CombatTables;
use crate::presentation::ui::navigation_bar::NavigationBar;
use crate::presentation::ui::shared::switch_and_icon::SwitchToggle;
use dioxus::prelude::*;

/// Sample nav bar + tables so changes are visible while editing.
#[component]
pub(super) fn LivePreview() -> Element {
    let context = use_context::<AppContext>();
    let raid_mode = *context.settings_preview_raid_mode.read();
    let label = translate(&translations().ui_schema["raid"]["tab_general"]["inner"]["view24_Number"]["tt"]);
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
                tbody { tr { td { class: "text-[1.2rem]", class: if raid_mode { "text-accent" } else { "text-[rgba(189,189,189,.5)]" }, "{label}" } td { SwitchToggle { is_on: raid_mode } } } }
            }
        }
    }
}
