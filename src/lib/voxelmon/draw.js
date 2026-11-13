// @ts-nocheck  (plain JS on purpose: easy to read and edit; the game wraps it in src/lib/three/voxel.ts)
// How a Voxelmon is drawn. This file only contains the drawing method; the beasts themselves live in
// ./models/<id>.js as a palette plus rows of pixels (one character per pixel, '.' = empty).
// A model is either a BUILD FUNCTION (`build(v)`, see below), ONE picture (`pixels`, from the card) or
// TWO views (`front` + `side`).
//
// From a build function (the easiest to write and edit):
//   build(v) places voxels with a few commands; x = left/right, y = up (0 = ground), z = forwards (the head
//   looks to +z). Colours are palette keys; use null to cut voxels away again.
//     v.box(x0, y0, z0, x1, y1, z1, c)   fill a block (corners included)
//     v.ball(x, y, z, rx, ry, rz, c)     fill an ellipsoid around (x, y, z)
//     v.dome(x, y, z, rx, ry, rz, c)     the upper half of that ellipsoid (shells, caps, canopies)
//     v.set(x, y, z, c)                  one voxel
//     v.face(x, y, c)                    paint the voxel you see first from the front at (x, y): eyes, mouths
//     v.side(fn)                         run fn(s) for s = -1 and s = 1: both sides, mirrored
//   The card picture is drawn from the same voxels: straight from the front as pixel art (drawPixelPicture);
//   once the card became a real Voxelmon, rendered in 3D from the front (voxelPortrait in src/lib/three/voxel.ts).
//
// From one picture, step by step:
//  1. every pixel is a column of voxels pointing towards the viewer (x = column, y = row, z = depth)
//  2. columns get thicker towards the middle of the body, so the flat picture becomes a round, chunky body
//     (a model may set `depth`: rows of digits 0-9 = half thickness per pixel, to shape it by hand)
//  3. the front half shows the pixel colours; the back half gets the body colour, so eyes and mouths
//     are not painted on the back of the head
//  4. only the outside faces of the voxels become triangles (one mesh, vertex colours)
//
// From two views (`front`: seen from the front; `side`: seen from the side, head on the left; same rows):
//  1. a voxel exists where BOTH views are solid (front gives width x height, side gives length x height),
//     so four legs and a belly come out by themselves; each slice is only as wide as the side view is
//     thick there, so horn, tail and legs stay thin and the edges get round
//  2. the surface you see from the front gets the front view's colours, all other faces the side view's colours
import * as THREE from 'three';

/**
 * @typedef {{ name?: string, palette: Record<string, string>, pixels?: string[], depth?: string[], front?: string[], side?: string[] }} VoxelModel
 */

// the 6 faces of a unit cube: normal + 4 corners (counter-clockwise seen from outside)
const FACES = [
	[[1, 0, 0], [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]]],
	[[-1, 0, 0], [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]]],
	[[0, 1, 0], [[0, 1, 1], [1, 1, 1], [1, 1, 0], [0, 1, 0]]],
	[[0, -1, 0], [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]]],
	[[0, 0, 1], [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]]],
	[[0, 0, -1], [[1, 0, 0], [0, 0, 0], [0, 1, 0], [1, 1, 0]]]
];
/** a little brick-to-brick colour variation, so the voxels read as single bricks */
const jitter = (x, y, z) => 0.94 + (((x * 73856093) ^ (y * 19349663) ^ (z * 83492791)) & 15) / 150;

