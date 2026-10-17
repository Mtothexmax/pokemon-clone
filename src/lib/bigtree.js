/**
 * bigtree.js — large "cloud tree" generator.
 *
 * Ported from the reference "Procedural Pixel Art Cloud Tree Generator"
 * (classic RPG offset-outline circle stacking): a cone of foliage tiers
 * built from pixel circles, a trunk with root flare, and a ground shadow.
 * The shadow is drawn semi-transparent so the map ground shows through.
 *
 * Pure: the caller passes a 2d context, so headless tests can drive it
 * with a stub context (no DOM here).
 */

/** Six palettes from the reference tool (game uses classic-pine/emerald). */
export const BIG_PALETTES = [
	{
		id: 'classic-pine', name: 'Classic RPG Pine',
		outline: '#101a10', trunkDark: '#2b1911', trunk: '#50311f',
		shadowGreen: '#18381e', baseGreen: '#2e6332',
		highlightGreen: '#4d9d4f', topGreen: '#85d972'
	},
	{
		id: 'emerald-rpg', name: 'Emerald Kingdom',
		outline: '#08140e', trunkDark: '#23150d', trunk: '#422819',
		shadowGreen: '#0f482d', baseGreen: '#1a7a4c',
		highlightGreen: '#2bc07b', topGreen: '#62f0a8'
	},
	{
		id: 'autumn-maple', name: 'Autumn Gold',
		outline: '#1a0c06', trunkDark: '#2e190e', trunk: '#54301a',
		shadowGreen: '#5e1b0d', baseGreen: '#a33b15',
		highlightGreen: '#db6b1d', topGreen: '#f5b138'
	},
	{
		id: 'cherry-blossom', name: 'Sakura Blossom',
		outline: '#1a0d18', trunkDark: '#301c22', trunk: '#52313a',
		shadowGreen: '#732247', baseGreen: '#b84277',
		highlightGreen: '#e673a5', topGreen: '#fcaed3'
	},
	{
		id: 'cyber-neon', name: 'Cyberpunk Neon',
		outline: '#08081a', trunkDark: '#1c1038', trunk: '#371e6e',
		shadowGreen: '#1e0f52', baseGreen: '#4d1994',
		highlightGreen: '#9d21d9', topGreen: '#00f0ff'
	},
	{
		id: 'frost-fir', name: 'Frost Fir',
		outline: '#0b141a', trunkDark: '#1b2631', trunk: '#34495e',
		shadowGreen: '#1c3d5a', baseGreen: '#2b6cb0',
		highlightGreen: '#4299e1', topGreen: '#90cdf4'
	}
];

/** Seedable PRNG (Lehmer / Park-Miller) for reproducible trees. */
export function createPRNG(seed) {
	let s = seed % 2147483647;
	if (s <= 0) s += 2147483646;
	return function () {
		s = (s * 16807) % 2147483647;
		return (s - 1) / 2147483646;
	};
}

/**
 * Crisp integer pixel circle. The optional outline circle is drawn first,
 * offset downward by outlineOffsetY, so the fill leaves a dark rim below —
 * the signature "cloud" look.
 */
