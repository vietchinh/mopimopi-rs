//! The kinds of participant an Encounter holds. Who is a player, an NPC, a companion, the Limit Break or none of these is
//! decided once, when the Encounter is built (`Participant::sort`); everything after that asks the type.
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

use super::{ParticipantKind, PetJob, Stats, LOCAL_PLAYER};
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

impl Participant {
    pub fn player(stats: Stats) -> Self {
        Participant::Player(Player { stats })
    }

    /// Decides what ACT's row is, from its name and job text, the way the original does. A pet or chocobo is a companion
    /// here whatever its owner; `Encounter::from_stats` demotes the ones whose owner is absent.
    pub fn sort(stats: Stats) -> Self {
        if stats.name == LOCAL_PLAYER {
            return Participant::You(You { stats });
        }
        let kind = ParticipantKind::classify(stats.identity());
        match kind {
            ParticipantKind::Player => Participant::Player(Player { stats }),
            ParticipantKind::LimitBreak if stats.job == "Limit Break" || is_limit_break_name(&stats.name) => {
                Participant::LimitBreak(LimitBreak { stats })
            }
            ParticipantKind::LimitBreak => Participant::Npc(Npc { stats }),
            ParticipantKind::Pet { owner_name, job } => {
                let owner = owner_name.to_owned();
                Participant::Companion(PlayerCompanion { stats, owner, kind: CompanionKind::Pet(job) })
            }
            ParticipantKind::Chocobo { owner_name } => {
                let owner = owner_name.to_owned();
                Participant::Companion(PlayerCompanion { stats, owner, kind: CompanionKind::Chocobo })
            }
            ParticipantKind::Unknown => Participant::Unclassified(Unclassified { stats, was_companion_of: None }),
        }
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

    /// A trust or duty-support NPC. The original draws these exactly like the gauge (see `ParticipantKind::classify`);
    /// that is a Display decision.
    pub fn is_npc(&self) -> bool {
        matches!(self, Participant::Npc(_))
    }

    /// Player, pet, chocobo or Limit Break, as the original's look decides them (see `ParticipantKind`). NPCs look like the gauge.
    pub fn kind(&self) -> ParticipantKind<'_> {
        match self {
            Participant::You(_) | Participant::Player(_) => ParticipantKind::Player,
            Participant::Npc(_) | Participant::LimitBreak(_) => ParticipantKind::LimitBreak,
            Participant::Companion(c) => companion_kind(&c.owner, c.kind),
            Participant::Unclassified(u) => match &u.was_companion_of {
                Some((owner, kind)) => companion_kind(owner, *kind),
                None => ParticipantKind::Unknown,
            },
        }
    }


    /// The original's `Class`: what a bar's colour is looked up by. (The job *icon* is looked up by the
    /// raw job instead, see `job_icon_view`.) The two differ for the base classes, which take their
    /// advanced job's colour (`GLA` -> `PLD`), for pets, which take their owner's job, and for the
    /// Limit Break and chocobo rows, whose job text is blank or `0`.
    pub fn class_code(&self) -> String {
        use crate::domain::combat::PetJob::*;
        match self.kind() {
            ParticipantKind::LimitBreak => "LMB".into(),
            ParticipantKind::Chocobo { .. } => "CBO".into(),
            ParticipantKind::Pet { job, .. } => match job {
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
            _ => match self.job.to_uppercase().as_str() {
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
            || matches!(self.kind(), ParticipantKind::Pet { job: PetJob::DarkKnight, .. })
    }

    pub fn is_healer(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "SCH" | "WHM" | "AST" | "SGE" | "CNJ")
            || matches!(self.kind(), ParticipantKind::Pet { job: PetJob::Scholar | PetJob::Astrologian | PetJob::WhiteMage | PetJob::Sage, .. })
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
        } else if matches!(self.kind(), ParticipantKind::Player | ParticipantKind::Pet { .. }) {
            // A pet is a plain "DPS" unless it is one of the healing/tanking pets handled above.
            "DPS"
        } else if matches!(self.kind(), ParticipantKind::Chocobo { .. }) {
            "CBO"
        } else {
            ""
        }
    }



}

/// The gauge's name in every language the original's dictionary (`dic.js`, `LMB.tt`) translates it into. Only the English
/// name has been seen in ACT data; the others are a guess that costs nothing (it only decides whether the port-only NPC look
/// applies, never what the original would draw: both kinds look like the gauge by default). A capture would confirm it.
const LIMIT_BREAK_NAMES: [&str; 6] = ["Limit Break", "리미트 브레이크", "リミットブレイク", "Transcendance", "Limitrausch", "极限技"];

fn is_limit_break_name(name: &str) -> bool {
    LIMIT_BREAK_NAMES.contains(&name)
}

fn companion_kind(owner: &str, kind: CompanionKind) -> ParticipantKind<'_> {
    match kind {
        CompanionKind::Pet(job) => ParticipantKind::Pet { owner_name: owner, job },
        CompanionKind::Chocobo => ParticipantKind::Chocobo { owner_name: owner },
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/participant.rs"]
mod tests;
