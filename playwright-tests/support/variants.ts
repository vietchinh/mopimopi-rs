// The settings sweep. Instead of a hand-written list, the variants are generated from the app's own schema
// (`src/data/l.json`) and defaults (`src/data/defaults.json`), so a setting added later is covered by the
// next run without touching this file:
//   - every on/off option is flipped,           - every choice is set to each of its other options,
//   - every slider is moved,                    - every colour is changed,
//   - every column that is off is switched on,  - plus column order and abbreviations.
import { DEFAULTS, SCHEMA, radioEntries, radioKeys, sliderEntries, type Variant } from "./settings";

/** Options that are not settings of the picture (fonts are typed text, the rest is bookkeeping). */
const NOT_VISUAL = new Set(["Lang", "backupDate", "overlayBgImg", "fTime", "fTarget", "fRPS", "fHd", "fBody"]);

const isFlag = (value: unknown) => value === 0 || value === 1;
/** A choice is stored as a number when its setting default is one (`dpsType: 0`), otherwise as text. */
const choiceValue = (key: string, option: string): unknown => (typeof DEFAULTS.q[key] === "number" && option.trim() !== "" && !Number.isNaN(Number(option)) ? Number(option) : option);

/** A slider value that differs from the default and stays in range: 30, or 50 when 30 is the default. */
function movedSlider(key: string): number {
  const { min, max } = sliderEntries.get(key) as { min: number; max: number };
  const current = DEFAULTS.Range[key];
  const clamp = (v: number) => Math.min(max, Math.max(min, v));
  let value = clamp(current === 30 ? 50 : 30);
  if (value === current) value = current === max ? min : max;
  return value;
}

export function baseVariants(): Variant[] {
  const out: Variant[] = [{ name: "defaults" }];
  for (const [key, current] of Object.entries<any>(DEFAULTS.q)) {
    if (NOT_VISUAL.has(key)) continue;
    if (radioKeys.includes(key)) {
      for (const option of Object.keys(radioEntries.get(key).m)) {
        if (String(current) !== option) out.push({ name: `q.${key}=${option}`, q: { [key]: choiceValue(key, option) } });
      }
    } else if (isFlag(current)) {
      out.push({ name: `q.${key}=${1 - current}`, q: { [key]: 1 - current } });
    }
  }
  for (const key of Object.keys(DEFAULTS.Range)) {
    const value = movedSlider(key);
    out.push({ name: `Range.${key}=${value}`, Range: { [key]: value } });
  }
  for (const key of Object.keys(DEFAULTS.Color)) {
    if (DEFAULTS.Color[key].toUpperCase() !== "FF00FF") out.push({ name: `Color.${key}`, Color: { [key]: "FF00FF" } });
  }
  for (const [column, def] of Object.entries<any>(DEFAULTS.ColData)) {
    if (def.DPS !== 1 || def.HPS !== 1) out.push({ name: `col.${column}.on`, ColData: { [column]: { DPS: 1, HPS: 1 } } });
  }
  const reversed = (list: string[]) => [...list].reverse();
  out.push({ name: "Order.DPS.reversed", Order: { ...DEFAULTS.Order, DPS: reversed(DEFAULTS.Order.DPS) } });
  out.push({ name: "Order.HPS.reversed", Order: { ...DEFAULTS.Order, HPS: reversed(DEFAULTS.Order.HPS) } });
  out.push({ name: "Alias.custom+abb", Alias: { Wildfire: "WF!", Broil: "Br." }, q: { abb: 1 } });
  return out;
}

/** The default table height (9 rows) hides the last rows; these show every row, which is what exposed a wrong colour on the Limit Break and Chocobo rows. */
const TALL = { sizeDPSTable: 25, sizeHPSTable: 25 };
export function tallVariants(): Variant[] {
  return baseVariants()
    .filter((v) => !v.Range || !("sizeDPSTable" in v.Range || "sizeHPSTable" in v.Range))
    .map((v) => ({ ...v, name: `tall|${v.name}`, Range: { ...TALL, ...v.Range } }));
}

