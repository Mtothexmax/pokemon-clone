/**
 * game.js — state machine, grid movement, camera, NPCs, dialogue, warps.
 *
 * No DOM at import time (rendering takes a ctx + prerendered art), so the
 * headless Node test can drive update() directly.
 */

import { MAPS, tileAt, isSolid, warpAt, npcAt, signAt, bigTreeAt } from './maps.js';
import { tileFrame } from './tiles.js';
import { normCell, runPos, RUN_POS } from './cells.js';
import { VW, VH, TILE, drawText, drawBox, wrap, blip } from './core.js';

export const DIRS = {
	up: [0, -1], down: [0, 1], left: [-1, 0], right: [1, 0]
};

export function createGame() {
	const game = {
		state: 'walk',
		mapId: 'pallet',
		mx: 11, my: 9, dir: 'down',
		sx: 0, sy: 0, prog: 0, moving: false,
		npcs: [],
		text: null, // {pages:[[l,l]], page, chars}
		frame: 0,
		// Debug inspect mode (^ key): hover highlights, click copies the id.
		debug: false,
		hover: null, // {tx, ty} tile under the mouse
		propGrid: new Map(), // "x,y" -> house prop
		toast: null // {text, until}
	};
	const sp = MAPS.pallet.spawn;
	enterMap(game, 'pallet', sp.x, sp.y, sp.dir);
	return game;
}

export function enterMap(game, mapId, x, y, dir) {
	const map = MAPS[mapId];
	game.mapId = mapId;
	game.mx = x; game.my = y; game.dir = dir;
	game.sx = game.sy = game.prog = 0;
	game.moving = false;
	game.npcs = map.npcs.map((n) => ({ ...n, dialog: [...n.dialog] }));
	game.text = null;
	// House/field footprints for drawing + debug lookup.
	game.covered = new Set();
	game.propGrid = new Map();
	for (const p of map.props ?? []) {
		if (p.type === 'house') {
			const ph = p.h ?? 4;
			for (let dx = 0; dx < p.w; dx++) {
				for (let dy = 0; dy < ph; dy++) {
					game.covered.add(`${p.x + dx},${p.y + dy}`);
					game.propGrid.set(`${p.x + dx},${p.y + dy}`, p);
				}
			}
		} else if (p.type === 'field') {
			for (let dx = 0; dx < p.w; dx++) {
				for (let dy = 0; dy < p.h; dy++) game.covered.add(`${p.x + dx},${p.y + dy}`);
			}
		}
	}
}

export function startText(game, lines) {
	const wrapped = [];
	for (const line of lines) wrapped.push(...wrap(line));
	const pages = [];
	for (let i = 0; i < wrapped.length; i += 2) pages.push(wrapped.slice(i, i + 2));
	game.text = { pages, page: 0, chars: 0 };
	game.state = 'text';
	try { blip(660, 0.05); } catch { /* headless */ }
}

/** Advance dialogue. Returns true while text continues. */
export function advanceText(game) {
	const t = game.text;
	if (!t) return false;
	const page = t.pages[t.page];
	const full = page.join(' ').length;
	if (t.chars < full) {
		t.chars = full;
		return true;
	}
	t.page++;
	t.chars = 0;
	try { blip(520, 0.04); } catch { /* headless */ }
	if (t.page >= t.pages.length) {
		game.text = null;
		game.state = 'walk';
		return false;
	}
	return true;
}

function faceTile(game) {
	const [dx, dy] = DIRS[game.dir];
	return [game.mx + dx, game.my + dy];
}

