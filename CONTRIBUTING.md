# Contributing

Read in this order: `README.md`, `docs/ddd/vision.md`, `docs/ddd/glossary.md`, `MAINTAINING.md` (the folder map), `docs/ddd/decisions/`.

## Build and test

* Build the app: `BUILD_STD=1 ./build.sh`
* Unit tests: `cargo test --offline` (the crate is a binary; there is no `--lib`)
* Live comparison against the original MopiMopi: see `tests/README.md` (Playwright, `E2E_MODE=live`). Three tests in
  `bars.spec.ts` ("small bars") fail on the base commit too; do not count them as yours.

## Rules

* One idea per commit. A commit that changes behaviour says which rule it changes.
* The GUI and the settings import/export conform to the original MopiMopi. If your change shows on screen, run the live comparison.
* The settings file stays compatible: no new keys, no changed meanings. Port-only options live in `PortPreferences`.
* Words first: a new concept goes into `docs/ddd/glossary.md` before the code.
* Every change is added as a row to `docs/ddd/CHANGES.md` (the change, the DDD principle, the talk rule from `talk-rules.md`).
* A new decision gets a short file in `docs/ddd/decisions/`: what was decided, why, what was rejected.

## Where does a new rule go?

* About fights: on the Encounter or a value object in `domain/combat`.
* About how a number or word is shown: `domain/formatting`.
* About saved options: `domain/settings`.
* About ACT's message: `infrastructure/act` (and nowhere else; the architecture test checks).
* Only coordinates: `application`. Draws: `presentation`.

A rule that fits two places means a word in the glossary is missing.

## Guards that stay on

`tests/unit/architecture.rs` (layers, Combat's published language, ACT's wire types), the shareable-code golden test, and the
Playwright comparison.

## Small steps

Each phase of the plan in `docs/ddd/CHANGES.md` can be taken as a checklist; pick one row, keep its guard green.
