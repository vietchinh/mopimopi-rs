// Tailwind finds classes by reading src/**/*.rs as plain text (the `@source` lines in tailwind.css), so a stray word in the
// code that is also a Tailwind utility (`table`, `filter`, `static`, and worse `shadow`, `hidden`, `flex`, which the markup
// uses as legacy class names) would add a rule nobody asked for. tools/check-tailwind.mjs compiles the stylesheet with and
// without the scan and fails, naming the class, when the scan produces one that is not the project's own.
import { execFileSync } from "node:child_process";
import path from "node:path";
import { test, expect } from "../support/fixtures";

const root = path.resolve(__dirname, "../..");
function run(script: string, ...args: string[]): string {
  try {
    return execFileSync("node", [script, ...args], { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  } catch (error) {
    const failure = error as { stderr?: string; stdout?: string };
    throw new Error(`${failure.stderr ?? ""}${failure.stdout ?? ""}`);
  }
}

test("the Rust scan generates only the project's own classes", () => {
  expect(run("tools/check-tailwind.mjs")).toContain("ok:");
});

// `@import "tailwindcss"` brings in Preflight; tailwind.css undoes it with a block generated from Preflight itself.
test("the block that undoes Preflight matches the installed Tailwind", () => {
  expect(run("tools/cancel-preflight.mjs", "--check")).toContain("ok:");
});
