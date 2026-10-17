# 0008 Rows are shared, messages are read once, numbers are not routed through text
Context: the performance report (`MEASUREMENTS.md`, fourth pass) showed the reworked port costing 2.8% more instructions per message than the port before it, and showed where a message goes (76% reading the JSON, 19% drawing the cells).

Decision:
* The Encounter keeps its facts and participants behind `Rc` and `rows(PetView)` returns handles. This is immutable sharing with copy-on-write (`Rc::make_mut`, and only for an owner that takes in a pet): to every reader a row is still a plain value that never changes under it. Pets are still folded per call, so the "Combine Pets with Owner" option applies at once, as in the original (`js/process.js`, `update`). Equality of the `Rc` props is by value, so a row that did not change is still not drawn again.
* `OverlayMessage` is read once by a visitor that sees `"type"` first and reads straight into the data; a message with `"type"` elsewhere takes a second, slower route with the same result and the same errors.
* `round_to_two_decimals` rounds arithmetically when the value is clear of a tie and below 1e7 and takes the exact text route otherwise; `format_number` writes into one string. Same text out, tests hold both to the old route.

DDD principle: a performance change must not change the model's language or the behaviour the original has (conformist). The Encounter stays an aggregate with the same invariants (`indices_to_demote` is the pure rule check, `new` stays the strict door for tests, `check_rules` is kept as a `debug_assert`); the sharing is hidden behind `rows`/`shared_facts`. Talk rules: 1 (calculate vs act: `indices_to_demote` and `grouped_by_three` return values), 9/10 (values; sharing is immutable).
Status: accepted.
