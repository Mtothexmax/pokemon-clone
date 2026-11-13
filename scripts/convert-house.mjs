// Converts the big CAD LEGO house (OBJ + MTL colors) into a compact binary with vertex colors,
// simplified with meshoptimizer so it loads fast.
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { MeshoptSimplifier } from 'meshoptimizer';

const SRC = process.argv[2] ?? 'models-src/lego-house/Lego-house.obj';
const MTL = join(dirname(SRC), 'materials.mtl');
const RATIO = Number(process.argv[3] ?? 0.3);

const mtl = {};
let cur;
for (const l of readFileSync(MTL, 'utf8').split(/\r?\n/)) {
	if (l.startsWith('newmtl ')) cur = l.slice(7).trim();
	else if (l.startsWith('Kd ')) mtl[cur] = l.slice(3).trim().split(/\s+/).map(Number);
}
// map CAD colors to friendlier LEGO colors
const remap = ([r, g, b]) => {
	const k = `${r.toFixed(2)},${g.toFixed(2)},${b.toFixed(2)}`;
	const table = {
		'0.99,0.99,0.99': [244, 244, 244], '0.77,0.77,0.77': [160, 165, 169], '0.00,0.50,1.00': [0, 85, 191],
		'0.92,0.84,0.71': [228, 205, 158], '1.00,0.00,0.00': [201, 26, 9], '0.00,1.00,0.25': [75, 159, 74],
		'0.51,0.51,0.51': [108, 110, 104], '0.59,0.35,0.06': [130, 66, 42], '0.29,0.29,0.29': [60, 62, 60], '1.00,1.00,0.00': [242, 205, 55]
	};
	return table[k] ?? [r * 255, g * 255, b * 255];
};

const pos = [], nrm = [];
const outPos = [], outNrm = [], outCol = [];
const index = [];
const map = new Map();
let color = [200, 200, 200];
const text = readFileSync(SRC, 'utf8');
let i = 0;
for (const l of text.split('\n')) {
	const c0 = l.charCodeAt(0), c1 = l.charCodeAt(1);
	if (c0 === 118 && c1 === 32) { const p = l.split(' '); pos.push(+p[1], +p[2], +p[3]); }
	else if (c0 === 118 && c1 === 110) { const p = l.split(' '); nrm.push(+p[1], +p[2], +p[3]); }
	else if (c0 === 117 && l.startsWith('usemtl ')) color = remap(mtl[l.slice(7).trim()] ?? [0.8, 0.8, 0.8]);
	else if (c0 === 102 && c1 === 32) {
		const f = l.trim().split(' ').slice(1).map((s) => {
			const [v, , n] = s.split('/');
			const key = `${v}|${n}|${color[0]}|${color[1]}|${color[2]}`;
			let id = map.get(key);
			if (id === undefined) {
				id = outPos.length / 3;
				map.set(key, id);
				const vi = (+v - 1) * 3, ni = (+n - 1) * 3;
				// Z-up -> Y-up
				outPos.push(pos[vi], pos[vi + 2], -pos[vi + 1]);
				outNrm.push(nrm[ni], nrm[ni + 2], -nrm[ni + 1]);
				outCol.push(color[0], color[1], color[2], 255);
			}
			return id;
		});
		for (let k = 1; k < f.length - 1; k++) index.push(f[0], f[k], f[k + 1]);
	}
	i++;
}
console.log('verts', outPos.length / 3, 'tris', index.length / 3);

await MeshoptSimplifier.ready;
const P = new Float32Array(outPos);
let I = new Uint32Array(index);
const target = Math.floor((I.length * RATIO) / 3) * 3;
const [simp, err] = MeshoptSimplifier.simplify(I, P, 3, target, 0.0008, ['LockBorder']);
console.log('simplified tris', simp.length / 3, 'error', err);
// compact: drop unused vertices
const remapV = new Int32Array(P.length / 3).fill(-1);
const nP = [], nN = [], nC = [];
const nI = new Uint32Array(simp.length);
for (let k = 0; k < simp.length; k++) {
	const v = simp[k];
	if (remapV[v] < 0) {
		remapV[v] = nP.length / 3;
		nP.push(outPos[v * 3], outPos[v * 3 + 1], outPos[v * 3 + 2]);
		nN.push(outNrm[v * 3], outNrm[v * 3 + 1], outNrm[v * 3 + 2]);
		nC.push(...outCol.slice(v * 4, v * 4 + 4));
	}
	nI[k] = remapV[v];
}
// normalize: center on x/z, ground at y=0
let mn = [1e9, 1e9, 1e9], mx = [-1e9, -1e9, -1e9];
for (let k = 0; k < nP.length; k += 3) for (let d = 0; d < 3; d++) { mn[d] = Math.min(mn[d], nP[k + d]); mx[d] = Math.max(mx[d], nP[k + d]); }
// LEGO units: the CAD model is in meters, one stud = 8 mm -> scale 125 makes 1 world unit = 1 stud
const SCALE = 125;
for (let k = 0; k < nP.length; k += 3) {
	nP[k] = (nP[k] - (mn[0] + mx[0]) / 2) * SCALE;
	nP[k + 1] = (nP[k + 1] - mn[1]) * SCALE;
	nP[k + 2] = (nP[k + 2] - (mn[2] + mx[2]) / 2) * SCALE;
}
// shift so the baseplate studs sit on our stud grid (centers at n + 0.5)
const phase = (d) => {
	let c = 0, sn = 0;
	for (let k = 0; k < nP.length; k += 3) {
		const y = nP[k + 1];
		if (y < 0.15 || y > 0.45) continue;
		c += Math.cos(nP[k + d] * 2 * Math.PI);
		sn += Math.sin(nP[k + d] * 2 * Math.PI);
	}
	return Math.atan2(sn, c) / (2 * Math.PI);
};
const sx = 0.5 - phase(0), sz = 0.5 - phase(2);
for (let k = 0; k < nP.length; k += 3) { nP[k] += sx; nP[k + 2] += sz; }
console.log('stud phase shift', sx.toFixed(3), sz.toFixed(3));
console.log('size (studs)', ((mx[0] - mn[0]) * 125).toFixed(2), ((mx[1] - mn[1]) * 125).toFixed(2), ((mx[2] - mn[2]) * 125).toFixed(2));
const n = nP.length / 3;
const parts = [new Uint32Array([n, nI.length, 1]).buffer, new Float32Array(nP).buffer, new Float32Array(nN).buffer, new Uint8Array(nC).buffer, nI.buffer];
const buf = Buffer.concat(parts.map((p) => Buffer.from(p)));
writeFileSync('static/models/lego-house.bin', buf);
console.log('written', n, 'verts', (buf.length / 1024 / 1024).toFixed(1) + 'MB');
