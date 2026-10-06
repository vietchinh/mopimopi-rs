// Combat data the overlay is fed. Built from the original's real ACT sample (`previewLog.json`), so every
// field has the exact shape and formatting ACT sends, with a roster chosen to exercise everything the
// overlay special-cases: tanks, healers, DPS, pets (merged and not), Limit Break, a Chocobo and a crafter.
import fs from "node:fs";
import path from "node:path";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type CombatData = any;
export type RosterEntry = [name: string, job: string, damage: number, healed: number, seconds: number, extras?: Extras];
type Extras = { maxhit?: [string, number]; maxheal?: [string, number]; overheal?: number; shield?: number; deaths?: number; taken?: number };

const REAL = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../../src/data/previewLog.json"), "utf8").replace(/^\uFEFF/, ""));
const fmt = (n: number) => Math.round(n).toLocaleString("en-US");
const k = (n: number) => String(Math.round(n / 1000));
const two = (n: number) => n.toFixed(2);
const mmss = (s: number) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;

export const ROSTER: RosterEntry[] = [
  ["Player Nin", "Nin", 84000, 0, 34], ["YOU", "Sch", 77193, 12852, 34, { shield: 6426, maxheal: ["Adloquium", 6426] }],
  ["Player Whm", "Whm", 21000, 62000, 35, { overheal: 14000, shield: 3200, maxheal: ["Cure III", 9100] }],
  ["Player Mch", "Mch", 128399, 0, 23, { maxhit: ["Wildfire", 18106] }], ["Player War", "War", 98000, 3000, 35, { maxhit: ["Inner Chaos", 12800], deaths: 1 }],
  ["Player Blm", "Blm", 91000, 0, 33, { maxhit: ["Flare", 24000] }], ["Player Drk", "Drk", 60000, 1500, 35, { maxhit: ["Disesteem", 15000] }],
  ["Player Ast", "Ast", 24000, 48000, 35, { overheal: 9000, maxheal: ["Aspected Helios", 5200] }], ["Player Rdm", "Rdm", 72000, 0, 34],
  ["Eos (YOU)", "", 0, 18000, 30, { overheal: 5000, maxheal: ["Embrace", 2100] }],
  ["Rook Autoturret (Player Mch)", "", 29046, 0, 19, { maxhit: ["Rook Overload", 14809] }],
  ["Limit Break", "", 15000, 0, 4, { maxhit: ["Limit Break", 15000] }],
  ["Chocobo (Player War)", "0", 6500, 0, 20, { maxhit: ["Peck", 900] }], ["Player Crp", "Crp", 1200, 0, 20],
];

/** The roster minus everyone tied to `YOU`, for "no local player in the data" (the original crashes on orphaned pets). */
export const ROSTER_WITHOUT_YOU = ROSTER.filter(([name]) => name !== "YOU" && !name.includes("(YOU)"));

