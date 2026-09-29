import { test, expect } from "../support/fixtures";
import { openSettings } from "../support/steps";

test("the port starts and draws the fight it is sent, without errors", async ({ app }) => {
  const problems: string[] = [];
  app.context.on("page", (page) => {
    page.on("pageerror", (error) => problems.push(`page error: ${error}`));
    page.on("console", (message) => message.type() === "error" && problems.push(`console error: ${message.text()}`));
  });
  const { page } = await app.openPort();
  await expect(page.locator("#DPSBody")).toContainText("Player Mch");
  await expect(page.locator("#HPSBody")).toContainText("Player Whm");
  await expect(page.locator("nav[name=main]")).toContainText("Striking Dummy");
  expect(problems).toEqual([]);
});

test("the settings preview shows the sample fight with its pets folded into their owners", async ({ app }) => {
  // (a regression the pixel tests found: the preview once listed every pet as a row of its own)
  const { page } = await app.openPort();
  await openSettings(page);
  await page.locator("li#Data").click();
  await expect(page.locator("#DPSBody_P")).toContainText("Player Mch");
  await expect(page.locator("#DPSBody_P")).not.toContainText("Rook Autoturret");
  await expect(page.locator("#HPSBody_P")).not.toContainText("Eos");
});
