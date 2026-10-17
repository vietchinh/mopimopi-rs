import json,re,collections,os,sys
F=json.load(open('facts.json'))
OUT=sys.argv[1]
COMMIT=sys.argv[2]
NOISE={('presentation/mod.rs','default'),('presentation/mod.rs','new'),('infrastructure/act/data/combat_data_peek.rs','.contains()'),('infrastructure/act/data/combat_data_peek.rs','.ends_with()')}
def nice(p): return p.replace('presentation/ui/','').replace('.rs','')
def pkg_of(rel):
    if rel=='presentation/mod.rs': return '(root)'
    parts=rel.replace('presentation/ui/','').split('/')
    if len(parts)>2 and parts[0] in('settings_screens','shared'): return parts[0]+'/'+parts[1]
    return parts[0] if len(parts)>1 else '(root)'
def label(rel):
    n=nice(rel)
    return n
def layer(rel): return rel.split('/')[0]
def ident(rel): return re.sub(r'\W','_',nice(rel))
def first_sentence(t,n=230):
    t=re.sub(r'\s+',' ',t).strip()
    m=re.match(r'(.+?[.!?])(\s|$)',t)
    s=m.group(1) if m else t
    return (s[:n-1]+'…') if len(s)>n else s
def module_summary(rel):
    d=F[rel]['doc'] if rel in F else ''
    return first_sentence(d.split('\n\n')[0]) if d else ''
# overrides for items without docs
OVR={
 ('presentation/ui/app_shell.rs','App'):'Root component. Builds the shared state and every context, starts the connection to ACT, wires the effects (combat data → display, settings → save, language) and draws the page shell around the current screen.',
 ('presentation/ui/app_shell.rs','clock_font_face'):'`@font-face` rule of the clock font, with asset URLs that are right wherever the app is hosted.',
 ('presentation/ui/combat_tables/graph_bars.rs','graph_bars'):'Draws the main bar and the small pet / overheal / shield bars of one row, each with the move it should play (see `animation_of`).',
 ('presentation/ui/combat_tables/raid_grid.rs','raid_grid'):'One grid of cards for a table: the visible players, each as a `raid_card`.',
 ('presentation/ui/combat_tables/raid_grid.rs','raid_card'):'One card: colour strip, job icon, name and the DPS or HPS figure.',
 ('presentation/ui/combat_tables/standard_table.rs','standard_table'):'One whole table: the header and a body with a row per visible player. Sets every shared CSS variable once on the wrappers.',
 ('presentation/ui/combat_tables/standard_table.rs','player_row'):'One row: job icon, one cell per column, and the bars behind it.',
 ('presentation/ui/combat_tables/standard_table.rs','cell'):'One body cell: the column\'s value for the player, as text fragments.',
 ('presentation/ui/combat_tables/standard_table.rs','header_table'):'The header: one cell per shown column, with the column\'s (translated or custom) title and tooltip.',
 ('presentation/ui/combat_tables/visible_participants.rs','visible_participants'):'Combatants a table shows, in the order it draws them, after the job filter.',
 ('presentation/ui/navigation_bar/buttons.rs','NavigationButtons'):'The buttons at the right of the top bar: the pinned ones, and the ⋮ menu button that reveals the rest.',
 ('presentation/ui/navigation_bar/buttons.rs','NavigationButton'):'One round icon button with its tooltip, flash animation and click action.',
 ('presentation/ui/navigation_bar/buttons.rs','press_button'):'What a button does: Capture, open History, request the end of the encounter, or open the ⋮ menu.',
 ('presentation/ui/navigation_bar/mod.rs','NavigationBar'):'The top bar: encounter time, target name, summary line and buttons. Also drawn as the sample bar of the settings preview.',
 ('presentation/ui/overlay_plugin_context.rs','OverlayPluginContext'):'The signals of the connection (status, last combat data, local player, error), provided as context.',
 ('presentation/ui/overlay_plugin_context.rs','keep_connected'):'The connection loop: connect, subscribe, feed messages to `handle_message`, and reconnect with a growing delay when the socket closes.',
 ('presentation/ui/overlay_plugin_context.rs','handle_message'):'Decides what one raw ACT message means: skips idle heartbeats (never the first message), parses the rest, and sets the combat data or the local player.',
 ('presentation/ui/overlay_plugin_context.rs','parse_overlay_plugin_url'):'The ACT address from the page URL (`?OVERLAY_WS=` / `?HOST_PORT=`), validated.',
 ('presentation/ui/overlay_plugin_context.rs','overlay_ws_from_page'):'Reads the query string of the page.',
 ('presentation/ui/settings_screens/column_pages/abbreviation_list.rs','*'):'The saved action abbreviations, each with a remove button.',
 ('presentation/ui/settings_screens/column_pages/column_order_page.rs','*'):'Up / down buttons that reorder the columns of a table.',
 ('presentation/ui/shared/rankings_source.rs','*'):'Which rankings a table draws (live, or the sample in the settings preview).',
 ('presentation/ui/history_screen/mod.rs','HistoryScreen'):'The History screen: the list of finished encounters with the same header as the tables.',
 ('presentation/ui/history_screen/mod.rs','HistoryNavigationBar'):'Top bar of the History screen: title, capture and a close button.',
 ('presentation/ui/start_screen/mod.rs','StartScreen'):'The notice shown before any data arrives: introduction, language links, tips and update notes.',
 ('presentation/ui/start_screen/mod.rs','MainScreen'):'The start screen until the first message, then the tables (hidden while standby is on).',
 ('presentation/ui/settings_screens/mod.rs','SettingsScreen'):'The settings screen: live preview, tab buttons and the scrollable rows of the current page.',
 ('presentation/ui/shared/switch_and_icon.rs','SwitchToggle'):'The sliding on/off switch drawn at the right of a row or menu item.',
 ('presentation/ui/dropdown_menus/mod.rs','DropdownMenu'):'The pop-up menu currently open (⋮ menu, or a choice list), chosen from `DropdownContext::open_dropdown`.',
 ('presentation/ui/combat_tables/mod.rs','CombatTables'):'Entry of the tables: reads the displayed combat data and the settings, then draws the damage / healing tables, standard or raid grid, in the configured order.',
}
def describe(rel,it):
    return OVR.get((rel,it['name'])) or first_sentence(it['doc']) or ''
