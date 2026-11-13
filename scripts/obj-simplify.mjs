// Shrinks big OBJ files before anyone looks at them: welds duplicate vertices, then lets meshoptimizer
// collapse edges down to a fraction of the triangles (positions stay in the original coordinates).
// Writes small OBJs (positions + faces only) and a stats line per file.
//
//   node scripts/obj-simplify.mjs <in dir> <out dir> [--ratio=0.08] [--error=0.01]
//
// --ratio: target share of triangles to keep; --error: allowed shape change, relative to the model size.
import { readFileSync, writeFileSync, readdirSync, mkdirSync } from 'node:fs';
import { MeshoptSimplifier } from 'meshoptimizer';

await MeshoptSimplifier.ready;
const [inDir, outDir] = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const opt = (k, d) => +(process.argv.find((a) => a.startsWith(`--${k}=`))?.split('=')[1] ?? d);
const RATIO = opt('ratio', 0.08);
const ERROR = opt('error', 0.01);
if (!inDir || !outDir) {
	console.error('usage: node scripts/obj-simplify.mjs <in dir> <out dir> [--ratio=0.08] [--error=0.01]');
	process.exit(1);
}
mkdirSync(outDir, { recursive: true });

/** positions + triangle indices from an OBJ (polygons are fanned into triangles) */
function readObj(text) {
	const pos = [];
	const tris = [];
	for (const l of text.split('\n')) {
		if (l.charCodeAt(0) === 118 && l.charCodeAt(1) === 32) {
			const p = l.split(' ');
			pos.push(+p[1], +p[2], +p[3]);
		} else if (l.charCodeAt(0) === 102 && l.charCodeAt(1) === 32) {
			const ids = l.trim().split(/\s+/).slice(1).map((t) => parseInt(t, 10) - 1);
			for (let k = 1; k + 1 < ids.length; k++) tris.push(ids[0], ids[k], ids[k + 1]);
		}
	}
	return { pos, tris };
}

/** merges vertices closer than eps (CAD exports repeat every corner per face) */
function weld(pos, tris, eps = 1e-4) {
	const map = new Map();
	const out = [];
	const remap = new Uint32Array(pos.length / 3);
	for (let i = 0; i < pos.length / 3; i++) {
		const key = `${Math.round(pos[i * 3] / eps)},${Math.round(pos[i * 3 + 1] / eps)},${Math.round(pos[i * 3 + 2] / eps)}`;
		let j = map.get(key);
		if (j === undefined) {
			j = out.length / 3;
			map.set(key, j);
			out.push(pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]);
		}
		remap[i] = j;
	}
	const idx = [];
	for (let t = 0; t < tris.length; t += 3) {
		const a = remap[tris[t]], b = remap[tris[t + 1]], c = remap[tris[t + 2]];
		if (a !== b && b !== c && a !== c) idx.push(a, b, c); // drop degenerate triangles
	}
	return { pos: new Float32Array(out), idx: new Uint32Array(idx) };
}

const files = readdirSync(inDir).filter((f) => f.toLowerCase().endsWith('.obj'));
let totalIn = 0, totalOut = 0;
for (const f of files) {
	const { pos, tris } = readObj(readFileSync(`${inDir}/${f}`, 'utf8'));
	const w = weld(pos, tris);
	const before = w.idx.length / 3;
	const target = Math.max(12, Math.floor((w.idx.length * RATIO) / 3) * 3);
	const [idx, err] = before ? MeshoptSimplifier.simplify(w.idx, w.pos, 3, target, ERROR) : [w.idx, 0];
	// keep only the vertices still used
	const used = new Map();
	const vs = [];
	const fs = [];
	for (let t = 0; t < idx.length; t += 3) {
		const face = [];
		for (let k = 0; k < 3; k++) {
			const i = idx[t + k];
			if (!used.has(i)) {
				used.set(i, used.size + 1);
				vs.push(`v ${w.pos[i * 3].toFixed(4)} ${w.pos[i * 3 + 1].toFixed(4)} ${w.pos[i * 3 + 2].toFixed(4)}`);
			}
			face.push(used.get(i));
		}
		fs.push(`f ${face.join(' ')}`);
	}
	writeFileSync(`${outDir}/${f}`, `# ${f} simplified: ${before} -> ${idx.length / 3} triangles\n${vs.join('\n')}\n${fs.join('\n')}\n`);
	totalIn += before;
	totalOut += idx.length / 3;
	console.log(`${f}: ${before} -> ${idx.length / 3} tris (error ${err.toFixed(4)})`);
}
console.log(`total: ${totalIn} -> ${totalOut} triangles in ${files.length} files`);
