# Deviations from the original MopiMopi

The port is a conformist copy. Every place where it deliberately differs is listed here, with the reason, where it is
visible and how the parity tests treat it. There are two so far (D1 is an option; D2 is an edge case).

## D1: "NPCs look like themselves" (port-only option)

* **What.** Trust and duty-support NPCs (ACT sends them with no job and no "(owner)" in the name) are drawn by the
  original exactly like the Limit Break: its icon and its gold bar. With the option on, they get their own icon (`NPC.png`
  in every icon set) and a colour of their own. **Off by default**, so out of the box the port matches the original.
* **Where it lives.** A separate store, `localStorage["MopiMopi_port_preferences"]`, with its own export and import
  (`MOPIPORT1:` + JSON). It is never part of `Mopi2_HAERU` or of the "Custom UI Data" code, and neither import touches the
  other store (tests: `tests/unit/domain/settings/port_preferences.rs`, `playwright-tests/tests/port-npc-look.spec.ts`).
* **Where you see it.** At the bottom of Settings > Tool > Custom UI Data, in a box labelled "Port-only options" that says
  so on screen.
* **Parity tests.** The box is hidden (screenshots), skipped (computed-style capture) or filtered out (page text) in the
  three specs that visit that page, so the rest of the page is still compared with the original.
* **To confirm with the raider.** The icon (a teal tint of the Limit Break icon, a placeholder) and the default colour `#4DB6AC`.
* **DDD.** Anticorruption: the shared published language (the original's settings and code) is kept clean of the port's own
  extensions.

## D2: a pet whose owner is not in the fight stays visible

* **What.** With "Combine Pets with Owner" on, the original hides every pet row (`visible = false` for each `isPet`,
  `Combatant.prototype.sort` in `core.js`), whether or not its owner is in the data, and throws when it later reads the missing
  owner (`AttachPets`: `owner.mergedmaxhitval`). The port keeps such a pet as its own row, drawn as a pet (`Unclassified` with
  `was_companion_of`).
* **Why.** Nothing ACT sent should vanish, and a crash is not behaviour worth copying. ACT lists the owner in every real fight we
  have captured, so this only shows with partial data.
* **Parity tests.** The live comparison never feeds it (`ROSTER_WITHOUT_YOU` exists because the original crashes); unit tests cover it
  (`tests/unit/domain/combat/encounter.rs`).
* **DDD.** An aggregate invariant (a companion has an owner) with a safe place for data that breaks it.
