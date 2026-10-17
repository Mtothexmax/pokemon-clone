/**
 * rivergen.js — procedural river/water animator (pure, no DOM).
 *
 * Ported from the reference "Pixel-Art Fluss / Wasser Animator": a vertical
 * (or horizontal) water channel with a deep-centre colour gradient, animated
 * sine waves + foam crests, optional bank foam, shore/sand trim, rocks and
 * drifting leaves. The reference keeps its options on a global `state`; here
 * they arrive through `params` so tests can drive it headlessly.
 *
 * `transparent: true` skips the land/shore fill so an underlying grass tile
 * (painted by the two-pass renderer) shows through the semi-transparent water.
 *
 * params: { palette | paletteIndex, direction(down|right), widthPct, speed,
 *   waveDensity, turbulence, ticks, transparent, waterAlpha,
 *   showBankFoam, showShoreGrass, showRocks, showLeaves, seed }
 */
import { createPRNG } from './flowerfield.js';

/** Reference palettes (GBA/SNES water aesthetics). */
export const RIVER_PALETTES = [
	{
		id: 'gba-classic', name: 'Kanto River (GBA Blue)',
		deep: '#0f3875', mid: '#1d63b8', light: '#389bf2', foam: '#aee3f9',
		shore: '#3d6118', sand: '#c2b280', rock: '#52606d', rockHighlight: '#8a99a8',
		leaf: '#22c55e'
	},
	{
		id: 'crystal-blue', name: 'Crystal Spring',
		deep: '#033a56', mid: '#0284c7', light: '#38bdf8', foam: '#f0f9ff',
		shore: '#15803d', sand: '#e2e8f0', rock: '#475569', rockHighlight: '#94a3b8',
		leaf: '#4ade80'
	},
	{
		id: 'autumn-stream', name: 'Autumn Wood Creek',
		deep: '#1e3a5f', mid: '#2b6cb0', light: '#63b3ed', foam: '#e2e8f0',
		shore: '#854d0e', sand: '#d97706', rock: '#4a5568', rockHighlight: '#a0aec0',
		leaf: '#ef4444'
	},
	{
		id: 'poison-slime', name: 'Toxic Sludge River',
		deep: '#14532d', mid: '#16a34a', light: '#4ade80', foam: '#facc15',
		shore: '#3f6212', sand: '#a3e635', rock: '#334155', rockHighlight: '#64748b',
		leaf: '#eab308'
	},
	{
		id: 'lava-flow', name: 'Lava Flow',
		deep: '#7f1d1d', mid: '#dc2626', light: '#f97316', foam: '#fef08a',
		shore: '#1c1917', sand: '#44403c', rock: '#292524', rockHighlight: '#57534e',
		leaf: '#fdba74'
	}
];

