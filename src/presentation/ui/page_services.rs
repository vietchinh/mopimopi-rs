//! Jobs that run for as long as the page does, each a component of its own that draws nothing. `App` lists them; what each one
//! needs it reads from the contexts above it.

use crate::application::app_state::{register_save_on_page_hide, restart_standby_timer, schedule_settings_save, AppActions, NoticesContext, ScreenContext, SettingsContext, SettingsScreenContext};
use crate::presentation::ui::areas::SettingsView;
use dioxus::prelude::*;

/// Keeps the translations in the language of the `Lang` setting. The memo means only a change of language (not every settings
/// change) gets as far as the bundle.
#[component]
pub fn LanguageSync() -> Element {
    let view = use_context::<SettingsView>();
    let mut translations = dioxus_i18n::prelude::i18n();
    let language_code = use_memo(move || view.page.read().language_code.clone());
    use_effect(move || {
        let tag = crate::application::i18n::language_tag(&language_code.read());
        if translations.language() != tag {
            translations.set_language(tag);
        }
    });
    rsx! {}
}

/// Saves the settings shortly after the last change (see `settings_saving`), and at once when the page is hidden.
#[component]
pub fn SettingsSaver() -> Element {
    let settings = use_context::<SettingsContext>().settings;
    use_hook(|| register_save_on_page_hide(settings));
    let mut is_first_run = use_hook(|| true);
    use_effect(move || {
        let _ = settings.read(); // re-run after every change
        if is_first_run {
            is_first_run = false; // the settings just loaded are what is stored already
            return;
        }
        schedule_settings_save(settings);
    });
    rsx! {}
}

/// Starts the standby countdown when the page opens (a message, a screen change and so on restart it; see `standby_mode`).
#[component]
pub fn StandbyTimer() -> Element {
    let actions = use_context::<AppActions>();
    use_hook(|| restart_standby_timer(actions));
    rsx! {}
}

/// A tooltip belongs to the page it was shown on: it is dismissed when the screen or the settings page changes.
#[component]
pub fn TooltipReset() -> Element {
    let screen = use_context::<ScreenContext>();
    let settings_screen = use_context::<SettingsScreenContext>();
    let notices = use_context::<NoticesContext>();
    use_effect(move || {
        let _ = screen.current_screen.read();
        let _ = settings_screen.settings_location.read();
        let mut tooltip = notices.tooltip_html;
        if tooltip.peek().is_some() {
            tooltip.set(None);
        }
    });
    rsx! {}
}
