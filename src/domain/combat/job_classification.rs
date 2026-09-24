//! Decides a record's job code, class (icon), role and pet-ness.
//!
//! ACT only sends a job abbreviation for real players. Pets have no job, so they are
//! recognised by name; anything else with "(Owner)" in its name is an owned combatant.

use super::pet_names::find_pet_owner_job;
use super::player_role::PlayerRole;

/// Job code the overlay uses for pets that belong to a player.
pub const PET_JOB_CODE: &str = "AVA";
/// Job code for the Limit Break pseudo-combatant.
pub const LIMIT_BREAK_JOB_CODE: &str = "LMB";
/// Job code for owned combatants that are not known pets (for example chocobos).
pub const COMBATANT_JOB_CODE: &str = "CBO";

const NO_JOB_TEXT: &str = "0";
const LIMIT_BREAK_JOB_TEXT: &str = "Limit Break";

#[derive(Clone, Debug, PartialEq)]
pub struct JobClassification {
    pub job_code: String,
    /// Which icon and colour to use; base classes map to their advanced job (GLA -> PLD).
    pub class_code: String,
    pub role: PlayerRole,
    pub is_pet: bool,
    /// Name of the owning player, when the record is a pet or owned combatant.
    pub pet_owner_name: String,
}

/// Text inside the first pair of parentheses: "Eos (YOU)" -> "YOU".
pub fn text_in_first_parentheses(name: &str) -> Option<String> {
    let after_open = &name[name.find('(')? + 1..];
    let close_index = after_open.find(')')?;
    Some(after_open[..close_index].to_string())
}

/// Classifies one combatant from its name and the job text ACT sent.
pub fn classify(name: &str, job_text: &str) -> JobClassification {
    let has_job = !job_text.is_empty() && job_text != NO_JOB_TEXT;
    let mut job_code = if has_job { job_text.to_string() } else { NO_JOB_TEXT.to_string() };
    let mut class_code = if has_job { job_text.to_uppercase() } else { String::new() };
    let mut role = PlayerRole::Damage;
    let mut is_pet = false;

    if job_text == LIMIT_BREAK_JOB_TEXT {
        job_code = LIMIT_BREAK_JOB_CODE.to_string();
        class_code = LIMIT_BREAK_JOB_CODE.to_string();
    }
    if has_job {
        apply_base_class_mapping(&job_text.to_uppercase(), &mut class_code, &mut role);
    }
    if let Some(role_of_class) = role_of_class(&class_code) {
        role = role_of_class;
    }

    if class_code.is_empty() {
        let name_without_owner = name.split(" (").next().unwrap_or("");
        if let Some(pet_job) = find_pet_owner_job(name_without_owner) {
            job_code = PET_JOB_CODE.to_string();
            class_code = pet_job.class_code.to_string();
            is_pet = true;
            if let Some(forced_role) = pet_job.role {
                role = forced_role;
            }
        } else if !name.contains('(') {
            job_code = LIMIT_BREAK_JOB_CODE.to_string();
            class_code = LIMIT_BREAK_JOB_CODE.to_string();
        }
    }

    let mut pet_owner_name = String::new();
    let is_ownerless_or_pet = job_code == NO_JOB_TEXT || job_code == PET_JOB_CODE;
    if let Some(owner) = text_in_first_parentheses(name).filter(|_| is_ownerless_or_pet) {
        pet_owner_name = owner;
    }
    if !pet_owner_name.is_empty() && job_code == NO_JOB_TEXT {
        job_code = COMBATANT_JOB_CODE.to_string();
        class_code = COMBATANT_JOB_CODE.to_string();
        role = PlayerRole::OwnedCombatant;
    }

    JobClassification { job_code, class_code, role, is_pet, pet_owner_name }
}

/// Base classes share the icon and colour of the job they become (GLA -> PLD), and crafters
/// and gatherers get their own roles.
fn apply_base_class_mapping(upper_job: &str, class_code: &mut String, role: &mut PlayerRole) {
    match upper_job {
        "GLD" | "GLA" => *class_code = "PLD".into(),
        "MRD" => *class_code = "WAR".into(),
        "PUG" | "PGL" => *class_code = "MNK".into(),
        "LNC" => *class_code = "DRG".into(),
        "ROG" => *class_code = "NIN".into(),
        "ARC" => *class_code = "BRD".into(),
        "THM" => *class_code = "BLM".into(),
        "ACN" => *class_code = "SMN".into(),
        "CNJ" => *class_code = "WHM".into(),
        "CRP" | "BSM" | "ARM" | "GSM" | "LTW" | "WVR" | "ALC" | "CUL" => *role = PlayerRole::Crafter,
        "BTN" | "MIN" | "FSH" => *role = PlayerRole::Gatherer,
        _ => {}
    }
}

fn role_of_class(class_code: &str) -> Option<PlayerRole> {
    match class_code {
        "SCH" | "WHM" | "AST" | "SGE" => Some(PlayerRole::Healer),
        "PLD" | "WAR" | "DRK" | "GNB" => Some(PlayerRole::Tank),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/combat/job_classification.rs"]
mod tests;
