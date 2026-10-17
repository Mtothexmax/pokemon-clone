/**
 * cells.js — flexible map cells: string shorthand | layer tuple | descriptor.
 *
 * A grid row is normally an ASCII string (one char per cell), but any cell may
 * instead be a descriptor. Descriptors let you describe several stacked things
 * at one tile and let the engine derive what used to need extra tile chars:
 *
 *   import { desk, door, run, tile, stack } from './cells.js';
 *
 *   const LAB = [
 *     'HHHHHHHH',
 *     [ 'H','=','L', desk(), desk(), desk(), desk(), 'L','=','H' ],
 *   ];
 *
 * `run('desk')` marks a layer whose left/middle/right position is derived from
 * its horizontal neighbours at draw time — no DeskLeft/DeskMid/DeskRight tiles.
 *
 * `normCell(map, x, y)` is the single source of truth for the renderer,
 * collision and the debug inspector. It returns:
 *   { ch, ground, layers: [{ name, params, run }], solid, stamp }
 */
import { GROUND, SOLID } from './tiles.js';

// ---- builders ------------------------------------------------------------

/** One layer: painter `name` (see tiles.js `painters`) + optional params. */
export function layer(name, params = null, run = false) {
	return { name, params, run };
}
export const tile = (name, params) => layer(name, params, false);
export const run = (name, params) => layer(name, params, true);

/** A cell with an explicit ground char + stacked layers (bottom -> top). */
export function stack(ground, ...layers) {
	return { ground, layers };
}

/** A multi-tile object drawn as one sprite (declared like a prop). */
export function stamp(type, w, h, params = {}) {
	return { stamp: { type, w, h, ...params } };
}

// ---- game cell registry (ASCII shorthand) --------------------------------
// These are what the string grids' chars resolve to. Inline `desk()`/`door()`
// in an array row are equivalent.

/** Oak's lab desk: floor + desk + a row of monitors (all run-aware). */
export function desk() {
	return { id: 'K', ground: 'L', solid: true, layers: [run('desk'), run('computer')] };
}
/** A door: a single run-aware char, so left/right leaves join automatically. */
export function door() {
	return { id: 'd', ground: 'H', layers: [run('d')] };
}

export const CELLS = {
	d: door(),
	K: desk()
};

// ---- resolution ----------------------------------------------------------

function toLayer(x) {
	if (typeof x === 'string') return layer(x);
	return { name: x.name, params: x.params ?? null, run: !!x.run };
}

function resolveDesc(d) {
	const layers = (d.layers ?? []).map(toLayer);
	const ground = d.ground ?? GROUND[layers[0]?.name] ?? ',';
	const solid = !!d.solid || layers.some((l) => SOLID.has(l.name));
	return { ch: d.id ?? null, ground, layers, solid, stamp: d.stamp ?? null };
}

/** Normalize the raw grid cell at (x,y) into the canonical descriptor. */
export function normCell(map, x, y) {
	const row = map.grid[y];
	const raw = !row || x < 0 || x >= row.length ? '#' : row[x];
	if (raw == null) return { ch: '#', ground: '#', layers: [], solid: true, stamp: null };
	if (typeof raw === 'string') {
		const desc = (map.cells && map.cells[raw]) || CELLS[raw];
		if (desc) return { ...resolveDesc(desc), ch: raw };
		const g = GROUND[raw] ?? raw;
		return { ch: raw, ground: g, layers: GROUND[raw] ? [layer(raw)] : [], solid: SOLID.has(raw), stamp: null };
	}
	if (Array.isArray(raw)) {
		const layers = raw.map(toLayer);
		const ground = GROUND[layers[0]?.name] ?? ',';
		return { ch: null, ground, layers, solid: layers.some((l) => SOLID.has(l.name)), stamp: null };
	}
	return resolveDesc(raw);
}

export const RUN_POS = ['left', 'middle', 'right', 'single'];

/** Run position (0 left, 1 middle, 2 right, 3 single) of layer `name` at (x,y). */
export function runPos(map, x, y, name) {
	const has = (nx) => normCell(map, nx, y).layers.some((l) => l.name === name);
	const l = has(x - 1);
	const r = has(x + 1);
	if (l && r) return 1;
	if (l) return 2;
	if (r) return 0;
	return 3;
}
