# Domain vision

**Statement.** MopiMopi shows a raider, while they play, how a fight is going: who did how much damage and
healing, at what rate, and how that compares with earlier fights. It is a *conformist* port of the original
MopiMopi overlay: what it shows and how it is configured must match the original.

*DDD principle: domain vision statement (strategic design). Status: draft, awaiting raider confirmation.*

## Subdomains

| Subdomain | Kind | Why |
|---|---|---|
| Combat (encounter, combatants, rates, history) | **Core** | The reason the overlay exists; the rules are subtle (pets, Limit Break, DPS meanings). |
| Display (columns, formatting, themes, layout) | Supporting | Must match the original; no competitive edge. |
| Settings (shareable code, import/export) | Supporting | Must stay byte-compatible with the original. |
| ACT edge (websocket, wire messages) | Generic | Someone else's protocol; we only translate it. |

## Bounded contexts and context map

* **ACT edge** (infrastructure/act) -> **Combat**: *anticorruption layer*. The wire format never leaks inward.
* **Combat** -> **Display**: *open host service* with a *published language* (see `glossary.md`).
* **Settings** -> Combat/Display: settings arrive as parameters; Combat never reads storage.
* **Port -> original MopiMopi**: *conformist*. The port has no say in upstream; `main` of the repo is the pinned original.

## Layers (large-scale structure)

`presentation -> application -> infrastructure -> domain` (the former `common` is folded into the domain).
A layer only imports from layers to its right. Enforced by `tests/unit/architecture.rs`.
