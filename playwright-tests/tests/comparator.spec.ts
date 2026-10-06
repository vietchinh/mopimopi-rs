// The comparison itself must be able to fail. If it silently tolerated differences, every other test would be
// worthless, so these check it on pictures with a known, tiny difference, and check a wrong reference is rejected.
import { PNG } from "pngjs";
import { test, expect, pixelDifference, pixelHash, MODE } from "../support/fixtures";

function picture(mutate?: (image: PNG) => void): Buffer {
  const image = new PNG({ width: 40, height: 20 });
  for (let i = 0; i < image.data.length; i += 4) image.data.set([33, 33, 33, 255], i); // the overlay's dark grey
  mutate?.(image);
  return PNG.sync.write(image);
}

test("identical pictures compare equal", () => {
  expect(pixelDifference(picture(), picture()).differing).toBe(0);
  expect(pixelHash(picture())).toBe(pixelHash(picture()));
});

test("one pixel one colour level off is caught (no tolerance)", () => {
  const off = picture((image) => image.data.set([33, 33, 34, 255], 4 * (5 * 40 + 7)));
  expect(pixelDifference(picture(), off).differing).toBe(1);
  expect(pixelHash(picture())).not.toBe(pixelHash(off));
});

test("a picture of a different size is caught", () => {
  const small = PNG.sync.write(new PNG({ width: 40, height: 19 }));
  expect(pixelDifference(picture(), small).sizeMismatch).toBe(true);
  expect(pixelHash(picture())).not.toBe(pixelHash(small));
});

test("the port's screen is not accepted for the wrong reference", async ({ app }) => {
  // the port with default settings must NOT match what the original draws with pets left unmerged
  const { page } = await app.openPort({});
  const actual = pixelHash(await app.shoot(page));
  let wrong: string;
  try { wrong = await app.referenceHash("tables/base/q.pets=0", { variant: { q: { pets: 0 } } }); } catch (error) {
    test.skip(MODE === "golden", `no golden recorded for the comparison picture: ${error}`);
    throw error;
  }
  expect(actual).not.toBe(wrong);
});
