//! The kinds of member an Encounter holds. Who is a player, an NPC, a companion, the Limit Break or none of these is
//! decided once, when the Encounter is built (`Member::sort`); everything after that asks the type.
//!
//! * `Player`          – a character; "YOU" is the local one.
//! * `Npc`             – a trust or duty-support helper: no job, no owner in its name, not the gauge.
//! * `PlayerCompanion` – a pet or chocobo; it always has an owner, named as ACT names it.
//! * `LimitBreak`      – the shared gauge. There is at most one per Encounter.
//! * `Unclassified`    – technical: ACT sent something the model cannot place (a pet missing from the known list, a
//!   companion whose owner is not in the fight, a second gauge). It is shown as its own row, so nothing ACT sent is lost.
//!
//! (DDD: entities of the aggregate; the weakest-parameter and "preconditions in types" rules of the talks.)

use super::{Figures, MemberKind, PetJob};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Player {
    pub figures: Figures,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Npc {
    pub figures: Figures,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LimitBreak {
    pub figures: Figures,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionKind {
    Pet(PetJob),
    Chocobo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerCompanion {
    pub figures: Figures,
    /// The owner's name as it stands in the companion's own name: "Eos (Future Fade)" has owner "Future Fade".
    pub owner: String,
    pub kind: CompanionKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Unclassified {
    pub figures: Figures,
    /// Set when this was a companion whose owner is not in the fight: it still looks like the pet or chocobo it is.
    pub was_companion_of: Option<(String, CompanionKind)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Member {
    Player(Player),
    Npc(Npc),
    Companion(PlayerCompanion),
    LimitBreak(LimitBreak),
    Unclassified(Unclassified),
}

impl Default for Member {
    fn default() -> Self {
        Member::Player(Player::default())
    }
}

impl Deref for Member {
    type Target = Figures;
    fn deref(&self) -> &Figures {
        match self {
            Member::Player(m) => &m.figures,
            Member::Npc(m) => &m.figures,
            Member::Companion(m) => &m.figures,
            Member::LimitBreak(m) => &m.figures,
            Member::Unclassified(m) => &m.figures,
        }
    }
}

impl DerefMut for Member {
    fn deref_mut(&mut self) -> &mut Figures {
        match self {
            Member::Player(m) => &mut m.figures,
            Member::Npc(m) => &mut m.figures,
            Member::Companion(m) => &mut m.figures,
            Member::LimitBreak(m) => &mut m.figures,
            Member::Unclassified(m) => &mut m.figures,
        }
    }
}

impl Member {
    pub fn player(figures: Figures) -> Self {
        Member::Player(Player { figures })
    }

    /// Decides what ACT's row is, from its name and job text, the way the original does. A pet or chocobo is a companion
    /// here whatever its owner; `Encounter::from_figures` demotes the ones whose owner is absent.
    pub fn sort(figures: Figures) -> Self {
        let kind = MemberKind::classify(figures.identity());
        match kind {
            MemberKind::Player => Member::Player(Player { figures }),
            MemberKind::LimitBreak if figures.job == "Limit Break" || figures.name == "Limit Break" => {
                Member::LimitBreak(LimitBreak { figures })
            }
            MemberKind::LimitBreak => Member::Npc(Npc { figures }),
            MemberKind::Pet { owner_name, job } => {
                let owner = owner_name.to_owned();
                Member::Companion(PlayerCompanion { figures, owner, kind: CompanionKind::Pet(job) })
            }
            MemberKind::Chocobo { owner_name } => {
                let owner = owner_name.to_owned();
                Member::Companion(PlayerCompanion { figures, owner, kind: CompanionKind::Chocobo })
            }
            MemberKind::Unknown => Member::Unclassified(Unclassified { figures, was_companion_of: None }),
        }
    }

    /// This companion, placed as unclassified because its owner is not in the fight.
    pub(super) fn demoted(self) -> Self {
        match self {
            Member::Companion(PlayerCompanion { figures, owner, kind }) => {
                Member::Unclassified(Unclassified { figures, was_companion_of: Some((owner, kind)) })
            }
            Member::LimitBreak(LimitBreak { figures }) => Member::Unclassified(Unclassified { figures, was_companion_of: None }),
            other => other,
        }
    }

    /// A trust or duty-support NPC. The original draws these exactly like the gauge (see `MemberKind::classify`);
    /// that is a Display decision.
    pub fn is_npc(&self) -> bool {
        matches!(self, Member::Npc(_))
    }

    /// Player, pet, chocobo or Limit Break, as the original's look decides them (see `MemberKind`). NPCs look like the gauge.
    pub fn kind(&self) -> MemberKind<'_> {
        match self {
            Member::Player(_) => MemberKind::Player,
            Member::Npc(_) | Member::LimitBreak(_) => MemberKind::LimitBreak,
            Member::Companion(c) => companion_kind(&c.owner, c.kind),
            Member::Unclassified(u) => match &u.was_companion_of {
                Some((owner, kind)) => companion_kind(owner, *kind),
                None => MemberKind::Unknown,
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
            MemberKind::LimitBreak => "LMB".into(),
            MemberKind::Chocobo { .. } => "CBO".into(),
            MemberKind::Pet { job, .. } => match job {
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

    /// Whether this member counts as a tank / healer / crafter-or-gatherer for the job-filter
    /// settings ("`DPS_T`", "`HPS_H`", ... in `visible_players`) and the "role" palette mode. Checked
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
            || matches!(self.kind(), MemberKind::Pet { job: PetJob::DarkKnight, .. })
    }

    pub fn is_healer(&self) -> bool {
        matches!(self.job.to_uppercase().as_str(), "SCH" | "WHM" | "AST" | "SGE" | "CNJ")
            || matches!(self.kind(), MemberKind::Pet { job: PetJob::Scholar | PetJob::Astrologian | PetJob::WhiteMage | PetJob::Sage, .. })
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
        } else if matches!(self.kind(), MemberKind::Player | MemberKind::Pet { .. }) {
            // A pet is a plain "DPS" unless it is one of the healing/tanking pets handled above.
            "DPS"
        } else if matches!(self.kind(), MemberKind::Chocobo { .. }) {
            "CBO"
        } else {
            ""
        }
    }



}

fn companion_kind(owner: &str, kind: CompanionKind) -> MemberKind<'_> {
    match kind {
        CompanionKind::Pet(job) => MemberKind::Pet { owner_name: owner, job },
        CompanionKind::Chocobo => MemberKind::Chocobo { owner_name: owner },
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/member.rs"]
mod tests;
