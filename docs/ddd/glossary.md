# Glossary (ubiquitous language)

Words used identically in code, docs and conversation. Raider-owned terms are marked **(confirm)**.

| Term | Meaning | Code |
|---|---|---|
| Encounter | One fight as reported by ACT; the aggregate root. | `domain::combat` |
| Player | A player character in the encounter. | `Member::Player` |
| NPC | A trust or duty-support helper: no job, no owner in its name, not the gauge. | `Member::Npc` |
| Companion | Pet or chocobo, always owned by a player (`PlayerCompanion`, with `owner` and `kind`). | `Member::Companion` |
| Limit Break | The shared party gauge. At most one per encounter. | `Member::LimitBreak` |
| Unclassified | Technical bucket for rows that fit no other kind (an unknown pet, a companion whose owner is absent, a second gauge). | `Member::Unclassified` |
| DPS / HPS | Damage / healing per second **(confirm: per-encounter rate, MopiMopi `encdps`)**. | |
| P.DPS | Damage per second over the member's own active time. | `Figures::personal_damage_per_second` |
| History | The list of finished encounters; its own aggregate. | `domain::combat::History` |
| Column | One of the 43 displayable stats, keyed as in `defaults.json`. | |
| Shareable code | The exported settings string; must stay compatible with the original. | |
| Pinned original | The MopiMopi version on `main`; the port conforms to it. | |
| Member | Any row of an encounter, whichever kind. The word "combatant" is ACT's and stays inside the translator. | `domain::combat::Member` |
| Figures | A member's numbers and names, without its kind. | `domain::combat::Figures` |
| Standing | Where the local player stands: My DPS, My HPS, rank. | `Encounter::local_standing` |
