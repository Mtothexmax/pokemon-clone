// Converts the source OBJ models into a compact binary format (.bin) for fast loading.
// Layout: u32 vertexCount, u32 indexCount, u32 flags(1=color,2=uv), f32 pos[3n], f32 normal[3n],
//         u8 color[4n] (if flag 1), f32 uv[2n] (if flag 2), u32 index[m]
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import * as THREE from 'three';
import { OBJLoader } from 'three/addons/loaders/OBJLoader.js';
import { mergeVertices, mergeGeometries, toCreasedNormals } from 'three/addons/utils/BufferGeometryUtils.js';
import { MeshoptSimplifier } from 'meshoptimizer';

await MeshoptSimplifier.ready;
/**
 * Polygon reduction: weld the vertices, let meshoptimizer collapse edges down to `ratio` of the triangles
 * (stopping early when the shape would change by more than `maxErr`, relative to the model size), then
 * rebuild normals with a crease angle so LEGO edges stay sharp. Returns a position+normal geometry.
 */
function simplify(g, ratio, maxErr = 0.02, label = '') {
	let m = new THREE.BufferGeometry();
	m.setAttribute('position', g.attributes.position.clone());
	if (g.index) m.setIndex(g.index.clone());
	m = mergeVertices(m, 1e-4);
	const before = m.index.count / 3;
	const target = Math.floor((m.index.count * ratio) / 3) * 3;
	const [idx, err] = MeshoptSimplifier.simplify(new Uint32Array(m.index.array), new Float32Array(m.attributes.position.array), 3, target, maxErr);
	m.setIndex(new THREE.BufferAttribute(idx, 1));
	const out = toCreasedNormals(m.toNonIndexed(), (35 * Math.PI) / 180);
	if (label) console.log(`simplify ${label}: ${before} -> ${idx.length / 3} tris (error ${err.toFixed(4)})`);
	return out;
}

function load(file) {
	const obj = new OBJLoader().parse(readFileSync(file, 'utf8'));
	const geos = [];
	obj.traverse((o) => {
		if (o.isMesh) {
			const g = o.geometry.clone();
			for (const k of Object.keys(g.attributes)) if (!['position', 'normal', 'uv'].includes(k)) g.deleteAttribute(k);
			geos.push(g);
		}
	});
	return mergeGeometries(geos);
}

function normalize(g, height, zUp) {
	if (zUp) g.rotateX(-Math.PI / 2);
	g.computeBoundingBox();
	const b = g.boundingBox;
	const s = height / (b.max.y - b.min.y);
	g.translate(-(b.min.x + b.max.x) / 2, -b.min.y, -(b.min.z + b.max.z) / 2);
	g.scale(s, s, s);
	return g;
}

function colorize(g, trunkFrac, trunkCol, leafCol, baseCol) {
	const pos = g.attributes.position;
	g.computeBoundingBox();
	const H = g.boundingBox.max.y;
	const col = new Uint8Array(pos.count * 4);
	const t = new THREE.Color(trunkCol), l = new THREE.Color(leafCol), bc = new THREE.Color(baseCol);
	for (let i = 0; i < pos.count; i++) {
		const h = pos.getY(i) / H;
		const c = h < 0.095 ? bc : h < trunkFrac ? t : l;
		col.set([c.r * 255, c.g * 255, c.b * 255, 255], i * 4);
	}
	g.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
	return g;
}

function write(g, out, { color, uv, groups }) {
	if (!uv) g.deleteAttribute('uv');
	g = mergeVertices(g, 1e-4);
	const n = g.attributes.position.count;
	const idx = g.index.array;
	const parts = [new Uint32Array([n, idx.length, (color ? 1 : 0) | (uv ? 2 : 0) | (groups ? 4 : 0)]).buffer, new Float32Array(g.attributes.position.array).buffer, new Float32Array(g.attributes.normal.array).buffer];
	if (color) parts.push(new Uint8Array(g.attributes.color.array).buffer);
	if (uv) parts.push(new Float32Array(g.attributes.uv.array).buffer);
	parts.push(new Uint32Array(idx).buffer);
	if (groups) parts.push(new Uint32Array([groups.length, ...groups.flatMap((q) => [q.start, q.count, q.mat])]).buffer);
	const buf = Buffer.concat(parts.map((p) => Buffer.from(p)));
	writeFileSync(out, buf);
	console.log(out, n, 'verts', idx.length / 3, 'tris', (buf.length / 1024).toFixed(0) + 'KB');
}

const round = colorize(simplify(normalize(load('models-src/round-tree.obj'), 1, false), 0.25, 0.01, 'round tree'), 0.27, '#82422a', '#4b9f4a', '#82422a');
write(round, 'static/models/round-tree.bin', { color: true });
const spruce = colorize(simplify(normalize(load('models-src/spruce-tree.obj'), 1, false), 0.3, 0.01, 'spruce'), 0.245, '#82422a', '#237841', '#82422a');
write(spruce, 'static/models/spruce-tree.bin', { color: true });
const bush = normalize(load('models-src/bush.obj'), 1, true);
write(bush, 'static/models/bush.bin', { uv: true });

