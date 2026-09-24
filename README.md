# DEV NOTE

THIS INITIAL CODE IS FULL CONVERSION FROM MOPIMOPI TO DIOXUS RUST BY CLAUDE AI, SEE PULL REQUEST OF ME TRYING TO SIMPLIFY THE CODE.
USERS OF THIS BRANCH ARE BETA TESTING CODE THAT IS IN PROGRESS OF BEING VERIFIED BY THE DEV.
FROM MY LIMITED TIME TESTING, IT IS QUITE BUGGY.

# Development

Your new bare-bones project includes minimal organization with a single `main.old` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

## Performance compared with the original overlay

This port renders with Rust and WebAssembly (Dioxus) and only touches the page when data arrives. The original overlay
is jQuery-based and redraws continuously. I recorded a Chrome performance trace of each while connected to live ACT data,
served from GitHub Pages in both cases, at almost the same message rate (2.7 and 2.6 combat messages per second), and
compared them.

### Results

| Measurement | Original | This port | Difference |
|---|---|---|---|
| Main thread busy (ms per second) | 36.8 | **2.6** | 14× less |
| Main thread per combat message (ms) | 13.5 | **1.0** | 13× less |
| `requestAnimationFrame` callbacks per second | 236 | **0** | no animation loop |
| Redraws (compositor commits) per second | 60 | **4.8** | 12× fewer |
| Layouts per second | 18.7 | **0.9** | about 20× fewer |
| Style recalculations per second | 23.2 | **0.2** | nearly none |
| GPU + compositor + raster threads (ms per second) | 34.6 | **14.0** | 2.5× less |
| Garbage collection on the main thread (ms per second) | 4.9 | **0** | – |
| First contentful paint (ms) | 294 | **108** | 2.7× earlier |
| Main thread busy during the first 1.5 s (ms) | 129 | **98** | 1.3× less |
| Longest single task (ms) | **12** | 30 | original is shorter |
| Data downloaded, JS + wasm (KB) | **462** | 1,057 | original is 2.3× smaller |
| Requests | 21 | **11** | – |

"Main thread" is the renderer's main thread. Steady-state rows ignore the first 0.5 s (page load). Recordings are 5.6 s
(original, `haeruhaeru.github.io/mopimopi/`) and 5.1 s (this port, `vietchinh.github.io/mopimopi-rs/`).

### What the numbers mean

- **After start-up this port is much lighter.** It uses about 14 times less main-thread time per second at the same
  message rate. Between messages it does nothing: no timers, no animation frames. The original keeps a
  `requestAnimationFrame` loop running (about 236 callbacks per second) and redraws about 60 times per second even when
  nothing changes.
- **Less redraw work follows.** About 12 times fewer compositor commits, about 20 times fewer layouts, almost no style
  recalculations, and about 2.5 times less GPU and compositor time. On a machine that is also running the game, this
  smaller load is the practical gain.
- **Total CPU time is lower too.** Adding up the task time on every Chrome thread (renderer, GPU and browser
  processes), the original used about 84 ms per second (roughly 8% of one core) and this port about 20 ms per second
  (roughly 2%), so about 4 times less overall. The renderer process, which runs the page, dropped about 11 times (44.5
  to 3.9 ms per second). The GPU process dropped about 2 times (27.6 to 13.0 ms per second), and it is most of what
  remains, because the page is still redrawn a few times per second. These figures are sums of task durations from the
  same two traces, not operating-system CPU counters, and they include some DevTools and tracing overhead in the
  browser process. They are not produced by `tools/analyze_trace.py`.
- **Start-up is a trade.** The original is a few small scripts; this port must download and compile a roughly 1 MB
  WebAssembly file first, so its download is more than twice as large and its single longest task is longer (30 ms
  against 12 ms). Even so, its first paint was earlier in this recording and the main-thread work in the first 1.5 s was
  lower. First paint depends on the host, caching and the network, so I would not read much into a 2.7× gap.

### Caveats

- Each recording is a single run of about 5 seconds with 13 to 15 combat messages. Treat differences of a few ms per
  second as noise; the main-thread and animation-loop gaps above are far larger than that. Small counts (for example,
  the style recalculations) are only rough.
- This port's cost rises when the data changes more. Another recording of it with busier data (4.6 messages per second)
  measured 9.2 ms per second of main-thread time and about 11 layouts per second. The original's cost is largely a
  constant animation loop, so it stays high either way. Against that busier recording the main-thread gap is about 4
  times, not 14.
- The wasm was transferred at almost its full size (about 1 MB), so it does not appear to have been compressed by the
  host. I have not confirmed why.
- The extra layouts in the busier recording happen while no script runs. I have not investigated, but it looks like the
  bar-width transitions (the "animate bars" setting). Animating with `transform` instead of `width` would avoid them;
  this is untested.

