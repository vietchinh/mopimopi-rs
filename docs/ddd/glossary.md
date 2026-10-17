# Glossary (ubiquitous language)

Words used identically in code, docs and conversation. Raider-owned terms are marked **(confirm)**.

| Term | Meaning | Code |
|---|---|---|
| Encounter | One fight as reported by ACT; the aggregate root. | `domain::combat` |
| You | The local player, whom ACT always lists as "YOU". At most one per encounter. The real character name arrives separately as `YouChanged` and is used to find your pets and to show your name. | `Participant::You` |
| Player | Any other player character in the encounter. | `Participant::Player` |
| NPC | A trust or duty-support helper: no job, no owner in its name, not the gauge. | `Participant::Npc` |
| Companion | Pet or chocobo, always owned by a player (`PlayerCompanion`, with `owner` and `kind`). | `Participant::Companion` |
| Limit Break | The shared party gauge. At most one per encounter. | `Participant::LimitBreak` |
| Unclassified | Technical bucket for rows that fit no other kind (an unknown pet, a companion whose owner is absent, a second gauge). | `Participant::Unclassified` |
| DPS / HPS | Damage / healing per second **(confirm: per-encounter rate, MopiMopi `encdps`)**. | |
| P.DPS | Damage per second over the participant's own active time. | `Stats::personal_damage_per_second` |
| History | The list of finished encounters; its own aggregate. | `domain::combat::History` |
| Column | One of the 43 displayable stats, keyed as in `defaults.json`. | |
| Shareable code | The exported settings string; must stay compatible with the original. | |
| Pinned original | The MopiMopi version on `main`; the port conforms to it. | |
| Participant | Any row of an encounter, whichever kind. The word "combatant" is ACT's and stays inside the translator. | `domain::combat::Participant` |
| Stats | A participant's numbers and names, without its kind. | `domain::combat::Stats` |
| Standing | Where the local player stands: My DPS, My HPS, rank. | `Encounter::local_standing` |
| Placed participant | A row whose kind the ACT edge has already decided; the domain only receives these (`Participant::you/player/npc/limit_break/pet/chocobo/unclassified`). | `infrastructure/act/data/sorting.rs` |
| Ownership | Whose a participant is, from your point of view: You, one of Your companions, or Others. | `domain::combat::Ownership` |
| Phase | Where a fight is in its life: Running, then Over. | `domain::combat::Phase` |
| Setting key | A setting the program itself uses, by meaning; the text key is the original's and is only for the schema-driven pages. | `domain::settings::SettingKey` |
| Reported | A ratio or per-second figure exactly as ACT reported it, never rounded by us (the original shows these as they came). | `domain::combat::Reported` |
