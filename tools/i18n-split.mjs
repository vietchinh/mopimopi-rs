#!/usr/bin/env node
// One-off migration: splits the combined translation files into
//   * `l.json` / `d.json`      – structure only (page layout, row types, icons, ranges), where every translated
//                                text has become an id: "i18n:<message id>"
//   * `locales/<lang>.ftl`     – the texts, as Project Fluent files, one per language
//
// The combined files it reads (the shape `src/data` had before) are kept in tools/i18n-source/. After this migration
// the .ftl files are what translators edit; this script only exists to show where they came from and to re-run the
// split against the original data.
//
//   node tools/i18n-split.mjs [--out DIR]       (default: writes into src/data and src/locales)
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const out = process.argv.includes("--out") ? path.resolve(process.argv[process.argv.indexOf("--out") + 1]) : null;
const dataDir = out ? path.join(out, "data") : path.join(root, "src/data");
const localesDir = out ? path.join(out, "locales") : path.join(root, "src/locales");
fs.mkdirSync(dataDir, { recursive: true });
fs.mkdirSync(localesDir, { recursive: true });

/** The overlay's language codes and the language tags they become. */
const LANGUAGES = { KR: "ko-KR", JP: "ja-JP", EN: "en-US", FR: "fr-FR", DE: "de-DE", CN: "zh-CN" };
export const ID_PREFIX = "i18n:";

/**
 * A translated text: an object of strings keyed by two-letter language codes, at least one of them a language the overlay
 * has. Codes it does not have are ignored, and reported: the original data has one text keyed `HE` where `DE` was meant, and
 * as its text is the same in every language, nothing is lost by leaving it out.
 */
const ignored = [];
const isTranslation = (node) =>
  node && typeof node === "object" && !Array.isArray(node) && Object.keys(node).length > 0 &&
  Object.keys(node).every((key) => /^[A-Z]{2}$/.test(key)) && Object.keys(node).some((key) => key in LANGUAGES) &&
  Object.values(node).every((value) => typeof value === "string");

/** Fluent ids: letters, digits, `-` and `_`; and at most one `.` is special to dioxus-i18n, so none is used. */
const sanitize = (part) => part.replace(/[^A-Za-z0-9_-]/g, "_");

/** A text as Fluent source: plain when that is safe, otherwise an exact quoted literal. */
function fluentValue(text) {
  const plain = text.length > 0 && !/[{}\r\n\t]/.test(text) && text === text.trim() && !/^[\[*.]/.test(text) && !/[\u0000-\u001f\u2066-\u2069]/.test(text);
  if (plain) return text;
  const escaped = [...text].map((c) => (c === "\\" ? "\\\\" : c === '"' ? '\\"' : c.codePointAt(0) < 0x20 ? `\\u${c.codePointAt(0).toString(16).padStart(4, "0")}` : c)).join("");
  return `{ "${escaped}" }`;
}

const messages = Object.fromEntries(Object.keys(LANGUAGES).map((code) => [code, []]));
const used = new Set();
const sections = [];

function split(node, prefix, trail) {
  if (isTranslation(node)) {
    let id = prefix + trail.map(sanitize).join("-");
    for (let n = 2; used.has(id); n++) id = `${prefix}${trail.map(sanitize).join("-")}-${n}`;
    used.add(id);
    for (const [code, text] of Object.entries(node)) {
      if (code in LANGUAGES) messages[code].push({ id, text, section: trail[0] });
      else ignored.push(`${id}: ${code}`);
    }
    return ID_PREFIX + id;
  }
  if (Array.isArray(node)) return node.map((item, i) => split(item, prefix, [...trail, String(i)]));
  if (node && typeof node === "object") return Object.fromEntries(Object.entries(node).map(([key, value]) => [key, split(value, prefix, [...trail, key])]));
  return node;
}

for (const [file, prefix] of [["l", "l-"], ["d", "d-"]]) {
  const source = JSON.parse(fs.readFileSync(path.join(root, "tools/i18n-source", `${file}.json`), "utf8"));
  fs.writeFileSync(path.join(dataDir, `${file}.json`), JSON.stringify(split(source, prefix, [])));
}

for (const [code, tag] of Object.entries(LANGUAGES)) {
  let lastSection = null;
  const lines = [`# ${tag}: every text the overlay shows. Generated once from the original lang.js / dic.js by tools/i18n-split.mjs;`, "# this file is now the place to edit a translation. A text missing here falls back to en-US."];
  for (const { id, text, section } of messages[code]) {
    if (section !== lastSection) { lines.push("", `## ${section}`); lastSection = section; }
    lines.push(`${id} = ${fluentValue(text)}`);
  }
  fs.writeFileSync(path.join(localesDir, `${tag}.ftl`), lines.join("\n") + "\n");
}
if (ignored.length) console.log(`ignored texts under language codes the overlay does not have: ${ignored.join(", ")}`);
console.log(`split ${used.size} translated texts into ${Object.keys(LANGUAGES).length} languages -> ${localesDir}`);
console.log(Object.entries(messages).map(([c, m]) => `${c}:${m.length}`).join(" "));
