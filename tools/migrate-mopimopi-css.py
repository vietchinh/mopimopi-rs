#!/usr/bin/env python3
"""One-off migration: turns assets/mopimopi.css (the original overlay's minified stylesheet) into a readable, sectioned block
of plain CSS inside tailwind.css, and deletes the file. Kept so the conversion can be audited (`git log` has the input).

Rules stay UN-LAYERED and in their original order, with their original selectors, so their place in the cascade is unchanged
(un-layered legacy rules < the `--accent` overrides and utilities that follow them in tailwind.css). Only cleanups that cannot
change what a browser does are made, each one listed in CLEANUPS below and written into the output.
"""
import re, sys

CLEANUPS = [
    "`@charset` dropped (the file is ASCII).",
    "Vendor aliases that duplicate a standard property in the same rule (`-webkit-animation-*`, `-webkit-transition` next to `transition`, "
    "`-ms-/-moz-/-khtml-/-webkit-user-select` next to `user-select`) are dropped; Tailwind's Lightning CSS adds prefixes itself when a target needs them.",
    "`-webkit-transition` / `-webkit-filter` / `-webkit-appearance` that had NO standard twin are renamed to the standard property (Blink treats them as aliases).",
    "`-webkit-transition:all 03s` (an invalid time, ignored by every browser) dropped; the valid `transition:all 0.3s` next to it is kept.",
    "`@-webkit-keyframes flash` dropped (`@keyframes flash` is right next to it).",
    "The selector `input [type=\"file\"]` (a descendant of an <input>, which matches nothing) and the type selector `scrollbar` (no such element) dropped.",
    "Stray `;;` removed.",
    "Kept as they are, because they mean something: `-webkit-app-region`, `-webkit-linear-gradient(...)` (legacy start-side direction), `-webkit-tap-highlight-color`, "
    "`-webkit-font-smoothing`, `-moz-osx-font-smoothing`, and the `-webkit-scrollbar` / `-webkit-slider-*` pseudo-elements.",
]

SECTIONS = {  # first selector of a rule -> heading printed above it
    "*": "Reset, page and typography",
    ".material-icons": "Helper classes the markup uses by name",
    "#wrap": "The window and the top bar",
    "[name=notice]": "Start screen (notice, update log)",
    "#DPSBody,#HPSBody,#HISTORYBody": "Tables: header, bars, cells, raid cards",
    ".dropdown": "Menus, dropdown lists and the dimmed backdrop",
    ".previewArea>div": "Settings screens: preview, rows, tabs",
    ".animated": "Animations",
    "input[type=checkbox],input[type=radio],input [type=\"file\"]": "Form controls: switch, text box, slider, colour box",
    "#tooltip": "Tooltip and toasts",
}

def parse(css):
    css = re.sub(r"@charset[^;]*;", "", css)
    rules, buf, depth = [], "", 0
    for ch in css:
        buf += ch
        if ch == "{": depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0: rules.append(buf.strip()); buf = ""
    return rules

def clean_decls(decls):
    props = [d.split(":", 1)[0].strip() for d in decls]
    out = []
    for d in decls:
        p, v = (x.strip() for x in d.split(":", 1))
        if p in ("-ms-user-select", "-moz-user-select", "-khtml-user-select", "-webkit-user-select"): continue
        if p.startswith("-webkit-animation-") and p[len("-webkit-"):] in props: continue
        if p == "-webkit-transition":
            if "transition" in props: continue
            p = "transition"
        if p == "transition" and re.search(r"\ball 03s\b", v): continue
        if p == "-webkit-transition" and v == "all 03s": continue
        if p == "-webkit-filter": p = "filter"
        if p == "-webkit-appearance": p = "appearance"
        out.append(f"{p}: {v};")
    return out

def convert(rules):
    out = []
    for r in rules:
        if r.startswith("@-webkit-keyframes"): continue
        head, body = r.split("{", 1); body = body.rsplit("}", 1)[0]
        head = head.strip()
        selectors = [s.strip() for s in head.split(",")] if not head.startswith("@") else None
        title = SECTIONS.get(head)
        if title: out.append(f"\n/* --- {title} --- */")
        if selectors is not None:
            selectors = [s for s in selectors if s not in ('input [type="file"]', "scrollbar")]
            decls = [d for d in body.split(";") if ":" in d]
            lines = clean_decls(decls)
            sel = ",\n".join(selectors)
            out.append(sel + " {\n" + "\n".join("  " + l for l in lines) + "\n}")
        else:  # @keyframes: keep the body, one keyframe per line
            inner = re.sub(r"\}\s*", "}\n  ", body.strip()).strip()
            frames = "\n".join("  " + f.strip() for f in re.findall(r"[^{}]+\{[^{}]*\}", body))
            out.append(head + " {\n" + frames + "\n}")
    return "\n".join(out)

if __name__ == "__main__":
    src = open("tools/migration-source/mopimopi.css", encoding="utf-8").read()
    block = convert(parse(src))
    header = ("/* ===================================================================================================\n"
              "   The original overlay's stylesheet (was assets/mopimopi.css, kept in tools/migration-source/), moved here as plain, un-layered CSS.\n"
              "   Order and selectors are unchanged. Cleanups made in the move (all no-ops for a browser):\n"
              + "\n".join(f"   - {c}" for c in CLEANUPS) +
              "\n   =================================================================================================== */\n")
    open("/tmp/legacy_block.css", "w", encoding="utf-8").write(header + block + "\n")
    print(len(parse(src)), "rules ->", block.count("{"), "blocks;", len((header+block).splitlines()), "lines")
