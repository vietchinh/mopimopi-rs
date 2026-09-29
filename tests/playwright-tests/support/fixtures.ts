// The `app` fixture: opens the original overlay and the port with identical settings and data, and compares them.
//
// Three modes (E2E_MODE, or picked from whether MOPIMOPI_ORIGINAL_DIR is set):
//   live    render the original next to the port, every run                (default when the original is available)
//   record  render the original and save what it showed as goldens          (npm run goldens)
//   golden  compare the port with the saved goldens; no original needed    (default otherwise, e.g. CI)
import { test as base, expect, type BrowserContext, type Page, type TestInfo } from "@playwright/test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
import { makeData, ROSTER_WITHOUT_YOU, type CombatData } from "./data";
import { settingsFor, type Settings, type Variant } from "./settings";

import { ORIGINAL_URL, PORT_URL, SUPPORT_URL, WS_URL } from "./urls";
export { ORIGINAL_URL, PORT_URL, SUPPORT_URL, WS_URL };
export type Mode = "live" | "record" | "golden";
export const MODE: Mode = (process.env.E2E_MODE as Mode) ?? (process.env.MOPIMOPI_ORIGINAL_DIR ? "live" : "golden");
const GOLDENS = path.resolve(__dirname, "../goldens");
const JQUERY = fs.readFileSync(require.resolve("jquery/dist/jquery.min.js"));

export type Kind = "original" | "port";
export type Handle = { kind: Kind; /** Feeds the page a further combat-data update, as ACT would. */ send(data: CombatData): Promise<void> };
export type Spec = {
  variant?: Partial<Variant>;
  settings?: Settings;
  data?: CombatData;
  /** Runs after the data has arrived and before the screenshot; the same code drives both apps. */
  after?: (page: Page, handle: Handle) => Promise<void>;
};

/**
 * A file name for a test name. Every character that is not safe in a file name is written as `~<hex>~`, so two
 * different names can never share a file (`q.ds=,` and `q.ds=_` once collapsed into one golden).
 */
const slug = (name: string) => name.replace(/[^\w.=+-]/gu, (c) => `~${c.codePointAt(0)!.toString(16)}~`);
const goldenFile = (name: string, extension: "png" | "json" | "sha256") => path.join(GOLDENS, `${slug(name)}.${extension}`);
const STYLE_FOR_SCREENSHOTS = "html{background-color:#7a7a7a !important} *{caret-color:transparent !important}";

export class App {
  private counter = 0;
  constructor(readonly context: BrowserContext, private readonly info: TestInfo) {}

  private resolve(spec: Spec) {
    const variant = spec.variant ?? {};
    return {
      settings: spec.settings ?? settingsFor(variant),
      data: spec.data ?? makeData(variant.noYou ? { roster: ROSTER_WITHOUT_YOU } : {}),
      width: variant.width ?? 1100,
    };
  }

  /** The fonts, jQuery and OverlayPlugin helper the original pulls from the internet are replaced, so a run needs no network and looks the same everywhere. */
  private async stubExternalResources() {
    await this.context.route("**/*", async (route) => {
      const url = route.request().url();
      if (/ajax\.googleapis\.com.*jquery/.test(url)) return route.fulfill({ contentType: "text/javascript", body: JQUERY });
      if (/common\.min\.js/.test(url)) return route.fulfill({ contentType: "text/javascript", body: "/* OverlayPlugin helper not available here */" });
      if (/^https?:\/\/(fonts\.|ngld\.|ajax\.)/.test(url)) return route.fulfill({ contentType: "text/css", body: "" });
      return route.fallback();
    });
  }

  private async newPage(settings: Settings, width: number): Promise<Page> {
    if (this.counter++ === 0) await this.stubExternalResources();
    const page = await this.context.newPage();
    await page.setViewportSize({ width, height: 640 });
    await page.addInitScript((blob) => localStorage.setItem("Mopi2_HAERU", blob), JSON.stringify(settings));
    return page;
  }

  private sessionId(): string {
    return `${this.info.workerIndex}-${this.info.testId}-${this.counter}`.replace(/[^\w-]/g, "");
  }

