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
    network/          WebSocket / OverlayPlugin connection to ACT
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
        │  WebSocket text  (or OverlayPluginApi callback, or DOM event)
        ▼
 network/            open socket, reconnect, keep-alive, hand each text to the parser
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
formatting   theme   network
combat       settings  translations
act_data
javascript_compat
```
`network` uses `act_data` and the `settings` browser-storage helpers; `formatting` uses `combat`,
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
| `incoming_message.rs` | Every wire shape: OverlayPlugin `CombatData` / `ChangePrimaryPlayer` (internally tagged enum on `type`), MiniParse `broadcast` (envelope with `msgtype` + free-form `msg`, read in a second serde step so unknown message types with any payload are ignored). Output is `ActEvent::{CombatData, LocalPlayerName}`. `Ok(None)` = valid but uninteresting, `Err` = malformed. `parse_bare_combat_data` handles the legacy DOM event. Includes tests with a real MiniParse capture. | `combat_data_message` |

Test fixture: `src/data/captures/mini_parse_beastmaster.json` (a real capture).

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
| `browser_storage.rs` | `localStorage` get/set, errors ignored. Also used by `network` for the saved address. |
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

## 9. `network/` – talking to ACT

| File | What it does |
|---|---|
| `mod.rs` | Public entry points: `start_listening(callbacks)` (legacy DOM event, discovered endpoint, else poll for the in-game API) and `connect_to_address(text, callbacks)` (start-screen box). |
| `act_endpoint.rs` | `ActEndpoint::{MiniParse, OverlayPlugin}`; discovery from `?HOST_PORT=`, `?OVERLAY_WS=` or the saved address; `from_user_text`; saved address helpers. Tested. |
| `websocket_connection.rs` | Opens the socket, sends the OverlayPlugin subscription or MiniParse `set_id`, answers keep-alive `.`, reconnects every 5 s, sends `overlayAPI` requests. Keeps the socket and endpoint in `thread_local!` statics (wasm is single-threaded). `handle_incoming_text` calls `act_data::parse_incoming_message` and dispatches or logs an error. |
| `overlay_plugin_bridge.rs` | Inside OverlayPlugin's browser: finds `OverlayPluginApi`, installs `__OverlayCallback`, polls up to 5 s for the API, finds the overlay window id (global or GUID in the user agent), `endEncounter`, and the legacy `onOverlayDataUpdate` DOM event. |
| `callbacks.rs` | `ActEventCallbacks`: three `Rc<dyn Fn>` (combat data, local name, status) that `app_state` provides; `dispatch(ActEvent)`. |
| `connection_status.rs` | `ConnectionStatus` enum. |
| `javascript_json.rs` | JS value -> JSON text; console warning for unparseable messages. |

---------------------------------------------------------------------------------------------------

## 10. `app_state/` – shared state and actions

| File | What it does |
|---|---|
| `mod.rs` | Docs and re-exports. |
| `app_context.rs` | `AppContext`: a `Copy` bundle of ~20 Dioxus signals/memos (settings, current screen, latest vs displayed combat data, rankings memo, sample rankings memo, local name, connection status, history, dropdown, toast, tooltip, standby, blurred rows...). Also `Screen`, `Dropdown`, `SettingsLocation`, `ToastState`; `language_code()` and `edit_settings()`. |
| `sample_fight.rs` | Parses `previewLog.json` once (settings previews, "Show sample data"). |
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
| root | `mod.rs`, `app_shell.rs`, `overlays.rs` | `App` creates the `AppContext`, starts the network once, saves settings after each change, injects the theme `<style>`, and picks the screen. `make_act_callbacks` connects `network` to `app_state`. `Tooltip` and `Toast` components. |
| `shared/` | `palette`, `row_identity`, `text_display`, `rankings_source`, `switch_and_icon`, `option_choice`, `safe_markup/` | Bar colours by palette mode; element ids of rows; fragments/job icons as DOM; live vs sample rankings; on/off switch and row icon; setting values as list keys; `safe_markup` renders the HTML fragments of the translation files (see below). |
| `dropdown_menus/` | `mod`, `menu_item`, `navigation_menu`, `choice_menus` | The open `Dropdown` variant becomes a list: the ⋮ menu, single choice, several toggles, column alignment. |
| `navigation_bar/` | `mod`, `summary_line`, `buttons` | Time, target, summary text, Capture/History/End/⋮ buttons; `capture_screenshot` is shared with the history screen. |
| `combat_tables/` | `mod`, `table_environment`, `visible_players`, `standard_table`, `graph_bars`, `raid_grid` | `CombatTables` chooses raid grid or normal tables in the configured order; job filters; header, rows, cells; coloured and small pet/overheal/shield bars; blur names by clicking the icon. |
| `start_screen/` | `mod`, `language_links`, `connect_box` | Notice before data arrives, language links, ACT address box and "Show sample data". |
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
2. `callbacks.dispatch` -> `on_combat_data` (from `app_shell::make_act_callbacks`) -> `app_state::handle_combat_data_received`.
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
| `public/` | `mopimopi.css` (original styles), `app.css` (fixes: `#main{width:100%;height:100%}` is essential, scrollbar hiding...), `images/`, `font/`. |
| `web/index.html` | Page shell; loads `pkg/mopimopi-dioxus.js` (hyphen, not underscore). |
| `build.sh` | wasm build -> `dist/` (`BUILD_STD=1` for toolchains without a prebuilt wasm std). |
| `.github/workflows/pages.yml` | GitHub Pages deploy. |
| `tools/smoke-test.mjs` | jsdom smoke test of the built wasm (needs `npm i jsdom`). |

---------------------------------------------------------------------------------------------------

## 14. Recipes

- **Support a new ACT field:** add it to `CombatantRecord` (with `rename` + `lenient_*`), then to `PlayerStats`/`Player` if it needs summing, then a match arm in `formatting/column_cell.rs`, and a column in `defaults.json`/`l.json`.
- **New pet:** add its names to `combat/pet_names.rs`.
- **New job icon:** add `<JOB>.png` to each set in `public/images/icon/` (currently missing: `BST`).
- **New setting:** add it to `defaults.json` and `l.json`; read it with `option_*`; style-related ones go into a `theme/` section.
- **New protocol message:** add a variant in `act_data/incoming_message.rs` and a test with a captured line.

## 15. Gotchas

- Dioxus mounts into `#main`; it must fill `#wrap` (see `app.css`) or percentage layouts collapse.
- Do not write signals while rendering; network startup is deferred with `spawn` for that reason.
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
Browser behaviour is checked with `tools/smoke-test.mjs`.
