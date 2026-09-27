//! A built-in sample fight (from the original's `previewLog.json`).

use std::sync::OnceLock;
use crate::infrastructure::act::data::CombatDataMessage;

pub fn sample_combat_message() -> &'static CombatDataMessage {
    static SAMPLE: OnceLock<CombatDataMessage> = OnceLock::new();
    SAMPLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/previewLog.json")).expect("data/previewLog.json is valid combat data")
    })
}
