#!/usr/bin/env node
// Checks what Tailwind generates from the Rust sources.
//
// Tailwind finds classes by reading `src/**/*.rs` as plain text (see the `@source` lines in tailwind.css), so any
// word in the code that happens to be a Tailwind utility (`table`, `filter`, `static`, `fixed`, ...) makes it emit a rule
// nobody asked for -- and for `flex`, `hidden` and `shadow`, which the markup also uses as *legacy* class names,
// a rule that changes how the overlay looks. Those words are listed in `@source not inline(...)` in tailwind.css. This
// script is what keeps that list complete: it compiles tailwind.css twice, with and without the Rust scan, and fails when
// the scan produces a class that is not one of the project's own.
//
//   node tools/check-tailwind.mjs [path/to/tailwind.css]      (npm run check:css)
//
// A class the scan may produce: a custom `@utility` of tailwind.css (also behind a variant: `first:corner-header-left`), an arbitrary value /
// property / variant written on purpose (`not-last:[border-right:var(--x)]`, `w-(--x)`), or a built-in utility with a value, written
// as such (`w-15`, `text-accent`): a hyphenated name is a class somebody wrote, unless a stylesheet already defines a class of that name.
// Anything else is a stray word: a bare word like `table` or `lowercase`, or a built-in that collides with a class of the stylesheets.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = fs.readFileSync(path.resolve(process.argv[2] ?? path.join(root, "tailwind.css")), "utf8");

function compile(css, name) {
  const input = path.join(root, name); // beside tailwind.css, so its `@import`s and `@source` paths resolve the same way
  const output = path.join(os.tmpdir(), `${name}.out.css`);
  fs.writeFileSync(input, css);
  try {
    execFileSync("npx", ["@tailwindcss/cli", "-i", name, "-o", output, "--minify"], { cwd: root, stdio: ["ignore", "pipe", "pipe"] });
    return fs.readFileSync(output, "utf8");
  } finally {
    fs.rmSync(input, { force: true });
  }
}

/** Top-level rules of minified CSS (`a{..}`, `@media{..}`, `@layer x;`), by brace matching. */
function rulesOf(css) {
  const rules = [];
  let depth = 0, start = 0;
  for (let i = 0; i < css.length; i++) {
    if (css[i] === "{") depth++;
    else if (css[i] === "}" && --depth === 0) { rules.push(css.slice(start, i + 1)); start = i + 1; }
    else if (css[i] === ";" && depth === 0) { rules.push(css.slice(start, i + 1)); start = i + 1; }
  }
  return rules;
}

const withScan = compile(source, ".tailwind-check-on.css");
const withoutScan = compile(source.replace(/^@source\b[^\n]*;[ \t]*$/gm, ""), ".tailwind-check-off.css");
const before = new Set(rulesOf(withoutScan));
const generated = rulesOf(withScan).filter((rule) => !before.has(rule));

// Our own utilities are `@utility` blocks in css/*.css (and, if any, in tailwind.css itself).
const sheets = [source, ...fs.readdirSync(path.join(root, "css")).filter((f) => f.endsWith(".css")).map((f) => fs.readFileSync(path.join(root, "css", f), "utf8"))];
const declared = new Set(sheets.flatMap((css) => [...css.matchAll(/^@utility ([\w-]+)/gm)].map((m) => m[1])));
// Classes the stylesheets of the overlay define (the original's, the layout glue, the page rules): a built-in with the same name would change them.
const definedClasses = new Set(
  ["public/base.css", "css/legacy.css", "css/page.css"].flatMap((file) => [...fs.readFileSync(path.join(root, file), "utf8").replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/\.([A-Za-z_][\w-]*)/g)].map((m) => m[1])),
);
const unescape = (name) => name.replace(/\\(.)/g, "$1");
/** The utility a class is made of, without its variants (`first:corner-header-left` -> `corner-header-left`). */
const utilityOf = (name) => name.replace(/^(?:[\w-]+:)+/, "");

const strays = new Map(); // class -> first rule that produced it
const seen = new Set();
for (const rule of generated) {
  for (const [, prelude] of rule.matchAll(/([^{};]+)\{/g)) {
    for (const selector of prelude.split(",")) {
      // Only the class a rule is *for*, the one its selector starts with. A utility may style other classes below itself
      // (`.table-text-other td .ex`, a legacy class it deliberately restyles); those are not classes Tailwind generated.
      const leading = selector.trim().match(/^\.((?:\\.|[\w-])+)/);
      if (!leading) continue;
      const name = unescape(leading[1]);
      seen.add(utilityOf(name));
      const utility = utilityOf(name);
      const deliberate = declared.has(utility) || /[\[(]/.test(name) || (utility.includes("-") && !definedClasses.has(utility));
      if (!deliberate && !strays.has(name)) {
        // the rule itself, not the start of the layer block it sits in
        const at = rule.indexOf(`${prelude}{`);
        strays.set(name, (at >= 0 ? rule.slice(at, rule.indexOf("}", at) + 1) : rule).slice(0, 100));
      }
    }
  }
}

const unused = [...declared].filter((name) => !seen.has(name));
if (unused.length) console.warn(`warning: @utility defined but never used in src/: ${unused.join(", ")} (dead CSS, or the class is built at runtime, which Tailwind cannot see)`);

if (strays.size) {
  console.error(`\nTailwind generated ${strays.size} class(es) from words in the Rust sources that are not this project's own:\n`);
  for (const [name, rule] of strays) console.error(`  .${name}    ${rule}`);
  console.error(`\nAdd them to the \`@source not inline("{...}")\` list in tailwind.css (https://tailwindcss.com/docs/detecting-classes-in-source-files#explicitly-excluding-classes),`);
  console.error(`or, if one is meant to be a class, define it with \`@utility\` in tailwind.css.`);
  process.exit(1);
}
console.log(`ok: the Rust scan generated ${generated.length} rule(s), all of them the project's own (${declared.size} @utility definitions, ${seen.size} classes seen).`);
