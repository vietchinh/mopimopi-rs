// The edge-case settings profiles (tests/fixtures/settings), drawn by the port and by the original with the same data: the pictures must
// be identical. They are the reference for refactoring the styling code, which has to leave every one of them looking exactly the same.
import { test } from "../support/fixtures";
import { PROFILES, loadProfile } from "../support/profiles";

for (const name of PROFILES) {
  test(`profile: ${name}`, async ({ app }) => {
    await app.matchScreenshot(`profiles/${name}`, { settings: loadProfile(name) });
  });
}
