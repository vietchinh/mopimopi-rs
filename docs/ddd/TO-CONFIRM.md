# To confirm

Each item was first decided by the developer on the raider's behalf ("do all the phases, I confirm after the fact"). Where the
original MopiMopi's own source (`js/core.js`, `js/process.js`, `js/dic.js` of the pinned copy) settles the question, the answer is
taken from there and marked **Answered by the source**. What is left needs a person or a capture.

## Answered by the original's source

| # | Question | Answer, with the evidence | Where in the port |
|---|---|---|---|
| 1 | Does "DPS" mean damage over the whole encounter's time? | **Yes.** `dic.js` labels `encdps` "DPS" and describes it as "the damage total of the combatant divided by the duration of the encounter. This is more commonly used than P.DPS"; `dps` is labelled "P.DPS". `core.js` computes `encdps = mergedDamage / parent.DURATION` and sorts by `encdps` by default. | `EncounterFacts::damage_per_second_of`, `Stats::personal_damage_per_second` |
| 2 | Are participant names unique within one encounter? | **Yes, by construction.** `Combatant` is a JS object keyed by name: `this.Combatant[tmp[i].val.name] = ...` (`core.js`, `Combatant.prototype.sort`), so the original itself cannot hold two with one name. Pets are found by name too (`petOwner`). | `Encounter::you`, `fold_pets_into_owners` |
| 3 | NPC icon and colour | **What the original draws is the Limit Break look**, and that stays the default: a row with a blank job and no "(" in its name becomes `Job = "LMB"`, `Class = "LMB"` (`core.js` ~538), and its colour is `init.Color.LMB` (`process.js` ~560). The settings page even calls the group "Users – Limit Break, Chocobo, etc." The source has no NPC icon or colour, so the toggle's teal tint (`#4DB6AC`) is still a design choice, not a fact. | `public/images/icon/*/NPC.png`, `DEFAULT_NPC_COLOR` (D1) |
| 5 | The Limit Break's name in other languages | **Partly.** The original never matches a localised name; it matches `Job == "Limit Break"` (comment: "6.0 Korean server Limit Break support") and then treats *any* blank-job row without "(" as the gauge, in every language. It only translates the name for display (`dic.js` `LMB.tt`: リミットブレイク, Transcendance, Limitrausch, 极限技, 리미트 브레이크). The port now also recognises those names as the gauge (affects only the port-only NPC look). Whether ACT ever sends them still needs a non-English capture. | `Participant::sort`, `LIMIT_BREAK_NAMES` |
| 8 | Does toggling "Combine Pets with Owner" re-fold what is on screen? | **Yes.** `process.js` ~57-68 calls `DetachPets()` / `AttachPets()` on the already displayed `lastDPS` / `lastHPS`. The port matches. | `app_state::pet_view`, `Encounter::rows` |
| 9 | Should the history have a size limit? | **The original has none**: `encounterArray.unshift(...)` with no cap (`process.js` `saveLog`). The port matches (parity); a cap would be a deviation. | `History::record`; `MEASUREMENTS.md` |
| 11 | Can a pet's owner be missing from a fight, and what happens? | **The original hides the pet and then crashes.** With merging on, every `isPet` row gets `visible = false` whether or not its owner exists, and `AttachPets` reads `owner.mergedmaxhitval` of a missing owner (a TypeError). The port keeps such a pet visible as an `Unclassified` row drawn as a pet. Recorded as deviation D2. | `Encounter::from_stats`, `deviations.md` D2 |

## Still open

| # | Question | Assumed | Where | If wrong |
|---|---|---|---|---|
| 3b | Icon and colour for the NPC toggle | A teal tint of the Limit Break icon, `#4DB6AC`, off by default | `NPC.png`, `DEFAULT_NPC_COLOR` | Replace the PNGs / the constant |
| 4 | Where the port-only options sit on the settings screen, and their code format | Bottom of Settings > Tool > Custom UI Data; `MOPIPORT1:` + JSON. The original's source cannot say, since it has no such section. | `settings_screens/port_section.rs`, `domain/settings/port_preferences.rs` | Move the call in `settings_screens/mod.rs` |
| 5b | Does ACT send the Limit Break under a localised name in other client languages? | The English name only is certain | `LIMIT_BREAK_NAMES` | Add the real names and a fixture |
| 6 | Where to stop | All seven phases, then a second pass to match the plan's designs. | - | - |
| 7 | The real id of the pinned MopiMopi commit | The pinned copy has no version string and no git data (checked: `README.md`, `index.html`, `js/init.js`), so only the file-tree digest in `pinned-original.md` identifies it. Needs GitHub. | `pinned-original.md` | Fill in the commit id |
| 10 | Event storming | Not held: `event-storming.md` lists the events found in the code and asks the raider to confirm, rename or add. | `docs/ddd/event-storming.md` | Rename in code and glossary |
