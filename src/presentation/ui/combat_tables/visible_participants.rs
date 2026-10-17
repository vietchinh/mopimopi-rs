//! Which participants a table shows, in the order it should show them.

use crate::domain::combat::ParticipantKind;
use crate::presentation::ui::areas::JobFilter;
use crate::domain::combat::Participant;

/// The job filter of the table ("tanks in the damage table", "healers in the healing table", ...).
fn passes_job_filter(filter: &JobFilter, participant: &Participant) -> bool {
    let is_chocobo = matches!(participant.kind(), ParticipantKind::Chocobo { .. });
    (filter.tanks && participant.is_tank())
        || (filter.healers && participant.is_healer())
        // "DPS" in the original is the default role: everyone who is not a tank, healer, crafter, gatherer or chocobo
        || (filter.damage_dealers && !participant.is_tank() && !participant.is_healer() && !participant.is_crafter_or_gatherer() && !is_chocobo)
        || (filter.chocobos && is_chocobo)
        || (filter.crafters_and_gatherers && participant.is_crafter_or_gatherer())
}

/// A row of a table together with its rank in that table (0 = first), which is what `rank` in the
/// name cell prints. The rank counts every participant in the table's own order, before the job
/// filter hides any: hiding tanks does not renumber the rest (the original's `rank` is assigned
/// while sorting, before it filters).
pub(super) struct RankedParticipant<'a> {
    pub rank: usize,
    pub participant: &'a Participant,
}

/// Participants this table shows, in the order it should draw them, after the job filter. The damage
/// table takes ACT's list as `Encounter::sort_by_damage` left it; the healing table sorts
/// it by healed, best first, rows that tie in arrival order (as the original's stable sort does).
pub(super) fn visible_participants<'a>(filter: &JobFilter, participants: &'a [Participant], is_healing: bool) -> Vec<RankedParticipant<'a>> {
    let mut ordered: Vec<&Participant> = participants.iter().collect();
    if is_healing {
        ordered.sort_by(|a, b| b.healed.partial_cmp(&a.healed).unwrap_or(std::cmp::Ordering::Equal).then(a.arrival_index.cmp(&b.arrival_index)));
    }
    ordered
        .into_iter()
        .enumerate()
        .filter(|(_, participant)| passes_job_filter(filter, participant))
        .map(|(rank, participant)| RankedParticipant { rank, participant })
        .collect()
}
