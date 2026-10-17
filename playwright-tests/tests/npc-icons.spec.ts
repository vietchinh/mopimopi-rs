// Trust and duty-support NPCs (ACT sends them with no job and no "(owner)" in the name) are drawn by the original as
// Limit Break: the Limit Break icon, its gold bar, and its label when names are hidden. The port used to give them a
// broken image and a black bar.
import { test } from "../support/fixtures";
import { makeData, ROSTER, type RosterEntry } from "../support/data";

const NPCS: RosterEntry[] = [
  ["Aw'aern", "", 51000, 0, 34],
  ["Aw'zdei", "", 2155, 0, 34, { deaths: 4 }],
];
const data = makeData({ roster: [...ROSTER, ...NPCS] });
const TALL = { sizeDPSTable: 25, sizeHPSTable: 25 };

const VARIANTS: [name: string, q: Record<string, unknown>][] = [
  ["default", {}],
  ["names hidden", { hideName: 1 }],
  ["role palette", { palette: "role" }],
  ["me/you palette", { palette: "meYou" }],
  ["raid mode", { view24_Number: 6 }],
  ["gradient bars", { gradient: 1 }],
  ["tanks only", { DPS_D: 0, DPS_H: 0, DPS_C: 0, DPS_M: 0 }],
  ["combined pets off", { pets: 0 }],
];
for (const [name, q] of VARIANTS) {
  test(`NPCs with no job look like the original's: ${name}`, async ({ app }) => {
    await app.matchScreenshot(`npc-icons/${name}`, { variant: { q, Range: TALL }, data });
  });
}

// the icon set is chosen in the settings; the fallback has to exist in each one
for (const iconSet of ["frame", "glow", "antique", "black"]) {
  test(`NPC icon in the "${iconSet}" set`, async ({ app }) => {
    await app.matchScreenshot(`npc-icons/set-${iconSet}`, { variant: { q: { iconSet }, Range: TALL }, data });
  });
}

test("no broken images anywhere in the table", async ({ app }) => {
  const { page } = await app.openPort({ variant: { Range: TALL }, data });
  await page.waitForTimeout(600);
  const broken = await page.evaluate(() =>
    [...document.querySelectorAll<HTMLImageElement>("#DPSBody img, #HPSBody img")].filter((img) => !img.complete || img.naturalWidth === 0).map((img) => img.getAttribute("src")));
  if (broken.length) throw new Error(`broken images: ${broken.join(", ")}`);
});
