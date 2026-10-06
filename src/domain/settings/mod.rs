//! The user's settings.
//!
//! Settings keep the *same JSON shape* the original overlay stored in `localStorage`
//! (`q`, `Color`, `Range`, `Alias`, `Order`, `ColData`), so backups and shared "Custom UI Data"
//! codes from the original overlay import cleanly. The settings pages are generated from a
//! schema (`data/l.json`) whose entries name settings by string key, which is why settings are
//! looked up by key instead of being a fixed struct.
//!
//! * `settings_file`      – the settings file as a typed value: sections, value types, keyed get/set
//! * `import`             – JSON text -> `SettingsFile`; the one place that deals with files from other versions
//! * `user_settings`      – the `Settings` type, which wraps the file
//! * `option_access`      – read options, colours and slider values
//! * `option_updates`     – change options, colours and slider values
//! * `column_layout`      – which table columns exist, are enabled, and in which order
//! * `persistence`        – load and save
//! * `browser_storage`    – thin wrapper over `localStorage`
//! * `language_detection` – choose the first UI language from the browser
//! * `shareable_code`     – "Custom UI Data" export and import
//! * `default_settings`   – the built-in defaults (`data/defaults.json`)
//! * `json_coercion`      – JavaScript-style truthiness and number coercion of JSON values

mod browser_storage;
mod column_layout;
mod default_settings;
mod import;
mod json_coercion;
mod language_detection;
mod option_access;
mod option_updates;
mod persistence;
mod settings_file;
mod shareable_code;
mod user_settings;

pub use browser_storage::{read_local_storage, write_local_storage};
pub use persistence::BACKUP_STORAGE_KEY;
pub use settings_file::{Align, Hex, OptionValue, SettingsFile};
pub use user_settings::Settings;
