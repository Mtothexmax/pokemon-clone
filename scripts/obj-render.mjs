// Draws OBJ files into a PNG (software rasteriser, no browser needed): to look at models before using them.
// Every file gets its own colour (or the colours from a JSON map), lit from the top left.
//
//   node scripts/obj-render.mjs <dir or file.obj> <out.png> [--view=top|iso|front|side] [--size=1200]
//        [--only=model_1,model_2] [--box=x0,z0,x1,z1] (only that part of the scene, seen from above)
import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs';
import { deflateSync } from 'node:zlib';

const [src, out] = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const arg = (k, d) => process.argv.find((a) => a.startsWith(`--${k}=`))?.split('=')[1] ?? d;
const VIEW = arg('view', 'iso');
const SIZE = +arg('size', 1200);
const only = arg('only', '') ? new Set(arg('only', '').split(',').map((s) => s.replace(/\.obj$/, ''))) : null;
const box = arg('box', '') ? arg('box', '').split(',').map(Number) : null;
const labels = process.argv.includes('--labels');

const files = statSync(src).isDirectory() ? readdirSync(src).filter((f) => f.endsWith('.obj')).map((f) => `${src}/${f}`) : [src];
// a stable colour per file name
const hue = (s) => {
	let h = 0;
	for (const c of s) h = (h * 31 + c.charCodeAt(0)) >>> 0;
	return h;
};
const hsl = (h, s, l) => {
	const f = (n) => {
		const k = (n + h / 30) % 12;
		return l - s * Math.min(l, 1 - l) * Math.max(-1, Math.min(k - 3, 9 - k, 1));
	};
	return [f(0), f(8), f(4)];
};