  async openOriginal(spec: Spec = {}): Promise<{ page: Page; handle: Handle }> {
    const { settings, data, width } = this.resolve(spec);
    const page = await this.newPage(settings, width);
    // `HOST_PORT` points the original at a socket that isn't there; data is handed to it directly instead.
    await page.goto(`${ORIGINAL_URL}/index.html?HOST_PORT=127.0.0.1:1`);
    await page.waitForFunction(() => typeof (window as any).onBroadcastMessage === "function" && typeof (window as any).init !== "undefined");
    await page.waitForTimeout(300); // the original finishes its own start-up (jQuery ready handlers) a moment after load
    const send = (payload: CombatData) => page.evaluate((m) => (window as any).onBroadcastMessage({ detail: { msgtype: "CombatData", msg: m } }), payload);
    const handle: Handle = { kind: "original", send };
    await send(data);
    await spec.after?.(page, handle);
    return { page, handle };
  }

  async openPort(spec: Spec = {}): Promise<{ page: Page; handle: Handle }> {
    const { settings, data, width } = this.resolve(spec);
    const page = await this.newPage(settings, width);
    const session = this.sessionId();
    const put = (method: "PUT" | "POST", kind: "session" | "push", payload: CombatData) =>
      fetch(`${SUPPORT_URL}/__${kind}/${session}`, { method, body: JSON.stringify({ type: "CombatData", ...payload }) });
    await put("PUT", "session", data);
    // the support server plays OverlayPlugin: the page subscribes, and gets `data` back
    await page.goto(`${PORT_URL}/?OVERLAY_WS=${WS_URL}/${session}/ws`);
    await page.locator("#wrap, nav").first().waitFor();
    await page.waitForTimeout(900); // the port draws the first data a moment after its socket opens
    const handle: Handle = { kind: "port", send: async (payload) => void (await put("POST", "push", payload)) };
    await spec.after?.(page, handle);
    return { page, handle };
  }

  /** A screenshot that has stopped changing (two identical frames in a row), with a fixed background and no caret. */
  async shoot(page: Page, settleMs = 400): Promise<Buffer> {
    await page.waitForTimeout(settleMs);
    await page.addStyleTag({ content: STYLE_FOR_SCREENSHOTS });
    let previous = await page.screenshot({ type: "png" });
    for (let attempt = 0; attempt < 8; attempt++) {
      await page.waitForTimeout(120);
      const next = await page.screenshot({ type: "png" });
      if (previous.equals(next)) return next;
      previous = next;
    }
    return previous;
  }

  /** The original's screenshot for `spec`, rendered now. */
  private async renderOriginal(spec: Spec): Promise<Buffer> {
    const { page } = await this.openOriginal(spec);
    try { return await this.shoot(page); } finally { await page.close(); }
  }

  /**
   * A golden is the hash of the original's exact pixels (a few bytes; the comparison is exact, so nothing else is
   * needed). Set E2E_GOLDEN_IMAGES=1 while recording to keep the PNG too, which lets a golden-mode failure show a picture.
   */
  private recordGolden(name: string, png: Buffer) {
    fs.mkdirSync(GOLDENS, { recursive: true });
    fs.writeFileSync(goldenFile(name, "sha256"), pixelHash(png) + "\n");
    if (process.env.E2E_GOLDEN_IMAGES) fs.writeFileSync(goldenFile(name, "png"), png);
  }

  private readGoldenHash(name: string): string {
    const file = goldenFile(name, "sha256");
    if (!fs.existsSync(file)) throw new Error(`No golden for "${name}" (${path.relative(process.cwd(), file)}). Record them with: npm run goldens`);
    return fs.readFileSync(file, "utf8").trim();
  }

  /** The hash of what the original shows for `spec`: rendered now (live/record) or read from the goldens. */
  async referenceHash(name: string, spec: Spec): Promise<string> {
    if (MODE === "golden") return this.readGoldenHash(name);
    return pixelHash(await this.renderOriginal(spec));
  }

