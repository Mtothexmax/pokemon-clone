/**
 * main.js — Svelte entry: startGame(canvas) builds art and runs the loop.
 *
 * The game itself stays framework-free; Svelte only hosts the canvas
 * and HUD. Call from onMount, hooks.onMap fires on map switches.
 */
import { TILE, keys, pressed, bindInput, endFrame, fitScreen } from './core.js';
import { buildArt, paintBuilding, BUILDING_H, paintField, paintFoam, paintFringe } from './tiles.js';
import { buildActors } from './chargen.js';
import { drawBigTree } from './bigtree.js';
import { drawSmallTree } from './smalltree.js';
import { drawSunflowerPatch } from './sunflower.js';
import { MAPS } from './maps.js';
import { createGame, update, drawGame, gameCam, describeAt, enterMap } from './game.js';

export function startGame(canvas, hooks = {}) {
	const g = canvas.getContext('2d');
	g.imageSmoothingEnabled = false;

	const art = buildArt();
	art.actors = buildActors();
	// Whole-object houses / buildings + foam/fringe edge pieces (bitmask
	// n=1,s=2,e=4,w=8). Building style selects the facade generator.
	art.houses = {};
	for (const [id, map] of Object.entries(MAPS)) {
		map.clearTiles = new Set();
		art.houses[id] = (map.props ?? [])
			.filter((p) => p.type === 'house')
			.map((p) => {
				const style = p.style ?? 'house';
				const h = BUILDING_H[style] ?? BUILDING_H.house;
				const rows = Math.round(h / 32);
				const c = document.createElement('canvas');
				c.width = p.w * 32;
				c.height = h;
				const hg = c.getContext('2d');
				// Draw the facade on transparency first, so we can find the
				// footprint tiles it leaves empty. Grass shows through those, so
				// they should stay walkable rather than act as invisible walls.
				paintBuilding(hg, p.w, rows, p);
				try {
					const data = hg.getImageData(0, 0, c.width, c.height).data;
					if (data.length >= c.width * c.height * 4) {
						for (let ty = 0; ty < rows; ty++) {
							for (let tx = 0; tx < p.w; tx++) {
								let opaque = false;
								for (let yy = ty * 32; yy < (ty + 1) * 32 && !opaque; yy++) {
									for (let xx = tx * 32; xx < (tx + 1) * 32; xx++) {
										if (data[(yy * c.width + xx) * 4 + 3] > 0) {
											opaque = true;
											break;
										}
									}
								}
								if (!opaque) map.clearTiles.add(`${p.x + tx},${p.y + ty}`);
							}
						}
					}
				} catch {
					// No pixel access (e.g. headless canvas stub): leave all solid.
				}
				// Covered tiles are skipped by the tile pass, so transparent edges
				// would show the cleared canvas. Paint grass *behind* the facade.
				const grass = art.tiles[','];
				hg.globalCompositeOperation = 'destination-over';
				for (let ty = 0; ty < rows; ty++) {
					for (let tx = 0; tx < p.w; tx++) {
						hg.drawImage(grass[(tx * 3 + ty * 5) % grass.length], tx * 32, ty * 32);
					}
				}
				hg.globalCompositeOperation = 'source-over';
				return { x: p.x, y: p.y, w: p.w, h: rows, img: c };
			});
	}
	// Whole flower-field regions: one continuous patch over a grass base.
	art.fields = {};
	for (const [id, map] of Object.entries(MAPS)) {
		art.fields[id] = (map.props ?? [])
			.filter((p) => p.type === 'field')
			.map((p) => {
				const w = p.w * 32;
				const h = p.h * 32;
				const base = document.createElement('canvas');
				base.width = w;
				base.height = h;
				const bg = base.getContext('2d');
				const grass = art.tiles[','];
				for (let ty = 0; ty < p.h; ty++) {
					for (let tx = 0; tx < p.w; tx++) {
						bg.drawImage(grass[(tx * 3 + ty * 5) % grass.length], tx * 32, ty * 32);
					}
				}
				// Pre-render gentle wind-sway phases (slow, low-amplitude).
				const NF = 6;
				const frames = [];
				for (let k = 0; k < NF; k++) {
					const c = document.createElement('canvas');
					c.width = w;
					c.height = h;
					const fg = c.getContext('2d');
					fg.drawImage(base, 0, 0);
					if (p.flowers !== false) paintField(fg, w, h, p.seed ?? 1, 1, (k / NF) * Math.PI * 2);
					frames.push(c);
				}
				return { x: p.x, y: p.y, w: p.w, h: p.h, img: frames[0], frames };
			});
	}
	// Generated cloud-trees (big trees) as multi-tile sprites; slender
	// "tall small-trees" (`small: true`) reuse the smalltree generator.
	art.bigtrees = {};
	for (const [id, map] of Object.entries(MAPS)) {
		art.bigtrees[id] = (map.props ?? [])
			.filter((p) => p.type === 'bigtree')
			.map((p) => {
				const w = (p.w ?? 2) * 32;
				const h = (p.h ?? 3) * 32;
				const c = document.createElement('canvas');
				c.width = w;
				c.height = h;
				if (p.small) {
					drawSmallTree(c.getContext('2d'), w, h, {
						seed: p.seed,
						paletteIndex: p.pal ?? 0,
						trunkStyle: 'straight',
						trunkHeight: 28,
						foliageWidth: 16,
						cloudRadius: 5.2,
						strokeOffset: 1,
						showBranches: true,
						showGroundShadow: true,
						showFruits: false,
						fruitCount: 0
					});
					return { x: p.x, y: p.y, w: p.w ?? 1, h: p.h ?? 2, img: c };
				}
				drawBigTree(c.getContext('2d'), w, h, {
					seed: p.seed,
					paletteIndex: p.pal ?? 0,
					tiers: p.tiers,
					density: p.density,
					minRad: p.minRad,
					maxRad: p.maxRad,
					strokeOffset: p.strokeOffset,
					jitter: p.jitter,
					trunkWidth: p.trunkWidth,
					trunkHeight: p.trunkHeight,
					rootFlare: p.rootFlare,
					showShadow: p.showShadow,
					showTopLight: p.showTopLight
				});
				return { x: p.x, y: p.y, w: p.w ?? 2, h: p.h ?? 3, img: c };
			});
	}
	// Sunflowers: decorative Y-sorted props (transparent over grass).
	art.sunflowers = {};
	for (const [id, map] of Object.entries(MAPS)) {
		art.sunflowers[id] = (map.props ?? [])
			.filter((p) => p.type === 'sunflower')
			.map((p) => {
				const w = (p.w ?? 3) * 32;
				const h = (p.h ?? 2) * 32;
				const c = document.createElement('canvas');
				c.width = w;
				c.height = h;
				drawSunflowerPatch(c.getContext('2d'), p.w ?? 3, p.h ?? 2, {
					count: p.count ?? 3,
					seed: p.seed ?? 1,
					showGroundShadow: true
				});
				return { x: p.x, y: p.y, w: p.w ?? 3, h: p.h ?? 2, img: c };
			});
	}
	art.foam = [];
	art.fringe = [];
	for (let i = 1; i < 16; i++) {
		const c = document.createElement('canvas');
		c.width = 32;
		c.height = 32;
		paintFoam(c.getContext('2d'), 0, 0, { n: i & 1, s: i & 2, e: i & 4, w: i & 8 });
		art.foam[i] = c;
		const f = document.createElement('canvas');
		f.width = 32;
		f.height = 32;
		paintFringe(f.getContext('2d'), 0, 0, { n: i & 1, s: i & 2, e: i & 4, w: i & 8 });
		art.fringe[i] = f;
	}
	const game = createGame();
	// Fit the canvas to the window (capped to the current map), preserving the
	// aspect ratio so the art is never stretched. Re-fitted on resize / map change.
	let curMap = game.mapId;
	const mapPx = (id) => {
		const m = MAPS[id];
		return [m.grid[0].length * TILE, m.grid.length * TILE];
	};
	fitScreen(canvas, ...mapPx(curMap));
	let resizeT = 0;
	const onResize = () => {
		clearTimeout(resizeT);
		resizeT = setTimeout(() => fitScreen(canvas, ...mapPx(curMap)), 150);
	};
	window.addEventListener('resize', onResize);
	window.__game = game; // debug handle (CDP tours, tests)
	window.__keys = keys;
	window.__art = art; // debug handle (sprite/tile inspection)
	window.__enterMap = enterMap; // debug handle (jump to a map/coords)
	bindInput();

	// Debug inspect: hover highlights a tile, click copies its id string
	// (for bug reports). Toggle with the ^/` key, polled per frame below.
	const toTile = (e) => {
		const r = canvas.getBoundingClientRect();
		const sx = (e.clientX - r.left) * (canvas.width / Math.max(1, r.width));
		const sy = (e.clientY - r.top) * (canvas.height / Math.max(1, r.height));
		const [cx, cy] = gameCam(game);
		return { tx: Math.floor((sx + cx) / TILE), ty: Math.floor((sy + cy) / TILE) };
	};
	canvas.addEventListener('mousemove', (e) => {
		game.hover = toTile(e);
	});
	canvas.addEventListener('mouseleave', () => {
		game.hover = null;
	});
	canvas.addEventListener('click', (e) => {
		if (!game.debug) return;
		const s = describeAt(game, toTile(e).tx, toTile(e).ty);
		if (!s) return;
		game.toast = { text: 'COPIED ' + s.slice(0, 40), until: game.frame + 150 };
		try {
			navigator.clipboard?.writeText(s).catch(() => {});
		} catch {
			/* clipboard unavailable — toast still shows the id */
		}
		window.__debugCopy = s;
	});

	let last = performance.now();
	let acc = 0;
	let raf = 0;
	let lastMap = null;
	let lastDebug = false;

	function frame(now) {
		raf = requestAnimationFrame(frame);
		acc += Math.min(100, now - last);
		last = now;
		while (acc >= 1000 / 60) {
			acc -= 1000 / 60;
			update(game, keys, pressed);
			endFrame();
		}
		if (game.mapId !== lastMap) {
			lastMap = game.mapId;
			curMap = game.mapId;
			fitScreen(canvas, ...mapPx(curMap));
			hooks.onMap?.(MAPS[game.mapId]);
		}
		if (game.debug !== lastDebug) {
			lastDebug = game.debug;
			hooks.onDebug?.(game.debug);
		}
		drawGame(g, art, game);
	}
	raf = requestAnimationFrame(frame);

	return {
		dispose() {
			cancelAnimationFrame(raf);
			window.removeEventListener('resize', onResize);
		}
	};
}
