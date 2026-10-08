//! Unit tests for `presentation::ui::shared::text_display`: the job icons it points at must exist.

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
