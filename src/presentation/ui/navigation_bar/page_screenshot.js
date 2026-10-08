// Renders part of the page to a PNG and downloads it.
//
// OverlayPlugin cannot do this for a WebSocket client (its legacy endpoint only logs "ACTWS Capture is not supported outside
// of overlays"), so the page draws itself: the DOM is copied with every computed style written into it, wrapped in an SVG
// <foreignObject> and drawn onto a canvas. Images and the page's own fonts are embedded as data URLs, because an SVG drawn as
// an image may not load anything from the network.

// Not part of the screenshot: the buttons (the original hid them for its capture too), menus, tooltips and toasts.
const LEFT_OUT = ".btn_wrap, #tooltip, .toast, #blackBg, .dropdown";
// The screenshot ends below the last of these (the overlay window is usually much taller than its content).
const CONTENT = "nav, .tableWrap, .rRow";
const MAX_SCALE = 2;

/** @param {string} rootSelector element to draw, @param {string} fileName name of the downloaded file */
export async function capturePagePng(rootSelector, fileName) {
  const root = document.querySelector(rootSelector);
  if (!root) throw new Error(`nothing to capture: ${rootSelector}`);

  const rootBox = root.getBoundingClientRect();
  const width = Math.ceil(rootBox.width);
  const height = Math.ceil(lowestBottom(root, rootBox.top));
  if (width < 1 || height < 1) throw new Error("nothing visible to capture");

  const clone = root.cloneNode(true);
  copyComputedStyles(root, clone);
  for (const element of clone.querySelectorAll(LEFT_OUT)) element.remove();
  Object.assign(clone.style, { position: "relative", left: "0", top: "0", margin: "0", width: `${width}px`, height: `${height}px`, overflow: "hidden" });

  await Promise.all([...clone.querySelectorAll("img")].map(inlineImage));
  const fontCss = await embeddedFontCss();

  const markup = new XMLSerializer().serializeToString(clone);
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}">`
    + `<style>${fontCss}</style><foreignObject x="0" y="0" width="100%" height="100%">${markup}</foreignObject></svg>`;

  const image = new Image();
  image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  await image.decode();

  const scale = Math.min(MAX_SCALE, window.devicePixelRatio || 1);
  const canvas = document.createElement("canvas");
  canvas.width = Math.round(width * scale);
  canvas.height = Math.round(height * scale);
  const context = canvas.getContext("2d");
  context.scale(scale, scale);
  context.drawImage(image, 0, 0, width, height);
  const blob = await new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("the canvas could not be turned into an image"))), "image/png"));

  download(blob, fileName);
  return { fileName, width: canvas.width, height: canvas.height, bytes: blob.size };
}

/** The lowest edge of the content below `top`, in pixels from `top`; the whole element if it has no content marks. */
function lowestBottom(root, top) {
  let bottom = 0;
  for (const element of root.querySelectorAll(CONTENT)) {
    if (element.closest(LEFT_OUT)) continue;
    const box = element.getBoundingClientRect();
    if (box.height > 0) bottom = Math.max(bottom, box.bottom - top);
  }
  return bottom > 0 ? bottom : root.getBoundingClientRect().height;
}

/** Writes every computed style of `source` and its descendants into the matching element of `target` (a clone). */
function copyComputedStyles(source, target) {
  const computed = getComputedStyle(source);
  let css = "";
  for (const property of computed) css += `${property}:${computed.getPropertyValue(property)};`;
  target.setAttribute("style", css);
  for (let index = 0; index < source.children.length; index++) copyComputedStyles(source.children[index], target.children[index]);
}

async function inlineImage(image) {
  try {
    image.setAttribute("src", await dataUrl(image.src));
    image.removeAttribute("srcset");
  } catch {
    // an image that cannot be fetched is simply left out
  }
}

/** The `@font-face` rules of the page's own stylesheets, with the font files embedded. Stylesheets from other sites are skipped. */
async function embeddedFontCss() {
  const rules = [];
  for (const sheet of document.styleSheets) {
    let cssRules;
    try { cssRules = sheet.cssRules; } catch { continue; }
    for (const rule of cssRules) {
      if (rule.type !== CSSRule.FONT_FACE_RULE) continue;
      const source = /url\(["']?([^"')]+)["']?\)/.exec(rule.style.getPropertyValue("src"));
      if (!source) continue;
      try {
        const url = new URL(source[1], sheet.href || document.baseURI).href;
        const style = rule.style;
        rules.push(`@font-face{font-family:${style.getPropertyValue("font-family")};font-weight:${style.getPropertyValue("font-weight") || "normal"};`
          + `font-style:${style.getPropertyValue("font-style") || "normal"};src:url(${await dataUrl(url)});}`);
      } catch {
        // a font that cannot be fetched falls back to the next one in the font stack
      }
    }
  }
  return rules.join("");
}

async function dataUrl(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: ${response.status}`);
  const blob = await response.blob();
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

function download(blob, fileName) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}
