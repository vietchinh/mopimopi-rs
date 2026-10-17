//! One participant of an Encounter as the domain knows it: a plain value, with no serde and no ACT field names.
//! The ACT translator (`infrastructure::act`) builds these; everything that decides what a participant is
//! (`kind`, `class_code`, roles) or how participants combine (`absorb_pet`) lives here.
//!
//! (DDD: the wire type stays at the edge, an anticorruption layer; the domain owns its own model.)

use super::encounter::{Amount, Rate};
use super::{percent_of, CombatantIdentity, CombatantKind, Healing, PetJob};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Member {
    /// Position in ACT's message (0 = first). Not part of the JSON. Rows that tie on the value a table
    /// is sorted by keep this order, which is what the original's stable sort does.
    pub arrival_index: usize,
    pub name: String,
    pub job_text: String,
    /// The combatant's own duration, e.g. "00:34".
    pub duration_text: String,
    pub duration_seconds: f64,

    pub damage: f64,
    pub damage_per_second: f64,
    pub hits: f64,
    pub misses: f64,
    pub swings: f64,
    pub critical_hits: f64,
    pub direct_hits: f64,
    pub critical_direct_hits: f64,
    pub damage_taken: f64,

    pub heals: f64,
    /// Everything: HP restored + overheal + shields. See `healing()`.
    pub healed: f64,
    pub heal_per_second: f64,
    pub cures: f64,
    pub critical_heals: f64,
    pub heals_taken: f64,
    pub damage_shield: f64,
    pub over_heal: f64,
    pub absorb_heal: f64,

    pub damage_per_second_last_10_seconds: f64,
    pub damage_per_second_last_30_seconds: f64,
    pub damage_per_second_last_60_seconds: f64,
    pub damage_per_second_last_180_seconds: f64,

    /// The action of the strongest hit ("Shieldsplitter"); the amount is
    /// `strongest_hit_amount`. Empty when there was none. (ACT sends "Shieldsplitter-1819".)
    pub strongest_hit_name: String,
    pub strongest_hit_amount: f64,
    /// The action of the strongest heal ("Divine Veil (*)"); the amount is
    /// `strongest_heal_amount`. Empty when there was none.
    pub strongest_heal_name: String,
    pub strongest_heal_amount: f64,


    pub avoided_hits: f64,
    pub mana_restored: f64,
    pub parry_percent: f64,
    pub block_percent: f64,
    pub deaths: f64,

    /// After pets are merged: the part of `damage` that came from pets. 0 otherwise.
    /// For drawing the pet's segment of the DPS bar.
    pub pet_damage: f64,
    /// After pets are merged: the part of the effective healing that came from pets.
    /// For drawing the pet's segment of the HPS bar.
    pub pet_effective_healed: f64,
}

impl Member {
    /// A trust or duty-support NPC: a row with no job and no owner in its name that is not the Limit Break gauge itself.
    /// The original draws these exactly like the gauge (see `CombatantKind::classify`); that is a Display decision.
    pub fn is_npc(&self) -> bool {
        self.kind() == CombatantKind::LimitBreak && self.job_text != "Limit Break" && self.name != "Limit Break"
    }

    /// Damage per second over the member's *own* active time (ACT's `dps`, the overlay's "P.DPS"). A time of 0 counts as
    /// 1 second, as in the original.
    pub fn personal_damage_per_second(&self) -> Rate {
        let seconds = if self.duration_seconds == 0.0 { 1.0 } else { self.duration_seconds };
        Rate::per_second(Amount::new(self.damage), seconds)
    }

