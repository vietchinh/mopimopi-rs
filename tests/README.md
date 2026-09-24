# Tests

All unit tests live here, outside `src/`. Each file mirrors the source file it tests
(`src/models/act_data/incoming_message.rs` -> `tests/unit/models/act_data/incoming_message.rs`).

They are still compiled as a child module of the file they test, through a one-line declaration in
that source file:

```rust
#[cfg(test)]
#[path = "../../../tests/unit/models/act_data/incoming_message.rs"]
mod tests;
```

This keeps access to private items (`classify`, the serde visitors, ...) without making them public.
A classic `tests/*.rs` integration test would need a library crate and a public API for all of them.

* `cargo test` runs everything.
* `cargo test --release --offline benchmarks -- --ignored --nocapture` runs the timing benchmarks
  (`tests/unit/benchmarks.rs`).
* Test data: `src/data/previewLog.json` (sample fight) and `src/data/captures/` (real ACT captures).
