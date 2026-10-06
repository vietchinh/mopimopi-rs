//! Root component: builds the shared state and the page shell around the current screen.

use super::dropdown_menus::DropdownMenu;
use super::history_screen::{HistoryNavigationBar, HistoryScreen};
use super::navigation_bar::NavigationBar;
use super::overlay_plugin_context;
use super::overlay_plugin_context::OverlayPluginContext;
use super::overlays::{Toast, Tooltip};
use super::settings_screens::{SettingsNavigationBar, SettingsScreen};
use super::start_screen::MainScreen;
use crate::application::app_state::{AppContext, Screen, SettingsLocation, Dropdown, ToastState, on_combat_data_changed, register_save_on_page_hide, schedule_settings_save, restart_standby_timer};
use crate::domain::settings::Settings;
use dioxus::prelude::*;
use std::collections::HashSet;

/// Creates every piece of shared state (see `app_state::AppContext` for what each one means).
fn create_app_context() -> AppContext {
    let settings = use_signal(Settings::load_from_browser);

    AppContext {
        settings,
        displayed_combat_data: use_signal(|| None),
        local_player_name: use_signal(String::new),
        current_screen: use_signal(|| Screen::Main),
        has_received_data: use_signal(|| false),
        encounter_was_active: use_signal(|| false),
        encounter_history: use_signal(Vec::new),
        encounters_in_current_zone: use_signal(|| 0usize),
        viewed_history_key: use_signal(|| None::<String>),
        settings_location: use_signal(SettingsLocation::top_level),
        open_dropdown: use_signal(|| None::<Dropdown>),
        settings_preview_enabled: use_signal(|| false),
        settings_preview_raid_mode: use_signal(|| false),
        toast_message: use_signal(ToastState::default),
        toast_generation: use_signal(|| 0u32),
        tooltip_html: use_signal(|| None::<String>),
        is_standby_hidden: use_signal(|| false),
        blurred_player_rows: use_signal(HashSet::<String>::new),
        nav_buttons_expanded: use_signal(|| false),
        capture_flash_active: use_signal(|| false),
    }
}

// tailwind.css is a Dioxus asset, so `dx` minifies it and gives it a content-hashed file name (long-term
// cacheable). It is the generated part of the CSS: the Preflight undo, the utilities, and the parts of the
// original overlay's stylesheet that do not paint on a refresh (menus, settings screens, forms, tooltip).
//
// The base layout is NOT here. It is `public/base.css`, a static, render-blocking `<link>` in the page's head
// (`style` in Dioxus.toml, `<link>` in web/index.html), because a `document::Stylesheet` only reaches the page
// after the wasm has run and drawn its first frame, and until then that frame has no app CSS at all: the page
// bounced on every refresh. Bare local paths in `[web.resource] style` are not processed by `dx`, which is why
// this one is still a runtime asset: it needs the hashing, and a static file cannot have it. The layer order
// (app < legacy < utilities) is what keeps the two files' rules in the right order; the header of tailwind.css
// has the whole picture.
// Generated from `tailwind.css` at the project root: by `dx` (it writes this file itself) or by
// `build.sh`.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// The URL of an asset. Under `dx` that is the processed file (minified and content-hashed for CSS,
/// prefixed with the app's base path). A plain `cargo` build (`build.sh`) has no `dx` to process
/// assets, so `asset!` yields a placeholder instead of a path; then the plain file that `build.sh`
/// copies next to the page is used, relative so that hosting under a sub-path still works.
fn asset_url(asset: Asset, plain_file: &'static str) -> String {
    let url = asset.to_string();
    if url.contains(manganis::BundledAsset::PLACEHOLDER_HASH) { plain_file.to_string() } else { url }
}

// The clock font. Its `@font-face` can't stay in the stylesheet: the stylesheet is served from a hashed
// `/assets/` URL, so a relative `url(font/...)` inside it would point at `/assets/font/...`. As an
// asset the font gets a URL that is correct wherever the app is hosted.
const CLOCK_FONT_WOFF: Asset = asset!("/assets/font/DS-DIGIB.woff");
const CLOCK_FONT_TTF: Asset = asset!("/assets/font/DS-DIGIB.ttf");

fn clock_font_face() -> String {
    format!(
        "@font-face{{font-family:'DS-Digital';src:url({}) format('woff'),url({}) format('truetype')}}",
        asset_url(CLOCK_FONT_WOFF, "assets/font/DS-DIGIB.woff"),
        asset_url(CLOCK_FONT_TTF, "assets/font/DS-DIGIB.ttf"),
    )
}

