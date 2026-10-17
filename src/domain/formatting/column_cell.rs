//! Content of every table column. The column names are ACT's / the original's field names.

use super::column_key::ColumnKey;
use super::number_format::NumberFormat;
use super::player_name::{display_name, NameOptions};
use super::strongest_action_text::strongest_action_fragments;
use super::text_fragment::{join_plain_text, TextFragment};
use crate::domain::combat::{Amount, Percent};
use crate::domain::settings::Settings;
use crate::domain::combat::{EncounterFacts, Member};

/// Everything needed to format cells, gathered once per table.
pub struct CellContext<'a> {
    pub settings: &'a Settings,
    /// What the name cell shows for the Limit Break row when names are hidden (a translated text, so the caller supplies it).
    pub limit_break_name: String,
    pub local_player_name: &'a str,
    number_format: NumberFormat,
    name_options: NameOptions,
}

impl<'a> CellContext<'a> {
    pub fn new(settings: &'a Settings, limit_break_name: String, local_player_name: &'a str) -> CellContext<'a> {
        CellContext {
            settings,
            limit_break_name,
            local_player_name,
            number_format: NumberFormat::from_settings(settings),
            name_options: NameOptions::from_settings(settings),
        }
    }
}

/// A percentage, or 0 when the whole is 0 (ACT would show NaN there).
fn share(part: f64, whole: f64, format: &NumberFormat) -> Vec<TextFragment> {
    format.percent_fragments(Percent::share(Amount::new(part), Amount::new(whole)).value())
}

/// Fragments of one cell, for a row with no rank (see `cell_fragments_ranked`).
pub fn cell_fragments(column: &ColumnKey, member: &Member, encounter: &EncounterFacts, context: &CellContext) -> Vec<TextFragment> {
    cell_fragments_ranked(column, member, encounter, context, 0)
}

/// Fragments of one cell. The job icon column (`Class`) is drawn by the caller. `rank` is the row's
/// 0-based position in its table, which the name cell prints when the "rank" setting is on.
pub fn cell_fragments_ranked(column: &ColumnKey, member: &Member, encounter: &EncounterFacts, context: &CellContext, rank: usize) -> Vec<TextFragment> {
    let format = &context.number_format;
    match column {
        ColumnKey::Name => vec![TextFragment::Plain(display_name(
            member,
            context.local_player_name,
            &context.name_options,
            &context.limit_break_name,
            rank,
        ))],
        ColumnKey::Duration => vec![TextFragment::Plain(member.duration_text.clone())],
        ColumnKey::EncounterDuration => vec![TextFragment::Plain(encounter.duration_text.clone())],

        // The original (core.js, Person.prototype.recalculate) never trusts ACT's own per-member
        // rate fields for these: it recomputes damage/duration itself on every redraw, using the
        // member's own duration for "dps" and the *encounter's* duration for "encdps"/"enchps".
        // That matters while a player is still in the fight but not currently dealing damage: their
        // own raw stat block can go a while without a fresh update from ACT, but the encounter's
        // duration keeps ticking every message, so recomputing here is what makes DPS visibly fall
        // as time passes with no new damage, instead of freezing at whatever ACT last reported.
        ColumnKey::Dps => format.rate_fragments(member.personal_damage_per_second().value()),
        ColumnKey::Encdps => format.rate_fragments(encounter.damage_per_second_of(member).value()),
        ColumnKey::Enchps => format.rate_fragments(encounter.healing_per_second_of(member).value()),
        ColumnKey::MergedLast10DPS => format.rate_fragments(member.damage_per_second_last_10_seconds),
        ColumnKey::MergedLast30DPS => format.rate_fragments(member.damage_per_second_last_30_seconds),
        ColumnKey::MergedLast60DPS => format.rate_fragments(member.damage_per_second_last_60_seconds),
        ColumnKey::MergedLast180DPS => format.rate_fragments(member.damage_per_second_last_180_seconds),

        ColumnKey::MergedDamage => format.amount_fragments(member.damage),
        ColumnKey::MergedSwings => format.amount_fragments(member.swings),
        ColumnKey::MergedHits => format.amount_fragments(member.hits),
        ColumnKey::MergedDirectHitCount => format.amount_fragments(member.direct_hits),
        ColumnKey::MergedCrithits => format.amount_fragments(member.critical_hits),
        ColumnKey::MergedCritDirectHitCount => format.amount_fragments(member.critical_direct_hits),
        ColumnKey::MergedMisses => format.amount_fragments(member.misses),
        ColumnKey::Hitfailed => format.amount_fragments(member.avoided_hits),
        ColumnKey::MergedDamagetaken => format.amount_fragments(member.damage_taken),
        ColumnKey::MergedHealstaken => format.amount_fragments(member.heals_taken),
        ColumnKey::MergedHealed => format.amount_fragments(member.healed),
        ColumnKey::MergedEffHealed => format.amount_fragments(member.healing().effective()),
        ColumnKey::MergedDamageShield => format.amount_fragments(member.damage_shield),
        ColumnKey::MergedOverHeal => format.amount_fragments(member.over_heal),
        ColumnKey::MergedHeals => format.amount_fragments(member.heals),
        ColumnKey::MergedCritheals => format.amount_fragments(member.critical_heals),
        ColumnKey::MergedCures => format.amount_fragments(member.cures),
        ColumnKey::MergedAbsorbHeal => format.amount_fragments(member.absorb_heal),
        ColumnKey::Powerheal => format.amount_fragments(member.mana_restored),
        ColumnKey::Deaths => format.amount_fragments(member.deaths),

        ColumnKey::ParryPct => whole_percent(member.parry_percent, format),
        ColumnKey::BlockPct => whole_percent(member.block_percent, format),
        ColumnKey::Maxhit => strongest_action_fragments(&member.strongest_hit_name, member.strongest_hit_amount, context.settings, format),
        ColumnKey::Maxheal => strongest_action_fragments(&member.strongest_heal_name, member.strongest_heal_amount, context.settings, format),

        ColumnKey::DamagePct => share(member.damage, encounter.total_damage, format),
        ColumnKey::HealedPct => share(member.healed, encounter.total_healed, format),
        ColumnKey::OverHealPct => format.percent_fragments(member.healing().overheal_share().value()),
        ColumnKey::DirectHitPct => share(member.direct_hits, member.hits, format),
        ColumnKey::CrithitPct => share(member.critical_hits, member.hits, format),
        ColumnKey::CritDirectHitPct => share(member.critical_direct_hits, member.hits, format),
        ColumnKey::CrithealPct => format.percent_fragments(member.critical_heal_share().value()),
        ColumnKey::Tohit => share(member.hits, member.swings, format),

        ColumnKey::Class | ColumnKey::Unknown(_) => format.percent_fragments(0.0),
    }
}

