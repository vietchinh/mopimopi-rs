//! A built-in sample fight (from the original's `previewLog.json`).

use crate::domain::combat::Encounter;
use crate::infrastructure::act::data::encounter_from_json;

/// The sample fight the settings screens preview. Whether its pets are folded into their owners is decided where it is
/// shown (`Encounter::rows`), like for a live fight.
pub fn sample_encounter() -> &'static Encounter {
    // Built once per thread and kept for the life of the page; the app is single-threaded, and an Encounter shares its data with `Rc`.
    thread_local! {
        static SAMPLE: &'static Encounter =
            Box::leak(Box::new(encounter_from_json(include_str!("../../data/previewLog.json")).expect("data/previewLog.json is valid combat data")));
    }
    SAMPLE.with(|sample| *sample)
}
