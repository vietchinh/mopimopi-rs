//! Content of every table column. The column names are ACT's / the original's field names.

use super::number_format::NumberFormat;
use super::player_name::{display_name, NameOptions};
use super::strongest_action_text::strongest_action_fragments;
use super::text_fragment::{join_plain_text, TextFragment};
use crate::domain::combat::percent_of;
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

/// A percentage, or 0 when the whole is 0 (ACT would show NaN there).
fn share(part: f64, whole: f64, format: &NumberFormat) -> Vec<TextFragment> {
    format.percent_fragments(percent_of(part, whole).unwrap_or(0.0))
}

/// Fragments of one cell. The job icon column (`Class`) is drawn by the caller.
pub fn cell_fragments(column: &str, combatant: &CombatantRecord, encounter: &EncounterRecord, context: &CellContext) -> Vec<TextFragment> {
    let format = &context.number_format;
    match column {
        "name" => vec![TextFragment::Plain(display_name(
            combatant,
            context.local_player_name,
            &context.name_options,
            context.translations,
            &context.language_code,
        ))],
        "duration" => vec![TextFragment::Plain(combatant.duration_text.clone())],
        "EncounterDuration" => vec![TextFragment::Plain(encounter.duration_text.clone())],

        // The original (core.js, Person.prototype.recalculate) never trusts ACT's own per-combatant
        // rate fields for these: it recomputes damage/duration itself on every redraw, using the
        // combatant's own duration for "dps" and the *encounter's* duration for "encdps"/"enchps".
        // That matters while a player is still in the fight but not currently dealing damage: their
        // own raw stat block can go a while without a fresh update from ACT, but the encounter's
        // duration keeps ticking every message, so recomputing here is what makes DPS visibly fall
        // as time passes with no new damage, instead of freezing at whatever ACT last reported.
        "dps" => {
            let personal_duration = if combatant.duration_seconds == 0.0 { 1.0 } else { combatant.duration_seconds };
            format.rate_fragments(combatant.damage / personal_duration)
        }
        "encdps" => format.rate_fragments(combatant.damage / encounter.duration_seconds),
        "enchps" => format.rate_fragments(combatant.healed / encounter.duration_seconds),
        "mergedLast10DPS" => format.rate_fragments(combatant.damage_per_second_last_10_seconds),
        "mergedLast30DPS" => format.rate_fragments(combatant.damage_per_second_last_30_seconds),
        "mergedLast60DPS" => format.rate_fragments(combatant.damage_per_second_last_60_seconds),
        "mergedLast180DPS" => format.rate_fragments(combatant.damage_per_second_last_180_seconds),

        "mergedDamage" => format.amount_fragments(combatant.damage),
        "mergedSwings" => format.amount_fragments(combatant.swings),
        "mergedHits" => format.amount_fragments(combatant.hits),
        "mergedDirectHitCount" => format.amount_fragments(combatant.direct_hits),
        "mergedCrithits" => format.amount_fragments(combatant.critical_hits),
        "mergedCritDirectHitCount" => format.amount_fragments(combatant.critical_direct_hits),
        "mergedMisses" => format.amount_fragments(combatant.misses),
        "hitfailed" => format.amount_fragments(combatant.avoided_hits),
        "mergedDamagetaken" => format.amount_fragments(combatant.damage_taken),
        "mergedHealstaken" => format.amount_fragments(combatant.heals_taken),
        "mergedHealed" => format.amount_fragments(combatant.healed),
        "mergedEffHealed" => format.amount_fragments(combatant.healing().effective()),
        "mergedDamageShield" => format.amount_fragments(combatant.damage_shield),
        "mergedOverHeal" => format.amount_fragments(combatant.over_heal),
        "mergedHeals" => format.amount_fragments(combatant.heals),
        "mergedCritheals" => format.amount_fragments(combatant.critical_heals),
        "mergedCures" => format.amount_fragments(combatant.cures),
        "mergedAbsorbHeal" => format.amount_fragments(combatant.absorb_heal),
        "powerheal" => format.amount_fragments(combatant.mana_restored),
        "deaths" => format.amount_fragments(combatant.deaths),

        "ParryPct" => whole_percent(combatant.parry_percent, format),
        "BlockPct" => whole_percent(combatant.block_percent, format),
        "maxhit" => strongest_action_fragments(&combatant.strongest_hit_name, combatant.strongest_hit_amount, context.settings, format),
        "maxheal" => strongest_action_fragments(&combatant.strongest_heal_name, combatant.strongest_heal_amount, context.settings, format),

        "damagePct" => share(combatant.damage, encounter.total_damage, format),
        "healedPct" => share(combatant.healed, encounter.total_healed, format),
        "overHealPct" => format.percent_fragments(combatant.healing().overheal_percent().unwrap_or(0.0)),
        "DirectHitPct" => share(combatant.direct_hits, combatant.hits, format),
        "crithitPct" => share(combatant.critical_hits, combatant.hits, format),
        "CritDirectHitPct" => share(combatant.critical_direct_hits, combatant.hits, format),
        "crithealPct" => format.percent_fragments(combatant.critical_heal_percent().unwrap_or(0.0)),
        "tohit" => share(combatant.hits, combatant.swings, format),

        _ => format.percent_fragments(0.0),
    }
}

