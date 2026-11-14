// Runs every Voxelmon model's build function (tools/beasts_src, copied from the three.js version) and writes
// tools/beasts.json: per beast its palette and voxels [x, y, z, paletteIndex], the bounds' min/max.
import { readdirSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { buildVoxels } from './beasts_src/build_voxels.js';
const dir = new URL('./beasts_src/models/', import.meta.url);
const out = {};
let total = 0;
for (const f of readdirSync(dir).filter((f) => f.endsWith('.js')).sort()) {
	const id = f.replace('.js', '');
	const model = (await import(new URL(f, dir))).default;
	const { vox, min, max } = buildVoxels(model);
	const keys = Object.keys(model.palette);
	const voxels = [];
	for (const [k, c] of vox) {
		const [x, y, z] = k.split(',').map(Number);
		voxels.push([x, y, z, keys.indexOf(c)]);
	}
	out[id] = { palette: keys.map((k) => model.palette[k]), voxels, min, max };
	total += voxels.length;
}
writeFileSync(new URL('./beasts.json', import.meta.url), JSON.stringify(out));
console.log(Object.keys(out).length, 'beasts,', total, 'voxels');