/** collects quads into one mesh with vertex colours */
function meshBuilder() {
	const pos = [], nor = [], cols = [];
	const tmp = new THREE.Color();
	return {
		/** one face of the voxel whose lower corner is (vx, vy, vz); c = [r, g, b] 0..1 sRGB */
		face(n, quad, vx, vy, vz, c) {
			tmp.setRGB(c[0], c[1], c[2], THREE.SRGBColorSpace);
			const v = quad.map(([a, b, d]) => [vx + a, vy + b, vz + d]);
			for (const i of [0, 1, 2, 0, 2, 3]) {
				pos.push(...v[i]);
				nor.push(...n);
				cols.push(tmp.r, tmp.g, tmp.b);
			}
		},
		geometry(size) {
			const g = new THREE.BufferGeometry();
			g.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
			g.setAttribute('normal', new THREE.Float32BufferAttribute(nor, 3));
			g.setAttribute('color', new THREE.Float32BufferAttribute(cols, 3));
			g.userData.size = size;
			return g;
		}
	};
}

/** half thickness of every pixel column (in voxels), -1 = empty */
export function thickness(model) {
	const { pixels } = model;
	const h = pixels.length, w = Math.max(...pixels.map((r) => r.length));
	const solid = (x, y) => y >= 0 && y < h && x >= 0 && x < w && (pixels[y][x] ?? '.') !== '.';
	const half = new Int16Array(w * h).fill(-1);
	if (model.depth) {
		// hand-made thickness
		for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) if (solid(x, y)) half[y * w + x] = +(model.depth[y]?.[x] ?? 0) || 0;
		return { w, h, half };
	}
	// distance of every pixel to the outline (chamfer 3-4), in two passes
	const dist = new Float32Array(w * h);
	const at = (x, y) => (x < 0 || y < 0 || x >= w || y >= h ? 0 : dist[y * w + x]);
	for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) dist[y * w + x] = solid(x, y) ? 1e6 : 0;
	for (let y = 0; y < h; y++)
		for (let x = 0; x < w; x++)
			if (solid(x, y)) dist[y * w + x] = Math.min(dist[y * w + x], at(x - 1, y) + 3, at(x, y - 1) + 3, at(x - 1, y - 1) + 4, at(x + 1, y - 1) + 4);
	for (let y = h - 1; y >= 0; y--)
		for (let x = w - 1; x >= 0; x--)
			if (solid(x, y)) dist[y * w + x] = Math.min(dist[y * w + x], at(x + 1, y) + 3, at(x, y + 1) + 3, at(x + 1, y + 1) + 4, at(x - 1, y + 1) + 4);
	let R = 1;
	for (let i = 0; i < w * h; i++) R = Math.max(R, dist[i] / 3);
	for (let y = 0; y < h; y++)
		for (let x = 0; x < w; x++) {
			if (!solid(x, y)) continue;
			const e = dist[y * w + x] / 3; // pixels to the edge
			const round = Math.sqrt(e * (2 * R - e)); // cross-section of a circle
			half[y * w + x] = Math.max(0, Math.round(0.95 * Math.min(round, e * 1.8)));
		}
	return { w, h, half };
}

