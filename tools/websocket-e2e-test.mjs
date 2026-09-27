// End-to-end test of the OverlayPlugin socket: the built app runs in jsdom and talks to a real local WebSocket server that
// behaves like OverlayPlugin's WSServer (path /ws, answers a `subscribe` call with CombatData).
//
//   cd tools && npm i jsdom ws      (once, in any folder; run this file from a folder that can resolve both)
//   node websocket-e2e-test.mjs [path/to/dist]       (default: ../dist, build it first with ./build.sh)
//
// The address always comes from the page's URL (?OVERLAY_WS= or ?HOST_PORT=), as in the real app.
import { JSDOM } from "jsdom";
import { WebSocketServer } from "ws";
import http from "http";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(process.argv[2] || path.join(here, "..", "dist"));
const sampleFight = JSON.parse(fs.readFileSync(path.join(here, "..", "src", "data", "previewLog.json"), "utf8"));
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
let failures = 0;
const check = (label, ok, extra = "") => { console.log(`  ${ok ? "PASS" : "FAIL"}  ${label}${extra ? "  " + extra : ""}`); if (!ok) failures++; };

// ---- a fake OverlayPlugin WSServer ---------------------------------------------------------------------------------
function startServer(port, { sendDelayMs = 0, title = "Striking Dummy", withoutYou = false } = {}) {
  return new Promise((resolve) => {
    const state = { connections: 0, subscribes: 0, sockets: new Set() };
    const server = http.createServer((req, res) => { res.statusCode = 400; res.end(); });
    server.on("connection", () => state.connections++);
    const wss = new WebSocketServer({ server, path: "/ws" });
    wss.on("connection", (socket) => {
      state.sockets.add(socket);
      socket.on("close", () => state.sockets.delete(socket));
      socket.on("message", (data) => {
        const call = JSON.parse(data.toString());
        if (call.call !== "subscribe") return;
        state.subscribes++;
        setTimeout(() => {
          const combatants = Object.fromEntries(Object.entries(sampleFight.Combatant).filter(([, c]) => !(withoutYou && c.name === "YOU")));
          socket.send(JSON.stringify({ type: "ChangePrimaryPlayer", charName: "Eos Fair" }));
          socket.send(JSON.stringify({ type: "CombatData", ...sampleFight, Combatant: combatants, Encounter: { ...sampleFight.Encounter, title }, isActive: "true" }));
        }, sendDelayMs);
      });
    });
    server.listen(port, "127.0.0.1", () => resolve({ state, dropClients: () => state.sockets.forEach((s) => s.terminate()),
      stop: () => new Promise((r) => { state.sockets.forEach((s) => s.terminate()); wss.close(); server.close(r); }) }));
  });
}

// ---- the app in jsdom ---------------------------------------------------------------------------------------------
async function loadApp(pageUrl) {
  const dom = new JSDOM(`<!DOCTYPE html><html><head></head><body><div id="main"></div></body></html>`, { url: pageUrl, pretendToBeVisual: true });
  const w = dom.window;
  for (const k of Object.getOwnPropertyNames(w)) if (!(k in globalThis)) { try { globalThis[k] = w[k]; } catch {} }
  globalThis.window = w; globalThis.document = w.document; globalThis.self = w;
  Object.defineProperty(globalThis, "navigator", { value: w.navigator, configurable: true });
  for (const k of ["WebSocket", "MessageEvent", "CustomEvent", "localStorage"]) {
    Object.defineProperty(globalThis, k, { value: w[k], configurable: true, writable: true });   // Node has its own, jsdom events are not instances of them
  }
  const glue = await import(`${dist}/pkg/mopimopi-dioxus.js`);
  await glue.default({ module_or_path: fs.readFileSync(`${dist}/pkg/mopimopi-dioxus_bg.wasm`) });   // runs main(): the socket is created here
  return w;
}
const text = (w, selector) => w.document.querySelector(selector)?.textContent ?? "";
async function until(w, predicate, timeoutMs = 8000) { const start = Date.now(); while (Date.now() - start < timeoutMs) { if (predicate()) return true; await wait(50); } return false; }

console.error = () => {}; console.warn = () => {};
const PORT = 18801;
const scenarios = {};
const scenario = (id, title, body) => { scenarios[id] = { title, body }; };
const page = (query) => `http://127.0.0.1:8080/${query}`;
const rows = (w) => !!w.document.querySelector("#DPSBody .tableWrap");

scenario("1", "?OVERLAY_WS=ws://127.0.0.1:<port>/ws connects at once and streams data", async () => {
  const server = await startServer(PORT, { sendDelayMs: 700 });
  const w = await loadApp(page(`?OVERLAY_WS=ws://127.0.0.1:${PORT}/ws`));
  check("says it is connected, with the URL", await until(w, () => text(w, ".stat").includes(`Connected to ws://127.0.0.1:${PORT}/ws.`), 4000), text(w, ".stat").slice(0, 70));
  check("checklist for a connected but silent ACT is shown", text(w, ".connectHelp").includes("YOU"));
  check("tables appear when data arrives", await until(w, () => rows(w)));
  check("the player name from the name channel is used", text(w, "#YOU .name") === "Eos Fair" || w.document.body.textContent.includes("Eos Fair"));
  check("exactly one subscription and one connection (the socket exists once)", server.state.subscribes === 1 && server.state.connections === 1, `(${server.state.subscribes} / ${server.state.connections})`);
  w.close(); await server.stop();
});

