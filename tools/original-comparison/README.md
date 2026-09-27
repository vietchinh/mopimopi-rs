# Comparing with the original MopiMopi

These scripts run the **original** overlay (the JavaScript of the upstream repository) and **this port** in the same simulated
browser (jsdom) against a fake OverlayPlugin / ACTWebSocket server. Setup, once: `npm i jsdom ws jquery` in this folder
(or anywhere `node` can resolve them), a checkout of the original repository, and a built `dist/` (`./build.sh`).

| Script | What it does |
|---|---|
| `run-original.mjs <original repo> messages` | Prints what the original sends over the socket: the keep-alive answer `.` and the `overlayAPI` requests behind the Capture and End-encounter buttons. |
| `run-original.mjs <original repo> table [fight.json]` | Prints the DPS / HPS rows the original draws for a fight, as JSON. |
| `run-port.mjs [dist] [fight.json]` | The same for this port. |
| `compare-tables.mjs <original repo> [dist]` | Draws three fights (the sample fight, a real Beastmaster capture, and a generated 24-player raid with pets and a limit break) with both and compares every cell. Exits with 1 on any difference, or if nothing was drawn. |

Notes: the original treats the zone name `HAERU` as its own preview data, so the scripts rename the zone. jsdom is not a real
browser: this compares the data pipeline (parsing, pet merging, ranking, number and name formatting), not pixels.
