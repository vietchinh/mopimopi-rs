//! Sorts one of ACT's rows into the domain's kinds of Participant. This is where ACT's conventions live (an
//! anticorruption layer, in Evans's sense): the domain never sees "job 0 means chocobo" or "a blank job means pet".
//!
//! * the local player is listed under the name "YOU";
//! * a pet or chocobo carries its owner in its name, "Eos (Future Fade)" (character names cannot contain parentheses);
//! * a chocobo has the job text "0"; a pet has a blank job;
//! * the Limit Break gauge has the job text "Limit Break", but on some captures a blank job and the name instead;
//! * the original (`core.js`) draws every other blank-job row without an owner as the gauge, which is how trust and
//!   duty-support NPCs look. They are `Npc` here, and the domain decides how an Npc is drawn.

use crate::domain::combat::{job_of_pet, Participant, PetJob, Stats};

/// ACT's name for the local player's own row.
const ACT_LOCAL_PLAYER: &str = "YOU";

/// The gauge's name in every language the original's dictionary (`dic.js`, `LMB.tt`) translates it into. Only the English
/// name has been seen in ACT data; the others are a guess that costs nothing (it only decides whether the port-only NPC look
/// applies, never what the original would draw: both kinds look like the gauge by default). A capture would confirm it.
const LIMIT_BREAK_NAMES: [&str; 6] = ["Limit Break", "리미트 브레이크", "リミットブレイク", "Transcendance", "Limitrausch", "极限技"];

pub(super) fn sort(stats: Stats) -> Participant {
    if stats.name == ACT_LOCAL_PLAYER {
        return Participant::you(stats);
    }
    // Most rows are players with a job text: decide those first, without looking at the name's shape.
    match stats.job.as_str() {
        "Limit Break" => return Participant::limit_break(stats),
        "" | "0" => {}
        _ => return Participant::player(stats),
    }
    let is_chocobo = stats.job.as_str() == "0";
    let (base_name, owner) = split_owner(&stats.name);
    let owner = owner.map(str::to_owned);
    let pet_job = (!is_chocobo).then(|| job_of_pet(base_name));
    match (is_chocobo, owner, pet_job) {
        (true, Some(owner), _) => Participant::chocobo(stats, owner),
        (true, None, _) => Participant::unclassified(stats),
        (false, _, _) if LIMIT_BREAK_NAMES.contains(&stats.name.as_str()) => Participant::limit_break(stats),
        (false, Some(owner), Some(job)) => Participant::pet(stats, owner, job.unwrap_or(PetJob::Beastmaster)),
        // A known pet's name with no owner in it: not something ACT sends, kept as its own row.
        (false, None, Some(Some(_))) => Participant::unclassified(stats),
        _ => Participant::npc(stats),
    }
}

/// FFXIV character names cannot contain parentheses, so the last " (" starts the owner.
fn split_owner(name: &str) -> (&str, Option<&str>) {
    match name.strip_suffix(')').and_then(|rest| rest.rsplit_once(" (")) {
        Some((base_name, owner_name)) if !owner_name.is_empty() => (base_name, Some(owner_name)),
        _ => (name, None),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/infrastructure/act_data/sorting.rs"]
mod tests;