// ---------------- LEGO 30176 bamboo leaves ----------------
const bamboo = normalize(load('models-src/bamboo.obj'), 1, true);
write(bamboo, 'static/models/bamboo.bin', { uv: true });

// ---------------- Minifigure (split into animatable parts) ----------------
// The source mesh is Z-up and faces +Y; its UVs are a front projection of the drawing in figure.png
// (left figure: red torso/arms, blue legs). We split triangles into parts by where their UV lands
// in that drawing, so arms and legs can swing around their own pivots.
{
	const FIG_H = 4.9;
	const src = load('models-src/figure/figure.obj');
	src.rotateX(-Math.PI / 2); // Z-up -> Y-up (front now -Z)
	src.rotateY(Math.PI); // front -> +Z
	src.computeBoundingBox();
	const bb = src.boundingBox;
	const s = FIG_H / (bb.max.y - bb.min.y);
	src.translate(-(bb.min.x + bb.max.x) / 2, -bb.min.y, -(bb.min.z + bb.max.z) / 2);
	src.scale(s, s, s);
	// (the source triangles are already wound counter-clockwise = outward; no flip needed)
	src.computeVertexNormals();

	const P = src.attributes.position, U = src.attributes.uv;
	const px = (i) => [U.getX(i) * 1024, (1 - U.getY(i)) * 512];
	// torso trapezoid in drawing pixels
	const torsoHalf = (y) => {
		const t = Math.min(1, Math.max(0, (y - 150) / 155));
		return { l: 150 + (105 - 150) * t, r: 370 + (405 - 370) * t };
	};
	// material slots: 0 torso/sleeve, 1 legs/hip, 2 skin, 3 torso print (front), 4 face (front)
	const parts = { head: [], torso: [], armL: [], armR: [], legL: [], legR: [] };
	const a = new THREE.Vector3(), b = new THREE.Vector3(), c = new THREE.Vector3();
	const e1 = new THREE.Vector3(), e2 = new THREE.Vector3();
	for (let t = 0; t < P.count; t += 3) {
		let cx = 0, cy = 0;
		for (let k = 0; k < 3; k++) {
			const [x, y] = px(t + k);
			cx += x / 3;
			cy += y / 3;
		}
		a.fromBufferAttribute(P, t);
		b.fromBufferAttribute(P, t + 1);
		c.fromBufferAttribute(P, t + 2);
		const n = e1.subVectors(b, a).cross(e2.subVectors(c, a)).normalize();
		const my = (a.y + b.y + c.y) / 3, mx = (a.x + b.x + c.x) / 3;
		let part, mat;
		if (cy < 150) {
			part = 'head';
			mat = n.z > 0.2 && my > 3.5 && my < 4.65 ? 4 : 2;
		} else if (cy > 305 && cx > 100 && cx < 410) {
			if (my > 1.55) {
				part = 'torso';
				mat = 1; // hip piece
			} else {
				part = mx < 0 ? 'legL' : 'legR';
				mat = 1;
			}
		} else {
			const h = torsoHalf(cy);
			if (cx < h.l - 2 || cx > h.r + 2) {
				part = mx < 0 ? 'armL' : 'armR';
				mat = cy > 290 ? 2 : 0; // hand incl. wrist peg = skin
			} else {
				part = 'torso';
				mat = n.z > 0.55 && my > 1.95 ? 3 : 0;
			}
		}
		// the source arms and legs face backwards (feet/hands point to -z): mirror them in z and keep the winding
		if (part.startsWith('arm') || part.startsWith('leg')) parts[part].push({ pos: [a.x, a.y, -a.z, c.x, c.y, -c.z, b.x, b.y, -b.z], mat });
		else parts[part].push({ pos: [a.x, a.y, a.z, b.x, b.y, b.z, c.x, c.y, c.z], mat });
	}
	// arms: the source mesh uses long thin triangles running from the sleeve into the hand, so colouring whole
	// triangles gives a saw-tooth seam. Cut every arm triangle at the cuff height instead:
	// above = sleeve colour, below = hand (skin).
	// the cuff is slanted, so fit a plane through the real rim of the sleeve (per arm)
	let plane = { a: 2.06, b: 0, c: 0 };
	const side = (p) => p[1] - (plane.a + plane.b * p[0] + plane.c * p[2]);
	const lerp = (p, q, t) => [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, p[2] + (q[2] - p[2]) * t];
	const clip = (poly, keepAbove) => {
		const out = [];
		for (let i = 0; i < poly.length; i++) {
			const p = poly[i], q = poly[(i + 1) % poly.length];
			const sp = side(p), sq = side(q);
			const pin = keepAbove ? sp >= 0 : sp <= 0;
			const qin = keepAbove ? sq >= 0 : sq <= 0;
			if (pin) out.push(p);
			if (pin !== qin) out.push(lerp(p, q, sp / (sp - sq)));
		}
		return out;
	};
	const fitRim = (tris) => {
		// short sleeve triangles (no long slivers) near the lower end of the sleeve
		const pts = [];
		for (const t of tris) {
			if (t.mat !== 0) continue;
			const ys = [t.pos[1], t.pos[4], t.pos[7]];
			if (Math.max(...ys) - Math.min(...ys) > 0.25) continue;
			for (let k = 0; k < 9; k += 3) if (t.pos[k + 1] < 2.45) pts.push([t.pos[k], t.pos[k + 1], t.pos[k + 2]]);
		}
		const cx = pts.reduce((s, p) => s + p[0], 0) / pts.length, cz = pts.reduce((s, p) => s + p[2], 0) / pts.length;
		// lowest point in each of 24 directions around the arm = the rim
		const bins = new Map();
		for (const p of pts) {
			const k = Math.floor(((Math.atan2(p[2] - cz, p[0] - cx) + Math.PI) / (2 * Math.PI)) * 24);
			if (!bins.has(k) || bins.get(k)[1] > p[1]) bins.set(k, p);
		}
		const rim = [...bins.values()];
		// least squares y = a + b x + c z
		let n = rim.length, sx = 0, sz = 0, sy = 0, sxx = 0, szz = 0, sxz = 0, sxy = 0, szy = 0;
		for (const [x, y, z] of rim) { sx += x; sz += z; sy += y; sxx += x * x; szz += z * z; sxz += x * z; sxy += x * y; szy += z * y; }
		const M = [[n, sx, sz], [sx, sxx, sxz], [sz, sxz, szz]], v = [sy, sxy, szy];
		const det = (m) => m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
		const D = det(M);
		const col = (i) => M.map((r, j) => r.map((x, k) => (k === i ? v[j] : x)));
		return { a: det(col(0)) / D + 0.14, b: det(col(1)) / D, c: det(col(2)) / D, n }; // +0.14: up to the visible outer cuff
	};
	for (const arm of ['armL', 'armR']) {
		plane = fitRim(parts[arm]);
		console.log(arm, 'cuff plane', plane.a.toFixed(3), plane.b.toFixed(3), plane.c.toFixed(3), 'rim pts', plane.n);
		const next = [];
		for (const t of parts[arm]) {
			// short triangles keep their colour from the drawing (clean shapes: the C hand, the sleeve);
			// only the long slivers that run from the sleeve into the hand get cut at the rim
			// short hand triangles (the C hand itself) stay skin even where they rise above the rim;
			// everything else is cut at the rim: above = sleeve, below = skin
			const ys = [t.pos[1], t.pos[4], t.pos[7]];
			if (t.mat === 2 && Math.max(...ys) - Math.min(...ys) < 0.22) {
				next.push(t);
				continue;
			}
			const poly = [t.pos.slice(0, 3), t.pos.slice(3, 6), t.pos.slice(6, 9)];
			for (const [keepAbove, mat] of [[true, 0], [false, 2]]) {
				const piece = clip(poly, keepAbove);
				for (let k = 1; k + 1 < piece.length; k++) next.push({ pos: [...piece[0], ...piece[k], ...piece[k + 1]], mat });
			}
		}
		parts[arm] = next;
	}
	const meta = {};
	for (const [name, tris] of Object.entries(parts)) {
		tris.sort((p, q) => p.mat - q.mat);
		// planar front UVs for the print/face groups
		const bounds = {};
		for (const t of tris) {
			if (t.mat < 3) continue;
			const bb = (bounds[t.mat] ??= { x0: 1e9, x1: -1e9, y0: 1e9, y1: -1e9 });
			for (let k = 0; k < 9; k += 3) {
				bb.x0 = Math.min(bb.x0, t.pos[k]);
				bb.x1 = Math.max(bb.x1, t.pos[k]);
				bb.y0 = Math.min(bb.y0, t.pos[k + 1]);
				bb.y1 = Math.max(bb.y1, t.pos[k + 1]);
			}
		}
		const uv = [];
		for (const t of tris) {
			const bb = bounds[t.mat];
			for (let k = 0; k < 9; k += 3) {
				if (!bb) uv.push(0, 0);
				else uv.push((t.pos[k] - bb.x0) / (bb.x1 - bb.x0), (t.pos[k + 1] - bb.y0) / (bb.y1 - bb.y0));
			}
		}
		const groups = [];
		tris.forEach((t, i) => {
			const last = groups[groups.length - 1];
			if (last && last.mat === t.mat) last.count += 3;
			else groups.push({ start: i * 3, count: 3, mat: t.mat });
		});
		const g = new THREE.BufferGeometry();
		g.setAttribute('position', new THREE.Float32BufferAttribute(tris.flatMap((t) => t.pos), 3));
		g.setAttribute('uv', new THREE.Float32BufferAttribute(uv, 2));
		g.computeVertexNormals();
		g.computeBoundingBox();
		const { min, max } = g.boundingBox;
		meta[name] = { min: min.toArray().map((x) => +x.toFixed(3)), max: max.toArray().map((x) => +x.toFixed(3)), groups: groups.map((q) => [q.mat, q.count / 3]), bounds };
		write(g, `static/models/fig-${name}.bin`, { uv: true, groups });
	}
	writeFileSync('static/models/figure.json', JSON.stringify(meta, null, 1));
	console.log(meta);
}

