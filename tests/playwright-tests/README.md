# End-to-end tests

These tests check that the Dioxus port **looks and behaves like the original MopiMopi overlay**: the same
combat data and the same settings must give screenshots that are identical, pixel for pixel, and the same
values when you click things.

They follow the Dioxus guide (<https://dioxuslabs.com/learn/0.7/guides/testing/web>): Playwright starts the app
with `dx serve` (the real wasm dev server) and reuses it if it is already running.

## Run them

```sh
cd playwright-tests
npm install
npx playwright install chromium      # or set E2E_CHROME=/path/to/chrome to use one you already have
npm test                             # everything (about 12-25 minutes, depending on the mode)
npm run test:quick                   # skips the "@tall" group (every setting again with all rows visible)
npx playwright test -g "raid"        # one group or one setting
npm run report                       # open the HTML report of the last run
```

`dx serve` must be on your `PATH`. The app is then at `http://127.0.0.1:8080/mopimopi-rs/` (the `base_path` in
`Dioxus.toml`); `support/urls.ts` knows that.

## Three modes

| Mode | What it does | Needs |
|---|---|---|
| `live` | renders the **original** next to the port on every run and compares them | `MOPIMOPI_ORIGINAL_DIR=/path/to/original` |
| `record` | renders the original and stores what it showed as **goldens** (`goldens/`) | the original |
| `golden` | compares the port with the stored goldens | nothing but the goldens |

`E2E_MODE` picks one. Left unset it is `live` when `MOPIMOPI_ORIGINAL_DIR` is set and `golden` otherwise, so CI
(no original) replays the goldens, and you can check against the original itself whenever you have it:

```sh
MOPIMOPI_ORIGINAL_DIR=~/src/mopimopi-original npm run test:live
MOPIMOPI_ORIGINAL_DIR=~/src/mopimopi-original npm run goldens        # re-record after the original's behaviour is deliberately matched differently
```

### What a golden is
The comparison is **exact** (no colour tolerance, anti-aliased pixels counted), so a golden is just a hash of the
original's pixels: a few bytes per screenshot, and `goldens/` stays small enough to commit. The catch is that a
hash says *that* something differs, not *where*. To see where, rerun in live mode with
`E2E_ARTIFACTS_DIR=/tmp/diffs`, which writes `<name>.original.png`, `.port.png` and `.difference.png`; or record
with `npm run goldens:images` to keep PNGs (they are git-ignored).

Screenshots depend on the browser, its version and the OS. Record and replay in the same environment: the
official Playwright image (`mcr.microsoft.com/playwright:v1.49.1-jammy`) is the easy way to make that stable.

## What is tested

| File | Checks |
|---|---|
| `tests/tables.spec.ts` | the main screen across **every setting**: each flag flipped, each choice set to each option, each slider moved, each colour changed, each column switched on, column order, abbreviations; plus all rows visible (`tall`), other window widths, raid mode, empty tables, no local player, gradient directions, and choices stored as text like the original stores them |
| `tests/settings-screens.spec.ts` | every settings page and tab, and the colour picker's states |
| `tests/color-picker.spec.ts` | presses and drags in the picker, typed hex, the stored text |
| `tests/persistence.spec.ts` | what clicking 11 kinds of controls stores (which keys, which values, which types) |
| `tests/animation.spec.ts` | the bar-growth curve and its length |
| `tests/performance.spec.ts` | layout budget for live updates; in live mode, at most half of the original's layout and script work |
| `tests/comparator.spec.ts` | the comparison itself: a one-level colour difference and a size difference must be caught |
| `tests/smoke.spec.ts` | the port starts without errors; the settings preview folds pets into owners |

The sweep is **generated from the app's own schema** (`src/data/l.json`, `src/data/defaults.json`) in
`support/variants.ts`, so a setting added later is covered by the next run without editing a list.

## How it works

* `support/server.mjs` is a small Node server. It plays OverlayPlugin for the port: a test registers combat data
  for a session (`PUT /__session/<id>`), the page connects to `ws://…/<id>/ws`, subscribes, and gets that data
  (`POST /__push/<id>` sends a later update). With `--static DIR` it also serves the original overlay.
* `support/fixtures.ts` is the `app` fixture: `openOriginal`, `openPort`, `shoot` (a screenshot that has stopped
  changing), `matchScreenshot`, `matchValue`.
* `support/steps.ts` is a small action language (`li#Design`, `tab:tab_name`, `type:navBg=ff8000`, `mouse:759,425`,
  `drag:…`) so one list of steps drives both apps. The port keeps the original's ids and class names, which is
  what makes that possible.
* Fonts, jQuery and OverlayPlugin's helper script, which the original loads from the internet, are replaced, so
  a run needs no network. (Material Icons therefore show as their names, in both apps.)
* `support/data.ts` builds combat data from the original's real ACT sample, with a roster that exercises tanks,
  healers, DPS, pets, Limit Break, a Chocobo and a crafter, in scrambled order.

## Settings

| Variable | Meaning |
|---|---|
| `MOPIMOPI_ORIGINAL_DIR` | checkout of the original overlay (live / record) |
| `E2E_MODE` | `live`, `record` or `golden` |
| `E2E_CHROME` | use this browser instead of Playwright's download |
| `E2E_DX_FLAGS` | flags for `dx serve` (default `--hot-reload false`); `--release` tests the optimized wasm |
| `E2E_SERVE=dist` | serve an existing `dist/` instead of running `dx serve` |
| `E2E_PIXEL_THRESHOLD` | colour tolerance, default `0` (exact) |
| `E2E_ARTIFACTS_DIR` | write original / port / difference images of failures here |
| `E2E_GOLDEN_IMAGES=1` | also keep PNG goldens when recording |
| `E2E_PORT_URL`, `E2E_SUPPORT_URL` | where the apps are served |

## Known, deliberate differences from the original

* The capture tooltip does not mention ACTWebSocket (the port does not support it).
* A typed 3-digit hex colour is drawn as intended; the original draws it wrongly.
  (`tests/color-picker.spec.ts` asserts the port's behaviour on its own.)

## Not covered

Tooltips and toasts, the non-English languages, standby and `autoHideTime`, the history screen, real slider drags in
the settings, touch input in the colour picker, and real fights (only the synthetic roster is used).
