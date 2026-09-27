// Differential test: draws the same fights with the ORIGINAL MopiMopi and with this port, and compares the DPS / HPS tables
// cell by cell.
//
//   npm i jsdom ws jquery
//   node compare-tables.mjs <path to the original repo checkout> [path/to/dist]
import { spawnSync } from "child_process";
import fs from "fs";
import os from "os";
import path from "path";
import { fileURLToPath } from "url";

const here = path.dirname(fileURLToPath(import.meta.url));
const originalRepo = path.resolve(process.argv[2] || "");
const dist = path.resolve(process.argv[3] || path.join(here, "..", "..", "dist"));
const data = path.join(here, "..", "..", "src", "data");
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "mopimopi-fights-"));

// ---- fights ------------------------------------------------------------------------------------------------------------
const sample = path.join(data, "previewLog.json");
const capture = path.join(tmp, "beastmaster.json");
fs.writeFileSync(capture, fs.readFileSync(path.join(data, "captures", "overlay_plugin_beastmaster.json")));

/** A 24-player raid: mixed jobs, healers and tanks, four pets, one limit break row, values that differ per player. */
function raidFight() {
  const template = JSON.parse(fs.readFileSync(path.join(data, "captures", "overlay_plugin_beastmaster.json"), "utf8")).Combatant.YOU;
  const jobs = ["Sch", "War", "Drg", "Blm", "Mch", "Whm", "Nin", "Brd", "Pld", "Ast", "Drk", "Smn"];
  const combatants = {}; let totalDamage = 0, totalHealed = 0;
  const add = (name, job, damage, healed, extra = {}) => {
    combatants[name] = { ...template, name, Job: job, damage: String(damage), healed: String(healed), hits: String(100 + (damage % 37)),
      crithits: String(20 + (damage % 11)), DirectHitCount: String(30 + (damage % 13)), swings: String(120 + (damage % 41)),
      maxhit: `Attack-${1000 + (damage % 9000)}`, MAXHIT: String(1000 + (damage % 9000)), overHeal: String(Math.floor(healed / 5)), ...extra };
    totalDamage += damage; totalHealed += healed;
  };
  for (let i = 0; i < 24; i++) {
    const name = i === 0 ? "YOU" : `Player ${i} Name`;
    const healer = ["Sch", "Whm", "Ast"].includes(jobs[i % jobs.length]);
    add(name, jobs[i % jobs.length], 250000 - i * 7331, healer ? 90000 - i * 1201 : (i % 5) * 300);
  }
  add("Eos (YOU)", "", 4200, 31000); add("Selene (Player 1 Name)", "", 3900, 12000);
  add("Rook Autoturret (Player 4 Name)", "", 61000, 0); add("Limit Break", "", 0, 0);
  const encounter = { ...JSON.parse(fs.readFileSync(path.join(data, "captures", "overlay_plugin_beastmaster.json"), "utf8")).Encounter,
    title: "Raid boss", damage: String(totalDamage), healed: String(totalHealed), DURATION: "300", duration: "05:00" };
  return { Encounter: encounter, Combatant: combatants, isActive: "true" };
}
const raid = path.join(tmp, "raid.json");
fs.writeFileSync(raid, JSON.stringify(raidFight()));

const fights = [["sample fight (4 players, 2 pets)", sample], ["real Beastmaster capture (companion, no healing)", capture], ["24-player raid with pets and limit break", raid]];

// ---- run both and compare ---------------------------------------------------------------------------------------------
function draw(script, args) {
  const run = spawnSync(process.execPath, [path.join(here, script), ...args], { encoding: "utf8", timeout: 90000, maxBuffer: 1 << 26 });
  try { return JSON.parse(run.stdout); } catch { return { error: (run.stderr || run.stdout).slice(0, 400) }; }
}
let differences = 0;
for (const [title, file] of fights) {
  const original = draw("run-original.mjs", [originalRepo, "table", file]);
  const port = draw("run-port.mjs", [dist, file]);
  console.log(`\n${title}`);
  if (!original.error && !port.error && (original.DPS.length === 0 || port.DPS.length === 0)) { original.error = port.error = "nothing was drawn (an empty result must not count as a match)"; }
  if (original.error || port.error) { console.log("  could not draw:", original.error || port.error); differences++; continue; }
  for (const table of ["DPS", "HPS"]) {
    const a = original[table], b = port[table];
    console.log(`  ${table}: original ${a.length} rows, port ${b.length} rows`);
    if (a.length !== b.length) { differences++; }
    for (let i = 0; i < Math.max(a.length, b.length); i++) {
      if (JSON.stringify(a[i]) === JSON.stringify(b[i])) continue;
      differences++;
      console.log(`    row ${i} differs`);
      console.log(`      original: ${JSON.stringify(a[i])}`);
      console.log(`      port    : ${JSON.stringify(b[i])}`);
    }
  }
}
console.log(differences === 0 ? "\nNO DIFFERENCES" : `\n${differences} DIFFERENCE(S)`);
process.exit(differences === 0 ? 0 : 1);
