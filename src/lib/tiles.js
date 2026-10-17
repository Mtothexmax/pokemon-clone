/**
 * tiles.js — procedural pixel art, zero image files.
 *
 * Every tile/sprite is painted once into an offscreen 16x16 canvas;
 * the game then only blits. `document` is touched solely inside the
 * build functions (called from main.js), so this file imports cleanly
 * in Node for headless tests.
 */

import { coat } from './texture.js';
import { TREE_PAL, TREE } from './tree.js';
import { fbm } from './noise.js';
import { cloudDisc, edgeOutline } from './outline.js';
import { drawFlowerField, GAME_FIELD_PAL } from './flowerfield.js';
import { drawSmallTree } from './smalltree.js';
import { drawHouse } from './housegen.js';
import { drawCyberLab, drawAshHouse, drawGaryHouse } from './buildings.js';
import { drawRiver, drawSeamlessRiver, SEAMLESS_RIVER_PALETTES } from './rivergen.js';
import { drawLabEquipment } from './labgen.js';

/** Solid tiles block movement (checked via cells.normCell). '# is the
 * synthetic void outside a map's grid (painted black, blocks). */
export const SOLID = new Set(['#', '~', 'r', 'T', 't', 'f', 'S', 'R', 'H', 'w', '=', 'B', 'P', '*', 'o', 'b', 'K', 'X', 'Q', 'Z']);

/** Ground beneath object tiles (two-pass render: ground, then decor).
 * Object painters below draw ONLY their object on transparency. Full-cover
 * tiles (Y wheat, M mat, B shelf, R/H/w/d walls) need no entry. */
export const GROUND = {
	T: ',', F: ',', f: ',', S: ',', b: ',', t: ',', '*': ',',
	'=': 'L', P: 'L', X: 'L', Q: 'L', Z: 'L',
	o: '~', '+': ',', V: ',', r: ',',
	d: 'H', K: 'L', G: ',', W: ','
};

/** Small tree variants (see smalltree.js). Exactly one bears fruit so the
 * orchard reads as "some trees have apples", not a uniform crop. */
export const SMALLTREE_VARIANTS = [
	{ seed: 1337, paletteIndex: 0, trunkStyle: 'straight', trunkHeight: 9, foliageWidth: 9, cloudRadius: 3.4, strokeOffset: 1, showBranches: true, showGroundShadow: true, showFruits: true, fruitCount: 3, fruitType: 'apples', fruitScatter: 'spread' },
	{ seed: 4211, paletteIndex: 1, trunkStyle: 'curved', trunkHeight: 10, foliageWidth: 9, cloudRadius: 3.2, strokeOffset: 1, showBranches: true, showGroundShadow: true, showFruits: false, fruitCount: 0 },
	{ seed: 9021, paletteIndex: 0, trunkStyle: 'birch', trunkHeight: 11, foliageWidth: 8, cloudRadius: 3.0, strokeOffset: 1, showBranches: false, showGroundShadow: true, showFruits: false, fruitCount: 0 },
	{ seed: 5150, paletteIndex: 2, trunkStyle: 'straight', trunkHeight: 9, foliageWidth: 9, cloudRadius: 3.3, strokeOffset: 1, showBranches: true, showGroundShadow: true, showFruits: false, fruitCount: 0 },
	{ seed: 777, paletteIndex: 1, trunkStyle: 'multistem', trunkHeight: 8, foliageWidth: 10, cloudRadius: 3.6, strokeOffset: 1, showBranches: true, showGroundShadow: true, showFruits: false, fruitCount: 0 },
	{ seed: 616, paletteIndex: 0, trunkStyle: 'curved', trunkHeight: 10, foliageWidth: 9, cloudRadius: 3.2, strokeOffset: 1, showBranches: true, showGroundShadow: true, showFruits: false, fruitCount: 0 }
];

/** Lab equipment variants (see labgen.js) — server rack, analyzer, incubator. */
export const LAB_EQUIP = [
	{ itemType: 'mainframe', itemWidth: 16, itemHeight: 20, deskStyle: 'tech' },
	{ itemType: 'analyzer', itemWidth: 22, itemHeight: 16, deskStyle: 'tech' },
	{ itemType: 'incubator', itemWidth: 20, itemHeight: 20, deskStyle: 'tech' }
];

/** Layers whose left/middle/right variant is baked (see cells.runPos). */
export const RUN_LAYERS = new Set(['desk', 'computer', 'd', 'mat']);
/** Deterministic 0..1 hash for decoration scatter. */
export function hash(x, y, s = 0) {
	let h = (x * 374761393 + y * 668265263 + s * 974634) | 0;
	h = (h ^ (h >> 13)) * 1274126177;
	return ((h ^ (h >> 16)) >>> 0) / 4294967295;
}

