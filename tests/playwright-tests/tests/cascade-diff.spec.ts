// Compares the computed style of EVERY element, on every screen, between two builds of the port. Meant for changes to how the CSS
// is put together (imports, cascade layers, moving rules between files), where the rules stay the same but which one wins can change
// silently: a pixel test only sees what happens to be visible, this sees every property of every element.
//
//   CASCADE_BEFORE_URL=http://127.0.0.1:8080 CASCADE_AFTER_URL=http://127.0.0.1:8083 npx playwright test cascade-diff
//
// Both are addresses of a built port (a `dist/` served as it is), so the wasm is the same and only the CSS differs. Skipped without them.
import fs from "node:fs";
import type { BrowserContext, Page } from "@playwright/test";
import { test, expect } from "../support/fixtures";
import { makeData } from "../support/data";
import { settingsFor, type Variant } from "../support/settings";
import { PROFILES, loadProfile } from "../support/profiles";
import { SUPPORT_URL, WS_URL } from "../support/urls";
import { openSettings, runSteps } from "../support/steps";

const BEFORE = process.env.CASCADE_BEFORE_URL, AFTER = process.env.CASCADE_AFTER_URL;
const path = (...steps: string[]) => steps.map((s) => (/^(tab|input|type|mouse|drag|dropdown|range|js):/.test(s) ? s : `li#${s}`));
const TALL = { sizeDPSTable: 25, sizeHPSTable: 25 };

type Scenario = { name: string; variant?: Partial<Variant>; /** a settings profile from tests/fixtures/settings, instead of `variant` */ profile?: string; noData?: boolean; after?: (page: Page, push: (data: ReturnType<typeof makeData>) => Promise<void>) => Promise<void> };
const settingsPage = (...steps: string[]): Scenario => ({ name: `settings ${steps.join(" > ") || "(top)"}`, after: async (page) => { await openSettings(page); await runSteps(page, path(...steps)); } });
const SCENARIOS: Scenario[] = [
  { name: "main, every row visible", variant: { Range: TALL } },
  { name: "main, raid mode", variant: { q: { view24_Number: 6 }, Range: TALL } },
  { name: "main, gradient bars on the right", variant: { q: { gradient: 1, bar_position: "right", bar_position_DPS: "right" }, Range: TALL } },
  { name: "start screen", noData: true },
  // The history list: a fight is seen running and then ends, which adds it to the list, and the History button (pinned) opens it. "Body
  // italic" and "Header italic" are on, because the list does not follow them and a shared text rule must not change that.
  { name: "history screen", variant: { q: { btn_History: 1, body_italic: 1, header_italic: 1, borderTextType: "outline" } }, after: async (page, push) => {
      await push({ ...makeData(), isActive: "false" });
      await page.waitForTimeout(600);
      await page.locator("nav[name=main] div[name=History]").click();
      await page.locator("#HISTORYBody .tableWrap").first().waitFor();
    } },
  { name: "nav menu open", after: async (page) => { await page.locator("nav[name=main] div[name=More]").click(); await page.waitForTimeout(400); } },
  { name: "tooltip", variant: { q: { tooltips: 1 } }, after: async (page) => { await page.locator("nav[name=main] div[name=More]").hover(); await page.locator("#tooltip").waitFor({ state: "visible" }); } },
  // the edge-case profiles: applyScope 1/2/3, bold and italic, every gradient direction, raid mode, pets merged and not, outline and shadow text
  ...PROFILES.map((profile): Scenario => ({ name: `profile ${profile}`, profile })),
  settingsPage(), settingsPage("Data"), settingsPage("Data", "format"), settingsPage("Data", "order"), settingsPage("Data", "tab:tab_mhh", "abbset"),
  settingsPage("Design"), settingsPage("Design", "font"), settingsPage("Design", "color"), settingsPage("Design", "opacity"), settingsPage("Design", "size"),
  settingsPage("Design", "cells", "tab:tab_align"), settingsPage("Design", "shape"), settingsPage("Design", "advanced"), settingsPage("Design", "raid"),
  settingsPage("Overlay"), settingsPage("Tool"), settingsPage("Tool", "custom"),
  // The sample tables above some settings pages (here: Color) can be switched to raid mode; the switch's label then takes the accent colour.
  { name: "settings Design > color, raid preview on", after: async (page) => { await openSettings(page); await runSteps(page, path("Design", "color")); await page.locator("#preview24").click({ timeout: 10_000 }); await page.waitForTimeout(400); } },
  { name: "settings Design > color, picker open", after: async (page) => { await openSettings(page); await runSteps(page, path("Design", "color", "input:navBg")); } },
];

