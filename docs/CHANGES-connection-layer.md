> **Superseded.** The network module described here was later replaced by a single OverlayPlugin WebSocket client
> (`infrastructure/overlay_plugin_socket`); see `MAINTAINING.md` section 9 and section 18. This report is kept as history.

# Connection layer: detailed change report

Scope: everything done to the way the overlay gets its data from ACT, from the redesign based on the setup guides up to the
event channel. Earlier work (performance, memory, icons, deployment) was reported separately and is only listed at the end.

## 1. Summary

| Area | Before | After |
|---|---|---|
| Address handling | A typed `host:port` became `ws://host:port/ws` only | Empty box = `127.0.0.1:10501`; several candidate URLs (`wss://` and `ws://`) tried in turn |
| Reconnecting | Fixed 5 s retry, one URL | Rounds over all candidates; delays 1, 1, 2, 4, 8, then 15 s; a dropped connection is retried after 1 s |
| Status | Idle / Connecting / Connected / Disconnected, no details | `Connecting { attempt }`, `Disconnected { retry_in_seconds }`, connected URL shown, checklists, top-bar warning |
| Missing `YOU` row | Tables silently empty | A hint that says what to change in ACT |
| Legacy protocol | ACTWebSocket / MiniParse messages, handshake and keep-alive | Removed. OverlayPlugin's API only. `?HOST_PORT=` and saved `/MiniParse` addresses still work |
| In-game (OverlayPlugin browser) | Untested; status showed "Connected to ." and could be overwritten by "Not connected" | Two bugs fixed and covered by a test |
| Start-up call | `spawn`, then a direct call in `use_hook` | `use_effect` (the pattern the Dioxus docs give) |
| Network to app | `ActEventCallbacks`: three `Rc<dyn Fn>` built from `AppContext` | An `mpsc::UnboundedSender<NetworkEvent>`; one task in the app reads the receiver |
| Verification | Unit tests only | 73 unit tests, 8 browser scenarios (25 checks) against a real local WebSocket server, and a cell-by-cell comparison with the original overlay |

## 2. Why: what the guides said

- The original author's setup guide: OverlayPlugin's WSServer at `127.0.0.1:10501` with **Enable SSL** ticked and a generated
  certificate (so the server speaks `wss://`); `YOU` in ACT (Options, Miscellaneous); the FFXIV plugin's Disable Combine Pets
  with Owner ticked, Disable Damage Shield estimates unticked, Parse Filter Alliance recommended.
- OverlayPlugin's own source (`GetModernUrl`): its URL generator appends `OVERLAY_WS=ws[s]://<ip>:<port>/ws`, using `wss` when
  SSL is on. It also serves `/MiniParse` through a `LegacyHandler` for old overlays.
- OverlayPlugin's setup guide: overlays are added as **Custom**, type **MiniParse** (an overlay type, unrelated to the
  protocol), URL = the page; encounters should end via "End ACT encounter after wipe / out of combat".
- The old code turned every typed address into `ws://`. With SSL on and the page served over https, the documented setup could
  not connect from the connect box.

## 3. Changes in order

### 3.1 Connection redesign

`src/infrastructure/network/act_endpoint.rs` (rewritten)
- `ConnectionPlan { candidate_urls }`. `from_user_text(text, page_is_secure)` accepts: empty, `host`, `host:port`,
  `ws(s)://` and `http(s)://` URLs, with or without a path. Missing port becomes 10501, missing path `/ws`.
- An explicit scheme is respected (one candidate). Without a scheme both are tried, `wss` first when the page is https.
- `retry_delay_seconds(round)`: 1, 1, 2, 4, 8, 15, 15...
- `discover_plan`: `?OVERLAY_WS=`, then `?HOST_PORT=`, then the saved address (`Mopi2_ws`). `DEFAULT_ADDRESS` = `127.0.0.1:10501`.

`src/infrastructure/network/websocket_connection.rs` (rewritten as a state machine)
- One session in one thread-local slot. A round tries every candidate, starting with the one that worked last; a failed round
  waits `retry_delay_seconds`; a connection that was open and then closed is retried after 1 s.
- Every browser callback carries a generation number, so late events of a replaced session are ignored.
- Only the `close` listener drives retries (browsers fire `close` after `error`; handling both would retry twice).
- Closures of a finished socket are dropped from a zero-delay timer, never from inside themselves.

`connection_status.rs`: `Idle`, `Connecting { attempt }`, `Connected`, `Disconnected { retry_in_seconds }` (still `Copy`).

