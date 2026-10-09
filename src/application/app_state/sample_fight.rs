//! A built-in sample fight (from the original's `previewLog.json`).

use crate::infrastructure::act::data::CombatDataMessage;
use std::sync::OnceLock;

/// The sample fight the settings screens preview, prepared like real data is: its pets folded into
/// their owners when the "Combine Pets with Owner" setting is on, and always sorted by damage.
pub fn sample_combat_message(merge_pets_into_owners: bool) -> &'static CombatDataMessage {
    static UNMERGED: OnceLock<CombatDataMessage> = OnceLock::new();
    static MERGED: OnceLock<CombatDataMessage> = OnceLock::new();
    let prepared = |merge: bool| {
        let mut message: CombatDataMessage =
            serde_json::from_str(include_str!("../../data/previewLog.json")).expect("data/previewLog.json is valid combat data");
        if merge {
            message.merge_pets_into_owners(None);
        }
        message.sort_by_damage();
        message
    };
    if merge_pets_into_owners { MERGED.get_or_init(|| prepared(true)) } else { UNMERGED.get_or_init(|| prepared(false)) }
}
