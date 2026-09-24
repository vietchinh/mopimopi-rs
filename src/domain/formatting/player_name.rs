//! How a player's name is displayed in the name column.

use crate::domain::combat::{Player, COMBATANT_JOB_CODE, LIMIT_BREAK_JOB_CODE, LOCAL_PLAYER_ROW_NAME};
use crate::domain::settings::Settings;
use crate::domain::translations::Translations;

/// How much of a "First Last" name is shortened (the "Name" settings page).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NameAbbreviation {
    FullName,
    AbbreviatedLastName,
    AbbreviatedFirstName,
    AbbreviatedBothNames,
}

impl NameAbbreviation {
    fn from_option_number(number: i32) -> NameAbbreviation {
        match number {
            1 => NameAbbreviation::FullName,
            2 => NameAbbreviation::AbbreviatedLastName,
            3 => NameAbbreviation::AbbreviatedFirstName,
            _ => NameAbbreviation::AbbreviatedBothNames,
        }
    }
}

/// Name settings, read once from the user's settings.
pub struct NameOptions {
    abbreviation: NameAbbreviation,
    /// Keep the "YOU" label for the local player and "(YOU)" for their pets, instead of
    /// showing the character name.
    keep_you_label: bool,
    hide_names: bool,
    prefix_rank: bool,
}

impl NameOptions {
    pub fn from_settings(settings: &Settings) -> NameOptions {
        NameOptions {
            abbreviation: NameAbbreviation::from_option_number(settings.option_number("cnt") as i32),
            keep_you_label: settings.option_enabled("myName"),
            hide_names: settings.option_enabled("hideName"),
            prefix_rank: settings.option_enabled("rank"),
        }
    }
}

fn first_character(text: &str) -> String {
    text.chars().next().map(String::from).unwrap_or_default()
}

/// "Eos Fair" -> "Eos F." / "E. Fair" / "E. F." depending on the setting; one-word names stay.
fn abbreviate_name(name: &str, abbreviation: NameAbbreviation) -> String {
    let words: Vec<&str> = name.split(' ').collect();
    if words.len() < 2 {
        return name.to_string();
    }
    match abbreviation {
        NameAbbreviation::FullName => name.to_string(),
        NameAbbreviation::AbbreviatedLastName => format!("{} {}.", words[0], first_character(words[1])),
        NameAbbreviation::AbbreviatedFirstName => format!("{}. {}", first_character(words[0]), words[1]),
        NameAbbreviation::AbbreviatedBothNames => format!("{}. {}.", first_character(words[0]), first_character(words[1])),
    }
}

/// Abbreviates the owner inside "Pet (Owner)" names, and plain names as a whole.
fn abbreviate_pet_or_player_name(name: &str, local_player_name: &str, options: &NameOptions) -> String {
    let Some((pet_part, rest)) = name.split_once('(') else {
        return abbreviate_name(name, options.abbreviation);
    };
    let owner = rest.strip_suffix(')').unwrap_or(rest);
    let shown_owner = if !options.keep_you_label && owner == LOCAL_PLAYER_ROW_NAME {
        let character = if local_player_name.is_empty() { LOCAL_PLAYER_ROW_NAME } else { local_player_name };
        abbreviate_name(character, options.abbreviation)
    } else {
        abbreviate_name(owner, options.abbreviation)
    };
    format!("{} ({})", pet_part.trim_end(), shown_owner)
}

/// "Eos(YOU)" for pets, or the translated companion label for owned combatants.
fn label_for_own_pet(player: &Player, translations: &Translations, language_code: &str) -> String {
    if player.job_code == COMBATANT_JOB_CODE {
        format!("{} ({LOCAL_PLAYER_ROW_NAME})", translations.dictionary_title(COMBATANT_JOB_CODE, language_code))
    } else {
        format!("{}({LOCAL_PLAYER_ROW_NAME})", player.name.split('(').next().unwrap_or(""))
    }
}

fn is_owned_by_local_player(player: &Player, local_player_name: &str) -> bool {
    !player.pet_owner_name.is_empty()
        && (player.pet_owner_name == local_player_name || player.pet_owner_name == LOCAL_PLAYER_ROW_NAME)
}

/// The text of the name cell.
pub fn display_name(
    player: &Player,
    local_player_name: &str,
    options: &NameOptions,
    translations: &Translations,
    language_code: &str,
) -> String {
    let is_local_row = player.name == LOCAL_PLAYER_ROW_NAME;
    let name = if !options.hide_names {
        if is_local_row && !options.keep_you_label {
            if local_player_name.is_empty() {
                player.name.clone()
            } else {
                abbreviate_pet_or_player_name(local_player_name, local_player_name, options)
            }
        } else if is_owned_by_local_player(player, local_player_name) && options.keep_you_label {
            label_for_own_pet(player, translations, language_code)
        } else {
            abbreviate_pet_or_player_name(&player.name, local_player_name, options)
        }
    } else if is_local_row {
        player.name.clone()
    } else if is_owned_by_local_player(player, local_player_name) && !local_player_name.is_empty() {
        label_for_own_pet(player, translations, language_code)
    } else if player.job_code == LIMIT_BREAK_JOB_CODE {
        translations.dictionary_title(LIMIT_BREAK_JOB_CODE, language_code)
    } else {
        String::new()
    };
    if options.prefix_rank { format!("{}. {}", player.rank + 1, name) } else { name }
}

#[cfg(test)]
#[path = "../../../tests/unit/domain/formatting/player_name.rs"]
mod tests;
