//! The kinds of participant an Encounter holds. Who is a player, an NPC, a companion, the Limit Break or none of these is
//! decided once, by the ACT edge, when the Encounter is built; everything after that asks the type.
//!
//! * `You`             – the local player, whom ACT always lists as "YOU". At most one per Encounter.
//! * `Player`          – any other player character.
//! * `Npc`             – a trust or duty-support helper: no job, no owner in its name, not the gauge.
//! * `PlayerCompanion` – a pet or chocobo; it always has an owner, named as ACT names it.
//! * `LimitBreak`      – the shared gauge. There is at most one per Encounter.
//! * `Unclassified`    – technical: ACT sent something the model cannot place (a pet missing from the known list, a
//!   companion whose owner is not in the fight, a second gauge). It is shown as its own row, so nothing ACT sent is lost.
//!
//! (DDD: entities of the aggregate; the weakest-parameter and "preconditions in types" rules of the talks.)

use super::{PetJob, Stats, LOCAL_PLAYER};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Player {
    pub stats: Stats,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct You {
    pub stats: Stats,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Npc {
    pub stats: Stats,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LimitBreak {
    pub stats: Stats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionKind {
    Pet(PetJob),
    Chocobo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerCompanion {
    pub stats: Stats,
    /// The owner's name as it stands in the companion's own name: "Eos (Future Fade)" has owner "Future Fade".
    pub owner: String,
    pub kind: CompanionKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Unclassified {
    pub stats: Stats,
    /// Set when this was a companion whose owner is not in the fight: it still looks like the pet or chocobo it is.
    pub was_companion_of: Option<(String, CompanionKind)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Participant {
    You(You),
    Player(Player),
    Npc(Npc),
    Companion(PlayerCompanion),
    LimitBreak(LimitBreak),
    Unclassified(Unclassified),
}

impl Default for Participant {
    fn default() -> Self {
        Participant::Player(Player::default())
    }
}

impl Deref for Participant {
    type Target = Stats;
    fn deref(&self) -> &Stats {
        match self {
            Participant::You(m) => &m.stats,
            Participant::Player(m) => &m.stats,
            Participant::Npc(m) => &m.stats,
            Participant::Companion(m) => &m.stats,
            Participant::LimitBreak(m) => &m.stats,
            Participant::Unclassified(m) => &m.stats,
        }
    }
}

impl DerefMut for Participant {
    fn deref_mut(&mut self) -> &mut Stats {
        match self {
            Participant::You(m) => &mut m.stats,
            Participant::Player(m) => &mut m.stats,
            Participant::Npc(m) => &mut m.stats,
            Participant::Companion(m) => &mut m.stats,
            Participant::LimitBreak(m) => &mut m.stats,
            Participant::Unclassified(m) => &mut m.stats,
        }
    }
}

/// Whose a participant is, from the local player's point of view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    You,
    /// A pet or chocobo of yours.
    YourCompanion,
    Others,
}

impl Participant {
    // The ways to make a Participant. Deciding which one a row is belongs to whoever knows the sender's conventions
    // (the ACT edge, an anticorruption layer); the domain only receives the answer.

    pub fn you(stats: Stats) -> Self {
        Participant::You(You { stats })
    }

    pub fn player(stats: Stats) -> Self {
        Participant::Player(Player { stats })
    }

    pub fn npc(stats: Stats) -> Self {
        Participant::Npc(Npc { stats })
    }

    pub fn limit_break(stats: Stats) -> Self {
        Participant::LimitBreak(LimitBreak { stats })
    }

    /// A pet of `owner`, named as the owner is named in the fight.
    pub fn pet(stats: Stats, owner: impl Into<String>, job: PetJob) -> Self {
        Participant::Companion(PlayerCompanion { stats, owner: owner.into(), kind: CompanionKind::Pet(job) })
    }

    pub fn chocobo(stats: Stats, owner: impl Into<String>) -> Self {
        Participant::Companion(PlayerCompanion { stats, owner: owner.into(), kind: CompanionKind::Chocobo })
    }

    /// A row the sender gave no place to (see `Unclassified`).
    pub fn unclassified(stats: Stats) -> Self {
        Participant::Unclassified(Unclassified { stats, was_companion_of: None })
    }

    /// This companion, placed as unclassified because its owner is not in the fight.
    pub(super) fn demoted(self) -> Self {
        match self {
            Participant::Companion(PlayerCompanion { stats, owner, kind }) => {
                Participant::Unclassified(Unclassified { stats, was_companion_of: Some((owner, kind)) })
            }
            Participant::LimitBreak(LimitBreak { stats }) => Participant::Unclassified(Unclassified { stats, was_companion_of: None }),
            other => other,
        }
    }

    /// A trust or duty-support NPC. The original draws these exactly like the gauge (see `infrastructure::act::data::sorting`);
    /// that is a Display decision.
    pub fn is_npc(&self) -> bool {
        matches!(self, Participant::Npc(_))
    }

    pub fn ownership(&self) -> Ownership {
        match self.companion_of() {
            _ if self.is_you() => Ownership::You,
            Some((owner, _)) if owner == LOCAL_PLAYER => Ownership::YourCompanion,
            _ => Ownership::Others,
        }
    }

    pub fn is_your_pet(&self) -> bool {
        self.ownership() == Ownership::YourCompanion && self.is_pet()
    }

    /// You, or a pet of yours: the rows the original draws in "my" text colour.
    pub fn is_yours_to_highlight(&self) -> bool {
        self.is_you() || self.is_your_pet()
    }

    pub fn is_you(&self) -> bool {
        matches!(self, Participant::You(_))
    }

    /// The gauge, which the original also draws for trust and duty-support NPCs.
    pub fn looks_like_limit_break(&self) -> bool {
        matches!(self, Participant::Npc(_) | Participant::LimitBreak(_))
    }

    /// What this participant is a companion of, also when its owner is absent and it is shown as its own row.
    pub fn companion_of(&self) -> Option<(&str, CompanionKind)> {
        match self {
            Participant::Companion(c) => Some((&c.owner, c.kind)),
            Participant::Unclassified(Unclassified { was_companion_of: Some((owner, kind)), .. }) => Some((owner, *kind)),
            _ => None,
        }
    }

    /// The owner's name when this is a pet (a chocobo has an owner too, but is never folded into it).
    pub fn pet_owner(&self) -> Option<&str> {
        match self.companion_of()? {
            (owner, CompanionKind::Pet(_)) => Some(owner),
            _ => None,
        }
    }

    pub fn is_pet(&self) -> bool {
        self.pet_owner().is_some()
    }

    pub fn is_chocobo(&self) -> bool {
        matches!(self.companion_of(), Some((_, CompanionKind::Chocobo)))
    }

    fn pet_job(&self) -> Option<PetJob> {
        match self.companion_of()? {
            (_, CompanionKind::Pet(job)) => Some(job),
            _ => None,
        }
    }

    /// The original's `Class`: what a bar's colour is looked up by. (The job *icon* is looked up by the
    /// raw job instead, see `job_icon_view`.) The two differ for the base classes, which take their
    /// advanced job's colour (`GLA` -> `PLD`), for pets, which take their owner's job, and for the
    /// Limit Break and chocobo rows, whose job text is blank or `0`.
    pub fn class_code(&self) -> String {
        use crate::domain::combat::PetJob::*;
        match (self.looks_like_limit_break(), self.companion_of()) {
            (true, _) => "LMB".into(),
            (_, Some((_, CompanionKind::Chocobo))) => "CBO".into(),
            (_, Some((_, CompanionKind::Pet(job)))) => match job {
                Summoner => "SMN",
                Scholar => "SCH",
                Machinist => "MCH",
                DarkKnight => "DRK",
                Ninja => "NIN",
                Astrologian => "AST",
                WhiteMage => "WHM",
                Sage => "SGE",
                Beastmaster => "AVA",
            }
            .into(),
            (false, None) => match self.job.to_uppercase().as_str() {
                "GLD" | "GLA" => "PLD".into(),
                "MRD" => "WAR".into(),
                "PUG" | "PGL" => "MNK".into(),
                "LNC" => "DRG".into(),
                "ROG" => "NIN".into(),
                "ARC" => "BRD".into(),
                "THM" => "BLM".into(),
                "ACN" => "SMN".into(),
                "CNJ" => "WHM".into(),
                other => other.to_string(),
            },
        }
    }

    /// Whether this participant counts as a tank / healer / crafter-or-gatherer for the job-filter
    /// settings ("`DPS_T`", "`HPS_H`", ... in `visible_participants`) and the "role" palette mode. Checked
    /// directly against `job_text` (base classes included as their own entries, not remapped to
    /// their advanced job), the same way the icon is looked up (`job_text.to_uppercase()`) --
    /// except for a handful of specific pets the original also hardcodes a role for (`core.js`:
    /// `schPetsList`/`astPetsList`/`whmPetsList`/`sgePetsList` -> `role = "Healer"`,
    /// `drkPetsList` -> `role = "Tanker"`). Without this, a pet like Scholar's fairy has blank
    /// job text, matches neither list, and silently falls out of both the "Healer" filter
    /// checkbox and the "Healer" role colour -- the original explicitly avoids that for these
    /// specific pets; every other pet (Summoner's, Machinist's, ...) is *not* hardcoded this way
    /// in the original either, and stays plain DPS here too.
    pub fn is_tank(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "PLD" | "WAR" | "DRK" | "GNB" | "GLA" | "MRD")
            || self.pet_job() == Some(PetJob::DarkKnight)
    }

    pub fn is_healer(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "SCH" | "WHM" | "AST" | "SGE" | "CNJ")
            || matches!(self.pet_job(), Some(PetJob::Scholar | PetJob::Astrologian | PetJob::WhiteMage | PetJob::Sage))
    }

    pub fn is_crafter(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "CRP" | "BSM" | "ARM" | "GSM" | "LTW" | "WVR" | "ALC" | "CUL")
    }

    pub fn is_gatherer(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "BTN" | "MIN" | "FSH")
    }

    pub fn is_crafter_or_gatherer(&self) -> bool {
        self.is_crafter() || self.is_gatherer()
    }

    /// The colour key for the "role" palette mode (settings: `Color.Tanker`, `.Healer`,
    /// `.Crafter`, `.Gathering`). Empty for most pets, chocobos and Limit Break, which fall back
    /// to their own job-code colour instead (see `presentation::ui::shared::palette`) -- except
    /// the specific healing/tanking pets `is_tank`/`is_healer` already account for, which get the
    /// same "Tanker"/"Healer" role colour their owner's job would, matching the original.
    pub fn role_palette_key(&self) -> &'static str {
        if self.is_tank() {
            "Tanker"
        } else if self.is_healer() {
            "Healer"
        } else if self.is_crafter() {
            "Crafter"
        } else if self.is_gatherer() {
            "Gathering"
        } else if matches!(self, Participant::You(_) | Participant::Player(_)) || self.is_pet() {
            // A pet is a plain "DPS" unless it is one of the healing/tanking pets handled above.
            "DPS"
        } else if self.is_chocobo() {
            "CBO"
        } else {
            ""
        }
    }



}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/participant.rs"]
mod tests;
