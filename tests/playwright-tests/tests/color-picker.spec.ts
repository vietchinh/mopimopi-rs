// Behaviour of the colour picker (its looks are in settings-screens.spec.ts): the same presses and typing
// must give the same colours, and the same stored text, as the original's jscolor.
import { test, expect } from "../support/fixtures";
import { changedSettings, openSettings, readSettings, runSteps } from "../support/steps";
import { settingsFor } from "../support/settings";

const openColorPage = async (page: import("@playwright/test").Page) => { await openSettings(page); await runSteps(page, ["li#Design", "li#color"]); };
const boxValue = (page: import("@playwright/test").Page, id: string) => page.locator(`li#${id} input`).inputValue();

test("pressing and dragging in the pad and slider picks the same colours", async ({ app }) => {
  await app.matchValue("picker-behaviour/interaction", { after: openColorPage }, async (page) => {
    const box = (await page.locator("li#navBg input").boundingBox())!;
    // the panel opens straight below the box; its pad is 14px in, its slider 268px in
    const pad = { x: box.x + 14, y: box.y + box.height + 14 };
    const at = (dx: number, dy: number) => [pad.x + dx, pad.y + dy] as const;
    const log: Record<string, unknown> = {};
    await page.locator("li#navBg input").click();
    log.opened = await boxValue(page, "navBg");
    await page.mouse.click(...at(100, 60)); log.padPress = await boxValue(page, "navBg");
    await page.mouse.click(box.x + 268 + 8, pad.y + 35); log.sliderPress = await boxValue(page, "navBg");
    await page.mouse.move(...at(41, 15)); await page.mouse.down(); await page.mouse.move(...at(191, 115), { steps: 8 }); await page.mouse.up();
    log.padDrag = await boxValue(page, "navBg");
    await page.mouse.move(box.x + 276, pad.y + 15); await page.mouse.down(); await page.mouse.move(box.x + 276, pad.y + 135, { steps: 8 }); await page.mouse.up();
    log.sliderDrag = await boxValue(page, "navBg");
    await page.mouse.click(300, 200); // anywhere else closes the panel
    log.panelStillOpen = (await page.locator("canvas").count()) > 0;
    return log;
  });
});

for (const [id, typed] of [["navBg", "ff8000"], ["navBg", "FfA0b1"], ["accent", "123456"]] as const) {
  test(`typing ${typed} into "${id}" tints the box and stores the text as typed`, async ({ app }) => {
    const before = settingsFor({});
    await app.matchValue(`picker-behaviour/typing-${id}-${typed}`, { after: openColorPage }, async (page) => {
      await runSteps(page, [`type:${id}=${typed}`]);
      const input = page.locator(`li#${id} input`);
      return {
        shown: await input.inputValue(),
        background: await input.evaluate((e: HTMLElement) => e.style.backgroundColor),
        textColor: await input.evaluate((e: HTMLElement) => e.style.color),
        stored: changedSettings(before, await readSettings(page)),
      };
    });
  });
}

// A deliberate difference, so it is asserted on the port alone: the original draws a typed 3-digit colour
// wrongly (its own hex-to-rgba code mishandles it), the port draws what was meant.
test("a typed 3-digit colour is drawn as intended (the original draws it wrongly)", async ({ app }) => {
  const { page } = await app.openPort({ after: openColorPage });
  await runSteps(page, ["type:navBg=0f0"]);
  await expect(page.locator("li#navBg input")).toHaveCSS("background-color", "rgb(0, 255, 0)");
  expect(changedSettings(settingsFor({}), await readSettings(page))).toEqual({ "/Color/navBg": '"212121" -> "00FF00"' });
});