  /** The port must draw exactly what the original draws: not one pixel of difference. */
  async matchScreenshot(name: string, spec: Spec): Promise<void> {
    if (MODE === "record") { this.recordGolden(name, await this.renderOriginal(spec)); return; }
    const expected = MODE === "live" ? await this.renderOriginal(spec) : undefined;
    const { page } = await this.openPort(spec);
    let actual: Buffer;
    try { actual = await this.shoot(page); } finally { await page.close(); }
    if (expected) return expectPixelIdentical(this.info, name, expected, actual);
    // golden mode: a stored picture gives a pixel diff; otherwise the hashes are compared
    const picture = goldenFile(name, "png");
    if (fs.existsSync(picture)) return expectPixelIdentical(this.info, name, fs.readFileSync(picture), actual);
    await this.info.attach("port", { body: actual, contentType: "image/png" });
    expect(pixelHash(actual), `${name}: the port's pixels differ from the recorded original (rerun in live mode with E2E_ARTIFACTS_DIR=<folder> to see the difference)`).toBe(this.readGoldenHash(name));
  }

  /**
   * The port must produce the same values as the original for the same actions. `probe` drives the page (the same
   * code for both apps) and returns something JSON-friendly; it is run on the original (live/record) or read from the goldens.
   */
  async matchValue<T>(name: string, spec: Spec, probe: (page: Page, handle: Handle) => Promise<T>, agree: (expected: T, actual: T) => void = (e, a) => expect(a).toEqual(e)): Promise<void> {
    const file = goldenFile(name, "json");
    let expected: T;
    if (MODE === "golden") {
      if (!fs.existsSync(file)) throw new Error(`No golden for "${name}" (${path.relative(process.cwd(), file)}). Record them with: npm run goldens`);
      expected = JSON.parse(fs.readFileSync(file, "utf8"));
    } else {
      const { page, handle } = await this.openOriginal(spec);
      try { expected = await probe(page, handle); } finally { await page.close(); }
      if (MODE === "record") { fs.mkdirSync(GOLDENS, { recursive: true }); fs.writeFileSync(file, JSON.stringify(expected, null, 2)); return; }
    }
    const { page, handle } = await this.openPort(spec);
    let actual: T;
    try { actual = await probe(page, handle); } finally { await page.close(); }
    agree(expected, actual);
  }
}

/** A fingerprint of a screenshot's exact pixels (and its size). Equal hashes mean identical images. */
export function pixelHash(png: Buffer): string {
  const image = PNG.sync.read(png);
  return crypto.createHash("sha256").update(`${image.width}x${image.height}:`).update(image.data).digest("hex");
}

export function pixelDifference(expected: Buffer, actual: Buffer): { differing: number; sizeMismatch: boolean; diff?: Buffer } {
  const a = PNG.sync.read(expected);
  const b = PNG.sync.read(actual);
  if (a.width !== b.width || a.height !== b.height) return { differing: -1, sizeMismatch: true };
  const diff = new PNG({ width: a.width, height: a.height });
  // Exact by default: threshold 0 counts any pixel whose colour differs at all (pixelmatch's usual 0.1 would let
  // small colour differences through), and anti-aliased pixels are counted too. E2E_PIXEL_THRESHOLD loosens it.
  const threshold = Number(process.env.E2E_PIXEL_THRESHOLD ?? 0);
  const differing = pixelmatch(a.data, b.data, diff.data, a.width, a.height, { threshold, includeAA: true });
  return { differing, sizeMismatch: false, diff: differing ? PNG.sync.write(diff) : undefined };
}

export async function expectPixelIdentical(info: TestInfo, name: string, expected: Buffer, actual: Buffer): Promise<void> {
  const result = pixelDifference(expected, actual);
  if (result.differing !== 0) {
    // E2E_ARTIFACTS_DIR=/some/folder also writes the three images to disk (the line reporter keeps no attachments)
    const folder = process.env.E2E_ARTIFACTS_DIR;
    if (folder) {
      fs.mkdirSync(folder, { recursive: true });
      const base = path.join(folder, slug(name));
      fs.writeFileSync(`${base}.original.png`, expected);
      fs.writeFileSync(`${base}.port.png`, actual);
      if (result.diff) fs.writeFileSync(`${base}.difference.png`, result.diff);
    }
    await info.attach("original", { body: expected, contentType: "image/png" });
    await info.attach("port", { body: actual, contentType: "image/png" });
    if (result.diff) await info.attach("difference", { body: result.diff, contentType: "image/png" });
  }
  expect(result.sizeMismatch, `${name}: screenshots differ in size`).toBe(false);
  expect(result.differing, `${name}: ${result.differing} pixels differ from the original`).toBe(0);
}

export const test = base.extend<{ app: App }>({
  app: async ({ context }, use, info) => { await use(new App(context, info)); },
});
export { expect };
