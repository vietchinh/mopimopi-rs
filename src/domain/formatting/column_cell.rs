//! Content of every table column. The column names are ACT's / the original's field names.

use crate::domain::formatting::strongest_action_text::strongest_action_fragments;
use super::number_format::NumberFormat;
use super::player_name::{display_name, NameOptions};
use super::text_fragment::{join_plain_text, TextFragment};
use crate::domain::settings::Settings;
use crate::domain::translations::Translations;
use crate::infrastructure::act::data::{CombatantRecord, EncounterRecord};

/// Everything needed to format cells, gathered once per table.
pub struct CellContext<'a> {
    pub settings: &'a Settings,
    pub translations: &'a Translations,
    pub local_player_name: &'a str,
    number_format: NumberFormat,
    name_options: NameOptions,
    language_code: String,
}

impl<'a> CellContext<'a> {
    pub fn new(settings: &'a Settings, translations: &'a Translations, local_player_name: &'a str) -> CellContext<'a> {
        CellContext {
            settings,
            translations,
            local_player_name,
            number_format: NumberFormat::from_settings(settings),
            name_options: NameOptions::from_settings(settings),
            language_code: settings.language_code(),
        }
    }
}

/// Fragments of one cell. The job icon column (`Class`) is drawn by the caller.
pub fn cell_fragments(column: &str, combatant_record: &CombatantRecord, encounter_record: &EncounterRecord, context: &CellContext) -> Vec<TextFragment> {
    let format = &context.number_format;
    match column {
        "name" => vec![TextFragment::Plain(display_name(
            combatant_record,
            context.local_player_name,
            &context.name_options,
            context.translations,
            &context.language_code,
        ))],
        "duration" => vec![TextFragment::Plain(combatant_record.duration_text.clone())],
        "EncounterDuration" => vec![TextFragment::Plain(encounter_record.duration_text.clone())],

        "dps" => vec![TextFragment::Plain(combatant_record.damage.to_string())],
        "encdps" => vec![TextFragment::Plain(combatant_record.damage_per_second.to_string())],
        "enchps" => vec![TextFragment::Plain(combatant_record.heal_per_second.to_string())],
        "mergedLast10DPS" => vec![TextFragment::Plain(combatant_record.damage_per_second_last_10_seconds.to_string())],
        "mergedLast30DPS" => vec![TextFragment::Plain(combatant_record.damage_per_second_last_30_seconds.to_string())],
        "mergedLast60DPS" => vec![TextFragment::Plain(combatant_record.damage_per_second_last_60_seconds.to_string())],
        "mergedLast180DPS" => vec![TextFragment::Plain(combatant_record.damage_per_second_last_180_seconds.to_string())],

        "mergedDamage" => vec![TextFragment::Plain(combatant_record.damage.to_string())],
        "mergedSwings" => vec![TextFragment::Plain(combatant_record.swings.to_string())],
        "mergedHits" => vec![TextFragment::Plain(combatant_record.hits.to_string())],
        "mergedDirectHitCount" => vec![TextFragment::Plain(combatant_record.direct_hits.to_string())],
        "mergedCrithits" => vec![TextFragment::Plain(combatant_record.critical_hits.to_string())],
        "mergedCritDirectHitCount" => vec![TextFragment::Plain(combatant_record.critical_direct_hits.to_string())],
        "mergedMisses" => vec![TextFragment::Plain(combatant_record.misses.to_string())],
        "hitfailed" => vec![TextFragment::Plain(combatant_record.avoided_hits.to_string())],
        "mergedDamagetaken" => vec![TextFragment::Plain(combatant_record.damage_taken.to_string())],
        "mergedHealstaken" => vec![TextFragment::Plain(combatant_record.heals_taken.to_string())],
        "mergedHealed" => vec![TextFragment::Plain(combatant_record.healed.to_string())],
        "mergedEffHealed" => vec![TextFragment::Plain(combatant_record.healing().effective().to_string())],
        "mergedDamageShield" => vec![TextFragment::Plain(combatant_record.damage_shield.to_string())],
        "mergedOverHeal" => vec![TextFragment::Plain(combatant_record.over_heal.to_string())],
        "mergedHeals" => vec![TextFragment::Plain(combatant_record.heals.to_string())],
        "mergedCritheals" => vec![TextFragment::Plain(combatant_record.critical_heals.to_string())],
        "mergedCures" => vec![TextFragment::Plain(combatant_record.cures.to_string())],
        "mergedAbsorbHeal" => vec![TextFragment::Plain(combatant_record.absorb_heal.to_string())],
        "powerheal" => vec![TextFragment::Plain(combatant_record.mana_restored.to_string())],
        "deaths" => vec![TextFragment::Plain(combatant_record.deaths.to_string())],

        "ParryPct" => vec![TextFragment::Plain(combatant_record.parry_percent.to_string())],
        "BlockPct" => vec![TextFragment::Plain(combatant_record.block_percent.to_string())],
        "maxhit" => strongest_action_fragments(combatant_record.strongest_hit_name.as_str(), combatant_record.strongest_hit_amount, context.settings, &context.number_format),
        "maxheal" => vec![TextFragment::Plain(combatant_record.strongest_heal_amount.to_string())],

        "damagePct" => vec![TextFragment::Plain((combatant_record.damage / combatant_record.duration_seconds).to_string())],
        "healedPct" => vec![TextFragment::Plain((combatant_record.healed / combatant_record.duration_seconds).to_string())],
        "overHealPct" => vec![TextFragment::Plain(combatant_record.healing().overheal_percent().unwrap_or(0.0).to_string())],
        "DirectHitPct" => vec![TextFragment::Plain((combatant_record.direct_hits / combatant_record.hits * 100.0).to_string())],
        "crithitPct" => vec![TextFragment::Plain((combatant_record.critical_hits / combatant_record.hits * 100.0).to_string())],
        "CritDirectHitPct" => vec![TextFragment::Plain((combatant_record.critical_direct_hits / combatant_record.hits * 100.0).to_string())],
        "crithealPct" => vec![TextFragment::Plain((combatant_record.critical_heals / combatant_record.heals * 100.0).to_string())],
        "tohit" => vec![TextFragment::Plain((combatant_record.hits / combatant_record.swings * 100.0).to_string())],

        _ => vec![TextFragment::Plain(0.0.to_string()), TextFragment::Dimmed("%".into())],
    }
}

/// Parry and block are shown without decimals whatever the percentage setting says.
fn whole_percent(value: f64, format: &NumberFormat) -> Vec<TextFragment> {
    vec![TextFragment::Plain(format.format_number(value, 1.0, 0)), TextFragment::Dimmed("%".into())]
}

/// Plain text of a cell (used where dimming does not matter, e.g. the summary line).
pub fn cell_plain_text(column: &str, combatant_record: &CombatantRecord, encounter_record: &EncounterRecord, context: &CellContext) -> String {
    join_plain_text(&cell_fragments(column, combatant_record, encounter_record, context))
}
