# Presentation map

UML views of the 74 files under `src/presentation`, drawn from their real `use` statements, context calls and components. Diagrams are Mermaid, so GitHub renders them inline (also in comments).

## 1. Package dependencies

What each folder under `ui/` reaches outside presentation. Arrows point from the user to the used package. Only folders that have an outside dependency are connected.

```mermaid
flowchart LR
  subgraph PRES["presentation::ui"]
    app_shell
    navigation_bar
    combat_tables
    history_screen
    start_screen
    dropdown_menus
    settings_screens
    areas
    shared
  end
  subgraph APP["application"]
    app_state
    i18n
  end
  subgraph DOM["domain"]
    d_settings["settings"]
    formatting
    combat
    translations
  end
  subgraph INF["infrastructure"]
    act
    browser_websocket
  end
  common["common::javascript_compat"]
  app_shell --> app_state
  app_shell --> d_settings
  app_shell --> act
  app_shell --> browser_websocket
  navigation_bar --> app_state
  navigation_bar --> i18n
  navigation_bar --> translations
  navigation_bar --> formatting
  navigation_bar --> d_settings
  navigation_bar --> act
  navigation_bar --> common
  combat_tables --> app_state
  combat_tables --> i18n
  combat_tables --> formatting
  combat_tables --> combat
  combat_tables --> d_settings
  combat_tables --> act
  history_screen --> app_state
  history_screen --> formatting
  history_screen --> i18n
  start_screen --> app_state
  start_screen --> i18n
  dropdown_menus --> app_state
  dropdown_menus --> i18n
  dropdown_menus --> translations
  settings_screens --> app_state
  settings_screens --> i18n
  settings_screens --> translations
  settings_screens --> d_settings
  settings_screens --> common
  areas --> d_settings
  areas --> act
  shared --> combat
  shared --> formatting
  shared --> act
```

## 2. Component render tree

`App` picks the top bar and the body from `ScreenContext.current_screen` (Main, History or Settings). The same two components, `CombatTables` and `NavigationBar`, are reused by the main screen and by the settings live preview.

```mermaid
flowchart TD
  App --> MainScreen
  App --> HistoryNavigationBar
  App --> HistoryScreen
  App --> SettingsNavigationBar
  App --> SettingsScreen
  App --> NavigationBar
  App --> DropdownMenu
  App --> Tooltip
  App --> Toast
  App --> JsColorPicker
  MainScreen --> StartScreen
  MainScreen --> CombatTables
  StartScreen --> LanguageLinks
  NavigationBar --> NavigationButtons
  NavigationButtons --> NavigationButton
  SettingsScreen --> LivePreview
  SettingsScreen -->|"calls"| schema_rows_fns["schema_rows::*()"]
  SettingsScreen -->|"calls"| column_pages_fns["column_pages::*()"]
  LivePreview -->|"is_settings_preview=true"| CombatTables
  LivePreview -->|"is_settings_preview=true"| NavigationBar
  DropdownMenu --> ToggleOptions
  DropdownMenu --> ChooseOption
  DropdownMenu --> ChooseColumnAlignment
  CombatTables --> standard_table["standard_table()"]
  CombatTables --> raid_grid["raid_grid()"]
  standard_table --> graph_bars["graph_bars()"]
  raid_grid --> graph_bars
```

Lower-case names are plain functions that return `Element`, not components. `PageContent` is an enum that picks which settings page function to call. `history_screen` imports only `capture_screenshot` from `navigation_bar`.

## 3. Shared state and who reads it

`App` creates and provides these values with `use_context_provider`: one context per section of the page, `AppActions`, the settings view, the connection, the colour picker and the bar history. Everything below reads them with `use_context`.