export function drawPixelCircle(ctx, cx, cy, radius, fillColor, outlineColor = null, outlineOffsetY = 0) {
	cx = Math.round(cx);
	cy = Math.round(cy);
	radius = Math.round(radius);
	if (outlineColor) {
		ctx.fillStyle = outlineColor;
		const oY = cy + Math.round(outlineOffsetY);
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
 * Render one tree onto a low-res pixel context (e.g. 64x96 for a 2x3 tile
 * tree). params = {seed, paletteIndex, tiers, density, minRad, maxRad,
 * strokeOffset, jitter, trunkWidth, trunkHeight, rootFlare, showShadow,
 * showTopLight, shadowColor}.
 */
export function drawBigTree(ctx, width, height, params = {}) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, width, height);

	const prng = createPRNG(params.seed ?? 1337);
	const palette = BIG_PALETTES[params.paletteIndex] ?? BIG_PALETTES[0];
	const shadowColor = params.shadowColor ?? 'rgba(0,0,20,0.25)';
	const centerX = Math.floor(width / 2);
	const groundY = height - 6;

	// --- 1. Ground shadow (transparent so grass shows through) ---
	if (params.showShadow ?? true) {
		ctx.fillStyle = shadowColor;
		const shadowW = Math.round((params.rootFlare ?? 12) * 1.3);
		const shadowH = 3;
		for (let y = -shadowH; y <= shadowH; y++) {
			for (let x = -shadowW; x <= shadowW; x++) {
				if ((x * x) / (shadowW * shadowW) + (y * y) / (shadowH * shadowH) <= 1) {
					ctx.fillRect(centerX + x, groundY + 1 + y, 1, 1);
				}
			}
		}
	}

	// --- 2. Trunk + root flare (dark outline, then bark + left shade) ---
	const trunkH = Math.round(params.trunkHeight ?? 22);
	const trunkW = Math.round(params.trunkWidth ?? 6);
	const rootW = Math.round(params.rootFlare ?? 12);
	const trunkStartY = groundY - trunkH;

	ctx.fillStyle = palette.outline;
	for (let y = trunkStartY - 1; y <= groundY + 1; y++) {
		const progress = (y - trunkStartY) / trunkH;
		const currentW = Math.round(trunkW + (rootW - trunkW) * Math.pow(Math.max(0, progress), 2)) + 2;
		ctx.fillRect(centerX - Math.floor(currentW / 2), y, currentW, 1);
	}
	for (let y = trunkStartY; y <= groundY; y++) {
		const progress = (y - trunkStartY) / trunkH;
		const currentW = Math.round(trunkW + (rootW - trunkW) * Math.pow(Math.max(0, progress), 2));
		const leftX = centerX - Math.floor(currentW / 2);
		ctx.fillStyle = palette.trunk;
		ctx.fillRect(leftX, y, currentW, 1);
		ctx.fillStyle = palette.trunkDark;
		ctx.fillRect(leftX, y, Math.max(1, Math.floor(currentW * 0.35)), 1);
	}

	// --- 3. Foliage cloud stacking (cone tiers, bottom dark → top bright) ---
	const tiers = params.tiers ?? 7;
	const density = params.density ?? 10;
	const minRad = params.minRad ?? 3;
	const maxRad = params.maxRad ?? 7;
	const strokeOffset = params.strokeOffset ?? 2;
	const jitter = params.jitter ?? 0.5;

	const canopyBottomY = groundY - Math.round(trunkH * 0.7);
	const canopyTopY = 8;
	const canopyHeight = canopyBottomY - canopyTopY;

	for (let t = 0; t < tiers; t++) {
		const tierProgress = t / (tiers - 1); // 0 bottom → 1 peak
		const tierY = Math.round(canopyBottomY - tierProgress * canopyHeight);
		// 0.74 (vs the tool's 0.82) so the widest tier + blob radius stays
		// inside the sprite instead of clipping on the tile edges.
		const maxTierWidth = width * 0.74 * (1 - Math.pow(tierProgress, 1.2) * 0.7);

		let fillColor, shadeColor;
		if (tierProgress < 0.25) {
			fillColor = palette.shadowGreen;
			shadeColor = palette.outline;
		} else if (tierProgress < 0.6) {
			fillColor = palette.baseGreen;
			shadeColor = palette.shadowGreen;
		} else if (tierProgress < 0.88) {
			fillColor = palette.highlightGreen;
			shadeColor = palette.baseGreen;
		} else {
			fillColor = palette.topGreen;
			shadeColor = palette.highlightGreen;
		}

		const blobCount = Math.max(3, Math.round(density * (1 - tierProgress * 0.4)));
		for (let b = 0; b < blobCount; b++) {
			const normPos = b / (blobCount - 1 || 1) - 0.5;
			const jitterX = (prng() - 0.5) * jitter * 6;
			const jitterY = (prng() - 0.5) * jitter * 4;
			const blobX = centerX + normPos * maxTierWidth + jitterX;
			const blobY = tierY + jitterY;
			const blobRadius = minRad + prng() * (maxRad - minRad);
			drawPixelCircle(ctx, blobX, blobY, blobRadius, fillColor, shadeColor, strokeOffset);
		}
	}

	// --- 4. Top light cap ---
	if (params.showTopLight ?? true) {
		const topX = centerX + (prng() - 0.5) * 2;
		const topY = canopyTopY + 2;
		drawPixelCircle(ctx, topX, topY, Math.max(2, minRad - 1), palette.topGreen, palette.highlightGreen, 1);
	}
}