/** pixel colour as [r, g, b] 0..1 (sRGB) */
function rgb(hex) {
	const n = parseInt(hex.slice(1), 16);
	return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

/**
 * Builds the Voxelmon mesh: x to the right, y up, front towards +z; one unit per voxel,
 * origin at the bottom centre. userData.size = { w, h } in voxels.
 * @param {VoxelModel} model
 */
export function drawVoxelmon(model) {
	if (model.build) return drawFromBuild(model);
	return model.front && model.side ? drawFromViews(model) : drawFromPicture(model);
}

/** runs the model's build function; returns the voxels (key "x,y,z" -> palette key) and the bounds */
export function buildVoxels(model) {
	const vox = new Map();
	const k = (x, y, z) => `${Math.round(x)},${Math.round(y)},${Math.round(z)}`;
	const put = (x, y, z, c) => (c == null ? vox.delete(k(x, y, z)) : vox.set(k(x, y, z), c));
	const v = {
		set: put,
		box(x0, y0, z0, x1, y1, z1, c) {
			for (let x = Math.min(x0, x1); x <= Math.max(x0, x1); x++)
				for (let y = Math.min(y0, y1); y <= Math.max(y0, y1); y++) for (let z = Math.min(z0, z1); z <= Math.max(z0, z1); z++) put(x, y, z, c);
		},
		ball(cx, cy, cz, rx, ry, rz, c, upperOnly = false) {
			for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++)
				for (let y = Math.floor(upperOnly ? cy : cy - ry); y <= Math.ceil(cy + ry); y++)
					for (let z = Math.floor(cz - rz); z <= Math.ceil(cz + rz); z++)
						if (((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 + ((z - cz) / rz) ** 2 <= 1) put(x, y, z, c);
		},
		dome(cx, cy, cz, rx, ry, rz, c) {
			v.ball(cx, cy, cz, rx, ry, rz, c, true);
		},
		face(x, y, c) {
			let best = null;
			for (const key of vox.keys()) {
				const [vx, vy, vz] = key.split(',').map(Number);
				if (vx === x && vy === y && (best === null || vz > best)) best = vz;
			}
			if (best !== null) put(x, y, best, c);
		},
		side(fn) {
			fn(-1);
			fn(1);
		}
	};
	model.build(v);
	const min = [Infinity, Infinity, Infinity], max = [-Infinity, -Infinity, -Infinity];
	for (const key of vox.keys()) key.split(',').forEach((n, i) => ((min[i] = Math.min(min[i], +n)), (max[i] = Math.max(max[i], +n))));
	return { vox, min, max };
}

/** build function -> mesh (only the outside faces) */
function drawFromBuild(model) {
	const { vox, min, max } = buildVoxels(model);
	const m = meshBuilder();
	const cx = (min[0] + max[0] + 1) / 2, cz = (min[2] + max[2] + 1) / 2;
	for (const [key, c] of vox) {
		const [x, y, z] = key.split(',').map(Number);
		const base = rgb(model.palette[c] ?? '#ff00ff');
		const j = jitter(x, y, z);
		for (const [n, quad] of FACES) {
			if (vox.has(`${x + n[0]},${y + n[1]},${z + n[2]}`)) continue;
			m.face(n, quad, x - cx, y - min[1], z - cz, [base[0] * j, base[1] * j, base[2] * j]);
		}
	}
	return m.geometry({ w: max[0] - min[0] + 1, h: max[1] - min[1] + 1 });
}

/**
 * The card picture before the beast is a real Voxelmon: the voxels seen exactly from the front, one pixel
 * per voxel, like a pixel-art sprite (so nobody can tell it will be 3D one day). Each pixel shows the
 * frontmost voxel, a little darker the further back it sits, with a dark outline around the creature.
 */
export function drawPixelPicture(model, ctx, W, H) {
	const { vox, min, max } = buildVoxels(model);
	const w = max[0] - min[0] + 1, h = max[1] - min[1] + 1;
	// frontmost voxel per pixel
	const front = new Map();
	for (const [key, c] of vox) {
		const [x, y, z] = key.split(',').map(Number);
		const k = `${x},${y}`;
		const f = front.get(k);
		if (!f || z > f.z) front.set(k, { z, c });
	}
	const at = (x, y) => front.get(`${x},${y}`);
	// whole pixels, as big as fits
	const s = Math.max(1, Math.floor(Math.min((W * 0.86) / (w + 2), (H * 0.84) / (h + 2))));
	const ox = Math.round((W - w * s) / 2), oy = Math.round(H - 14 - h * s);
	const px = (x, y, fill) => {
		ctx.fillStyle = fill;
		ctx.fillRect(ox + (x - min[0]) * s, oy + (max[1] - y) * s, s, s);
	};
	const tone = (hex, k) => {
		const [r, g, b] = rgb(hex);
		return `rgb(${Math.round(Math.min(1, r * k) * 255)},${Math.round(Math.min(1, g * k) * 255)},${Math.round(Math.min(1, b * k) * 255)})`;
	};
	// outline: every empty pixel next to the creature gets a dark version of its neighbour
	for (let y = min[1] - 1; y <= max[1] + 1; y++)
		for (let x = min[0] - 1; x <= max[0] + 1; x++) {
			if (at(x, y)) continue;
			const n = at(x + 1, y) ?? at(x - 1, y) ?? at(x, y + 1) ?? at(x, y - 1);
			if (n) px(x, y, tone(model.palette[n.c] ?? '#ff00ff', 0.35));
		}
	for (const [k, f] of front) {
		const [x, y] = k.split(',').map(Number);
		// light from the top left; parts further back are a bit darker
		const depth = Math.max(0.72, 1 - (max[2] - f.z) * 0.035);
		const light = !at(x, y + 1) ? 1.12 : !at(x - 1, y) ? 1.05 : !at(x, y - 1) || !at(x + 1, y) ? 0.88 : 1;
		px(x, y, tone(model.palette[f.c] ?? '#ff00ff', depth * light));
	}
}

/** distance of every solid cell to the outline (in cells, chamfer 3-4), 0 outside */
function distanceField(w, h, solid) {
	const dist = new Float32Array(w * h);
	const at = (x, y) => (x < 0 || y < 0 || x >= w || y >= h ? 0 : dist[y * w + x]);
	for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) dist[y * w + x] = solid(x, y) ? 1e6 : 0;
	for (let y = 0; y < h; y++)
		for (let x = 0; x < w; x++)
			if (solid(x, y)) dist[y * w + x] = Math.min(dist[y * w + x], at(x - 1, y) + 3, at(x, y - 1) + 3, at(x - 1, y - 1) + 4, at(x + 1, y - 1) + 4);
	for (let y = h - 1; y >= 0; y--)
		for (let x = w - 1; x >= 0; x--)
			if (solid(x, y)) dist[y * w + x] = Math.min(dist[y * w + x], at(x + 1, y) + 3, at(x, y + 1) + 3, at(x + 1, y + 1) + 4, at(x - 1, y + 1) + 4);
	for (let i = 0; i < w * h; i++) dist[i] /= 3;
	return dist;
}

/**
 * two views: carve the body where front AND side are solid. Each slice of the side view is only as wide as
 * the side view is thick there, so horn, tail and legs stay thin and the body gets round edges.
 */
function drawFromViews(model) {
	const { front, side } = model;
	const H = Math.max(front.length, side.length);
	const W = Math.max(...front.map((r) => r.length)), D = Math.max(...side.map((r) => r.length));
	// rows are aligned at the bottom (the feet)
	const at = (rows, col, y) => rows[rows.length - 1 - y]?.[col] ?? '.';
	const solidF = (x, y) => at(front, x, y) !== '.';
	const solidS = (c, y) => at(side, c, y) !== '.';
	// how thick the side view is at every cell (thin horn / tail / legs -> small)
	const thickS = distanceField(D, H, solidS);
	// the middles of the pieces of the front view (each leg, the body, a vine) per row
	const mids = [];
	for (let y = 0; y < H; y++) {
		const list = [];
		for (let x = 0; x < W; ) {
			if (!solidF(x, y)) {
				x++;
				continue;
			}
			let e = x;
			while (e < W && solidF(e, y)) e++;
			list.push((x + e - 1) / 2);
			x = e;
		}
		mids.push(list);
	}
	// a slice may grow sideways around the pieces of its own row and of the rows just below,
	// so legs reach up under the armour instead of being centred
	const BELOW = 8;
	const near = new Float32Array(W * H);
	for (let y = 0; y < H; y++)
		for (let x = 0; x < W; x++) {
			let best = Infinity;
			for (let yy = y; yy >= Math.max(0, y - BELOW); yy--) for (const m of mids[yy]) best = Math.min(best, Math.abs(x - m));
			near[y * W + x] = best;
		}
	const ROUND = 1.25; // how far a slice may reach sideways per cell of side thickness
	// side column c: the head is on the left, and the head points to +z
	const filled = (x, y, c) =>
		x >= 0 && x < W && y >= 0 && y < H && c >= 0 && c < D && solidF(x, y) && solidS(c, y) && near[y * W + x] <= Math.max(0.5, thickS[y * D + c] * ROUND);
	const col = (k) => rgb(model.palette[k] ?? '#ff00ff');
	// what the front view really shows: the first voxel seen from the front (smallest side column)
	const firstSeen = new Int16Array(W * H).fill(-1);
	for (let y = 0; y < H; y++)
		for (let x = 0; x < W; x++)
			for (let c = 0; c < D; c++)
				if (filled(x, y, c)) {
					firstSeen[y * W + x] = c;
					break;
				}
	const m = meshBuilder();
	for (let y = 0; y < H; y++)
		for (let c = 0; c < D; c++) {
			if (!solidS(c, y)) continue;
			for (let x = 0; x < W; x++) {
				if (!filled(x, y, c)) continue;
				const vx = x - W / 2, vy = y, vz = D / 2 - 1 - c;
				const k = jitter(x, y, c);
				for (const [n, quad] of FACES) {
					// +z is towards the head = one column further left in the side view
					if (filled(x + n[0], y + n[1], c - n[2])) continue;
					// only the surface you see from the front gets the front picture; everything else the side picture
					const base = n[2] > 0 && firstSeen[y * W + x] === c ? col(at(front, x, y)) : col(at(side, c, y));
					m.face(n, quad, vx, vy, vz, [base[0] * k, base[1] * k, base[2] * k]);
				}
			}
		}
	return m.geometry({ w: W, h: H });
}

/** one picture: inflate it into a rounded body */
function drawFromPicture(model) {
	const { w, h, half } = thickness(model);
	const key = (x, y) => model.pixels[y]?.[x] ?? '.';
	const col = (x, y) => rgb(model.palette[key(x, y)] ?? '#ff00ff');
	const lum = (c) => 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
	const mid = (x, y) => key(x, y) !== '.' && lum(col(x, y)) > 0.16 && lum(col(x, y)) < 0.86;

	// back colour: dark / very light details inside the body take the body colour around them
	const back = (x, y) => {
		const c = col(x, y);
		if (mid(x, y) || half[y * w + x] <= 0) return c;
		let r = 0, g = 0, b = 0, n = 0;
		for (let rad = 2; rad <= 6 && n < 3; rad += 2)
			for (let dy = -rad; dy <= rad; dy++)
				for (let dx = -rad; dx <= rad; dx++) {
					if (!mid(x + dx, y + dy)) continue;
					const cj = col(x + dx, y + dy);
					r += cj[0];
					g += cj[1];
					b += cj[2];
					n++;
				}
		return n ? [r / n, g / n, b / n] : c;
	};

	const filled = (x, y, z) => x >= 0 && y >= 0 && x < w && y < h && half[y * w + x] >= 0 && Math.abs(z) <= half[y * w + x];
	const m = meshBuilder();
	for (let y = 0; y < h; y++)
		for (let x = 0; x < w; x++) {
			const hh = half[y * w + x];
			if (hh < 0) continue;
			const front = col(x, y), behind = back(x, y);
			for (let z = -hh; z <= hh; z++) {
				const vx = x - w / 2, vy = h - 1 - y, vz = z - 0.5;
				const c = z >= 0 ? front : behind;
				// darker towards the back
				const k = jitter(x, y, z) * (z < -hh / 2 ? 0.9 : 1);
				for (const [n, quad] of FACES) {
					if (filled(x + n[0], y - n[1], z + n[2])) continue;
					m.face(n, quad, vx, vy, vz, [c[0] * k, c[1] * k, c[2] * k]);
				}
			}
		}
	return m.geometry({ w, h });
}
