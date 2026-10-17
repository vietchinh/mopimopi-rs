//! Unit tests for `settings::settings_file`. They are compiled as a child module of that file
//! (so they can use its private items) but live here, outside `src/`.

use super::*;
use serde_json::json;

fn round_trip<T: Serialize + for<'a> Deserialize<'a>>(text: &str) -> String {
    serde_json::to_string(&serde_json::from_str::<T>(text).unwrap()).unwrap()
}

#[test]
fn numbers_are_written_back_as_they_were_read() {
    for text in ["21", "21.0", "0.5", "0", "-3", "100"] {
        assert_eq!(round_trip::<JsonNumber>(text), text);
    }
    // ...and a number made from a slider has no fraction when it has none
    assert_eq!(serde_json::to_string(&JsonNumber::from_f64(30.0)).unwrap(), "30");
    assert_eq!(serde_json::to_string(&JsonNumber::from_f64(0.25)).unwrap(), "0.25");
}

#[test]
fn an_option_keeps_its_kind() {
    assert_eq!(round_trip::<OptionValue>("1"), "1");
    assert_eq!(round_trip::<OptionValue>("\"1\""), "\"1\"");
    assert_eq!(round_trip::<OptionValue>("true"), "true");
    assert_eq!(round_trip::<OptionValue>("\"sans-serif\""), "\"sans-serif\"");
    assert!(matches!(serde_json::from_str::<OptionValue>("1").unwrap(), OptionValue::Number(_)));
    assert!(matches!(serde_json::from_str::<OptionValue>("\"1\"").unwrap(), OptionValue::Text(_)));
}

#[test]
fn an_option_is_truthy_like_javascript_says() {
    let truthy = |value: Value| OptionValue::from_value(&value).unwrap().is_truthy();
    for falsy in [json!(0), json!(0.0), json!(""), json!("0"), json!("false"), json!(false)] {
        assert!(!truthy(falsy.clone()), "{falsy}");
    }
    for on in [json!(1), json!(-1), json!("1"), json!("right"), json!(true)] {
        assert!(truthy(on.clone()), "{on}");
    }
}

#[test]
fn an_option_reads_as_a_number_or_as_text_the_way_the_original_did() {
    let option = |value: Value| OptionValue::from_value(&value).unwrap();
    assert_eq!(option(json!(2)).as_number(), 2.0);
    assert_eq!(option(json!(" 3 ")).as_number(), 3.0);
    assert_eq!(option(json!("left")).as_number(), 0.0);
    assert_eq!(option(json!(true)).as_number(), 1.0);
    assert_eq!(option(json!(2)).as_text(), "2");
    assert_eq!(option(json!("left")).as_text(), "left");
    assert_eq!(option(json!(true)).as_text(), "", "a switch written as true has no text");
    for not_an_option in [json!(null), json!([1]), json!({"a": 1})] {
        assert!(OptionValue::from_value(&not_an_option).is_none(), "{not_an_option}");
    }
}

#[test]
fn a_width_is_a_number_or_a_percentage_as_written() {
    assert_eq!(round_trip::<Width>("25"), "25");
    assert_eq!(round_trip::<Width>("\"100%\""), "\"100%\"");
    assert_eq!(serde_json::from_str::<Width>("25").unwrap().as_f64(), 25.0);
    assert_eq!(serde_json::from_str::<Width>("\"100%\"").unwrap().as_f64(), 0.0, "only a number is a size in tenths of a rem");
}

#[test]
fn an_alignment_is_left_center_or_right() {
    assert_eq!(round_trip::<Align>("\"left\""), "\"left\"");
    assert!(serde_json::from_str::<Align>("\"justify\"").is_err());
}

const JOB_COLUMN: &str = r#"{"tt":"Job","width":25,"padding":0,"alignHeader":"center","alignBody":"center","class":"Class","DPS":1,"HPS":1}"#;

#[test]
fn a_column_keeps_its_fields_in_the_original_order_including_the_one_it_does_not_know() {
    assert_eq!(round_trip::<ColumnData>(JOB_COLUMN), JOB_COLUMN);
    let column: ColumnData = serde_json::from_str(JOB_COLUMN).unwrap();
    assert_eq!(column.extra.get("class"), Some(&json!("Class")));
}

#[test]
fn a_column_is_read_and_changed_by_the_names_the_settings_screens_use() {
    let mut column: ColumnData = serde_json::from_str(JOB_COLUMN).unwrap();
    assert_eq!((column.text("tt"), column.text("width"), column.text("alignBody")), ("Job".into(), "25".into(), "center".into()));
    assert_eq!(column.number("width"), 25.0);
    assert!(column.is_on("DPS") && column.is_on("HPS"));
    assert_eq!(column.text("class"), "Class", "a field of the file this version does not know is still readable");
    assert_eq!(column.text("nothing"), "");

    column.set("alignBody", &json!("right"));
    column.set("DPS", &json!(0));
    column.set("width", &json!("100%"));
    column.set("padding", &json!("4"));
    column.set("tt", &json!("Role"));
    assert_eq!((column.align_body, column.is_on("DPS"), column.text("width"), column.number("padding"), column.tt.as_str()), (Align::Right, false, "100%".to_string(), 4.0, "Role"));

    // a value that does not fit is ignored, and a field nobody knows is kept
    column.set("alignHeader", &json!("justify"));
    column.set("padding", &json!({"x": 1}));
    column.set("somethingNew", &json!(7));
    assert_eq!((column.align_header, column.number("padding")), (Align::Center, 4.0));
    assert_eq!(column.extra.get("somethingNew"), Some(&json!(7)));
}

#[test]
fn a_column_with_an_unknown_alignment_is_not_a_column() {
    assert!(serde_json::from_str::<ColumnData>(&JOB_COLUMN.replace("\"alignBody\":\"center\"", "\"alignBody\":\"justify\"")).is_err());
}

#[test]
fn a_file_keeps_sections_and_keys_this_version_does_not_know() {
    let text = r#"{"q":{"pets":1,"aNewOne":"x"},"Color":{"accent":"03A9F4"},"Range":{"sizeNav":42},"Alias":{},"Order":{"DPS":["name"],"HPS":[]},"ColData":{},"Future":{"values":[1,2,3]}}"#;
    assert_eq!(round_trip::<SettingsFile>(text), text);
}

#[test]
fn setting_a_key_that_exists_keeps_its_place_and_a_new_one_goes_last() {
    let mut file: SettingsFile = serde_json::from_str(r#"{"q":{"a":1,"b":2},"Color":{},"Range":{"x":1,"y":2},"Alias":{},"Order":{},"ColData":{}}"#).unwrap();
    file.set_option("a", OptionValue::switch(false));
    file.set_option("c", OptionValue::Text("new".into()));
    file.set_range("x", 5.0);
    file.set_color("accent", "FF00FF");
    assert_eq!(serde_json::to_string(&file.options).unwrap(), r#"{"a":0,"b":2,"c":"new"}"#);
    assert_eq!(serde_json::to_string(&file.ranges).unwrap(), r#"{"x":5,"y":2}"#);
    assert_eq!((file.get_range("x"), file.get_range("none"), file.get_color("accent"), file.get_color("none")), (Some(5.0), None, Some("FF00FF"), None));
}