scenario("2", "?HOST_PORT=ws://127.0.0.1:<port> (no /ws) gets /ws added", async () => {
  const server = await startServer(PORT + 1);       // the server only answers on /ws
  const w = await loadApp(page(`?HOST_PORT=ws://127.0.0.1:${PORT + 1}`));
  check("connects and shows data", await until(w, () => rows(w), 8000));
  check("the connected URL ends with /ws", text(w, ".stat").includes("/ws") || rows(w));
  check("one subscription", server.state.subscribes === 1, `(${server.state.subscribes})`);
  w.close(); await server.stop();
});

scenario("3", "an escaped address (%3A%2F%2F) and a trailing slash are handled", async () => {
  const server = await startServer(PORT + 2);
  const w = await loadApp(page(`?OVERLAY_WS=ws%3A%2F%2F127.0.0.1%3A${PORT + 2}%2F`));
  check("connects and shows data", await until(w, () => rows(w), 8000));
  w.close(); await server.stop();
});

scenario("4", "no address in the URL: says how to set it up; sample data still works", async () => {
  const w = await loadApp(page(""));
  check("says the URL has no OverlayPlugin address", await until(w, () => text(w, ".stat").includes("No OverlayPlugin address"), 4000), text(w, ".stat").slice(0, 60));
  check("shows the parameter to add", text(w, ".stat").includes("?OVERLAY_WS=ws://127.0.0.1:10501/ws"));
  check("explains the WSServer steps", text(w, ".connectHelp").includes("WSServer") && text(w, ".connectHelp").includes("Running"));
  w.document.querySelector(".cbtn.alt").dispatchEvent(new w.MouseEvent("click", { bubbles: true }));
  check("'Show sample data' draws the tables", await until(w, () => rows(w), 4000));
  w.close();
});

scenario("8", "an empty or unusable ?OVERLAY_WS= counts as no address", async () => {
  const w = await loadApp(page("?OVERLAY_WS="));
  check("says the URL has no OverlayPlugin address", await until(w, () => text(w, ".stat").includes("No OverlayPlugin address"), 4000), text(w, ".stat").slice(0, 60));
  w.close();
});

scenario("5", "server not running at first: retries by itself with backoff, then connects", async () => {
  const w = await loadApp(page(`?OVERLAY_WS=ws://127.0.0.1:${PORT + 3}/ws`));
  const seen = []; let server;
  const sampler = setInterval(() => { const m = text(w, ".stat").match(/again in (\d+) s/); if (m && seen[seen.length - 1] !== m[1]) seen.push(m[1]); }, 60);
  await until(w, () => text(w, ".stat").includes("Could not connect"), 4000);
  check("says it could not connect, to which URL, and when it retries", /Could not connect to ws:\/\/127\.0\.0\.1:\d+\/ws\. Trying again in \d+ s/.test(text(w, ".stat")), text(w, ".stat").slice(0, 80));
  check("checklist for a failed connection mentions SSL and the WSServer", text(w, ".connectHelp").includes("wss://") && text(w, ".connectHelp").includes("Running"));
  await wait(3800);                                   // let a few attempts fail (delays 1, 1, 2 ...)
  server = await startServer(PORT + 3);
  check("connects once the server appears", await until(w, () => rows(w), 12000));
  clearInterval(sampler);
  check("retry delays grew: 1, then 2 seconds", seen.includes("1") && seen.includes("2"), `(seen: ${seen.join(",")})`);
  w.close(); await server.stop();
});

scenario("6", "connection lost during a session: warning, then reconnects and resubscribes", async () => {
  const server = await startServer(PORT + 4);
  const w = await loadApp(page(`?OVERLAY_WS=ws://127.0.0.1:${PORT + 4}/ws`));
  await until(w, () => rows(w));
  check("top bar shows the encounter while connected", text(w, "[name=target]").includes("Striking Dummy"), text(w, "[name=target]"));
  const showedWarning = until(w, () => text(w, "[name=target]").includes("Disconnected"), 4000);
  server.dropClients();
  check("top bar warns about the lost connection", await showedWarning, text(w, "[name=target]"));
  check("reconnects by itself and the top bar recovers", await until(w, () => text(w, "[name=target]").includes("Striking Dummy"), 8000));
  check("the server got a second subscription after the reconnect", await until(w, () => server.state.subscribes >= 2, 3000), `(${server.state.subscribes})`);
  w.close(); await server.stop();
});

scenario("7", "data without a row named YOU: explains what to check in ACT", async () => {
  const server = await startServer(PORT + 5, { withoutYou: true });
  const w = await loadApp(page(`?OVERLAY_WS=ws://127.0.0.1:${PORT + 5}/ws`));
  check("shows the hint instead of empty tables", await until(w, () => text(w, ".connectBox").includes("no row named YOU"), 8000), text(w, ".connectBox").slice(0, 60));
  check("the hint says to enter YOU under Options / Miscellaneous", text(w, ".connectHelp").includes("Miscellaneous"));
  check("no tables are drawn", !rows(w));
  w.close(); await server.stop();
});

const chosen = process.argv[3];
if (chosen) {
  console.log(`${chosen}. ${scenarios[chosen].title}`);
  await scenarios[chosen].body();
  process.exit(failures === 0 ? 0 : 1);
}
// no scenario given: run each in its own process (an app instance leaves timers behind)
const { spawnSync } = await import("child_process");
let failed = 0;
for (const id of Object.keys(scenarios)) {
  const run = spawnSync(process.execPath, [fileURLToPath(import.meta.url), dist, id], { encoding: "utf8", timeout: 90000 });
  process.stdout.write(run.stdout);
  if (run.status !== 0) { failed++; if (run.stderr) process.stdout.write(run.stderr.split("\n").slice(0, 6).join("\n") + "\n"); }
}
console.log(failed === 0 ? "\nALL PASSED" : `\n${failed} SCENARIO(S) FAILED`);
process.exit(failed === 0 ? 0 : 1);