def produces(it):
    r=it['ret']
    if it['kind']=='fn':
        if r=='Element' : return 'DOM (Dioxus `Element`)'
        if r in('String',): return '`String`'+(' (CSS)' if re.search('style|css|vars|rule|class',it['name']) else '')
        if r: return '`'+r.replace('|','\\|')+'`'
        return 'side effect (signals / DOM events)' if it['level']=='top' else '—'
    return {'struct':'value type','enum':'value type','const':'constant','trait':'trait','type':'type alias'}.get(it['kind'],'')
def calls_of(rel,it,full=False):
    c=[]
    for tgt,syms in it['calls'].items():
        syms=[s for s in syms if (tgt,s) not in NOISE]
        if not syms or tgt==rel: continue
        c.append((tgt,syms))
    for kind,ty,f in it.get('contexts',[]):
        c.append((f,['context '+ty+(' (provides)' if kind=='use_context_provider' else '')]))
    for fld in it.get('area_reads',[]):
        c.append(('presentation/ui/areas/'+fld+'.rs',['SettingsView.'+fld]))
    return c
def fmt_calls(c,limit=7):
    parts=[]
    for tgt,syms in c[:limit]:
        parts.append('`'+nice(tgt)+'`'+(': '+', '.join('`'+s+'`' if not s.startswith('context') else s for s in syms[:3]) if syms else ''))
    s='; '.join(parts)
    if len(c)>limit: s+=f'; … (+{len(c)-limit})'
    return s.replace('|','\\|')
