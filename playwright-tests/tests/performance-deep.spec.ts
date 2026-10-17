// In-depth cost of live updates, for the original and the port, under a raid-sized load. Writes one JSON line per
// scenario to $PERF_OUT. Run: PERF_LABEL=new PERF_APPS=original,port PERF_OUT=/tmp/x.jsonl npx playwright test performance-deep
import fs from "node:fs";
import type { Page } from "@playwright/test";
import { test, MODE, type Handle } from "../support/fixtures";
import { makeData, ROSTER, type RosterEntry } from "../support/data";

const APPS = (process.env.PERF_APPS ?? "port").split(",");
const OUT = process.env.PERF_OUT ?? "/tmp/perf.jsonl";
const LABEL = process.env.PERF_LABEL ?? "run";

const JOBS = ["Nin", "Whm", "Mch", "War", "Blm", "Drk", "Ast", "Rdm", "Sch", "Pld", "Drg", "Brd", "Smn", "Mnk", "Sam", "Gnb"];
/** 24 players (one is YOU) plus a pet or a chocobo for some, a trust NPC and the Limit Break. */
function raid(step: number, title = "Striking Dummy", active = true): any {
  const roster: RosterEntry[] = [];
  for (let i = 0; i < 24; i++) {
    const name = i === 0 ? "YOU" : `Player ${String(i).padStart(2, "0")}`;
    const dmg = Math.round(60000 * (1 + 0.5 * Math.sin(step * 1.7 + i)) + i * 900);
    roster.push([name, i === 0 ? "Sch" : JOBS[i % JOBS.length], dmg, i % 4 === 0 ? 30000 + i * 700 : 0, 34]);
    if (i % 3 === 0) roster.push([`Eos (${name})`, "", Math.round(dmg / 6), 4000, 30]);
    if (i % 8 === 5) roster.push([`Chocobo (${name})`, "0", 900, 0, 20]);
  }
  roster.push(["Aw'aern", "", 20000, 0, 30], ["Limit Break", "", 15000, 0, 4]);
  const data = makeData({ roster, encounter: { title } });
  data.isActive = active ? "true" : "false";
  return data;
}

async function metrics(page: Page, cdp: any) {
  const all = Object.fromEntries((await cdp.send("Performance.getMetrics")).metrics.map((m: any) => [m.name, m.value]));
  return all as Record<string, number>;
}

async function startProbes(page: Page) {
  await page.evaluate(() => {
    const w = window as any;
    w.__gaps = []; w.__long = 0; w.__longMs = 0;
    let last = performance.now();
    const frame = (t: number) => { w.__gaps.push(t - last); last = t; requestAnimationFrame(frame); };
    requestAnimationFrame(frame);
    try { new PerformanceObserver((l) => { for (const e of l.getEntries()) { w.__long++; w.__longMs += e.duration; } }).observe({ entryTypes: ["longtask"] }); } catch {}
  });
}

async function run(name: string, app: "original" | "port", variant: any, body: (page: Page, handle: Handle) => Promise<number>) {
  test(`${name} [${app}]`, async ({ app: harness }) => {
    test.skip(!APPS.includes(app) || (app === "original" && MODE !== "live"), "not selected");
    test.setTimeout(240_000);
    const spec = { variant, data: raid(0) };
    const { page, handle } = app === "original" ? await harness.openOriginal(spec) : await harness.openPort(spec);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Performance.enable");
    await page.waitForTimeout(2000);
    await startProbes(page);
    await page.waitForTimeout(300);
    const before = await metrics(page, cdp);
    const wallStart = Date.now();
    const updates = await body(page, handle);
    // let queued work drain: wait for the main thread to be quiet for 400 ms
    let last = (await metrics(page, cdp)).TaskDuration;
    for (let i = 0; i < 40; i++) { await page.waitForTimeout(400); const now = (await metrics(page, cdp)).TaskDuration; if (now - last < 0.004) break; last = now; }
    const wallMs = Date.now() - wallStart;
    const after = await metrics(page, cdp);
    const probe = await page.evaluate(() => { const w = window as any; const g = [...w.__gaps].sort((a: number, b: number) => a - b); const q = (p: number) => g[Math.min(g.length - 1, Math.floor(g.length * p))] ?? 0; return { frames: g.length, p50: q(0.5), p95: q(0.95), p99: q(0.99), max: g[g.length - 1] ?? 0, longTasks: w.__long, longTaskMs: w.__longMs }; });
    const d = (k: string) => after[k] - before[k];
    const row = {
      label: LABEL, app, scenario: name, updates, wallMs,
      layouts: d("LayoutCount"), styleRecalcs: d("RecalcStyleCount"), layoutMs: d("LayoutDuration") * 1000, styleMs: d("RecalcStyleDuration") * 1000,
      scriptMs: d("ScriptDuration") * 1000, taskMs: d("TaskDuration") * 1000, taskMsPerUpdate: updates ? (d("TaskDuration") * 1000) / updates : 0,
      heapBeforeMB: before.JSHeapUsedSize / 1048576, heapAfterMB: after.JSHeapUsedSize / 1048576, nodesBefore: before.Nodes, nodesAfter: after.Nodes, listeners: after.JSEventListeners,
      ...probe,
    };
    fs.appendFileSync(OUT, JSON.stringify(row) + "\n");
    test.info().annotations.push({ type: "perf", description: JSON.stringify(row) });
  });
}

for (const app of ["original", "port"] as const) {
  run("steady 15/s, 150 updates, animation on", app, { q: { ani: 1 } }, async (page, handle) => {
    for (let s = 1; s <= 150; s++) { await handle.send(raid(s)); await page.waitForTimeout(66); }
    return 150;
  });
  run("steady 15/s, 150 updates, animation off", app, { q: { ani: 0 } }, async (page, handle) => {
    for (let s = 1; s <= 150; s++) { await handle.send(raid(s)); await page.waitForTimeout(66); }
    return 150;
  });
  run("pets kept separate, 150 updates", app, { q: { ani: 0, pets: 0 } }, async (page, handle) => {
    for (let s = 1; s <= 150; s++) { await handle.send(raid(s)); await page.waitForTimeout(66); }
    return 150;
  });
  run("burst, 300 updates back to back", app, { q: { ani: 0 } }, async (_page, handle) => {
    for (let s = 1; s <= 300; s++) await handle.send(raid(s));
    return 300;
  });
  run("40 fights end and restart (history growth)", app, { q: { ani: 0 } }, async (page, handle) => {
    let n = 0;
    for (let f = 1; f <= 40; f++) {
      for (let s = 0; s < 3; s++) { await handle.send(raid(f * 7 + s, `Fight ${f}`, true)); n++; await page.waitForTimeout(40); }
      await handle.send(raid(f * 7 + 3, `Fight ${f}`, false)); n++; await page.waitForTimeout(40);
    }
    return n;
  });
}