```mermaid
classDiagram
  direction LR
  class App
  class SettingsContext {
    +Signal~Settings~ settings
    +edit_settings(change)
  }
  class ScreenContext
  class TablesContext {
    +displayed_combat_data
    +local_player_name
    +has_received_data
    +blurred_player_rows
    +is_standby_hidden
  }
  class HistoryContext
  class SettingsScreenContext
  class NavigationBarContext
  class DropdownContext
  class NoticesContext
  class AppActions {
    <<actions, private fields>>
  }
  class SettingsView {
    +Memo~NavSettings~ nav
    +Memo~TableSettings~ table
    +Memo~ColumnSettings~ columns
    +Memo~BarSettings~ bars
    +Memo~RaidSettings~ raid
    +Memo~PageSettings~ page
  }
  class OverlayPluginContext {
    +connection_status()
    +get_is_encounter_active()
    +combat_data_message()
  }
  class ColorPickerState
  class BarHistory
  App ..> SettingsContext : provides
  App ..> ScreenContext : provides
  App ..> TablesContext : provides
  App ..> HistoryContext : provides
  App ..> SettingsScreenContext : provides
  App ..> NavigationBarContext : provides
  App ..> DropdownContext : provides
  App ..> NoticesContext : provides
  App ..> AppActions : provides
  App ..> SettingsView : provides via provide_settings_view
  App ..> OverlayPluginContext : provides
  App ..> ColorPickerState : provides
  App ..> BarHistory : provides
  CombatTables ..> SettingsContext
  CombatTables ..> TablesContext
  CombatTables ..> SettingsScreenContext
  CombatTables ..> SettingsView
  CombatTables ..> OverlayPluginContext
  NavigationBar ..> SettingsContext
  NavigationBar ..> TablesContext
  NavigationBar ..> SettingsView
  NavigationButtons ..> NavigationBarContext
  NavigationButtons ..> NoticesContext
  NavigationButtons ..> DropdownContext
  NavigationButtons ..> AppActions
  NavigationButtons ..> OverlayPluginContext
  NavigationButtons ..> SettingsView
  HistoryScreen ..> HistoryContext
  HistoryScreen ..> SettingsContext
  HistoryScreen ..> AppActions
  HistoryScreen ..> SettingsView
  StartScreen ..> TablesContext
  DropdownMenu ..> DropdownContext
  DropdownMenu ..> SettingsContext
  DropdownMenu ..> AppActions
  Overlays ..> NoticesContext
  Overlays ..> AppActions
  SettingsScreen ..> SettingsContext
  SettingsScreen ..> SettingsScreenContext
  SettingsScreen ..> DropdownContext
  SettingsScreen ..> AppActions
  LivePreview ..> SettingsScreenContext
  JsColorInput ..> SettingsContext
  JsColorInput ..> ColorPickerState
  graph_bars ..> BarHistory
```

`graph_bars` reaches `BarHistory` through a consume call, not a hook, because it runs once per row.

## 4. Settings to style pipeline

The refactor's core path: the raw settings file becomes six typed area structs, each of which turns itself into inline CSS variables through the shared style types.

```mermaid
classDiagram
  direction LR
  class SettingsFile
  class SettingsView {
    +provide_settings_view(settings)
  }
  class NavSettings {
    +from_raw(raw)
    +bar_vars()
    +time_vars()
    +target_vars()
    +summary_vars()
  }
  class TableSettings {
    +from_raw(raw)
    +raid_mode(n)
    +tables_in_order()
    +header_vars()
    +body_vars()
  }
  class ColumnSettings {
    +from_raw(raw)
    +of(is_healing)
  }
  class BarSettings {
    +from_raw(raw)
    +side(is_healing)
  }
  class BarPalette {
    +color_of(key, role, row)
    +participant_color(player, row)
  }
  class RaidSettings {
    +from_raw(raw)
    +grid_vars()
  }
  class PageSettings {
    +from_raw(raw)
    +root_rule()
  }
  class StyleReads {
    <<trait>>
  }
  class Vars {
    +set(name, value)
    +finish()
  }
  class Values {
    Size
    Opacity
    Paint
    FontStack
    Shadow
  }
  class Shapes {
    TextStyle
    Line
    Corners
  }
  SettingsFile --> SettingsView : read by 6 memos
  SettingsView *-- NavSettings
  SettingsView *-- TableSettings
  SettingsView *-- ColumnSettings
  SettingsView *-- BarSettings
  SettingsView *-- RaidSettings
  SettingsView *-- PageSettings
  BarSettings *-- BarPalette
  NavSettings ..> StyleReads
  TableSettings ..> StyleReads
  BarSettings ..> StyleReads
  RaidSettings ..> StyleReads
  PageSettings ..> StyleReads
  ColumnSettings ..> StyleReads
  StyleReads ..> Values
  StyleReads ..> Shapes
  Shapes ..> Vars : write
  Values ..> Vars
  NavSettings ..> Vars
  TableSettings ..> Vars
  RaidSettings ..> Vars
```

