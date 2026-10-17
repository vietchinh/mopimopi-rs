# Glossary (ubiquitous language)

Words used identically in code, docs and conversation. Raider-owned terms are marked **(confirm)**.

| Term | Meaning | Code |
|---|---|---|
| Encounter | One fight as reported by ACT; the aggregate root. | `domain::combat` |
| Player | A player character in the encounter. | |
| NPC | A non-player combatant (enemy or friendly). | |
| Companion | Pet or chocobo, always owned by a player. | |
| Limit Break | The shared party gauge, reported as a pseudo-combatant. | |
| Unclassified | Technical bucket for rows that fit no other kind. | |
| DPS / HPS | Damage / healing per second **(confirm: per-encounter rate, MopiMopi `encdps`)**. | |
| P.DPS | Damage per second over the combatant's own active time. | |
| History | The list of finished encounters; its own aggregate. | `application::app_state::encounter_history` |
| Column | One of the 43 displayable stats, keyed as in `defaults.json`. | |
| Shareable code | The exported settings string; must stay compatible with the original. | |
| Pinned original | The MopiMopi version on `main`; the port conforms to it. | |
