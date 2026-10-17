//! Unit tests for `settings::import`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;

const DEFAULTS_TEXT: &str = include_str!("../../../../src/data/defaults.json");

fn defaults_value() -> Value {
    serde_json::from_str(DEFAULTS_TEXT).unwrap()
}

/// Imports `document` and gives the result back as JSON, to look at one section or key.
fn imported(document: Value) -> Value {
    serde_json::to_value(import_value(document).expect("imports")).unwrap()
}

#[test]
fn the_built_in_defaults_are_a_settings_file_and_save_back_unchanged() {
    let saved = serde_json::to_string(defaults()).unwrap();
    assert_eq!(saved, serde_json::to_string(&defaults_value()).unwrap());
}

#[test]
fn text_that_is_not_a_settings_object_is_refused() {
    assert_eq!(import("{not json"), Err(ImportError::NotJson));
    for text in ["[1,2]", "7", "\"text\"", "null"] {
        assert_eq!(import(text), Err(ImportError::NotAnObject), "{text}");
    }
}

#[test]
fn a_value_of_the_wrong_kind_is_replaced_by_the_default_and_nothing_else_is_lost() {
    let mut document = defaults_value();
    document["Color"]["accent"] = json!(7); // a colour that is not text
    document["Color"]["navBg"] = json!("FF00FF"); // a setting the user changed
    document["q"]["pets"] = json!(null); // an option that is nothing
    document["q"]["gradient"] = json!(1);
    document["Range"]["sizeNav"] = json!([1]);
    document["Range"]["sizeBody"] = json!(33);
    let loaded = imported(document);
    assert_eq!(loaded["Color"]["accent"], defaults_value()["Color"]["accent"]);
    assert_eq!(loaded["q"]["pets"], defaults_value()["q"]["pets"]);
    assert_eq!(loaded["Range"]["sizeNav"], defaults_value()["Range"]["sizeNav"]);
    assert_eq!((&loaded["Color"]["navBg"], &loaded["q"]["gradient"], &loaded["Range"]["sizeBody"]), (&json!("FF00FF"), &json!(1), &json!(33)));
}

#[test]
fn a_wrong_value_for_a_key_nobody_knows_is_dropped_and_a_right_one_is_kept() {
    let mut document = defaults_value();
    document["q"]["fromTheFuture"] = json!(null);
    document["q"]["alsoFromTheFuture"] = json!("kept");
    document["Color"]["noColour"] = json!(5);
    let loaded = imported(document);
    assert!(loaded["q"].get("fromTheFuture").is_none() && loaded["Color"].get("noColour").is_none());
    assert_eq!(loaded["q"]["alsoFromTheFuture"], json!("kept"));
}

#[test]
fn a_size_written_as_text_or_as_a_switch_becomes_the_number_it_reads_as() {
    let mut document = defaults_value();
    document["Range"]["sizeNav"] = json!("50");
    document["Range"]["sizeBody"] = json!(" 12.5 ");
    document["Range"]["sizeIcon"] = json!(true);
    let loaded = imported(document);
    assert_eq!((&loaded["Range"]["sizeNav"], &loaded["Range"]["sizeBody"], &loaded["Range"]["sizeIcon"]), (&json!(50), &json!(12.5), &json!(1)));
}

#[test]
fn an_abbreviation_that_is_not_text_is_dropped_and_the_others_stay_in_order() {
    let mut document = defaults_value();
    document["Alias"] = json!({"First": "A", "Broken": 3, "Last": "Z"});
    assert_eq!(serde_json::to_string(&imported(document)["Alias"]).unwrap(), r#"{"First":"A","Last":"Z"}"#);
}

#[test]
fn a_damaged_column_takes_its_default_definition_and_an_unknown_damaged_one_is_dropped() {
    let mut document = defaults_value();
    document["ColData"]["name"]["alignBody"] = json!("justify");
    document["ColData"]["name"]["tt"] = json!("My name title");
    document["ColData"]["invented"] = json!({"tt": "x"});
    let loaded = imported(document);
    assert_eq!(loaded["ColData"]["name"], defaults_value()["ColData"]["name"]);
    assert!(loaded["ColData"].get("invented").is_none());
}

#[test]
fn a_table_whose_column_list_is_not_a_list_gets_the_default_list() {
    let mut document = defaults_value();
    document["Order"]["DPS"] = json!("name");
    document["Order"]["HPS"] = json!(["name", 7, "encdps"]);
    let loaded = imported(document);
    assert_eq!(loaded["Order"]["DPS"], defaults_value()["Order"]["DPS"]);
    assert_eq!(loaded["Order"]["HPS"], json!(["name", "encdps"]));
}

#[test]
fn a_column_newer_versions_added_is_added_switched_off() {
    let mut document = defaults_value();
    document["ColData"].as_object_mut().unwrap().shift_remove("duration");
    let loaded = imported(document);
    let added = &loaded["ColData"]["duration"];
    assert_eq!((&added["DPS"], &added["HPS"]), (&json!(0), &json!(0)));
    assert_eq!(added["tt"], defaults_value()["ColData"]["duration"]["tt"]);
}

#[test]
fn the_raid_threshold_of_older_files_is_brought_to_the_current_one() {
    let mut document = defaults_value();
    document["q"]["view24_Number"] = json!(10);
    assert_eq!(imported(document)["q"]["view24_Number"], json!(14));
    let mut chosen = defaults_value();
    chosen["q"]["view24_Number"] = json!(6);
    assert_eq!(imported(chosen)["q"]["view24_Number"], json!(6), "a threshold the user chose stays");
}

#[test]
fn the_obsolete_cell_keys_are_dropped_in_every_section() {
    let mut document = defaults_value();
    document["q"]["oldCellFlag"] = json!(1);
    document["Color"]["oldCellColour"] = json!("FF0000");
    document["Range"]["oldCellSize"] = json!(5);
    let loaded = imported(document);
    assert!(loaded["q"].get("oldCellFlag").is_none() && loaded["Color"].get("oldCellColour").is_none() && loaded["Range"].get("oldCellSize").is_none());
}

#[test]
fn sections_and_keys_this_version_does_not_know_survive() {
    let mut document = defaults_value();
    document["Future"] = json!({"a": [1, 2]});
    document["ColData"]["name"]["fromTheFuture"] = json!("kept");
    let loaded = imported(document);
    assert_eq!(loaded["Future"], json!({"a": [1, 2]}));
    assert_eq!(loaded["ColData"]["name"]["fromTheFuture"], json!("kept"));
}