## 5. combat_tables, file by file

How `CombatTables` builds a table. `TableEnvironment` bundles the area structs and the cell context so the drawing functions take one argument.

```mermaid
flowchart LR
  mod["combat_tables/mod.rs<br/>CombatTables"] --> env["table_environment.rs<br/>TableEnvironment"]
  mod --> opc["ui/overlay_plugin_context.rs"]
  mod -->|"context"| views["areas::SettingsView"]
  mod -->|"mode: standard"| std["standard_table.rs"]
  mod -->|"mode: raid"| raid["raid_grid.rs"]
  env --> areas["areas/mod.rs<br/>Table, Columns, Bars, Raid"]
  std --> env
  std --> vis["visible_participants.rs"]
  std --> bars["graph_bars.rs<br/>BarGeometry, Flip, BarHistory"]
  std --> cls["cell_classes.rs"]
  std --> td["shared/text_display.rs"]
  std --> ri["shared/row_identity.rs"]
  raid --> env
  raid --> vis
  raid --> td
  raid --> ri
  vis --> areas
  bars --> env
  bars --> areas
  bars --> pal["shared/palette.rs"]
  hist["history_screen/*"] --> cls
```

## 6. settings_screens, file by file

Pages are driven by data (`data/l.json`), so `schema_rows` has one file per row kind. Almost everything depends on `row_context` and `row_layout`.

```mermaid
flowchart TD
  subgraph SR["schema_rows"]
    srmod["mod.rs"]
    choice["choice_rows"]
    link["link_and_action_rows"]
    text["text_rows"]
    value["value_rows"]
  end
  subgraph CP["column_pages"]
    cpmod["mod.rs"]
    titles["column_titles"]
    order["column_order_page"]
    size["column_size_page"]
    align["column_alignment_page"]
    header["header_text_page"]
    abbr["abbreviation_list"]
  end
  rowgroups["row_groups"] --> pagecontent["page_content"]
  rowgroups --> srmod
  rowgroups --> rowctx["row_context"]
  tabbar["tab_bar"] --> pagecontent
  tabbar --> rowctx
  srmod --> pagecontent
  srmod --> rowctx
  choice --> srmod
  choice --> rowlayout["row_layout"]
  choice --> rowctx
  link --> srmod
  link --> rowlayout
  text --> srmod
  text --> formactions["form_actions"]
  text --> textbox["text_box"]
  value --> srmod
  value --> jscolor["js_color"]
  value --> slider["slider_row"]
  value --> rowlayout
  slider --> rowlayout
  formactions --> b64["base64_encoding"]
  formactions --> textbox
  textbox --> rowctx
  align --> titles
  size --> titles
  size --> slider
  header --> titles
  header --> formactions
  header --> textbox
  header --> rowlayout
  abbr --> rowlayout
  cpmod --> rowctx
  rowlayout --> shared["shared: safe_markup, switch_and_icon"]
  livepreview["live_preview"] --> tables["combat_tables"]
  livepreview --> navbar["navigation_bar"]
```