UI (`presentation/ui`)
- `start_screen/connect_box.rs`: prefilled with the saved or default address; a headline per state; the connected URL; the
  input and Connect button only while not connected.
- `start_screen/connection_help.rs` (new): checklists from the guides. Not connected: WSServer running, address and port match,
  Enable SSL / Generate SSL Certificate. Connected but silent: `YOU`, the FFXIV plugin options, Parse Filter.
- `navigation_bar/mod.rs`: while disconnected, the target text becomes "Disconnected from ACT, retrying in N s".
- `combat_tables/setup_hint.rs` (new): shown instead of the tables when data has no row named `YOU`.
- `../assets`: styles for the checklist.

### 3.2 Removal of the legacy MiniParse / ACTWebSocket protocol

- `models/act_data/incoming_message.rs`: only `CombatData` and `ChangePrimaryPlayer` are understood; the `{"type":"broadcast"}`
  envelope types are gone (such messages are ignored). One serde pass reads only `type`, the second reads the payload.
- Network: `Protocol` enum, the `set_id` handshake, the `.` keep-alive and the `/MiniParse` endpoint choice removed.
- Compatibility kept: `?HOST_PORT=ws://127.0.0.1:10501/` (same address as `OVERLAY_WS`), and any address ending in `/MiniParse`
  (case-insensitive, trailing slash allowed) is switched to `/ws`.
- The real Beastmaster capture fixture was converted from the MiniParse envelope to the OverlayPlugin shape
  (`src/data/captures/overlay_plugin_beastmaster.json`); the benchmark builds its message the same way.
- What "MiniParse" meant (source and docs): a `LegacyHandler` path in OverlayPlugin's WSServer for ACTWebSocket-era overlays,
  and separately the name of OverlayPlugin's generic overlay type.

### 3.3 Verification against the original overlay (`tools/original-comparison/`)

- `run-original.mjs <repo> messages`: runs the original's JavaScript in jsdom against a fake server and records what it sends:
  the `.` keep-alive answer and `{"type":"overlayAPI","to":<window id>,"msgtype":"Capture" | "RequestEnd"}`. Capture sends only
  that request, and the original's own tooltip labels it "for PC, ACTWebSocket"; End encounter also calls
  `OverlayPluginApi.endEncounter()`.
- `run-original.mjs <repo> table`, `run-port.mjs`, `compare-tables.mjs`: draw the same fights with both and compare every cell.
  Three fights (sample; the real Beastmaster capture; a generated 24-player raid with pets and a limit break, drawn as raid-mode
  cards): **no differences**. A negative control (different fights) is reported as different, and an empty result counts as
  a failure. The original treats zone `HAERU` as its own preview data, so the scripts rename the zone.

### 3.4 Bugs found by the tests

| Found by | Bug | Fix |
|---|---|---|
| Unit test | A legacy path with a trailing slash (`/MiniParse/`) was not recognised | Trim trailing `/` before comparing |
| Browser scenario 8 | In the in-game path the start screen read "Connected to ." (no URL exists there) | Falls back to "OverlayPlugin" |
| Browser scenario 8 | `start_listening` reported `Idle` after starting the API poll, overwriting the immediate `Connected` | Report `Idle` first, then poll |
| Browser harness (not the app) | jsdom events were not instances of Node's global `MessageEvent`; two app instances in one process mixed timers | Harness overrides the classes the app checks and runs one scenario per process |

### 3.5 Start-up call: `spawn`, direct, `use_effect`

1. You asked whether `spawn` was needed. A direct call passed all scenarios with no console warnings, so I removed it.
2. The Dioxus 0.7 "Intro to Reactivity" page says data must not be modified while rendering (it queues re-renders and can loop),
   and the hooks documentation says effects run after the component has rendered. My "harmless because `App` doesn't read that
   signal" reasoning only covered the loop case. It also contradicted my earlier statement that the docs had no such rule.
3. The call is now in `use_effect` (it reads no signals, so it runs once). All 8 scenarios and the unit tests pass.

### 3.6 Event channel instead of callbacks

- New `infrastructure/network/events.rs` (replaces `callbacks.rs`): `NetworkEvent::{Status, CombatData, LocalPlayerName}`,
  `ActEventSender` (clone-able wrapper around `futures_channel::mpsc::UnboundedSender`), `ActEventReceiver`, `event_channel()`,
  and `receive_text` (serde parsing moved here so it is unit-testable). A send to a closed channel is ignored.
- New `application/app_state/network_events.rs`: `apply_network_event(context, event)` writes the status and player name and
  calls `handle_combat_data_received`.