/// Parry and block are shown without decimals whatever the percentage setting says.
fn whole_percent(value: f64, format: &NumberFormat) -> Vec<TextFragment> {
    vec![TextFragment::Plain(format.format_number(value, 1.0, 0)), TextFragment::Dimmed("%".into())]
}

/// Plain text of a cell (used where dimming does not matter, e.g. the summary line).
pub fn cell_plain_text(column: &str, combatant: &CombatantRecord, encounter: &EncounterRecord, context: &CellContext) -> String {
    join_plain_text(&cell_fragments(column, combatant, encounter, context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::settings::Settings;
    use crate::domain::translations::translations;

    fn combatant(damage: f64, hits: f64, swings: f64) -> CombatantRecord {
        CombatantRecord { damage, hits, swings, damage_per_second: damage, ..Default::default() }
    }

    #[test]
    fn zero_over_zero_percentages_are_zero_not_nan() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, translations(), "");
        let combatant = combatant(0.0, 0.0, 0.0);
        let encounter = EncounterRecord::default();
        for column in ["damagePct", "healedPct", "DirectHitPct", "crithitPct", "CritDirectHitPct", "crithealPct", "tohit"] {
            let text = cell_plain_text(column, &combatant, &encounter, &context);
            assert!(!text.contains("NaN"), "{column} produced {text:?}");
        }
    }

    #[test]
    fn dps_uses_personal_duration_encdps_uses_the_encounters() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, translations(), "");
        let combatant = CombatantRecord { damage: 1000.0, duration_seconds: 10.0, ..Default::default() };
        let encounter = EncounterRecord { duration_seconds: 20.0, ..Default::default() };
        assert_eq!(cell_plain_text("dps", &combatant, &encounter, &context), "100"); // 1000 / 10 (personal)
        assert_eq!(cell_plain_text("encdps", &combatant, &encounter, &context), "50"); // 1000 / 20 (encounter)
    }

    #[test]
    fn a_personal_duration_of_zero_divides_by_one_instead_of_by_zero() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, translations(), "");
        let combatant = CombatantRecord { damage: 1234.0, duration_seconds: 0.0, ..Default::default() };
        let encounter = EncounterRecord::default();
        assert_eq!(cell_plain_text("dps", &combatant, &encounter, &context), "1,234");
    }

    /// The regression this pair of columns exists to fix: a player's own raw rate field can go
    /// stale while they land no new hits, but "encdps" must still visibly fall as the fight goes on
    /// without them, because it is recomputed from the (unchanging) damage and the (ever-increasing)
    /// encounter duration on every redraw, exactly like the original overlay's own `recalculate()`.
    #[test]
    fn encdps_keeps_falling_while_damage_is_flat_and_the_encounter_keeps_going() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, translations(), "");
        // Same combatant, same damage, only the encounter's duration has moved on: this is exactly
        // what "still in combat but not attacking" looks like across two messages.
        let combatant = CombatantRecord { damage: 1000.0, ..Default::default() };
        let earlier = EncounterRecord { duration_seconds: 10.0, ..Default::default() };
        let later = EncounterRecord { duration_seconds: 20.0, ..Default::default() };
        assert_eq!(cell_plain_text("encdps", &combatant, &earlier, &context), "100");
        assert_eq!(cell_plain_text("encdps", &combatant, &later, &context), "50");
    }
}