// ---------------- LEGO flowers in a pot (3 parts -> one vertex-colored model) ----------------
// Source is already in stud units (the pot is 4 studs wide, 2 bricks tall).
{
	const parts = [
		['models-src/flowerpot/flower pot.obj', () => '#a0522d'],
		['models-src/flowerpot/flower bottom green.obj', () => '#237841'],
		['models-src/flowerpot/flower heads.obj', null] // one color per flower head
	];
	const heads = ['#c91a09', '#f2cd37', '#0055bf', '#fe8a18', '#ff8ac0', '#f4f4f4'];
	const geos = [];
	for (const [file, colorFn] of parts) {
		let g = simplify(load(file), colorFn ? 0.35 : 0.45, 0.006, `flowerpot ${file.split('/').pop()}`);
		// the source normals of the blossoms are flipped in places (renders black/white): rebuild them from the faces
		if (!colorFn) g.computeVertexNormals();
		const P = g.attributes.position;
		const col = new Uint8Array(P.count * 4);
		if (colorFn) {
			const c = new THREE.Color(colorFn());
			for (let i = 0; i < P.count; i++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], i * 4);
		} else {
			// flower heads are welded together: find blossoms with k-means on 3D triangle centroids
			const cents = [];
			for (let t = 0; t < P.count; t += 3)
				cents.push([0, 1, 2].map((d) => (P.getComponent(t, d) + P.getComponent(t + 1, d) + P.getComponent(t + 2, d)) / 3));
			const d2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2;
			const kmeans = (k) => {
				// farthest-point initialisation
				let centers = [cents[0]];
				while (centers.length < k) {
					let far = cents[0], fd = -1;
					for (const c of cents) {
						const d = Math.min(...centers.map((k2) => d2(k2, c)));
						if (d > fd) { fd = d; far = c; }
					}
					centers.push(far);
				}
				const asg = new Int32Array(cents.length);
				for (let it = 0; it < 30; it++) {
					cents.forEach((c, i) => {
						let best = 0, bd = Infinity;
						centers.forEach((q, j) => { const d = d2(q, c); if (d < bd) { bd = d; best = j; } });
						asg[i] = best;
					});
					const sum = centers.map(() => [0, 0, 0, 0]);
					cents.forEach((c, i) => { const q = sum[asg[i]]; q[0] += c[0]; q[1] += c[1]; q[2] += c[2]; q[3]++; });
					centers = sum.map((q, j) => (q[3] ? [q[0] / q[3], q[1] / q[3], q[2] / q[3]] : centers[j]));
				}
				// spread = largest distance of any triangle to its blossom centre
				let spread = 0;
				cents.forEach((c, i) => (spread = Math.max(spread, Math.sqrt(d2(c, centers[asg[i]])))));
				return { asg, centers, spread };
			};
			let res = kmeans(2);
			for (let k = 2; k <= 12; k++) {
				res = kmeans(k);
				console.log('  k', k, 'spread', res.spread.toFixed(2));
				if (res.spread < 0.95) break;
			}
			const assign = res.asg;
			const centers = res.centers;
			cents.forEach((_, i) => {
				const c = new THREE.Color(heads[assign[i] % heads.length]);
				for (let k = 0; k < 3; k++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], (i * 3 + k) * 4);
			});
			const ids = { size: centers.length };
			console.log('flower heads:', ids.size);
		}
		g.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
		g.deleteAttribute('uv');
		geos.push(g);
	}
	const g = mergeGeometries(geos);
	g.computeBoundingBox();
	const b = g.boundingBox;
	g.translate(-(b.min.x + b.max.x) / 2, -b.min.y, -(b.min.z + b.max.z) / 2);
	g.computeBoundingBox();
	const sz = g.boundingBox.getSize(new THREE.Vector3());
	console.log('flowerpot size', sz.x.toFixed(2), sz.y.toFixed(2), sz.z.toFixed(2));
	write(g, 'static/models/flowerpot.bin', { color: true });
}

