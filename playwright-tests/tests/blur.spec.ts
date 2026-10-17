// Clicking a row's job icon hides that player's name (and clicking again shows it), but only when no fight is running. The click
// handler reads the shared app context when it runs, so this exercises that path for real.
import { test, expect } from "../support/fixtures";
import { makeData } from "../support/data";

const finishedFight = () => ({ ...makeData(), isActive: "false" });

test("clicking the job icon hides the name and clicking again shows it (outside a fight)", async ({ app }) => {
  const errors: string[] = [];
  app.context.on("page", (page) => page.on("pageerror", (error) => errors.push(String(error))));
  // A fight is seen running first and then ends, like a real one (data that is inactive from the start is not drawn at all).
  const { page, handle } = await app.openPort({ data: makeData() });
  await handle.send(finishedFight());
  const name = page.locator("#DPSBody #YOU td.name");
  const icon = page.locator("#DPSBody #YOU td.Class");
  await expect(name).toBeVisible();
  await page.waitForTimeout(300); // the update that ends the fight reaches the page
  await icon.click();
  await expect(name).toBeHidden();
  await icon.click();
  await expect(name).toBeVisible();
  expect(errors).toEqual([]);
});

test("during a fight the job icon does not hide the name", async ({ app }) => {
  const { page } = await app.openPort({ data: makeData() }); // isActive: "true"
  const name = page.locator("#DPSBody #YOU td.name");
  await expect(name).toBeVisible();
  await page.locator("#DPSBody #YOU td.Class").click();
  await expect(name).toBeVisible();
});
