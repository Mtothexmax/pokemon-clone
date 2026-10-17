/**
 * flowerfield.js — procedural flower-field tile generator (pure, no DOM).
 *
 * Ported from the reference "Blumenfeld-Generator": a noise-shaded ground
 * with dirt patches, scattered grass tufts, and noise-clustered, Y-sorted
 * pixel-stamp flowers. The `ctx` is passed in so headless tests can drive it
 * with a stub. `drawFlowerField` clears and fills the whole target, so it is
 * a full-cover tile (like the wheat field).
 */

// Reference palettes (index-selectable), plus the game's own tuned palette.
export const FIELD_PALETTES = [
	{
		id: 'wildflower-meadow',
		name: 'Wildblumen Wiese',
		groundDark: '#1a331e',
		groundBase: '#2a5431',
		groundLight: '#3d7a47',
		dirtDark: '#2b1b12',
		dirtBase: '#4d3324',
		outline: '#0f1a11',
		blooms: ['#e11d48', '#f59e0b', '#3b82f6', '#ec4899', '#ffffff', '#8b5cf6'],
		centers: ['#facc15', '#ffffff', '#f59e0b']
	},
	{
		id: 'spring-breeze',
		name: 'Frühlingsbrise',
		groundDark: '#1b3d2b',
		groundBase: '#2d6a4f',
		groundLight: '#52b788',
		dirtDark: '#261c14',
		dirtBase: '#4a3728',
		outline: '#0b1a12',
		blooms: ['#f472b6', '#fb7185', '#38bdf8', '#a78bfa', '#fef08a'],
		centers: ['#fef08a', '#ffffff']
	},
	{
		id: 'golden-hour',
		name: 'Goldene Stunde',
		groundDark: '#2e3314',
		groundBase: '#4e571d',
		groundLight: '#788628',
		dirtDark: '#362013',
		dirtBase: '#5c3a21',
		outline: '#141708',
		blooms: ['#f97316', '#eab308', '#ef4444', '#facc15', '#f43f5e'],
		centers: ['#78350f', '#451a03']
	},
	{
		id: 'fantasy-enchanted',
		name: 'Zauberwald (Fantasy)',
		groundDark: '#131e3a',
		groundBase: '#1d2d5b',
		groundLight: '#2e4380',
		dirtDark: '#1e112a',
		dirtBase: '#3a2054',
		outline: '#080d1a',
		blooms: ['#22d3ee', '#c084fc', '#f43f5e', '#a855f7', '#38bdf8'],
		centers: ['#ffffff', '#fef08a']
	},
	{
		id: 'night-meadow',
		name: 'Mondschein Nacht',
		groundDark: '#0a141d',
		groundBase: '#132433',
		groundLight: '#203a52',
		dirtDark: '#10141a',
		dirtBase: '#202833',
		outline: '#04080c',
		blooms: ['#818cf8', '#6366f1', '#38bdf8', '#c084fc', '#e0e7ff'],
		centers: ['#67e8f9', '#ffffff']
	},
	{
		id: 'autumn-fields',
		name: 'Herbstfeld',
		groundDark: '#292014',
		groundBase: '#473620',
		groundLight: '#6e542e',
		dirtDark: '#24140b',
		dirtBase: '#3d2211',
		outline: '#120d08',
		blooms: ['#dc2626', '#d97706', '#b45309', '#7c2d12', '#f59e0b'],
		centers: ['#fef08a', '#78350f']
	}
];

/** Game palette: greens that blend with the map grass, game flower hues. */
export const GAME_FIELD_PAL = {
	id: 'pallet-meadow',
	name: 'Pallet Meadow',
	groundDark: '#5fae44',
	groundBase: '#7ec850',
	groundLight: '#a4e878',
	dirtDark: '#6b4a2a',
	dirtBase: '#8a6238',
	outline: '#1e2b1a',
	blooms: ['#e74c3c', '#f1c40f', '#fafafa', '#ec7fa9', '#8b5cf6', '#3b82f6'],
	centers: ['#f1c40f', '#ffffff', '#f59e0b']
};

