// A new fight starts with ACT sending *active* messages that have no combatants yet (real capture: title "Encounter", "Combatant":{}).
// The original overlay redraws (empties) its tables on them and treats the first inactive message after them as the end of that fight.
// If they are dropped as heartbeats, the old table stays up and a short fight that has ended by its first real message is never shown.
import { test, expect } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { settingsFor } from "../support/settings";
import { PORT_URL } from "../support/urls";
const BASE = PORT_URL;
const RAW = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../../tests/fixtures/overlay/ended-after-5s-fight.json"), "utf8"));
const fight = (title: string, active: boolean, k: number, seconds: string) => {
  const d = JSON.parse(JSON.stringify(RAW)); d.isActive = active ? "true" : "false"; d.Encounter.title = title; d.Encounter.duration = seconds;
  for (const c of Object.values<any>(d.Combatant)) { c.damage = String(Math.round(Number(c.damage) * k)); }
  return d;
};
const empty = (seconds: string) => { const d = fight("Encounter", true, 0, seconds); d.Combatant = {}; d.Encounter.damage = "0"; d.Encounter.maxhit = ""; return d; };
test("a new encounter that starts empty clears the old table, and a fight already over at its first message is shown", async ({ browser }) => {
  const page = await browser.newPage({ viewport: { width: 1100, height: 640 } });
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push("pageerror: " + e.message.slice(0, 200)));
  await page.addInitScript((b) => localStorage.setItem("Mopi2_HAERU", b), JSON.stringify(settingsFor({ q: { btn_History: 1 } })));
  const id = "empty" + Date.now();
  const send = (d: any, first = false) => fetch(`http://127.0.0.1:9090/${first ? "__session" : "__push"}/${id}`, { method: first ? "PUT" : "POST", body: JSON.stringify(d) });
  await send(fight("Shinryu", true, 5, "09:55"), true);
  await page.goto(`${BASE}/?OVERLAY_WS=ws://127.0.0.1:9090/${id}/ws`);
  await page.waitForTimeout(1500);
  const expectTable = async (rows: number, title: string) => {
    await expect.poll(() => page.evaluate(() => ({ rows: document.querySelectorAll("#DPSBody tr").length, nav: document.querySelector("nav")?.textContent || "" })), { timeout: 5000 })
      .toMatchObject({ rows, nav: expect.stringContaining(title) });
  };
  await expectTable(3, "Shinryu");
  await send(fight("Shinryu", false, 5, "09:55")); await expectTable(3, "Shinryu");
  for (const s of ["00:01", "00:03", "00:06"]) await send(empty(s));
  await expectTable(0, "Encounter");
  await send(fight("Palace Slime", false, 0.2, "00:03")); await expectTable(3, "Palace Slime");
  await send(fight("Palace Slime", false, 0.2, "00:03")); await expectTable(3, "Palace Slime");
  await send(empty("00:01")); await send(fight("Imp", true, 0.3, "00:04")); await expectTable(3, "Imp");
  await send(fight("Imp", false, 0.3, "00:04")); await expectTable(3, "Imp");
  expect(errors).toEqual([]);
});