const C = {
	// outlines + light (light comes from top-left, like the reference)
	edge: '#1e2b1a', edgeWood: '#3a2a1a', shadow: 'rgba(0,0,20,0.22)',
	grass1: '#7ec850', grass2: '#69b840', grassDark: '#4c9440', grassHi: '#a4e878', tuft: '#3f8f3f',
	path1: '#e8d5a0', path2: '#d9c48c', pathDark: '#c0a878', pebble: '#b09a70', pebbleHi: '#e8dcc0',
	sand1: '#f0e0b0', sand2: '#e2d09a',
	water1: '#3f8fd8', water2: '#2f6fb8', waterDark: '#2560a0', foam: '#d8f0ff',
	trunk: '#7a5230', trunkDark: '#4e342e', trunkHi: '#a88058',
	leafD: '#1e7a38', leaf1: '#2e9e4f', leaf2: '#3fbf5f', leafHi: '#7fe890',
	fence: '#f2ede2', fenceDark: '#b8b0a0', fenceShadow: '#8a8478',
	signPost: '#7a5230', signBoard: '#c89868', signInk: '#4e342e', signHi: '#e8c898',
	roofR: '#d8452e', roofRd: '#a02818', roofRh: '#f4785a',
	roofB: '#4a7fd4', roofBd: '#2f56a0', roofBh: '#86b4f0',
	roofG: '#8a94a6', roofGd: '#5e6675', roofGh: '#bcc4d4',
	wall: '#f5efe0', wallDark: '#d8cbb0', siding: '#e2d8c2',
	winGlass: '#3f7fc8', winGlassD: '#2a5a94', winShine: '#cfe8ff', winFrame: '#ffffff',
	door: '#8d5524', doorDark: '#5e3a16', doorHi: '#b87f42', knob: '#f1c40f',
	flowerR: '#e74c3c', flowerY: '#f1c40f', flowerW: '#fafafa', flowerP: '#ec7fa9', stem: '#2e8f3f',
	floorL: '#dcdcdc', floorD: '#c4c4c4', floorEdge: '#a8a8a8',
	counter: '#e8e8e8', counterD: '#808080', counterT: '#f8f8f8',
	shelf: '#6d4c41', shelfD: '#4e342e', bookR: '#c0392b', bookB: '#21618c', bookG: '#27ae60',
	podBase: '#78909c', podBaseD: '#546e7a', podGlass: '#9fd8f5', podBulb: '#2e9e5b', podChar: '#e67e22', podSquir: '#2f6fd0',
	mat: '#c0392b', matD: '#96281b', matIn: '#f7f7f7',
	leaf: '#2e7d32', leafD2: '#1b5e20', pot: '#8d5524', potD: '#6b3f1a'
};

function px(g, x, y, c) {
	g.fillStyle = c;
	g.fillRect(x, y, 1, 1);
}
function rect(g, x, y, w, h, c) {
	g.fillStyle = c;
	g.fillRect(x, y, w, h);
}

/** Scatter n single pixels of color c inside the 32px tile. */
function scatter(g, ox, oy, tx, ty, n, c, seed) {
	for (let i = 0; i < n; i++) {
		px(g, ox + Math.floor(hash(tx, ty, seed + i) * 32), oy + Math.floor(hash(tx, ty, seed + 50 + i) * 32), c);
	}
}

/** 5px grass tuft. */
function tuft(g, ox, oy, x, y) {
	px(g, ox + x, oy + y, C.tuft);
	px(g, ox + x, oy + y + 1, C.tuft);
	px(g, ox + x, oy + y + 2, C.tuft);
	px(g, ox + x - 1, oy + y + 2, C.grassHi);
	px(g, ox + x + 1, oy + y + 1, C.grassHi);
	px(g, ox + x, oy + y + 3, C.grassDark);
}

/** Paint one lab-equipment variant (see LAB_EQUIP) into a 32px tile. */
function paintLabEquip(g, ox, oy, i) {
	const cfg = LAB_EQUIP[i] ?? LAB_EQUIP[0];
	g.save();
	if (ox || oy) g.translate(ox, oy);
	drawLabEquipment(g, 32, 32, {
		...cfg,
		seed: 1337 + i * 97,
		paletteIndex: 0,
		showGroundShadow: true,
		showScreenGlow: true,
		cablesLevel: 0,
		showPokeballs: false,
		showPapers: false,
		showBooks: false,
		showBeakers: false
	});
	g.restore();
}