# ---------- aggregate file edges
file_edges=collections.defaultdict(lambda: collections.defaultdict(set))   # src->tgt->symbols
renders=collections.defaultdict(lambda: collections.defaultdict(set))
for rel,d in F.items():
    for it in d['items']:
        if it['kind']=='mod' : continue
        for tgt,syms in calls_of(rel,it):
            for s in syms: file_edges[rel][tgt].add(s)
        for tgt,syms in it['renders'].items():
            for s in syms: renders[rel][tgt].add(s)
pk=collections.OrderedDict()
for rel in sorted(F): pk.setdefault(pkg_of(rel),[]).append(rel)
pk=collections.OrderedDict(sorted(pk.items(),key=lambda kv:(kv[0]!='(root)',kv[0])))
PKDESC={
 '(root)':'The root component and the one connection to ACT.',
 'areas':'Typed settings, one struct per area of the UI (`SettingsView`). The only code that knows the settings keys of the presentation, and the names of the CSS variables each area writes.',
 'combat_tables':'The DPS / HPS tables, the raid grid and the graph bars behind the rows.',
 'dropdown_menus':'Pop-up menus: the ⋮ menu and the pick-one / toggle lists.',
 'history_screen':'The list of finished encounters.',
 'navigation_bar':'The top bar of the main screen and its buttons.',
 'overlays':'Floating messages: the hover tooltip and the sliding toast.',
 'settings_screens':'The settings screens: pages from `l.json`, the colour picker, forms, and the special column pages.',
 'shared':'Small pieces used by several screens: typed styling values, safe markup, palette, ids, icons.',
 'start_screen':'What is shown before the first message from ACT.',
 'settings_screens/schema_rows':'One builder per kind of entry in `l.json`: pick-one, switch, link, action, colour, slider, text, file.',
 'settings_screens/column_pages':'Settings pages built from the current settings instead of from `l.json`: column order, width, alignment, header text, abbreviations.',
 'shared/style':'Typed styling values (`Size`, `Opacity`, `Paint`, `FontStack`, `Shadow`, `Vars`), shapes (`TextStyle`, `Line`, `Corners`) and the typed readers on `SettingsFile`.',
 'shared/safe_markup':'Turns the small HTML fragments of the translation files into Dioxus elements without ever giving the text to the browser as HTML.',
}
L=[]
w=L.append
w('# Presentation layer — detailed UML report\n')
w(f'*Generated from the source at commit `{COMMIT}` by `docs/uml/tools/extract.py` and `generate.py`. Every table and edge below comes from the code (doc comments, signatures, `use` paths, call sites); the descriptions are the code\'s own doc comments. Regenerate: `REPO=. python3 docs/uml/tools/extract.py && python3 docs/uml/tools/generate.py docs/uml/presentation-detailed.md $(git rev-parse --short HEAD)` (run from a scratch folder; `extract.py` writes `facts.json` in the current directory).*\n')
w('The presentation layer is `src/presentation/ui` — **%d files, %d items**. It draws the screens, reads the state that `application` owns, and never talks to ACT or the browser except through `application` and `infrastructure`.\n'%(len(F),sum(len(d['items']) for d in F.values())))
w('## Contents\n')
w('1. [How to read the diagrams](#1-how-to-read-the-diagrams)\n2. [The layers](#2-the-layers)\n3. [Runtime flows](#3-runtime-flows) — start-up, data to screen, one table, settings edit, standby\n4. [Shared state and contexts](#4-shared-state-and-contexts)\n5. [Component render tree](#5-component-render-tree)\n6. [Package by package](#6-package-by-package) — each with a diagram and a table of what every file does, produces and calls\n')
w('## 1. How to read the diagrams\n')
w('| Notation | Meaning |\n|---|---|\n| class box | one source file (`foo/bar.rs`); its members are its public functions, components (`«C»`) and types |\n| `A ..> B : x, y` | code in A calls / uses `x`, `y` defined in B |\n| `A *--> B : renders X` | the component A draws component X (defined in B) in its `rsx!` |\n| `<<application>>`, `<<domain>>`, `<<infrastructure>>` | a file outside the presentation layer |\n| **Produces** (tables) | what the item returns: `DOM` is a Dioxus `Element`; CSS variables and class lists are `String`s; *side effect* means it writes signals or the DOM instead of returning a value |\n')
w('## 2. The layers\n')
w('```mermaid\nflowchart TB\n  subgraph presentation["presentation/ui  (this report)"]\n    shell["app_shell: App"]\n    ctx["overlay_plugin_context"]\n    screens["screens: start, tables, history, settings"]\n    areas["areas: SettingsView (typed settings)"]\n    shared["shared: style values, safe markup, icons"]\n  end\n  subgraph application["application"]\n    appstate["app_state: section contexts, AppActions,\\ndata_ingestion, standby, toasts, settings saving"]\n    i18n["i18n: translate / message"]\n  end\n  subgraph domain["domain"]\n    settings["settings: Settings, SettingsFile, options"]\n    formatting["formatting: cells, numbers"]\n    translations["translations: ui schema"]\n  end\n  subgraph infrastructure["infrastructure"]\n    act["act: protocol, CombatDataMessage"]\n    ws["browser_websocket: BrowserWebsocketClient"]\n  end\n  ACT[("ACT / OverlayPlugin")]\n  shell --> appstate\n  shell --> ctx\n  shell --> areas\n  ctx --> act\n  ctx --> ws\n  ws <--> ACT\n  screens --> areas\n  screens --> appstate\n  screens --> formatting\n  screens --> i18n\n  areas --> settings\n  areas --> shared\n  screens --> shared\n  appstate --> settings\n  i18n --> translations\n```\n')
w('**Rule of thumb.** Components read state (their section\'s context, `OverlayPluginContext`, `SettingsView`) and call `application` functions to change it; `domain` decides what numbers and settings mean; `infrastructure` is the only side that touches the network.\n')
# ---- flows
w('## 3. Runtime flows\n')
w('### 3.1 Start-up\n')
w('```mermaid\nsequenceDiagram\n  autonumber\n  participant Main as main.rs\n  participant App as app_shell::App\n  participant Ctx as AppActions::provide\n  participant View as areas::provide_settings_view\n  participant OPC as overlay_plugin_context\n  participant WS as BrowserWebsocketClient\n  participant I18n as i18n::use_init_translations\n  participant Stand as standby_mode\n  Main->>App: launch(App)\n  App->>Ctx: one context per section (Settings, Screen, Tables, History, SettingsScreen, Dropdown, Notices) + NavigationBarContext\n  App->>View: one memo per settings area (SettingsView)\n  App->>OPC: new(signals) + spawn_connection_task\n  OPC->>OPC: parse_overlay_plugin_url (?OVERLAY_WS= / ?HOST_PORT=)\n  OPC->>WS: connect, then send subscribe\n  App->>I18n: translations for the Lang setting\n  App->>Stand: restart_standby_timer\n  App->>App: use_effect: combat data → on_combat_data_changed\n  App->>App: use_effect: settings → schedule_settings_save\n```\n')
w('### 3.2 From an ACT message to the table on screen\n')
w('```mermaid\nsequenceDiagram\n  autonumber\n  participant ACT\n  participant WS as BrowserWebsocketClient\n  participant KC as keep_connected\n  participant HM as handle_message\n  participant Peek as peek_combat_data\n  participant Proto as interpret (protocol)\n  participant Sig as combat_data_message (signal)\n  participant Eff as App effect\n  participant DI as on_combat_data_changed\n  participant Disp as TablesContext.displayed_combat_data\n  participant CT as CombatTables\n  ACT->>WS: text frame\n  WS->>KC: message channel\n  KC->>HM: raw text\n  HM->>Peek: look at the tail without parsing\n  alt idle heartbeat and not the first message\n    HM-->>KC: skipped\n  else anything else\n    HM->>Proto: parse (merge pets, local player)\n    Proto-->>HM: CombatData / PrimaryPlayerChanged\n    HM->>Sig: set(message)\n  end\n  Sig-->>Eff: changed\n  Eff->>DI: AppActions, message\n  DI->>Disp: show (also leaves the start screen, restarts standby)\n  DI->>DI: a finished fight is added to the history\n  Disp-->>CT: re-render\n  CT->>CT: visible_participants, standard_table or raid_grid\n```\n')
w('### 3.3 Drawing one table\n')
w('```mermaid\nsequenceDiagram\n  autonumber\n  participant CT as CombatTables\n  participant TE as TableEnvironment\n  participant ST as standard_table\n  participant VP as visible_participants\n  participant PR as player_row\n  participant GB as graph_bars\n  participant BH as BarHistory\n  participant Cell as cell / job_icon_cell\n  participant Fmt as domain::formatting\n  CT->>TE: new(settings areas, names, blur)\n  CT->>ST: environment, combatants, encounter, is_healing\n  ST->>VP: filter + order the players\n  ST->>ST: header_table (columns, titles)\n  loop each visible player\n    ST->>PR: row\n    PR->>GB: bars for this row\n    GB->>BH: animation_of: where was this bar?\n    BH-->>GB: flip transform or none\n    PR->>Cell: one per column\n    Cell->>Fmt: cell_fragments_ranked\n    Fmt-->>Cell: text fragments (plain and dimmed)\n  end\n  ST-->>CT: DOM\n```\n')
w('### 3.4 Changing a setting\n')
w('```mermaid\nsequenceDiagram\n  autonumber\n  participant Row as settings row (schema_rows / menus / colour picker)\n  participant AC as SettingsContext.edit_settings\n  participant Set as settings (signal)\n  participant SV as SettingsView memos\n  participant Comp as components of that area\n  participant Save as schedule_settings_save\n  participant LS as localStorage\n  Row->>AC: closure that edits Settings (option_updates)\n  AC->>Set: write\n  Set-->>SV: recompute\n  SV-->>Comp: only the area that changed notifies its readers\n  Set-->>Save: effect\n  Save->>LS: save 300 ms after the last change\n```\n')
w('### 3.5 Standby (hiding the tables after a quiet time)\n')
w('```mermaid\nsequenceDiagram\n  autonumber\n  participant Any as any event (message, screen change)\n  participant SM as restart_standby_timer\n  participant Timer as Timeout (autoHideTime minutes)\n  participant Ctx as TablesContext / DropdownContext\n  participant Toast as show_toast_message\n  Any->>SM: restart\n  SM->>Ctx: is_standby_hidden = false\n  SM->>Timer: arm\n  Timer-->>SM: fires inside the app scope\n  SM->>Ctx: close dropdown, is_standby_hidden = true\n  SM->>Toast: hiddenTable\n  Note over Ctx: MainScreen sets display:none on the tables while hidden\n```\n')
# contexts table
w('## 4. Shared state and contexts\n')
w('Provided once by `App`; every component reaches them with `use_context`. The application state is split **one context per section of the page**: a component reads only its own section\'s context. `AppActions` is what starts an action that changes several sections; its fields are private, so it cannot be used to read state.\n')
w('| Context | Defined in | Holds | Read by (files) |\n|---|---|---|---|')
ctxdesc={'SettingsContext':('application/app_state/contexts/settings_context.rs','the user\'s settings and `edit_settings()` (every section)'),
 'ScreenContext':('application/app_state/contexts/screen_context.rs','which full screen is shown (`Screen`)'),
 'TablesContext':('application/app_state/contexts/tables_context.rs','tables section: displayed combat data, local player, first-data flag, standby flag, blurred rows'),
 'HistoryContext':('application/app_state/contexts/history_context.rs','History screen: finished encounters, zone counter, viewed entry'),
 'SettingsScreenContext':('application/app_state/contexts/settings_screen_context.rs','settings screens: open page and tab (`SettingsLocation`), sample tables on/off, raid-mode sample'),
 'NavigationBarContext':('application/app_state/contexts/navigation_bar_context.rs','top bar: buttons expanded, capture icon flashing'),
 'DropdownContext':('application/app_state/contexts/dropdown_context.rs','the open pop-up menu (`Dropdown`)'),
 'NoticesContext':('application/app_state/contexts/notices_context.rs','floating messages: toast state, toast generation, tooltip'),
 'AppActions':('application/app_state/app_actions.rs','actions that change several sections (open settings, new fight, standby, backup); holds the contexts privately, so nothing can be read through it'),
 'OverlayPluginContext':('presentation/ui/overlay_plugin_context.rs','connection status, the last combat data message, the local player, errors'),
 'SettingsView':('presentation/ui/areas/mod.rs','one memo per settings area: bars, columns, nav, page, raid, table'),
 'BarHistory':('presentation/ui/combat_tables/graph_bars.rs','where each graph bar was last drawn, so a change can be animated'),
 'ColorPickerState':('presentation/ui/settings_screens/js_color.rs','what the one shared colour picker panel is doing'),
 'I18n':('application/i18n.rs','the Fluent translations for the current language')}
