// The graph bars: they must animate on the compositor thread, and come to rest exactly where the original's do.
import { test, expect } from "../support/fixtures";
import { shiftingData } from "../support/data";

const TALL = { sizeDPSTable: 25, sizeHPSTable: 25 };

test("every bar animation runs on the compositor (no 'animation not composited' failures)", async ({ app, browser }) => {
  const { page, handle } = await app.openPort({ variant: { q: { ani: 1 }, Range: TALL }, data: shiftingData(0) });
  await page.waitForTimeout(1500); // let the entry animations finish
  // Record Chrome's own verdicts: each Animation event gets a `compositeFailed` code (0 = it ran on the compositor).
  await browser.startTracing(page, { screenshots: false, categories: ["blink.animations", "devtools.timeline", "disabled-by-default-devtools.timeline"] });
  for (let step = 1; step <= 6; step++) { await handle.send(shiftingData(step)); await page.waitForTimeout(700); }
  const trace = JSON.parse((await browser.stopTracing()).toString());

  type Ev = { name: string; ph: string; id2?: { local: string }; args?: { data?: { nodeName?: string; displayName?: string; name?: string; compositeFailed?: number } } };
  const events: Ev[] = trace.traceEvents ?? trace;
  const described = new Map<string, string>(); // animation id -> "<property or animation name> on <element>"
  for (const e of events) if (e.name === "Animation" && e.ph === "b" && e.id2) described.set(e.id2.local, `${e.args?.data?.displayName || e.args?.data?.name} on ${e.args?.data?.nodeName}`);
  const verdicts = events.filter((e) => e.name === "Animation" && e.ph === "n" && e.args?.data && "compositeFailed" in e.args.data && e.id2);
  const bars = verdicts.filter((e) => /chrome-bar/.test(described.get(e.id2!.local) ?? ""));
  const failed = bars.filter((e) => e.args!.data!.compositeFailed !== 0);
  test.info().annotations.push({ type: "bar animations", description: `${bars.length} traced, ${failed.length} not composited` });

  expect(bars.length, "the trace should contain the bars' animations").toBeGreaterThan(10);
  const reasons = [...new Set(failed.map((e) => `${described.get(e.id2!.local)} (compositeFailed ${e.args!.data!.compositeFailed})`))];
  expect(failed.length, `not composited: ${reasons.join("; ")}`).toBe(0);
});

// After the animations finish the bars must be exactly where the original's are. Two updates, so bars that move (the
// small pet / shield / overheal bars sit side by side) settle from a real previous position, on either side.
const RESTING: [name: string, q: Record<string, unknown>][] = [
  ["small bars on the left", { bar_position: "left", bar_position_DPS: "left" }],
  ["small bars on the right", { bar_position: "right", bar_position_DPS: "right" }],
  ["small bars on the right, gradient", { bar_position: "right", bar_position_DPS: "left", gradient: 1 }],
];
for (const [name, q] of RESTING) {
  test(`after animating, the bars rest exactly like the original's: ${name}`, async ({ app }) => {
    await app.matchScreenshot(`bars/rest-${name}`, {
      variant: { q: { ani: 1, ...q }, Range: TALL },
      data: shiftingData(0),
      after: async (page, handle) => {
        for (const step of [1, 2]) { await handle.send(shiftingData(step)); await page.waitForTimeout(1000); }
      },
    });
  });
}
