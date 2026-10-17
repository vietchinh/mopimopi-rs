//! The settings the program itself reads or writes, by name. The settings *file* keeps the original's keys (`swap`, `cnt`,
//! `mhh_unit`, ...) so backups and shared codes stay compatible, and the generated settings pages go through those keys as
//! text because the schema names them. Code that knows what a setting means uses this enum instead, so a typo does not
//! compile and the meaning of an obscure key is written down once.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKey {
    Language,
    /// Which of the two tables the summary line shows (damage or healing).
    SwapTables,
    AutoHide,
    AutoHideSeconds,
    Toasts,
    BackupDate,
    Palette,
    OverlayBackground,
    OverlayBackgroundImage,
    /// Show "YOU" as it is instead of the character name.
    KeepYouLabel,
    HideNames,
    PrefixRank,
    NameAbbreviation,
    AbbreviateActions,
    StrongestActionLayout,
    StrongestActionSeparator,
    ShortenValues,
    ShortenStrongestAction,
    DecimalPlaces,
    ThousandsMark,
    DecimalMark,
    RateDecimals,
    PercentDecimals,
    AmountDecimals,
}

impl SettingKey {
    #[cfg(test)]
    pub const ALL: [SettingKey; 24] = [
        SettingKey::Language,
        SettingKey::SwapTables,
        SettingKey::AutoHide,
        SettingKey::AutoHideSeconds,
        SettingKey::Toasts,
        SettingKey::BackupDate,
        SettingKey::Palette,
        SettingKey::OverlayBackground,
        SettingKey::OverlayBackgroundImage,
        SettingKey::KeepYouLabel,
        SettingKey::HideNames,
        SettingKey::PrefixRank,
        SettingKey::NameAbbreviation,
        SettingKey::AbbreviateActions,
        SettingKey::StrongestActionLayout,
        SettingKey::StrongestActionSeparator,
        SettingKey::ShortenValues,
        SettingKey::ShortenStrongestAction,
        SettingKey::DecimalPlaces,
        SettingKey::ThousandsMark,
        SettingKey::DecimalMark,
        SettingKey::RateDecimals,
        SettingKey::PercentDecimals,
        SettingKey::AmountDecimals,
    ];

    /// The key in the settings file and in the schema.
    pub const fn id(self) -> &'static str {
        match self {
            SettingKey::Language => "Lang",
            SettingKey::SwapTables => "swap",
            SettingKey::AutoHide => "autoHide",
            SettingKey::AutoHideSeconds => "autoHideTime",
            SettingKey::Toasts => "toast",
            SettingKey::BackupDate => "backupDate",
            SettingKey::Palette => "palette",
            SettingKey::OverlayBackground => "overlayBg",
            SettingKey::OverlayBackgroundImage => "overlayBgImg",
            SettingKey::KeepYouLabel => "myName",
            SettingKey::HideNames => "hideName",
            SettingKey::PrefixRank => "rank",
            SettingKey::NameAbbreviation => "cnt",
            SettingKey::AbbreviateActions => "abb",
            SettingKey::StrongestActionLayout => "mhh",
            SettingKey::StrongestActionSeparator => "mhh_unit",
            SettingKey::ShortenValues => "unit",
            SettingKey::ShortenStrongestAction => "max_unit",
            SettingKey::DecimalPlaces => "ns",
            SettingKey::ThousandsMark => "gs",
            SettingKey::DecimalMark => "ds",
            SettingKey::RateDecimals => "dpsType",
            SettingKey::PercentDecimals => "perType",
            SettingKey::AmountDecimals => "dmgType",
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/settings/setting_key.rs"]
mod tests;