/** Every element's computed style: `{ properties, rows: { "<path to the element>": [value per property] } }`. */
const capture = (page: Page) => page.evaluate(() => {
  const rows: Record<string, string[]> = {};
  let properties: string[] = [];
  const pathOf = (element: Element) => {
    const parts: string[] = [];
    for (let e: Element | null = element; e; e = e.parentElement) parts.push(`${e.tagName.toLowerCase()}${e.id ? "#" + e.id : ""}[${e.parentElement ? [...e.parentElement.children].indexOf(e) : 0}]`);
    return parts.reverse().join(">");
  };
  for (const element of document.querySelectorAll("html, body, body *")) {
    if (["SCRIPT", "STYLE", "LINK", "NOSCRIPT"].includes(element.tagName)) continue;
    const style = getComputedStyle(element);
    // `-webkit-locale` comes from the page's `lang` attribute (`dx`'s generated page has none, build.sh's has `en`), not from any CSS
    if (!properties.length) properties = Array.from({ length: style.length }, (_, i) => style[i]).filter((property) => property !== "-webkit-locale");
    // (absolute URLs carry the address the build is served from, and under `dx serve` its base path; neither is part of the comparison)
    rows[pathOf(element)] = properties.map((property) => style.getPropertyValue(property).split(location.origin).join("").split("/mopimopi-rs").join(""));
  }
  return { properties, rows };
});

async function open(context: BrowserContext, base: string, scenario: Scenario, session: string): Promise<Page> {
  const page = await context.newPage();
  await page.setViewportSize({ width: 1100, height: 640 });
  await page.route(/^https?:\/\/(fonts\.|ngld\.|ajax\.)/, (route) => route.fulfill({ contentType: /\.js$/.test(route.request().url()) ? "text/javascript" : "text/css", body: "" }));
  await page.addInitScript((blob) => localStorage.setItem("Mopi2_HAERU", blob), JSON.stringify(scenario.profile ? loadProfile(scenario.profile) : settingsFor(scenario.variant ?? {})));
  if (!scenario.noData) await fetch(`${SUPPORT_URL}/__session/${session}`, { method: "PUT", body: JSON.stringify({ type: "CombatData", ...makeData() }) });
  await page.goto(`${base}/?OVERLAY_WS=${WS_URL}/${session}/ws`);
  await page.locator("#wrap, nav").first().waitFor();
  await page.waitForTimeout(1200);
  await scenario.after?.(page, async (data) => { await fetch(`${SUPPORT_URL}/__push/${session}`, { method: "POST", body: JSON.stringify({ type: "CombatData", ...data }) }); });
  await page.waitForTimeout(900); // transitions (`html{transition:.3s}`) finish, so a value is never caught mid-way
  return page;
}

test.describe("computed style of every element, before and after", () => {
  test.skip(!BEFORE || !AFTER, "set CASCADE_BEFORE_URL and CASCADE_AFTER_URL to the two builds");
  test.setTimeout(240_000);
  const reportDir = process.env.CASCADE_REPORT_DIR ?? "/tmp/cascade";
  fs.mkdirSync(reportDir, { recursive: true });

  for (const scenario of SCENARIOS) {
    test(scenario.name, async ({ context }, info) => {
      const id = info.testId.replace(/\W/g, "");
      const [a, b] = [await open(context, BEFORE!, scenario, `${id}a`), await open(context, AFTER!, scenario, `${id}b`)];
      const [before, after] = [await capture(a), await capture(b)];
      expect(after.properties).toEqual(before.properties);
      const differences: { element: string; property: string; before: string; after: string }[] = [];
      const missing = [...Object.keys(before.rows).filter((k) => !(k in after.rows)), ...Object.keys(after.rows).filter((k) => !(k in before.rows))];
      for (const [element, values] of Object.entries(before.rows)) {
        const other = after.rows[element];
        if (!other) continue;
        values.forEach((value, i) => { if (value !== other[i]) differences.push({ element, property: before.properties[i], before: value, after: other[i] }); });
      }
      // one file per scenario: a failing test restarts the worker, so nothing can be collected in memory across tests
      fs.writeFileSync(`${reportDir}/${scenario.name.replace(/\W+/g, "_")}.json`, JSON.stringify({ scenario: scenario.name, elements: Object.keys(before.rows).length, differences, missing }, null, 1));
      info.annotations.push({ type: "compared", description: `${Object.keys(before.rows).length} elements x ${before.properties.length} properties, ${differences.length} difference(s)` });
      await a.close(); await b.close();
      expect(missing, "elements present in only one build").toEqual([]);
      expect(differences.slice(0, 5), `${differences.length} computed-style difference(s), the first five`).toEqual([]);
    });
  }
});