// ---------------- the pot's flowers without the pot (prepared in Blender: models-src/flower-only) ----------------
// model_0 = green leaf base, model_1 = blossoms (each connected blossom gets its own colour, like in the pot)
{
	const obj = new OBJLoader().parse(readFileSync('models-src/flower-only/flower-only.obj', 'utf8'));
	const heads = ['#c91a09', '#f2cd37', '#0055bf', '#fe8a18', '#ff8ac0', '#f4f4f4'];
	const parts = [];
	obj.traverse((o) => {
		if (!o.isMesh) return;
		let g = o.geometry.clone().toNonIndexed();
		for (const k of Object.keys(g.attributes)) if (k !== 'position') g.deleteAttribute(k);
		g.computeVertexNormals();
		const P = g.attributes.position;
		const col = new Uint8Array(P.count * 4);
		if (o.name === 'model_1') {
			// blossoms: connected pieces via shared (rounded) positions -> one colour per blossom
			const key = (i) => `${P.getX(i).toFixed(4)},${P.getY(i).toFixed(4)},${P.getZ(i).toFixed(4)}`;
			const par = new Map();
			const find = (k) => {
				while (par.get(k) !== k) {
					par.set(k, par.get(par.get(k)));
					k = par.get(k);
				}
				return k;
			};
			for (let i = 0; i < P.count; i++) if (!par.has(key(i))) par.set(key(i), key(i));
			for (let t = 0; t < P.count; t += 3) for (let k = 1; k < 3; k++) par.set(find(key(t)), find(key(t + k)));
			// a blossom is 3 stacked pieces: group the pieces whose centres are close in x/z
			const cent = new Map();
			for (let t = 0; t < P.count; t += 3) {
				const r = find(key(t));
				const c = cent.get(r) ?? { x: 0, z: 0, n: 0 };
				for (let k = 0; k < 3; k++) (c.x += P.getX(t + k)), (c.z += P.getZ(t + k)), c.n++;
				cent.set(r, c);
			}
			const blossoms = [];
			const blossomOf = new Map();
			for (const [r, c] of cent) {
				const x = c.x / c.n, z = c.z / c.n;
				let b = blossoms.findIndex((q) => Math.hypot(q.x - x, q.z - z) < 0.6);
				if (b < 0) b = blossoms.push({ x, z }) - 1;
				blossomOf.set(r, b);
			}
			for (let t = 0; t < P.count; t += 3) {
				const c = new THREE.Color(heads[blossomOf.get(find(key(t))) % heads.length]);
				for (let k = 0; k < 3; k++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], (t + k) * 4);
			}
			console.log('flower-only blossoms:', blossoms.length);
		} else {
			const c = new THREE.Color('#237841');
			for (let i = 0; i < P.count; i++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], i * 4);
		}
		g.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
		parts.push(g);
	});
	const f = mergeGeometries(parts);
	f.computeBoundingBox();
	const fb = f.boundingBox;
	f.translate(-(fb.min.x + fb.max.x) / 2, -fb.min.y, -(fb.min.z + fb.max.z) / 2); // centred, standing on the ground
	f.computeBoundingBox();
	console.log('flower-only size', f.boundingBox.getSize(new THREE.Vector3()).toArray().map((v) => v.toFixed(2)).join(' x '));
	write(f, 'static/models/flowerpot-flowers.bin', { color: true });
}

