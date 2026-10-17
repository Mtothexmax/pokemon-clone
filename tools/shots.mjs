/**
 * tools/shots.mjs — one command for the whole visual-check loop.
 *
 *   node tools/shots.mjs --build --test          rebuild, run tests, screenshot
 *   node tools/shots.mjs --only=highgrass         one shot
 *   node tools/shots.mjs --out=shots/foo          custom output dir
 *
 * It (optionally) builds, starts `vite preview` and a headless Chrome, drives
 * them over CDP, saves PNGs and prints a compact one-line-per-item summary.
 * Everything is cleaned up automatically (no orphan servers / chrome).
 */
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';

const arg = (k, d) => {
	const hit = process.argv.slice(2).find((a) => a.startsWith(`--${k}=`));
	return hit ? hit.slice(k.length + 3) : d;
};
const has = (k) => process.argv.slice(2).some((a) => a === `--${k}` || a.startsWith(`--${k}=`));

const PORT = Number(arg('port', 4174));
const CDP = Number(arg('cdp', 9222));
const OUT = arg('out', path.join(process.env.TEMP || '.', 'shots'));
const ONLY = arg('only', '');
const BUILD = has('build');
const TEST = has('test');
const ROOT = path.resolve(import.meta.dirname, '..');
fs.mkdirSync(OUT, { recursive: true });

// ---------------------------------------------------------------- shots
const enter = (map, x, y, dir) =>
	`window.__enterMap(window.__game,'${map}',${x},${y},'${dir}'); window.__game.frame=0;`;
const SHOTS = [
	{ name: 'pallet_wide', viewport: [3840, 1080], pre: enter('pallet', 34, 16, 'down') },
	{ name: 'pallet_hd', viewport: [1920, 1080], pre: enter('pallet', 34, 16, 'down') },
	{ name: 'highgrass', viewport: [1280, 720], pre: enter('pallet', 29, 2, 'up') },
	{ name: 'lab_building', viewport: [1280, 720], pre: enter('pallet', 8, 10, 'up') },
	{ name: 'lab_inside', viewport: [1920, 1080], pre: enter('lab', 7, 9, 'up') },
	{ name: 'river', viewport: [1280, 720], pre: enter('pallet', 24, 12, 'down') },
	{ name: 'bridge', viewport: [1280, 720], pre: enter('pallet', 25, 15, 'down') },
	{ name: 'houses', viewport: [1280, 720], pre: enter('pallet', 38, 13, 'up') },
	{ name: 'flowers', viewport: [1280, 720], pre: enter('pallet', 40, 27, 'up') },
	// On-screen touch pad (force with ?touch=1). `mode: window` captures the DOM.
	{ name: 'touch_desktop', viewport: [1280, 720], query: 'touch=1', mode: 'window', pre: enter('pallet', 34, 16, 'down') },
	{ name: 'touch_mobile', viewport: [844, 390], query: 'touch=1', mode: 'window', pre: enter('pallet', 34, 16, 'down') },
	{ name: 'touch_phone', viewport: [390, 844], query: 'touch=1', mode: 'window', pre: enter('pallet', 34, 16, 'down') }
];

// ------------------------------------------------------------- lifecycle
const children = [];
function bg(file, cmd, args, cwd) {
	const out = fs.openSync(file, 'w');
	const p = spawn(cmd, args, { cwd, stdio: ['ignore', out, out], shell: false });
	children.push(p);
	return p;
}
function cleanup() {
	for (const p of children) {
		try { p.kill(); } catch {}
	}
}
process.on('exit', cleanup);
process.on('SIGINT', () => { cleanup(); process.exit(130); });

const t0 = Date.now();
const log = (s) => console.log(s);
log(`shots → ${OUT}`);

if (BUILD) {
	log('build …');
	const r = spawnSync('cmd', ['/c', 'npx vite build'], { cwd: ROOT, encoding: 'utf8' });
	const m = (r.stdout + r.stderr).match(/built in [\d.]+m?s/g);
	log(r.status === 0 ? `build ok ${m ? '(' + m.at(-1) + ')' : ''}` : 'build FAILED\n' + (r.stderr || '').slice(-400));
	if (r.status !== 0) process.exit(1);
}

if (TEST) {
	for (const t of ['headless', 'bootGame', 'bootStrict']) {
		const r = spawnSync('node', [`test/${t}.mjs`], { cwd: ROOT, encoding: 'utf8' });
		const bad = (r.stdout + r.stderr).split('\n').filter((l) => l.includes('FAIL')).slice(0, 4);
		log(`test ${t}: ${r.status === 0 ? 'ok' : 'FAIL ' + bad.join(' | ')}`);
	}
}

const previewLog = path.join(OUT, '_preview.log');
// Spawn the local vite binary with node directly (not via `cmd /c npx`) so the
// child we hold is the actual server and kill() cleanly frees the port.
bg(previewLog, process.execPath, [
	path.join(ROOT, 'node_modules', 'vite', 'bin', 'vite.js'),
	'preview', '--port', String(PORT), '--strictPort'
]);
const chromeLog = path.join(OUT, '_chrome.log');
const chromeBin = process.env.CHROME || 'C:/Program Files/Google/Chrome/Application/chrome.exe';
const profile = path.join(process.env.TEMP || '.', `shots-profile-${Date.now()}`);
bg(chromeLog, chromeBin, [
	'--headless=new', `--remote-debugging-port=${CDP}`, `--user-data-dir=${profile}`,
	'--no-first-run', '--hide-scrollbars', '--disable-gpu', 'about:blank'
]);

