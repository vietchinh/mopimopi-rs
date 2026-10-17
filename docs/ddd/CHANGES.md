# DDD change log

Every change: what, which DDD principle, which talk rule (see `talk-rules.md`). Newest last.
Status "to confirm" = needs the raider afterwards.

| # | Change | DDD principle | Talk rule | Status |
|---|---|---|---|---|
| 1 | Vision, glossary, 6 decisions, talk checklist (docs/ddd) | Domain vision; ubiquitous language; context map | all | done |
| 2 | Architecture test with shrinking allow-list | Large-scale structure (layers) | T1 honest dependencies | done |
| 3 | Pinned-original note | Conformist | n/a | done; upstream id to confirm |
| 4 | EncounterKey value object for the history key | Value object | T1/T3 preconditions in types | done |
| 5 | Browser storage + language lookup moved from domain/settings to infrastructure/browser_storage; Settings::from_stored is pure | Repository/infrastructure out of the domain; side-effect-free functions | T1 honest signature, T3 calculate vs act | done |
| 6 | common folded into domain as javascript_number (shared kernel); parse_leading_float moved to the ACT edge; architecture layers now 4 | Shared kernel; anticorruption layer (ACT-specific parsing at the edge); modules by meaning | T3 behaviour near the data it needs | done |
| 7 | MAINTAINING.md top rewritten for the current layers | Ubiquitous language / documentation | n/a | done (sections 2-18 still use old names; mapped in the note) |
| 8 | Encounter aggregate with Amount/Rate/Percent/Job, member kinds, events, folded view as a pure method (not wired in yet) | Aggregate root + invariant; value objects; domain events; side-effect-free functions | T1 preconditions in types; T3 calculate vs act, return values | done; member-name uniqueness and DPS meaning: to confirm |
| 9 | Domain `Member` (plain participant) and `Encounter` (+ `EncounterFacts`); ACT's `CombatantRecord`/`EncounterRecord`/`CombatDataMessage` stay at the edge and convert with `From` | Anticorruption layer; aggregate root; ubiquitous language ("Combatant" retired from the domain) | T1 honest signature; T3 minimal inputs | done |
| 10 | Pets are no longer merged when a message arrives; `Encounter::rows(PetView)` returns the folded or separate view and the application layer reads the `pets` option (`app_state::pet_view`) | Side-effect-free function; aggregate stays true; application service decides, domain answers | T3 calculate vs act; T1 calculate vs act | done; behaviour change: toggling `pets` now re-folds the data already on screen (the original does the same on redraw) |
| 11 | The two upward imports (`column_cell`, `player_name` -> infrastructure) are gone; the architecture allow-list is empty | Large-scale structure | T1 | done |
| 12 | DPS/HPS: `EncounterFacts::damage_per_second_of` (encdps, over the encounter), `Member::personal_damage_per_second` (dps, 0 s counts as 1), summary line and history use them | Behaviour in the model (against an anemic model); value object `Rate` | T3 behaviour near the data | done; "does DPS mean the per-encounter rate" to confirm |
| 13 | `History` aggregate: skip a repeat of the previous fight, number within the zone; the zone-counter signal is removed | Aggregate with rules that span entities | T3 calculate vs act; T2 locality of reasoning | done; the list is still unbounded (phase 7) |
| 14 | `ColumnKey` value object for the 43 columns, with an `Unknown(String)` that round-trips | Value object; published language between Display and Settings | T1 preconditions in types | done |
| 15 | `Encounter::ended_since` returns `EncounterEnded`; the primary player arrives as `YouChanged` | Domain events | T3 name for what it does | done |
| 16 | ACT socket task and message sink moved from presentation to `application::app_state::act_connection` | Application service layer | T1 effects at the outer layer | done |
| 17 | Port-only NPC look: `PortPreferences` (own store, own `MOPIPORT1:` code), `Member::is_npc`, NPC icons, a labelled section on the Custom UI Data page; off by default | Anticorruption between the shared published language and port extensions; value object | T1 preconditions in types | done; icon and colour to confirm (D1) |
| 18 | Benchmarks rewritten; measured the per-message and history costs | Evidence before optimising (keep it small) | n/a | done, see MEASUREMENTS.md |
