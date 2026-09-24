//! Business domain: the rules of the overlay, with the objects they work on.
//! Pure Rust: no Dioxus components and no network code.
//!
//! * `combat`        – players, roles, pets, ranking and pet merging
//! * `formatting`    – how values become table text (numbers, names, cells)
//! * `settings`      – the user's settings and their rules (columns, sharing, defaults)
//! * `translations`  – translated texts and the settings-page schema

pub mod combat;
pub mod formatting;
pub mod settings;
pub mod translations;
