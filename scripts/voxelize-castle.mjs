// Turns the castle of the "lego city" asset (Mecabricks units: 1 stud = 0.8, 1 plate = 0.32) into clean
// LEGO bricks: the original mesh is sampled into a grid of half-stud x plate cells, the inside of the walls
// is filled, equal faces are merged into big rectangles (greedy meshing) and fresh low-poly studs go on
// every free top. Much cleaner than simplifying the CAD mesh, which turns all the little studs into spikes.
//
//   node scripts/voxelize-castle.mjs "<lego city folder>"      -> static/models/castle.bin
import { readFileSync, writeFileSync } from 'node:fs';

const SRC = process.argv[2] ?? 'C:/Users/mtoth/Downloads/lego city';
// part file -> colour (the baseplate, files 142/143, is left out: the town has its own ground)
const PARTS = {
	// the wall files are height layers: one stone grey for all (the per-brick shading shows the bricks)
	148: '#4c4f4b', 150: '#4c4f4b', 152: '#4c4f4b', 149: '#4c4f4b', 151: '#4c4f4b', 153: '#4c4f4b',
	139: '#323431', // battlements, gratings
	145: '#a8936a', 146: '#a8936a', // floor plates, flag
	147: '#5a2c18' // door, cart
};
const SX = 0.4, SY = 0.32, SZ = 0.4; // cell size: half a stud wide, one plate high
const STUD = 0.8;

// ---------- sample every triangle into cells ----------
const tris = [];
for (const [n, hex] of Object.entries(PARTS)) {
	const pos = [];
	for (const l of readFileSync(`${SRC}/model_${n}.obj`, 'utf8').split('\n')) {
		if (l.startsWith('v ')) pos.push(l.split(' ').slice(1, 4).map(Number));
		else if (l.startsWith('f ')) {
			const ids = l.trim().split(/\s+/).slice(1).map((t) => parseInt(t, 10) - 1);
			for (let k = 1; k + 1 < ids.length; k++) tris.push([pos[ids[0]], pos[ids[k]], pos[ids[k + 1]], hex]);
		}
	}
}
const mn = [Infinity, Infinity, Infinity], mx = [-Infinity, -Infinity, -Infinity];
for (const t of tris) for (let k = 0; k < 3; k++) for (let a = 0; a < 3; a++) [mn[a], mx[a]] = [Math.min(mn[a], t[k][a]), Math.max(mx[a], t[k][a])];
const NX = Math.ceil((mx[0] - mn[0]) / SX) + 2, NY = Math.ceil((mx[1] - mn[1]) / SY) + 2, NZ = Math.ceil((mx[2] - mn[2]) / SZ) + 2;
const grid = new Int16Array(NX * NY * NZ).fill(-1); // colour index, -1 empty
const colours = Object.values(PARTS).filter((c, i, a) => a.indexOf(c) === i);
const at = (x, y, z) => (x * NY + y) * NZ + z;
const cell = (p) => [Math.floor((p[0] - mn[0]) / SX) + 1, Math.floor((p[1] - mn[1] + 1e-4) / SY) + 1, Math.floor((p[2] - mn[2]) / SZ) + 1];
for (const [a, b, c, hex] of tris) {
	// points on the triangle closer than half a cell
	const len = (u, v) => Math.hypot(u[0] - v[0], u[1] - v[1], u[2] - v[2]);
	const steps = Math.max(1, Math.ceil(Math.max(len(a, b), len(a, c), len(b, c)) / 0.14));
	const ci = colours.indexOf(hex);
	for (let i = 0; i <= steps; i++)
		for (let j = 0; j <= steps - i; j++) {
			const u = i / steps, v = j / steps, w = 1 - u - v;
			const [x, y, z] = cell([a[0] * w + b[0] * u + c[0] * v, a[1] * w + b[1] * u + c[1] * v, a[2] * w + b[2] * u + c[2] * v]);
			grid[at(x, y, z)] = ci;
		}
}
// ---------- close the hollow LEGO bricks: grow every solid cell by one, then shrink back ----------
// (CAD bricks are hollow underneath; their thin inner tubes would otherwise show as grey stripes)
{
	const N6 = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
	const inGrid = (x, y, z) => x >= 0 && y >= 0 && z >= 0 && x < NX && y < NY && z < NZ;
	const grown = new Int16Array(grid);
	for (let x = 0; x < NX; x++)
		for (let y = 0; y < NY; y++)
			for (let z = 0; z < NZ; z++) {
				if (grid[at(x, y, z)] >= 0) continue;
				for (const [dx, dy, dz] of N6) {
					const X = x + dx, Y = y + dy, Z = z + dz;
					if (inGrid(X, Y, Z) && grid[at(X, Y, Z)] >= 0) {
						grown[at(x, y, z)] = grid[at(X, Y, Z)];
						break;
					}
				}
			}
	let closed = 0;
	for (let x = 0; x < NX; x++)
		for (let y = 0; y < NY; y++)
			for (let z = 0; z < NZ; z++) {
				const i = at(x, y, z);
				if (grid[i] >= 0 || grown[i] < 0) continue;
				// keep a grown cell only if all its neighbours are solid after growing (= it sits inside)
				if (N6.every(([dx, dy, dz]) => inGrid(x + dx, y + dy, z + dz) && grown[at(x + dx, y + dy, z + dz)] >= 0)) {
					grid[i] = grown[i];
					closed++;
				}
			}
	console.log(`${closed} hollow cells closed`);
}
// ---------- fill the inside of the walls: everything the outside air can't reach ----------
const outside = new Uint8Array(NX * NY * NZ);
const stack = [0];
outside[0] = 1;
while (stack.length) {
	const i = stack.pop();
	const z = i % NZ, y = Math.floor(i / NZ) % NY, x = Math.floor(i / NZ / NY);
	for (const [dx, dy, dz] of [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]]) {
		const X = x + dx, Y = y + dy, Z = z + dz;
		if (X < 0 || Y < 0 || Z < 0 || X >= NX || Y >= NY || Z >= NZ) continue;
		const j = at(X, Y, Z);
		if (outside[j] || grid[j] >= 0) continue;
		outside[j] = 1;
		stack.push(j);
	}
}
let filled = 0;
for (let x = 0; x < NX; x++)
	for (let y = 0; y < NY; y++)
		for (let z = 0; z < NZ; z++) {
			const i = at(x, y, z);
			if (grid[i] >= 0 || outside[i]) continue;
			// take the colour of a solid neighbour below or beside
			grid[i] = grid[at(x, Math.max(0, y - 1), z)] >= 0 ? grid[at(x, y - 1, z)] : grid[at(Math.max(0, x - 1), y, z)] >= 0 ? grid[at(x - 1, y, z)] : 0;
			filled++;
		}