const req = (url) => new Promise((res, rej) => http.get(url, (r) => { let d = ''; r.on('data', (c) => (d += c)); r.on('end', () => res(d)); }).on('error', rej));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function waitFor(fn, tries = 150) {
	for (let i = 0; i < tries; i++) {
		try { const v = await fn(); if (v) return v; } catch {}
		await sleep(200);
	}
	throw new Error('timeout');
}

let send, evaluate;
try {
	const info = await waitFor(async () => JSON.parse(await req(`http://127.0.0.1:${CDP}/json`)).find((t) => t.type === 'page' && t.webSocketDebuggerUrl));
	const ws = new WebSocket(info.webSocketDebuggerUrl);
	await new Promise((res, rej) => { ws.addEventListener('open', res, { once: true }); ws.addEventListener('error', rej, { once: true }); });
	let id = 0;
	const pending = new Map();
	ws.addEventListener('message', (e) => {
		const m = JSON.parse(e.data);
		if (m.id && pending.has(m.id)) { const p = pending.get(m.id); pending.delete(m.id); m.error ? p.rej(new Error(JSON.stringify(m.error))) : p.res(m.result); }
	});
	send = (method, params = {}) => new Promise((res, rej) => { const i = ++id; pending.set(i, { res, rej }); ws.send(JSON.stringify({ id: i, method, params })); });
	evaluate = async (expression) => {
		const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
		if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || 'eval error');
		return r.result.value;
	};
	await send('Page.enable');
	await send('Runtime.enable');

	await req(`http://localhost:${PORT}/`).catch(() => {});
	await waitFor(() => req(`http://localhost:${PORT}/`).then(() => true));
	let currentUrl = `http://localhost:${PORT}/`;
	await send('Page.navigate', { url: currentUrl });
	await waitFor(() => evaluate('!!window.__game'));
	log('ready');

	let ok = 0, fail = 0;
	for (const s of SHOTS) {
		if (ONLY && s.name !== ONLY) continue;
		const t = Date.now();
		try {
			const want = `http://localhost:${PORT}/${s.query ? '?' + s.query : ''}`;
			if (want !== currentUrl) {
				await send('Page.navigate', { url: want });
				await waitFor(() => evaluate('!!window.__game'));
				currentUrl = want;
			}
			if (s.viewport) await send('Emulation.setDeviceMetricsOverride', { width: s.viewport[0], height: s.viewport[1], deviceScaleFactor: 1, mobile: false });
			if (s.pre) await evaluate(`(() => { ${s.pre} return 1; })()`);
			await sleep(260);
			let data;
			if (s.mode === 'window') data = (await send('Page.captureScreenshot', { format: 'png' })).data;
			else data = await evaluate(s.mode === 'expr' ? s.expr : "document.getElementById('game').toDataURL('image/png')");
			fs.writeFileSync(path.join(OUT, `${s.name}.png`), Buffer.from(data.split(',')[1] || data, 'base64'));
			if (process.env.SHOTS_DEBUG) {
				const r = await evaluate(
					"(() => { const c = document.getElementById('game'); const b = c.getBoundingClientRect(); return JSON.stringify({ vw: innerWidth, vh: innerHeight, cw: Math.round(b.width), ch: Math.round(b.height), left: Math.round(b.left), top: Math.round(b.top) }); })()"
				);
				log(`       ${r}`);
				const stick = await evaluate(`(() => {
					const el = document.querySelector('.stick');
					if (!el) return 'no .stick';
					el.setPointerCapture = () => {};
					const b = el.getBoundingClientRect();
					const cx = b.left + b.width / 2, cy = b.top + b.height / 2;
					const ev = (t, x, y) => el.dispatchEvent(new PointerEvent(t, { pointerId: 1, clientX: x, clientY: y, bubbles: true, cancelable: true }));
					const out = [];
					for (const [name, fx, fy] of [['right', 0.42, 0], ['up', 0, -0.42], ['down', 0, 0.42], ['left', -0.42, 0]]) {
						ev('pointerdown', cx + b.width * fx, cy + b.height * fy);
						const k = window.__keys;
						out.push(name + '=' + (k[name] ? 'on' : 'off'));
						ev('pointerup', cx + b.width * fx, cy + b.height * fy);
					}
					const knob = document.querySelector('.stick-knob');
					return out.join(' ') + ' | knob-reset=' + (knob && /translate\(0px, 0px\)/.test(knob.style.transform));
				})()`);
				log(`       ${stick}`);
			}
			log(`  ok  ${s.name} ${Date.now() - t}ms`);
			ok++;
		} catch (e) {
			log(` FAIL ${s.name}: ${e.message}`);
			fail++;
		}
	}
	log(`done ${ok}/${ok + fail} in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
	process.exit(fail ? 1 : 0);
} catch (e) {
	log(`ERROR ${e.message}`);
	log('preview:', fs.existsSync(previewLog) ? fs.readFileSync(previewLog, 'utf8').slice(-300) : 'n/a');
	process.exit(1);
}