const painters = {
	',': (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.grass1);
		const bx = Math.floor(hash(tx, ty, 1) * 20);
		const by = Math.floor(hash(tx, ty, 2) * 20);
		rect(g, ox + bx, oy + by, 11, 8, C.grass2);
		rect(g, ox + bx + 2, oy + by + 2, 7, 4, C.grassDark);
		const bx2 = Math.floor(hash(tx, ty, 21) * 22);
		const by2 = Math.floor(hash(tx, ty, 22) * 22);
		rect(g, ox + bx2, oy + by2, 8, 5, C.grass2);
		scatter(g, ox, oy, tx, ty, 16, C.grassDark, 3);
		scatter(g, ox, oy, tx, ty, 8, C.grassHi, 18);
		scatter(g, ox, oy, tx, ty, 5, C.leaf1, 23);
		tuft(g, ox, oy, 4 + Math.floor(hash(tx, ty, 4) * 24), 6 + Math.floor(hash(tx, ty, 5) * 20));
		tuft(g, ox, oy, 4 + Math.floor(hash(tx, ty, 6) * 24), 6 + Math.floor(hash(tx, ty, 7) * 20));
		tuft(g, ox, oy, 4 + Math.floor(hash(tx, ty, 19) * 24), 6 + Math.floor(hash(tx, ty, 20) * 20));
		tuft(g, ox, oy, 4 + Math.floor(hash(tx, ty, 24) * 24), 6 + Math.floor(hash(tx, ty, 25) * 20));
	},
	'.': (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.path1);
		coat(g, 'grain', ox, oy, 32, 32, 0.8);
		const bx = Math.floor(hash(tx, ty, 8) * 20);
		rect(g, ox + bx, oy + 6 + Math.floor(hash(tx, ty, 9) * 18), 12, 6, C.path2);
		scatter(g, ox, oy, tx, ty, 8, C.pathDark, 10);
		for (let i = 0; i < 3; i++) {
			const qx = ox + 4 + Math.floor(hash(tx, ty, 11 + i) * 24);
			const qy = oy + 4 + Math.floor(hash(tx, ty, 21 + i) * 24);
			rect(g, qx, qy, 3, 2, C.pebble);
			px(g, qx, qy, C.pebbleHi);
		}
	},
	':': (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.sand1);
		scatter(g, ox, oy, tx, ty, 16, C.sand2, 5);
		scatter(g, ox, oy, tx, ty, 5, C.pebble, 13);
	},
	'~': [
		(g, ox, oy, tx, ty) => {
			rect(g, ox, oy, 32, 32, C.water1);
			rect(g, ox, oy + 4 + Math.floor(hash(tx, ty, 15) * 10), 32, 8, C.water2);
			coat(g, 'waves', ox, oy, 32, 32, 0.9);
			coat(g, 'foam', ox, oy, 32, 32, 0.6);
			for (const [gx, gy] of [[8, 10], [20, 20], [25, 12], [4, 24]]) {
				px(g, ox + gx, oy + gy, C.foam); px(g, ox + gx + 1, oy + gy, C.foam);
			}
		},
		(g, ox, oy, tx, ty) => {
			rect(g, ox, oy, 32, 32, C.water1);
			rect(g, ox, oy + 2 + Math.floor(hash(tx, ty, 15) * 10), 32, 8, C.water2);
			coat(g, 'waves', ox, oy, 32, 32, 0.9);
			coat(g, 'foam', ox, oy, 32, 32, 0.6);
			px(g, ox + 8, oy + 10, C.foam); px(g, ox + 9, oy + 10, C.foam);
			px(g, ox + 20, oy + 20, C.foam); px(g, ox + 21, oy + 20, C.foam);
		}
	],
	// River: the ported "Nahtloses Flusswasser" seamless generator (rivergen.js).
	// Fully periodic in Y so the 32px tiles join without a seam; drawn
	// semi-transparent so the meadow ground tile shows through. 8 phases
	// advance the flow. Left/right tiles sample the same 64px-wide pattern so
	// the two river columns read as ONE stream (not two identical waves).
	'r': [0, 1, 2, 3, 4, 5, 6, 7].map((ph) => (g, ox, oy, tx, ty) => {
		g.save();
		if (ox || oy) g.translate(ox, oy);
		g.globalAlpha = 0.82;
		drawSeamlessRiver(g, 32, 32, ph * 4, SEAMLESS_RIVER_PALETTES.gbaClassic, {
			patternWidth: 64,
			xOffset: (((tx % 2) + 2) % 2) * 32
		});
		g.globalAlpha = 1;
		g.restore();
	}),
	T: (g, ox, oy, tx, ty) => {
		// Transparent: ground pass paints the grass, this draws only the tree.
		// Per-variant treatment so no two trees share the exact noise.
		const vv = ((tx % 4) + 4) % 4;
		g.fillStyle = C.shadow;
		g.fillRect(ox + 4, oy + 27, 24, 4);
		for (let y = 0; y < 32; y++) {
			for (let x = 0; x < 32; x++) {
				const v = TREE[y][x];
				if (!v) continue;
				px(g, ox + x, oy + y, TREE_PAL[v]);
			}
		}
		// Sun dapple through the crown: seeded per cell, subtle.
		const dappleN = 8 + vv * 2;
		for (let i = 0; i < dappleN; i++) {
			const dx = Math.floor(hash(tx, ty, 60 + i) * 30) + 1;
			const dy = Math.floor(hash(tx, ty, 70 + i) * 30) + 1;
			const n = fbm(tx * 0.9 + dx * 0.35, ty * 0.9 + dy * 0.35, 2, i + vv * 13);
			if (n > 0.62) {
				px(g, ox + dx, oy + dy, C.leafHi);
			} else if (n < 0.36) {
				px(g, ox + dx, oy + dy, C.leafD);
			}
		}
		if (vv >= 2) {
			px(g, ox + 11, oy + 5, C.leafHi);
			px(g, ox + 20, oy + 7, C.leafHi);
		}
		// Organic silhouette puffs (same outline shader as the bush).
		const isF = (x, y) => x >= 0 && y >= 0 && x < 32 && y < 32 && !!TREE[y][x];
		edgeOutline(isF, (x, y, c) => px(g, ox + x, oy + y, c), 32, 32, tx * 3.7 + ty * 9.1, TREE_PAL[1], 0.74 - vv * 0.045);
	},
	// shrub: irregular cloud-noise silhouette (not a circle), shaded crown
	b: (g, ox, oy, tx, ty) => {
		g.fillStyle = C.shadow;
		g.fillRect(ox + 8, oy + 25, 16, 3);
		const cx = 16 + Math.floor(hash(tx, ty, 81) * 5) - 2;
		const ops = {
			rect: (x, y, w, h, c) => rect(g, ox + x, oy + y, w, h, c)
		};
		const seed = hash(tx, ty, 83) * 100;
		// Fuzzy outline layer, then fuzzy green body: the whole shape is lumpy.
		cloudDisc(ops, cx, 15, 10.5, C.edge, seed, 3.2);
		cloudDisc(ops, cx, 15, 9, C.leafD, seed + 1.7, 2.9);
		cloudDisc(ops, cx - 1, 13.5, 6.5, C.leaf1, seed + 3.3, 2.4);
		cloudDisc(ops, cx - 2, 12, 3.8, C.leaf2, seed + 4.9, 1.8);
		const n = Math.floor(hash(tx, ty, 82) * 4);
		px(g, ox + cx - 4 + n, oy + 10, C.leafHi);
		px(g, ox + cx - 2 + n, oy + 9, C.leafHi);
		px(g, ox + cx + 3 - n, oy + 12, C.leaf1);
	},
	F: (g, ox, oy, tx, ty) => {
		const cols = [C.flowerR, C.flowerY, C.flowerW, C.flowerP];
		for (let i = 0; i < 5; i++) {
			const fx = ox + 2 + Math.floor(hash(tx, ty, 10 + i) * 26);
			const fy = oy + 2 + Math.floor(hash(tx, ty, 20 + i) * 24);
			const c = cols[Math.floor(hash(tx, ty, 30 + i) * 4)];
			px(g, fx + 1, fy + 4, C.stem); px(g, fx + 1, fy + 5, C.stem);
			px(g, fx, fy + 5, C.leaf1); px(g, fx + 2, fy + 5, C.leaf1);
			rect(g, fx, fy, 3, 3, c);
			rect(g, fx, fy, 3, 1, C.flowerW);
			px(g, fx + 1, fy + 1, c);
		}
		tuft(g, ox, oy, 3 + Math.floor(hash(tx, ty, 41) * 24), 4 + Math.floor(hash(tx, ty, 42) * 22));
	},
	// white picket fence with posts, shading, ground shadow
	f: (g, ox, oy, tx, ty) => {
		g.fillStyle = C.shadow;
		g.fillRect(ox, oy + 26, 32, 4);
		for (const px0 of [2, 13, 24]) {
			rect(g, ox + px0, oy + 6, 6, 21, C.fence);
			rect(g, ox + px0, oy + 6, 2, 21, C.fenceDark);
			rect(g, ox + px0, oy + 6, 6, 2, '#ffffff');
			px(g, ox + px0 + 2, oy + 4, C.fence);
			px(g, ox + px0 + 2, oy + 5, C.fence);
			rect(g, ox + px0, oy + 25, 6, 2, C.fenceShadow);
		}
		rect(g, ox, oy + 12, 32, 4, C.fenceDark);
		rect(g, ox, oy + 12, 32, 2, C.fence);
		rect(g, ox, oy + 20, 32, 3, C.fenceDark);
	},
	// plank bridge (walkable) with rails top/bottom, transparent over river
	'+': (g, ox, oy, tx, ty) => {
		// deck: vertical boards laid across the walkway (perpendicular to travel)
		rect(g, ox, oy + 4, 32, 24, C.signPost);
		for (let x = 0; x < 32; x += 8) {
			rect(g, ox + x, oy + 4, 1, 24, C.signHi);
			rect(g, ox + x + 7, oy + 4, 1, 24, C.edgeWood);
		}
		rect(g, ox, oy + 4, 32, 1, C.signHi);
		rect(g, ox, oy + 27, 32, 1, C.edgeWood);
		// rails along both banks (north + south)
		rect(g, ox, oy, 32, 4, C.fenceDark);
		rect(g, ox, oy, 32, 2, C.fence);
		rect(g, ox, oy + 28, 32, 4, C.fenceDark);
		rect(g, ox, oy + 30, 32, 2, C.fence);
		// rail posts at the tile seams
		for (const px0 of [0, 29]) {
			rect(g, ox + px0, oy, 3, 6, C.edgeWood);
			rect(g, ox + px0, oy + 26, 3, 6, C.edgeWood);
		}
		// plank shadow on the water beneath
		rect(g, ox, oy + 31, 32, 1, 'rgba(2,6,23,0.35)');
	},
	// wheat field: tilled rows with golden stalks (solid decor)
	Y: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.doorDark);
		for (let x = 2; x < 32; x += 6) rect(g, ox + x, oy, 2, 32, C.trunkDark);
		for (let x = 2; x < 32; x += 6) {
			for (let y = 2 + ((x + tx * 3 + ty * 5) % 3); y < 32; y += 6) {
				rect(g, ox + x - 1, oy + y, 4, 1, C.leaf);
				rect(g, ox + x, oy + y - 4, 2, 5, C.flowerY);
				rect(g, ox + x, oy + y - 4, 1, 5, C.pebbleHi);
				px(g, ox + x, oy + y - 5, C.flowerY);
			}
		}
		scatter(g, ox, oy, tx, ty, 5, C.trunkDark, 40);
	},
	// lily pads + reeds on water (transparent decor over animated lake)
	o: (g, ox, oy, tx, ty) => {
		const pad = (px0, py0, r) => {
			g.fillStyle = C.edge;
			g.beginPath(); g.arc(ox + px0, oy + py0, r + 1, 0, 7); g.fill();
			g.fillStyle = C.leaf;
			g.beginPath(); g.arc(ox + px0, oy + py0, r, 0, 7); g.fill();
			px(g, ox + px0 + 1, oy + py0 - 1, C.leafHi);
		};
		pad(9, 10, 5);
		pad(23, 20, 6);
		pad(22, 8, 4);
		px(g, ox + 9, oy + 8, C.flowerP); px(g, ox + 10, oy + 8, C.flowerP);
		// reed cluster bottom-left
		rect(g, ox + 3, oy + 20, 2, 10, C.stem);
		rect(g, ox + 6, oy + 22, 2, 8, C.grassDark);
		rect(g, ox + 2, oy + 18, 4, 2, C.flowerY);
	},
	S: (g, ox, oy, tx, ty) => {
		g.fillStyle = C.shadow;
		g.fillRect(ox + 10, oy + 28, 12, 3);
		rect(g, ox + 14, oy + 14, 4, 18, C.signPost);
		rect(g, ox + 14, oy + 14, 1, 18, C.edgeWood);
		rect(g, ox + 17, oy + 16, 1, 14, C.signHi);
		rect(g, ox + 4, oy + 2, 24, 14, C.edgeWood);
		rect(g, ox + 6, oy + 4, 20, 10, C.signBoard);
		rect(g, ox + 6, oy + 4, 20, 2, C.signHi);
		rect(g, ox + 8, oy + 8, 16, 2, C.signInk);
		rect(g, ox + 8, oy + 11, 10, 2, C.signInk);
		px(g, ox + 25, oy + 5, C.signHi);
	},
	R: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.roofR);
		for (let y = 4; y < 32; y += 8) rect(g, ox, oy + y, 32, 2, C.roofRd);
		rect(g, ox, oy, 32, 2, C.roofRh);
		rect(g, ox, oy + 30, 32, 2, C.roofRd);
	},
	H: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.wall);
		for (let y = 6; y < 32; y += 8) rect(g, ox, oy + y, 32, 1, C.wallDark);
		rect(g, ox, oy + 30, 32, 2, C.wallDark);
	},
	w: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.wall);
		rect(g, ox + 6, oy + 8, 20, 16, C.winFrame);
		rect(g, ox + 8, oy + 10, 16, 12, C.winGlass);
		rect(g, ox + 14, oy + 10, 4, 12, C.winFrame);
		rect(g, ox, oy + 30, 32, 2, C.wallDark);
	},
	// Smart door: one char for all widths. `pos` (left/middle/right/single)
	// comes from cells.runPos, so a lone door centres and a pair joins at the
	// seam — no separate D/E tiles. Transparent over the wall.
	d: (g, ox, oy, tx, ty, pos = 3) => {
		if (pos === 0) {
			rect(g, ox + 14, oy + 2, 2, 30, C.edgeWood);
			rect(g, ox + 16, oy + 2, 16, 30, C.door);
			rect(g, ox + 16, oy + 2, 16, 2, C.doorHi);
			rect(g, ox + 18, oy + 8, 4, 10, C.doorDark);
			rect(g, ox + 18, oy + 20, 4, 8, C.doorDark);
			rect(g, ox + 27, oy + 14, 2, 4, C.knob);
			rect(g, ox + 14, oy + 30, 18, 2, C.floorEdge);
		} else if (pos === 2) {
			rect(g, ox + 16, oy + 2, 2, 30, C.edgeWood);
			rect(g, ox, oy + 2, 16, 30, C.door);
			rect(g, ox, oy + 2, 16, 2, C.doorHi);
			rect(g, ox + 10, oy + 8, 4, 10, C.doorDark);
			rect(g, ox + 10, oy + 20, 4, 8, C.doorDark);
			rect(g, ox + 3, oy + 14, 2, 4, C.knob);
			rect(g, ox, oy + 30, 18, 2, C.floorEdge);
		} else {
			rect(g, ox + 6, oy + 2, 20, 30, C.edgeWood);
			rect(g, ox + 7, oy + 2, 18, 30, C.door);
			rect(g, ox + 7, oy + 2, 18, 2, C.doorHi);
			rect(g, ox + 9, oy + 8, 5, 10, C.doorDark);
			rect(g, ox + 18, oy + 8, 5, 10, C.doorDark);
			rect(g, ox + 9, oy + 20, 5, 8, C.doorDark);
			rect(g, ox + 18, oy + 20, 5, 8, C.doorDark);
			rect(g, ox + 19, oy + 15, 2, 4, C.knob);
			rect(g, ox + 6, oy + 30, 20, 2, C.floorEdge);
		}
	},
	// Doormat (run-aware): like the door leaves, each end tile carries only its
	// inner 50% and the middle tile is the full mat — so a 3-wide run reads as
	// one doormat that joins seamlessly across the tiles.
	mat: (g, ox, oy, tx, ty, pos = 3) => {
		const x0 = pos === 0 ? 16 : 0;
		const x1 = pos === 2 ? 16 : 32;
		const w = x1 - x0;
		const borderL = pos === 0 || pos === 3;
		const borderR = pos === 2 || pos === 3;
		rect(g, ox + x0, oy + 2, w, 2, C.matD);
		rect(g, ox + x0, oy + 28, w, 2, C.matD);
		if (borderL) rect(g, ox + x0, oy + 2, 2, 28, C.matD);
		if (borderR) rect(g, ox + x1 - 2, oy + 2, 2, 28, C.matD);
		const b0 = x0 + (borderL ? 2 : 0);
		const b1 = x1 - (borderR ? 2 : 0);
		rect(g, ox + b0, oy + 4, b1 - b0, 24, C.mat);
		const i0 = x0 + (borderL ? 4 : 0);
		const i1 = x1 - (borderR ? 4 : 0);
		rect(g, ox + i0, oy + 8, i1 - i0, 16, C.matIn);
	},
	// Desk surface (run-aware): continuous top + front panel across a run.
	desk: (g, ox, oy, tx, ty, pos = 3) => {
		const x0 = pos === 0 ? 4 : 0;
		const x1 = pos === 2 ? 28 : 32;
		const w = x1 - x0;
		rect(g, ox + x0, oy + 15, w, 6, C.door);
		rect(g, ox + x0, oy + 15, w, 2, C.doorHi);
		rect(g, ox + x0, oy + 15, 1, 6, C.edgeWood);
		rect(g, ox + x1 - 1, oy + 15, 1, 6, C.edgeWood);
		rect(g, ox + x0, oy + 21, w, 8, C.doorDark);
		rect(g, ox + x0, oy + 21, w, 1, C.edgeWood);
		if (pos === 0 || pos === 3) rect(g, ox + 4, oy + 29, 4, 3, C.doorDark);
		if (pos === 2 || pos === 3) rect(g, ox + 24, oy + 29, 4, 3, C.doorDark);
		g.fillStyle = C.shadow;
		g.fillRect(ox + x0, oy + 30, w, 2);
	},
	// Monitor on the desk (run-aware): a row of screens = multi-monitor setup.
	computer: (g, ox, oy, tx, ty, pos = 3) => {
		const cx = 16;
		rect(g, ox + cx - 9, oy + 2, 18, 14, C.edge);
		rect(g, ox + cx - 8, oy + 3, 16, 12, C.counterD);
		rect(g, ox + cx - 7, oy + 4, 14, 9, C.winGlass);
		rect(g, ox + cx - 7, oy + 4, 14, 2, C.winGlassD);
		rect(g, ox + cx - 5, oy + 7, 10, 1, C.podBulb);
		rect(g, ox + cx - 5, oy + 9, 7, 1, C.winShine);
		px(g, ox + cx + 3, oy + 10, C.flowerY);
		px(g, ox + cx - 5, oy + 6, C.winShine);
		rect(g, ox + cx - 3, oy + 16, 6, 2, C.counter);
		rect(g, ox + cx - 5, oy + 18, 10, 2, C.counterD);
	},
	// Tall grass swaying in the wind (walkable). Dense blades that span the
	// whole tile (full width, up to the top edge) so a patch reads as a solid
	// clump of grass, not a bare tile with a few spikes in one corner.
	G: [0, 1].map((ph) => (g, ox, oy, tx, ty) => {
		const N = 14;
		for (let i = 0; i < N; i++) {
			// Stratify across the full width (hash-only placement can cluster
			// in one half for some tiles, leaving the other half bare).
			const bx = Math.round(((i + 0.12 + hash(tx, ty, 50 + i) * 0.76) / N) * 29);
			const bh = 19 + Math.floor(hash(tx, ty, 60 + i) * 13); // 19..31
			const by = 32 - bh;
			const x = ox + bx + (ph ? 1 : 0);
			rect(g, x, oy + by, 3, bh, C.grassDark);
			rect(g, x + 1, oy + by + 2, 2, bh - 2, C.tuft);
			if (by > 0) px(g, x + 1, oy + by - 1, C.grassHi);
		}
		// A few brighter tips scattered through the clump for depth.
		for (let i = 0; i < 6; i++) {
			const bx = Math.round(((i + 0.25 + hash(tx, ty, 90 + i) * 0.5) / 6) * 29);
			const by = 4 + Math.floor(hash(tx, ty, 96 + i) * 24);
			px(g, ox + bx, oy + by, C.grassHi);
		}
		tuft(g, ox, oy, 3, 27);
		tuft(g, ox, oy, 26, 25);
	}),
	// Plain meadow: soft swaying blades, no blooms (walkable, 2 frames).
	W: [0, 1].map((ph) => (g, ox, oy, tx, ty) => {
		const lean = ph ? 1 : 0;
		const N = 9;
		for (let i = 0; i < N; i++) {
			const bx = Math.round(((i + 0.2 + hash(tx, ty, 70 + i) * 0.6) / N) * 29);
			const bh = 9 + Math.floor(hash(tx, ty, 80 + i) * 8);
			const by = 30 - bh;
			rect(g, ox + bx + lean, oy + by, 2, bh, C.grassDark);
			rect(g, ox + bx + 1 + lean, oy + by + 2, 1, bh - 2, C.grassHi);
			px(g, ox + bx + lean, oy + by - 1, C.grassHi);
		}
		tuft(g, ox, oy, 5, 25);
		tuft(g, ox, oy, 25, 21);
	}),
	// Flower field: the ported Blumenfeld-Generator. Transparent ground so the
	// grass tile painted underneath shows through ("Wiese" behind the blooms).
	V: (g, ox, oy, tx, ty) => {
		drawFlowerField(g, 32, 32, {
			seed: Math.floor(hash(tx, ty, 7) * 99999) + 1,
			palette: GAME_FIELD_PAL,
			density: 30,
			noiseScale: 0.09,
			grassDensity: 55,
			strokeOffset: 1,
			bloomRadius: 2,
			flowerType: 'mixed',
			transparent: true,
			showDirt: false,
			showStems: true
		});
	},
	L: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, (tx + ty) % 2 ? C.floorL : C.floorD);
		rect(g, ox, oy, 32, 1, C.floorEdge);
		rect(g, ox, oy + 31, 32, 1, C.floorEdge);
		rect(g, ox, oy, 1, 32, C.floorEdge);
		rect(g, ox + 31, oy, 1, 32, C.floorEdge);
	},
	'=': (g, ox, oy, tx, ty) => {
		rect(g, ox, oy + 6, 32, 20, C.edge);
		rect(g, ox + 2, oy + 8, 28, 16, C.counter);
		rect(g, ox + 2, oy + 8, 28, 4, C.counterT);
		rect(g, ox + 2, oy + 20, 28, 4, C.counterD);
		rect(g, ox + 2, oy + 8, 2, 16, C.counterT);
	},
	B: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.shelfD);
		rect(g, ox + 2, oy + 2, 28, 28, C.shelf);
		rect(g, ox + 4, oy + 4, 24, 24, '#3e2723');
		const cols = [C.bookR, C.bookB, C.bookG, C.flowerY];
		for (let r = 0; r < 3; r++) {
			rect(g, ox + 4, oy + 6 + r * 8, 24, 2, C.shelf);
			for (let i = 0; i < 11; i++)
				rect(g, ox + 4 + i * 2 + (r % 2), oy + 6 + r * 8 - 6, 2, 6, cols[(i + r + tx + ty) % 4]);
		}
	},
	P: (g, ox, oy, tx, ty, n = 0) => {
		// Wooden pedestal table with a big Pokéball (transparent bg).
		// Button dot keeps the starter color of this slot.
		rect(g, ox + 4, oy + 22, 24, 3, C.edgeWood);
		rect(g, ox + 5, oy + 22, 22, 2, C.signPost);
		rect(g, ox + 6, oy + 25, 3, 7, C.edgeWood);
		rect(g, ox + 23, oy + 25, 3, 7, C.edgeWood);
		g.fillStyle = C.edge;
		g.beginPath(); g.arc(ox + 16, oy + 12, 10, 0, 7); g.fill();
		g.fillStyle = C.winFrame;
		g.beginPath(); g.arc(ox + 16, oy + 12, 9, 0, 7); g.fill();
		g.fillStyle = C.flowerR;
		g.beginPath(); g.arc(ox + 16, oy + 12, 9, Math.PI, 0); g.fill();
		rect(g, ox + 7, oy + 11, 18, 3, C.edge);
		g.fillStyle = C.winFrame;
		g.beginPath(); g.arc(ox + 16, oy + 12.5, 3, 0, 7); g.fill();
		g.fillStyle = C.edge;
		g.beginPath(); g.arc(ox + 16, oy + 12.5, 3, 0, 7);
		g.lineWidth = 1; g.stroke();
		const pc = [C.podBulb, C.podChar, C.podSquir][(tx + n) % 3];
		px(g, ox + 15, oy + 12, pc); px(g, ox + 16, oy + 12, pc);
		px(g, ox + 15, oy + 13, pc); px(g, ox + 16, oy + 13, pc);
		px(g, ox + 11, oy + 6, '#ffffff'); px(g, ox + 12, oy + 5, '#ffffff');
	},
	// Doormat left half — pairs with N (right half) like the D/E door leaves.
	// Right edge is flush with the tile border so the two halves join.
	M: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.floorL);
		rect(g, ox + 2, oy + 2, 30, 2, C.matD);
		rect(g, ox + 2, oy + 28, 30, 2, C.matD);
		rect(g, ox + 2, oy + 2, 2, 28, C.matD);
		rect(g, ox + 4, oy + 4, 28, 24, C.mat);
		rect(g, ox + 8, oy + 8, 24, 16, C.matIn);
	},
	N: (g, ox, oy, tx, ty) => {
		rect(g, ox, oy, 32, 32, C.floorL);
		rect(g, ox, oy + 2, 30, 2, C.matD);
		rect(g, ox, oy + 28, 30, 2, C.matD);
		rect(g, ox + 28, oy + 2, 2, 28, C.matD);
		rect(g, ox, oy + 4, 28, 24, C.mat);
		rect(g, ox, oy + 8, 24, 16, C.matIn);
	},
	'*': (g, ox, oy, tx, ty) => {
		g.fillStyle = C.shadow;
		g.fillRect(ox + 8, oy + 26, 16, 4);
		rect(g, ox + 11, oy + 20, 10, 9, C.edge);
		rect(g, ox + 12, oy + 20, 8, 7, C.pot);
		rect(g, ox + 12, oy + 20, 2, 7, C.potD);
		rect(g, ox + 12, oy + 27, 8, 2, C.potD);
		// Fuzzy foliage: cloud-noise blob, not a hard circle.
		const ops = { rect: (x, y, w, h, c) => rect(g, ox + x, oy + y, w, h, c) };
		const seed = 4.2 + (tx % 3) * 1.3;
		cloudDisc(ops, 16, 12, 9.5, C.edge, seed, 3);
		cloudDisc(ops, 16, 12, 8, C.leaf, seed + 1.6, 2.6);
		cloudDisc(ops, 19, 15, 4.6, C.leafD2, seed + 3.1, 2);
		px(g, ox + 11, oy + 7, C.leafHi); px(g, ox + 14, oy + 5, C.leafHi);
		px(g, ox + 17, oy + 6, C.leafHi); px(g, ox + 20, oy + 11, C.leaf1);
	},
	// Lab equipment: the ported Laborbedarf-Generator. One char per variant
	// (server rack / analyzer / incubator) so a lab can place distinct machines.
	X: (g, ox, oy) => paintLabEquip(g, ox, oy, 0),
	Q: (g, ox, oy) => paintLabEquip(g, ox, oy, 1),
	Z: (g, ox, oy) => paintLabEquip(g, ox, oy, 2)
};