export function makeData(options: { roster?: RosterEntry[]; encounterSeconds?: number; zone?: string; encounter?: Record<string, unknown> } = {}): CombatData {
  const roster = options.roster ?? ROSTER;
  const encDur = options.encounterSeconds ?? 35;
  const total = roster.reduce((sum, r) => sum + r[2], 0);
  const totalHeal = roster.reduce((sum, r) => sum + r[3], 0);
  const template = REAL.Combatant["YOU"];
  const combatants: Record<string, unknown> = {};
  for (const [name, job, dmg, heal, dur, x = {}] of roster) {
    const mh = x.maxhit ?? ["Broil", Math.round(dmg / 9) || 0];
    const mheal = x.maxheal ?? ["", 0];
    combatants[name] = {
      ...template, name, Job: job, duration: mmss(dur), DURATION: String(dur), damage: String(dmg), "damage-m": two(dmg / 1e6), "damage-b": "0.00",
      "DAMAGE-k": k(dmg), "DAMAGE-m": String(Math.round(dmg / 1e6)), "DAMAGE-b": "0", "damage%": Math.round((dmg / total) * 100) + "%",
      dps: two(dmg / dur), DPS: String(Math.round(dmg / dur)), "DPS-k": k(dmg / dur), "DPS-m": "0", encdps: two(dmg / encDur), ENCDPS: String(Math.round(dmg / encDur)),
      "ENCDPS-k": k(dmg / encDur), "ENCDPS-m": "0", hits: String(20 + (dmg % 13)), crithits: String(4 + (dmg % 5)), "crithit%": 12 + (dmg % 17) + "%",
      swings: String(20 + (dmg % 13)), maxhit: dmg ? `${mh[0]}-${fmt(mh[1])}` : "", MAXHIT: dmg ? fmt(mh[1]) : "",
      healed: String(heal), "healed%": totalHeal ? Math.round((heal / totalHeal) * 100) + "%" : "0%", enchps: two(heal / encDur), ENCHPS: String(Math.round(heal / encDur)),
      "ENCHPS-k": k(heal / encDur), heals: String(heal ? 9 : 0), critheals: String(heal ? 2 : 0),
      maxheal: heal ? `${mheal[0] || "Cure"}-${fmt(mheal[1] || 3000)}` : "", MAXHEAL: heal ? fmt(mheal[1] || 3000) : "", maxhealward: "", MAXHEALWARD: "",
      healstaken: String(Math.round(dmg / 12)), damagetaken: String(x.taken ?? 0), deaths: String(x.deaths ?? 0),
      Last10DPS: two((dmg / dur) * 1.1), Last30DPS: two((dmg / dur) * 1.05), Last60DPS: two(dmg / dur), Last180DPS: two(dmg / dur),
      overHeal: String(x.overheal ?? 0), damageShield: String(x.shield ?? 0), absorbHeal: "0", OverHealPct: heal ? Math.round(((x.overheal ?? 0) / heal) * 100) + "%" : "0%",
      DirectHitPct: 20 + (dmg % 9) + "%", DirectHitCount: String(6 + (dmg % 5)), CritDirectHitCount: String(1 + (dmg % 3)), CritDirectHitPct: 3 + (dmg % 8) + "%",
      ParryPct: "0%", BlockPct: "0%", threatstr: "+(0)0/-(0)0", threatdelta: "0",
    };
  }
  // JSON objects keep insertion order: scramble it (alphabetical) so nothing depends on already-sorted input.
  const scrambled: Record<string, unknown> = {};
  for (const name of Object.keys(combatants).sort((a, b) => a.localeCompare(b))) scrambled[name] = combatants[name];
  const encounter = {
    ...REAL.Encounter, title: "Striking Dummy", CurrentZoneName: options.zone ?? "Middle La Noscea", duration: mmss(encDur), DURATION: String(encDur),
    damage: String(total), "damage-m": two(total / 1e6), "DAMAGE-k": k(total), dps: two(total / encDur), DPS: String(Math.round(total / encDur)), "DPS-k": k(total / encDur),
    encdps: two(total / encDur), ENCDPS: String(Math.round(total / encDur)), "ENCDPS-k": k(total / encDur), healed: String(totalHeal), enchps: two(totalHeal / encDur),
    ENCHPS: String(Math.round(totalHeal / encDur)), "ENCHPS-k": k(totalHeal / encDur), deaths: "1", ...(options.encounter ?? {}),
  };
  return { Encounter: encounter, Combatant: scrambled, isActive: "true" };
}

/**
 * Data whose bars change *relative to each other* from one update to the next (`step` 0, 1, 2 ...), so bar
 * widths really animate. (Scaling everyone by the same factor leaves every bar's width unchanged.)
 */
export const shiftingData = (step: number): CombatData =>
  makeData({ roster: ROSTER.map(([name, job, dmg, ...rest], j) => [name, job, Math.round(dmg * (1 + 0.5 * Math.sin(step * 1.7 + j))), ...rest] as RosterEntry) });