export function drawRiver(ctx, w, h, params = {}) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, w, h);
	const transparent = params.transparent ?? false;
	const palette = params.palette || RIVER_PALETTES[params.paletteIndex ?? 0] || RIVER_PALETTES[0];
	const isVert = (params.direction ?? 'down') === 'down';
	const widthPct = params.widthPct ?? 100;
	const speed = params.speed ?? 1.5;
	const waveDensity = params.waveDensity ?? 3;
	const turbulence = params.turbulence ?? 1;
	const ticks = params.ticks ?? 0;
	const waterAlpha = params.waterAlpha ?? 1;
	const showBankFoam = params.showBankFoam ?? true;
	const showShoreGrass = params.showShoreGrass ?? true;
	const showRocks = params.showRocks ?? false;
	const showLeaves = params.showLeaves ?? false;

	const mainDim = isVert ? w : h;
	const margin = Math.floor(((mainDim * (100 - widthPct)) / 100) / 2);
	const riverStart = margin;
	const riverEnd = mainDim - margin;
	const riverCenter = Math.floor((riverStart + riverEnd) / 2);
	const time = ticks * 0.05 * speed;

	// 1. Land / shore background (skipped when the caller paints grass beneath).
	if (!transparent) {
		if (showShoreGrass) {
			ctx.fillStyle = palette.shore;
			ctx.fillRect(0, 0, w, h);
		}
	}

	// 2. Water surface, pixel by pixel.
	ctx.globalAlpha = waterAlpha;
	for (let y = 0; y < h; y++) {
		for (let x = 0; x < w; x++) {
			const pos = isVert ? x : y;
			const flowPos = isVert ? y : x;
			if (pos >= riverStart && pos < riverEnd) {
				const distToEdge = Math.min(pos - riverStart, riverEnd - 1 - pos);
				const distFromCenter = Math.abs(pos - riverCenter);

				let color = palette.mid;
				if (distFromCenter < (riverEnd - riverStart) * 0.25) color = palette.deep;

				const waveScale = 0.15 * waveDensity;
				const turbFactor = turbulence * 0.4;
				const waveVal =
					Math.sin(flowPos * waveScale - time * 2.0 + Math.cos(pos * 0.2) * turbFactor) +
					Math.cos((flowPos + pos) * 0.1 - time * 1.5);
				if (waveVal > 1.6) color = palette.foam;
				else if (waveVal > 1.1) color = palette.light;

				if (showBankFoam && distToEdge <= 1) {
					const foamNoise = Math.sin(flowPos * 0.5 + time * 3.0);
					if (foamNoise > -0.2) color = palette.foam;
				} else if (showBankFoam && distToEdge <= 2 && Math.sin(flowPos * 0.3 + time * 2) > 0.5) {
					color = palette.light;
				}

				ctx.fillStyle = color;
				ctx.fillRect(x, y, 1, 1);
			} else if (showShoreGrass && !transparent) {
				if (pos === riverStart - 1 || pos === riverEnd) {
					ctx.fillStyle = palette.sand;
					ctx.fillRect(x, y, 1, 1);
				} else if (pos === riverStart - 2 || pos === riverEnd + 1) {
					if ((x + y) % 2 === 0) {
						ctx.fillStyle = palette.sand;
						ctx.fillRect(x, y, 1, 1);
					}
				}
			}
		}
	}
	ctx.globalAlpha = 1;

	// 3. Rocks with upstream foam (deterministic placement).
	if (showRocks) {
		const prng = createPRNG(params.seed ?? 9001);
		const count = 2;
		for (let i = 0; i < count; i++) {
			const relX = 0.2 + prng() * 0.6;
			const relY = prng() * 0.8;
			const size = 2 + Math.floor(prng() * 2);
			const rx = Math.floor(riverStart + relX * (riverEnd - riverStart));
			const ry = Math.floor(relY * h);
			const rockX = isVert ? rx : ry;
			const rockY = isVert ? ry : rx;
			ctx.fillStyle = palette.foam;
			if (isVert) {
				ctx.fillRect(rockX - 1, rockY - 1, size + 2, 1);
				ctx.fillRect(rockX - 2, rockY, 1, size);
			} else {
				ctx.fillRect(rockX - 1, rockY - 1, 1, size + 2);
			}
			ctx.fillStyle = palette.rock;
			ctx.fillRect(rockX, rockY, size, size);
			ctx.fillStyle = palette.rockHighlight;
			ctx.fillRect(rockX, rockY, Math.max(1, Math.floor(size / 2)), 1);
		}
	}

	// 4. Drifting leaves (position advanced by the animation tick).
	if (showLeaves) {
		const prng = createPRNG((params.seed ?? 9001) + 7);
		const count = 4;
		for (let i = 0; i < count; i++) {
			const size = 1 + Math.floor(prng() * 2);
			const speedOffset = 0.5 + prng();
			const relX = prng();
			let relY = prng();
			relY = (relY + ticks * 0.005 * speed * speedOffset) % 1.05 - 0.05;
			const lx = Math.floor(riverStart + 2 + relX * (riverEnd - riverStart - 4));
			const ly = Math.floor(relY * h);
			const leafX = isVert ? lx : ly;
			const leafY = isVert ? ly : lx;
			if (leafX >= riverStart && leafX < riverEnd && leafY >= 0 && leafY < h) {
				ctx.fillStyle = palette.leaf;
				ctx.fillRect(leafX, leafY, size, size);
			}
		}
	}
}

