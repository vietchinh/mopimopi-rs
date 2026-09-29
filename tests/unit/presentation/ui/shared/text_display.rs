//! Unit tests for `presentation::ui::shared::text_display`: the job icons it points at must exist.

use super::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn icon_names(set_directory: &Path) -> BTreeSet<String> {
    fs::read_dir(set_directory)
        .expect("icon set folder can be read")
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".png"))
        .collect()
}

#[test]
fn every_icon_set_contains_the_same_icons() {
    let icons = Path::new(env!("CARGO_MANIFEST_DIR")).join("public/images/icon");
    let sets: Vec<_> = fs::read_dir(&icons).unwrap().filter_map(|entry| entry.ok()).filter(|entry| entry.path().is_dir()).collect();
    assert!(sets.len() >= 12, "expected all icon sets, found {}", sets.len());

    let reference = icon_names(&icons.join("frame"));
    for set in sets {
        let names = icon_names(&set.path());
        let missing: Vec<_> = reference.difference(&names).collect();
        let extra: Vec<_> = names.difference(&reference).collect();
        assert!(missing.is_empty() && extra.is_empty(), "icon set {:?}: missing {missing:?}, unexpected {extra:?}", set.file_name());
    }
}

#[test]
fn beastmaster_has_an_icon_in_every_set() {
    let icons = Path::new(env!("CARGO_MANIFEST_DIR")).join("public/images/icon");
    for set in fs::read_dir(&icons).unwrap().filter_map(|entry| entry.ok()).filter(|entry| entry.path().is_dir()) {
        assert!(set.path().join("BST.png").is_file(), "BST.png missing in {:?}", set.file_name());
    }
}

#[test]
fn the_list_of_icon_codes_is_exactly_the_icon_files() {
    let icons = Path::new(env!("CARGO_MANIFEST_DIR")).join("public/images/icon");
    let files: BTreeSet<String> = icon_names(&icons.join("frame")).into_iter().map(|name| name.trim_end_matches(".png").to_string()).collect();
    let listed: BTreeSet<String> = ICON_CODES.iter().map(|code| code.to_string()).collect();
    assert_eq!(listed, files, "ICON_CODES in text_display.rs and the files in public/images/icon must list the same jobs");
    assert!(files.contains("LMB"), "the fallback icon has to exist");
}

#[test]
fn a_job_without_an_icon_shows_limit_break_s() {
    for known in ["PLD", "VPR", "PCT", "AVA", "CBO", "LMB", "BST"] {
        assert_eq!(icon_code(known), known);
    }
    for unknown in ["", "XYZ", "0", "pld", "Limit Break"] {
        assert_eq!(icon_code(unknown), "LMB", "{unknown:?}");
    }
}