const solid = (x, y, z) => x >= 0 && y >= 0 && z >= 0 && x < NX && y < NY && z < NZ && grid[at(x, y, z)] >= 0;
// a little colour variation per brick (2 x 2 cells wide, 3 plates high, rows staggered) so the bricks read
const brickShade = (x, y, z) => {
	const row = Math.floor(y / 3);
	const bx = Math.floor((x + (row % 2)) / 2), bz = Math.floor((z + (row % 2)) / 2);
	const h = ((bx * 73856093) ^ (row * 19349663) ^ (bz * 83492791)) >>> 0;
	return 0.93 + (h % 9) / 100;
};
const rgb = (hex, k) => [0, 2, 4].map((o) => Math.min(255, Math.round(parseInt(hex.slice(1 + o, 3 + o), 16) * k)));

// ---------- greedy meshing: merge equal neighbouring faces into rectangles ----------
const P = [], N = [], C = [], I = [];
const quad = (corners, n, col) => {
	const base = P.length / 3;
	for (const c of corners) {
		P.push(c[0] * SX + mn[0], c[1] * SY + mn[1] - SY, c[2] * SZ + mn[2]); // cell 1 starts at the minimum
		N.push(...n);
		C.push(...col, 255);
	}
	I.push(base, base + 1, base + 2, base, base + 2, base + 3);
};
const dims = [NX, NY, NZ];
for (let d = 0; d < 3; d++) {
	const u = (d + 1) % 3, v = (d + 2) % 3;
	for (const dir of [1, -1]) {
		const n = [0, 0, 0];
		n[d] = dir;
		for (let s = 0; s < dims[d]; s++) {
			// mask of visible faces in this slice: key = colour + shade
			const mask = new Array(dims[u] * dims[v]).fill(null);
			for (let a = 0; a < dims[u]; a++)
				for (let b = 0; b < dims[v]; b++) {
					const p = [0, 0, 0];
					p[d] = s;
					p[u] = a;
					p[v] = b;
					if (!solid(...p)) continue;
					const q = [...p];
					q[d] += dir;
					if (solid(...q)) continue;
					const ci = grid[at(...p)];
					const k = brickShade(...p);
					mask[a * dims[v] + b] = `${ci}|${k.toFixed(2)}`;
				}
			// greedy rectangles
			for (let a = 0; a < dims[u]; a++)
				for (let b = 0; b < dims[v]; ) {
					const key = mask[a * dims[v] + b];
					if (!key) {
						b++;
						continue;
					}
					let w = 1;
					while (b + w < dims[v] && mask[a * dims[v] + b + w] === key) w++;
					let h = 1;
					outer: while (a + h < dims[u]) {
						for (let k = 0; k < w; k++) if (mask[(a + h) * dims[v] + b + k] !== key) break outer;
						h++;
					}
					for (let i = 0; i < h; i++) for (let k = 0; k < w; k++) mask[(a + i) * dims[v] + b + k] = null;
					const [ci, shade] = key.split('|');
					const col = rgb(colours[+ci], +shade);
					const plane = dir > 0 ? s + 1 : s;
					const corner = (da, db) => {
						const c = [0, 0, 0];
						c[d] = plane;
						c[u] = a + da;
						c[v] = b + db;
						return c;
					};
					// counter-clockwise seen from outside
					const cs = [corner(0, 0), corner(h, 0), corner(h, w), corner(0, w)];
					quad(dir > 0 ? cs : cs.reverse(), n, col);
					b += w;
				}
		}
	}
}
// ---------- studs on every free top, on the stud grid ----------
let studs = 0;
const SEG = 8, R = 0.24, H = 0.17;
for (let x = 1; x + 1 < NX; x += 2)
	for (let z = 1; z + 1 < NZ; z += 2)
		for (let y = NY - 2; y >= 1; y--) {
			// a 2 x 2 cell top (one stud) that is solid and free above
			const cells = [[x, z], [x + 1, z], [x, z + 1], [x + 1, z + 1]];
			if (!cells.every(([a, b]) => solid(a, y, b) && !solid(a, y + 1, b))) continue;
			const col = rgb(colours[grid[at(x, y, z)]], brickShade(x, y, z) * 1.03);
			const cx = (x + 1) * SX + mn[0], cz = (z + 1) * SZ + mn[2], y0 = (y + 1) * SY + mn[1] - SY;
			const base = P.length / 3;
			for (let i = 0; i < SEG; i++) {
				const t = (i / SEG) * Math.PI * 2, ox = Math.cos(t), oz = Math.sin(t);
				P.push(cx + ox * R, y0, cz + oz * R, cx + ox * R, y0 + H, cz + oz * R);
				N.push(ox, 0, oz, ox, 0, oz);
				C.push(...col, 255, ...col, 255);
			}
			const top = P.length / 3;
			P.push(cx, y0 + H, cz);
			N.push(0, 1, 0);
			C.push(...col, 255);
			const topRing = P.length / 3;
			for (let i = 0; i < SEG; i++) {
				const t = (i / SEG) * Math.PI * 2;
				P.push(cx + Math.cos(t) * R, y0 + H, cz + Math.sin(t) * R);
				N.push(0, 1, 0);
				C.push(...col, 255);
			}
			for (let i = 0; i < SEG; i++) {
				const a = base + i * 2, b = base + ((i + 1) % SEG) * 2;
				I.push(a, a + 1, b + 1, a, b + 1, b); // side (outward)
				I.push(top, topRing + ((i + 1) % SEG), topRing + i); // top
			}
			studs++;
			break;
		}

