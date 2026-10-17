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

| History size | Time to record (incl. building each Encounter) | History screen redraw (`entries().to_vec()`) | Member data kept alive (structs only) |
|---:|---:|---:|---:|
| 10 fights | 6 ms | 0.2 µs | ~105 KiB |
| 100 fights | 62 ms | 3.8 µs | ~1 MiB |
| 1000 fights | 617 ms (all parse) | 42 µs | ~10 MiB |

## Conclusions

* The DDD design (Encounter per message, views computed on demand) is not a performance risk. No memo or cache was added for
  the folded view; add one only if a profile in the browser ever shows `rows` on the hot path.
* The history is **unbounded**, as before. A raid night (about 100 fights) is about 1 MiB of member data plus their strings,
  which is fine. Thousands of fights in one session would grow without limit. A cap is a one-line rule in
  `History::record` if the raider wants one (listed in `TO-CONFIRM.md`, item 9); it was not added because the original
  has none and the port must match it.
* Not measured here: wasm timings in the browser. The existing Playwright `performance.spec.ts` (skipped in the parity
  run) is the place for those.