export function update(game, keys, pressed) {
	game.frame++;
	if (game.state === 'text') {
		const t = game.text;
		const page = t.pages[t.page];
		const full = page.join(' ').length;
		if (t.chars < full) t.chars++;
		if (pressed.a || pressed.b) advanceText(game);
		return;
	}
	// walk
	if (pressed.debug) {
		game.debug = !game.debug;
		game.hover = null;
		try { blip(game.debug ? 880 : 440, 0.05); } catch { /* headless */ }
	}
	if (game.moving) {
		game.prog += keys.b ? 8 : 4;
		if (game.prog >= TILE) {
			game.mx += game.sx;
			game.my += game.sy;
			game.sx = game.sy = game.prog = 0;
			game.moving = false;
			const w = warpAt(game.mapId, game.mx, game.my);
			if (w) {
				try { blip(330, 0.08); } catch { /* headless */ }
				enterMap(game, w.to, w.tx, w.ty, w.dir);
			}
		}
		return;
	}
	for (const d of ['up', 'down', 'left', 'right']) {
		if (keys[d]) {
			game.dir = d;
			const [dx, dy] = DIRS[d];
			const nx = game.mx + dx;
			const ny = game.my + dy;
			if (!isSolid(game.mapId, nx, ny, game.npcs)) {
				game.sx = dx; game.sy = dy; game.prog = 0;
				game.moving = true;
			}
			break;
		}
	}
	if (pressed.a) {
		const [fx, fy] = faceTile(game);
		let npc = npcAt(game.npcs, fx, fy);
		if (!npc && isSolid(game.mapId, fx, fy, game.npcs)) {
			// Talk across a counter/desk: look one tile past the obstacle.
			const [dx, dy] = DIRS[game.dir];
			npc = npcAt(game.npcs, fx + dx, fy + dy);
		}
		if (npc) {
			// NPC turns to face the player.
			npc.dir = game.dir === 'up' ? 'down' : game.dir === 'down' ? 'up' : game.dir === 'left' ? 'right' : 'left';
			startText(game, npc.dialog);
			return;
		}
		const sign = signAt(game.mapId, fx, fy);
		if (sign) {
			startText(game, sign.text);
			return;
		}
		const ft = tileAt(MAPS[game.mapId], fx, fy);
		if (ft === 'd' && !warpAt(game.mapId, fx, fy)) {
			startText(game, ['LOCKED...', 'MAYBE LATER.']);
		}
	}
}

// ------------------------------------------------------------- render

/** Camera top-left in px (also used for mouse->tile in debug mode). */
export function gameCam(game) {
	const map = MAPS[game.mapId];
	const mw = map.grid[0].length * TILE;
	const mh = map.grid.length * TILE;
	const px = (game.mx + (game.sx * game.prog) / TILE) * TILE + 8;
	const py = (game.my + (game.sy * game.prog) / TILE) * TILE + 8;
	return [
		Math.max(0, Math.min(mw - VW, Math.round(px - VW / 2))),
		Math.max(0, Math.min(mh - VH, Math.round(py - VH / 2)))
	];
}

