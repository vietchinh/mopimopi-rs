//! Dioxus components, one folder per screen area.
//!
//! * `app_shell`          – root component: creates the shared state and the page shell
//! * `overlays`           – tooltip and toast
//! * `dropdown_menus`     – pop-up menus (⋮ menu, choice lists)
//! * `navigation_bar`     – top bar of the main screen (time, target, summary, buttons)
//! * `combat_tables`      – DPS / HPS tables and the raid grid
//! * `start_screen`       – what is shown before data arrives (language links, connect box)
//! * `history_screen`     – finished encounters
//! * `settings_screens`   – all settings pages (driven by `data/l.json`)
//! * `shared`             – helpers used by several of the above

mod app_shell;
mod combat_tables;
mod dropdown_menus;
mod history_screen;
mod navigation_bar;
mod overlays;
mod settings_screens;
mod shared;
mod start_screen;

pub use app_shell::App;
