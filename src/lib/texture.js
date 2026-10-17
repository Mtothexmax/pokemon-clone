/**
 * texture.js — procedural micro-textures, coated onto sprites and tiles.
 *
 * A texture is a small tileable canvas (weave, denim twill, polka dots).
 * `coat()` stamps it over a region with low alpha + multiply/overlay, so
 * base colors gain fabric without extra sprite work. `shade()` adds a
 * soft directional shadow (light comes from the left).
 *
 * Only `makeTex`/`coat` touch the DOM (called from chargen at boot);
 * `TEX` recipes are pure data for headless tests.
 */

/** Recipe table: id -> {size, pixels fn(x, y) -> 0..1 coverage}. */
export const TEX = {
	/** Cross-hatch fabric (shirts, lab coats). */
	weave: {
		size: 4,
		fn: (x, y) => ((x + y) % 2 === 0 ? 0.5 : 0)
	},
	/** Diagonal twill (jeans, overalls). */
	denim: {
		size: 4,
		fn: (x, y) => ((x - y + 8) % 4 === 0 ? 0.6 : 0)
	},
	/** Polka dots (dresses). */
	dots: {
		size: 6,
		fn: (x, y) => (x % 3 === 1 && y % 3 === 1 ? 0.55 : 0)
	},
	/** Fine knit (sweaters, hats). */
	knit: {
		size: 3,
		fn: (x, y) => ((x + y) % 3 === 0 ? 0.4 : 0)
	},
	// --------------------------------------------------- terrain recipes
	// The same coater serves tiles: water gets current bands + foam.
	/** Horizontal current bands (water). */
	waves: {
		size: 16,
		fn: (x, y) => (y % 5 === 0 ? 0.4 : 0)
	},
	/** Sparse foam speckle (water, shores). */
	foam: {
		size: 8,
		fn: (x, y) => ((x * 5 + y * 3) % 7 === 0 ? 0.5 : 0)
	},
	/** Pebble grain (paths, caves). */
	grain: {
		size: 8,
		fn: (x, y) => ((x * 3 + y * 7) % 5 === 0 ? 0.45 : 0)
	}
};

/** Bake a recipe into a tileable canvas (dark pixels on transparent). */
export function makeTex(id) {
	const r = TEX[id];
	if (!r) throw new Error('unknown texture: ' + id);
	const c = document.createElement('canvas');
	c.width = r.size;
	c.height = r.size;
	const g = c.getContext('2d');
	for (let y = 0; y < r.size; y++) {
		for (let x = 0; x < r.size; x++) {
			const a = r.fn(x, y);
			if (a > 0) {
				g.fillStyle = `rgba(0,0,24,${a * 0.35})`;
				g.fillRect(x, y, 1, 1);
			}
		}
	}
	return c;
}

/**
 * Stamp texture `id` over rect (tiled), darkening slightly like fabric.
 * Alpha scales the effect (0.5 = subtle, 1 = full recipe strength).
 */
export function coat(g, id, x, y, w, h, alpha = 0.7) {
	const tex = makeTex(id);
	g.save();
	g.globalAlpha = alpha;
	g.globalCompositeOperation = 'multiply';
	const pat = g.createPattern(tex, 'repeat');
	g.translate(x, y);
	g.fillStyle = pat;
	g.fillRect(0, 0, w, h);
	g.restore();
}