export function drawGame(g, art, game) {
	g.fillStyle = '#000';
	g.fillRect(0, 0, VW, VH);
	const map = MAPS[game.mapId];
	const [cx, cy] = gameCam(game);
	const x0 = Math.floor(cx / TILE);
	const y0 = Math.floor(cy / TILE);
	for (let ty = y0; ty <= y0 + VH / TILE; ty++) {
		for (let tx = x0; tx <= x0 + VW / TILE; tx++) {
			if (game.covered.has(tx + ',' + ty)) continue;
			const cell = normCell(map, tx, ty);
			if (cell.ch === '#') continue; // outside the grid stays black void
			const dpx = tx * TILE - cx;
			const dpy = ty * TILE - cy;
			// Ground pass (two-pass render: ground, then transparent decor).
			const gframes = art.tiles[cell.ground] ?? art.tiles[','];
			const gf = gframes.length > 1 ? tileFrame(cell.ground, tx, ty, game.frame) : 0;
			g.drawImage(gframes[gf], dpx, dpy);
			// Decor layers, bottom -> top. Run-aware layers pick left/mid/right.
			if (cell.layers.length) {
				const runLayer = cell.layers.find((l) => l.run);
				const pos = runLayer ? runPos(map, tx, ty, runLayer.name) : 0;
				for (const l of cell.layers) {
					const lf = art.tiles[l.name];
					if (!lf) continue;
					const idx = l.run ? pos : lf.length > 1 ? tileFrame(l.name, tx, ty, game.frame) : 0;
					g.drawImage(lf[idx] ?? lf[0], dpx, dpy);
				}
			}
			if (cell.ch === '~' || cell.ch === 'r' || cell.ch === 'o') {
				let m = 0;
				const wet = (c) => c === '~' || c === 'o' || c === 'r';
				if (!wet(tileAt(map, tx, ty - 1))) m |= 1;
				if (!wet(tileAt(map, tx, ty + 1))) m |= 2;
				if (!wet(tileAt(map, tx + 1, ty))) m |= 4;
				if (!wet(tileAt(map, tx - 1, ty))) m |= 8;
				if (m) g.drawImage(art.foam[m], dpx, dpy);
			}
			if (cell.ch === '.') {
				let m = 0;
				const gr = (c) => c === ',' || c === 'F';
				if (gr(tileAt(map, tx, ty - 1))) m |= 1;
				if (gr(tileAt(map, tx, ty + 1))) m |= 2;
				if (gr(tileAt(map, tx + 1, ty))) m |= 4;
				if (gr(tileAt(map, tx - 1, ty))) m |= 8;
				if (m) g.drawImage(art.fringe[m], dpx, dpy);
			}
		}
	}
	// Whole flower-field regions (continuous ground patches, wind-sway frames).
	for (const f of art.fields?.[game.mapId] ?? []) {
		const frames = f.frames ?? [f.img];
		const img = frames[Math.floor(game.frame / 26) % frames.length];
		g.drawImage(img, f.x * TILE - cx, f.y * TILE - cy);
	}
	// Whole-object houses (facades with chimney, windows, door).
	for (const h of art.houses[game.mapId] ?? []) {
		g.drawImage(h.img, h.x * TILE - cx, h.y * TILE - cy);
	}
	// Y-sorted actors with ground shadows (outlined 36x36 frames).
	const bob = game.moving && Math.floor(game.frame / 8) % 2 === 0 ? -2 : 0;
	const stepFrame = game.moving ? Math.floor(game.frame / 8) % 2 : 0;
	const actors = game.npcs.map((n) => ({
		y: n.y,
		px: n.x * TILE - cx,
		py: n.y * TILE - cy,
		look: n.look,
		dir: n.dir
	}));
	{
		const ppx = (game.mx + (game.sx * game.prog) / TILE) * TILE - cx;
		const ppy = (game.my + (game.sy * game.prog) / TILE) * TILE - cy;
		actors.push({
			y: game.my + (game.sy * game.prog) / TILE,
			px: ppx, py: ppy, player: true,
			look: 'player',
			dir: game.dir
		});
	}
	// Big cloud-trees sort with actors by their base row (baked shadow).
	for (const t of art.bigtrees[game.mapId] ?? []) {
		actors.push({
			y: t.y + t.h - 1,
			px: t.x * TILE - cx,
			py: t.y * TILE - cy,
			tree: t
		});
	}
	// Sunflowers sort with actors by their base row (baked shadow).
	for (const s of art.sunflowers?.[game.mapId] ?? []) {
		actors.push({
			y: s.y + s.h - 1,
			px: s.x * TILE - cx,
			py: s.y * TILE - cy,
			tree: s
		});
	}
	for (const a of actors) if (!a.tree) g.drawImage(art.actors.shadow, a.px + 3, a.py + 22);
	actors.sort((a, b) => a.y - b.y);
	for (const a of actors) {
		if (a.tree) {
			g.drawImage(a.tree.img, a.px, a.py);
			continue;
		}
		const key = a.dir === 'left' || a.dir === 'right' ? 'side' : a.dir;
		const frames = art.actors[a.look][key];
		const img = frames[a.player ? stepFrame : 0] ?? frames[0];
		const oy = a.py - 1 + (a.player ? bob : 0);
		if (a.dir === 'left') {
			g.save();
			g.translate(a.px - 1 + 34, oy);
			g.scale(-1, 1);
			g.drawImage(img, 0, 0);
			g.restore();
		} else {
			g.drawImage(img, a.px - 1, oy);
		}
	} // end actors

	// Dialogue box.
	if (game.state === 'text' && game.text) {
		const t = game.text;
		drawBox(g, 8, VH - 88, VW - 16, 80);
		const page = t.pages[t.page];
		let shown = t.chars;
		page.forEach((line, i) => {
			const part = line.slice(0, Math.max(0, shown));
			drawText(g, part, 20, VH - 76 + i * 22, '#202020', 2);
			shown -= line.length;
		});
		if (t.chars >= page.join(' ').length && Math.floor(game.frame / 20) % 2 === 0) {
			drawText(g, 'v', VW - 30, VH - 30, '#202020', 2);
		}
	}

	// Debug inspect highlight + hovered coordinates.
	if (game.debug && game.hover) {
		const hx = game.hover.tx * TILE - cx;
		const hy = game.hover.ty * TILE - cy;
		g.strokeStyle = '#ffffff';
		g.lineWidth = 2;
		g.strokeRect(hx + 1, hy + 1, TILE - 2, TILE - 2);
		const label = `${game.hover.tx},${game.hover.ty}`;
		const wpx = label.length * 6 + 6;
		g.fillStyle = 'rgba(0,0,0,0.7)';
		g.fillRect(hx + 3, hy - 13, wpx, 12);
		drawText(g, label, hx + 6, hy - 11, '#fff');
	}

	// Copy toast (set by the shell after clipboard write).
	if (game.toast && game.frame < game.toast.until) {
		const msg = game.toast.text;
		const wpx = Math.min(VW - 16, msg.length * 6 + 14);
		const tx = Math.round((VW - wpx) / 2);
		g.fillStyle = 'rgba(0,0,0,0.75)';
		g.fillRect(tx, 8, wpx, 16);
		drawText(g, msg.slice(0, 44), tx + 7, 11, '#7fe890');
	} else {
		game.toast = null;
	}
}