// ---------------- small LEGO flower plant (flowersss.obj: a 1x1 flower + a 2x2 leaf base) ----------------
// Source is Z-up in millimetres (8 mm = 1 stud); the two parts lie far apart, so split by position and stack them.
{
	const src = load('models-src/flowers2/flowersss.obj').toNonIndexed();
	const P = src.attributes.position;
	const head = [], leaf = [];
	for (let t = 0; t < P.count; t += 3) {
		const tri = [];
		for (let k = 0; k < 3; k++) tri.push(P.getX(t + k), P.getY(t + k), P.getZ(t + k));
		(tri[0] < 0 ? head : leaf).push(...tri);
	}
	const mk = (arr) => {
		let g = new THREE.BufferGeometry();
		g.setAttribute('position', new THREE.Float32BufferAttribute(arr, 3));
		g.rotateX(-Math.PI / 2); // Z-up -> Y-up
		g.scale(0.125, 0.125, 0.125); // mm -> studs
		g.computeBoundingBox();
		const b = g.boundingBox;
		g.translate(-(b.min.x + b.max.x) / 2, -b.min.y, -(b.min.z + b.max.z) / 2);
		g.computeVertexNormals();
		return g;
	};
	const leafG = mk(leaf);
	const headG = mk(head);
	leafG.computeBoundingBox();
	// the flower sits on top of the leaf base
	headG.translate(0, leafG.boundingBox.max.y - 0.05, 0);
	headG.computeBoundingBox();
	console.log('flower plant leaf', leafG.boundingBox.getSize(new THREE.Vector3()).toArray().map((v) => v.toFixed(2)).join(' x '), 'top', headG.boundingBox.max.y.toFixed(2));
	write(leafG, 'static/models/flowerplant-leaf.bin', {});
	write(headG, 'static/models/flowerplant-head.bin', {});
}

