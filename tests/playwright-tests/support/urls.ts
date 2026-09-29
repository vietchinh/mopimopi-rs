// Where the two apps are served, in one place for the config and the fixtures.
//
// `dx serve` serves the port under the `base_path` in Dioxus.toml ("mopimopi-rs"), so its address is
// http://127.0.0.1:8080/mopimopi-rs/. A plain static server over `dist/` (E2E_SERVE=dist) has no such prefix.
export const PORT_URL = process.env.E2E_PORT_URL ?? (process.env.E2E_SERVE === "dist" ? "http://127.0.0.1:8080" : "http://127.0.0.1:8080/mopimopi-rs");
export const SUPPORT_URL = process.env.E2E_SUPPORT_URL ?? "http://127.0.0.1:9090";
export const ORIGINAL_URL = process.env.E2E_ORIGINAL_URL ?? SUPPORT_URL;
export const WS_URL = SUPPORT_URL.replace(/^http/, "ws");
