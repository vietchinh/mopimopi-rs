// Tests the Capture and End-encounter buttons in a REAL browser (headless Chrome), against a fake OverlayPlugin server that has
// both endpoints of the real one: /ws (subscribe -> CombatData) and /MiniParse (the legacy handler that implements RequestEnd).
//
//   npm i puppeteer-core @sparticuz/chromium       (a Chromium binary that runs on Linux without installing anything)
//   node chrome-buttons-test.mjs [path/to/dist] [output folder for the screenshot]
//   BASE_PATH=/mopimopi-rs node chrome-buttons-test.mjs <dx bundle output>/public     (a site built with a base_path)
import puppeteer from "puppeteer-core";
import chromium from "@sparticuz/chromium";
import { WebSocketServer } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.argv[2] || path.join(here, "..", "dist"));
const outDir = path.resolve(process.argv[3] || "/tmp/chrome-buttons");
const basePath = process.env.BASE_PATH || "";          // the folder the site is served from, as on GitHub Pages
fs.mkdirSync(outDir, { recursive: true });
const sampleFight = JSON.parse(fs.readFileSync(path.join(here, "..", "src", "data", "previewLog.json"), "utf8"));
sampleFight.Encounter.CurrentZoneName = "Shirogane";
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
let failures = 0;
const check = (label, ok, extra = "") => { console.log(`  ${ok ? "PASS" : "FAIL"}  ${label}${extra ? "  " + extra : ""}`); if (!ok) failures++; };

// ---- the built site --------------------------------------------------------------------------------------------------
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".png": "image/png", ".woff": "font/woff", ".ttf": "font/ttf", ".json": "application/json", ".svg": "image/svg+xml" };
const site = http.createServer((req, res) => {
  let pathname = decodeURIComponent(new URL(req.url, "http://x").pathname);
  if (basePath && pathname.startsWith(basePath)) pathname = pathname.slice(basePath.length) || "/";
  const file = path.join(dist, pathname.replace(/^\/+$/, "/index.html"));
  fs.readFile(fs.existsSync(file) && fs.statSync(file).isFile() ? file : path.join(dist, "index.html"), (error, data) => {
    if (error) { res.statusCode = 404; return res.end(); }
    res.setHeader("content-type", TYPES[path.extname(file)] || "text/html"); res.end(data);
  });
});
await new Promise((r) => site.listen(18900, "127.0.0.1", r));

// ---- a fake OverlayPlugin WSServer: /ws and (optionally) /MiniParse ------------------------------------------------------
function startServer(port, { legacy = true } = {}) {
  return new Promise((resolve) => {
    const legacyMessages = [];
    const server = http.createServer((req, res) => { res.statusCode = 400; res.end(); });
    const main = new WebSocketServer({ noServer: true }), old = new WebSocketServer({ noServer: true });
    main.on("connection", (socket) => socket.on("message", (data) => {
      if (JSON.parse(data.toString()).call !== "subscribe") return;
      socket.send(JSON.stringify({ type: "ChangePrimaryPlayer", charName: "Eos Fair" }));
      socket.send(JSON.stringify({ type: "CombatData", ...sampleFight, isActive: "true" }));
    }));
    old.on("connection", (socket) => {                 // like OverlayPlugin's LegacyHandler: greets, then handles msgtype requests
      socket.send(JSON.stringify({ type: "broadcast", msgtype: "SendCharName", msg: { charName: "Eos Fair" } }));
      socket.on("message", (data) => legacyMessages.push(JSON.parse(data.toString())));
    });
    server.on("upgrade", (request, socket, head) => {
      const target = request.url === "/ws" ? main : request.url === "/MiniParse" && legacy ? old : null;
      if (target) target.handleUpgrade(request, socket, head, (ws) => target.emit("connection", ws, request)); else socket.destroy();
    });
    server.listen(port, "127.0.0.1", () => resolve({ legacyMessages, stop: () => new Promise((r) => { main.clients.forEach((c) => c.terminate()); old.clients.forEach((c) => c.terminate()); server.close(r); }) }));
  });
}

