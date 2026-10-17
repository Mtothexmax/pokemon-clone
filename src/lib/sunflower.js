/**
 * sunflower.js — procedural sunflower sprite (pure, no DOM).
 *
 * Ported from the reference "Sonnenblume" generator: a 32x64 sprite with a big
 * filled blossom head (layered petals + seed core) over a thick stalk and two
 * leaves. `drawSunflower` paints one at (x,y); `drawSunflowerPatch` arranges a
 * few across a region for the map.
 */

const SUN_C = {
	petalLight: '#ffeb3b',
	petalMid: '#fbc02d',
	petalDark: '#f57f17',
	petalShadow: '#d84315',
	petalOutline: '#5d1000',
	seedLight: '#a1887f',
	seedMid: '#5d4037',
	seedDark: '#3e2723',
	seedOutline: '#1b0000',
	stemLight: '#7cb342',
	stemDark: '#33691e',
	leafLight: '#8bc34a',
	leafMid: '#558b2f',
	leafDark: '#2e7d32',
	outline: '#1b3e09'
};

/** Small deterministic PRNG so a patch varies but stays stable per prop. */
function prng(seed) {
	let s = (seed | 0) % 2147483647;
	if (s <= 0) s += 2147483646;
	return () => {
		s = (s * 16807) % 2147483647;
		return (s - 1) / 2147483646;
	};
}

/**
 * Paint one 32x64 sunflower with its blossom top-left at (x,y).
 * opts: { sway } small horizontal lean for variety.
 */
export function drawSunflower(ctx, x = 0, y = 0, opts = {}) {
	const sway = opts.sway ?? 0;
	const p = (px, py, color) => {
		ctx.fillStyle = color;
		ctx.fillRect(Math.round(x + px), Math.round(y + py), 1, 1);
	};
	const rect = (rx, ry, rw, rh, color) => {
		ctx.fillStyle = color;
		ctx.fillRect(Math.round(x + rx), Math.round(y + ry), rw, rh);
	};

	// --- stalk ---
	rect(14 + sway, 24, 4, 38, SUN_C.stemDark);
	rect(14 + sway, 24, 2, 38, SUN_C.stemLight);

	// --- big leaves (lower half) ---
	rect(4, 40, 10, 5, SUN_C.leafMid);
	rect(6, 38, 7, 2, SUN_C.leafLight);
	rect(3, 42, 10, 4, SUN_C.leafDark);
	rect(2, 41, 2, 4, SUN_C.outline);
	rect(4, 46, 9, 1, SUN_C.outline);

	rect(18, 35, 10, 5, SUN_C.leafMid);
	rect(19, 33, 7, 2, SUN_C.leafLight);
	rect(19, 37, 10, 4, SUN_C.leafDark);
	rect(28, 36, 2, 4, SUN_C.outline);
	rect(19, 41, 10, 1, SUN_C.outline);

	// --- big blossom head (top 32x32 tile) ---
	const cx = 15.5 + sway;
	const cy = 15.5;
	for (let px = 0; px < 32; px++) {
		for (let py = 0; py < 32; py++) {
			const dx = px - cx;
			const dy = py - cy;
			const dist = Math.sqrt(dx * dx + dy * dy);
			const angle = Math.atan2(dy, dx);
			const petalShape = Math.pow(Math.abs(Math.sin(6 * angle)), 0.4);
			const maxRadius = 10.0 + petalShape * 5.2;
			if (dist <= maxRadius && dist >= 4.5) {
				if (dist > maxRadius - 1.2) p(px, py, SUN_C.petalOutline);
				else if (dist > maxRadius - 2.8) p(px, py, dy > 2 ? SUN_C.petalShadow : SUN_C.petalDark);
				else if (dist > maxRadius - 5.5) p(px, py, SUN_C.petalMid);
				else p(px, py, SUN_C.petalLight);
			}
		}
	}

	// --- seed core ---
	for (let px = 0; px < 32; px++) {
		for (let py = 0; py < 32; py++) {
			const dx = px - cx;
			const dy = py - cy;
			const dist = Math.sqrt(dx * dx + dy * dy);
			if (dist <= 6.2) {
				if (dist > 5.2) p(px, py, SUN_C.seedOutline);
				else if (dist > 4.2) p(px, py, SUN_C.seedDark);
				else if (dx < -1 && dy < -1) p(px, py, SUN_C.seedLight);
				else p(px, py, SUN_C.seedMid);
			}
		}
	}
	p(14 + sway, 13, SUN_C.seedLight);
	p(16 + sway, 14, SUN_C.seedLight);
	p(13 + sway, 16, SUN_C.seedDark);
	p(17 + sway, 17, SUN_C.seedDark);
	p(15 + sway, 18, SUN_C.seedDark);
}

/**
 * Paint a few sunflowers across a w*32 x h*32 region (h should be 2).
 * params: { count, seed, showGroundShadow, scale }
 */
export function drawSunflowerPatch(ctx, w, h, params = {}) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, w * 32, h * 32);
	const count = params.count ?? 3;
	const seed = params.seed ?? 1;
	const s = params.scale ?? 0.75;
	const rnd = prng(seed);
	const bottom = h * 32;
	const rand = [];
	for (let i = 0; i < count; i++) rand.push({ s: rnd(), tw: rnd(), sh: rnd() });
	for (let i = 0; i < count; i++) {
		const cx = Math.round(((i + 0.5) / count) * w * 32 - 16 + (rand[i].s - 0.5) * 10);
		const x = cx;
		const y = bottom - 64 + Math.round((rand[i].tw - 0.5) * 6);
		if (params.showGroundShadow) {
			const sw = Math.round(20 * s);
			ctx.fillStyle = 'rgba(0,0,0,0.18)';
			ctx.fillRect(x + 16 - sw / 2, bottom - 3, sw, 3);
		}
		ctx.save();
		ctx.translate(x + 16 * (1 - s), y + 64 * (1 - s));
		ctx.scale(s, s);
		drawSunflower(ctx, 0, 0, { sway: rand[i].sh > 0.5 ? 1 : -1 });
		ctx.restore();
	}
}
