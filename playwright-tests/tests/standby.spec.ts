// First start: an idle ACT (only empty, inactive messages) must take the page off the start screen, and when the standby
// timer fires (outside any component) the tables hide with a toast instead of the app panicking and freezing.
import { test, expect } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import { settingsFor } from "../support/settings";
import { PORT_URL } from "../support/urls";
const RAW = JSON.parse(fs.readFileSync(path.resolve(__dirname, "../../tests/fixtures/overlay/ended-after-5s-fight.json"), "utf8"));
const message = (active: boolean, empty: boolean) => {
  const d = JSON.parse(JSON.stringify(RAW)); d.isActive = active ? "true" : "false";
  if (empty) { d.Combatant = {}; d.Encounter.damage = "0"; d.Encounter.maxhit = ""; }
  return d;
};
test("an idle ACT leaves the start screen, and standby hides the tables without a panic", async ({ browser }) => {
  const page = await browser.newPage({ viewport: { width: 1100, height: 640 } });
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push("pageerror: " + e.message.slice(0, 200)));
  page.on("console", (m) => { if (m.type() === "error" && /panicked/.test(m.text())) errors.push(m.text().slice(0, 300)); });
  await page.addInitScript((b) => localStorage.setItem("Mopi2_HAERU", b), JSON.stringify(settingsFor({ name: "standby", Range: { autoHideTime: 0.03 } })));
  const id = "standby" + Date.now();
  const send = (d: any, first = false) => fetch(`http://127.0.0.1:9090/${first ? "__session" : "__push"}/${id}`, { method: first ? "PUT" : "POST", body: JSON.stringify(d) });
  await send(message(false, true), true);
  await page.goto(`${PORT_URL}/?OVERLAY_WS=ws://127.0.0.1:9090/${id}/ws`);
  await expect(page.locator(".noticeBody")).toHaveCount(0);
  const mainDisplay = () => page.evaluate(() => { const e = document.querySelector(".mainBody"); return e ? getComputedStyle(e).display : "none"; });
  await expect.poll(mainDisplay, { timeout: 6000 }).toBe("none"); // standby after ~2 s
  await send(message(true, false));
  await expect.poll(() => page.evaluate(() => document.querySelectorAll("#DPSBody tr").length), { timeout: 5000 }).toBe(3);
  await expect.poll(mainDisplay).toBe("block");
  expect(errors).toEqual([]);
});
