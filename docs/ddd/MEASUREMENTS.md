# Measurements (phase 7)

Run with `cargo test --release --offline benchmarks -- --ignored --nocapture --test-threads=1`
(`tests/unit/benchmarks.rs`). Native release build, one machine, nothing else running: use the numbers to compare steps
with each other, not as browser timings. Message: 24 players + 4 pets, about 80 fields each, 43.8 KB.

| Step (per message) | Time | Reading |
|---|---:|---|
| Parse the wire message (serde, lenient values) | ~200 µs | The existing cost; unchanged by this work. |
| Parse **and** build a fresh Encounter | ~203 µs | Building the Encounter adds ~3 µs, about 1-2% of the parse. A fresh Encounter per message (ACT sends full snapshots) is not a cost worth optimising. |
| `rows`, pets separate (clone + sort) | ~4.5 µs | Paid per redraw of the tables, summary line and history. |
| `rows`, pets folded into owners | ~6.5 µs | The folded view costs ~2 µs more; no cache is needed. |

| History size | Time to record (incl. building each Encounter) | History screen redraw (`entries().to_vec()`) | Participant data kept alive (structs only) |
|---:|---:|---:|---:|
| 10 fights | 6 ms | 0.2 µs | ~105 KiB |
| 100 fights | 62 ms | 3.8 µs | ~1 MiB |
| 1000 fights | 617 ms (all parse) | 42 µs | ~10 MiB |

## Conclusions

* The DDD design (Encounter per message, views computed on demand) is not a performance risk. No memo or cache was added for
  the folded view; add one only if a profile in the browser ever shows `rows` on the hot path.
* The history is **unbounded**, as before. A raid night (about 100 fights) is about 1 MiB of participant data plus their strings,
  which is fine. Thousands of fights in one session would grow without limit. A cap is a one-line rule in
  `History::record` if the raider wants one (listed in `TO-CONFIRM.md`, item 9); it was not added because the original
  has none and the port must match it.
* Not measured here: wasm timings in the browser. The existing Playwright `performance.spec.ts` (skipped in the parity
  run) is the place for those.

## Second pass (typed participants): in-depth browser test

Spec: `playwright-tests/tests/performance-deep.spec.ts` (its name keeps it out of the parity run). A raid of 24 players with pets, a
chocobo, a trust NPC and the Limit Break (36 rows), served by the test server to three builds in headless Chromium, one test at a
time: the original MopiMopi, the port before this work (`4c46ec7`) and the port after. Run it with `run-perf` style env:
`PERF_LABEL=<name> PERF_APPS=original,port PERF_OUT=<file>.jsonl npx playwright test performance-deep --workers=1`.
Main-thread time includes a constant `requestAnimationFrame` loop that the probe runs in every build, so compare builds with each
other, not with zero. "Wall" is dominated by the test harness (HTTP push and `page.evaluate`), so it is not a speed measure.

| Scenario | Build | Layouts | Main thread (ms) | Script (ms) | Frame p95 / max (ms) | Long tasks |
|---|---|---:|---:|---:|---:|---:|
| 150 updates at ~15/s, animation on | original | 713 | 14091 | 3642 | 33 / 67 | 46 |
| | port before | 526 | 7582 | 1286 | 17 / 34 | 0 |
| | port after | 532 | 7006 | 1273 | 17 / 34 | 0 |
| 150 updates, animation off | original | 707 | 14707 | 3829 | 33 / 67 | 62 |
| | port before | 517 | 7565 | 1381 | 17 / 50 | 0 |
| | port after | 527 | 7097 | 1281 | 17 / 50 | 0 |
| 150 updates, pets kept separate | original | 711 | 15673 | 4023 | 33 / 67 | 71 |
| | port before | 462 | 8470 | 1576 | 17 / 67 | 0 |
| | port after | 521 | 8281 | 1488 | 17 / 67 | 0 |
| 300 updates back to back | original | 916 | 23485 | 6700 | 50 / 83 | 128 |
| | port before | 105 | 4081 | 1756 | 33 / 100 | 0 |
| | port after | 100 | 3549 | 1617 | 33 / 83 | 0 |
| 40 fights ending (history grows) | original | 708 | 15966 | 4477 | 33 / 83 | 85 |
| | port before | 376 | 6379 | 1425 | 17 / 33 | 0 |
| | port after | 389 | 6024 | 1359 | 17 / 67 | 0 |

