# What the three talks asked for, and where it shows in the code

Digest of the three talk transcripts (the "how to write the perfect function" video essay building on Tony Van Eerd's
parable, Sean Parent's "C++ Seasoning", and Tony Van Eerd's value-oriented programming talk, part one). The numbered rules are in
`talk-rules.md`. This page maps them to changes.

| Rule | Example in this repo |
|---|---|
| Calculate vs act, return values (T1, T3) | `Settings::from_stored(text, language)` is pure and `infrastructure::browser_storage` does the reading; `Encounter::rows(PetView)` returns a view instead of merging on arrival; `Encounter::ended_since` answers instead of a flag being flipped in several places |
| Honest signature, effects at the outer layer (T1) | storage and browser language no longer inside `domain/settings`; the socket task lives in `application`; the Encounter takes the pet choice as a parameter |
| Minimal inputs (T3) | `Rate::per_second(amount, seconds)`; `pet_view(fold, local_player_name)` instead of passing the whole settings |
| Preconditions in types (T1) | `EncounterKey`, `Rate`, `Percent`, `Amount`, `ColumnKey`, `PortPreferences::from_code` (hex colours checked once) |
| One abstraction level / no raw loops (T1, T2) | `History::record` reads as its two rules; `fold_pets_into_owners` is the one place that adds pets up |
| Name for what it does (T3) | `ended_since`, `damage_per_second_of`, `personal_damage_per_second`, `whole_damage_per_second_of` |
| Value types by default, strong ids (T2, T3) | `EncounterKey`, `ColumnKey`, `Member` is a plain cloneable value |
| Avoid shared mutable state (T2) | The Encounter is rebuilt per message and never changed; views are computed from it; `Rc` is only used for immutable sharing |
| Polymorphism as a value (T2) | `ColumnKey` and `PetView` are enums matched in one place instead of strings and booleans |
| Behaviour near the data (T3) | DPS/HPS on `EncounterFacts`/`Member`, no longer inline in the cells and the summary line |

Rules that did not apply, and why: the concurrency rules of T2 (the app is single-threaded wasm), and the C++-specific
ones (iterator pairs, `shared_ptr`).
