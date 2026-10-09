// Cost of live updates: the port must not do more work than the original. Measured over five updates that
// move the bars (so they really animate), with a steady frame loop so the browser does its per-frame work.
import type { Page } from "@playwright/test";
import { test, expect, MODE, type Handle } from "../support/fixtures";
import { shiftingData } from "../support/data";

async function measure(page: Page, handle: Handle) {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Performance.enable");
  await page.waitForTimeout(1800);
  await page.evaluate(() => { const frame = () => requestAnimationFrame(frame); frame(); });
  await page.waitForTimeout(300);
  const metrics = async () => Object.fromEntries((await cdp.send("Performance.getMetrics")).metrics.map((m) => [m.name, m.value]));
  const before = await metrics();
  for (let step = 1; step <= 5; step++) { await handle.send(shiftingData(step)); await page.waitForTimeout(1000); }
  const after = await metrics();
  const delta = (name: string) => after[name] - before[name];
  return { layouts: delta("LayoutCount"), styleRecalcs: delta("RecalcStyleCount"), scriptMs: delta("ScriptDuration") * 1000, mainThreadMs: delta("TaskDuration") * 1000 };
}

const spec = { variant: { q: { ani: 1 } }, data: shiftingData(0) };

test("five animated updates stay within a fixed layout budget", async ({ app }) => {
  const { page, handle } = await app.openPort(spec);
  const cost = await measure(page, handle);
  test.info().annotations.push({ type: "cost", description: JSON.stringify(cost) });
  // measured at 127 layouts / 127 style recalculations; the ceiling leaves room for another machine
  expect(cost.layouts).toBeLessThan(300);
  expect(cost.styleRecalcs).toBeLessThan(300);
});

test("the port does no more than half the original's layout and script work", async ({ app }) => {
  test.skip(MODE !== "live", "compares against the running original, so it needs live mode");
  const original = await measure(...(await app.openOriginal(spec).then(({ page, handle }) => [page, handle] as const)));
  const port = await measure(...(await app.openPort(spec).then(({ page, handle }) => [page, handle] as const)));
  test.info().annotations.push({ type: "cost", description: JSON.stringify({ original, port }) });
  expect(port.layouts).toBeLessThan(original.layouts * 0.5);
  expect(port.scriptMs).toBeLessThan(original.scriptMs * 0.5);
});
