# For the raider to confirm

Everything below was decided by the developer on the raider's behalf, as agreed ("do all the phases, I confirm after the
fact"). Each item says what was assumed, where it lives in the code, and what to change if the assumption is wrong.

| # | Question | Assumed | Where | If wrong |
|---|---|---|---|---|
| 1 | Does "DPS" mean damage over the **whole encounter's** time (MopiMopi's default `encdps`) rather than the player's own active time (`dps`, "P.DPS")? | Yes: the default column, the top bar's "My DPS" and the history's "My DPS" use the encounter's duration. | `EncounterFacts::damage_per_second_of`, `Member::personal_damage_per_second` | Swap which one the column and summary line call; glossary entry in `glossary.md` |
| 2 | Are member names unique within one encounter? | Yes (the Encounter finds owners and the local player by name; ACT keys its Combatant object by name). | `Encounter::local_player`, `fold_pets_into_owners` | Needs an id from ACT; see decision 0002 |
| 3 | NPC icon and colour (D1) | A teal tint of the Limit Break icon, `#4DB6AC`, off by default | `public/images/icon/*/NPC.png`, `DEFAULT_NPC_COLOR` | Replace the PNGs / the constant |
| 4 | Where the port-only options sit on the settings screen, and their code format | Bottom of Settings > Tool > Custom UI Data; `MOPIPORT1:` + JSON | `settings_screens/port_section.rs`, `domain/settings/port_preferences.rs` | Move the call in `settings_screens/mod.rs` |
| 5 | The Limit Break's name in other languages | Not handled: only the English name "Limit Break" is matched by name; a blank job with no owner is treated as the gauge/NPC like the original. Needs a non-English ACT capture to test. | `CombatantKind::classify`, `Member::is_npc` | Add the names and a fixture |
| 6 | Where to stop | All seven phases were done. | - | - |
| 7 | The real id of the pinned MopiMopi commit | Unknown from this environment (no GitHub access) | `pinned-original.md` | Fill in the commit id |
| 8 | Pets merging now follows the option on already-displayed data | Toggling "Combine Pets with Owner" re-folds what is on screen (and the history's "My DPS"); before, only the next message was affected. The original re-applies it on redraw. | `app_state::pet_view`, `Encounter::rows` | Fold on arrival again (not recommended) |
| 9 | The history has no size limit | Kept as is (parity) | `domain/combat/history.rs`; measured in `MEASUREMENTS.md` | Add a cap in `History::record` |