// ---------------- shared: split scene OBJs made of one long triangle strip into plants ----------------
// The "Lego Frog" / "Lego Trees and Plants" exports are one strip per file (Y-up, LDraw units: 20 = 1 stud):
// drop the strip's degenerate bridge triangles, find connected parts, then group parts whose boxes touch.
// files: [[path, materialSlot], ...] -> plants sorted by triangle count (largest first)
function stripPlants(files) {
	const strip = (file, src) => {
		const V = [], F = [];
		for (const l of readFileSync(file, 'utf8').split('\n')) {
			if (l.startsWith('v ')) V.push(l.split(/\s+/).slice(1, 4).map(Number));
			else if (l.startsWith('f ')) F.push(l.trim().split(/\s+/).slice(1).map((s) => parseInt(s) - 1));
		}
		const area = (f) => {
			const [A, B, Cc] = f.map((i) => V[i]);
			const u = [B[0] - A[0], B[1] - A[1], B[2] - A[2]], v = [Cc[0] - A[0], Cc[1] - A[1], Cc[2] - A[2]];
			return Math.hypot(u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]);
		};
		const tris = F.filter((f) => new Set(f).size === 3 && area(f) > 1e-6);
		const canon = new Map();
		const rep = V.map((p, i) => {
			const k = p.map((x) => x.toFixed(3)).join(',');
			if (!canon.has(k)) canon.set(k, i);
			return canon.get(k);
		});
		const par = V.map((_, i) => i);
		const find = (x) => {
			while (par[x] !== x) x = par[x] = par[par[x]];
			return x;
		};
		for (const f of tris) for (let i = 1; i < 3; i++) par[find(rep[f[0]])] = find(rep[f[i]]);
		const comps = new Map();
		for (const f of tris) {
			const r = find(rep[f[0]]);
			if (!comps.has(r)) comps.set(r, { tris: [], src, mn: [1e9, 1e9, 1e9], mx: [-1e9, -1e9, -1e9] });
			const c = comps.get(r);
			c.tris.push(f.map((i) => V[i]));
			for (const i of f) for (let k = 0; k < 3; k++) (c.mn[k] = Math.min(c.mn[k], V[i][k])), (c.mx[k] = Math.max(c.mx[k], V[i][k]));
		}
		return [...comps.values()];
	};
	const comps = files.flatMap(([f, src]) => strip(f, src));
	const cp = comps.map((_, i) => i);
	const cf = (x) => {
		while (cp[x] !== x) x = cp[x] = cp[cp[x]];
		return x;
	};
	for (let i = 0; i < comps.length; i++)
		for (let j = i + 1; j < comps.length; j++) {
			const a = comps[i], b = comps[j];
			let touch = true;
			for (let k = 0; k < 3; k++) if (a.mn[k] - 0.6 > b.mx[k] || b.mn[k] - 0.6 > a.mx[k]) touch = false;
			if (touch) cp[cf(i)] = cf(j);
		}
	const groups = new Map();
	comps.forEach((c, i) => {
		const r = cf(i);
		if (!groups.has(r)) groups.set(r, []);
		groups.get(r).push(c);
	});
	return [...groups.values()]
		.map((cs) => {
			const mn = [1e9, 1e9, 1e9], mx = [-1e9, -1e9, -1e9];
			for (const c of cs) for (let k = 0; k < 3; k++) (mn[k] = Math.min(mn[k], c.mn[k])), (mx[k] = Math.max(mx[k], c.mx[k]));
			return { cs, n: cs.reduce((s, c) => s + c.tris.length, 0), mn, mx };
		})
		.sort((a, b) => b.n - a.n || a.mn[0] - b.mn[0]);
}
/** one plant -> geometry in studs, centred on x/z, standing on y = 0 (only parts of material slot `src`, or all) */
function plantGeometry(plant, src = null) {
	const S = 0.05; // LDraw units -> studs
	const cx = (plant.mn[0] + plant.mx[0]) / 2, cz = (plant.mn[2] + plant.mx[2]) / 2;
	const arr = [];
	for (const c of plant.cs) if (src === null || c.src === src) for (const t of c.tris) for (const p of t) arr.push((p[0] - cx) * S, (p[1] - plant.mn[1]) * S, (p[2] - cz) * S);
	const g = new THREE.BufferGeometry();
	g.setAttribute('position', new THREE.Float32BufferAttribute(arr.length ? arr : [0, 0, 0, 0, 0, 0, 0, 0, 0], 3));
	g.computeVertexNormals();
	return g;
}

