// The settings profiles in tests/fixtures/settings (written by tools/make-settings-fixtures.mjs): complete settings documents for the
// cases where rendering has an edge. Shared by the specs here and by the Rust tests, which read the same files.
import fs from "node:fs";
import path from "node:path";
import type { Settings } from "./settings";

const DIRECTORY = path.resolve(__dirname, "../../tests/fixtures/settings");

export const PROFILES: string[] = fs.readdirSync(DIRECTORY).filter((file) => file.endsWith(".json")).map((file) => file.replace(/\.json$/, "")).sort();

export const loadProfile = (name: string): Settings => JSON.parse(fs.readFileSync(path.join(DIRECTORY, `${name}.json`), "utf8"));
