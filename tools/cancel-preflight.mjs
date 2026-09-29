#!/usr/bin/env node
// Prints the block in tailwind.css that undoes Tailwind's Preflight.
//
// `@import "tailwindcss"` (the Dioxus guide's first line) brings in Preflight, a reset applied to every element. The overlay is a
// port of a page that never had it, so its effects (box-sizing:border-box, `img{display:block}`, borders, list and heading
// resets, ...) are cancelled: for every selector Preflight styles, the same properties are set to `revert-layer` in the same
// layer, later, which rolls each one back to what it was before the `base` layer, i.e. the browser's own default.
// Generated from Preflight itself, so nothing is listed by hand; regenerate after a Tailwind upgrade:
//
//   node tools/cancel-preflight.mjs            print the block          node tools/cancel-preflight.mjs --check   fail if tailwind.css is out of date
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const preflight = fs.readFileSync(createRequire(path.join(root, "package.json")).resolve("tailwindcss/preflight.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "");

/** `selector { decls }` and `@at-rule (...) { rules }`, by brace matching. */
function parse(text) {
  const nodes = [];
  let i = 0;
  while (i < text.length) {
    const open = text.indexOf("{", i);
    if (open < 0) break;
    let depth = 1, j = open + 1;
    while (depth && j < text.length) depth += text[j] === "{" ? 1 : text[j] === "}" ? -1 : 0, j++;
    nodes.push({ head: text.slice(i, open).trim().replace(/\s+/g, " "), body: text.slice(open + 1, j - 1) });
    i = j;
  }
  return nodes;
}

function emit(nodes, indent) {
  return nodes.map(({ head, body }) => {
    if (head.startsWith("@")) return `${indent}${head} {\n${emit(parse(body), indent + "  ")}${indent}}\n`;
    const declarations = [...body.matchAll(/([\w-]+)\s*:[^;]*?(\s*!important)?\s*(?:;|$)/g)].map((m) => ({ property: m[1], important: !!m[2] }));
    const unique = [...new Map(declarations.map((d) => [d.property, d])).values()];
    const selectors = head.split(",").map((s) => s.trim()).join(`,\n${indent}`);
    return `${indent}${selectors} {\n${unique.map((d) => `${indent}  ${d.property}: revert-layer${d.important ? " !important" : ""};`).join("\n")}\n${indent}}\n`;
  }).join("");
}

const block = `@layer base {\n${emit(parse(preflight), "  ")}}\n`;
if (process.argv.includes("--check")) {
  const source = fs.readFileSync(path.join(root, "tailwind.css"), "utf8");
  const start = source.indexOf("/* BEGIN cancel-preflight"), end = source.indexOf("/* END cancel-preflight");
  const embedded = source.slice(source.indexOf("\n", start) + 1, end);
  if (embedded.trim() !== block.trim()) { console.error("tailwind.css: the block that cancels Preflight is out of date; regenerate it with `node tools/cancel-preflight.mjs`"); process.exit(1); }
  console.log("ok: the block that cancels Preflight is up to date");
} else process.stdout.write(block);
