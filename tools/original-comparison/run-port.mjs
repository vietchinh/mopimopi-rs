// Runs THIS PORT in jsdom against a fake OverlayPlugin server and prints the DPS / HPS tables it draws for the sample
// fight, in the same JSON shape as run-original.mjs (so the two can be compared with compare-tables.mjs).
//
//   npm i jsdom ws
//   node run-port.mjs [path/to/dist]        (default ../../dist, build it first with ./build.sh)
import { JSDOM } from "jsdom";
import { WebSocketServer } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.argv[2] || path.join(here, "..", "..", "dist"));
// the fight to draw: a JSON file with Encounter / Combatant (default: the sample fight of this repository)
const fightFile = process.argv[3] || path.join(here, "..", "..", "src", "data", "previewLog.json");
const sampleFight = JSON.parse(fs.readFileSync(fightFile, "utf8"));
delete sampleFight.type;
sampleFight.Encounter.CurrentZoneName = "Shirogane";       // the original treats zone "HAERU" as its own preview data
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const PORT = 18821;

const server = http.createServer((req, res) => { res.statusCode = 400; res.end(); });
const wss = new WebSocketServer({ server, path: "/ws" });
wss.on("connection", (socket) => socket.on("message", () => {
  socket.send(JSON.stringify({ type: "ChangePrimaryPlayer", charName: "Eos Fair" }));
  socket.send(JSON.stringify({ type: "CombatData", ...sampleFight, isActive: "true" }));
}));
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

const dom = new JSDOM(`<!DOCTYPE html><html><body><div id="main"></div></body></html>`, { url: `http://127.0.0.1:8080/?OVERLAY_WS=ws://127.0.0.1:${PORT}/ws`, pretendToBeVisual: true });
const w = dom.window;
for (const k of Object.getOwnPropertyNames(w)) if (!(k in globalThis)) { try { globalThis[k] = w[k]; } catch {} }
globalThis.window = w; globalThis.document = w.document; globalThis.self = w;
Object.defineProperty(globalThis, "navigator", { value: w.navigator, configurable: true });
for (const k of ["WebSocket", "MessageEvent", "CustomEvent", "localStorage"]) Object.defineProperty(globalThis, k, { value: w[k], configurable: true, writable: true });
console.error = () => {}; console.warn = () => {};
const glue = await import(`${dist}/pkg/mopimopi-dioxus.js`);
await glue.default({ module_or_path: fs.readFileSync(`${dist}/pkg/mopimopi-dioxus_bg.wasm`) });

const start = Date.now();
while (Date.now() - start < 8000 && !w.document.querySelector("#DPSBody .tableWrap, #DPSBody .rCell")) await wait(50);
await wait(500);
const tables = {};
for (const id of ["DPS", "HPS"]) {
  tables[id] = [...w.document.querySelectorAll(`#${id}Body .tableWrap, #${id}Body .rCell`)].map((row) => ({
    id: row.id, cells: [...row.querySelectorAll("td")].map((td) => td.textContent.replace(/\s+/g, " ").trim())
  }));
}
process.stdout.write(JSON.stringify(tables, null, 2) + "\n");
process.exit(0);