#[component]
pub fn App() -> Element {
    let context = create_app_context();
    // Each area of the settings as a typed value, for the components that draw it (see `areas`). Created first: what is set up below
    // already needs the page's settings.
    let view = super::areas::provide_settings_view(context.settings);

    let connection_status = use_signal(|| overlay_plugin_context::ConnectionStatus::NotConfigured);
    let combat_data_message = use_signal(|| None);
    let player_name_signal = use_signal(String::new);
    let connection_error = use_signal(|| None);
    let merge_pets_into_owner = use_signal(|| view.page.peek().merge_pets);

    let overlay_plugin_context =
        OverlayPluginContext::new(connection_status, combat_data_message, player_name_signal, connection_error, merge_pets_into_owner);
    overlay_plugin_context::spawn_connection_task(connection_status, combat_data_message, player_name_signal, connection_error, merge_pets_into_owner);

    use_context_provider(|| context);
    let color_picker = use_context_provider(super::settings_screens::ColorPickerState::new);
    // Where each graph bar was last drawn, so the next change of a bar can be animated from there.
    use_context_provider(super::combat_tables::BarHistory::default);
    // Text in the language of the `Lang` setting (see application::i18n). It follows the setting; the memo means
    // only a change of language (not every settings change) gets as far as the bundle.
    let mut translations = crate::application::i18n::use_init_translations(&view.page.peek().language_code);
    let language_code = use_memo(move || view.page.read().language_code.clone());
    use_effect(move || {
        let tag = crate::application::i18n::language_tag(&language_code.read());
        if translations.language() != tag {
            translations.set_language(tag);
        }
    });
    use_context_provider(|| overlay_plugin_context);

    use_hook(|| restart_standby_timer(context));

    use_effect(move || {
        let merge_pets = view.page.read().merge_pets;
        overlay_plugin_context.set_merge_pets_into_owner(merge_pets);
    });

    use_effect(move || {
        if let Some(message) = overlay_plugin_context.combat_data_message() {
            on_combat_data_changed(context, message);
        }
    });
    use_effect(move || {
        let name = overlay_plugin_context.local_player_name();
        if !name.is_empty() {
            let mut local_player_name = context.local_player_name;
            local_player_name.set(name);
        }
    });

    // Save the settings shortly after the last change (see `settings_saving`).
    use_hook(|| register_save_on_page_hide(context.settings));
    use_effect(move || {
        let _ = context.settings.read(); // re-run after every change
        schedule_settings_save(context.settings);
    });

    use_effect(move || {
        let _ = context.current_screen.read();
        let _ = context.settings_location.read();
        let mut tooltip = context.tooltip_html;
        if tooltip.peek().is_some() {
            tooltip.set(None);
        }
    });

    // Page-wide variables (font size, background image, accent colour): owned here since
    // app_shell is the page's actual root, and `:root` in an HTML document *is* the `<html>`
    // element -- there's no other `rsx!` element this could attach to. Rebuilt only when the
    // settings change, not when the screen or a dropdown does. Everything else styling-related
    // is computed inline by whichever component owns the element it styles; this is the one
    // exception, because nothing else in the render tree is html's actual owner.
    let root_style = use_memo(move || view.page.read().root_rule());
    let screen = *context.current_screen.read();
    let font_face = clock_font_face();
    let show_resize_handle = screen != Screen::Settings && view.page.read().corner_handle;

    rsx! {
        // The Material Icons font (Google's stylesheet), from dioxus-material-icons.
        dioxus_material_icons::MaterialIconStylesheet {}
        document::Stylesheet { href: asset_url(TAILWIND_CSS, "assets/tailwind.css") }
        style { "{font_face}" }
        style { "{root_style}" }
        // "Resizing Arrow" (`arrow`): the original sets the handle image on #wrap at start-up and when it
        // returns from the settings, and clears it when the settings open (ui.js), so: on unless in settings.
        super::settings_screens::JsColorPicker {}
        div {
            id: "wrap",
            background_image: if show_resize_handle { "url(images/handle.svg)" },
            // Pressing anywhere but on the picker or its boxes closes the picker (they stop the event).
            onmousedown: move |_| color_picker.hide(),
            if context.open_dropdown.read().is_some() {
                DropdownMenu {}
                div {
                    id: "blackBg",
                    onclick: move |_| {
                        let mut dropdown = context.open_dropdown;
                        dropdown.set(None);
                    },
                }
            }
            match screen {
                Screen::Main => rsx! { NavigationBar { is_settings_preview: false } },
                Screen::History => rsx! { HistoryNavigationBar {} },
                Screen::Settings => rsx! { SettingsNavigationBar {} },
            }
            div { id: "content",
                Tooltip {}
                Toast {}
                match screen {
                    Screen::Main => rsx! { MainScreen {} },
                    Screen::History => rsx! { HistoryScreen {} },
                    Screen::Settings => rsx! { SettingsScreen {} },
                }
            }
        }
    }
}
