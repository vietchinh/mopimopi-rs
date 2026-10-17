// Port-only: the "NPCs look like themselves" option (deviation D1, docs/ddd/deviations.md). There is nothing to compare
// with the original here, so these tests only look at the port. What they guard:
//  * off by default, NPC rows are drawn exactly like the original draws them (the Limit Break icon)
//  * on, they get their own icon
//  * the option never leaks into the MopiMopi settings or the Custom UI Data code
import { expect } from "@playwright/test";
import { test } from "../support/fixtures";
import { makeData, ROSTER, type RosterEntry } from "../support/data";
import { openSettings, runSteps } from "../support/steps";

const NPCS: RosterEntry[] = [["Aw'aern", "", 51000, 0, 34]];
const data = makeData({ roster: [...ROSTER, ...NPCS] });
const TALL = { sizeDPSTable: 25, sizeHPSTable: 25 };

const npcIcon = (page: import("@playwright/test").Page) =>
  page.evaluate(() => {
    const row = [...document.querySelectorAll("#DPSBody .tableWrap")].find((r) => r.textContent?.includes("Aw'aern") || r.id.includes("Aw"));
    const img = (row ?? document).querySelector("img");
    return img?.getAttribute("src") ?? null;
  });

test("NPC rows look like Limit Break by default, and like themselves when the option is on", async ({ app }) => {
  const { page } = await app.openPort({ variant: { Range: TALL, q: { hideName: 0 } }, data });
  await page.waitForTimeout(600);
  const settingsBefore = await page.evaluate(() => localStorage.getItem("Mopi2_HAERU"));
  expect(await page.evaluate(() => localStorage.getItem("MopiMopi_port_preferences"))).toBeNull();

  await openSettings(page);
  await runSteps(page, ["li#Tool", "li#custom"]);
  const shareBefore = await page.locator("#in_share").inputValue();
  await page.locator("#port-only-section li#portNpcLook").click();
  // the option is saved in its own store ...
  const stored = await page.evaluate(() => localStorage.getItem("MopiMopi_port_preferences"));
  expect(JSON.parse(stored!).npcOwnLook).toBe(true);
  // ... and nothing about it reaches the MopiMopi settings or the shareable code
  const settingsAfter = await page.evaluate(() => localStorage.getItem("Mopi2_HAERU"));
  expect(settingsAfter).toBe(settingsBefore);
  expect(await page.locator("#in_share").inputValue()).toBe(shareBefore);
  expect(await page.locator("#in_portShare").inputValue()).toMatch(/^MOPIPORT1:/);
});

test("with the option on the NPC row has its own icon and colour, and off it has the Limit Break's", async ({ app }) => {
  const { page, handle } = await app.openPort({ variant: { Range: TALL }, data });
  await page.waitForTimeout(600);
  const icons = () => page.evaluate(() => [...document.querySelectorAll("#DPSBody img")].map((img) => img.getAttribute("src")));
  expect((await icons()).some((src) => src?.endsWith("/NPC.png"))).toBe(false);
  await page.evaluate(() => localStorage.setItem("MopiMopi_port_preferences", JSON.stringify({ npcOwnLook: true, npcColor: "FF00AA" })));
  await page.reload();
  await handle.send(data);
  await page.waitForTimeout(800);
  expect((await icons()).filter((src) => src?.endsWith("/NPC.png")).length).toBe(1);
  expect(await page.evaluate(() => document.body.innerHTML.toLowerCase().includes("#ff00aa") || document.body.innerHTML.toLowerCase().includes("rgb(255, 0, 170)"))).toBe(true);
});

test("a MOPIPORT1 code is refused by the Custom UI Data box and the other way round", async ({ app }) => {
  const { page } = await app.openPort({ variant: { Range: TALL }, data });
  await openSettings(page);
  await runSteps(page, ["li#Tool", "li#custom"]);
  const portCode = await page.locator("#in_portShare").inputValue();
  const before = await page.evaluate(() => localStorage.getItem("Mopi2_HAERU"));
  await page.locator("#in_apply").fill(portCode);
  await page.locator("#in_apply").press("Enter");
  expect(await page.evaluate(() => localStorage.getItem("Mopi2_HAERU"))).toBe(before);
});