/** Grass fringe intruding onto path tiles (soft edges like the reference).
 * edges = {n,s,e,w} of neighboring grass/flowers/trees. */
export function paintFringe(g, ox, oy, edges) {
	const blade = (x, y) => {
		px(g, ox + x, oy + y, C.tuft);
		px(g, ox + x, oy + y + 1, C.grassDark);
	};
	if (edges.n) for (let x = 1; x < 32; x += 3) blade(x + (x % 2), 1 + (x % 2));
	if (edges.s) for (let x = 1; x < 32; x += 3) blade(x, 29 - (x % 2));
	if (edges.w) for (let y = 1; y < 32; y += 3) blade(1 + (y % 2), y);
	if (edges.e) for (let y = 1; y < 32; y += 3) blade(29 - (y % 2), y);
}

/** Foam strips where water meets land. edges = {n,s,e,w}. */
export function paintFoam(g, ox, oy, edges) {
	g.fillStyle = C.foam;
	if (edges.n) {
		g.fillRect(ox, oy, 32, 4);
		for (let x = 0; x < 32; x += 8) g.fillRect(ox + x, oy + 4, 4, 2);
	}
	if (edges.s) {
		g.fillRect(ox, oy + 28, 32, 4);
		for (let x = 4; x < 32; x += 8) g.fillRect(ox + x, oy + 26, 4, 2);
	}
	if (edges.w) {
		g.fillRect(ox, oy, 4, 32);
		for (let y = 0; y < 32; y += 8) g.fillRect(ox + 4, oy + y, 2, 4);
	}
	if (edges.e) {
		g.fillRect(ox + 28, oy, 4, 32);
		for (let y = 4; y < 32; y += 8) g.fillRect(ox + 26, oy + y, 2, 4);
	}
}

