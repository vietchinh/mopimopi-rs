//! Combat's open host service, tested from the outside: one test per consumer, each using only what
//! `domain::combat` publishes (see the module's doc comment and docs/ddd/decisions/0003-combat-open-host-service.md).

use crate::domain::combat::{Amount, Encounter, EncounterFacts, Figures, History, Percent, PetView, Rate};
use crate::domain::formatting::{cell_plain_text, CellContext, ColumnKey};
use crate::domain::settings::Settings;
use std::rc::Rc;

fn figures(name: &str, job: &str, damage: f64) -> Figures {
    Figures { name: name.into(), job: job.to_string().into(), damage, healed: damage / 2.0, duration_seconds: 10.0, ..Figures::default() }
}

fn fight(title: &str) -> Encounter {
    let facts = EncounterFacts {
        title: title.into(),
        zone_name: "Zurvan".into(),
        duration_seconds: 10.0,
        total_damage: 600.0,
        total_healed: 300.0,
        ..EncounterFacts::default()
    };
    Encounter::from_figures(facts, vec![figures("YOU", "Sch", 400.0), figures("Eos (Future Fade)", "", 100.0), figures("Tank", "Pld", 100.0)], false)
}

/// Consumer: Formatting. It reads members and the Encounter's facts, and asks the Encounter for rates.
#[test]
fn formatting_reads_cells_through_the_published_language() {
    let encounter = fight("Boss");
    let settings = Settings::defaults();
    let context = CellContext::new(&settings, "LB".into(), "Future Fade");
    let rows = encounter.rows(PetView::Separate);
    let you = rows.iter().find(|row| row.name == "YOU").unwrap();
    assert_eq!(cell_plain_text(&ColumnKey::Encdps, you, encounter.facts(), &context), "40");
}

/// Consumer: the tables. They ask for rows, folded or not, and never change the Encounter.
#[test]
fn the_tables_ask_for_rows_in_either_pet_view() {
    let encounter = fight("Boss");
    assert_eq!(encounter.rows(PetView::Separate).len(), 3);
    let folded = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    assert_eq!(folded.len(), 2);
    assert_eq!(folded[0].damage, 500.0);
    assert_eq!(encounter.members().len(), 3);
}

/// Consumer: the top bar. It asks for the local player's standing.
#[test]
fn the_top_bar_asks_for_the_local_players_standing() {
    let standing = fight("Boss").local_standing(PetView::Separate).unwrap();
    assert_eq!((standing.damage_rank, standing.of), (1, 3));
    assert_eq!(standing.damage_per_second, Rate::new(40.0));
}

/// Consumer: the history screen. It reads entries, their number in the zone and the local player's whole-number rates.
#[test]
fn the_history_screen_reads_entries_numbered_within_their_zone() {
    let mut history = History::default();
    history.record(Rc::new(fight("Boss")));
    assert!(history.record(Rc::new(fight("Boss"))).is_none(), "a repeat of the previous fight is dropped");
    let second = history.record(Rc::new(fight("Adds"))).unwrap();
    assert_eq!(second.number_in_zone(), 2);
    assert_eq!(history.entries().len(), 2);
    assert_eq!(history.entries()[1].local_player_dps(PetView::Separate), Some(40.0));
}

/// Consumer: the application layer. It learns that a fight ended, and nothing else.
#[test]
fn the_application_layer_learns_that_a_fight_ended() {
    let encounter = fight("Boss");
    assert_eq!(encounter.ended_since(true).map(|event| event.key), Some(encounter.key()));
    assert_eq!(encounter.ended_since(false), None);
}

#[test]
fn the_value_objects_are_part_of_the_published_language() {
    assert_eq!(Percent::share(Amount::new(1.0), Amount::new(3.0)).value(), 33.33);
}