// ---------------- LEGO plants from the "Lego Frog" scene (models-src/frog-plants) ----------------
// stems.obj = green stems & leaves, flowers.obj = 1x1 flower plates. Each plant -> <name>-green.bin + <name>-flower.bin
// (flowers get colours per instance).
{
	const plants = stripPlants([
		['models-src/frog-plants/stems.obj', 0],
		['models-src/frog-plants/flowers.obj', 1]
	]);
	// picked by looking at a contact sheet of all plants (sorted by triangle count)
	const PICK = { 'frog-leafy-a': 6, 'frog-leafy-b': 7, 'frog-sprout': 8, 'frog-bouquet': 1, 'frog-stem-a': 2, 'frog-stem-b': 4 };
	for (const [name, idx] of Object.entries(PICK)) {
		for (const src of [0, 1]) {
			const g = plantGeometry(plants[idx], src);
			const tris = g.attributes.position.count / 3;
			write(tris > 60 ? simplify(g, 0.4, 0.01, `${name}-${src ? 'flower' : 'green'}`) : g, `static/models/${name}-${src ? 'flower' : 'green'}.bin`, {});
		}
		const p = plants[idx];
		console.log(name, 'size', [0, 1, 2].map((k) => ((p.mx[k] - p.mn[k]) * 0.05).toFixed(2)).join(' x '));
	}
}

// ---------------- "Lego Trees and Plants": only what the village did not have yet ----------------
// tall grass tuft (model_2), big fern (model_6), the slim cypress + the small pine (model_5; the middle pine = our spruce)
{
	const one = (file) => stripPlants([[file, 0]]);
	const grass = one('models-src/trees-plants/model_2.obj')[0];
	const fern = one('models-src/trees-plants/model_6.obj')[0];
	const pines = one('models-src/trees-plants/model_5.obj').sort((a, b) => b.mx[1] - b.mn[1] - (a.mx[1] - a.mn[1])); // tallest first
	const out = { 'tall-grass': grass, fern, cypress: pines[0], 'pine-small': pines[pines.length - 1] };
	for (const [name, p] of Object.entries(out)) {
		// the pines' thin layered branches tear apart when cut too hard: keep more of them
		const ratio = name === 'fern' ? 0.2 : name === 'cypress' || name === 'pine-small' ? 0.55 : 0.35;
		let g = simplify(plantGeometry(p), ratio, name === 'cypress' || name === 'pine-small' ? 0.004 : 0.01, name);
		if (name === 'cypress' || name === 'pine-small') {
			// brown base plate + trunk, green crown (same look as the spruce)
			g.computeBoundingBox();
			const H = g.boundingBox.max.y;
			const pos = g.attributes.position;
			const col = new Uint8Array(pos.count * 4);
			const trunk = new THREE.Color('#82422a'), leaf = new THREE.Color(name === 'cypress' ? '#2f8a3c' : '#237841');
			const crownFrom = name === 'cypress' ? 0.16 : 0.14;
			for (let i = 0; i < pos.count; i++) {
				const c = pos.getY(i) / H < crownFrom ? trunk : leaf;
				col.set([c.r * 255, c.g * 255, c.b * 255, 255], i * 4);
			}
			g.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
			write(g, `static/models/${name}.bin`, { color: true });
		} else write(g, `static/models/${name}.bin`, {});
		console.log(name, 'size', [0, 1, 2].map((k) => ((p.mx[k] - p.mn[k]) * 0.05).toFixed(2)).join(' x '));
	}
}

