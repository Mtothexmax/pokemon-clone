/**
 * smalltree.js — procedural small tree / berry-bush generator (pure, no DOM).
 *
 * Ported from the reference "Kleiner Baum & Beeren" generator: a slender
 * trunk (straight/curved/birch/multistem) under egg-shaped foliage built from
 * cloud-noise circles (shadow -> dark -> mid -> light), plus optional fruit
 * scattered over the canopy. `ctx` is passed in so headless tests can drive it.
 */
import { createPRNG, drawPixelStampCircle } from './flowerfield.js';

export const SMALLTREE_PALETTES = [
	{
		id: 'apple-classic',
		name: 'Klassisches Apfelbäumchen',
		foliageLight: '#82c91e',
		foliageMid: '#40c057',
		foliageDark: '#2b8a3e',
		foliageShadow: '#1b4332',
		trunkLight: '#a67c52',
		trunkMid: '#795238',
		trunkDark: '#432818',
		outline: '#102015',
		defaultFruit: '#e03131',
		fruitHighlight: '#ff8787'
	},
	{
		id: 'berry-bush',
		name: 'Blaubeer & Moos-Strauch',
		foliageLight: '#a9e18a',
		foliageMid: '#51cf66',
		foliageDark: '#2f9e44',
		foliageShadow: '#18522b',
		trunkLight: '#8d6e63',
		trunkMid: '#5d4037',
		trunkDark: '#3e2723',
		outline: '#0d2818',
		defaultFruit: '#3b82f6',
		fruitHighlight: '#93c5fd'
	},
	{
		id: 'birch-lemon',
		name: 'Birke mit Zitronen',
		foliageLight: '#d8f5a2',
		foliageMid: '#94d82d',
		foliageDark: '#5c940d',
		foliageShadow: '#2b4c02',
		trunkLight: '#f1f5f9',
		trunkMid: '#cbd5e1',
		trunkDark: '#334155',
		outline: '#0f172a',
		defaultFruit: '#fcc419',
		fruitHighlight: '#ffec99'
	},
	{
		id: 'cherry-blossom',
		name: 'Kirschblüte / Rote Beeren',
		foliageLight: '#ffc9c9',
		foliageMid: '#fcc2d7',
		foliageDark: '#f06595',
		foliageShadow: '#a61e4d',
		trunkLight: '#8b5a2b',
		trunkMid: '#5c3a1e',
		trunkDark: '#361e0c',
		outline: '#2b0914',
		defaultFruit: '#c92a2a',
		fruitHighlight: '#ff8787'
	},
	{
		id: 'autumn-orange',
		name: 'Herbstlicher Apfelsinenbaum',
		foliageLight: '#ffd43b',
		foliageMid: '#f76707',
		foliageDark: '#d9480f',
		foliageShadow: '#7a1705',
		trunkLight: '#9c6644',
		trunkMid: '#7f4f24',
		trunkDark: '#582f0e',
		outline: '#2e0a03',
		defaultFruit: '#ff922b',
		fruitHighlight: '#ffe8cc'
	},
	{
		id: 'magic-night',
		name: 'Zauberwald / Leuchtbeeren',
		foliageLight: '#67e8f9',
		foliageMid: '#06b6d4',
		foliageDark: '#0e7490',
		foliageShadow: '#164e63',
		trunkLight: '#a855f7',
		trunkMid: '#7e22ce',
		trunkDark: '#3b0764',
		outline: '#081a24',
		defaultFruit: '#f43f5e',
		fruitHighlight: '#fecdd3'
	}
];

/**
 * params: { seed, palette | paletteIndex, trunkStyle, trunkHeight, foliageWidth,
 *           cloudRadius, strokeOffset, showBranches, showGroundShadow,
 *           showFruits, fruitCount, fruitType, fruitScatter }
 */
