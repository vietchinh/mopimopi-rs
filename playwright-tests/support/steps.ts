// A tiny action language, so one list of steps drives the original and the port the same way. Both apps
// use the same ids and class names (the port keeps the original's markup), which is what makes this possible.
//
//   li#Design            click that element              tab:tab_name        click a settings tab
//   input:navBg          click a colour box (opens the picker)      type:navBg=ff8000   focus a colour box and type
//   mouse:759,425        click at page coordinates       drag:700,380>850,480   press, drag, release
//   dropdown:2           pick the 2nd entry of an open choice list  range:li#sizeNav input=60   set a slider
//   js:li#pets           click via the DOM (rows that are present but not scrolled into view)
import type { Page } from "@playwright/test";

export async function openSettings(page: Page): Promise<void> {
  await page.locator("nav[name=main] div[name=More]").click();
  await page.locator("#settings").click();
  await page.locator(".tab_box, .scrollArea li").first().waitFor();
}

export async function runSteps(page: Page, steps: string[]): Promise<void> {
  for (const step of steps) {
    const [, kind, rest] = step.match(/^(tab|input|type|mouse|drag|dropdown|range|js):(.*)$/) ?? [];
    switch (kind) {
      // `.first()` throughout: an id can appear twice (the ⋮ menu and the settings page both have `li#pets`); the first match is what a plain query would give
      case "tab": await page.locator(`.tab_box[name=${rest}]`).first().click(); break;
      case "input": await page.locator(`li#${rest} input`).first().click(); break;
      case "type": {
        const [id, text] = [rest.slice(0, rest.indexOf("=")), rest.slice(rest.indexOf("=") + 1)];
        const box = page.locator(`li#${id} input`).first();
        await box.focus(); // (a plain click can leave the keystrokes with the page)
        await box.evaluate((element: HTMLInputElement) => element.select());
        await page.keyboard.type(text, { delay: 40 });
        break;
      }
      case "mouse": { const [x, y] = rest.split(",").map(Number); await page.mouse.click(x, y); await page.waitForTimeout(250); break; }
      case "drag": {
        const [from, to] = rest.split(">").map((p) => p.split(",").map(Number));
        await page.mouse.move(from[0], from[1]); await page.mouse.down(); await page.mouse.move(to[0], to[1], { steps: 8 }); await page.mouse.up();
        await page.waitForTimeout(250);
        break;
      }
      case "dropdown": await page.locator(`.dropdown li:nth-child(${rest})`).first().click(); break;
      case "range": {
        // (split at the last "=": the selector itself contains one, in `[type=range]`)
        const [selector, value] = [rest.slice(0, rest.lastIndexOf("=")), rest.slice(rest.lastIndexOf("=") + 1)];
        await page.locator(selector).first().evaluate((element: HTMLInputElement, v: string) => {
          element.value = v;
          element.dispatchEvent(new Event("input", { bubbles: true }));
          element.dispatchEvent(new Event("change", { bubbles: true }));
        }, value);
        break;
      }
      case "js": await page.locator(rest).first().evaluate((element: HTMLElement) => element.click()); break;
      default: await page.locator(step).first().click();
    }
  }
}

/** What the overlay has in its settings right now: the original keeps them in memory (`init`); the port saves when the page is hidden. */
export function readSettings(page: Page): Promise<any> {
  return page.evaluate(() => {
    if (typeof (window as any).init !== "undefined") return JSON.parse(JSON.stringify((window as any).init));
    window.dispatchEvent(new Event("pagehide"));
    document.dispatchEvent(new Event("visibilitychange"));
    return JSON.parse(localStorage.getItem("Mopi2_HAERU") as string);
  });
}

/** Paths of the settings that differ between two settings objects, as `"/q/pets": "1 -> 0"`. Ignores bookkeeping the original flips while a preview is showing. */
export function changedSettings(before: any, after: any): Record<string, string> {
  const flat = (node: any, prefix = "", out: Record<string, string> = {}) => {
    if (node && typeof node === "object") for (const key of Object.keys(node)) flat(node[key], `${prefix}/${key}`, out);
    else out[prefix] = JSON.stringify(node);
    return out;
  };
  const [a, b] = [flat(before), flat(after)];
  const changed: Record<string, string> = {};
  for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
    if (a[key] !== b[key] && !/backupDate|\/q\/preview$/.test(key)) changed[key] = `${a[key]} -> ${b[key]}`;
  }
  return changed;
}
