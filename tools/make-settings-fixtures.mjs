#!/usr/bin/env node
// Writes tests/fixtures/settings/*.json: complete settings documents (the built-in defaults plus the overrides below) that between them
// cover the cases where the overlay's rendering has an edge, so a refactor can be checked against them:
//
//   header-only-bold-italic   applyScope 1 (corners only on the header), bold and italic everywhere, outlined text, gradient bars (top), pets merged
//   body-only-gradient-left   applyScope 2 (corners only on the body), gradient bars (left), role palette, pets NOT merged, small bars swapped
//   both-gradient-bottom      applyScope 3 (header and body), gradient bars (bottom), "me/you" palette, pets merged, bold only for other rows
//   raid-outline-gradient     raid mode, applyScope 3, gradient bars (right), outlined text, italic, pets NOT merged
//
// They are used by the Playwright specs (screenshots against the original, computed styles against another build) and are meant to be
// used by the Rust tests that check the typed settings against these same documents. Every document shows all rows (`sizeDPSTable` and
// `sizeHPSTable` at their maximum), in English, with animation off, like the other specs' settings.
//
//
// It also writes tests/fixtures/settings/import/*.json: older, partial and extended settings files for the loading code (the `.expected.json` next to
// each is what loading produced when the case was recorded; see tests/unit/domain/settings/persistence.rs):
//   older-version        keys added by newer versions missing, the "Cell" keys an old version had, the outdated raid threshold, and a section and keys this version doesn't know
//   no-columns           the column definitions and order missing (a file from before columns were configurable)
//   only-options         nothing but a partial `q`
//
//   node tools/make-settings-fixtures.mjs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const defaults = JSON.parse(fs.readFileSync(path.join(root, "src/data/defaults.json"), "utf8"));
const ALL_ROWS = { sizeDPSTable: 25, sizeHPSTable: 25 };

const profiles = {
  "header-only-bold-italic": {
    q: { applyScope: 1, boldYOU: 1, boldOther: 1, header_italic: 1, body_italic: 1, time_italic: 1, target_italic: 1, rps_italic: 1,
         borderTextType: "outline", gradient: 1, direction: "top", pets: 1, rd_tableTL: 1, rd_tableTR: 1, rd_tableBL: 1, rd_tableBR: 1 },
    Range: { ...ALL_ROWS, sizeRadiusTable: 12 },
  },
  "body-only-gradient-left": {
    q: { applyScope: 2, gradient: 1, direction: "left", palette: "role", borderTextType: "shadow", pets: 0, bar_position: "left", bar_position_DPS: "right",
         rd_tableTL: 1, rd_tableTR: 1, rd_tableBL: 1, rd_tableBR: 1 },
    Range: { ...ALL_ROWS, sizeRadiusTable: 12 },
  },
  "both-gradient-bottom": {
    q: { applyScope: 3, gradient: 1, direction: "bottom", palette: "meYou", myColorUse: 1, borderTextType: "shadow", pets: 1, boldYOU: 0, boldOther: 1,
         rd_tableTL: 1, rd_tableTR: 1, rd_tableBL: 1, rd_tableBR: 1 },
    Range: { ...ALL_ROWS, sizeRadiusTable: 12 },
  },
  "raid-outline-gradient": {
    q: { view24_Number: 6, applyScope: 3, gradient: 1, direction: "right", borderTextType: "outline", body_italic: 1, boldYOU: 0, pets: 0 },
    Range: { ...ALL_ROWS },
  },
};

fs.mkdirSync(path.join(root, "tests/fixtures/settings"), { recursive: true });
for (const [name, overrides] of Object.entries(profiles)) {
  const document = JSON.parse(JSON.stringify(defaults));
  document.q.Lang = "EN";
  document.q.ani = 0;
  for (const [section, values] of Object.entries(overrides)) {
    for (const [key, value] of Object.entries(values)) {
      if (!(key in document[section])) throw new Error(`${name}: ${section}.${key} is not a setting (typo, or the setting was renamed)`);
      document[section][key] = value;
    }
  }
  fs.writeFileSync(path.join(root, "tests/fixtures/settings", `${name}.json`), JSON.stringify(document, null, 2) + "\n");
  console.log(`tests/fixtures/settings/${name}.json`);
}

// ---- loading cases
const importDirectory = path.join(root, "tests/fixtures/settings/import");
fs.mkdirSync(importDirectory, { recursive: true });
const clone = () => JSON.parse(JSON.stringify(defaults));
const writeCase = (name, document) => { fs.writeFileSync(path.join(importDirectory, `${name}.json`), JSON.stringify(document, null, 2) + "\n"); console.log(`tests/fixtures/settings/import/${name}.json`); };

const older = clone();
for (const key of ["VPR", "PCT", "BST"]) delete older.Color[key];           // colours added with newer jobs
for (const key of ["resolution", "overlayBgSize", "view24"]) delete older.q[key];
for (const key of ["sizeSomethingCellHeight"]) older.Range[key] = 5;       // "Cell" keys were removed from the original in an old version
older.q.tableCellOld = 1;
older.Color.tableCellColor = "FF0000";
older.q.view24_Number = 10;                                                // the original changed this default from 10 to 14
older.q.aKeyFromTheFuture = 7;                                             // unknown keys and sections must survive
older.Future = { note: "a section this version does not know", values: [1, 2, 3] };
writeCase("older-version", older);

const noColumns = clone();
delete noColumns.ColData; delete noColumns.Order;
writeCase("no-columns", noColumns);

writeCase("only-options", { q: { pets: 0, gradient: 1, direction: "left", palette: "role" } });