readers=collections.defaultdict(set)
for rel,d in F.items():
    for it in d['items']:
        for kind,ty,f in it.get('contexts',[]):
            if kind!='use_context_provider': readers[ty].add(nice(rel))
        # SettingsView reads through fields
        if it.get('area_reads'): readers['SettingsView'].add(nice(rel))
for k,(f,desc) in ctxdesc.items():
    r=sorted(readers.get(k,[]))
    w(f'| `{k}` | `{f.replace("presentation/ui/","")}` | {desc} | {", ".join("`"+x+"`" for x in r) or "(read through application functions)"} |')
w('')
# render tree
w('## 5. Component render tree\n')
w('Which component draws which (from the `rsx!` of every component).\n')
w('```mermaid\nflowchart LR')
seen=set()
for rel,tg in sorted(renders.items()):
    for t,names in tg.items():
        for n in sorted(names):
            comp=[it for it in F[rel]['items'] if '#[component]' in it['attrs']]
            for it in F[rel]['items']:
                pass
# build from item level for accuracy
edges=set()
for rel,d in F.items():
    for it in d['items']:
        for t,names in it['renders'].items():
            for n in names:
                src=it['name'] if '#[component]' in it['attrs'] or it['name'][0].isupper() else None
                if not src:
                    # helper function: attribute to the file's component(s) via name of fn
                    src=it['name']
                edges.add((src,n))