export function drawSmallTree(ctx, width, height, params) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, width, height);

	const prng = createPRNG(params.seed);
	const palette = params.palette || SMALLTREE_PALETTES[params.paletteIndex] || SMALLTREE_PALETTES[0];

	const centerX = Math.floor(width / 2);
	const groundY = height - 4;

	// 1. Ground shadow.
	if (params.showGroundShadow) {
		ctx.fillStyle = 'rgba(10, 20, 15, 0.4)';
		const shadowW = Math.round(params.foliageWidth * 0.9);
		for (let sx = -shadowW; sx <= shadowW; sx++) {
			ctx.fillRect(centerX + sx, groundY, 1, 1);
			if (Math.abs(sx) <= shadowW - 1) ctx.fillRect(centerX + sx, groundY + 1, 1, 1);
		}
	}

	// 2. Trunk & branches.
	const trunkHeight = params.trunkHeight;
	const trunkStartY = groundY;
	const trunkTopY = groundY - trunkHeight;

	if (params.trunkStyle === 'multistem') {
		ctx.fillStyle = palette.trunkDark;
		ctx.fillRect(centerX - 2, trunkStartY, 1, 2);
		ctx.fillRect(centerX + 2, trunkStartY, 1, 2);
		ctx.fillStyle = palette.trunkMid;
		for (let y = trunkStartY; y >= trunkTopY; y--) {
			ctx.fillRect(centerX - Math.floor((trunkStartY - y) * 0.3), y, 1, 1);
			ctx.fillRect(centerX + Math.floor((trunkStartY - y) * 0.35), y, 1, 1);
			ctx.fillRect(centerX, y, 1, 1);
		}
	} else if (params.trunkStyle === 'birch') {
		for (let y = trunkStartY; y >= trunkTopY; y--) {
			ctx.fillStyle = palette.trunkMid;
			ctx.fillRect(centerX - 1, y, 2, 1);
			if ((y + Math.floor(prng() * 3)) % 3 === 0) {
				ctx.fillStyle = palette.trunkDark;
				ctx.fillRect(centerX - (prng() > 0.5 ? 1 : 0), y, 1, 1);
			} else {
				ctx.fillStyle = palette.trunkLight;
				ctx.fillRect(centerX, y, 1, 1);
			}
		}
	} else if (params.trunkStyle === 'curved') {
		let currentX = centerX;
		for (let y = trunkStartY; y >= trunkTopY; y--) {
			if (prng() < 0.25 && y < trunkStartY - 2) currentX += prng() > 0.5 ? 1 : -1;
			ctx.fillStyle = palette.trunkDark;
			ctx.fillRect(currentX - 1, y, 2, 1);
			ctx.fillStyle = palette.trunkMid;
			ctx.fillRect(currentX, y, 1, 1);
		}
	} else {
		for (let y = trunkStartY; y >= trunkTopY; y--) {
			ctx.fillStyle = palette.trunkDark;
			ctx.fillRect(centerX - 1, y, 2, 1);
			ctx.fillStyle = palette.trunkMid;
			ctx.fillRect(centerX, y, 1, 1);
			if (y === trunkStartY) {
				ctx.fillRect(centerX - 2, y, 1, 1);
				ctx.fillRect(centerX + 1, y, 1, 1);
			}
		}
	}

	if (params.showBranches) {
		ctx.fillStyle = palette.trunkMid;
		const branchY1 = trunkTopY + Math.floor(trunkHeight * 0.3);
		const branchY2 = trunkTopY + Math.floor(trunkHeight * 0.6);
		ctx.fillRect(centerX - 1, branchY1, 1, 1);
		ctx.fillRect(centerX - 2, branchY1 - 1, 1, 1);
		ctx.fillRect(centerX - 3, branchY1 - 2, 1, 1);
		ctx.fillRect(centerX + 1, branchY2, 1, 1);
		ctx.fillRect(centerX + 2, branchY2 - 1, 1, 1);
		ctx.fillRect(centerX + 3, branchY2 - 2, 1, 1);
	}

	// 3. Foliage clouds (shadow -> dark/mid -> light).
	const foliageWidth = params.foliageWidth;
	const cloudRadius = params.cloudRadius;
	const cloudCircles = [];
	const layers = 5;
	for (let l = 0; l < layers; l++) {
		const layerRatio = l / (layers - 1);
		const layerY = trunkTopY + 2 - l * ((foliageWidth * 1.1) / layers);
		const widthAtLayer = foliageWidth * Math.sin(Math.PI * (layerRatio * 0.75 + 0.2));
		const countAtLayer = Math.max(2, Math.round(widthAtLayer * 0.7));
		for (let c = 0; c < countAtLayer; c++) {
			const offsetFactor = (c - (countAtLayer - 1) / 2) / Math.max(1, countAtLayer - 1);
			const cx = centerX + offsetFactor * widthAtLayer * 0.8 + (prng() - 0.5) * 1.5;
			const cy = layerY + (prng() - 0.5) * 1.5;
			const r = cloudRadius + (prng() - 0.5) * 0.8;
			cloudCircles.push({ x: cx, y: cy, r: Math.max(1.5, r), layerRatio });
		}
	}
	cloudCircles.sort((a, b) => b.y - a.y);

	for (const circle of cloudCircles) {
		drawPixelStampCircle(ctx, circle.x, circle.y, circle.r, palette.foliageShadow, palette.outline, params.strokeOffset);
	}
	for (const circle of cloudCircles) {
		const col = circle.layerRatio < 0.35 ? palette.foliageDark : palette.foliageMid;
		drawPixelStampCircle(ctx, circle.x, circle.y, circle.r - 0.6, col, null, 0);
	}
	for (const circle of cloudCircles) {
		if (circle.layerRatio > 0.45 || prng() < 0.3) {
			drawPixelStampCircle(ctx, circle.x - 0.5, circle.y - 0.8, Math.max(1, circle.r - 1.4), palette.foliageLight, null, 0);
		}
	}

	// 4. Fruits & berries, mask-culled to the canopy.
	if (params.showFruits && params.fruitCount > 0) {
		const fruitColor = palette.defaultFruit;
		const highlightColor = palette.fruitHighlight;
		const candidates = [];
		const minY = Math.min(...cloudCircles.map((c) => c.y - c.r));
		const maxY = Math.max(...cloudCircles.map((c) => c.y + c.r));
		const minX = Math.min(...cloudCircles.map((c) => c.x - c.r));
		const maxX = Math.max(...cloudCircles.map((c) => c.x + c.r));
		for (let py = Math.floor(minY + 1); py <= Math.ceil(maxY - 1); py++) {
			for (let px = Math.floor(minX + 1); px <= Math.ceil(maxX - 1); px++) {
				let inside = false;
				for (const circle of cloudCircles) {
					const dx = px - circle.x;
					const dy = py - circle.y;
					if (dx * dx + dy * dy <= (circle.r - 0.5) * (circle.r - 0.5)) {
						inside = true;
						break;
					}
				}
				if (inside) candidates.push({ x: px, y: py, normY: (py - minY) / (maxY - minY || 1) });
			}
		}
		if (candidates.length > 0) {
			const selected = [];
			const requested = Math.min(params.fruitCount, candidates.length);
			if (params.fruitScatter === 'edge') {
				const edge = candidates.filter((c) => c.normY < 0.3 || c.normY > 0.7 || prng() < 0.4);
				const shuffled = [...(edge.length ? edge : candidates)].sort(() => prng() - 0.5);
				for (let i = 0; i < requested && i < shuffled.length; i++) selected.push(shuffled[i]);
			} else if (params.fruitScatter === 'dense') {
				const shuffled = [...candidates].sort(() => prng() - 0.5);
				for (let i = 0; i < requested && i < shuffled.length; i++) selected.push(shuffled[i]);
			} else {
				const sectorHeight = (maxY - minY) / requested;
				for (let i = 0; i < requested; i++) {
					const lo = minY + i * sectorHeight;
					let pool = candidates.filter((c) => c.y >= lo && c.y < lo + sectorHeight);
					if (!pool.length) pool = candidates;
					let best = null;
					let bestMin = -1;
					for (const cand of [...pool].sort(() => prng() - 0.5)) {
						if (!selected.length) {
							best = cand;
							break;
						}
						let minDist = Infinity;
						for (const f of selected) {
							const d = (cand.x - f.x) ** 2 + (cand.y - f.y) ** 2;
							if (d < minDist) minDist = d;
						}
						if (minDist > bestMin) {
							bestMin = minDist;
							best = cand;
						}
					}
					if (best) selected.push(best);
				}
			}
			for (const fruit of selected) {
				const fx = Math.round(fruit.x);
				const fy = Math.round(fruit.y);
				if (params.fruitType === 'apples') {
					drawPixelStampCircle(ctx, fx, fy, 1.5, fruitColor, palette.outline, 1);
					ctx.fillStyle = highlightColor;
					ctx.fillRect(fx - 1, fy - 1, 1, 1);
				} else if (params.fruitType === 'miniapples') {
					ctx.fillStyle = palette.outline;
					ctx.fillRect(fx, fy + 1, 2, 1);
					ctx.fillStyle = fruitColor;
					ctx.fillRect(fx, fy, 2, 1);
					ctx.fillStyle = palette.foliageLight;
					ctx.fillRect(fx, fy - 1, 1, 1);
					ctx.fillStyle = highlightColor;
					ctx.fillRect(fx + 1, fy, 1, 1);
				} else if (params.fruitType === 'cherries') {
					ctx.fillStyle = palette.trunkDark;
					ctx.fillRect(fx, fy - 1, 1, 1);
					ctx.fillStyle = fruitColor;
					ctx.fillRect(fx - 1, fy, 1, 1);
					ctx.fillRect(fx + 1, fy + 1, 1, 1);
					ctx.fillStyle = highlightColor;
					ctx.fillRect(fx - 1, fy, 1, 1);
				} else if (params.fruitType === 'lemons') {
					ctx.fillStyle = palette.outline;
					ctx.fillRect(fx - 1, fy, 3, 2);
					ctx.fillStyle = fruitColor;
					ctx.fillRect(fx - 1, fy, 3, 1);
					ctx.fillRect(fx, fy + 1, 1, 1);
					ctx.fillStyle = highlightColor;
					ctx.fillRect(fx - 1, fy, 1, 1);
				} else {
					ctx.fillStyle = palette.outline;
					ctx.fillRect(fx, fy + 1, 1, 1);
					ctx.fillStyle = fruitColor;
					ctx.fillRect(fx, fy, 1, 1);
					if (prng() > 0.5) ctx.fillRect(fx + 1, fy, 1, 1);
				}
			}
		}
	}
}