/** Map house roof colour -> generator style config (see housegen.js). */
const HOUSE_STYLES = {
	red: { paletteIndex: 0, wallStyle: 'wood', roofType: 'gabled' },
	blue: { paletteIndex: 1, wallStyle: 'siding', roofType: 'gabled' },
	gray: { paletteIndex: 5, wallStyle: 'brick', roofType: 'gabled' }
};

/** Height of a house sprite canvas in px (4 tiles + shadow overhang). */
export const HOUSE_H = 136;

/**
 * Whole house facade (like the reference: object-painted, not tiled).
 * w tiles wide; draws into a (w*32) x HOUSE_H canvas via housegen.js.
 * `roof` (red|blue|gray) selects palette + wall/roof style; `floors` 2 draws
 * a taller two-storey facade (chimney + porch gable). The generator centres
 * the door, matching the map's middle `d` tile for odd widths.
 * Transparent roof corners let the caller's grass background show through.
 */
export function paintHouse(g, ox, oy, w, roof, doorTx, floors = 1) {
	const cfg = HOUSE_STYLES[roof] ?? HOUSE_STYLES.red;
	const two = floors === 2;
	g.save();
	if (ox || oy) g.translate(ox, oy);
	drawHouse(g, w * 32, HOUSE_H, {
		...cfg,
		seed: 1337 + w * 31 + doorTx * 7,
		floors,
		wallHeight: two ? 80 : 64,
		roofHeight: two ? 48 : 64,
		showChimney: two,
		porchRoof: two ? 'small-gable' : null,
		scale: 3,
		showGroundShadow: true
	});
	g.restore();
}