// ---- the browser -----------------------------------------------------------------------------------------------------
const browser = await puppeteer.launch({ args: [...chromium.args, "--no-sandbox"], executablePath: await chromium.executablePath(), headless: "shell" });
async function openApp(wsPort, downloadDir) {
  const page = await browser.newPage();
  await page.setViewport({ width: 900, height: 620 });
  const client = await page.createCDPSession();
  await client.send("Browser.setDownloadBehavior", { behavior: "allow", downloadPath: downloadDir });
  await page.goto(`http://127.0.0.1:18900${basePath}/?OVERLAY_WS=ws://127.0.0.1:${wsPort}/ws`);
  await page.waitForSelector("#DPSBody .tableWrap", { timeout: 15000 });
  return page;
}
async function pressButton(page, name) {
  await page.hover("[name=More]");                     // the hidden buttons appear while the mouse is over the menu button
  await page.waitForSelector(`[name=${name}]`, { timeout: 4000 });
  await page.click(`[name=${name}]`);
}
const toastText = (page) => page.evaluate(() => document.querySelector(".toast")?.textContent ?? "");
async function until(fn, ms = 10000) { const t = Date.now(); while (Date.now() - t < ms) { const v = await fn(); if (v) return v; await wait(100); } return null; }

// 1 -----------------------------------------------------------------------------------------------------------------------
console.log("1. End encounter reaches OverlayPlugin's legacy endpoint");
{
  const server = await startServer(18901);
  const page = await openApp(18901, outDir);
  await pressButton(page, "RequestEnd");
  const request = await until(() => server.legacyMessages.find((m) => m.msgtype === "RequestEnd"), 6000);
  check("the server received a RequestEnd message on /MiniParse", !!request, JSON.stringify(request));
  check("the message has both `type` and `msgtype` (the legacy handler needs both)", !!request && request.type === "overlayAPI" && request.msgtype === "RequestEnd");
  check("a toast says the request was sent", !!(await until(async () => (await toastText(page)).includes("Asked ACT to end the encounter"), 4000)), await toastText(page));
  await page.close(); await server.stop();
}

// 2 -----------------------------------------------------------------------------------------------------------------------
console.log("2. End encounter says so when the legacy endpoint is not there");
{
  const server = await startServer(18902, { legacy: false });
  const page = await openApp(18902, outDir);
  await pressButton(page, "RequestEnd");
  check("a toast says OverlayPlugin could not be reached", !!(await until(async () => (await toastText(page)).includes("Could not reach OverlayPlugin"), 8000)), await toastText(page));
  await page.close(); await server.stop();
}

// 3 -----------------------------------------------------------------------------------------------------------------------
console.log("3. Capture downloads a PNG of the overlay");
{
  const server = await startServer(18903);
  const downloads = path.join(outDir, "downloads"); fs.rmSync(downloads, { recursive: true, force: true }); fs.mkdirSync(downloads);
  const page = await openApp(18903, downloads);
  await page.screenshot({ path: path.join(outDir, "page.png") });          // what the page looks like, for comparison
  await pressButton(page, "Capture");
  const file = await until(() => fs.readdirSync(downloads).find((f) => f.endsWith(".png")), 15000);
  check("a .png file was downloaded", !!file, file || "");
  if (file) {
    const bytes = fs.readFileSync(path.join(downloads, file));
    const isPng = bytes.subarray(0, 8).toString("hex") === "89504e470d0a1a0a";
    const [w, h] = [bytes.readUInt32BE(16), bytes.readUInt32BE(20)];
    check("it is a real PNG", isPng, `${bytes.length} bytes`);
    check("its size is sensible (as wide as the window, shorter than it)", w >= 800 && h >= 40 && h <= 620 * 2, `${w}x${h}`);
    check("the file name says what it is", /^MopiMopi_.*\.png$/.test(file), file);
    fs.copyFileSync(path.join(downloads, file), path.join(outDir, "capture.png"));
  }
  check("a toast says the screenshot was downloaded", !!(await until(async () => (await toastText(page)).includes("screenshot was downloaded"), 6000)), await toastText(page));
  await page.close(); await server.stop();
}

await browser.close(); site.close();
console.log(failures === 0 ? "\nALL PASSED" : `\n${failures} CHECK(S) FAILED`);
process.exit(failures === 0 ? 0 : 1);
