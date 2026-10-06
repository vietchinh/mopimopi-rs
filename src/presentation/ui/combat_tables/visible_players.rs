//! Which combatants a table shows, in the order it should show them.

use crate::domain::combat::CombatantKind;
use crate::presentation::ui::areas::JobFilter;
use crate::infrastructure::act::data::CombatantRecord;

/// The job filter of the table ("tanks in the damage table", "healers in the healing table", ...).
fn passes_job_filter(filter: &JobFilter, combatant: &CombatantRecord) -> bool {
    let is_chocobo = matches!(combatant.kind(), CombatantKind::Chocobo { .. });
    (filter.tanks && combatant.is_tank())
        || (filter.healers && combatant.is_healer())
        // "DPS" in the original is the default role: everyone who is not a tank, healer, crafter, gatherer or chocobo
        || (filter.damage_dealers && !combatant.is_tank() && !combatant.is_healer() && !combatant.is_crafter_or_gatherer() && !is_chocobo)
        || (filter.chocobos && is_chocobo)
        || (filter.crafters_and_gatherers && combatant.is_crafter_or_gatherer())
}

/// A row of a table together with its rank in that table (0 = first), which is what `rank` in the
/// name cell prints. The rank counts every combatant in the table's own order, before the job
/// filter hides any: hiding tanks does not renumber the rest (the original's `rank` is assigned
/// while sorting, before it filters).
pub(super) struct RankedPlayer<'a> {
    pub rank: usize,
    pub combatant: &'a CombatantRecord,
}

/// Combatants this table shows, in the order it should draw them, after the job filter. The damage
/// table takes ACT's list as `CombatDataMessage::sort_by_damage` left it; the healing table sorts
/// it by healed, best first, rows that tie in arrival order (as the original's stable sort does).
pub(super) fn visible_players<'a>(filter: &JobFilter, combatants: &'a [CombatantRecord], is_healing: bool) -> Vec<RankedPlayer<'a>> {
    let mut ordered: Vec<&CombatantRecord> = combatants.iter().collect();
    if is_healing {
        ordered.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(std::cmp::Ordering::Equal).then(a.arrival_index.cmp(&b.arrival_index)));
    }
    ordered
        .into_iter()
        .enumerate()
        .filter(|(_, combatant)| passes_job_filter(filter, combatant))
        .map(|(rank, combatant)| RankedPlayer { rank, combatant })
        .collect()
}