/** Palettes for the seamless river (deep/mid/light/foam/highlight). */
export const SEAMLESS_RIVER_PALETTES = {
	gbaClassic: { deep: '#2058a8', mid: '#3878d0', light: '#58a0e8', foam: '#a8e0f8', highlight: '#ffffff' },
	crystalBlue: { deep: '#0f4c81', mid: '#1b75bc', light: '#29abe2', foam: '#7ecef4', highlight: '#ffffff' },
	poisonSwamp: { deep: '#2d1b4e', mid: '#52277a', light: '#8238b8', foam: '#b266e3', highlight: '#e1b3ff' },
	deepOcean: { deep: '#0a192f', mid: '#112240', light: '#233554', foam: '#64ffda', highlight: '#e6f1ff' }
};

/**
 * Seamless GBA river water (ported from the "Nahtloses Flusswasser" generator).
 * Fully periodic in Y for the given heightPx, so it tiles vertically without a
 * visible seam. Fills the whole width x height area (opaque).
 * params: { ticks, palette | paletteIndex }
 *
 * opts.patternWidth + opts.xOffset let a wider pattern be sliced into tiles so
 * side-by-side river tiles read as ONE continuous stream instead of repeating
 * identical waves: each tile draws the [xOffset, xOffset+widthPx) window of a
 * patternWidth-wide river.
 */
export function drawSeamlessRiver(ctx, widthPx, heightPx, ticks = 0, palette = SEAMLESS_RIVER_PALETTES.gbaClassic, opts = {}) {
	const pw = opts.patternWidth ?? widthPx;
	const xo = opts.xOffset ?? 0;
	if (pw === widthPx && !xo) {
		paintSeamlessRiver(ctx, widthPx, heightPx, ticks, palette);
		return;
	}
	ctx.save();
	ctx.beginPath();
	ctx.rect(0, 0, widthPx, heightPx);
	ctx.clip();
	ctx.translate(-xo, 0);
	paintSeamlessRiver(ctx, pw, heightPx, ticks, palette);
	ctx.restore();
}

function paintSeamlessRiver(ctx, widthPx, heightPx, ticks, palette) {
	ctx.fillStyle = palette.deep;
	ctx.fillRect(0, 0, widthPx, heightPx);

	const progress = ticks % heightPx;
	const streams = [
		{ xRatio: 0.3, width: 14, colMid: palette.mid, colLight: palette.light, colFoam: palette.foam, waveFreq: ((Math.PI * 2) / heightPx) * 2, amp: Math.max(2, heightPx * 0.22), speed: 1.0 },
		{ xRatio: 0.7, width: 12, colMid: palette.mid, colLight: palette.light, colFoam: palette.highlight, waveFreq: ((Math.PI * 2) / heightPx) * 3, amp: Math.max(2, heightPx * 0.28), speed: 1.3 },
		{ xRatio: 0.5, width: 22, colMid: palette.deep, colLight: palette.mid, colFoam: palette.light, waveFreq: (Math.PI * 2) / heightPx, amp: Math.max(3, heightPx * 0.44), speed: 0.7 }
	];

	for (let y = 0; y < heightPx; y++) {
		const shiftedY = (y + Math.floor(progress)) % heightPx;
		streams.forEach((st, idx) => {
			const waveAngle = y * st.waveFreq + ticks * 0.04 * st.speed + idx * 2.1;
			const waveX = Math.floor(widthPx * st.xRatio + Math.sin(waveAngle) * st.amp);
			const halfW = Math.floor(st.width / 2);
			const leftX = Math.max(0, waveX - halfW);
			const rightX = Math.min(widthPx, waveX + halfW);
			const spanW = rightX - leftX;
			if (spanW <= 0) return;
			ctx.fillStyle = st.colMid;
			ctx.fillRect(leftX, shiftedY, spanW, 1);
			const innerW = Math.max(2, Math.floor(spanW * 0.5));
			const innerX = leftX + Math.floor((spanW - innerW) / 2);
			ctx.fillStyle = st.colLight;
			ctx.fillRect(innerX, shiftedY, innerW, 1);
			if ((y + idx * 5) % 16 === 0) {
				ctx.fillStyle = st.colFoam;
				ctx.fillRect(innerX + Math.floor(innerW / 3), shiftedY, Math.max(1, Math.floor(innerW / 3)), 1);
			}
		});
	}
}

