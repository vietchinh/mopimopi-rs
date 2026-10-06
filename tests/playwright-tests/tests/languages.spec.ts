// Every language against the original: the same texts, on the settings pages, the start screen, the tooltips and the
// names the domain layer supplies, and the same texts again after switching language while the app is running.
import type { Page } from "@playwright/test";
import { test } from "../support/fixtures";
import { openSettings, runSteps } from "../support/steps";
import { radioEntries } from "../support/settings";

const LANGUAGES = ["KR", "JP", "EN", "FR", "DE", "CN"];
const path = (...steps: string[]) => steps.map((s) => (/^(tab|input|type|mouse|drag|dropdown|range|js):/.test(s) ? s : `li#${s}`));
const rows = (page: Page) => page.locator(".scrollArea li").allInnerTexts();

const PAGES: string[][] = [[], ["Data"], ["Data", "tab:tab_mhh", "abbset"], ["Design", "font"], ["Design", "cells"], ["Design", "color", "tab:tab_graph"], ["Overlay"], ["Tool"], ["Tool", "custom"]];

for (const lang of LANGUAGES) {
  test.describe(lang, () => {
    for (const steps of PAGES) {
      test(`settings page ${steps.join(" > ") || "(top)"}`, async ({ app }) => {
        await app.matchValue(`languages/${lang}/page-${steps.join("-") || "top"}`, {
          variant: { q: { Lang: lang } },
          after: async (page) => { await openSettings(page); await runSteps(page, path(...steps)); },
        }, rows);
      });
    }

    test("start screen (before any data arrives)", async ({ app }) => {
      await app.matchValue(`languages/${lang}/start-screen`, { variant: { q: { Lang: lang } }, noData: true },
        (page) => page.locator("body").innerText());
    });

    test("start screen looks the same", async ({ app }) => {
      await app.matchScreenshot(`languages/${lang}/start-screen-picture`, { variant: { q: { Lang: lang } }, noData: true });
    });

    test("hover tooltip of the ⋮ button", async ({ app }) => {
      await app.matchValue(`languages/${lang}/tooltip`, { variant: { q: { Lang: lang, tooltips: 1 } } }, async (page) => {
        await page.locator("nav[name=main] div[name=More]").hover();
        await page.locator("#tooltip").waitFor({ state: "visible" });
        return page.locator("#tooltip").innerText();
      });
    });

    test("the Limit Break row's name with names hidden (a text the domain layer needs)", async ({ app }) => {
      await app.matchValue(`languages/${lang}/limit-break-name`, { variant: { q: { Lang: lang, hideName: 1, DPS_T: 1 }, Range: { sizeDPSTable: 25 } } },
        (page) => page.locator("#DPSBody").innerText());
    });
  });
}

test("switching language while the app is running changes every text at once", async ({ app }) => {
  const order = Object.keys(radioEntries.get("Lang")!.m); // the order the options are listed in
  await app.matchValue("languages/switch-at-runtime", { after: openSettings }, async (page) => {
    const seen: Record<string, string[]> = { start: await rows(page) };
    for (const lang of ["KR", "CN", "FR", "EN"]) {
      await page.locator("li#Lang").first().evaluate((row: HTMLElement) => row.click()); // (a real click is blocked by the closing menu)
      await page.locator(`.dropdown li:nth-child(${order.indexOf(lang) + 1})`).click();
      await page.waitForTimeout(300);
      seen[lang] = await rows(page);
    }
    return seen;
  });
});

test("start screen: the intro and tip stay put and the update log fills the rest of the window", async ({ app }) => {
  await app.matchValue("start-screen/geometry", { noData: true }, (page) =>
    page.evaluate(() => Object.fromEntries(["#strong", "#tip", "#update"].map((selector) => {
      const element = document.querySelector(selector) as HTMLElement, box = element.getBoundingClientRect(), style = getComputedStyle(element);
      // whole pixels: the original sizes the log from whole-pixel offsetHeights (398px here); flex layout gives 398.1
      return [selector, { top: Math.round(box.top), height: Math.round(box.height), overflowY: style.overflowY }];
    }))));
});
