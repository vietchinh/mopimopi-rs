import { defineConfig, devices } from "@playwright/test";
import path from "node:path";
import { PORT_URL, SUPPORT_URL } from "./support/urls";

// Where the two apps are served comes from support/urls.ts (override with E2E_PORT_URL / E2E_SUPPORT_URL).
const repoRoot = path.join(__dirname, "..");
const originalDir = process.env.MOPIMOPI_ORIGINAL_DIR; // a checkout of the original overlay (live / record modes)

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  workers: process.env.CI ? 2 : undefined,
  timeout: 90_000,
  expect: { timeout: 10_000 },
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: PORT_URL,
    // The overlay is compared pixel for pixel, so the window is fixed.
    viewport: { width: 1100, height: 640 },
    deviceScaleFactor: 1,
    launchOptions: {
      // Use a browser you already have instead of Playwright's download, e.g. E2E_CHROME=/usr/bin/chromium
      executablePath: process.env.E2E_CHROME || undefined,
      args: ["--force-device-scale-factor=1"],
    },
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"], viewport: { width: 1100, height: 640 }, deviceScaleFactor: 1 } }],

  // As in the Dioxus guide (https://dioxuslabs.com/learn/0.7/guides/testing/web): Playwright starts the app
  // itself. Set E2E_SERVE=dist to serve an existing `dist/` build instead of running `dx serve`.
  webServer: [
    {
      // OverlayPlugin stand-in for the port, and (when given) the original overlay to compare against.
      command: `node support/server.mjs --port 9090${originalDir ? ` --static "${originalDir}"` : ""}`,
      cwd: __dirname,
      url: `${SUPPORT_URL}/__health`,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
    {
      // The real thing by default: `dx serve` builds the wasm and serves it. (E2E_SERVE=dist serves an existing build instead.)
      // E2E_DX_FLAGS replaces the default `--hot-reload false`; use `E2E_DX_FLAGS=--release` to test the optimized wasm
      // (`--release` cannot be combined with `--hot-reload`).
      command: process.env.E2E_SERVE === "dist"
        ? `node playwright-tests/support/server.mjs --port 8080 --static dist`
        : `dx serve --web ${process.env.E2E_DX_FLAGS ?? "--hot-reload false"} --port 8080 --addr 127.0.0.1 --open false --interactive false`,
      cwd: repoRoot,
      url: `${PORT_URL}/`,
      timeout: 10 * 60 * 1000,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
  ],
});
