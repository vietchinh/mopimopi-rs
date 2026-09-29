# MopiMopi (Rust / Dioxus) – code guide

A Rust + Dioxus 0.7 port of the MopiMopi FFXIV ACT combat overlay. It compiles to WebAssembly and runs
as a static site (GitHub Pages). This guide describes every Rust file: what it does and how it connects
to the others. **111 files, about 6,500 lines**, one small responsibility per file.

---------------------------------------------------------------------------------------------------

## 0. Layers (folder structure)

Code is grouped into layers. Each layer only imports from layers listed **below** it.

```
src/
  presentation/     everything the user sees
    ui/               Dioxus components (screens, tables, menus, settings pages)
    theme/            settings -> one CSS stylesheet
  application/      application state and actions (Dioxus signals live here)
    app_state/
  infrastructure/   the outside world
    overlay_plugin_socket/   OverlayPlugin's WebSocket: the only source of combat data
  domain/           business domain: rules + the objects they work on (no Dioxus)
    combat/           players, roles, pets, rankings, pet merging
    formatting/       how values become table text
    settings/         the user's settings and their rules
    translations/     translated texts, settings-page schema
  models/           pure data objects, no business rules
    act_data/         ACT records, the combat message, wire message shapes (serde)
  common/           helpers shared by all layers
    javascript_compat.rs
tests/unit/         unit tests, mirroring this structure (see tests/README.md)
```

Where the older sections below say `combat/`, `ui/` etc., read them as `domain/combat/`,
`presentation/ui/` and so on. Rust paths are `crate::domain::combat`, `crate::presentation::ui`, ...

Known impurity: `domain/settings` still contains `browser_storage.rs`, `language_detection.rs` and
`persistence.rs`, which use browser APIs (`localStorage`, `navigator`). They belong in
`infrastructure/` in a stricter split; they stay because moving them needs small visibility changes.

## 1. The big picture

```
 ACT / OverlayPlugin
        │  WebSocket text (the address comes from the page URL: ?OVERLAY_WS= or ?HOST_PORT=)
        ▼
 overlay_plugin_socket/   created once in main; connects, subscribes, reconnects; sends data down three channels
        │  &str
        ▼
 act_data/           serde: text -> typed ActEvent (CombatDataMessage | LocalPlayerName)
        │  ActEvent
        ▼
 app_state/          handle_combat_data_received: update signals, history, standby timer
        │  Signal<Rc<CombatDataMessage>>  ("displayed_combat_data")
        ▼
 combat/             Memo: CombatDataMessage -> EncounterRankings (players, pets merged, sorted)
        │  EncounterRankings
        ▼
 formatting/  theme/ ui/     cells' text  |  one CSS stylesheet  |  Dioxus components
```

Settings (`settings/`) and translations (`translations/`) sit beside this flow and are read by
`formatting`, `theme`, `app_state` and `ui`. Nothing in `act_data`, `combat`, `settings`,
`translations` or `theme` knows about Dioxus components; only `app_state` (signals) and `ui` do.

### Allowed dependency directions (each line may use the ones below it)

```
ui
app_state                 (Dioxus signals)
formatting   theme   overlay_plugin_socket
combat       settings  translations
act_data
javascript_compat
```
`overlay_plugin_socket` uses `act_data` only; `formatting` uses `combat`,
`settings` and `translations`; `theme` uses only `settings`.

---------------------------------------------------------------------------------------------------

## 2. Top-level files

| File | What it does |
|---|---|
| `src/main.rs` | Declares the modules and calls `dioxus::launch(ui::App)`. Its doc comment states the data flow. |
| `src/javascript_compat.rs` | Reproduces the JavaScript number behaviour the original relied on: `round_to_two_decimals` (`toFixed(2)` then parse), `parse_leading_float` (`parseFloat`), `number_to_javascript_string` (`Number.toString`, integers without `.0`), `to_fixed`. Used by `act_data`, `combat` and `formatting` so numbers display exactly as in the original. |

---------------------------------------------------------------------------------------------------

## 3. `act_data/` – parsing ACT with serde

ACT is inconsistent: numbers arrive as text (`"6,703.94"`, `"12%"`), `"---"` means "no value", booleans
arrive as `"true"`. All of that is absorbed here, so the rest of the program sees clean `f64`,
`String` and `bool`.

