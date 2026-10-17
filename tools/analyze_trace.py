#!/usr/bin/env python3
"""Summarises Chrome performance traces and prints them side by side.

Record a trace in Chrome DevTools (Performance tab -> record ~5 s while the overlay receives data -> save profile,
.json or .json.gz), then:

    python3 tools/analyze_trace.py official.json.gz ours.json.gz [--skip 0.5]

Steady-state numbers ignore the first --skip seconds (page load). "Main thread" is the renderer's main thread.
"""
import argparse, collections, gzip, json, os

def load(path):
    opener = gzip.open if path.endswith('.gz') else open
    with opener(path, 'rt', encoding='utf-8') as f:
        data = json.load(f)
    return data['traceEvents'] if isinstance(data, dict) else data

def busiest_renderer_main(events):
    """The renderer main thread that ran the most tasks (a trace can contain several renderers)."""
    best = None
    for m in events:
        if m.get('ph') == 'M' and m.get('name') == 'thread_name' and m['args'].get('name') == 'CrRendererMain':
            key = (m['pid'], m['tid'])
            count = sum(1 for e in events if (e.get('pid'), e.get('tid')) == key and e.get('name') == 'RunTask')
            if best is None or count > best[1]:
                best = (key, count)
    return best[0]

def summarise(path, skip):
    events = load(path)
    stamps = [e['ts'] for e in events if e.get('ts', 0) > 0]
    t0, span = min(stamps), max(stamps) - min(stamps)
    main_key = busiest_renderer_main(events)
    main = [e for e in events if (e.get('pid'), e.get('tid')) == main_key and e.get('ph') == 'X']
    window = (span - skip * 1e6) / 1e6
    in_window = lambda e: e['ts'] - t0 >= skip * 1e6
    tasks = [e for e in main if e['name'] == 'RunTask']
    busy = sum(e['dur'] for e in tasks if in_window(e)) / 1e3
    messages = sum(1 for e in events if e.get('name') == 'WebSocketReceive' and in_window(e))
    count = lambda name: sum(1 for e in main if e['name'] == name and in_window(e))
    thread_names = {(m['pid'], m['tid']): m['args']['name'] for m in events if m.get('ph') == 'M' and m.get('name') == 'thread_name'}
    gpu = sum(e['dur'] for e in events if e.get('ph') == 'X' and e.get('name') in ('RunTask', 'GPUTask', 'RasterTask') and in_window(e)
              and (thread_names.get((e['pid'], e['tid'])) in ('Compositor', 'CrGpuMain', 'VizCompositorThread') or e['name'] == 'RasterTask')) / 1e3
    navigation = min([e['ts'] for e in events if e.get('name') == 'navigationStart'], default=t0)
    fcp = [e['ts'] for e in events if e.get('name') == 'firstContentfulPaint']
    requests, sizes = {}, collections.Counter()
    for e in events:
        d = e.get('args', {}).get('data', {})
        if e.get('name') == 'ResourceSendRequest': requests[d.get('requestId')] = d.get('url', '')
        if e.get('name') == 'ResourceReceivedData': sizes[d.get('requestId')] += d.get('encodedDataLength', 0)
        if e.get('name') == 'ResourceFinish': sizes[d.get('requestId')] = max(sizes[d.get('requestId')], d.get('encodedDataLength', 0))
    code_kb = sum(sizes[r] for r, u in requests.items() if u.split('?')[0].endswith(('.js', '.wasm'))) / 1024
    return {
        'Trace length (s)': span / 1e6,
        'WebSocket messages per second': messages / window,
        'Main thread busy (ms per second)': busy / window,
        'Main thread per message (ms)': busy / max(messages, 1),
        'requestAnimationFrame callbacks per second': count('FireAnimationFrame') / window,
        'Redraws (compositor commits) per second': count('Commit') / window,
        'Layouts per second': count('Layout') / window,
        'Style recalculations per second': count('UpdateLayoutTree') / window,
        'GPU + compositor + raster (ms per second)': gpu / window,
        'Garbage collection on main thread (ms per second)': sum(e['dur'] for e in main if 'GC' in e['name'] and in_window(e)) / 1e3 / window,
        'First contentful paint (ms)': (min(fcp) - navigation) / 1e3 if fcp else float('nan'),
        'Main thread busy, first 1.5 s (ms)': sum(e['dur'] for e in tasks if e['ts'] - t0 < 1.5e6) / 1e3,
        'Longest task (ms)': max(e['dur'] for e in tasks) / 1e3,
        'Requests': len(requests),
        'Transferred (KB)': sum(sizes.values()) / 1024,
        'JS + wasm transferred (KB)': code_kb,
    }

def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('traces', nargs='+', help='Chrome trace files (.json or .json.gz)')
    parser.add_argument('--skip', type=float, default=0.5, help='seconds of page load to ignore in steady-state numbers')
    args = parser.parse_args()
    results = [summarise(p, args.skip) for p in args.traces]
    names = [os.path.basename(p)[:22] for p in args.traces]
    width = max(len(k) for k in results[0]) + 2
    print(' ' * width + ''.join(f'{n:>24s}' for n in names))
    for key in results[0]:
        print(f'{key:<{width}}' + ''.join(f'{r[key]:>24.1f}' if isinstance(r[key], float) else f'{r[key]:>24}' for r in results))

if __name__ == '__main__':
    main()