// ---------------- LEGO computer from the "Tiny LEGO Office Room" (model_27: computer + other pieces) ----------------
// Same strip format (Y-up, LDraw units). The computer is the biggest group of touching parts; front = +z (keyboard).
// Coloured here because the export has no materials: light grey housing, dark key tops, and the slanted
// screen as its own model so the game can make it glow.
{
	const pc = stripPlants([['models-src/office/model_27.obj', 0]])[0];
	const S = 0.05;
	const cx = (pc.mn[0] + pc.mx[0]) / 2, cz = (pc.mn[2] + pc.mx[2]) / 2;
	const housing = [], keys = [], screen = [];
	for (const c of pc.cs)
		for (const t of c.tris) {
			const [A, B, Cc] = t;
			const u = [B[0] - A[0], B[1] - A[1], B[2] - A[2]], v = [Cc[0] - A[0], Cc[1] - A[1], Cc[2] - A[2]];
			const n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
			const l = Math.hypot(...n) || 1;
			const ny = n[1] / l, nz = n[2] / l;
			const cyy = (A[1] + B[1] + Cc[1]) / 3, czz = (A[2] + B[2] + Cc[2]) / 3;
			const pts = t.flatMap((p) => [(p[0] - cx) * S, (p[1] - pc.mn[1]) * S, (p[2] - cz) * S]);
			// The slanted front is open between thin side strips (the real part has a printed screen). The recessed
			// screen surface behind it faces inwards (back/down), so from the front it was invisible = "transparent".
			// Use that surface as the screen, flipped to face out; the strips stay part of the grey frame.
			const cxx = (A[0] + B[0] + Cc[0]) / 3;
			if (ny < -0.55 && nz < -0.55 && cyy > 11 && cyy < 33 && Math.abs(cxx) < 17.5) continue; // replaced by the panel below
			// key tops: small upward faces on the keyboard in front of the housing
			else if (ny > 0.9 && czz > 0.5 && cyy > 7.5 && cyy < 9) keys.push(...pts);
			else housing.push(...pts);
		}
	// The screen: one opaque panel over the whole opening (x ±16.9, slanted plane y + z = 12 from
	// (y 12.8, z -0.8) to (y 31.2, z -19.3)).
	{
		// flush with the frame (a hair in front, so no inner gap shows when looking at it from the side) and wide
		// enough to overlap the inner half of the side strips (x 16.9..19.9)
		const X = 18.4, out = 0.2; // out = offset along the face normal (0, .71, .71)
		const bot = [12.3 + out, -0.3 + out], top = [31.6 + out, -19.6 + out]; // [y, z], both on y + z = 12.4
		const P = (x, yz) => [(x - cx) * S, (yz[0] - pc.mn[1]) * S, (yz[1] - cz) * S];
		const a = P(-X, bot), b = P(X, bot), c = P(X, top), d = P(-X, top);
		screen.push(...a, ...b, ...c, ...a, ...c, ...d); // faces +y/+z (towards the viewer)
	}
	// the source models every key in detail (30k triangles): simplify each colour on its own, then colour + merge
	const part = (arr, ratio, hex, label) => {
		const g0 = new THREE.BufferGeometry();
		g0.setAttribute('position', new THREE.Float32BufferAttribute(arr, 3));
		const g1 = simplify(g0, ratio, 0.004, label);
		const c = new THREE.Color(hex);
		const col = new Uint8Array(g1.attributes.position.count * 4);
		for (let i = 0; i < g1.attributes.position.count; i++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], i * 4);
		g1.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
		return g1;
	};
	const g = mergeGeometries([part(housing, 0.12, '#a0a5a9', 'computer housing'), part(keys, 0.3, '#3a3f45', 'computer keys')]);
	// the housing as edited in Blender (models-src/office/Computer.obj, already in game units) replaces the
	// automatic version; the OBJ has no colours, so housing / key tops are coloured again here
	const edited = 'models-src/office/Computer.obj';
	if (existsSync(edited)) {
		// drop the loose edges ('l' lines): with them the loader would turn the whole object into lines
		const obj = new OBJLoader().parse(readFileSync(edited, 'utf8').replace(/^l .*$/gm, ''));
		const mesh = obj.children.find((o) => o.isMesh && o.name === 'lab-computer'); // (the file also has loose edges as a line object)
		let h = mesh.geometry.clone().toNonIndexed();
		for (const k of Object.keys(h.attributes)) if (k !== 'position') h.deleteAttribute(k);
		h.computeVertexNormals();
		const P = h.attributes.position;
		const col = new Uint8Array(P.count * 4);
		const grey = new THREE.Color('#a0a5a9'), keyC = new THREE.Color('#3a3f45');
		const A = new THREE.Vector3(), B = new THREE.Vector3(), C = new THREE.Vector3(), n = new THREE.Vector3();
		for (let t = 0; t < P.count; t += 3) {
			A.fromBufferAttribute(P, t);
			B.fromBufferAttribute(P, t + 1);
			C.fromBufferAttribute(P, t + 2);
			n.subVectors(C, B).cross(new THREE.Vector3().subVectors(A, B)).normalize();
			const cy = (A.y + B.y + C.y) / 3, cz = (A.z + B.z + C.z) / 3;
			// key tops: upward faces at keyboard height in front of the housing
			const c = Math.abs(n.y) > 0.9 && cy > 0.37 && cy < 0.46 && cz > 0.52 ? keyC : grey;
			for (let k = 0; k < 3; k++) col.set([c.r * 255, c.g * 255, c.b * 255, 255], (t + k) * 4);
		}
		h.setAttribute('color', new THREE.BufferAttribute(col, 4, true));
		console.log('lab computer: using the Blender edit', edited, P.count / 3, 'tris');
		write(h, 'static/models/lab-computer.bin', { color: true });
	} else write(g, 'static/models/lab-computer.bin', { color: true });
	const sg = new THREE.BufferGeometry();
	sg.setAttribute('position', new THREE.Float32BufferAttribute(screen, 3));
	sg.computeVertexNormals();
	write(sg, 'static/models/lab-computer-screen.bin', {});
	console.log('lab computer size', [0, 1, 2].map((k) => ((pc.mx[k] - pc.mn[k]) * S).toFixed(2)).join(' x '), 'screen tris', screen.length / 9);
}

// ---------------- asset version (cache busting for the game's model loader) ----------------
writeFileSync('src/lib/three/assets-version.ts', `// generated by scripts/convert-models.mjs — bump forces browsers to load fresh models\nexport const ASSET_VERSION = '${Date.now().toString(36)}';\n`);
console.log('asset version written');