| File | What it does | Interacts with |
|---|---|---|
| `mod.rs` | Module docs and re-exports (`CombatDataMessage`, `CombatantRecord`, `EncounterRecord`, `ActEvent`, `parse_incoming_message`, `parse_bare_combat_data`, `describes_a_name_not_a_number`). | – |
| `lenient_values.rs` | Custom serde deserializers used with `#[serde(deserialize_with)]`: `lenient_number` (JSON numbers, formatted text, placeholders -> `f64`, unknown -> 0), `lenient_text`, `lenient_bool`. Internally an untagged `AnyScalar` enum accepts any scalar so one odd field never fails a whole message. Also `parse_formatted_number` and `describes_a_name_not_a_number` (text with anything besides digits `.` `,` `%` is a name such as `"Broil-8,765"`, not a number). | `javascript_compat` |
| `encounter_record.rs` | `EncounterRecord` – the `Encounter` object (title, duration text/seconds, total damage/healed, encounter DPS/HPS, zone). Verbose Rust names mapped to ACT keys with `rename`. | `lenient_values` |
| `combatant_record.rs` | `CombatantRecord` – one player/pet/NPC entry: name, `Job`, own duration, ~20 counters, last-10/30/60/180 s DPS, strongest hit/heal (text + amount), parry/block, deaths. | `lenient_values` |
| `combat_data_message.rs` | `CombatDataMessage { encounter, combatants, is_encounter_active }`. `Combatant` is a JSON object keyed by name; a custom visitor turns it into a `Vec` **in arrival order** (ties in ranking depend on it). `Encounter` is required, so non-combat objects fail to parse. | the two record files |
| `incoming_message.rs` | The WebSocket message shapes of OverlayPlugin: `CombatData` and `ChangePrimaryPlayer`; everything else (including the legacy MiniParse `broadcast` envelope) is ignored. Two passes: a first serde pass reads only the `type`, the second reads the payload straight into its final type. `parse_bare_combat_data` handles the legacy DOM event. Includes tests with a real capture. |

Test fixture: `src/data/captures/overlay_plugin_beastmaster.json` (a real capture).

---------------------------------------------------------------------------------------------------

## 4. `combat/` – the domain model

Turns records into players and rankings. Pure logic, no UI, fully unit-tested.