/** Canvas height per building style in px (footprint rows × 32). */
export const BUILDING_H = { house: HOUSE_H, lab: 128, ash: 128, gary: 128 };

/**
 * Paint one building facade by style into a (w*32) x h*32 canvas.
 * styles: 'house' (housegen), 'lab' (cyber lab), 'ash', 'gary'.
 */
export function paintBuilding(g, w, h, p) {
	const style = p.style ?? 'house';
	g.save();
	if (style === 'lab') drawCyberLab(g, w * 32, h * 32, { ticks: 0 });
	else if (style === 'ash') drawAshHouse(g, w * 32, h * 32);
	else if (style === 'gary') drawGaryHouse(g, w * 32, h * 32);
	else paintHouse(g, 0, 0, w, p.roof, p.door, p.floors);
	g.restore();
}

/**
 * Whole flower-field region — one continuous patch instead of per-tile tiles,
 * so the blooms and tufts cross tile borders with no visible seams. Painted
 * transparent over the caller's grass background.
 */
export function paintField(g, w, h, seed, sway = 0, swayPhase = 0) {
	drawFlowerField(g, w, h, {
		seed,
		palette: GAME_FIELD_PAL,
		density: 26,
		noiseScale: 0.06,
		grassDensity: 45,
		strokeOffset: 1,
		bloomRadius: 2,
		flowerType: 'mixed',
		transparent: true,
		skipClear: true,
		showDirt: false,
		showStems: true,
		sway,
		swayPhase
	});
}