/// Parry and block are shown without decimals whatever the percentage setting says.
fn whole_percent(value: f64, format: &NumberFormat) -> Vec<TextFragment> {
    vec![TextFragment::Plain(format.format_number(value, 1.0, 0)), TextFragment::Dimmed("%".into())]
}

/// Plain text of a cell (used where dimming does not matter, e.g. the summary line).
pub fn cell_plain_text(column: &ColumnKey, member: &Member, encounter: &EncounterFacts, context: &CellContext) -> String {
    join_plain_text(&cell_fragments(column, member, encounter, context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::combat::Figures;
    use crate::domain::settings::Settings;

    fn member(damage: f64, hits: f64, swings: f64) -> Member {
        Member::player(Figures { damage, hits, swings, damage_per_second: damage, ..Default::default() })
    }

    #[test]
    fn zero_over_zero_percentages_are_zero_not_nan() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, "Limit Break".to_string(), "");
        let member = member(0.0, 0.0, 0.0);
        let encounter = EncounterFacts::default();
        for column in [ColumnKey::DamagePct, ColumnKey::HealedPct, ColumnKey::DirectHitPct, ColumnKey::CrithitPct, ColumnKey::CritDirectHitPct, ColumnKey::CrithealPct, ColumnKey::Tohit] {
            let text = cell_plain_text(&column, &member, &encounter, &context);
            assert!(!text.contains("NaN"), "{column:?} produced {text:?}");
        }
    }

    #[test]
    fn dps_uses_personal_duration_encdps_uses_the_encounters() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, "Limit Break".to_string(), "");
        let member = Member::player(Figures { damage: 1000.0, duration_seconds: 10.0, ..Default::default() });
        let encounter = EncounterFacts { duration_seconds: 20.0, ..Default::default() };
        assert_eq!(cell_plain_text(&ColumnKey::Dps, &member, &encounter, &context), "100"); // 1000 / 10 (personal)
        assert_eq!(cell_plain_text(&ColumnKey::Encdps, &member, &encounter, &context), "50"); // 1000 / 20 (encounter)
    }

    #[test]
    fn a_personal_duration_of_zero_divides_by_one_instead_of_by_zero() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, "Limit Break".to_string(), "");
        let member = Member::player(Figures { damage: 1234.0, duration_seconds: 0.0, ..Default::default() });
        let encounter = EncounterFacts::default();
        assert_eq!(cell_plain_text(&ColumnKey::Dps, &member, &encounter, &context), "1,234");
    }

    /// The regression this pair of columns exists to fix: a player's own raw rate field can go
    /// stale while they land no new hits, but "encdps" must still visibly fall as the fight goes on
    /// without them, because it is recomputed from the (unchanging) damage and the (ever-increasing)
    /// encounter duration on every redraw, exactly like the original overlay's own `recalculate()`.
    #[test]
    fn encdps_keeps_falling_while_damage_is_flat_and_the_encounter_keeps_going() {
        let settings = Settings::defaults();
        let context = CellContext::new(&settings, "Limit Break".to_string(), "");
        // Same member, same damage, only the encounter's duration has moved on: this is exactly
        // what "still in combat but not attacking" looks like across two messages.
        let member = Member::player(Figures { damage: 1000.0, ..Default::default() });
        let earlier = EncounterFacts { duration_seconds: 10.0, ..Default::default() };
        let later = EncounterFacts { duration_seconds: 20.0, ..Default::default() };
        assert_eq!(cell_plain_text(&ColumnKey::Encdps, &member, &earlier, &context), "100");
        assert_eq!(cell_plain_text(&ColumnKey::Encdps, &member, &later, &context), "50");
    }
}
