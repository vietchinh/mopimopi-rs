# 0007 Conformance review: seven findings and what was decided
Context: an audit against Evans's Domain-Driven Design Reference (read in full; Vernon's *Implementing DDD* is not freely available, so the points taken from it are from memory) found seven places where the code did not follow the model.

1. ACT's conventions (job "0" chocobo, blank job pet, "Name (Owner)", the gauge's job text) were in the domain. They now live in `infrastructure/act/data/sorting.rs` (Anticorruption Layer). The domain gets placed participants. Guard: `architecture::acts_conventions_stay_in_the_act_edge`.
2. Two vocabularies (`ParticipantKind` and `Participant`). `ParticipantKind` is deleted; the original's "NPC looks like the gauge" is one named question, `Participant::looks_like_limit_break`.
3. "YOU" in the presentation. Now `Participant::is_you`, `ownership`, `is_your_pet`. One deliberate deviation: the old test `row_id.contains("YOU")` also matched names that contain capital YOU; it no longer does (CHANGES row 33).
4. Primitive obsession. `Stats` holds `Amount`, `Rate`, `Percent`. Durations stay seconds (open).
5. Lifecycle. `Phase` (Running, Over). ACT sends no fight id, so identity stays the `EncounterKey` (title and totals, as the original), meaningful once the fight is Over. A fight is not an entity with an id in the upstream model; inventing one would not conform.
6. String-keyed settings. `SettingKey` for every setting the program knows by meaning; text keys remain for the schema-driven pages only. Guard: `architecture::known_settings_are_not_named_by_text_literals`. The file format is untouched.
7. Dead code removed.

DDD principle: Anticorruption Layer, Ubiquitous Language (one word per idea), Value Objects, Domain Events, Intention-Revealing Interfaces.
Status: accepted. Open: durations as a value type; the second reader of the books (Vernon) from the raider's copy.
