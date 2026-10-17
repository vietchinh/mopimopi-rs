//! Rust-level profile of what one combat message costs: wall time, heap allocations, and (under valgrind) instructions per
//! function. The same workload exists for the tree before the DDD rework and after it; only the calls differ.
//! Time and allocations: `cargo test --release --offline rust_profile -- --ignored --nocapture --test-threads=1 measure`
//! Instructions: `valgrind --tool=callgrind target/release/deps/<test binary> --ignored --exact rust_profile::profile_loop`

use serde_json::{json, Value};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(layout.size(), Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(new_size, Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}
#[global_allocator]
static COUNTING: Counting = Counting;

const PLAYERS: usize = 24;
const ITERATIONS: usize = 300;

fn synthetic_message() -> String {
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
        record["damage"] = json!((100_000 - index * 3_000).to_string());
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
    message["isActive"] = json!("true");
    message.to_string()
}

const COLUMNS: [&str; 11] = ["Name", "Encdps", "Dps", "Enchps", "DamagePct", "Crithit", "MergedDamage", "MergedHealed", "OverHealPct", "Maxhit", "Deaths"];

/// Median wall time in µs, and the allocations and bytes of one run, for one step.
fn step<T>(label: &str, mut work: impl FnMut() -> T) {
    for _ in 0..20 {
        std::hint::black_box(work());
    }
    let mut samples: Vec<f64> = (0..ITERATIONS)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(work());
            start.elapsed().as_secs_f64() * 1e6
        })
        .collect();
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (a0, b0) = (ALLOCS.load(Relaxed), BYTES.load(Relaxed));
    std::hint::black_box(work());
    let (allocs, bytes) = (ALLOCS.load(Relaxed) - a0, BYTES.load(Relaxed) - b0);
    println!("PROFILE|{label}|{:.1}|{allocs}|{bytes}", samples[samples.len() / 2]);
}

use crate::domain::combat::{Encounter, PetView};
use crate::domain::formatting::{cell_fragments, CellContext, ColumnKey};
use crate::domain::settings::Settings;
use crate::infrastructure::act::overlay_plugin_protocol::{interpret, OverlayPluginEvent};

fn encounter_of(text: &str) -> Encounter {
    match interpret(text) {
        Ok(Some(OverlayPluginEvent::CombatData(encounter))) => encounter,
        other => panic!("not combat data: {other:?}"),
    }
}

fn whole_message(text: &str, settings: &Settings, columns: &[ColumnKey]) -> usize {
    let context = CellContext::new(settings, "LB".into(), "Future Fade");
    let encounter = encounter_of(text);
    let rows = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    rows.iter().map(|row| columns.iter().map(|column| cell_fragments(column, row, encounter.facts(), &context).len()).sum::<usize>()).sum()
}

#[test]
#[ignore]
fn measure() {
    let text = synthetic_message();
    let settings = Settings::defaults();
    let columns: Vec<ColumnKey> = COLUMNS.iter().map(|key| ColumnKey::from_key(key)).collect();
    step("1 interpret: parse the wire message and build the model", || encounter_of(&text));
    let encounter = encounter_of(&text);
    step("2a rows, pets separate", || encounter.rows(PetView::Separate));
    step("2b rows, pets folded into owners", || encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") }));
    let rows = encounter.rows(PetView::FoldedIntoOwners { local_player_name: Some("Future Fade") });
    let context = CellContext::new(&settings, "LB".into(), "Future Fade");
    step("3 cells of one redraw (11 columns x all rows)", || {
        rows.iter().map(|row| columns.iter().map(|column| cell_fragments(column, row, encounter.facts(), &context).len()).sum::<usize>()).sum::<usize>()
    });
    step("4 whole message: 1 + 2b + 3", || whole_message(&text, &settings, &columns));
    step("5 clone of the model (what an Rc avoids)", || encounter.clone());
}

#[test]
#[ignore]
fn profile_loop() {
    let text = synthetic_message();
    let settings = Settings::defaults();
    let columns: Vec<ColumnKey> = COLUMNS.iter().map(|key| ColumnKey::from_key(key)).collect();
    let mut total = 0;
    for _ in 0..2000 {
        total += whole_message(&text, &settings, &columns);
    }
    std::hint::black_box(total);
}