// view rotation: maps world (x, y, z) to screen (u right, v up, depth towards the viewer)
const views = {
	top: (x, y, z) => [x, -z, y],
	front: (x, y, z) => [x, y, z],
	side: (x, y, z) => [-z, y, x],
	iso: (x, y, z) => {
		const a = Math.PI / 4, b = (35 * Math.PI) / 180;
		const u = x * Math.cos(a) - z * Math.sin(a);
		const zz = x * Math.sin(a) + z * Math.cos(a);
		return [u, y * Math.cos(b) - zz * Math.sin(b), y * Math.sin(b) + zz * Math.cos(b)];
	}
};
const proj = views[VIEW];
const tris = []; // [u0,v0,d0, u1,v1,d1, u2,v2,d2, r,g,b]
const centers = [];
for (const file of files) {
	const name = file.split('/').pop().replace('.obj', '');
	if (only && !only.has(name)) continue;
	const pos = [];
	const vcol = []; // vertex colours (OBJ lines "v x y z r g b"), if the file has them
	const col = hsl(hue(name) % 360, 0.55, 0.6);
	let cx = 0, cy = 0, cn = 0;
	for (const l of readFileSync(file, 'utf8').split('\n')) {
		if (l.startsWith('v ')) {
			const p = l.trim().split(/\s+/).slice(1).map(Number);
			pos.push(p.slice(0, 3));
			if (p.length >= 6) vcol[pos.length - 1] = p.slice(3, 6);
		}
		else if (l.startsWith('f ')) {
			const ids = l.trim().split(/\s+/).slice(1).map((t) => parseInt(t, 10) - 1);
			for (let k = 1; k + 1 < ids.length; k++) {
				const P = [pos[ids[0]], pos[ids[k]], pos[ids[k + 1]]];
				if (box && P.every((p) => p[0] < box[0] || p[0] > box[2] || p[2] < box[1] || p[2] > box[3])) continue;
				// lighting from the world normal
				const e1 = P[1].map((v, i) => v - P[0][i]), e2 = P[2].map((v, i) => v - P[0][i]);
				const n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
				const len = Math.hypot(...n) || 1;
				const L = [-0.4, 0.8, 0.45];
				const lit = 0.45 + 0.55 * Math.abs((n[0] * L[0] + n[1] * L[1] + n[2] * L[2]) / len);
				const q = P.map((p) => proj(...p));
				const c = vcol[ids[0]] ? [0, 1, 2].map((i) => (vcol[ids[0]][i] + vcol[ids[k]][i] + vcol[ids[k + 1]][i]) / 3) : col;
				tris.push([...q[0], ...q[1], ...q[2], c[0] * lit, c[1] * lit, c[2] * lit]);
				cx += q[0][0];
				cy += q[0][1];
				cn++;
			}
		}
	}
	if (cn) centers.push([name, cx / cn, cy / cn]);
}
// fit to the image
let u0 = Infinity, u1 = -Infinity, v0 = Infinity, v1 = -Infinity;
for (const t of tris) for (let k = 0; k < 9; k += 3) [u0, u1, v0, v1] = [Math.min(u0, t[k]), Math.max(u1, t[k]), Math.min(v0, t[k + 1]), Math.max(v1, t[k + 1])];
const s = (SIZE * 0.96) / Math.max(u1 - u0, v1 - v0);
const W = Math.ceil((u1 - u0) * s + SIZE * 0.04), H = Math.ceil((v1 - v0) * s + SIZE * 0.04);
const img = new Uint8Array(W * H * 3).fill(235);
const zb = new Float32Array(W * H).fill(-Infinity);
const X = (u) => (u - u0) * s + SIZE * 0.02, Y = (v) => (v1 - v) * s + SIZE * 0.02;
for (const t of tris) {
	const ax = X(t[0]), ay = Y(t[1]), bx = X(t[3]), by = Y(t[4]), cx = X(t[6]), cy = Y(t[7]);
	const area = (bx - ax) * (cy - ay) - (cx - ax) * (by - ay);
	if (Math.abs(area) < 1e-9) continue;
	const minX = Math.max(0, Math.floor(Math.min(ax, bx, cx))), maxX = Math.min(W - 1, Math.ceil(Math.max(ax, bx, cx)));
	const minY = Math.max(0, Math.floor(Math.min(ay, by, cy))), maxY = Math.min(H - 1, Math.ceil(Math.max(ay, by, cy)));
	for (let y = minY; y <= maxY; y++)
		for (let x = minX; x <= maxX; x++) {
			const w0 = ((bx - x) * (cy - y) - (cx - x) * (by - y)) / area;
			const w1 = ((cx - x) * (ay - y) - (ax - x) * (cy - y)) / area;
			const w2 = 1 - w0 - w1;
			if (w0 < 0 || w1 < 0 || w2 < 0) continue;
			const d = w0 * t[2] + w1 * t[5] + w2 * t[8];
			const i = y * W + x;
			if (d <= zb[i]) continue;
			zb[i] = d;
			img[i * 3] = Math.min(255, t[9] * 255);
			img[i * 3 + 1] = Math.min(255, t[10] * 255);
			img[i * 3 + 2] = Math.min(255, t[11] * 255);
		}
}
// PNG (RGB, no filter)
const raw = Buffer.alloc((W * 3 + 1) * H);
for (let y = 0; y < H; y++) Buffer.from(img.buffer, y * W * 3, W * 3).copy(raw, y * (W * 3 + 1) + 1);
const crcTable = Array.from({ length: 256 }, (_, n) => {
	let c = n;
	for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
	return c >>> 0;
});
const crc = (b) => {
	let c = 0xffffffff;
	for (const x of b) c = crcTable[(c ^ x) & 255] ^ (c >>> 8);
	return (c ^ 0xffffffff) >>> 0;
};
const chunk = (type, data) => {
	const len = Buffer.alloc(4);
	len.writeUInt32BE(data.length);
	const td = Buffer.concat([Buffer.from(type), data]);
	const c = Buffer.alloc(4);
	c.writeUInt32BE(crc(td));
	return Buffer.concat([len, td, c]);
};
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(W, 0);
ihdr.writeUInt32BE(H, 4);
ihdr[8] = 8;
ihdr[9] = 2;
writeFileSync(out, Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]));
console.log(`${out}: ${W}x${H}, ${tris.length} triangles`);
if (labels) for (const [n, u, v] of centers) console.log(`${n} at ${Math.round(X(u))},${Math.round(Y(v))}`);