/** Park–Miller PRNG (same as the reference generator). */
export function createPRNG(seed) {
	let s = seed % 2147483647;
	if (s <= 0) s += 2147483646;
	return function () {
		s = (s * 16807) % 2147483647;
		return (s - 1) / 2147483646;
	};
}

export function proceduralNoise2D(x, y, seed) {
	const n = Math.sin(x * 12.9898 + y * 78.233 + seed * 43758.5453) * 43758.5453;
	return n - Math.floor(n);
}

export function smoothNoise2D(x, y, scale, seed) {
	const nx = x * scale;
	const ny = y * scale;
	const x0 = Math.floor(nx);
	const y0 = Math.floor(ny);
	const x1 = x0 + 1;
	const y1 = y0 + 1;
	const sx = nx - x0;
	const sy = ny - y0;
	const n0 = proceduralNoise2D(x0, y0, seed);
	const n1 = proceduralNoise2D(x1, y0, seed);
	const n2 = proceduralNoise2D(x0, y1, seed);
	const n3 = proceduralNoise2D(x1, y1, seed);
	const ix0 = n0 + sx * (n1 - n0);
	const ix1 = n2 + sx * (n3 - n2);
	return ix0 + sy * (ix1 - ix0);
}

/** Math-rasterised filled circle with an optional Y-offset outline stamp. */
export function drawPixelStampCircle(ctx, cx, cy, radius, fillColor, outlineColor = null, offsetY = 0) {
	cx = Math.round(cx);
	cy = Math.round(cy);
	radius = Math.round(radius);
	if (outlineColor) {
		ctx.fillStyle = outlineColor;
		const oY = cy + Math.round(offsetY);
		const rOut = radius + 1;
		for (let y = -rOut; y <= rOut; y++) {
			for (let x = -rOut; x <= rOut; x++) {
				if (x * x + y * y <= rOut * rOut) ctx.fillRect(cx + x, oY + y, 1, 1);
			}
		}
	}
	ctx.fillStyle = fillColor;
	for (let y = -radius; y <= radius; y++) {
		for (let x = -radius; x <= radius; x++) {
			if (x * x + y * y <= radius * radius) ctx.fillRect(cx + x, cy + y, 1, 1);
		}
	}
}

/**
 * Draw a full flower-field into ctx at (0,0), size width x height.
 * params: { seed, palette | paletteIndex, density, noiseScale, grassDensity,
 *           strokeOffset, bloomRadius, flowerType, showDirt, showStems }
 */
