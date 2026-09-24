//! Content of every table column. The column names are ACT's / the original's field names.

use super::number_format::NumberFormat;
use super::player_name::{display_name, NameOptions};
use super::strongest_action_text::strongest_action_fragments;
use super::text_fragment::{join_plain_text, TextFragment};
use crate::domain::combat::{EncounterRanking, Player};
use crate::domain::settings::Settings;
use crate::domain::translations::Translations;

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
pub fn cell_fragments(column: &str, player: &Player, ranking: &EncounterRanking, context: &CellContext) -> Vec<TextFragment> {
    let format = &context.number_format;
    let merged = &player.merged_stats;
    let rates = &player.rates;
    match column {
        "name" => vec![TextFragment::Plain(display_name(
            player,
            context.local_player_name,
            &context.name_options,
            context.translations,
            &context.language_code,
        ))],
        "duration" => vec![TextFragment::Plain(player.personal_duration_text.clone())],
        "EncounterDuration" => vec![TextFragment::Plain(ranking.encounter.duration_text.clone())],

        "dps" => format.rate_fragments(rates.damage_per_second),
        "encdps" => format.rate_fragments(rates.encounter_damage_per_second),
        "enchps" => format.rate_fragments(rates.encounter_heal_per_second),
        "mergedLast10DPS" => format.rate_fragments(merged.damage_per_second_last_10_seconds),
        "mergedLast30DPS" => format.rate_fragments(merged.damage_per_second_last_30_seconds),
        "mergedLast60DPS" => format.rate_fragments(merged.damage_per_second_last_60_seconds),
        "mergedLast180DPS" => format.rate_fragments(merged.damage_per_second_last_180_seconds),

        "mergedDamage" => format.amount_fragments(merged.damage),
        "mergedSwings" => format.amount_fragments(merged.swings),
        "mergedHits" => format.amount_fragments(merged.hits),
        "mergedDirectHitCount" => format.amount_fragments(merged.direct_hits),
        "mergedCrithits" => format.amount_fragments(merged.critical_hits),
        "mergedCritDirectHitCount" => format.amount_fragments(merged.critical_direct_hits),
        "mergedMisses" => format.amount_fragments(merged.misses),
        "hitfailed" => format.amount_fragments(player.avoided_hits),
        "mergedDamagetaken" => format.amount_fragments(merged.damage_taken),
        "mergedHealstaken" => format.amount_fragments(merged.heals_taken),
        "mergedHealed" => format.amount_fragments(merged.healed),
        "mergedEffHealed" => format.amount_fragments(merged.effective_healed),
        "mergedDamageShield" => format.amount_fragments(merged.damage_shield),
        "mergedOverHeal" => format.amount_fragments(merged.over_heal),
        "mergedHeals" => format.amount_fragments(merged.heals),
        "mergedCritheals" => format.amount_fragments(merged.critical_heals),
        "mergedCures" => format.amount_fragments(merged.cures),
        "mergedAbsorbHeal" => format.amount_fragments(merged.absorb_heal),
        "powerheal" => format.amount_fragments(player.mana_restored),
        "deaths" => format.amount_fragments(player.deaths),

        "ParryPct" => whole_percent(player.parry_percent, format),
        "BlockPct" => whole_percent(player.block_percent, format),
        "maxhit" => strongest_action_fragments(&player.merged_strongest_hit, context.settings, format),
        "maxheal" => strongest_action_fragments(&player.merged_strongest_heal, context.settings, format),

        "damagePct" => format.percent_fragments(rates.damage_percent),
        "healedPct" => format.percent_fragments(rates.healed_percent),
        "overHealPct" => format.percent_fragments(rates.over_heal_percent),
        "DirectHitPct" => format.percent_fragments(rates.direct_hit_percent),
        "crithitPct" => format.percent_fragments(rates.critical_hit_percent),
        "CritDirectHitPct" => format.percent_fragments(rates.critical_direct_hit_percent),
        "crithealPct" => format.percent_fragments(rates.critical_heal_percent),
        "tohit" => format.percent_fragments(rates.accuracy_percent),
        _ => format.percent_fragments(0.0),
    }
}

/// Parry and block are shown without decimals whatever the percentage setting says.
fn whole_percent(value: f64, format: &NumberFormat) -> Vec<TextFragment> {
    vec![TextFragment::Plain(format.format_number(value, 1.0, 0)), TextFragment::Dimmed("%".into())]
}

/// Plain text of a cell (used where dimming does not matter, e.g. the summary line).
pub fn cell_plain_text(column: &str, player: &Player, ranking: &EncounterRanking, context: &CellContext) -> String {
    join_plain_text(&cell_fragments(column, player, ranking, context))
}
