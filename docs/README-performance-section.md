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

### Memory

I recorded a 5-minute "Allocations on timeline" heap profile in Chrome DevTools for each overlay (live ACT data, about
one combat update per second) and compared what is in memory at the end and what was allocated and kept during the
recording. The recordings are 302 s (original) and 303 s (this port).

| Measurement | Original | This port | Difference |
|---|---|---|---|
| Total size in the snapshot at the end | 10.6 MB | **8.1 MB** | 24% smaller |
| Objects in the snapshot | 174,676 | **83,589** | 2.1× fewer |
| JavaScript objects (arrays, closures, strings, shapes, objects) | 1.4 MB | **0.9 MB** | 37% less |
| Compiled JavaScript code | 1.4 MB | **0.4 MB** | 3.5× less |
| Native browser memory | 7.9 MB | **6.8 MB** | 13% less |
| ...of which wasm memory + compiled wasm module | – | 2.6 MB + 1.5 MB | – |
| Allocated during the recording and still alive | 38,217 objects, 2.8 MB | **1,354 objects, 57 KB** | 49× less |
| ...of which in the first minute | 2.7 MB | **6 KB** | – |
| Growth after the first minute | about 41 KB per minute | **about 13 KB per minute** | 3× lower |

- **This port uses less memory overall, by about a quarter of the snapshot.** It does pay 4.1 MB for wasm memory and the
  compiled wasm module, which the original does not have, and still ends up 2.5 MB smaller. The difference comes from
  holding half as many objects, much less compiled JavaScript, and far fewer browser internals. The snapshot is not the
  whole tab's memory; Chrome's Task Manager (Shift+Esc, "Memory footprint") shows that.
- **The original accumulates most of its growth at the start.** It kept 2.8 MB of new allocations, 2.7 MB of them in the
  first minute. About 1.5 MB of that is Blink accessibility objects (`AXDirtyObject`, `AXNodeObject`). I believe these come
  from the original rebuilding parts of the page as HTML text, so I would expect them to depend on how much the page
  changes. After the first minute it is nearly flat.
- **This port allocates almost nothing that stays.** Across 5 minutes it kept 57 KB. Most of that is code the browser
  compiled as it warmed up, plus a few hundred short strings and about 150 small `LayoutShiftAttribution` entries
  (Chrome's record of elements moving; see "Layout shifts" below). The amounts per 30-second window stayed between
  0 and 24 KB, with no upward trend.
- **A slow leak was found and fixed.** An earlier build of this port kept one `PerformanceMark` and one
  `PerformanceMeasure` per combat update (301 of each in 5 minutes, about 17 KB per minute, roughly 1 MB per hour of
  continuous combat). They came from Dioxus's `tracing` spans, which the browser keeps for the life of the page. The
  release build now compiles those spans out (`tracing` with `release_max_level_warn`), and the wasm is also about 2%
  smaller. Two Chrome recordings of the fixed build (83 s and 303 s) kept zero marks and zero measures.
- **Limits:** one recording each, both taken with DevTools open, which can itself create some native objects (the
  accessibility ones in particular). The wasm memory of this port started at 22 pages and was 39 pages (2.4 MiB) at the end
  of both fixed-build recordings; with one snapshot per recording I cannot say when it grew. Very long sessions
  (hours) have not been measured.

### Layout shifts

Chrome's layout shift score (CLS) measures how much visible content jumps around. Below 0.1 is considered good and above
0.25 poor. The traces show two different things in this port.

- **A large shift at start-up (score 0.95 in one trace, about 2.9 cumulative in another, before the fix).** The original
  scored about 0.006. In this port the stylesheets are added by the app itself after the wasm has started, so the first
  frame was drawn with the browser's defaults (an 8px `<body>` margin, a serif font) and the page jumped when the CSS
  arrived: the first shift in the trace was `<body>` moving from `[8,8,609,472]` to `[0,0,640,480]`. It is fixed by a tiny
  critical reset that is static in the page (a `data:` URI in `Dioxus.toml` for `dx`; the inline `<style>` in
  `web/index.html` for `build.sh`). A Chrome trace of the fixed page under `dx serve --release` (640x480, headless) has
  0 `LayoutShift` events. Listing the stylesheets themselves under `[web.resource] style` does not work: `dx` 0.7.10 emits
  them as literal `<link href>`s without copying the files, so with a `base_path` they 404 and the page is unstyled.
- **Small recurring shifts (0.0001 to 0.0013 each, about 0.002 in total).** They happen roughly once per combat update,
  which is why the heap timeline holds a `LayoutShiftAttribution` entry for each. The affected table cells move sideways in
  steps of about 7 px (for example 83, 76, 69, 76 px from the left) when values change. My guess is that a column's width
  follows the length of its text; I have not confirmed it. The score is far inside the "good" range, so I have left it.

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

### Reproduce it

1. Open the overlay in Chrome with the ACT address, for example `?HOST_PORT=ws://127.0.0.1:10501`, and let a fight run.
2. DevTools -> Performance -> record about 5 seconds -> save the profile (`.json` or `.json.gz`).
3. Compare traces side by side:

   ```
   python3 tools/analyze_trace.py original.json.gz this-port.json.gz
   ```
