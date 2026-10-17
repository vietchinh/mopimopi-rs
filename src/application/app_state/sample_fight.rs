//! A built-in sample fight (from the original's `previewLog.json`).

use crate::domain::combat::Encounter;
use crate::infrastructure::act::data::encounter_from_json;
use std::sync::OnceLock;

/// The sample fight the settings screens preview. Whether its pets are folded into their owners is decided where it is
/// shown (`Encounter::rows`), like for a live fight.
pub fn sample_encounter() -> &'static Encounter {
    static SAMPLE: OnceLock<Encounter> = OnceLock::new();
    SAMPLE.get_or_init(|| encounter_from_json(include_str!("../../data/previewLog.json")).expect("data/previewLog.json is valid combat data"))
}