    /// The name and job the domain rules work with.
    pub fn identity(&self) -> CombatantIdentity<'_> {
        CombatantIdentity { name: &self.name, job: &self.job_text }
    }

    /// Player, pet, chocobo or Limit Break (see `CombatantKind`).
    pub fn kind(&self) -> CombatantKind<'_> {
        CombatantKind::classify(self.identity())
    }

    /// The original's `Class`: what a bar's colour is looked up by. (The job *icon* is looked up by the
    /// raw job instead, see `job_icon_view`.) The two differ for the base classes, which take their
    /// advanced job's colour (`GLA` -> `PLD`), for pets, which take their owner's job, and for the
    /// Limit Break and chocobo rows, whose job text is blank or `0`.
    pub fn class_code(&self) -> String {
        use crate::domain::combat::PetJob::*;
        match self.kind() {
            CombatantKind::LimitBreak => "LMB".into(),
            CombatantKind::Chocobo { .. } => "CBO".into(),
            CombatantKind::Pet { job, .. } => match job {
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
            _ => match self.job_text.to_uppercase().as_str() {
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

    /// Whether this combatant counts as a tank / healer / crafter-or-gatherer for the job-filter
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
        matches!(self.job_text.to_uppercase().as_str(), "PLD" | "WAR" | "DRK" | "GNB" | "GLA" | "MRD")
            || matches!(self.kind(), CombatantKind::Pet { job: PetJob::DarkKnight, .. })
    }

    pub fn is_healer(&self) -> bool {
        matches!(self.job_text.to_uppercase().as_str(), "SCH" | "WHM" | "AST" | "SGE" | "CNJ")
            || matches!(self.kind(), CombatantKind::Pet { job: PetJob::Scholar | PetJob::Astrologian | PetJob::WhiteMage | PetJob::Sage, .. })
    }

    pub fn is_crafter(&self) -> bool {
        matches!(self.job_text.to_uppercase().as_str(), "CRP" | "BSM" | "ARM" | "GSM" | "LTW" | "WVR" | "ALC" | "CUL")
    }

    pub fn is_gatherer(&self) -> bool {
        matches!(self.job_text.to_uppercase().as_str(), "BTN" | "MIN" | "FSH")
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
        } else if matches!(self.kind(), CombatantKind::Player | CombatantKind::Pet { .. }) {
            // A pet is a plain "DPS" unless it is one of the healing/tanking pets handled above.
            "DPS"
        } else if matches!(self.kind(), CombatantKind::Chocobo { .. }) {
            "CBO"
        } else {
            ""
        }
    }

    /// Healing split into what was restored, overhealed and shielded.
    pub fn healing(&self) -> Healing {
        Healing { healed: self.healed, over_heal: self.over_heal, damage_shield: self.damage_shield }
    }

    /// Like ACT's `critheal%`, but exact (ACT rounds; `NumberFormat::percent` does the same).
    pub fn critical_heal_percent(&self) -> Option<f64> {
        percent_of(self.critical_heals, self.heals)
    }

    pub fn absorb_pet(&mut self, pet: &Member) {
        self.damage += pet.damage;
        self.damage_per_second += pet.damage_per_second;
        self.hits += pet.hits;
        self.misses += pet.misses;
        self.swings += pet.swings;
        self.critical_hits += pet.critical_hits;
        self.direct_hits += pet.direct_hits;
        self.critical_direct_hits += pet.critical_direct_hits;
        self.damage_taken += pet.damage_taken;

        self.heals += pet.heals;
        self.healed += pet.healed;
        self.heal_per_second += pet.heal_per_second;
        self.cures += pet.cures;
        self.critical_heals += pet.critical_heals;
        self.heals_taken += pet.heals_taken;
        self.damage_shield += pet.damage_shield;
        self.over_heal += pet.over_heal;
        self.absorb_heal += pet.absorb_heal;

        self.damage_per_second_last_10_seconds += pet.damage_per_second_last_10_seconds;
        self.damage_per_second_last_30_seconds += pet.damage_per_second_last_30_seconds;
        self.damage_per_second_last_60_seconds += pet.damage_per_second_last_60_seconds;
        self.damage_per_second_last_180_seconds += pet.damage_per_second_last_180_seconds;

        self.pet_damage += pet.damage;
        self.pet_effective_healed += pet.healing().effective();

        if pet.strongest_hit_amount > self.strongest_hit_amount {
            self.strongest_hit_amount = pet.strongest_hit_amount;
            self.strongest_hit_name.clone_from(&pet.strongest_hit_name);
        }
        if pet.strongest_heal_amount > self.strongest_heal_amount {
            self.strongest_heal_amount = pet.strongest_heal_amount;
            self.strongest_heal_name.clone_from(&pet.strongest_heal_name);
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/member.rs"]
mod tests;
