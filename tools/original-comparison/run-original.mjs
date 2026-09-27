// Runs the ORIGINAL MopiMopi (the JavaScript of the upstream repository) in jsdom against a fake OverlayPlugin server, for
// comparison with this port.
//
//   npm i jsdom ws jquery
//   node run-original.mjs <path to the original repo checkout> [messages|table]
//
//   messages  what the original sends over the WebSocket (handshake, keep-alive answer, Capture / End requests)
//   table     the DPS / HPS tables it draws for a sample fight, as JSON on stdout (compare with run-port.mjs)
import { JSDOM } from "jsdom";
import { WebSocketServer } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { createRequire } from "module";
import { fileURLToPath } from "url";

const here = path.dirname(fileURLToPath(import.meta.url));
const originalRepo = path.resolve(process.argv[2]);
const mode = process.argv[3] || "messages";
const require = createRequire(import.meta.url);
const jquery = fs.readFileSync(path.join(path.dirname(require.resolve("jquery")), "jquery.min.js"), "utf8");
// the fight to draw: a JSON file with Encounter / Combatant (default: the sample fight of this repository)
const fightFile = process.argv[4] || path.join(here, "..", "..", "src", "data", "previewLog.json");
const sampleFight = JSON.parse(fs.readFileSync(fightFile, "utf8"));
delete sampleFight.type;
sampleFight.Encounter.CurrentZoneName = "Shirogane";       // the original treats zone "HAERU" as its own preview data
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const PORT = 18820;
const WINDOW_ID = "0f8fad5b-d9cb-469f-a165-70867728950e";

// ---- fake ACTWebSocket-style server (what the original expects: path /MiniParse, "broadcast" envelopes) -------------
const received = [];
const server = http.createServer((req, res) => { res.statusCode = 400; res.end(); });
const wss = new WebSocketServer({ server, path: "/MiniParse" });
let clientSocket;
wss.on("connection", (socket) => {
  clientSocket = socket;
  socket.on("message", (data) => received.push(data.toString()));
});
await new Promise((r) => server.listen(PORT, "127.0.0.1", r));

// ---- the original page --------------------------------------------------------------------------------------------
const read = (file) => fs.readFileSync(path.join(originalRepo, file), "utf8");
const html = read("index.html").replace(/<script[\s\S]*?<\/script>/g, "").replace(/<link[^>]*(googleapis|icon)[^>]*>/g, "");
const dom = new JSDOM(html, { url: `http://127.0.0.1:8080/?HOST_PORT=ws://127.0.0.1:${PORT}`, runScripts: "outside-only", pretendToBeVisual: true,
  beforeParse(window) { window.overlayWindowId = WINDOW_ID; } });          // OverlayPlugin defines this global inside its overlays
const w = dom.window;
w.console.error = () => {}; w.console.log = () => {};
const run = (code) => w.eval(code);
run(jquery); run(read("js/jscolor.js")); run(`var wsUri = "ws://@HOST_PORT@/MiniParse";`);
for (const file of ["dic.js", "lang.js", "init.js", "core.js", "process.js", "ui.js"]) run(read(`js/${file}`));
w.document.dispatchEvent(new w.Event("DOMContentLoaded"));
w.dispatchEvent(new w.Event("load"));

const broadcast = (msgtype, msg) => clientSocket.send(JSON.stringify({ type: "broadcast", msgtype, msg }));
async function waitFor(condition, ms = 6000) { const start = Date.now(); while (Date.now() - start < ms) { if (condition()) return true; await wait(50); } return false; }

await waitFor(() => clientSocket);
await wait(300);

if (mode === "messages") {
  clientSocket.send(".");                                           // MiniParse keep-alive
  await wait(200);
  broadcast("SendCharName", { charName: "Eos Fair" });
  broadcast("CombatData", { ...sampleFight, isActive: "true" });
  await waitFor(() => w.document.querySelector("#DPSBody .tableWrap, #DPSBody .rCell"));
  received.length = 0;                                              // only what the buttons send from here on
  clientSocket.send(".");
  await wait(200);
  try { run("button('Capture')"); } catch (e) { console.error(e); }
  await wait(1800);                                                 // Capture waits 1.3 s before it sends
  try { run("button('RequestEnd')"); } catch (e) { console.error(e); }
  await wait(300);
  process.stdout.write(JSON.stringify({ messagesReceivedByServer: received }, null, 2) + "\n");
} else {
  broadcast("SendCharName", { charName: "Eos Fair" });
  broadcast("CombatData", { ...sampleFight, isActive: "true" });
  await waitFor(() => w.document.querySelector("#DPSBody .tableWrap, #DPSBody .rCell"));
  await wait(500);
  // the finished fight is shown on the next inactive message in the original; wait for the rows of the running one
  const tables = {};
  for (const id of ["DPS", "HPS"]) {
    tables[id] = [...w.document.querySelectorAll(`#${id}Body .tableWrap, #${id}Body .rCell`)].map((row) => ({
      id: row.id, cells: [...row.querySelectorAll("td")].map((td) => td.textContent.replace(/\s+/g, " ").trim())
    }));
  }
  process.stdout.write(JSON.stringify(tables, null, 2) + "\n");
}
process.exit(0);
