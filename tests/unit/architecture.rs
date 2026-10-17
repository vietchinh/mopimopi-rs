//! Architecture test: a layer may only import from the layers below it.
//!
//! DDD principle: large-scale structure (layers) is enforced by a test, not by good intentions.
//! `ALLOWED_VIOLATIONS` is the list of known upward imports; it may only shrink (the test fails
//! if an entry no longer violates, so it must be deleted when fixed).

use std::fs;
use std::path::{Path, PathBuf};

/// Top to bottom. A file in layer N may import `crate::M` only when M >= N.
const LAYERS: [&str; 4] = ["presentation", "application", "infrastructure", "domain"];

/// (file relative to src, imported layer). Empty since the ACT translator was built (phase 5): keep it empty.
const ALLOWED_VIOLATIONS: [(&str, &str); 0] = [];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn layer_index(name: &str) -> Option<usize> {
    LAYERS.iter().position(|l| *l == name)
}

/// Layers a file imports through `crate::<layer>` paths.
fn imported_layers(source: &str) -> Vec<&'static str> {
    LAYERS
        .iter()
        .copied()
        .filter(|layer| source.contains(&format!("crate::{layer}")))
        .collect()
}

#[test]
fn layers_only_import_downwards() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut found: Vec<(String, &str)> = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        let Some(own) = relative.split('/').next().and_then(layer_index) else { continue };
        let source = fs::read_to_string(&file).unwrap();
        for layer in imported_layers(&source) {
            if layer_index(layer).unwrap() < own {
                found.push((relative.clone(), layer));
            }
        }
    }
    let allowed: Vec<(String, &str)> =
        ALLOWED_VIOLATIONS.iter().map(|(f, l)| ((*f).to_string(), *l)).collect();
    let new: Vec<_> = found.iter().filter(|v| !allowed.contains(v)).collect();
    assert!(new.is_empty(), "new upward imports: {new:?}");
    let stale: Vec<_> = allowed.iter().filter(|v| !found.contains(v)).collect();
    assert!(stale.is_empty(), "fixed; remove from ALLOWED_VIOLATIONS: {stale:?}");
}

/// Combat's open host service: everything outside `domain/combat` reaches it through the published language (the
/// names `domain::combat` itself exports), never through its inner modules.
#[test]
fn combat_is_used_only_through_its_published_language() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut offenders = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        if relative.starts_with("domain/combat/") {
            continue;
        }
        let source = fs::read_to_string(&file).unwrap();
        for line in source.lines() {
            let line = line.trim_start();
            if line.starts_with("//") {
                continue;
            }
            // `domain::combat::something::` where `something` is a module (lower case) and not a type or function.
            if let Some(rest) = line.split("domain::combat::").nth(1) {
                if rest.chars().next().is_some_and(|c| c.is_lowercase()) && rest.split(['{', ',', ';']).next().is_some_and(|p| p.contains("::")) {
                    offenders.push(format!("{relative}: {line}"));
                }
            }
        }
    }
    assert!(offenders.is_empty(), "reach into Combat's inner modules: {offenders:?}");
}

/// ACT's words stop at the translator: the wire types and field names exist only in `infrastructure/act`.
#[test]
fn acts_wire_types_stay_in_the_act_edge() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut offenders = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        if relative.starts_with("infrastructure/act/") {
            continue;
        }
        let source = fs::read_to_string(&file).unwrap();
        for word in ["CombatantRecord", "EncounterRecord", "CombatDataMessage", "OverlayMessage", "lenient_"] {
            if source.contains(word) {
                offenders.push(format!("{relative}: {word}"));
            }
        }
    }
    assert!(offenders.is_empty(), "ACT's wire types leak out of the edge: {offenders:?}");
}

fn sources_outside(prefixes: &[&str]) -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    files
        .into_iter()
        .filter_map(|file| {
            let relative = file.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
            (!prefixes.iter().any(|prefix| relative.starts_with(prefix))).then(|| (relative, fs::read_to_string(&file).unwrap()))
        })
        .collect()
}

/// ACT's conventions (job "0" is a chocobo, a blank job a pet, "Name (Owner)") are translated at the edge. The domain
/// receives placed participants and never matches on ACT's job text or name shapes.
#[test]
fn acts_conventions_stay_in_the_act_edge() {
    let mut offenders = Vec::new();
    for (relative, source) in sources_outside(&["infrastructure/act/"]) {
        for word in ["split_owner", "\"Limit Break\"", "job == \"0\"", "ParticipantKind", "Participant::sort"] {
            // production code only: the tests of a file come after its `#[cfg(test)]`
            let production = source.split("#[cfg(test)]").next().unwrap();
            if production.contains(word) {
                offenders.push(format!("{relative}: {word}"));
            }
        }
    }
    assert!(offenders.is_empty(), "ACT's conventions leak out of the edge: {offenders:?}");
}

/// A setting the program knows by meaning is read through `SettingKey`; text keys are for the pages generated from the schema.
#[test]
fn known_settings_are_not_named_by_text_literals() {
    let readers = ["option_enabled(\"", "option_number(\"", "option_text(\"", "option_value(\"", "slider_value(\"", "set_option(\"", "set_option_enabled(\"", "set_slider_value(\""];
    let mut offenders = Vec::new();
    for (relative, source) in sources_outside(&["domain/settings/"]) {
        let production = source.split("#[cfg(test)]").next().unwrap();
        for reader in readers {
            if production.contains(reader) {
                offenders.push(format!("{relative}: {reader}"));
            }
        }
    }
    assert!(offenders.is_empty(), "use SettingKey instead of a text key: {offenders:?}");
}
