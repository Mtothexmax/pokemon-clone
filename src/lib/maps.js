/**
 * maps.js — Pallet Town and Oak's lab as tile grids + pure game data.
 *
 * Legend (outside): , grass  . path  : sand  ~ lake  r river  T tree  F flowers
 *   t small tree (solid, generated; some bear fruit)
 *   G tall grass (walkable, sways)  W meadow (walkable, sways)
 *   V flower field (walkable, generated)  Y wheat field (walkable)
 *   * potted plant (solid)  f fence  S sign  R roof  H wall  w window
 *   d door (smart, walkable warp)
 * Legend (lab): L floor  = counter  B shelf  P pod  M/N doormat halves
 *   X/Q/Z lab equipment (solid: rack/analyzer/incubator)  H wall
 *   d door (smart, walkable warp)
 *
 * Rows are usually ASCII strings, but a cell may be a descriptor instead —
 * see cells.js. The lab desk row uses `desk()` (floor + desk + monitor layers,
 * left/middle/right auto-derived) and the door row uses `door()`.
 *
 * Solid tiles block movement; d/M/L are walkable. No DOM here — the
 * headless Node test imports this directly.
 */

import { SOLID } from './tiles.js';
import { desk, door, normCell } from './cells.js';

export { SOLID };

const PALLET = [
	'TTTTTTTTTTTTTTTTTTTTTTTTTTTTTGTTTTTTTTTTTTTTTTTT',
	'T,,,,,,,,,,,,,,,,,,,,,,,rr,,,.,,,,,,,,,,,,,,,,,T',
	'T,,,,,,,,,,,,,,,,,,,,,,,rr,S,.,,,,,,,,,,,,,,,,,T',
	'T,,,,,,,,,,,,,,,,,,,,,,,rr,,,.,,,,,,,,,,,,,,,,,T',
	'T,t,,RRRRRRR,,,,,,,,,,,,rr,,,.,,,,,,,,,,,,,,TTTT',
	'T,,,,RRRRRRR,,,,,,,,,,,,rr,,,.,,,,,,,,,,,,,,TTTT',
	'T,,,,HHHHHHH,,,,,,t,,,,,rr,,,.,,,,,,,,,,,,,,TTTT',
	'T,,,,HHHdHHH,,,,,,,,t,,,rr,,,.,,,,,,,,,,,,,,TTTT',
	'T,,,,,,,.,,,FFS,,,,,,,,,rr,,,.,,RRRRR,RRRRRRTTTT',
	'T,,,,,,,.,ffff,b,,,,,,,,rr,,,.,,RRRRR,RRRRRRTTTT',
	'TTT,,,,,.,,,,,,,,,,,,,,,rr,,,.,,RRRRR,RRRRRRTTTT',
	'TT,,,,,,.,,,,,,,,,,,,,,,rr,,,.,,HdHHH,HHHHdHTTTT',
	'T,,,,,,,.,,,,,,F,,,,,,,,rr,,,.FF,,.,b,,,,,,,TTTT',
	'TTTt,,,,.,,,,,,,,,,,,,,,rr,,,.,,,,.,,,,,t,,,TTTT',
	'T,,,,,t,.,,,,,,,,,,,,,,,rr,,,.ffff..fff,,,,,TTTT',
	'TTTT,,,,.,,,,,,,,,,,,,,,rr,,,.,,,,,,,,,,,,,,,,,T',
	'T,......................++..................TTTT',
	'T,,,,,,,,,,,,,,F,,,,,,,,rr,,,.,,,,,Ft,,,,,,,,,,T',
	'T,,,,,,,,,,,,,,,,,,,,,b,rr,,,,t,,,,,,,,,,,,,,,,T',
	'T,fffffffffff,,,,,,,,,,,rr,,,,,,,,,,,,,,,,,,,,,T',
	'T,,,RRRRRRR,,,,,,,,,,,,,rr,,,,,,,,,,,,,,b,,,t,,T',
	'T,,,RRRRRRR,,,,,,,,,,,,,rr,,,,,,,,,,,,,,,,,,,,,T',
	'T,,,HHHHHHH,,,,,,,,,,,,,rr,,,,,,,,,,,,,,,,,,,,,T',
	'T,,,HHHdHHH,,,,,,,,,,,,,rr,,,,,,,,,,,,,,,,,,,,,T',
	'T,ffff*,*ffff,,,,,,,,,,,rr,,,,,,,,,,,fffffffff,T',
	'T,,,,,,,,,,,,S,,,,,,,,,,rr,,,,,,,,,,,VVVVVVVVV,T',
	'T,ffffffffff,,,,,,,,,,,,rr,,,,,,,,,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,,,,,,rr,,,,,,,,,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,,::::::::::b,,,,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,T::~~~~~~::,,FF,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,,::~~o~~~::,S,,,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,,::~~~~o~::,,,,,,,VVVVVVVVV,T',
	'T,YYYYYYYYYY,,,,,,,,::::::::::,,,,,,,VVVVVVVVV,T',
	'T,,,,,,,,,,,,,,,,,,,::::::::::,,,,,,,,,,,,,,,,,T',
	'T,,,,,,,,,,,,,,,,,,,,,WWWWWW,,,,,,,,,,,,,,,,,,,T',
	'TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT',
];

