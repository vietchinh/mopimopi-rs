import { JSDOM } from "jsdom";
import fs from "fs";
const dom = new JSDOM(`<!DOCTYPE html><html><body><div id="main"></div></body></html>`, { url: "http://127.0.0.1:8080/", pretendToBeVisual: true });
const w = dom.window;
for (const k of Object.getOwnPropertyNames(w)) { if (!(k in globalThis)) { try { globalThis[k] = w[k]; } catch {} } }
globalThis.window = w; globalThis.document = w.document; globalThis.self = w;
Object.defineProperty(globalThis, "navigator", { value: w.navigator, configurable: true });
const errors = []; process.on("exit", () => console.log("EXIT ERRORS:", errors.slice(0,6)));
process.on("uncaughtException", e => errors.push("uncaught: " + e.message));
process.on("unhandledRejection", e => errors.push("rejection: " + e));
const origErr = console.error; console.error = (...a) => { errors.push("console.error: " + a.join(" ").slice(0,300)); };
const mod = await import("/home/claude/mopimopi-dioxus/dist/pkg/mopimopi-dioxus.js");
await mod.default({ module_or_path: fs.readFileSync("/home/claude/mopimopi-dioxus/dist/pkg/mopimopi-dioxus_bg.wasm") });
const d = w.document;
const wait = (ms=200) => new Promise(r => setTimeout(r, ms));
const click = async (el) => { el.dispatchEvent(new w.MouseEvent("click", { bubbles: true, cancelable: true })); await wait(250); };
const txt = () => d.body.textContent.replace(/\s+/g, " ");
await wait(500);
console.log("start screen has connect box:", !!d.querySelector(".connectBox"));
await click(d.querySelector(".cbtn.alt"));
console.log("tables:", !!d.querySelector("#DPSBody"), d.querySelectorAll(".tableWrap").length, "rows");
console.log("nav:", d.querySelector("nav[name=main]").textContent.slice(0,120));
// open menu -> settings
await click(d.querySelector("[name=More]"));
console.log("dropdown items:", [...d.querySelectorAll(".dropdown li")].map(l => l.id).join(","));
await click(d.querySelector(".dropdown li#settings"));
console.log("settings rows:", d.querySelectorAll(".scrollArea li").length);
async function visit(id) {
  const li = d.querySelector(`.scrollArea li#${id}`);
  if (!li) { console.log("  (no", id, ")"); return false; }
  await click(li); return true;
}
for (const path of [["Design","color"],["Design","opacity"],["Design","size"],["Design","cells"],["Design","shape"],["Design","raid"],["Design","advanced"],["Design","font"],["Data"],["Data","format"],["Data","order"],["Data","abbset"],["Overlay"],["Tool"],["Tool","custom"]]) {
  let guard=0; while (d.querySelector("nav[name=settings]") && guard++<6) { const b = d.querySelector("[name=Back]"); if(!b) break; console.log("  back from", d.querySelector("nav .nav_title")?.textContent); await click(b); }
  if (d.querySelector("nav[name=settings]")) { console.log("STUCK; nav:", d.querySelector("nav")?.outerHTML.slice(0,200)); break; }
  await click(d.querySelector("nav[name=main] [name=More]"));
  await click(d.querySelector(".dropdown li#settings"));
  let ok = true;
  for (const id of path) ok = ok && await visit(id);
  if (path[1] === "format" || path[1]==="order") { const t = d.querySelectorAll(".tab_box"); }
  const title = d.querySelector("nav[name=settings] .nav_title")?.textContent;
  console.log(path.join(">"), "→", title, "| rows:", d.querySelectorAll(".scrollArea li").length, "| tabs:", d.querySelectorAll(".tab_box").length, "| preview:", !!d.querySelector(".previewArea #DPSBody_P"));
}

// ---- interactions ----
const ls = () => JSON.parse(w.localStorage.getItem("Mopi2_HAERU") || "{}");
async function toSettings() { let g=0; while (d.querySelector("nav[name=settings]") && g++<6) await click(d.querySelector("[name=Back]")); await click(d.querySelector("nav[name=main] [name=More]")); await click(d.querySelector(".dropdown li#settings")); }
await toSettings();
await visit("Data");
const tabs = () => [...d.querySelectorAll(".tab_box")];
await click(tabs()[1]);   // Number tab
console.log("number tab rows:", d.querySelectorAll(".scrollArea li").length);
const before = ls().q?.dpsType;
await click(d.querySelector(".scrollArea li#dpsType"));
console.log("radio dropdown options:", d.querySelectorAll(".dropdown li").length);
await click(d.querySelectorAll(".dropdown li")[1]);
await click(d.querySelector("#blackBg"));
console.log("dpsType", before, "->", ls().q?.dpsType);
await click(tabs()[3]);   // Action name tab
await visit("abbset");
const box = d.querySelector("#in_abbOld"), box2 = d.querySelector("#in_abbNew");
box.value = "Test Action"; box.dispatchEvent(new w.InputEvent("input", {bubbles:true})); await wait();
box2.value = "TA"; box2.dispatchEvent(new w.InputEvent("input", {bubbles:true})); await wait();
await click(d.querySelector("li.sendBtn"));
console.log("alias saved:", ls().Alias?.["Test Action"]);
await click(d.querySelector("[name=Back]")); await click(d.querySelector("[name=Back]")); 
await visit("Design"); await visit("color");
const inp = d.querySelector('li#accent input[type=color]');
inp.value = "#ff0000"; inp.dispatchEvent(new w.InputEvent("input", {bubbles:true})); await wait();
console.log("accent:", ls().Color?.accent);
await click(d.querySelector("[name=Back]")); await visit("size");
const sl = d.querySelector('li#sizeBody input[type=range]'); sl.value = "30"; sl.dispatchEvent(new w.InputEvent("input", {bubbles:true})); await wait();
console.log("sizeBody:", ls().Range?.sizeBody);
// exercise controls on the last-visited pages
await click(d.querySelector("nav[name=settings] [name=Back]"));
console.log("ERRORS:", errors.slice(0,8));
process.exit(0);