- `presentation/ui/app_shell.rs`: creates the channel in `use_hook`, puts the sender in the context (`use_context_provider`),
  reads the receiver in one `use_future` task, and starts the network in the effect. `make_act_callbacks` is deleted.
- `start_screen/connect_box.rs` gets the sender with `use_context::<ActEventSender>()`.
- Dependencies added: `futures-channel` and `futures-util` (both `default-features = false, features = ["std"]`; already in the
  build through Dioxus).
- Effect: the network layer no longer holds anything built from the application. Events sent before the task starts reading
  are kept. Each event is applied one executor tick later than the old direct call.

## 4. Files

| File | Change |
|---|---|
| `infrastructure/network/act_endpoint.rs` | rewritten (plan, candidates, backoff, discovery) |
| `infrastructure/network/websocket_connection.rs` | rewritten (state machine, generation guard) |
| `infrastructure/network/connection_status.rs` | new variants with data |
| `infrastructure/network/events.rs` | new (replaces `callbacks.rs`) |
| `infrastructure/network/overlay_plugin_bridge.rs` | uses the sender; unchanged logic |
| `infrastructure/network/mod.rs` | entry points take the sender; page-security check; status order fix |
| `models/act_data/incoming_message.rs` | legacy shapes removed |
| `application/app_state/network_events.rs` | new |
| `presentation/ui/app_shell.rs` | channel, task, effect |
| `presentation/ui/start_screen/{connect_box,connection_help}.rs` | new status UI and checklists |
| `presentation/ui/combat_tables/setup_hint.rs` | new |
| `presentation/ui/navigation_bar/mod.rs` | disconnected warning |
| `tests/unit/infrastructure/network/{act_endpoint,events}.rs` | rewritten / new |
| `tools/websocket-e2e-test.mjs` | new (8 scenarios) |
| `tools/original-comparison/*` | new (4 scripts and a README) |
| `MAINTAINING.md`, `docs/README-setup-section.md` | updated / new |
| `Cargo.toml` | `futures-channel`, `futures-util` |

## 5. What users will notice

- The connect box works with the documented setup (SSL on, https page) and starts from the default address.
- Failures say what to check; a lost connection is visible in the top bar and recovers by itself.
- Missing `YOU` is explained instead of showing nothing.
- Inside OverlayPlugin the start screen says "Connected to OverlayPlugin".
- The Capture button probably no longer does anything (an ACTWebSocket feature in the original).
- Anyone still on the real ACTWebSocket plugin can no longer use the overlay.

## 6. Verification

| Check | Result |
|---|---|
| `cargo test` | 73 passed (6 of them new for the channel, 9 or so for the endpoint rules) |
| Browser scenarios (`tools/websocket-e2e-test.mjs`) | 8 of 8, 25 checks: plain server; https page with `wss` then `ws` fallback; server down then up (backoff 1, then 2 s); connection lost mid-session; missing `YOU`; `?HOST_PORT=` URL; saved `/MiniParse` address; in-game `OverlayPluginApi` path |
| Original comparison | identical on 3 fights (sample, real capture, 24-player raid) |
| Smoke test (`tools/smoke-test.mjs`) | identical to the previous build |
| Size | wasm 1,429,485 bytes |

Run them: `cargo test`; `cd tools && npm i jsdom ws && node websocket-e2e-test.mjs`;
`cd tools/original-comparison && npm i jsdom ws jquery && node compare-tables.mjs <original repo>`.

## 7. Limits and open items

- Everything ran in jsdom against a fake server, not in Chrome and not against a real ACT with OverlayPlugin. `wss://` was only
  tested as a failing handshake against a plain server. That an SSL-enabled WSServer speaks `wss` only is taken from the guides
  and OverlayPlugin's source.
- Capture and End encounter over a WebSocket: whether OverlayPlugin's `/ws` honours the old `overlayAPI` request is unverified.
  The requests are still sent. Decision pending: hide the Capture button.
- Only one source should be active at a time (WebSocket, `OverlayPluginApi`, legacy DOM event); this is not enforced or tested.
- The new messages are English only (no translations).
- The plugin option names in the original guide (pets, shield estimates) are not verified against the current FFXIV plugin.

## 8. Earlier changes in the same period (reported before)

Performance: faster serde parsing and rankings, `Rc` rankings, memoised theme, delayed settings save, single standby timer.
Memory: `tracing` capped at WARN in release (removed a `PerformanceMark` leak). Layout: stylesheets moved into `<head>` through
`Dioxus.toml` (removed a 0.95 layout shift). Deployment: `dx` GitHub Pages workflow with tests as a gate, `base_path` from the
repository name. Icons: Beastmaster icon in all 12 sets.
