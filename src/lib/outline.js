/**
 * outline.js — one shared "cloud outline" tool for organic silhouettes.
 *
 * Two entry points, same idea (puffy bumpy edges from noise offsets):
 * - cloudRing(ops, ...): vector ring — stacked noisy circles in outline
 *   color. Used by vector-painted canopies (bush).
 * - edgeOutline(isFilled, paint, ...): matrix pass — finds silhouette
 *   edge cells of pixel art and grows noise-selected bumps outward.
 *   Used by matrix sprites (tree) without touching their data.
 *
 * ops = {rect, px} (canvas-bound or test stub). Pure + headless-testable.
 */
import { fbm } from './noise.js';

/** Filled disc via ops (r in px). */
function disc(ops, cx, cy, r, color, plot) {
	for (let y = Math.floor(cy - r); y <= Math.ceil(cy + r); y++) {
		for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x++) {
			const dx = x - cx;
			const dy = y - cy;
			if (dx * dx + dy * dy <= r * r) plot(x, y, color);
		}
	}
}

/**
 * Fuzzy filled blob: a disc whose radius wobbles with noise around the
 * angle, so the silhouette itself is irregular (not a perfect circle).
 * (cx,cy) center, r radius, amp wobble amount, seed varies the lump.
 */
export function cloudDisc(ops, cx, cy, r, color, seed, amp = 3) {
	const plot = (x, y, c) => ops.rect(x, y, 1, 1, c);
	const R = r + amp;
	for (let y = Math.floor(cy - R); y <= Math.ceil(cy + R); y++) {
		for (let x = Math.floor(cx - R); x <= Math.ceil(cx + R); x++) {
			const dx = x - cx;
			const dy = y - cy;
			const d = Math.sqrt(dx * dx + dy * dy);
			const a = Math.atan2(dy, dx);
			const wob = (fbm(Math.cos(a) * 1.5 + seed * 9.1, Math.sin(a) * 1.5, 2, seed) - 0.5) * amp * 2;
			if (d <= r + wob) plot(x, y, color);
		}
	}
}

/**
 * Puffy cloud ring: bumps around an ellipse, radii noise-offset.
 * (cx,cy) center, (rx,ry) radii, seed varies the bumps.
 */
export function cloudRing(ops, cx, cy, rx, ry, seed, color) {
	const plot = (x, y, c) => ops.rect(x, y, 1, 1, c);
	const steps = Math.max(16, Math.floor((rx + ry) * 1.2));
	for (let i = 0; i < steps; i++) {
		const a = (i / steps) * Math.PI * 2;
		const wob = (fbm(Math.cos(a) * 1.4 + seed * 9.1, Math.sin(a) * 1.4, 2, seed) - 0.5) * 6;
		const px = cx + Math.cos(a) * (rx + wob);
		const py = cy + Math.sin(a) * (ry + wob);
		disc(ops, px, py, 2.4, color, plot);
	}
}

/**
 * Grow organic bumps on a matrix silhouette: empty cells touching filled
 * cells get outline paint where fbm exceeds thresh. isFilled(x,y)->bool
 * (return false out of bounds), paint(x,y,color) draws one px.
 */
export function edgeOutline(isFilled, paint, w, h, seed, color, thresh = 0.68) {
	for (let y = 0; y < h; y++) {
		for (let x = 0; x < w; x++) {
			if (isFilled(x, y)) continue;
			let adj = false;
			for (let dy = -1; dy <= 1 && !adj; dy++) {
				for (let dx = -1; dx <= 1; dx++) {
					if (!dx && !dy) continue;
					if (isFilled(x + dx, y + dy)) {
						adj = true;
						break;
					}
				}
			}
			if (!adj) continue;
			if (fbm(x * 0.31 + seed * 13.7, y * 0.31 + seed * 5.3, 2, seed) > thresh) {
				paint(x, y, color);
			}
		}
	}
}
