// buildVoxels of the three.js version's src/lib/voxelmon/draw.js (without the three.js parts)
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
