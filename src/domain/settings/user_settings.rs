//! The settings.

use super::settings_file::SettingsFile;

/// Settings shared with others (the "Custom UI Data" code): these sections, and not the personal ones.
pub(super) const SHAREABLE_SECTIONS: [&str; 3] = [super::settings_file::OPTIONS_SECTION, super::settings_file::COLORS_SECTION, super::settings_file::RANGES_SECTION];

/// Overlay settings: the settings file, with the readers and writers the app uses (`option_access`, `option_updates`, `column_layout`).
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub(super) file: SettingsFile,
}

impl Settings {
    /// The typed file, for what needs to see all of it.
    pub fn file(&self) -> &SettingsFile {
        &self.file
    }
}
