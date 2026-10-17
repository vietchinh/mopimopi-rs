//! Everything the user sees.
//!
//! * `ui`  – Dioxus components (screens, tables, menus, settings pages). Styling driven by
//!           settings lives directly in whichever component owns the element it styles, as
//!           inline `--variable:value` declarations read by static Tailwind classes -- there is
//!           no separate theme-generation module.

pub mod ui;
