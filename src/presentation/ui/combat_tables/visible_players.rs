//! Which combatants a table shows, in the order it should show them.

use crate::domain::combat::CombatantKind;
use crate::domain::settings::Settings;
use crate::infrastructure::act::data::CombatantRecord;

/// The job-filter options ("`DPS_T`" = tanks in the DPS table, "`HPS_H`" = healers in the HPS table ...).
fn passes_job_filter(settings: &Settings, is_healing: bool, combatant: &CombatantRecord) -> bool {
    let filter_enabled = |suffix: &str| settings.option_enabled(&format!("{}_{suffix}", super::table_label(is_healing)));
    (filter_enabled("T") && combatant.is_tank())
        || (filter_enabled("H") && combatant.is_healer())
        || (filter_enabled("D") && !combatant.is_tank() && !combatant.is_healer() && !combatant.is_crafter_or_gatherer())
        || (filter_enabled("C") && matches!(combatant.kind(), CombatantKind::Chocobo { .. }))
        || (filter_enabled("M") && combatant.is_crafter_or_gatherer())
}

/// Combatants this table shows, already in the order the table should draw them (damage: as ACT
/// sent them; healing: sorted here, best healer first, ties in ACT's arrival order), after the
/// job filter.
pub(super) fn visible_players<'a>(settings: &Settings, combatants: &'a [CombatantRecord], is_healing: bool) -> Vec<&'a CombatantRecord> {
    let mut ordered: Vec<&CombatantRecord> = combatants.iter().collect();
    if is_healing {
        ordered.sort_by(|a, b| b.heal_per_second.partial_cmp(&a.heal_per_second).unwrap_or(std::cmp::Ordering::Equal));
    }
    ordered.into_iter().filter(|combatant| passes_job_filter(settings, is_healing, combatant)).collect()
}