const LAB = [
	'HHHHHHHHHHHHHHHH',
	'HBBB==PPPP==BBBH',
	'HLLLLLLLLLLLLLLH',
	'HLLLXLLLLLLLLLLH',
	'HLLLLLLLLLLLLLLH',
	['H', '=', '=', 'L', 'L', 'L', desk(), desk(), desk(), desk(), 'L', 'L', 'L', '=', '=', 'H'],
	'HLLLLLLLLLLLLLLH',
	'HLLLLLLZLLQLLLLH',
	'HBLLLLLLLLLLLLBH',
	'HLLLLLLLLLLLLLLH',
	'HLLLLLLMNLLLLLLH',
	['H', 'H', 'H', 'H', 'H', 'H', 'H', door(), door(), 'H', 'H', 'H', 'H', 'H', 'H', 'H']
];

export const MAPS = {
	pallet: {
		name: 'PALLET TOWN',
		grid: PALLET,
		spawn: { x: 34, y: 16, dir: 'up' },
		warps: [{ x: 8, y: 7, to: 'lab', tx: 7, ty: 9, dir: 'up' }],
		// Whole-object houses (painted as facades, collision stays grid-based).
		// Big trees are generated cloud-trees (2x3 tiles); only the base row
		// blocks movement so the player can stand behind/in front of the canopy.
		props: [
			{ type: 'house', x: 5, y: 4, w: 7, style: 'lab', door: 3 },
			{ type: 'house', x: 32, y: 8, w: 5, style: 'ash', door: 1 },
			{ type: 'house', x: 38, y: 8, w: 6, style: 'gary', door: 4 },
			{ type: 'house', x: 4, y: 20, w: 7, roof: 'gray', door: 3, floors: 2 },
			{ type: 'bigtree', x: 12, y: 2, w: 2, h: 3, seed: 1337, pal: 0 },
			{ type: 'bigtree', x: 40, y: 2, w: 2, h: 3, seed: 4211, pal: 1 },
			{ type: 'bigtree', x: 16, y: 17, w: 2, h: 3, seed: 9021, pal: 0 },
			{ type: 'bigtree', x: 33, y: 17, w: 2, h: 3, seed: 5150, pal: 1 },
			{ type: 'bigtree', x: 28, y: 20, w: 2, h: 3, seed: 777, pal: 0 },
			{ type: 'bigtree', x: 41, y: 16, w: 1, h: 2, seed: 3313, pal: 0, small: true },
			{ type: 'sunflower', x: 17, y: 24, w: 3, h: 2, count: 3, seed: 4211 },
			// Whole flower-field region: drawn as ONE continuous patch (not per
			// tile) so the blooms connect across the 9x8 block. Covers the 'V'.
			{ type: 'field', x: 37, y: 25, w: 9, h: 8, seed: 20250, flowers: true }
		],
		npcs: [
			{
				id: 'mom', look: 'mom', x: 33, y: 13, dir: 'down',
				dialog: [
					'MOM: RIGHT. ALL BOYS LEAVE HOME SOME DAY.',
					'IT SAID SO ON TV.',
					'PROF. OAK, NEXT DOOR, IS LOOKING FOR YOU.'
				]
			},
			{
				id: 'farmer', look: 'farmer', x: 6, y: 25, dir: 'down',
				dialog: [
					'THE WHEAT IS HIGH THIS YEAR.',
					'MILTANK GIVES THE BEST MILK, THEY SAY.'
				]
			},
			{
				id: 'lass', look: 'lass', x: 20, y: 17, dir: 'left',
				dialog: [
					'THE RIVER RUNS ALL THE WAY TO THE SEA.',
					'I SAW A MAGIKARP SPLASHING ONCE!'
				]
			}
		],
		signs: [
			{ x: 27, y: 2, text: ["PALLET TOWN.", 'SHADES OF YOUR JOURNEY AWAIT!'] },
			{ x: 14, y: 8, text: ["OAK POKEMON RESEARCH LAB"] },
			{ x: 13, y: 25, text: ['GREENHILL FARM.', 'FRESH MILK DAILY.'] },
			{ x: 31, y: 30, text: ['QUIET POND.', 'MAGIKARP, MAYBE?'] }
		]
	},
	lab: {
		name: "OAK'S LAB",
		grid: LAB,
		spawn: { x: 7, y: 9, dir: 'up' },
		props: [],
		warps: [
			{ x: 7, y: 11, to: 'pallet', tx: 8, ty: 8, dir: 'down' },
			{ x: 8, y: 11, to: 'pallet', tx: 8, ty: 8, dir: 'down' }
		],
		npcs: [
			{
				id: 'oak', look: 'oak', x: 8, y: 4, dir: 'down',
				dialog: [
					'OAK: HELLO THERE! WELCOME TO THE WORLD OF POKEMON!',
					'MY NAME IS OAK. PEOPLE CALL ME THE POKEMON PROF.',
					'THIS WORLD IS INHABITED BY CREATURES CALLED POKEMON.',
					'THREE STARTER POKEMON WAIT IN THE PODS. PICK ONE... LATER!'
				]
			}
		],
		signs: []
	}
};

