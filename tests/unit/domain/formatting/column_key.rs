use super::*;

#[test]
fn every_column_of_the_defaults_is_known_and_round_trips() {
    let defaults: serde_json::Value = serde_json::from_str(include_str!("../../../../src/data/defaults.json")).unwrap();
    let keys: Vec<&String> = defaults["ColData"].as_object().unwrap().keys().collect();
    assert_eq!(keys.len(), 43);
    for key in keys {
        let column = ColumnKey::from_key(key);
        assert!(!matches!(column, ColumnKey::Unknown(_)), "{key} is not named");
        assert_eq!(column.key(), key);
    }
}

#[test]
fn an_unknown_key_is_kept_and_written_back_unchanged() {
    let column = ColumnKey::from_key("someFutureColumn");
    assert_eq!(column, ColumnKey::Unknown("someFutureColumn".into()));
    assert_eq!(column.key(), "someFutureColumn");
}
