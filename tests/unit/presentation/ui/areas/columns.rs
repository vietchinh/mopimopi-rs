//! Unit tests for `areas::columns`. They are compiled as a child module of that file but live here, outside `src/`.

use super::*;
use crate::domain::settings::Settings;
use serde_json::json;

/// The width, padding and alignment the string-based code worked out for a column, written out again here as the check.
fn expected(settings: &Settings, name: &str) -> (String, String, String, String) {
    let rem = |size: f64| format!("{}rem", size / 10.0);
    let width = if name == "name" { "100%".to_string() } else { rem(settings.column_number(name, "width")) };
    let align = |field: &str| match settings.column_text(name, field).as_str() {
        "left" => "left",
        "right" => "right",
        _ => "center",
    };
    (width, format!("0 {}", rem(settings.column_number(name, "padding"))), align("alignHeader").to_string(), align("alignBody").to_string())
}

fn described(column: &Column) -> (String, String, String, String) {
    (column.width.to_string(), column.padding_css(), column.header_align.as_str().to_string(), column.body_align.as_str().to_string())
}

#[test]
fn the_columns_of_each_table_are_the_ones_switched_on_in_the_order_set() {
    let settings = Settings::defaults();
    let columns = ColumnSettings::from_raw(settings.file());
    for (is_healing, table) in [(false, "DPS"), (true, "HPS")] {
        let names: Vec<&str> = columns.of(is_healing).iter().map(|column| column.name.as_str()).collect();
        assert_eq!(names, settings.file().enabled_columns(table), "{table}");
        assert!(names.len() > 3, "{table} has its columns");
    }
}

#[test]
fn every_column_is_laid_out_as_the_string_based_code_laid_it_out() {
    let mut changed = Settings::defaults();
    changed.set_column_field("encdps", "alignHeader", json!("right"));
    changed.set_column_field("encdps", "alignBody", json!("left"));
    changed.set_column_field("encdps", "width", json!(55));
    changed.set_column_field("encdps", "padding", json!(7));
    for (label, settings) in [("default", Settings::defaults()), ("changed", changed)] {
        let columns = ColumnSettings::from_raw(settings.file());
        for is_healing in [false, true] {
            for column in columns.of(is_healing) {
                assert_eq!(described(column), expected(&settings, &column.name), "{label}: {}", column.name);
                assert_eq!(column.title, settings.column_text(&column.name, "tt"), "{label}: title of {}", column.name);
            }
        }
    }
}

#[test]
fn the_name_column_takes_the_rest_of_the_row() {
    let columns = ColumnSettings::from_raw(Settings::defaults().file());
    let name = columns.of(false).iter().find(|column| column.name == "name").expect("the damage table has a name column");
    assert_eq!(name.width, ColumnWidth::Fill);
    assert!(columns.of(false).iter().filter(|column| column.name != "name").all(|column| matches!(column.width, ColumnWidth::Fixed(_))));
}

#[test]
fn a_column_switched_off_for_a_table_is_not_in_it_even_when_the_order_still_lists_it() {
    let mut settings = Settings::defaults();
    let listed = settings.column_order("DPS")[1].clone();
    settings.set_column_field(&listed, "DPS", json!(0)); // the order still has it, as a hand-edited file can
    let columns = ColumnSettings::from_raw(settings.file());
    assert!(!columns.of(false).iter().any(|column| column.name == listed));
    assert_eq!(columns.of(false).len(), settings.column_order("DPS").len() - 1);
}