export function tileAt(map, x, y) {
	const c = normCell(map, x, y);
	return c.ch ?? c.ground;
}

/** Big-tree prop occupying (x,y), or null. */
export function bigTreeAt(mapId, x, y) {
	const map = MAPS[mapId];
	return (map.props ?? []).find(
		(p) =>
			p.type === 'bigtree' &&
			x >= p.x && x < p.x + (p.w ?? 2) &&
			y >= p.y && y < p.y + (p.h ?? 3)
	) ?? null;
}

/** Solid tile, big-tree base, or NPC on it? Doors (d) and mats are walkable. */
export function isSolid(mapId, x, y, npcs) {
	const map = MAPS[mapId];
	if (normCell(map, x, y).solid) return true;
	const tree = bigTreeAt(mapId, x, y);
	if (tree && y === tree.y + (tree.h ?? 3) - 1) return true; // base row only
	return npcs.some((n) => n.x === x && n.y === y);
}

export function warpAt(mapId, x, y) {
	return MAPS[mapId].warps.find((w) => w.x === x && w.y === y) ?? null;
}

export function npcAt(npcs, x, y) {
	return npcs.find((n) => n.x === x && n.y === y) ?? null;
}

export function signAt(mapId, x, y) {
	return MAPS[mapId].signs.find((s) => s.x === x && s.y === y) ?? null;
}

/** Validate all rows share one width + known chars (fails loudly in tests). */
export function validateMaps() {
	const errors = [];
	const KNOWN = ',.~:TFtfSRHwDL=BP*MN+YobrGEWVdKXQZ';
	for (const [id, m] of Object.entries(MAPS)) {
		const w = m.grid[0].length;
		m.grid.forEach((row, i) => {
			if (row.length !== w) errors.push(`${id} row ${i}: ${row.length} != ${w}`);
			for (const cell of row) {
				if (typeof cell !== 'string') continue; // descriptor cells are free-form
				if (!KNOWN.includes(cell)) errors.push(`${id}: unknown tile '${cell}'`);
			}
		});
	}
	return errors;
}