/** Window widths at which bar edges land on fractional pixels. */
export function widthVariants(): Variant[] {
  const out: Variant[] = [];
  for (const width of [640, 777, 933, 1000]) {
    out.push({ name: `width ${width}|defaults`, width, Range: TALL });
    out.push({ name: `width ${width}|gradient`, width, Range: TALL, q: { gradient: 1 } });
    out.push({ name: `width ${width}|hps bars left`, width, Range: TALL, q: { bar_position: "left" } });
  }
  return out;
}

/** Raid mode (a grid of small cards). It switches on at a party of `view24_Number`, so 6 turns it on for the sample roster. */
export function raidVariants(): Variant[] {
  const raid = (name: string, extra: Partial<Variant> = {}): Variant => ({ name: `raid|${name}`, ...extra, q: { view24_Number: 6, ...extra.q }, Range: { ...extra.Range }, Color: { ...extra.Color } });
  const out = [raid("base")];
  for (const key of ["size24TableSlice", "size24TableHeight", "size24TableIdxWd", "size24BodyIcon", "size24BodyNameText", "size24BodyDataText"]) {
    const { min, max } = sliderEntries.get(key) as { min: number; max: number };
    out.push(raid(key, { Range: { [key]: key === "size24TableSlice" ? 3 : Math.floor((min + max) / 2) } }));
  }
  for (const key of ["view24BgYOU", "view24BgOther", "view24TableYOU", "view24TableOther"]) {
    out.push(raid(`Range.${key}`, { Range: { [key]: 30 } }));
    out.push(raid(`Color.${key}`, { Color: { [key]: "FF00FF" } }));
  }
  for (const [key, values] of [["palette", ["role", "meYou"]], ["gradient", [1]], ["hideName", [1]], ["rank", [1]], ["cnt", [2, 3, 4]], ["boldYOU", [0]], ["boldOther", [1]], ["body_italic", [1]], ["pets", [0]], ["view24", [0]], ["DPS_D", [0]], ["viewDPS", [0]], ["viewHPS", [0]], ["tableOrder", [2]]] as const) {
    for (const value of values) out.push(raid(`q.${key}=${value}`, { q: { [key]: value } }));
  }
  out.push(raid("line thickness and strength", { Range: { sizeLine: 4, tableLine: 60 } }));
  out.push(raid("line colour", { Color: { tableLine: "FF0000" } }));
  return out;
}

export function gradientVariants(): Variant[] {
  return ["top", "bottom", "left", "right"].map((direction) => ({ name: `gradient ${direction}`, q: { gradient: 1, direction }, Range: TALL }));
}

/** Tables with nothing to show, and data without the local player (the original builds no tables then). */
export function edgeVariants(): Variant[] {
  const off = (table: "DPS" | "HPS") => Object.fromEntries(["T", "H", "D", "C", "M"].map((role) => [`${table}_${role}`, 0]));
  return [
    { name: "empty|every DPS filter off", q: off("DPS") },
    { name: "empty|every HPS filter off", q: off("HPS") },
    { name: "empty|both off, raid", q: { ...off("DPS"), ...off("HPS"), view24_Number: 6 } },
    { name: "no local player|standard", noYou: true },
    { name: "no local player|raid", noYou: true, q: { view24_Number: 6 } },
  ];
}

/**
 * The original writes a choice into its settings as text (`"1"`), so real users' data holds strings where the
 * defaults hold numbers. The port must draw exactly the same from that data.
 */
export function stringRadioVariants(): Variant[] {
  return baseVariants()
    .filter((v) => v.name === "defaults" || Object.keys(v.q ?? {}).some((key) => radioKeys.includes(key)))
    .map((v) => ({ ...v, name: `text choices|${v.name}`, stringRadios: true }));
}

export const groups = {
  base: baseVariants,
  tall: tallVariants,
  widths: widthVariants,
  raid: raidVariants,
  gradients: gradientVariants,
  edges: edgeVariants,
  stringChoices: stringRadioVariants,
} as const;

export { SCHEMA };
