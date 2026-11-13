// Shows a Voxelmon build model as text: front view, side view (head on the left) and top view,
// each as the colour you see first. Handy to check a model without a browser.
//
//   node scripts/voxel-preview.mjs titanoak
import { buildVoxels } from '../src/lib/voxelmon/draw.js';

const id = process.argv[2];
if (!id) {
	console.error('usage: node scripts/voxel-preview.mjs <id>');
	process.exit(1);
}
const model = (await import(`../src/lib/voxelmon/models/${id}.js`)).default;
if (!model.build) {
	console.error(`${id} has no build function`);
	process.exit(1);
}
const { vox, min, max } = buildVoxels(model);
const keys = Object.keys(model.palette);
const CH = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
const ch = (c) => CH[keys.indexOf(c)] ?? '?';
const unknown = [...new Set(vox.values())].filter((c) => !keys.includes(c));
const get = (x, y, z) => vox.get(`${x},${y},${z}`);

/** looks along one axis and returns rows of the first colour seen */
function view(cols, rows, depth, pick) {
	const out = [];
	for (const r of rows) {
		let line = '';
		for (const c of cols) {
			let hit = '.';
			for (const d of depth) {
				const v = pick(c, r, d);
				if (v) {
					hit = ch(v);
					break;
				}
			}
			line += hit;
		}
		out.push(line);
	}
	return out;
}
const range = (a, b, step = 1) => Array.from({ length: Math.floor((b - a) / step) + 1 }, (_, i) => a + i * step);
const ys = range(max[1], min[1], -1);
const front = view(range(min[0], max[0]), ys, range(max[2], min[2], -1), (x, y, z) => get(x, y, z));
const side = view(range(max[2], min[2], -1), ys, range(max[0], min[0], -1), (z, y, x) => get(x, y, z));
const top = view(range(min[0], max[0]), range(min[2], max[2]), range(max[1], min[1], -1), (x, z, y) => get(x, y, z));

console.log(`${model.name ?? id}: ${vox.size} voxels, size ${max[0] - min[0] + 1} wide x ${max[1] - min[1] + 1} tall x ${max[2] - min[2] + 1} long`);
console.log('colours: ' + keys.map((k) => `${ch(k)}=${k}`).join(' '));
if (unknown.length) console.log('UNKNOWN colour keys used: ' + unknown.join(', '));
const pad = Math.max(...front.map((r) => r.length)) + 4;
console.log('\nFRONT'.padEnd(pad + 1) + 'SIDE (head left)');
front.forEach((r, i) => console.log(r.padEnd(pad) + side[i]));
console.log('\nTOP (head at the bottom)');
top.forEach((r) => console.log(r));
