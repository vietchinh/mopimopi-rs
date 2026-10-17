# Event storming (for the raider)

The plan asks for one session with a raider. This is the sheet to bring: the events the code already knows, in the order
they happen in a fight. Confirm, rename or add; then the glossary and the code follow the names you pick.

| Order | Event (past tense) | Who causes it | What the program does | In code |
|---|---|---|---|---|
| 1 | ACT connected | the browser's socket | subscribes to `CombatData` and `ChangePrimaryPlayer` | application event `ActConnected` |
| 2 | Primary player changed | ACT | remembers the character name so the player's pets can be found | `YouChanged` |
| 3 | Combat data arrived | ACT, many times a second | builds a fresh Encounter, shows it | `Encounter::from_stats` |
| 4 | Encounter ended | ACT (an inactive message after active ones) | records the fight in the history | `EncounterEnded`, `History::record` |
| 5 | Encounter repeated | ACT (the same totals again) | drops it | `History::is_repeat` |
| 6 | Pet option changed | the user | the table shows pets folded or separate | `PetView` |
| 7 | ACT disconnected | the socket | shows the connection status, reconnects | application event `ActDisconnected` |

Questions for the raider:

1. Is "Encounter ended" the moment the fight is over for you, or is it later (the boss dying, the wipe being called)?
2. Is there an event we are missing (a wipe, a phase change, a new pull in the same zone)?
3. Is "Limit Break" a participant or just a gauge? (The code treats it as a participant that is at most one per fight.)
4. Are trust/duty NPCs players for you, or do they get their own kind? (The code has `Npc`.)
