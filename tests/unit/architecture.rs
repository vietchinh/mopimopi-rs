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
