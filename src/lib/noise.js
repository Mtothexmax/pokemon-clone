/**
 * noise.js — tiny deterministic value noise + fbm for organic shapes.
 *
 * Used for cloud blobs, tree-canopy dapple, bush shading and any
 * blobby natural variation. Integer-hash based (no tables, no DOM),
 * so it runs identically in Node tests and the browser.
 */

/** Integer hash -> 0..1 (deterministic, no state). */
export function hash2(x, y, seed = 0) {
	let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(seed, 974634)) | 0;
	h = Math.imul(h ^ (h >>> 13), 1274126177);
	return ((h ^ (h >>> 16)) >>> 0) / 4294967295;
}

function smooth(t) {
	return t * t * (3 - 2 * t);
}

/** Smooth value noise in [0,1]. */
export function vnoise(x, y, seed = 0) {
	const xi = Math.floor(x);
	const yi = Math.floor(y);
	const xf = x - xi;
	const yf = y - yi;
	const a = hash2(xi, yi, seed);
	const b = hash2(xi + 1, yi, seed);
	const c = hash2(xi, yi + 1, seed);
	const d = hash2(xi + 1, yi + 1, seed);
	const u = smooth(xf);
	const v = smooth(yf);
	return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}

/** Fractal Brownian motion in [0,1]: sum of octaves, normalized. */
export function fbm(x, y, octaves = 3, seed = 0) {
	let sum = 0;
	let amp = 0.5;
	let freq = 1;
	let norm = 0;
	for (let i = 0; i < octaves; i++) {
		sum += amp * vnoise(x * freq, y * freq, seed + i * 101);
		norm += amp;
		amp *= 0.5;
		freq *= 2.03;
	}
	return sum / norm;
}

/** Circle mask test: (x,y) inside radius r around (cx,cy)? */
export function inDisc(x, y, cx, cy, r) {
	const dx = x - cx;
	const dy = y - cy;
	return dx * dx + dy * dy <= r * r;
}