## 7. shared

Leaf helpers. `style` is used only by `areas`; `safe_markup` turns the translated HTML snippets into Dioxus elements.

```mermaid
flowchart LR
  areas["areas/*"] --> stylepkg
  subgraph stylepkg["shared/style"]
    reads["reads (StyleReads)"] --> shapes
    reads --> values
    shapes --> values
  end
  values --> cc["color_conversion"]
  subgraph sm["shared/safe_markup"]
    parser --> attr["attribute_rules"]
    parser --> ent["entities"]
    parser --> tree["markup_tree"]
    attr --> tree
    render --> tree
  end
  si["switch_and_icon"] --> sm
  menu["dropdown_menus/menu_item"] --> si
  menu --> sm
  rowlayout["settings_screens/row_layout"] --> si
  rowlayout --> sm
  overlays --> sm
  startscreen["start_screen"] --> sm
  tables["combat_tables"] --> td["text_display"]
  tables --> ri["row_identity"]
  tables --> pal["palette"]
  choicemenus["dropdown_menus/choice_menus"] --> oc["option_choice"]
```

## 8. A data update, end to end

ACT sends a message; the table redraws. Only the memos and signals that changed cause work.

```mermaid
sequenceDiagram
  participant ACT as OverlayPlugin (ws)
  participant Conn as overlay_plugin_context
  participant App as app_shell (App)
  participant State as application::app_state
  participant CT as CombatTables
  participant Env as TableEnvironment
  participant Tbl as standard_table / raid_grid
  participant Bars as graph_bars
  ACT->>Conn: CombatData JSON (about 1 per second)
  Conn->>App: combat_data_message signal changes
  App->>State: use_effect calls on_combat_data_changed(context, message)
  State->>State: update displayed_combat_data, history, standby timer
  State-->>CT: displayed_combat_data signal changes
  CT->>CT: read SettingsView memos (table, columns, bars, raid)
  CT->>Env: TableEnvironment::new(...)
  CT->>Tbl: raid_mode ? raid_grid : standard_table
  Tbl->>Tbl: visible_participants, cell_fragments (domain::formatting)
  Tbl->>Bars: graph_bars(geometry, BarHistory)
  Bars-->>Tbl: bar element, flip animation a/b
  Tbl-->>CT: rows as Element
```

## 9. Editing a setting

A slider or colour change on a settings page. Because each area is its own memo, only the areas whose values changed re-render their consumers.

```mermaid
sequenceDiagram
  participant Row as schema_rows (value_rows)
  participant Ctx as SettingsContext
  participant Sig as Signal of Settings
  participant Save as settings_saving
  participant View as SettingsView memos
  participant UI as CombatTables / NavigationBar
  Row->>Ctx: edit_settings(change)
  Ctx->>Sig: write
  Sig->>Save: schedule_settings_save
  Save-->>Save: 300 ms debounce, then write to browser storage
  Sig-->>View: all six memos re-run from_raw
  View-->>UI: only memos whose value changed notify
  UI->>UI: new inline CSS variables, CSS reacts
```

## Findings

- **No cycles between folders.** The dependency direction is `app_shell` to screens to `combat_tables`/`navigation_bar` to `areas` to `shared/style`.
- **Cross-screen reuse is small and one-way.** `settings_screens` uses `combat_tables` and `navigation_bar`; `history_screen` uses `navigation_bar` and `combat_tables::cell_classes`; `start_screen` uses `combat_tables`. Nothing imports a screen back.
- **`areas` and `shared/style` are pure.** They depend only on `domain::settings` (and one `infrastructure::act` type), never on Dioxus components or the application contexts.
- **Heaviest fan-in:** `row_context` (16 settings files), `safe_markup` (9 files) and `areas/mod.rs` (7 files).
- **Method.** Edges come from `use` statements resolved to files. Paths written inline, such as `use_context::<areas::SettingsView>()`, are added by hand from a grep of the context calls.