Reading:

* **The refactor costs nothing measurable in the browser.** After vs before: main-thread time is 0 to 13% lower, script time is
  equal or lower, layouts and style recalculations are the same. The port does about half the original's main-thread work and
  script time, and has no long task and no frame over 100 ms in any scenario.
* **The history does not leak in practice.** After 40 fights the JS heap was ~2 MB in both ports (the original: 17 MB). The DOM node
  counter ranged from 738 to 2450 across repeats of the same build, so it reflects garbage-collection timing, not growth. The
  Encounters themselves live in wasm memory, which this probe does not read: the native number below stands (~10 KiB per fight).
* **Native, quiet machine:** wire parse 236 µs; parse plus building the typed Encounter 244 µs (+8 µs, 3%; it was +3 µs before the
  participants were sorted into kinds); `rows` 5.3 µs separate, 6.9 µs folded; 1000 fights recorded in 0.9 s.
  (A first run during a parallel build showed +64 µs; it was CPU contention. Run benchmarks on a quiet machine.)
* Wasm size: 2.30 MB before, 2.35 MB after (+2%).
* Not covered: a real ACT session, a phone-class CPU, and wasm memory growth over hours. The first two need a device; the last one
  is bounded by the unbounded history (TO-CONFIRM item 9).

## Third pass (after the conformance review)

Native, release, median of 300 runs, the same message, the pre-review tree and the current tree measured alternately:

| Step | Before the review | After |
|---|---:|---:|
| Parse the wire message | ~230 µs | ~230 µs |
| Parse and build a fresh Encounter | ~235 µs | ~230 µs |
| Cells of one redraw (11 columns x 28 rows) | ~52 µs | ~50 µs |
| `rows`, pets folded | ~6.7 µs | ~6.7 µs |

A first version of the typed `Stats` rounded every wire figure at ingest (`p_float` formats and parses a string): 30-90 µs more per message, and it rounded figures the original shows raw. `Reported` fixed both (CHANGES row 39). `tests/unit/benchmarks.rs` now has `measure_one_redraw_of_the_rows`.

## Fourth pass: original, before DDD, after DDD (full report)

Five interleaved browser rounds of all five scenarios, native time and allocations, instructions per function (callgrind), wasm size by function (twiggy) and start-up, for `4c46ec7` (before) and this branch (after). The report is the "MopiMopi performance: original vs before DDD vs after" document; the raw data is in `docs/ddd/measurements/`:

* `browser-runs-5-rounds.jsonl`: one line per scenario run (labels `predd*` = before, `head*` = after; app `original` = upstream).
* `native-timing-runs.txt`: `build|round|step|µs|allocations|bytes`.
* `rust_profile_before_ddd.rs.txt`: the same workload as `tests/unit/rust_profile.rs`, written against the code before the rework (not compiled; rename to `.rs` and register it in `main.rs` in a checkout of `4c46ec7`).
* `wasm-size-by-module.txt`, `startup-runs.json`.

Headline (final build `87ccd18`): per message the port now costs 1,649,553 instructions against 3,334,195 before the rework (-50.5%; `04063f4` before the helpers were made pure: 1,633,732), 1,300 allocations against 2,698 (-52%), 134.2 µs against 293.4 µs native (same run); in the browser the main thread is level or better in every scenario (-0.5% to -14.8%, script -8% to -20%); wasm 2,284,390 B, -0.6%; start-up unchanged (measured on `04063f4`; browser runs not repeated for `87ccd18`). The first build had been +2.8% instructions; row sharing, a pruned model build, arithmetic rounding, one-pass message reading and single-string number formatting brought it down. Parity: `87ccd18` 600 passed, 3 failed (the known "small bars"), 30 skipped; `04063f4` had 599 passed, 4 failed (the same three plus the intermittent `animation.spec.ts:43` timing test). Raw data: `native-timing-runs-head.txt` (7 rounds, before DDD against `87ccd18`).
