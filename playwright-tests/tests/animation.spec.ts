// Bars grow the way the original's jQuery animation grows them: same shape, same length (about 400 ms).
import type { Page } from "@playwright/test";
import { test, expect } from "../support/fixtures";
import { makeData, type RosterEntry } from "../support/data";

const roster = (bob: number): RosterEntry[] => [["YOU", "Sch", 100000, 0, 35], ["Bob", "War", bob, 0, 35]];

/** Samples Bob's bar width on every frame from the moment it first changes. */
const record = (page: Page) =>
  page.evaluate(() => {
    const w = window as any;
    w.__samples = [];
    let last: number | null = null, start: number | null = null;
    const tick = (t: number) => {
      const bar = document.querySelector("#DPSBody #Bob .bar");
      if (bar) {
        const width = bar.getBoundingClientRect().width;
        if (last === null) last = width;
        if (start === null && Math.abs(width - last) > 0.5) { start = t; w.__samples.push([0, last]); }
        if (start !== null && t - start < 900) w.__samples.push([Math.round(t - start), Math.round(width * 10) / 10]);
        if (start === null) last = width;
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });

/** every 25 ms, so the comparison can allow for how late each side's timer started */
const SAMPLE_TIMES = Array.from({ length: 19 }, (_, i) => 25 * i);

function summarize(samples: [number, number][]) {
  const [first, final] = [samples[0][1], samples[samples.length - 1][1]];
  const at = (ms: number) => samples.filter(([t]) => t <= ms).pop()![1];
  return {
    from: first,
    to: final,
    /** share of the change done after N ms, in percent */
    curve: Object.fromEntries(SAMPLE_TIMES.map((ms) => [ms, Math.round(((at(ms) - first) / (final - first)) * 100)])),
    settledAfterMs: samples.find(([, width]) => Math.abs(width - final) < 0.6)![0],
  };
}

test("a bar grows along the original's curve and finishes in about 400 ms", async ({ app }) => {
  await app.matchValue(
    "animation/bar-growth",
    { variant: { q: { ani: 1 } }, data: makeData({ roster: roster(40000) }) },
    async (page, handle) => {
      await page.waitForTimeout(1500); // let the entry animation finish
      await record(page);
      await handle.send(makeData({ roster: roster(90000) }));
      await page.waitForTimeout(1400);
      return summarize(await page.evaluate(() => (window as any).__samples));
    },
    (expected, actual) => {
      const row = (curve: Record<number, number>) => SAMPLE_TIMES.map((ms) => `${ms}ms:${curve[ms]}%`).join(" ");
      test.info().annotations.push({ type: "curve", description: `original ${row(expected.curve)} (settled ${expected.settledAfterMs} ms) | port ${row(actual.curve)} (settled ${actual.settledAfterMs} ms)` });
      // The endpoints agree to a fraction of a pixel: the port animates a transform, and a transformed element's
      // bounding box carries a little matrix rounding (439.9996 where the original, animating `width`, reads 440).
      expect(Math.abs(actual.from - expected.from)).toBeLessThan(0.5);
      expect(Math.abs(actual.to - expected.to)).toBeLessThan(0.5);
      // The two run on different timers (jQuery's, the browser's) that start a few milliseconds apart, and the original's
      // own curve wanders by a few points from run to run. So the port's progress at each moment must lie between the
      // original's progress 25 ms earlier and 25 ms later (a little slack for rounding): the same easing, up to start-up.
      for (const ms of SAMPLE_TIMES.filter((t) => t >= 50 && t <= 375)) {
        const [earlier, later] = [expected.curve[ms - 25], expected.curve[ms + 25]];
        expect(actual.curve[ms], `after ${ms} ms (the original: ${earlier}%..${later}%)`).toBeGreaterThanOrEqual(earlier - 3);
        expect(actual.curve[ms], `after ${ms} ms (the original: ${earlier}%..${later}%)`).toBeLessThanOrEqual(later + 3);
      }
      expect(Math.abs(actual.settledAfterMs - expected.settledAfterMs)).toBeLessThanOrEqual(60);
    },
  );
});
