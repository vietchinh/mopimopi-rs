//! Rough timing of the paths that run for every combat message. Run with:
//! `cargo test --release --offline benchmarks -- --ignored --nocapture`

use crate::models::act_data::{parse_incoming_message, ActEvent};
use crate::domain::combat::{build_rankings, TableKind};
use crate::domain::formatting::{cell_fragments, CellContext};
use crate::domain::settings::Settings;
use crate::presentation::theme::build_theme_css;
use crate::domain::translations::translations;
use serde_json::{json, Value};
use std::time::Instant;

const PLAYERS: usize = 24;
const ITERATIONS: u32 = 300;

/// A MiniParse broadcast shaped like a real 24-player raid message: every combatant carries ~100 fields.
fn synthetic_broadcast() -> String {
    let template: Value = serde_json::from_str::<Value>(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/captures/mini_parse_beastmaster.json"))).unwrap();
    let mut combatant = template["msg"]["Combatant"]["YOU"].clone();
    for index in 0..60 {
        combatant[format!("extra-{index}")] = json!("1.00K");
    }
    let jobs = ["Sch", "War", "Drg", "Blm", "Mch", "Whm", "Nin", "Brd"];
    let mut combatants = serde_json::Map::new();
    for index in 0..PLAYERS {
        let name = if index == 0 { "YOU".to_string() } else { format!("Player {index} Name") };
        let mut record = combatant.clone();
        record["name"] = json!(name);
        record["Job"] = json!(jobs[index % jobs.len()]);
        record["damage"] = json!((100_000 - index * 3_000).to_string());
        record["healed"] = json!((50_000 - index * 1_000).to_string());
        combatants.insert(name, record);
    }
    for pet in 0..4 {
        let name = format!("Eos (Player {pet} Name)");
        let mut record = combatant.clone();
        record["name"] = json!(name);
        record["Job"] = json!("");
        combatants.insert(name, record);
    }
    let mut message = template["msg"].clone();
    message["Combatant"] = Value::Object(combatants);
    json!({"type": "broadcast", "msgtype": "CombatData", "msg": message}).to_string()
}

fn time<T>(label: &str, mut work: impl FnMut() -> T) {
    for _ in 0..20 {
        std::hint::black_box(work());
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(work());
    }
    let per_run = start.elapsed() / ITERATIONS;
    println!("{label:<34} {:>9.1} µs", per_run.as_secs_f64() * 1e6);
}

#[test]
#[ignore]
fn measure_hot_paths() {
    let text = synthetic_broadcast();
    println!("\nmessage size: {} KB, {} combatants", text.len() / 1024, PLAYERS + 4);
    time("parse message (serde)", || parse_incoming_message(&text).unwrap());

    let Some(ActEvent::CombatData(message)) = parse_incoming_message(&text).unwrap() else { panic!() };
    time("build_rankings (dps + hps)", || build_rankings(&message, true, "YOU"));

    let settings = Settings::defaults();
    let rankings = build_rankings(&message, true, "YOU");
    let columns = settings.column_order("DPS");
    time("format every cell of one table", || {
        let context = CellContext::new(&settings, translations(), "YOU");
        let ranking = rankings.ranking_for_table(TableKind::Damage);
        let mut cells = 0;
        for player in ranking.players.iter().filter(|player| player.is_visible) {
            for column in &columns {
                cells += cell_fragments(column, player, ranking, &context).len();
            }
        }
        cells
    });
    time("build theme stylesheet", || build_theme_css(&settings));
    time("serialise settings (save)", || settings.to_json_text());
    time("clone EncounterRankings", || rankings.clone());
}
