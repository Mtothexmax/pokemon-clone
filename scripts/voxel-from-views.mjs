// Turns a picture of a beast seen from the FRONT (left half of the image) and from the SIDE (right half,
// head on the left) into a Voxelmon model file: src/lib/voxelmon/models/<id>.js with `front` and `side`
// pixel rows. src/lib/voxelmon/draw.js then carves the 3D body from both views.
// The image needs a transparent background, both views next to each other, feet at the same height.
//
//   node scripts/voxel-from-views.mjs titanoak path/to/views.png [--height=64] [--colors=28]
import { writeFileSync, copyFileSync, mkdirSync } from 'node:fs';
import { decodePng } from './png.mjs';
import { CARD_MAP } from '../src/lib/game/cards.ts';

const [id, file] = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const opt = (k, d) => +(process.argv.find((a) => a.startsWith(`--${k}=`))?.split('=')[1] ?? d);
if (!id || !file) {
	console.error('usage: node scripts/voxel-from-views.mjs <card id> <image.png> [--height=64] [--colors=28]');
	process.exit(1);
}
const H = opt('height', 64);
const K = opt('colors', 28);
const img = decodePng(file);
const { w, h, rgba } = img;
const opaque = (x, y) => rgba[(y * w + x) * 4 + 3] > 120;

// ---- the two views: the two widest runs of columns that contain something ----
const runs = [];
let start = -1;
for (let x = 0; x <= w; x++) {
	let any = false;
	if (x < w) for (let y = 0; y < h && !any; y++) any = opaque(x, y);
	if (any && start < 0) start = x;
	if (!any && start >= 0) {
		runs.push([start, x - 1]);
		start = -1;
	}
}
const views = runs.sort((a, b) => b[1] - b[0] - (a[1] - a[0])).slice(0, 2).sort((a, b) => a[0] - b[0]);
if (views.length < 2) throw new Error('need two views next to each other (front left, side right)');
const box = views.map(([x0, x1]) => {
	let y0 = h, y1 = -1;
	for (let y = 0; y < h; y++) for (let x = x0; x <= x1; x++) if (opaque(x, y)) [y0, y1] = [Math.min(y0, y), Math.max(y1, y)];
	return { x0, x1, y0, y1 };
});
// one scale for both views (they are drawn at the same size): the taller view gets H voxels
const f = Math.max(...box.map((b) => b.y1 - b.y0 + 1)) / H;

// ---- palette: k-means over the opaque pixels of both views ----
const samples = [];
for (const b of box) for (let y = b.y0; y <= b.y1; y += 2) for (let x = b.x0; x <= b.x1; x += 2) if (opaque(x, y)) samples.push([0, 1, 2].map((k) => rgba[(y * w + x) * 4 + k]));
const lum = (c) => 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
samples.sort((a, b) => lum(a) - lum(b));
let centers = Array.from({ length: K }, (_, i) => [...samples[Math.floor(((i + 0.5) / K) * samples.length)]]);
const nearest = (c) => {
	let best = 0, bd = Infinity;
	centers.forEach((m, i) => {
		const d = (c[0] - m[0]) ** 2 + (c[1] - m[1]) ** 2 + (c[2] - m[2]) ** 2;
		if (d < bd) [bd, best] = [d, i];
	});
	return best;
};
for (let it = 0; it < 12; it++) {
	const sum = centers.map(() => [0, 0, 0, 0]);
	for (const c of samples) {
		const s = sum[nearest(c)];
		s[0] += c[0];
		s[1] += c[1];
		s[2] += c[2];
		s[3]++;
	}
	centers = centers.map((m, i) => (sum[i][3] ? sum[i].slice(0, 3).map((v) => v / sum[i][3]) : m));
}
const KEYS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
const used = new Set();

// ---- each view as H rows of characters, feet on the bottom row ----
function grid(b) {
	const cols = Math.round((b.x1 - b.x0 + 1) / f);
	const rows = [];
	for (let r = 0; r < H; r++) {
		// bottom-aligned: row H-1 ends at the feet
		const yb = b.y1 + 1 - (H - 1 - r) * f;
		const ya = yb - f;
		let line = '';
		for (let c = 0; c < cols; c++) {
			const xa = b.x0 + c * f, xb = xa + f;
			const count = new Map();
			let n = 0, tot = 0;
			for (let y = Math.max(0, Math.floor(ya)); y < Math.min(h, Math.ceil(yb)); y++)
				for (let x = Math.floor(xa); x < Math.min(w, Math.ceil(xb)); x++) {
					tot++;
					if (!opaque(x, y)) continue;
					n++;
					const k = nearest([0, 1, 2].map((q) => rgba[(y * w + x) * 4 + q]));
					count.set(k, (count.get(k) ?? 0) + 1);
				}
			if (!tot || n / tot < 0.45) {
				line += '.';
				continue;
			}
			const k = [...count].sort((p, q) => q[1] - p[1])[0][0];
			used.add(k);
			line += KEYS[k];
		}
		rows.push(line.replace(/\.+$/, ''));
	}
	return rows;
}
const [front, side] = box.map(grid);

const hex = (c) => '#' + c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('');
const name = CARD_MAP[id]?.name ?? id;
const rowsJs = (rows) => rows.map((r) => `\t\t'${r}'`).join(',\n');
const src = `// ${name} as a Voxelmon, made from a front and a side picture (scripts/voxel-from-views.mjs).
// The 3D body is carved from both views (see ../draw.js): a voxel exists where BOTH views are solid.
// front: seen from the front, columns = left to right, rows = top to bottom (feet on the last row)
// side:  seen from the side, HEAD ON THE LEFT, columns = nose to tail, same rows as front
// Every character is one voxel, '.' = empty. What you see from the front gets the front colours, everything else the side colours.
export default {
	name: '${name.replace(/'/g, "\\'")}',
	palette: {
${[...used].sort((a, b) => a - b).map((k) => `\t\t${KEYS[k]}: '${hex(centers[k])}'`).join(',\n')}
	},
	front: [
${rowsJs(front)}
	],
	side: [
${rowsJs(side)}
	]
};
`;
writeFileSync(`src/lib/voxelmon/models/${id}.js`, src);
// keep the source picture next to the models, so the model can be made again
mkdirSync('src/lib/voxelmon/refs', { recursive: true });
copyFileSync(file, `src/lib/voxelmon/refs/${id}.png`);
console.log(`${id}: front ${Math.max(...front.map((r) => r.length))}x${H}, side ${Math.max(...side.map((r) => r.length))}x${H}, ${used.size} colours`);
