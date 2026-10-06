// The overlay's settings blob (what it keeps in localStorage under `Mopi2_HAERU`) for a test variant.
import fs from "node:fs";
import path from "node:path";

/* eslint-disable @typescript-eslint/no-explicit-any */
export type Settings = any;
export type Variant = {
  name: string;
  q?: Record<string, unknown>;
  Range?: Record<string, number>;
  Color?: Record<string, string>;
  /** Replaces the whole section. */
  Alias?: Record<string, string>;
  Order?: Record<string, string[]>;
  /** Per column, values merged into its definition (`{ DPS: 1, HPS: 1 }` turns a column on). */
  ColData?: Record<string, Record<string, unknown>>;
  /** Window width in pixels (default 1100). */
  width?: number;
  /** Store the radio choices as text (`"1"`), the way the original writes them, instead of numbers. */
  stringRadios?: boolean;
  /** Use a data set without the local player (`YOU`). */
  noYou?: boolean;
};

export const DEFAULTS: Settings = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../../src/data/defaults.json"), "utf8"));
export const SCHEMA: any = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../../src/data/l.json"), "utf8"));

/**
 * Schema entries by row type, then by setting id. The type matters: one id can be several rows (`edge` is both a
 * colour on the colour page and a slider on the opacity page), and only the right one has the range or the options.
 */
const entriesByType: Map<string, Map<string, any>> = (() => {
  const found = new Map<string, Map<string, any>>();
  const walk = (node: any) => {
    if (!node || typeof node !== "object") return;
    for (const [key, value] of Object.entries<any>(node)) {
      if (value && typeof value === "object" && typeof value.e === "string") {
        if (!found.has(value.e)) found.set(value.e, new Map());
        if (!found.get(value.e)!.has(key)) found.get(value.e)!.set(key, value);
      }
      walk(value);
    }
  };
  walk(SCHEMA);
  return found;
})();
export const entriesOf = (...types: string[]): Map<string, any> => new Map(types.flatMap((type) => [...(entriesByType.get(type) ?? [])]));
export const sliderEntries = entriesOf("li_slider");
export const radioEntries = entriesOf("li_radio", "li_radio_change");
export const radioKeys: string[] = [...radioEntries.keys()];

/** English UI, animation off (so a screenshot never catches a bar half-grown), then the variant on top. */
export function settingsFor(variant: Partial<Variant>): Settings {
  const s = JSON.parse(JSON.stringify(DEFAULTS));
  s.q.Lang = "EN";
  s.q.ani = 0;
  if (variant.q) Object.assign(s.q, variant.q);
  if (variant.Color) Object.assign(s.Color, variant.Color);
  if (variant.Range) Object.assign(s.Range, variant.Range);
  if (variant.Alias) s.Alias = variant.Alias;
  if (variant.Order) s.Order = variant.Order;
  for (const [column, values] of Object.entries(variant.ColData ?? {})) Object.assign(s.ColData[column], values);
  if (variant.stringRadios) for (const key of radioKeys) if (typeof s.q[key] === "number") s.q[key] = String(s.q[key]);
  return s;
}