# merge helper fns into a short list
comp_names={it['name'] for d in F.values() for it in d['items'] if '#[component]' in it['attrs']}
for a,b in sorted(edges):
    w(f'  {a}["{a}"] --> {b}["{b}"]')
w('```\n')
w('Components with no parent here are drawn from `App` or from a `match` on the current screen (`Screen::Main` → `NavigationBar` + `MainScreen`; `Screen::History` → `HistoryNavigationBar` + `HistoryScreen`; `Screen::Settings` → `SettingsNavigationBar` + `SettingsScreen`).\n')
# packages
w('## 6. Package by package\n')
def short_name(rel):
    n=rel.split('/')[-1][:-3]
    return n if n!='mod' else (rel.split('/')[-2] if len(rel.split('/'))>1 else 'mod')+'/mod'
for p,files in pk.items():
    w(f'### 6.{list(pk).index(p)+1} `{p}`\n')
    w(PKDESC.get(p,'')+'\n')
    # class diagram
    ids={}
    ext={}
    lines=['classDiagram','  direction LR']
    files=[r for r in files if not (r in('presentation/mod.rs','presentation/ui/mod.rs'))]
    for rel in files:
        cid=ident(rel); ids[rel]=cid
        members=[]
        for it in F[rel]['items']:
            if it['kind'] in('mod','impl'): continue
            comp='#[component]' in it['attrs']
            if it['pub'] or comp:
                nm=it['name']
                if it['kind']=='fn':
                    members.append(f"+{nm}(){' «C»' if comp else ''}")
                elif it['kind'] in('struct','enum','trait','type'):
                    members.append(f"+{it['kind']} {nm}")
        members=members[:7]
        lines.append(f'  class {cid}["{short_name(rel)}"]')
        for m in members: lines.append(f'  {cid} : {m}')
    extra=[]
    for rel in files:
        for tgt,syms in sorted(file_edges[rel].items()):
            syms=sorted(s for s in syms if (tgt,s) not in NOISE)
            if not syms: continue
            lab=', '.join(syms[:3])+(' …' if len(syms)>3 else '')
            lab=re.sub(r'[^\w .,()\-+·:]','',lab).replace(':','')
            if tgt in ('presentation/mod.rs','presentation/ui/mod.rs'): continue
            if tgt in F and pkg_of(tgt)==p:
                rend=renders[rel].get(tgt)
                if rend: extra.append(f'  {ids[rel]} *--> {ids[tgt]} : renders {", ".join(sorted(rend))}')
                else: extra.append(f'  {ids[rel]} ..> {ids[tgt]} : {lab}')
            else:
                eid='x_'+ident(tgt)
                if eid not in ext:
                    ext[eid]=tgt
                rend=renders[rel].get(tgt)
                extra.append(f'  {ids[rel]} ..> {eid} : {("renders "+", ".join(sorted(rend))) if rend else lab}')
    for eid,tgt in ext.items():
        lay=layer(tgt)
        lines.append(f'  class {eid}["{nice(tgt) if lay=="presentation" else tgt.replace(".rs","")}"]')
        lines.append(f'  <<{ "presentation" if lay=="presentation" else lay}>> {eid}')
    lines+=extra
    w('```mermaid\n'+'\n'.join(lines)+'\n```\n')
    # tables
    for rel in files:
        d=F[rel]
        w(f'#### `{nice(rel)}`  ({d["lines"]} lines)\n')
        ms=module_summary(rel) or OVR.get((rel,'*'),'')
        if ms: w(ms+'\n')
        rows=[]
        for it in d['items']:
            if it['kind'] in('mod','impl') or it['level']=='traitmethod': continue
            if it['name'] in ('fmt','from_setting') : continue
            comp='#[component]' in it['attrs']
            if it['kind']=='const' and not it['pub']: continue
            if it['level']=='method' and not it['pub']: continue
            c=calls_of(rel,it)
            rn=[n for t,v in it['renders'].items() for n in v]
            desc=describe(rel,it)
            nm=f"`{it['name']}`"+(' «C»' if comp else '')+(' *(private)*' if not it['pub'] and not comp else '')
            kind={'fn':'component' if comp else ('method' if it['level']=='method' else 'function')}.get(it['kind'],it['kind'])
            callcell=fmt_calls(c)
            if rn: callcell=('**renders** '+', '.join(f'`{n}`' for n in rn)+('; ' if callcell else ''))+callcell
            rows.append(f"| {nm} | {kind} | {desc.replace('|','\\|')} | {produces(it)} | {callcell or '—'} |")
        if rows:
            w('| Item | Kind | What it does | Produces | Calls / uses |\n|---|---|---|---|---|')
            for r in rows: w(r)
            w('')
        else:
            w('*(declares modules only)*\n')
w('## Appendix — file index\n')
w('| File | Lines | Package |\n|---|---:|---|')
for rel in sorted(F): w(f'| `{nice(rel)}` | {F[rel]["lines"]} | {pkg_of(rel)} |')
open(OUT,'w').write('\n'.join(L)+'\n')
print(len('\n'.join(L)))
