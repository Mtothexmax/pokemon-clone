#!/usr/bin/env node
// Smoothness check: the hero walks east along the road while the engine traces its drawn position
// (render interpolation). Prints the trace summary and perf (frame_ms, missed frames).
//   node tools/smooth.mjs [--port 9223] [--url http://localhost:8091/] [--frames 300]
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(n); return i >= 0 ? args[i + 1] : d; };
const port = opt('--port', '9223'), url = opt('--url', 'http://localhost:8091/'), frames = Number(opt('--frames', 300));
const key = opt('--key', 'KeyD'), from = (opt('--from', '-22,0.5')).split(',').map(Number);
const BRIDGE = 'C:/dev/wasmOnly/tools/bridge.mjs';
const run = (lines) => {
  const f = path.join(os.tmpdir(), `smooth-${process.pid}.txt`);
  fs.writeFileSync(f, lines.join('\n') + '\n');
  try { return JSON.parse(execFileSync('node', [BRIDGE, 'run', f, '--port', port, '--url', url], { encoding: 'utf8', maxBuffer: 1 << 26 })); }
  catch (e) { return JSON.parse(e.stdout); }
};
const val = (r, cmd) => r.steps.find((s) => s.cmd.startsWith(cmd))?.value;

const a = run(['reload', 'screen play', 'wait_for game_state.state.assets_pending == 0 15000', 'frames 30', 'call game_state']);
const id = val(a, 'call game_state')?.state?.hero?.id;
if (id === undefined) { console.error(JSON.stringify(a, null, 1)); process.exit(1); }
const b = run([
  `call teleport {"x": ${from[0]}, "z": ${from[1]}}`, 'frames 10', `down ${key}`, 'frames 20',
  `call trace {"id": ${id}, "frames": ${frames}}`, `frames ${frames + 10}`, 'call trace {\"samples\": true}', `up ${key}`,
]);
const trace = b.steps.filter((s) => s.cmd.startsWith('call trace')).pop()?.value;
const ms = (trace?.samples ?? []).map((x) => x[0]).slice(1).sort((p, q) => p - q);
const perf = { frames: ms.length, median_ms: ms[ms.length >> 1], p95_ms: ms[Math.floor(ms.length * 0.95)], max_ms: ms[ms.length - 1], over_20ms: ms.filter((x) => x > 20).length };
const raw = (trace?.samples ?? []).map((x) => [Math.round(x[0]), x[1]]);
const slow = raw.map((x, i) => (x[0] > 20 ? i : -1)).filter((i) => i >= 0);
if (trace) delete trace.samples;
perf.slow_frame_indices = slow; perf.slow_gaps = slow.slice(1).map((v, i) => v - slow[i]);
console.log(JSON.stringify({ hero_id: id, trace, perf, failures: b.steps.filter((s) => !s.ok).map((s) => [s.cmd, s.error]) }, null, 1));
