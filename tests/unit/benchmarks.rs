//! Rough timing of what runs for every combat message, and of the structures that grow. Run with:
//! `cargo test --release --offline benchmarks -- --ignored --nocapture`
//!
//! These are native numbers, not wasm: use them to compare steps with each other (does building an Encounter cost more than
//! parsing? is folding pets cheap?), not as absolute browser timings. Phase 7 of the DDD plan uses them to prove that the
//! Encounter-per-message design costs next to nothing next to the JSON parse.

use crate::domain::combat::{Encounter, History, PetView};
use crate::infrastructure::act::overlay_plugin_protocol::{interpret, OverlayPluginEvent};
use serde_json::{json, Value};
use std::rc::Rc;
use std::time::Instant;

const PLAYERS: usize = 24;
const ITERATIONS: u32 = 300;

/// A CombatData message shaped like a real 24-player raid: every participant carries ~80 fields, plus four pets.
fn synthetic_message(active: bool, damage_scale: usize) -> String {
    let template: Value = serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/captures/overlay_plugin_beastmaster.json"))).unwrap();
    let mut participant = template["Combatant"]["YOU"].clone();
    for index in 0..60 {
        participant[format!("extra-{index}")] = json!("1.00K");
    }
    let jobs = ["Sch", "War", "Drg", "Blm", "Mch", "Whm", "Nin", "Brd"];
    let mut participants = serde_json::Map::new();
    for index in 0..PLAYERS {
        let name = if index == 0 { "YOU".to_string() } else { format!("Player {index} Name") };
        let mut record = participant.clone();
        record["name"] = json!(name);
        record["Job"] = json!(jobs[index % jobs.len()]);
        record["damage"] = json!(((100_000 - index * 3_000) * damage_scale).to_string());
        record["healed"] = json!((50_000 - index * 1_000).to_string());
        participants.insert(name, record);
    }
    for pet in 0..4 {
        let owner = if pet == 0 { "YOU".to_string() } else { format!("Player {pet} Name") };
        let name = format!("Eos ({owner})");
        let mut record = participant.clone();
        record["name"] = json!(name);
        record["Job"] = json!("");
        participants.insert(name, record);
    }
    let mut message = template.clone();
    message["Combatant"] = Value::Object(participants);
    message["isActive"] = json!(active.to_string());
    // a different title per fight, or the history would (correctly) skip every one as a repeat of the previous fight
    message["Encounter"]["title"] = json!(format!("Fight {damage_scale}"));
    message.to_string()
}

fn time<T>(label: &str, mut work: impl FnMut() -> T) -> f64 {
    for _ in 0..20 {
        std::hint::black_box(work());
    }
    // The median of the iterations: a shared machine adds long spikes that a mean would carry along.
    let mut samples: Vec<f64> = (0..ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(work());
            start.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let micros = samples[samples.len() / 2];
    println!("{label:<46} {micros:>9.1} µs");
    micros
}

fn encounter_of(text: &str) -> Encounter {
    match interpret(text) {
        Ok(Some(OverlayPluginEvent::CombatData(encounter))) => encounter,
        other => panic!("not combat data: {other:?}"),
    }
}

#[test]
#[ignore]
fn measure_per_message_cost() {
    let text = synthetic_message(true, 1);
    println!("\nmessage size: {} bytes, {} participants", text.len(), PLAYERS + 4);
    let wire = time("parse the wire message (serde)", || assert!(crate::infrastructure::act::overlay_plugin_protocol::parse_wire_only(&text)));
    let whole = time("parse + build a fresh Encounter", || encounter_of(&text));
    println!("  -> building the Encounter adds {:.1} µs ({:.0}% of the parse)", whole - wire, (whole - wire) / wire * 100.0);
    let encounter = encounter_of(&text);
    time("rows, pets separate (clone + sort)", || encounter.rows(PetView::Separate));
    time("rows, pets folded into owners", || encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") }));
}

#[test]
#[ignore]
fn measure_the_unbounded_history() {
    println!();
    for fights in [10usize, 100, 1000] {
        let mut history = History::default();
        let start = Instant::now();
        for index in 0..fights {
            history.record(Rc::new(encounter_of(&synthetic_message(false, index + 1))));
        }
        println!("recorded {fights:>4} fights in {:>8.1} ms (building each Encounter included)", start.elapsed().as_secs_f64() * 1e3);
        time(&format!("history screen: entries().to_vec() at {fights}"), || history.entries().to_vec());
        assert_eq!(history.entries().len(), fights, "every fight must have been recorded");
        let bytes_per_fight = std::mem::size_of::<crate::domain::combat::Participant>() * (PLAYERS + 4);
        println!("  ~{} KiB of participant data kept alive for {fights} fights (participants only, excluding strings)", bytes_per_fight * fights / 1024);
    }
}

/// What one redraw asks of each row: the cells of the usual columns, the colour key and the role.
#[test]
#[ignore]
fn measure_one_redraw_of_the_rows() {
    use crate::domain::formatting::{cell_fragments, CellContext, ColumnKey};
    use crate::domain::settings::Settings;
    let text = synthetic_message(true, 1);
    let encounter = encounter_of(&text);
    let settings = Settings::defaults();
    let context = CellContext::new(&settings, "LB".into(), "Future Fade");
    let columns: Vec<ColumnKey> = ["Name", "Encdps", "Dps", "Enchps", "DamagePct", "Crithit", "MergedDamage", "MergedHealed", "OverHealPct", "Maxhit", "Deaths"]
        .iter()
        .map(|key| ColumnKey::from_key(key))
        .collect();
    let rows = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    let facts = encounter.facts().clone();
    time("cells of one redraw (11 columns x all rows)", || {
        rows.iter().map(|row| columns.iter().map(|column| cell_fragments(column, row, &facts, &context).len()).sum::<usize>()).sum::<usize>()
    });
    time("colour key + role of every row", || rows.iter().map(|row| row.class_code().len() + row.role_palette_key().len()).sum::<usize>());
}
