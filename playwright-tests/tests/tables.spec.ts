// The main screen against the original, across every setting. One test per variant; each renders the same
// combat data with the same settings in both apps and requires the two screenshots to be pixel-identical.
//
//   npm test                      everything
//   npm run test:quick            skips the @tall group (every variant again with all rows visible)
//   npx playwright test -g "raid" one group / one setting
import { test } from "../support/fixtures";
import { groups } from "../support/variants";

for (const [group, variants] of Object.entries(groups)) {
  test.describe(group, () => {
    for (const variant of variants()) {
      const tag = group === "tall" ? " @tall" : "";
      test(`${variant.name}${tag}`, async ({ app }) => {
        await app.matchScreenshot(`tables/${group}/${variant.name}`, { variant });
      });
    }
  });
}
