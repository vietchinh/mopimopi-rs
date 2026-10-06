// Two jobs in one tiny Node server (no framework):
//
//  1. Plays OverlayPlugin for the port. Tests register the combat data a page should receive with
//     `PUT /__session/<id>` (JSON body); the page connects to `ws://host/<id>/ws` (the port insists the address ends in `/ws`, as OverlayPlugin's does), sends its `subscribe`
//     call, and gets that data back. `POST /__push/<id>` sends a further update to that session's open
//     sockets (used by the animation and performance tests).
//  2. Serves a directory of static files (`--static DIR`): the original overlay, or the built port.
//
//   node support/server.mjs --port 9090 --static /path/to/original      # original + OverlayPlugin mock
//   node support/server.mjs --port 8080 --static ../dist                # the built port (no `dx serve`)
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { WebSocketServer } from "ws";

const args = process.argv.slice(2);
const option = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
const port = Number(option("--port", 9090));
const root = option("--static") ? path.resolve(option("--static")) : null;

const MIME = {
  ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css",
  ".png": "image/png", ".svg": "image/svg+xml", ".woff": "font/woff", ".woff2": "font/woff2",
  ".ttf": "font/ttf", ".json": "application/json",
};

/** id -> { payload, sockets } */
const sessions = new Map();
const session = (id) => {
  if (!sessions.has(id)) sessions.set(id, { payload: null, sockets: new Set() });
  return sessions.get(id);
};
const readBody = (request) =>
  new Promise((resolve) => {
    let text = "";
    request.on("data", (chunk) => (text += chunk));
    request.on("end", () => resolve(text));
  });

const server = http.createServer(async (request, response) => {
  const url = new URL(request.url, "http://localhost");
  if (url.pathname === "/__health") return response.end("ok");

  const control = url.pathname.match(/^\/__(session|push)\/([\w-]+)$/);
  if (control && (request.method === "PUT" || request.method === "POST")) {
    const [, kind, id] = control;
    const payload = JSON.parse((await readBody(request)) || "null");
    const state = session(id);
    if (kind === "session") state.payload = payload;
    else for (const socket of state.sockets) if (socket.readyState === 1) socket.send(JSON.stringify(payload));
    return response.end("ok");
  }

  if (!root) { response.statusCode = 404; return response.end(); }
  const requested = decodeURIComponent(url.pathname).replace(/\/$/, "/index.html");
  const file = path.join(root, requested);
  if (!file.startsWith(root) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    response.statusCode = 404;
    return response.end();
  }
  response.setHeader("content-type", MIME[path.extname(file)] ?? "application/octet-stream");
  response.setHeader("cache-control", "no-store");
  fs.createReadStream(file).pipe(response);
});

const sockets = new WebSocketServer({ noServer: true });
server.on("upgrade", (request, socket, head) => {
  const match = new URL(request.url, "http://localhost").pathname.match(/^\/([\w-]+)\/ws$/);
  if (!match) return socket.destroy();
  sockets.handleUpgrade(request, socket, head, (ws) => {
    const state = session(match[1]);
    state.sockets.add(ws);
    ws.on("close", () => state.sockets.delete(ws));
    ws.on("message", (raw) => {
      let message;
      try { message = JSON.parse(raw.toString()); } catch { return; }
      // OverlayPlugin's WebSocket API: the overlay subscribes, and gets combat data from then on.
      if (message.call === "subscribe" && state.payload) ws.send(JSON.stringify(state.payload));
    });
  });
});

server.listen(port, "127.0.0.1", () => console.log(`support server on http://127.0.0.1:${port}${root ? ` serving ${root}` : ""}`));