export function drawFlowerField(ctx, width, height, params) {
	ctx.imageSmoothingEnabled = false;
	if (!params.skipClear) ctx.clearRect(0, 0, width, height);

	const prng = createPRNG(params.seed);
	const palette = params.palette || FIELD_PALETTES[params.paletteIndex] || FIELD_PALETTES[0];
	const noiseScale = params.noiseScale ?? 0.08;
	const density = params.density ?? 45;
	const grassDensity = params.grassDensity ?? 60;
	const strokeOffset = params.strokeOffset ?? 1;
	const bloomRadius = params.bloomRadius ?? 2;
	const flowerType = params.flowerType ?? 'mixed';
	const transparent = params.transparent ?? false;
	// Wind sway: displace each bloom horizontally from its base by a sine that
	// varies with world position, so a field ripples instead of moving rigidly.
	const swayAmp = params.sway ?? 0;
	const swayPhase = params.swayPhase ?? 0;

	// 1. Ground + raster texture (noise shading, optional dirt patches).
	//    Skipped in transparent mode so an underlying grass tile shows through.
	if (!transparent) {
		for (let y = 0; y < height; y++) {
			for (let x = 0; x < width; x++) {
				const noiseVal = smoothNoise2D(x, y, noiseScale, params.seed);
				const fineVal = proceduralNoise2D(x, y, params.seed);
				let pixelColor = palette.groundBase;
				if (noiseVal < 0.35) pixelColor = palette.groundDark;
				else if (noiseVal > 0.65) pixelColor = palette.groundLight;
				if (params.showDirt && noiseVal < 0.22 && fineVal > 0.3) {
					pixelColor = fineVal > 0.6 ? palette.dirtDark : palette.dirtBase;
				}
				ctx.fillStyle = pixelColor;
				ctx.fillRect(x, y, 1, 1);
			}
		}
	}

	// 2. Grass tufts on the denser green.
	const grassCount = Math.round(((width * height) / 16) * (grassDensity / 50));
	for (let i = 0; i < grassCount; i++) {
		const gx = Math.floor(prng() * width);
		const gy = Math.floor(prng() * height);
		const gVal = smoothNoise2D(gx, gy, noiseScale, params.seed);
		if (gVal > 0.2) {
			const grassColor = gVal > 0.55 ? palette.groundLight : palette.groundDark;
			const h = Math.floor(prng() * 2) + 1;
			ctx.fillStyle = grassColor;
			ctx.fillRect(gx, gy, 1, h);
			if (prng() > 0.5 && gx + 1 < width) ctx.fillRect(gx + 1, gy - 1, 1, 1);
		}
	}

	// 3. Flower placement via noise clustering, Y-sorted for depth.
	const targetFlowerCount = Math.round(((width * height) / 32) * (density / 40));
	const flowerList = [];
	for (let i = 0; i < targetFlowerCount * 3; i++) {
		if (flowerList.length >= targetFlowerCount) break;
		const fx = Math.floor(prng() * width);
		const fy = Math.floor(prng() * height);
		const nVal = smoothNoise2D(fx, fy, noiseScale, params.seed + 100);
		if (nVal > 0.38 || prng() < 0.15) flowerList.push({ x: fx, y: fy, r: prng() });
	}
	flowerList.sort((a, b) => a.y - b.y);

	// 4. Draw flowers.
	for (const flower of flowerList) {
		const fx = flower.x;
		const fy = flower.y;
		const sw = swayAmp ? Math.round(Math.sin(fx * 0.11 + fy * 0.05 + swayPhase) * swayAmp) : 0;
		const bx = fx + sw;
		let bloomColor, centerColor;
		if (flowerType === 'tulips') {
			bloomColor = ['#f43f5e', '#e11d48', '#fb7185', '#be123c'][Math.floor(flower.r * 4)];
			centerColor = '#fef08a';
		} else if (flowerType === 'daisies') {
			bloomColor = '#ffffff';
			centerColor = '#f59e0b';
		} else if (flowerType === 'poppies') {
			bloomColor = flower.r > 0.2 ? '#dc2626' : '#ef4444';
			centerColor = '#18181b';
		} else if (flowerType === 'lavender') {
			bloomColor = ['#8b5cf6', '#7c3aed', '#a855f7', '#c084fc'][Math.floor(flower.r * 4)];
			centerColor = '#e9d5ff';
		} else if (flowerType === 'sunflowers') {
			bloomColor = '#eab308';
			centerColor = '#451a03';
		} else {
			bloomColor = palette.blooms[Math.floor(flower.r * palette.blooms.length)];
			centerColor = palette.centers[Math.floor(flower.r * palette.centers.length) % palette.centers.length];
		}

		if (params.showStems) {
			ctx.fillStyle = palette.groundDark;
			ctx.fillRect(fx, fy + 1, 1, 2);
			if (flower.r > 0.5) ctx.fillRect(fx - 1, fy + 2, 1, 1);
		}

		if (flowerType === 'lavender') {
			drawPixelStampCircle(ctx, bx, fy, 1, bloomColor, palette.outline, strokeOffset);
			drawPixelStampCircle(ctx, bx, fy - 2, 1, bloomColor, palette.outline, strokeOffset);
			drawPixelStampCircle(ctx, bx, fy - 4, 1, centerColor, null, 0);
		} else if (flowerType === 'sunflowers') {
			drawPixelStampCircle(ctx, bx, fy, Math.max(2, bloomRadius + 1), bloomColor, palette.outline, strokeOffset);
			drawPixelStampCircle(ctx, bx, fy, 1, centerColor, null, 0);
		} else {
			drawPixelStampCircle(ctx, bx, fy, bloomRadius, bloomColor, palette.outline, strokeOffset);
			if (bloomRadius >= 2 || flower.r > 0.3) {
				ctx.fillStyle = centerColor;
				ctx.fillRect(bx, fy, 1, 1);
			}
		}
	}
}
