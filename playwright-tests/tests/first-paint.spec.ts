// The page must be styled from its very first paint: no flash of unstyled content while the wasm loads, and no
// jump ("bounce") once it does. Confirmed as a real bug against a DevTools trace: dx's own generated page (unlike
// the plain build.sh page, which has always had this) had no styling at all until the wasm booted and inserted
// the real stylesheets, so the browser first painted with its own defaults (an 8px <body> margin, a serif font)
// and then visibly snapped into place.
import { test, expect } from "../support/fixtures";
import { PORT_URL } from "../support/urls";

/** Like the `app` fixture: fonts, jQuery and OverlayPlugin's helper come from the internet in the real page; a run needs no network. */
async function withoutNetwork(page: import("@playwright/test").Page) {
  await page.route(/^https?:\/\/(fonts\.|ngld\.|ajax\.)/, (route) => route.fulfill({ contentType: /\.js$/.test(route.request().url()) ? "text/javascript" : "text/css", body: "" }));
}

test("the page has its critical CSS applied on the very first paint, before the wasm loads", async ({ page }) => {
  await withoutNetwork(page);
  // block the wasm bundle and its loader script entirely: whatever is on screen after that must already be styled
  await page.route("**/*.wasm", (route) => route.abort());
  await page.route("**/*mopimopi-dioxus*.js", (route) => route.abort());
  await page.goto(PORT_URL, { waitUntil: "domcontentloaded" });
  await expect(page.locator("body")).toHaveCSS("margin", "0px");
  await expect(page.locator("body")).toHaveCSS("background-color", "rgba(0, 0, 0, 0)"); // "transparent"
  // The base layout (public/base.css): the app's first frame is drawn with these already in place, or the page bounces on every refresh:
  // rem sizes come from html{font-size:62.5%} (16px would resize everything ~300ms later, `html{transition:.3s}`), and the body fills the window.
  await expect(page.locator("html")).toHaveCSS("font-size", "10px");
  await expect(page.locator("body")).toHaveCSS("height", `${page.viewportSize()!.height}px`);
});

test("loading the real page produces no layout shift", async ({ page, browser }) => {
  await withoutNetwork(page);
  // Chrome's own trace (the same events DevTools shows), not Playwright's trace archive: it carries the `LayoutShift` events
  await browser.startTracing(page, { screenshots: false, categories: ["loading", "devtools.timeline"] });
  await page.goto(PORT_URL);
  await page.waitForTimeout(2000);
  const trace = JSON.parse((await browser.stopTracing()).toString());
  const events: Array<{ name: string; args?: { data?: { score?: number } } }> = trace.traceEvents ?? trace;
  const shifts = events.filter((e) => e.name === "LayoutShift");
  const total = shifts.reduce((sum, e) => sum + (e.args?.data?.score ?? 0), 0);
  test.info().annotations.push({ type: "layout shift", description: `${shifts.length} shift(s), cumulative score ${total.toFixed(4)}` });
  // Measured: 0.0007 (2560x1300) and 0.0034 (1100x640) with base.css, against 0.066 and 0.14 without it. Chrome's own "good" threshold is 0.1.
  expect(total).toBeLessThan(0.02);
});