| File | What it does |
|---|---|
| `mod.rs` | Docs, re-exports, `LOCAL_PLAYER_ROW_NAME = "YOU"` (ACT names the local player's row `YOU`). |
| `player_stats.rs` | `PlayerStats`: additive counters. `from_record` builds it (also computes `effective_healed = healed - over_heal - damage_shield`); `add` folds a pet in. |
| `strongest_action.rs` | `StrongestAction { action_name, amount }`: `"Broil-8,765"` -> `Broil`, 8765; empty/numeric text -> "No Data". `beats` is used when merging pets. |
| `player_role.rs` | `PlayerRole` enum (Tank, Healer, Damage, Crafter, Gatherer, OwnedCombatant) and `palette_key()` (the string keys the colour palette uses: `Tanker`, `DPS`, `CBO`...). |
| `pet_names.rs` | `PET_OWNER_JOBS`: pet names in KR/JP/CN/DE/FR/EN per owning job (SMN, SCH, MCH, DRK, NIN, AST, WHM, SGE), with the role forced on the pet. `find_pet_owner_job(name)`. |
| `job_classification.rs` | `classify(name, job_text)` -> job code, class code (icon), role, `is_pet`, owner name. Handles base classes (GLA->PLD), crafters/gatherers, Limit Break (`LMB`), pets by name (`AVA`), owned unknown combatants (`CBO`). Constants `PET_JOB_CODE`, `LIMIT_BREAK_JOB_CODE`, `COMBATANT_JOB_CODE`. |
| `derived_rates.rs` | `DerivedRates::calculate`: DPS/HPS (own and encounter), damage/heal share, crit/direct-hit/accuracy percentages. NaN -> 0, infinity kept (shown as `∞`), rounded to 2 decimals. |
| `player.rs` | `Player`: identity, role, `own_stats` vs `merged_stats`, own vs merged strongest hit/heal, rates, rank, visibility. `from_record` combines `classify`, `PlayerStats`, `StrongestAction`, `DerivedRates`. `merge_pet_stats` and `reset_to_own_values` support pet merging. |
| `encounter_ranking.rs` | `EncounterRanking`: players sorted by damage or healing (stable, ties keep ACT's order), top value for bar widths, party size, encounter key (title + totals, used to dedupe history). Finds the local owner name (the pet owner that matches no real player = your character name). |
| `pet_merging.rs` | `impl EncounterRanking`: `merge_pet_into_its_owners` (also into the `YOU` row when the owner is the local character), `attach_pets` / `detach_pets` (setting "combine pets with owner"), `apply_pet_setting_and_sort`. |
| `rankings.rs` | `build_rankings(message, merge_pets, local_name) -> EncounterRankings { by_damage, by_healing }`, `TableKind` (Damage/Healing, `short_label()` = `DPS`/`HPS`) and the rankings tests (pet merge numbers). |

Flow inside `build_rankings`: records -> `Player::from_record` (classify, stats, rates) -> sort ->
merge pets into owners -> re-sort -> ranks and top value.

---------------------------------------------------------------------------------------------------

## 5. `settings/` – user settings

Settings keep the **same JSON shape as the original overlay** (`q`, `Color`, `Range`, `Alias`, `Order`,
`ColData` in `localStorage["Mopi2_HAERU"]`), so backups and shared codes import cleanly. They are read
by string key because the settings pages are generated from `data/l.json`, which names settings by key.

| File | What it does |
|---|---|
| `mod.rs` | Docs and re-exports (`Settings`, `is_truthy`, storage keys, `read/write_local_storage`). |
| `user_settings.rs` | `Settings { json_document }` and the section-name constants. |
| `default_settings.rs` | Loads `data/defaults.json`. |
| `option_access.rs` | Readers: `option_value/enabled/number/text`, `language_code`, `slider_value`, `color_hex`, `action_abbreviations`. |
| `option_updates.rs` | Writers: `set_option`, `set_option_enabled`, `set_option_from_text` (keeps number vs text type), `set_slider_value`, `set_color_hex`, abbreviation add/remove, `column_number`. |
| `column_layout.rs` | Columns: definitions, per-table enable flag, order, `set_column_enabled`, `move_column`. |
| `persistence.rs` | `defaults`, `load_from_browser` (falls back to defaults + detected language), `from_json_text`, `save_to_browser`, and `normalize_document` which fills options added by newer versions and migrates old data. Storage key constants. |
| `browser_storage.rs` | `localStorage` get/set, errors ignored. |
| `language_detection.rs` | Browser language -> KR/JP/CN/DE/FR/EN. |
| `shareable_code.rs` | "Custom UI Data" export (skips personal/technical options) and import. |
| `json_coercion.rs` | JavaScript truthiness and number coercion for the mixed-type values the original stored. |

---------------------------------------------------------------------------------------------------

## 6. `translations/`

| File | What it does |
|---|---|
| `mod.rs` | Loads `data/l.json` (`ui_schema`: every settings page, its rows and texts) and `data/d.json` (`dictionary`: column hints, alignment names...) once. `translate(value, language)` picks a language, falling back to English. `message()` and `dictionary_title()` helpers. |

---------------------------------------------------------------------------------------------------

## 7. `formatting/` – table cell text

| File | What it does |
|---|---|
| `mod.rs` | Docs and re-exports. |
| `text_fragment.rs` | `TextFragment::{Plain, Dimmed}` (units and action names are dimmed) and `join_plain_text`. |
| `number_format.rs` | `NumberFormat::from_settings`: digit grouping, decimal mark, decimals per kind (rates, percents, amounts), k/M shortening switches. Produces fragments for rates, amounts, MaxHit amounts, percents. |
| `player_name.rs` | `NameOptions` and `display_name`: name abbreviation modes, "keep YOU label", hiding names, `Pet (Owner)` handling, translated companion / Limit Break labels, rank prefix. |
| `strongest_action_text.rs` | The MaxHit/MaxHeal cell: optional abbreviation from the user's list and one of four layouts. |
| `column_cell.rs` | `CellContext` (bundles settings, translations, local name, and the format objects) and `cell_fragments(column, player, ranking, context)`, the single dispatch for every column. `cell_plain_text` is used by the summary line. |

---------------------------------------------------------------------------------------------------

## 8. `theme/` – one stylesheet from the settings

The original re-applied ~300 jQuery `.css()` calls after every render; here the same rules are generated
as one `<style>` string whenever settings change.

| File | What it does |
|---|---|
| `mod.rs` | `build_theme_css` (calls each section in order) and `table_body_height_rem`. Has the stylesheet test. |
| `style_inputs.rs` | `StyleInputs`: slider -> rem/opacity, colours, italic/bold, font stacks, corner-radius rules, text outline. |
| `color_conversion.rs` | Hex -> `rgba(...)` (3-digit colours keep the original's unscaled behaviour). |
| `page_background.rs` | Root font size, background image, accent colour rules. |
| `navigation_bar.rs` | Top bar: pattern background, border, radius, icons, time/target/summary text, 1-line vs 2-line layout. |
| `table_corners.rs` | Rounded corners of header/body and graph bars. |
| `table_text.rs` | Body text of other rows vs your own (`#YOU`, `.myPet`). |
| `raid_grid.rs` | Raid-mode card styles. |
| `table_chrome.rs` | Header, dividing lines, icon size, graph bar heights and float position. |
| `column_layout.rs` | Per-column width, padding, alignment. |

---------------------------------------------------------------------------------------------------

## 9. `overlay_plugin_socket/` – OverlayPlugin's WebSocket

The only source of combat data. One socket exists per page. `main` creates it (`start_from_page_url()`); it reads its address
from the page's own URL, connects at once, subscribes, and reconnects on its own. Everything it receives goes down channels
that the application reads later.

```
page URL  ?OVERLAY_WS=ws://127.0.0.1:10501/ws      (?HOST_PORT= works too; /ws is added when the path lacks it)
   │  start_from_page_url()  (main, once)
   ▼
OverlayPluginSocket ── connects, subscribes, reconnects, reports its own status
   ├─ combat_data        UnboundedReceiver<CombatDataMessage>
   ├─ local_player_name  UnboundedReceiver<String>
   └─ connection_status  UnboundedReceiver<ConnectionStatus>
```

| File | What it does |
|---|---|
| `mod.rs` | `start_from_page_url()` (once; later calls do nothing), `take_streams()` (the receiving ends, handed out once), `server_url()`, `request_end_encounter()` (async; `Ok` = the request reached the server, which does not answer it). Holds the socket in a `thread_local` so it lives as long as the page. |
| `server_url.rs` | Pure address rules. Reads `OVERLAY_WS` (first) or `HOST_PORT` from a query string, decodes `%3A%2F%2F` escapes, and turns the value into a WebSocket URL that ends with `/ws` (added unless already there; a trailing `/` is ignored; no scheme = `ws://`; `http(s)://` becomes `ws(s)://`). Empty or unusable = no address. `legacy_endpoint_url` gives the `/MiniParse` URL of the same server. Tested. |
| `retry_delay.rs` | 1, 1, 2, 4, 8, then 15 seconds between failed attempts; 1 second after an open connection dropped. Tested. |
| `connection_status.rs` | `NotConfigured`, `Connecting { attempt }`, `Connected`, `Disconnected { retry_in_seconds }`. |
| `streams.rs` | `OverlayPluginStreams` (receivers) and the private senders. `receive_text` parses a message with serde and sends it down the matching channel; unknown messages are ignored, malformed ones go to the console. Messages sent before the app reads are kept. Tested without a browser. |
| `legacy_command.rs` | The end-encounter request. OverlayPlugin's `/ws` has no such call, but its legacy `/MiniParse` endpoint (`LegacyHandler` in `WSServer.cs`) handles `{"type":..., "msgtype":"RequestEnd"}` by ending the ACT encounter. So this opens a short-lived connection to `/MiniParse` on the same server, sends the request and closes it (gives up after 5 s). |
| `socket.rs` | The WebSocket itself: opens the connection, subscribes to `CombatData` and `ChangePrimaryPlayer`, and retries on its own. Only the `close` listener drives retries (browsers fire `close` after `error`). Closures of a finished socket are dropped from a zero-delay timer, never from inside themselves. |

The application side is `application/app_state/overlay_plugin_events.rs`: `use_overlay_plugin_events(context)` (called once in
`App`) starts one task per channel and applies each value to the state.

What the user sees: with no address in the URL the start screen says how to add `?OVERLAY_WS=...` and lists the WSServer
steps (`start_screen/connection_panel.rs`, `connection_help.rs`); while connecting or disconnected it says to which URL and
when the next attempt is; once data is on screen a lost connection shows in the top bar; data without a row named `YOU` shows a
hint (`combat_tables/setup_hint.rs`).

`tools/chrome-buttons-test.mjs` tests the Capture and End-encounter buttons in real headless Chrome (`npm i puppeteer-core @sparticuz/chromium`).
`tools/websocket-e2e-test.mjs` runs the built app in jsdom against a real local WebSocket server that behaves like the WSServer
(`?OVERLAY_WS=`, `?HOST_PORT=` without `/ws`, an escaped address, no address, server down then up, connection lost mid-session,
missing `YOU`, an empty parameter).

## 10. `app_state/` – shared state and actions

| File | What it does |
|---|---|
| `mod.rs` | Docs and re-exports. |
| `app_context.rs` | `AppContext`: a `Copy` bundle of ~20 Dioxus signals/memos (settings, current screen, latest vs displayed combat data, rankings memo, sample rankings memo, local name, connection status, history, dropdown, toast, tooltip, standby, blurred rows...). Also `Screen`, `Dropdown`, `SettingsLocation`, `ToastState`; `language_code()` and `edit_settings()`. |
| `sample_fight.rs` | Parses `previewLog.json` once (settings previews, "Show sample data"). |
| `overlay_plugin_events.rs` | `use_overlay_plugin_events`: one task per channel of the socket; applies status, combat data and player name to the state. |
| `data_ingestion.rs` | `handle_combat_data_received`: always store as latest; display while a fight runs and once when it ends (then also record history); leave the display alone while the settings screen is open. |
| `encounter_history.rs` | `HistoryEntry`, open/close the history screen, show an entry. |
| `screen_navigation.rs` | Settings page/tab navigation, back behaviour, which pages have previews/tabs. |
| `toast_notifications.rs` | Timed messages; a generation counter makes old timers harmless. |
| `standby_mode.rs` | Hides tables after N minutes without a fight, restarted on any activity. |
| `settings_maintenance.rs` | Reset (confirm), backup, restore, fullscreen. |

---------------------------------------------------------------------------------------------------

## 11. `ui/` – Dioxus components

| Area | Files | What they do |
|---|---|---|
| root | `mod.rs`, `app_shell.rs`, `overlays.rs` | `App` creates the `AppContext`, starts the network once, saves settings after each change, injects the theme `<style>`, and picks the screen. `Tooltip` and `Toast` components. |
| `shared/` | `palette`, `row_identity`, `text_display`, `rankings_source`, `switch_and_icon`, `option_choice`, `safe_markup/` | Bar colours by palette mode; element ids of rows; fragments/job icons as DOM; live vs sample rankings; on/off switch and row icon; setting values as list keys; `safe_markup` renders the HTML fragments of the translation files (see below). |
| `dropdown_menus/` | `mod`, `menu_item`, `navigation_menu`, `choice_menus` | The open `Dropdown` variant becomes a list: the ⋮ menu, single choice, several toggles, column alignment. |
| `navigation_bar/` | `mod`, `summary_line`, `buttons` | Time, target, summary text, the buttons (Capture, History, End encounter, ⋮) and `screenshot.rs` + `page_screenshot.js` (Capture). |
| `combat_tables/` | `mod`, `table_environment`, `visible_players`, `standard_table`, `graph_bars`, `raid_grid` | `CombatTables` chooses raid grid or normal tables in the configured order; job filters; header, rows, cells; coloured and small pet/overheal/shield bars; blur names by clicking the icon. |
| `start_screen/` | `mod`, `language_links`, `connection_panel`, `connection_help` | Notice before data arrives, language links, connection status with checklists, and "Show sample data". |
| `history_screen/` | `mod`, `history_row` | Finished-encounter list. |
| `settings_screens/` | see below | All settings pages. |

### `shared/safe_markup/` – no `dangerous_inner_html`
The translation files contain small HTML fragments (`<b>`, `<br>`, `<font class="ex">`, `<a>`, `<img>`).
They are never given to the browser as HTML. `markup_view(text)` parses them into a tree
(`parser.rs`, `markup_tree.rs`), keeps only allowlisted tags and attributes (`attribute_rules.rs`:
classes, a text colour, `#`/http(s) links, images below `images/`; event handlers, scripts and
`javascript:` links are dropped), and renders real Dioxus elements (`render.rs`). `entities.rs`
decodes `&amp;`-style references. Use `markup_view` for any text from `l.json` / `d.json`.

### `settings_screens/`
Pages come from `l.json`; these files decide what to show and how to render each entry.

| File | What it does |
|---|---|
| `mod.rs` | `SettingsScreen`: preview, tab bar, body according to `PageContent`. |
| `navigation_bar.rs` | Top bar and page titles. |
| `page_content.rs` | `SchemaEntry`, `PageContent`, tab logic, `content_for(settings, location)`. |
| `live_preview.rs`, `tab_bar.rs`, `row_groups.rs` | Sample bar + tables with raid switch; tab buttons; rows grouped into boxes. |
| `row_context.rs`, `row_layout.rs`, `slider_row.rs`, `text_box.rs` | Shared context passed to row builders; standard row layout; slider; text box state. |
| `form_actions.rs`, `base64_encoding.rs` | Submit text / add abbreviation / set background image; tiny base64 encoder. |
| `schema_rows/` | `render_schema_row` picks the builder by entry type: `link_and_action_rows`, `choice_rows`, `value_rows`, `text_rows`. |
| `column_pages/` | Pages built from current settings: `column_order_page`, `column_size_page`, `column_alignment_page`, `header_text_page`, `abbreviation_list`, `column_titles`. |

---------------------------------------------------------------------------------------------------

## 12. How the parts interact – key scenarios

**A combat data message arrives**
1. `websocket_connection` receives text -> `act_data::parse_incoming_message` -> `ActEvent::CombatData`.
2. The socket sends the parsed message down the `combat_data` channel; the task started by `use_overlay_plugin_events` reads it and `handle_combat_data_received` runs.
3. It stores the message in `latest_combat_data`, and (if allowed) in `displayed_combat_data`.
4. The `rankings` memo in `app_shell` recomputes `combat::build_rankings`.
5. `CombatTables` and `NavigationBar` read the memo, call `formatting::cell_fragments` per cell, and render. The `<style>` from `theme::build_theme_css` styles it.

**The user changes a setting**
A row calls `context.edit_settings(...)` -> the `settings` signal changes -> `use_effect` saves to
`localStorage` -> the theme stylesheet is rebuilt, and the rankings memo is recomputed if the pet
setting changed. No manual redraw code exists.

**Pet merging** – `classify` marks known pets `AVA` and gives them an owner from `"Pet (Owner)"`;
`EncounterRanking::sort_and_rank` finds the local owner name; `merge_pet_into_its_owners` adds the pet's
`own_stats` to the owner's (and to `YOU` when it is you); pets get `is_visible = false`. Tables filter
them out through `visible_players`.

**Language** – `settings.language_code()` -> `translations::translate` for schema text; settings
values themselves stay untranslated.

---------------------------------------------------------------------------------------------------

## 13. Non-Rust files that matter

| Path | Purpose |
|---|---|
| `src/data/l.json`, `d.json`, `defaults.json` | Generated from the original `lang.js`/`dic.js`/`init.js` by `tools/extract.js`. `l.json` drives every settings page. |
| `src/data/previewLog.json` | Sample fight (also used by tests). |
| `src/data/captures/` | Real ACT captures used as test fixtures. |
| `assets/` | The generated `tailwind.css`, `font/`. |
| `public/base.css` | The **base layout**, a static render-blocking `<link>` (`style` in `Dioxus.toml`, `<link>` in `web/index.html`): the layout glue (`@layer app`, what `assets/app.css` was) and the first part of the original overlay's stylesheet through the tables (`@layer legacy`). It has to be in place when the app first paints, or every refresh bounces. |
| `tailwind.css` (project root) | Tailwind input, starting with `@import "tailwindcss"` as in the Dioxus guide. Cascade layers, low to high: `theme`, `base` (Preflight, undone by a generated block), `app` and `legacy` (both start in `public/base.css`; `legacy` continues in `tailwind.css`), `components`, `utilities` (the `@utility` definitions, then the rules after them). Classes are found by Tailwind reading `src/**/*.rs` (`@source`); `@theme static` holds the palette tokens. |
| `tools/cancel-preflight.mjs` | Prints the block in `tailwind.css` that undoes Preflight (`revert-layer` for every property Preflight sets); `--check` says whether the copy is current. Regenerate after upgrading Tailwind. |
| `tools/check-tailwind.mjs` | `npm run check:css` (also run by `build.sh` and the Playwright suite): fails, naming the class, when a stray word in the Rust sources makes Tailwind generate a utility. |
| `public/` | `images/` (served as they are, relative to the page). |
| `web/index.html` | Page shell; loads `pkg/mopimopi-dioxus.js` (hyphen, not underscore). |
| `build.sh` | wasm build -> `dist/` (`BUILD_STD=1` for toolchains without a prebuilt wasm std). |
| `.github/workflows/pages.yml` | GitHub Pages deploy. |
| `tools/smoke-test.mjs` | jsdom smoke test of the built wasm (needs `npm i jsdom`). |

---------------------------------------------------------------------------------------------------

## 14. Recipes

- **Support a new ACT field:** add it to `CombatantRecord` (with `rename` + `lenient_*`), then to `PlayerStats`/`Player` if it needs summing, then a match arm in `formatting/column_cell.rs`, and a column in `defaults.json`/`l.json`.
- **New pet:** add its names to `combat/pet_names.rs`.
- **New job icon:** add `<JOB>.png` to each set in `public/images/icon/` (a test fails if a set is missing one). `tools/make_bst_icons.py --source <icon.png>` draws one job's icon in every set's style (framed game icon of any square size, or a bare glyph); it made the Beastmaster icons and can be reused for other new jobs.
- **New Tailwind class:** write it in full in an `rsx!` class string (`class: "nav-icon"`); Tailwind finds it by reading the source, and `dx serve` rebuilds the CSS on save. A class of our own is an `@utility` in `tailwind.css`. Never build a class name with `format!` (there is nothing in the source for Tailwind to find); values that depend on a setting are an inline `style` or an inline CSS variable. If `npm run check:css` names a class you did not mean, add it to the `@source not inline(...)` list in `tailwind.css`.
- **New setting:** add it to `defaults.json` and `l.json`; read it with `option_*`; style-related ones go into a `theme/` section.
- **New protocol message:** add a variant in `act_data/incoming_message.rs` and a test with a captured line.

## 15. Gotchas

- Tailwind reads Rust as plain text, so words in code and comments are candidate classes. Most produce nothing; the ones that are Tailwind utilities (`table`, `filter`, `static`, `fixed`, ...) would add rules, and `flex`, `hidden` and `shadow` are also *legacy classes the markup uses*, so Tailwind's version would change how the overlay looks (`.shadow` replaces the three-layer Material shadow). They are excluded with `@source not inline(...)` in `tailwind.css`; `npm run check:css` keeps that list honest. **Cascade layers:** across layers the layer decides, not the specificity, and un-layered rules beat all of them. That is why the layout glue (`@layer app`) and the legacy stylesheet (`@layer legacy`) are layers below `utilities`, with the order declared at the top of `public/base.css` and `tailwind.css`; a rule you add to either now loses to a utility even if its selector is more specific. Anything left un-layered (the rules after the utilities, the critical-CSS reset in `<head>`) beats every layer. Cascade layers need Chrome 99+ (`revert-layer` too), and a browser without them drops the whole legacy block. After changing how the CSS is assembled, run `playwright-tests/tests/cascade-diff.spec.ts` against the old build: it compares every property of every element and reported 0 differences for this restructuring (Preflight left on: differences on every element; utilities below legacy: 116 of 467).
- **Refresh bounce.** `tailwind.css` is added by the app at runtime (`document::Stylesheet`, see `app_shell.rs`), so it arrives after the wasm has run and drawn its first frame. Whatever that frame needs to be laid out correctly must therefore already be on the page: that is `public/base.css`, a plain `<link>` in the head (`style` in `Dioxus.toml`, `web/index.html` for `build.sh`). Measured on a refresh at 2560px wide, without it: first frame with `html` at 16px and the body 377px tall, ~50 ms later a layout shift of ~0.9 when the sheets land, then `html{transition:.3s}` slides the font-size 16px -> 10px for another 300 ms. Rules that set the size or position of something on the start screen or the tables belong in `base.css`; the rest in `tailwind.css`. `legacy` must stay ordered: what is in `base.css` is a *prefix* of the original stylesheet, and rules in one layer keep their order across sheets. **Do not put a path in `[web.resource] style` that is not in `public/`**: `dx` 0.7.10 writes the string into a `<link href>` as it is, without the base path and without copying the file, so the page has no styling at all (checked). Use a relative path (`base.css`).
- `tracing` is capped at WARN in release builds (`release_max_level_warn` in `Cargo.toml`). Without it Dioxus's signal and memo
  spans become `PerformanceMark`/`PerformanceMeasure` entries that the browser never frees (about 1 MB per hour of combat).
- Dioxus mounts into `#main`; it must fill `#wrap` (see `public/base.css`) or percentage layouts collapse.
- Do not modify signals while rendering (Dioxus, "Intro to Reactivity": it queues re-renders and can loop). Network start-up
  therefore runs in a `use_effect`, which runs after the first render; it reads no signals, so it runs once.
- The WebSocket `error` event is intentionally not handled: `close` always follows and would recurse.
- Ties in rankings depend on ACT's combatant order, so the parser preserves arrival order.
- Settings stay JSON-shaped on purpose (original compatibility, schema-driven pages).

## 16. Tests

All tests live in `tests/unit/`, mirroring `src/` (see `tests/README.md`). Each source file that has
tests contains only a declaration such as `#[cfg(test)] #[path = "../../tests/unit/..."] mod tests;`,
so the tests keep access to private items. `cargo test` runs 58 tests: serde leniency and message
shapes, a real capture, classification, pet merging on the sample fight, number/name formatting,
endpoint parsing, GUID finding, the safe-markup allowlist, settings round-trips, theme output. `tests/unit/benchmarks.rs` holds
ignored timing benchmarks (`cargo test --release --offline benchmarks -- --ignored --nocapture`).
Parity with the original overlay is checked by `tools/original-comparison/compare-tables.mjs` (three fights, every table cell; see its README).
Browser behaviour is checked with `tools/smoke-test.mjs`, and the connection layer with `tools/websocket-e2e-test.mjs`
(both need `npm i jsdom ws`).

## 17. CI, deploying to GitHub Pages, requiring green tests

Repository: `vietchinh/<repository>`, branch `dioxus`, site `https://vietchinh.github.io/<repository>/` (the site
currently lives under `mopimopi-rs`).

* `.github/workflows/ci.yml` – job **Tests**: `cargo test --locked` and `cargo check --target wasm32-unknown-unknown`.
  Runs on every pull request into `dioxus`, and is reused by the deploy workflow.
* `.github/workflows/pages.yml` - on every push to `dioxus`: runs the tests first (`needs: test`), then builds
  with the Dioxus CLI and publishes with the Pages actions (no build output is committed):
  1. reads the `dioxus` version from `Cargo.lock` and downloads the prebuilt `dx` of the same version on every
     run (`dx-x86_64-unknown-linux-gnu.zip` from the GitHub release; nothing is compiled, nothing of `dx` is
     cached; `dx` fetches wasm-bindgen, esbuild and wasm-opt itself). Only the Rust build is cached (rust-cache),
  2. sets `base_path` in `Dioxus.toml` to the repository name with `tools/set_base_path.py` (only in CI, so `dx serve`
     still serves at the root; works whether or not the file already has a `base_path` line),
  3. `dx bundle --platform web --release --out-dir pages` - the finished site is `pages/public`,
  4. uploads `pages/public` and deploys it. (The deploy guide's "move `public/*` up" and `404.html` steps are
     for publishing from a `docs/` folder and for client-side routing; neither applies here.)

One-time setup:
1. Settings -> Pages -> Source: **GitHub Actions**.
2. Settings -> Environments -> `github-pages` -> Deployment branches: allow `dioxus` (by default only the
   default branch may deploy; skip this if `dioxus` is the default branch).
3. **Requiring green tests:** the workflow file cannot forbid merging, that is a repository setting. Run
   `tools/protect-branch.sh` (needs the GitHub CLI and admin rights) or set it by hand: Settings -> Branches ->
   add a rule for `dioxus` -> "Require a pull request before merging" and "Require status checks to pass" ->
   select **Tests** (the check appears in the list after the workflow has run once) -> "Require branches to
   be up to date". Also tick "Do not allow bypassing the above settings" to bind admins.

`build.sh` (plain cargo + wasm-bindgen) still works for local builds; `dx` uses its own HTML template, not
`web/index.html`.

## 18. Only OverlayPlugin's WebSocket is supported

Removed: the legacy ACTWebSocket / MiniParse *data* protocol (the `{"type":"broadcast"}` messages, the `set_id` handshake, the
`.` keep-alive), OverlayPlugin's in-game `OverlayPluginApi` and the legacy DOM event, the address box and the saved address,
and the `wss`/`ws` candidate fallback.

Consequence: an overlay added inside OverlayPlugin needs the address in its URL too, for example
`https://vietchinh.github.io/mopimopi-rs/?OVERLAY_WS=ws://127.0.0.1:10501/ws` (the WSServer tab's URL generator produces it).
Old `?HOST_PORT=ws://127.0.0.1:10501` links keep working.

### The two buttons that used the old protocol

Both original buttons sent an `overlayAPI` request to the ACTWebSocket-style endpoint. What OverlayPlugin's `WSServer.cs`
does with it:

| Request | OverlayPlugin (`LegacyHandler`, path `/MiniParse`) | What the port does |
|---|---|---|
| `RequestEnd` | `ActGlobals.oFormActMain.EndCombat(true)`, so it works | End encounter opens a short-lived connection to `/MiniParse` on the same server and sends `{"type":"overlayAPI","msgtype":"RequestEnd"}` (`legacy_command.rs`). A toast reports success or failure. |
| `Capture` | only logs "ACTWS Capture is not supported outside of overlays" | Capture is done in the browser: the page draws itself to a PNG and downloads it (`navigation_bar/screenshot.rs`, `page_screenshot.js`). |

`/ws` (`SocketHandler`) only understands `{"call": ...}` messages and ignores everything else; no end-encounter call is
documented for it (the docs list `getLanguage` and say others, such as `getCombatants`, `saveData`, `say`, exist).

How the screenshot works: the overlay element is cloned with every computed style copied into it, the buttons, menus, tooltip
and toast are left out, images and the page's own `@font-face` fonts are embedded as data URLs, and the clone is wrapped in an
SVG `<foreignObject>`, drawn onto a canvas and downloaded. The image is cropped below the last row. Fonts from other sites (the
Google fonts) are not embedded, so the screenshot uses the next font in the stack for them. It downloads through the browser,
which may not be possible inside OverlayPlugin's own overlay window (untested).

What "MiniParse" meant (from OverlayPlugin's source and docs): OverlayPlugin's WSServer serves `/ws` (the current API) and
also `/MiniParse` and `/BeforeLogLineRead` through a `LegacyHandler`, kept for overlays written for the old ACTWebSocket plugin
(archived in 2019). Separately, "MiniParse" is the name of OverlayPlugin's generic overlay *type* (the "Type" dropdown), which
has nothing to do with the WebSocket protocol.