// ------------------------------------------------------------- build

/** Prerender all tiles. Returns {tiles: {ch: [canvas...]}}. Sprites come
 * from the character generator (js/chargen.js). */
export function buildArt() {
	const tiles = {};
	for (const [ch, p] of Object.entries(painters)) {
		// Run-aware layers bake 4 variants (left/middle/right/single), selected
		// at draw time from the neighbours (cells.runPos).
		if (RUN_LAYERS.has(ch)) {
			tiles[ch] = [0, 1, 2, 3].map((pos) => {
				const c = document.createElement('canvas');
				c.width = 32; c.height = 32;
				p(c.getContext('2d'), 0, 0, 0, 0, pos);
				return c;
			});
			continue;
		}
		const fns = Array.isArray(p) ? p : [p];
		tiles[ch] = fns.map((fn) => {
			const c = document.createElement('canvas');
			c.width = 32; c.height = 32;
			fn(c.getContext('2d'), 0, 0, 0, 0);
			return c;
		});
	}
	// Grass/tree/flowers/bushes/field get per-tile scatter: bake 4 variants.
	// Trees additionally mirror every second variant (free canopy variety).
	for (const ch of [',', 'T', 'F', '.', ':', 'b', 'V']) {
		const list = [];
		for (let v = 0; v < 4; v++) {
			const c = document.createElement('canvas');
			c.width = 32; c.height = 32;
			const fn = Array.isArray(painters[ch]) ? painters[ch][0] : painters[ch];
			// re-run painter with variant coords so scatter differs
			fn(c.getContext('2d'), 0, 0, v * 7 + 1, v * 13 + 2);
			if (ch === 'T' && v % 2 === 1) {
				const f = document.createElement('canvas');
				f.width = 32; f.height = 32;
				const fg = f.getContext('2d');
				fg.translate(32, 0);
				fg.scale(-1, 1);
				fg.drawImage(c, 0, 0);
				list.push(f);
			} else {
				list.push(c);
			}
		}
		tiles[ch] = list;
	}
	// Small trees: baked from smalltree.js, one canvas per variant.
	tiles.t = SMALLTREE_VARIANTS.map((cfg) => {
		const c = document.createElement('canvas');
		c.width = 32;
		c.height = 32;
		drawSmallTree(c.getContext('2d'), 32, 32, cfg);
		return c;
	});
	const sprites = {};
	return { tiles, sprites };
}

/** Pick tile variant (stable per map cell; water/grass animate, river flows). */
export function tileFrame(ch, tx, ty, frame) {
	if (ch === '~' || ch === 'G' || ch === 'W') return Math.floor(frame / 30) % 2;
	if (ch === 'r') return Math.floor(frame / 15) % 8;
	if (ch === 't') return Math.floor(hash(tx, ty, 97) * 6) % 6;
	if (',TF.:bV'.includes(ch)) return Math.floor(hash(tx, ty, 99) * 4) % 4;
	return 0;
}