/**
 * Describe what's at a tile for the debug inspector. Returns a
 * copy-pasteable id string (npc / house / sign / warp / tile).
 */
export function describeAt(game, tx, ty) {
	const map = MAPS[game.mapId];
	const npc = npcAt(game.npcs, tx, ty);
	if (npc) return `${game.mapId} npc ${npc.id} (${tx},${ty}) look=${npc.look ?? npc.sprite} dir=${npc.dir}`;
	const w = warpAt(game.mapId, tx, ty);
	if (w) return `${game.mapId} warp (${tx},${ty}) -> ${w.to} (${w.tx},${w.ty})`;
	const prop = game.propGrid.get(`${tx},${ty}`);
	if (prop) {
		const ph = prop.h ?? 4;
		const label = prop.style ? `style=${prop.style}` : `roof=${prop.roof}`;
		const door = prop.door ?? Math.floor((prop.w - 1) / 2);
		return `${game.mapId} house (${prop.x}-${prop.x + prop.w - 1},${prop.y}-${prop.y + ph - 1}) ${label} door=${prop.x + door}`;
	}
	const tree = bigTreeAt(game.mapId, tx, ty);
	if (tree) {
		const kind = tree.small ? 'smalltree-tall' : 'bigtree';
		return `${game.mapId} ${kind} (${tree.x}-${tree.x + (tree.w ?? 2) - 1},${tree.y}-${tree.y + (tree.h ?? 3) - 1}) seed=${tree.seed ?? 1337} pal=${tree.pal ?? 0}`;
	}
	const sign = signAt(game.mapId, tx, ty);
	if (sign) return `${game.mapId} sign (${tx},${ty}) "${String(sign.text[0] ?? '').slice(0, 40)}"`;
	const cell = normCell(map, tx, ty);
	if (cell.ch === '#') return `${game.mapId} void (${tx},${ty})`;
	if (cell.ch === 'r') return `${game.mapId} river (${tx},${ty})`;
	if (cell.ch === '~') return `${game.mapId} lake (${tx},${ty})`;
	const runLayer = cell.layers.find((l) => l.run);
	const pos = runLayer ? ` pos=${RUN_POS[runPos(map, tx, ty, runLayer.name)]}` : '';
	const layers = cell.layers.map((l) => `'${l.name}'`).join(', ');
	return `${game.mapId} tile (${tx},${ty}) ground='${cell.ground}' layers=[${layers}]${pos}`;
}

