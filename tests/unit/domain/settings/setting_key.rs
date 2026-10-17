use super::*;
use crate::domain::settings::Settings;
use serde_json::Value;
use std::collections::HashSet;

#[test]
fn every_key_is_a_setting_the_defaults_know_so_a_renamed_setting_cannot_slip_through() {
    let defaults = Settings::defaults();
    for key in SettingKey::ALL {
        let known = defaults.option_value(key.id()) != Value::Null || defaults.slider_value(key.id()) != 0.0;
        assert!(known || key == SettingKey::BackupDate, "{key:?} ({}) is not in the defaults", key.id());
    }
}

#[test]
fn keys_have_distinct_ids() {
    let ids: HashSet<&str> = SettingKey::ALL.iter().map(|key| key.id()).collect();
    assert_eq!(ids.len(), SettingKey::ALL.len());
}