// centre the footprint on the origin, ground at y 0 (in asset units; the game scales it)
let x0 = Infinity, x1 = -Infinity, y0 = Infinity, z0 = Infinity, z1 = -Infinity;
for (let i = 0; i < P.length; i += 3) [x0, x1, y0, z0, z1] = [Math.min(x0, P[i]), Math.max(x1, P[i]), Math.min(y0, P[i + 1]), Math.min(z0, P[i + 2]), Math.max(z1, P[i + 2])];
for (let i = 0; i < P.length; i += 3) {
	P[i] -= (x0 + x1) / 2;
	P[i + 1] -= y0;
	P[i + 2] -= (z0 + z1) / 2;
}
const n = P.length / 3;
const buf = Buffer.concat([new Uint32Array([n, I.length, 1]).buffer, new Float32Array(P).buffer, new Float32Array(N).buffer, new Uint8Array(C).buffer, new Uint32Array(I).buffer].map((b) => Buffer.from(b)));
writeFileSync('static/models/castle.bin', buf);
console.log(`grid ${NX}x${NY}x${NZ}, ${filled} inside cells filled, ${studs} studs`);
console.log(`castle ${(x1 - x0).toFixed(1)} x ${(Math.max(...P.filter((_, i) => i % 3 === 1))).toFixed(1)} x ${(z1 - z0).toFixed(1)}: ${I.length / 3} tris, ${(buf.length / 1024).toFixed(0)} KB`);
