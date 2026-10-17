# DDD documentation

Start here. The port is a conformist copy of MopiMopi (see `vision.md`); this folder explains the domain model and records every
change made to reach it.

* `vision.md` – what the program is for, subdomains, bounded contexts, layers
* `glossary.md` – the words used in code and conversation
* `decisions/` – one short file per decision
* `CHANGES.md` – every change, the DDD principle behind it and the talk rule it follows
* `talk-rules.md`, `talks.md` – the checklist drawn from the three talks and where it shows in the code
* `deviations.md` – the only place the port differs from the original (D1)
* `pinned-original.md` – which MopiMopi the port follows and how upstream changes are brought in
* `MEASUREMENTS.md` – what the design costs
* `TO-CONFIRM.md` – what the raider still has to confirm

Keeping it DDD afterwards: new words go into `glossary.md` first; a new rule goes on the type that owns the data (not into a
component); a new import must pass `tests/unit/architecture.rs` (layers only import downwards, the exception list is empty and
must stay empty); a change to what the screen shows is checked with the live Playwright comparison against the original.
