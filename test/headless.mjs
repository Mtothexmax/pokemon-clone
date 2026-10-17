/**
 * headless.mjs — Node test for maps + game logic (no browser needed).
 *
 *   node test/headless.mjs
 */
import { MAPS, isSolid, warpAt, validateMaps, tileAt, bigTreeAt } from '../src/lib/maps.js';
import { createGame, enterMap, update, describeAt, gameCam } from '../src/lib/game.js';
import { LOOKS, paintActor } from '../src/lib/chargen.js';
import { TEX } from '../src/lib/texture.js';
import { ASH_PAL, ASH_DOWN, ASH_SIDE, stride, backView, buildAshFrames } from '../src/lib/ash.js';
import { TREE_PAL, TREE } from '../src/lib/tree.js';
import { tileFrame, GROUND, SMALLTREE_VARIANTS, LAB_EQUIP, BUILDING_H, paintBuilding } from '../src/lib/tiles.js';
import { normCell, runPos, RUN_POS } from '../src/lib/cells.js';
import { drawFlowerField, GAME_FIELD_PAL } from '../src/lib/flowerfield.js';
import { drawSmallTree, SMALLTREE_PALETTES } from '../src/lib/smalltree.js';
import { drawHouse, HOUSE_PALETTES } from '../src/lib/housegen.js';
import { drawLabEquipment, LAB_PALETTES } from '../src/lib/labgen.js';
import { drawRiver, RIVER_PALETTES } from '../src/lib/rivergen.js';
import { hash2, vnoise, fbm, inDisc } from '../src/lib/noise.js';
import { cloudRing, cloudDisc, edgeOutline } from '../src/lib/outline.js';
import { drawBigTree, drawPixelCircle, createPRNG, BIG_PALETTES } from '../src/lib/bigtree.js';

let fails = 0;
function ok(name, cond, extra = '') {
	console.log(`${cond ? '  ok  ' : ' FAIL '} ${name}${extra ? ' — ' + extra : ''}`);
	if (!cond) fails++;
}

const K = (over = {}) => ({ up: false, down: false, left: false, right: false, a: false, b: false, start: false, ...over });
const P = (over = {}) => ({ up: false, down: false, left: false, right: false, a: false, b: false, start: false, ...over });

// 0. maps valid
ok('maps valid', validateMaps().length === 0, validateMaps().join('; '));

// 1. game starts directly in walk (no title screen)
{
	const g = createGame();
	ok('starts in walk', g.state === 'walk' && g.mapId === 'pallet', g.state);
	ok('spawns at pallet spawn', g.mx === MAPS.pallet.spawn.x && g.my === MAPS.pallet.spawn.y, `${g.mx},${g.my}`);
}

// 2. walls block, paths pass
{
	const g = createGame();
	enterMap(g, 'pallet', 11, 9, 'down');
	g.state = 'walk';
	update(g, K({ up: true }), P()); // (11,9)->(11,8)? path row 9 is path; north of it row 8 x=11 is '.' path
	ok('path walkable', g.moving, `${g.mx},${g.my}`);
	for (let i = 0; i < 10; i++) update(g, K(), P());
	const g2 = createGame();
	enterMap(g2, 'pallet', 1, 1, 'left');
	g2.state = 'walk';
	update(g2, K({ left: true }), P()); // into border tree
	ok('tree blocks', !g2.moving && g2.mx === 1, `${g2.mx},${g2.my}`);
}

// 3. water blocks (pond shore)
{
	const g = createGame();
	enterMap(g, 'pallet', 23, 28, 'down');
	g.state = 'walk';
	update(g, K({ down: true }), P()); // (23,29) is '~'
	ok('water blocks', !g.moving, `${g.mx},${g.my}`);
}

// 3b. bridge crosses the river, wheat walkable, plant blocks
{
	const g = createGame();
	enterMap(g, 'pallet', 21, 16, 'right');
	g.state = 'walk';
	update(g, K({ right: true }), P());
	for (let i = 0; i < 80; i++) update(g, K({ right: true }), P());
	ok('bridge walkable over water', g.my === 16 && g.mx >= 26, `${g.mx},${g.my}`);
	const h = createGame();
	enterMap(h, 'pallet', 1, 27, 'right');
	h.state = 'walk';
	update(h, K({ right: true }), P()); // (2,27) is wheat, walkable now
	for (let i = 0; i < 10; i++) update(h, K(), P());
	ok('wheat walkable', h.mx === 2 && h.my === 27, `${h.mx},${h.my}`);
	const p = createGame();
	enterMap(p, 'lab', 10, 8, 'up');
	p.state = 'walk';
	update(p, K({ up: true }), P()); // (10,7) is a plant
	ok('plant blocks', !p.moving, `${p.mx},${p.my}`);
}

// 4. lab door warps inside (bigger inside!)
{
	const g = createGame();
	enterMap(g, 'pallet', 8, 8, 'up');
	g.state = 'walk';
	update(g, K({ up: true }), P()); // step onto (8,7) door
	for (let i = 0; i < 10; i++) update(g, K(), P());
	ok('lab door warps inside', g.mapId === 'lab' && g.mx === 7 && g.my === 9, `${g.mapId} ${g.mx},${g.my}`);
	ok('lab is bigger inside', MAPS.lab.grid.length * MAPS.lab.grid[0].length > 100, `${MAPS.lab.grid[0].length}x${MAPS.lab.grid.length}`);
}

// 4b. locked house door answers, farmer talks
{
	const g = createGame();
	enterMap(g, 'pallet', 33, 12, 'up');
	g.state = 'walk';
	update(g, K(), P({ a: true })); // face (33,11) locked door (Ash's)
	ok('locked door answers', g.state === 'text', g.state);
	while (g.state === 'text') update(g, K(), P({ a: true }));
	const h = createGame();
	enterMap(h, 'pallet', 1, 27, 'right');
	h.state = 'walk';
	update(h, K({ right: true }), P()); // (2,27) is wheat, walkable now
	for (let i = 0; i < 10; i++) update(h, K(), P());
	ok('wheat walkable', h.mx === 2 && h.my === 27, `${h.mx},${h.my}`);
	const p = createGame();
	enterMap(p, 'lab', 10, 8, 'up');
	p.state = 'walk';
	update(p, K({ up: true }), P()); // (10,7) is a plant
	ok('plant blocks', !p.moving, `${p.mx},${p.my}`);
}

// 3c. river flows (4 frames), lake shimmers (2), tall grass + meadow sway
{
	ok('river cycles 0..3', [0, 15, 30, 45].map((f) => tileFrame('r', 24, 13, f)).join() === '0,1,2,3');
	ok('lake cycles 0..1', [0, 30, 60].map((f) => tileFrame('~', 23, 30, f)).join() === '0,1,0');
	ok('tall grass sways', [0, 30].map((f) => tileFrame('G', 29, 0, f)).join() === '0,1');
	ok('meadow sways', [0, 30].map((f) => tileFrame('W', 24, 34, f)).join() === '0,1');
	ok('river water at (24,13)', MAPS.pallet.grid[13][24] === 'r');
	ok('bridge ground is grass', GROUND['+'] === ',' && GROUND.o === '~');
	const g = createGame();
	enterMap(g, 'pallet', 29, 1, 'up');
	g.state = 'walk';
	update(g, K({ up: true }), P()); // step onto tall grass (29,0)
	for (let i = 0; i < 10; i++) update(g, K(), P());
	ok('tall grass walkable', g.mx === 29 && g.my === 0, `${g.mx},${g.my}`);
	ok('river blocks wading', isSolid('pallet', 24, 13, []));
	ok('river described', describeAt(g, 24, 13) === 'pallet river (24,13)');
	ok('lake described', describeAt(g, 23, 30) === 'pallet lake (23,30)');
	ok('meadow walkable', !isSolid('pallet', 24, 34, []));
	ok('door leaves walkable', !isSolid('lab', 7, 11, []) && !isSolid('lab', 8, 11, []));
	// Smart cells: one desk/door char, auto left/middle/right via runPos.
	{
		const gl = createGame();
		enterMap(gl, 'lab', 8, 8, 'up');
		ok('door run left/right', RUN_POS[runPos(MAPS.lab, 7, 11, 'd')] === 'left' && RUN_POS[runPos(MAPS.lab, 8, 11, 'd')] === 'right');
		ok('desk is 3 layers', describeAt(gl, 7, 5) === "lab tile (7,5) ground='L' layers=['desk', 'computer'] pos=middle", describeAt(gl, 7, 5));
		ok('desk left end', describeAt(gl, 6, 5).endsWith('pos=left'), describeAt(gl, 6, 5));
		ok('desk right end', describeAt(gl, 9, 5).endsWith('pos=right'), describeAt(gl, 9, 5));
		ok('desk solid', isSolid('lab', 8, 5, []));
		const g = createGame();
		enterMap(g, 'lab', 8, 6, 'up');
		g.state = 'walk';
		update(g, K({ up: true }), P()); // desk blocks
		ok('desk blocks', !g.moving && g.mx === 8 && g.my === 6, `${g.mx},${g.my}`);
		const h = createGame();
		enterMap(h, 'lab', 8, 6, 'up');
		h.state = 'walk';
		update(h, K(), P({ a: true })); // talk across the desk to Oak
		ok('talk across desk', h.state === 'text', h.state);
	}
}

// 3d. flower field (V): right farm block is flowers, left stays wheat, walkable
{
	ok('right block is flower field', MAPS.pallet.grid[25][37] === 'V' && MAPS.pallet.grid[32][45] === 'V');
	ok('left block stays wheat', MAPS.pallet.grid[27][2] === 'Y' && MAPS.pallet.grid[32][11] === 'Y');
	ok(
		'flower field has scatter variants',
		[0, 1, 2, 3].includes(tileFrame('V', 40, 28, 0)) && tileFrame('V', 40, 28, 0) === tileFrame('V', 40, 28, 99)
	);
	ok('flower field walkable', !isSolid('pallet', 40, 28, []));
	ok('flower field is transparent over grass', GROUND.V === ',' && normCell(MAPS.pallet, 40, 28).ground === ',');
	ok('meadow has no blooms', MAPS.pallet.grid[34][23] === 'W' && GROUND.W === ',');
	const g = createGame();
	enterMap(g, 'pallet', 40, 24, 'down');
	g.state = 'walk';
	update(g, K({ down: true }), P()); // step onto (40,25) flower field
	for (let i = 0; i < 10; i++) update(g, K(), P());
	ok('flower field enterable', g.mx === 40 && g.my === 25, `${g.mx},${g.my}`);
	// Ported Blumenfeld-Generator: full cover, deterministic, game palette.
	{
		const painted = [];
		const ops = {
			fillStyle: '',
			imageSmoothingEnabled: false,
			clearRect() {},
			fillRect(x, y, w, h) {
				painted.push([x, y, w, h, this.fillStyle]);
			}
		};
		drawFlowerField(ops, 32, 32, {
			seed: 4242,
			palette: GAME_FIELD_PAL,
			density: 46,
			noiseScale: 0.08,
			grassDensity: 60,
			strokeOffset: 1,
			bloomRadius: 2,
			flowerType: 'mixed',
			showDirt: true,
			showStems: true
		});
		ok('flower field paints', painted.length > 900, `${painted.length} px`);
		ok('flower field full cover', painted.filter(([, , w, h]) => w === 1 && h === 1).length >= 1024);
		ok(
			'flower field blooms use palette',
			painted.some(([, , , , c]) => GAME_FIELD_PAL.blooms.includes(c))
		);
		const a = [];
		const b = [];
		const mk = (sink) => ({ fillStyle: '', clearRect() {}, fillRect: () => sink.push(1) });
		drawFlowerField(mk(a), 32, 32, { seed: 7, palette: GAME_FIELD_PAL });
		drawFlowerField(mk(b), 32, 32, { seed: 7, palette: GAME_FIELD_PAL });
		ok('flower field deterministic', a.length === b.length && a.length > 0, `${a.length}`);
	}
}

// 3e. big cloud-trees: placed on grass, base row blocks, Y-sorted sprite
{
	ok('bigtree prop found', bigTreeAt('pallet', 12, 2)?.type === 'bigtree');
	ok('bigtree covers base', bigTreeAt('pallet', 13, 4)?.type === 'bigtree');
	ok('bigtree base row solid', isSolid('pallet', 12, 4, []) && isSolid('pallet', 13, 4, []));
	ok('bigtree canopy walkable', !isSolid('pallet', 12, 2, []));
	ok('bigtree described', describeAt(createGame(), 12, 2) === 'pallet bigtree (12-13,2-4) seed=1337 pal=0', describeAt(createGame(), 12, 2));
	// every large bigtree sits on plain grass (the slender tall small-tree
	// may overhang a path tile) and all stay in bounds
	const errs = [];
	for (const p of MAPS.pallet.props.filter((p) => p.type === 'bigtree')) {
		const allowed = p.small ? ',.' : ',';
		for (let dx = 0; dx < (p.w ?? 2); dx++) {
			for (let dy = 0; dy < (p.h ?? 3); dy++) {
				const ch = MAPS.pallet.grid[p.y + dy]?.[p.x + dx];
				if (!allowed.includes(ch)) errs.push(`(${p.x + dx},${p.y + dy})='${ch}'`);
			}
		}
	}
	ok('bigtrees on grass', errs.length === 0, errs.join('; '));
	// Tall small-tree: 1x2 prop, base row solid, upper canopy walkable, described.
	ok('tall small-tree prop', MAPS.pallet.props.some((p) => p.type === 'bigtree' && p.small && p.x === 41 && p.y === 16));
	ok('tall small-tree base solid', isSolid('pallet', 41, 17, []) && !isSolid('pallet', 41, 16, []));
	ok('tall small-tree described', describeAt(createGame(), 41, 17) === 'pallet smalltree-tall (41-41,16-17) seed=3313 pal=0', describeAt(createGame(), 41, 17));
	// generator: deterministic PRNG, palette range, and it paints via a stub
	const r1 = createPRNG(99);
	const r2 = createPRNG(99);
	ok('bigtree prng deterministic', r1() === r2() && r1() === r2());
	ok('bigtree palettes', BIG_PALETTES.length === 6 && BIG_PALETTES.every((p) => p.outline && p.trunk && p.topGreen));
	const painted = [];
	const ops = {
		fillStyle: '',
		imageSmoothingEnabled: false,
		clearRect() {},
		fillRect(x, y, w, h) {
			painted.push([x, y, w, h, this.fillStyle]);
		}
	};
	drawBigTree(ops, 64, 96, { seed: 5, paletteIndex: 1 });
	ok('bigtree paints', painted.length > 2000, `${painted.length} px`);
	ok('bigtree has transparent shadow', painted.some(([, , , , c]) => /^rgba\(/.test(c)), 'rgba shadow');
	ok('bigtree stays near sprite', painted.every(([x, y]) => x >= -4 && x < 68 && y >= -4 && y < 100));
	const disc = [];
	drawPixelCircle({ fillStyle: '', fillRect: (x, y) => disc.push([x, y]) }, 10, 10, 3, '#fff');
	ok('drawPixelCircle filled', disc.length > 20, `${disc.length} px`);
}

// 3f. small trees (t): placed on grass, solid, 6 variants, one fruiting
{
	const spots = [[2, 4], [18, 6], [20, 7], [3, 13], [40, 13], [36, 17], [30, 18], [44, 20], [6, 14]];
	ok(
		'small trees planted',
		spots.every(([x, y]) => MAPS.pallet.grid[y][x] === 't'),
		spots.filter(([x, y]) => MAPS.pallet.grid[y][x] !== 't').map(([x, y]) => `${x},${y}`).join(';')
	);
	ok('small tree solid', isSolid('pallet', 2, 4, []));
	ok('small tree ground is grass', GROUND.t === ',' && normCell(MAPS.pallet, 2, 4).ground === ',');
	ok('small tree is a layer', normCell(MAPS.pallet, 2, 4).layers.some((l) => l.name === 't'));
	const g = createGame();
	enterMap(g, 'pallet', 1, 4, 'right');
	g.state = 'walk';
	update(g, K({ right: true }), P());
	ok('small tree blocks', !g.moving, `${g.mx},${g.my}`);
	ok('small tree picks a variant 0..5', [0, 1, 2, 3, 4, 5].includes(tileFrame('t', 2, 4, 0)));
	ok('exactly one fruiting small tree', SMALLTREE_VARIANTS.filter((v) => v.showFruits).length === 1);
	ok('smalltree palettes', SMALLTREE_PALETTES.length === 6 && SMALLTREE_PALETTES.every((p) => p.foliageMid && p.trunkMid));
	const painted = [];
	const ops = { imageSmoothingEnabled: true, clearRect() {}, fillStyle: '', fillRect(x, y, w, h) { painted.push([x, y, w, h, this.fillStyle]); } };
	drawSmallTree(ops, 32, 32, SMALLTREE_VARIANTS[1]);
	ok('smalltree paints', painted.length > 150, `${painted.length} px`);
	ok('smalltree stays in tile', painted.every(([x, y]) => x >= -4 && x < 36 && y >= -2 && y < 36));
	const mk = (sink) => ({ imageSmoothingEnabled: true, clearRect() {}, fillStyle: '', fillRect: () => sink.push(1) });
	const a = [];
	const b = [];
	drawSmallTree(mk(a), 32, 32, SMALLTREE_VARIANTS[0]);
	drawSmallTree(mk(b), 32, 32, SMALLTREE_VARIANTS[0]);
	ok('smalltree deterministic', a.length === b.length && a.length > 0, `${a.length}`);
}

// 3g. house generator: palettes, geometry in bounds, deterministic
{
	ok('house palettes', HOUSE_PALETTES.length >= 5 && HOUSE_PALETTES.every((p) => p.roofMid && p.wallMid && p.outline));
	const painted = [];
	const ops = {
		imageSmoothingEnabled: true,
		fillStyle: '',
		fillRect(x, y, w, h) {
			if (!Number.isFinite(x + y + w + h)) throw new Error(`non-finite ${x},${y},${w},${h}`);
			painted.push([x, y, w, h, this.fillStyle]);
		}
	};
	drawHouse(ops, 7 * 32, 136, { paletteIndex: 0, wallHeight: 64, roofHeight: 64, scale: 3, seed: 1 });
	ok('house paints', painted.length > 300, `${painted.length} px`);
	ok(
		'house in bounds',
		painted.every(([x, y, w, h]) => x >= -4 && x <= 7 * 32 + 4 && y >= -2 && y <= 140 && w >= 0 && h >= 0)
	);
	ok(
		'house has roof + wall + door colors',
		painted.some(([, , , , c]) => c === HOUSE_PALETTES[0].roofMid) &&
			painted.some(([, , , , c]) => c === HOUSE_PALETTES[0].wallMid) &&
			painted.some(([, , , , c]) => c === HOUSE_PALETTES[0].door)
	);
	const mk = (sink) => ({ imageSmoothingEnabled: true, fillStyle: '', fillRect: (x, y, w, h) => sink.push([x, y, w, h]) });
	const a = [];
	const b = [];
	drawHouse(mk(a), 5 * 32, 136, { paletteIndex: 1, seed: 5 });
	drawHouse(mk(b), 5 * 32, 136, { paletteIndex: 1, seed: 5 });
	ok('house deterministic', JSON.stringify(a) === JSON.stringify(b), `${a.length}`);
}

// 3h. lab equipment (X/Q/Z): replaces the old lab plants, solid, 3 variants
{
	ok('lab plants replaced', MAPS.lab.grid[3][4] === 'X' && MAPS.lab.grid[7][7] === 'Z' && MAPS.lab.grid[7][10] === 'Q');
	ok('lab has no * plants', MAPS.lab.grid.every((row) => [...row].every((ch) => ch !== '*')));
	let xcount = 0;
	for (const row of MAPS.lab.grid) for (const ch of row) if ('XQZ'.includes(ch)) xcount++;
	ok('lab has 3 equipment tiles', xcount === 3, `${xcount}`);
	ok('lab equipment solid', isSolid('lab', 4, 3, []) && isSolid('lab', 7, 7, []) && isSolid('lab', 10, 7, []));
	ok('lab equipment ground is floor', GROUND.X === 'L' && GROUND.Q === 'L' && GROUND.Z === 'L' && normCell(MAPS.lab, 4, 3).ground === 'L');
	ok('lab equipment is a layer', normCell(MAPS.lab, 4, 3).layers.some((l) => l.name === 'X'));
	ok('lab equipment variants defined', LAB_EQUIP.length === 3 && LAB_EQUIP.every((e) => e.itemType && e.itemWidth && e.itemHeight));
	ok('lab palettes', LAB_PALETTES.length >= 3 && LAB_PALETTES.every((p) => p.metalMid && p.screenGlow && p.outline));
	const painted = [];
	const ops = { imageSmoothingEnabled: true, clearRect() {}, fillStyle: '', fillRect(x, y, w, h) { painted.push([x, y, w, h, this.fillStyle]); } };
	drawLabEquipment(ops, 32, 32, { ...LAB_EQUIP[0], seed: 11, paletteIndex: 0, showGroundShadow: true, showScreenGlow: true });
	const area = painted.reduce((s, [, , w, h]) => s + w * h, 0);
	ok('lab equipment paints', painted.length > 8 && area > 150, `${painted.length} rects / ${area} px`);
	ok('lab equipment in tile', painted.every(([x, y, w, h]) => x >= -6 && x + w <= 38 && y >= -2 && y + h <= 34));
	const mk2 = (sink) => ({ imageSmoothingEnabled: true, clearRect() {}, fillStyle: '', fillRect: () => sink.push(1) });
	const ea = [];
	const eb = [];
	drawLabEquipment(mk2(ea), 32, 32, { ...LAB_EQUIP[1], seed: 5 });
	drawLabEquipment(mk2(eb), 32, 32, { ...LAB_EQUIP[1], seed: 5 });
	ok('lab equipment deterministic', ea.length === eb.length && ea.length > 0, `${ea.length}`);
}

// 3i. river is transparent water over grass (meadow shows through)
{
	ok('river ground is grass', GROUND.r === ',' && normCell(MAPS.pallet, 24, 13).ground === ',');
	ok('river is a layer', normCell(MAPS.pallet, 24, 13).layers.some((l) => l.name === 'r'));
	ok('river palettes', RIVER_PALETTES.length >= 3 && RIVER_PALETTES.every((p) => p.deep && p.mid && p.light && p.foam));
	const painted = [];
	const ops = { imageSmoothingEnabled: true, clearRect() {}, globalAlpha: 1, fillStyle: '', fillRect(x, y, w, h) { painted.push([x, y, w, h, this.fillStyle]); } };
	drawRiver(ops, 32, 32, { paletteIndex: 0, direction: 'down', widthPct: 100, transparent: true, waterAlpha: 0.7, ticks: 0 });
	ok('river paints pixels', painted.length > 500, `${painted.length} px`);
	ok('river skips shore in transparent mode', painted.every(([, , , , c]) => c !== RIVER_PALETTES[0].shore));
	const opaque = [];
	drawRiver({ imageSmoothingEnabled: true, clearRect() {}, globalAlpha: 1, fillStyle: '', fillRect(x, y, w, h) { opaque.push([this.fillStyle, w, h]); } }, 32, 32, { paletteIndex: 0, ticks: 0 });
	ok('river shore fills when opaque', opaque.some(([c, w, h]) => w === 32 && h === 32 && c === RIVER_PALETTES[0].shore));
}

// 3j. potted plants (*) moved outside the lab, onto grass
{
	ok('plants moved outside', MAPS.pallet.grid[24][6] === '*' && MAPS.pallet.grid[24][8] === '*');
	ok('plant ground is grass', GROUND['*'] === ',' && normCell(MAPS.pallet, 6, 24).ground === ',');
	ok('plant solid', isSolid('pallet', 6, 24, []) && isSolid('pallet', 8, 24, []));
}

// 5. lab exit warps back out
{
	const g = createGame();
	enterMap(g, 'lab', 7, 10, 'down');
	g.state = 'walk';
	update(g, K({ down: true }), P()); // step onto (7,11) door
	for (let i = 0; i < 10; i++) update(g, K(), P());
	ok('lab exit warps out', g.mapId === 'pallet' && g.mx === 8 && g.my === 8, `${g.mx},${g.my}`);
}

// 5b. double door: both leaves warp, mat is 2 wide, plant gone
{
	ok('lab door is smart d', normCell(MAPS.lab, 7, 11).ch === 'd' && normCell(MAPS.lab, 8, 11).ch === 'd');
	const g = createGame();
	enterMap(g, 'lab', 8, 10, 'down');
	g.state = 'walk';
	update(g, K({ down: true }), P()); // step onto (8,11) right leaf
	for (let i = 0; i < 10; i++) update(g, K(), P());
	ok('right leaf warps out too', g.mapId === 'pallet' && g.mx === 8 && g.my === 8, `${g.mapId} ${g.mx},${g.my}`);
	let mats = 0;
	for (const row of MAPS.lab.grid) for (const ch of row) if (ch === 'M' || ch === 'N') mats++;
	ok('doormat is one 2-wide pair', mats === 2 && MAPS.lab.grid[10][7] === 'M' && MAPS.lab.grid[10][8] === 'N', `${mats} halves`);
	ok('upper mats removed', MAPS.lab.grid[9][7] === 'L' && MAPS.lab.grid[9][8] === 'L');
	ok('doormat walkable', !isSolid('lab', 7, 10, []) && !isSolid('lab', 8, 10, []));
	ok('plant (11,3) removed', MAPS.lab.grid[3][11] === 'L');
	ok('E walkable', !isSolid('lab', 8, 11, []));
}

// 5d. outside the grid is a black void, not a tree
{
	ok('void beyond lab grid', tileAt(MAPS.lab, 17, 6) === '#' && tileAt(MAPS.lab, -1, 3) === '#');
	ok('void is solid', isSolid('lab', 17, 6, []));
	const g = createGame();
	enterMap(g, 'lab', 7, 6, 'up');
	ok('void described', describeAt(g, 17, 6) === 'lab void (17,6)', describeAt(g, 17, 6));
	ok('lab equipment reads as tile', describeAt(g, 7, 7) === "lab tile (7,7) ground='L' layers=['Z']", describeAt(g, 7, 7));
}

// 5c. side profile is slimmer than front (narrow torso, one arm)
{
	const torsoW = (dir) => {
		const [calls, ops] = stubOps();
		paintActor(ops, LOOKS.mom, dir, 0);
		return Math.max(
			...calls.seq
				.filter((s) => s[0] === 'r')
				.map((s) => s.slice(1).split(',').map(Number))
				.filter(([, y]) => y >= 14 && y < 24)
				.map(([, , w]) => w),
			0
		);
	};
	const sideW = torsoW('side');
	const downW = torsoW('down');
	ok('side torso narrower', sideW < downW, `side=${sideW} down=${downW}`);
}

// 6. talking to Oak opens dialogue, A walks through it
{
	const g = createGame();
	enterMap(g, 'lab', 8, 5, 'up');
	g.state = 'walk';
	update(g, K(), P({ a: true })); // face (8,4) = oak
	ok('oak talks', g.state === 'text', g.state);
	let guard = 0;
	while (g.state === 'text' && guard++ < 40) {
		update(g, K(), P());
		update(g, K(), P({ a: true }));
	}
	ok('dialogue finishes', g.state === 'walk', `guard=${guard}`);
}

// 7. warp lookup sanity
ok('warp data sane', warpAt('pallet', 8, 7)?.to === 'lab' && warpAt('lab', 7, 11)?.to === 'pallet');
ok('no stray warps', warpAt('pallet', 0, 0) === null && warpAt('lab', 0, 0) === null);

// 8. NPC tiles block
{
	const g = createGame();
	enterMap(g, 'lab', 8, 5, 'up');
	g.state = 'walk';
	const blocked = isSolid('lab', 8, 4, g.npcs);
	ok('npc blocks tile', blocked);
}

// 8b. debug inspect: toggle + describe npc/house/sign/warp/tile
{
	const g = createGame();
	ok('debug off by default', g.debug === false);
	update(g, K(), P({ debug: true }));
	ok('debug toggles on', g.debug === true);
	update(g, K(), P({ debug: true }));
	ok('debug toggles off', g.debug === false);
	g.debug = true;
	ok('npc id string', describeAt(g, 33, 13) === 'pallet npc mom (33,13) look=mom dir=down', describeAt(g, 33, 13));
	ok('house id string', describeAt(g, 34, 11) === 'pallet house (32-36,8-11) style=ash door=33', describeAt(g, 34, 11));
	ok('sign id string', describeAt(g, 27, 2).startsWith('pallet sign (27,2) "PALLET TOWN.'), describeAt(g, 27, 2));
	ok('warp id string', describeAt(g, 8, 7) === 'pallet warp (8,7) -> lab (7,9)', describeAt(g, 8, 7));
	ok('tile id string', describeAt(g, 34, 16) === "pallet tile (34,16) ground='.' layers=[]", describeAt(g, 34, 16));
	const [cx, cy] = gameCam(g);
	ok('gameCam sane', cx >= 0 && cy >= 0, `${cx},${cy}`);
}

// 9. character generator: every map look exists, every dir/phase paints,
// walk frames differ, textures are coated
function stubOps() {
	const calls = { area: 0, coat: [], shade: 0, seq: [] };
	return [
		calls,
		{
			rect: (x, y, w, h) => {
				calls.area += w * h;
				calls.seq.push(`r${x},${y},${w},${h}`);
			},
			px: (x, y) => {
				calls.area += 1;
				calls.seq.push(`p${x},${y}`);
			},
			coat: (id) => calls.coat.push(id),
			shade: () => calls.shade++
		}
	];
}
{
	const used = new Set();
	for (const m of Object.values(MAPS)) for (const n of m.npcs) used.add(n.look);
	ok('all npc looks defined', [...used].every((k) => k in LOOKS), [...used].join(','));
	// Ash matrices: 32x32, palette range, stride preserves pixels but moves
	// legs, back view repaints face+jacket, side differs from front.
	{
		const dimsOk = [ASH_DOWN, ASH_SIDE].every(
			(m) => m.length === 32 && m.every((r) => r.length === 32)
		);
		ok('ash matrices 32x32', dimsOk);
		const vals = new Set();
		for (const m of [ASH_DOWN, ASH_SIDE]) for (const r of m) for (const v of r) vals.add(v);
		ok('ash palette range', [...vals].every((v) => v in ASH_PAL), [...vals].join(','));
		const count = (m) => {
			const c = {};
			for (const r of m) for (const v of r) c[v] = (c[v] ?? 0) + 1;
			return JSON.stringify(c);
		};
		const strided = stride(ASH_DOWN, [{ rows: [23, 24, 25, 26, 27, 28, 29], l0: 8, l1: 12, r0: 15, r1: 20 }]);
		ok('stride preserves pixels', count(strided) === count(ASH_DOWN));
		ok('stride moves left leg', strided[24][7] === ASH_DOWN[24][8] && strided[24][7] !== 0);
		const back = backView(ASH_DOWN);
		ok('back view repaints face', back[10][12] === 6 && ASH_DOWN[10][12] !== 6);
		ok('back view keeps outline', back[10][8] === 1 || back[13][7] === 1);
		ok('side differs from front', JSON.stringify(ASH_SIDE) !== JSON.stringify(ASH_DOWN));
		const fr = buildAshFrames();
		ok('ash frames complete', ['down', 'up', 'side'].every((d) => fr[d]?.length === 2));
	}
	// Tree matrix: 32x32, palette range, real coverage (canopy + trunk).
	{
		ok('tree matrix 32x32', TREE.length === 32 && TREE.every((r) => r.length === 32));
		const vals = new Set();
		const counts = {};
		for (const r of TREE) for (const v of r) {
			vals.add(v);
			counts[v] = (counts[v] ?? 0) + 1;
		}
		ok('tree palette range', [...vals].every((v) => v in TREE_PAL), [...vals].join(','));
		ok('tree has canopy', (counts[2] ?? 0) + (counts[3] ?? 0) > 250, `${(counts[2] ?? 0) + (counts[3] ?? 0)} leaf px`);
		ok('tree has outline', (counts[1] ?? 0) > 60, `${counts[1] ?? 0} edge px`);
		ok('tree has trunk', (counts[5] ?? 0) + (counts[6] ?? 0) > 40, `${(counts[5] ?? 0) + (counts[6] ?? 0)} trunk px`);
	}
	// Noise: deterministic, [0,1] range, fbm(1 octave) == vnoise, disc test.
	{
		ok('hash deterministic', hash2(12, -7, 3) === hash2(12, -7, 3), String(hash2(12, -7, 3)));
		let inRange = true;
		let varied = new Set();
		for (let x = -5; x < 6; x++) {
			for (let y = -5; y < 6; y++) {
				const v = vnoise(x * 0.7, y * 0.7);
				const f = fbm(x * 0.7, y * 0.7, 3);
				if (v < 0 || v > 1 || f < 0 || f > 1) inRange = false;
				varied.add(Math.round(v * 100));
			}
		}
		ok('noise in [0,1]', inRange);
		ok('noise varies', varied.size > 20, `${varied.size} levels`);
		ok('fbm1 == vnoise', Math.abs(fbm(3.2, -1.7, 1, 9) - vnoise(3.2, -1.7, 9)) < 1e-12);
		ok('inDisc', inDisc(1, 1, 0, 0, 2) && !inDisc(2, 2, 0, 0, 2));
	}
	// Outline shader: ring paints a band around the ellipse, edge pass only
	// touches empty neighbors of filled cells (deterministic per seed).
	{
		const painted = [];
		const ops = { rect: (x, y, w, h, c) => painted.push([x, y, w, h, c]) };
		cloudRing(ops, 16, 16, 10, 8, 7, '#1E2B1A');
		ok('cloudRing paints', painted.length > 40, `${painted.length} blobs`);
		const dists = painted.map(([x, y]) => Math.hypot(x - 16, y - 16));
		ok(
			'cloudRing stays in band',
			Math.min(...dists) > 3 && Math.max(...dists) < 20,
		 `min=${Math.min(...dists).toFixed(1)} max=${Math.max(...dists).toFixed(1)}`
		);
		ok('cloudRing uses outline color', painted.every(([, , , , c]) => c === '#1E2B1A'));
		// cloudDisc: filled fuzzy blob — many pixels, but radius varies by angle
		// (not a perfect circle).
		const discPx = [];
		const discOps = { rect: (x, y, w, h, c) => discPx.push([x, y, c]) };
		cloudDisc(discOps, 16, 16, 8, '#1E2B1A', 5);
		ok('cloudDisc fills', discPx.length > 120, `${discPx.length} px`);
		const radii = new Set(
			discPx.map(([x, y]) => Math.round(Math.hypot(x - 16, y - 16)))
		);
		ok('cloudDisc is lumpy', radii.size > 3, `${radii.size} radii`);
		const block = [
			[0, 0, 0, 0, 0],
			[0, 1, 1, 1, 0],
			[0, 1, 1, 1, 0],
			[0, 1, 1, 1, 0],
			[0, 0, 0, 0, 0]
		];
		const bumps = [];
		edgeOutline(
			(x, y) => x >= 0 && y >= 0 && x < 5 && y < 5 && block[y][x] === 1,
			(x, y, c) => bumps.push([x, y, c]),
			5,
			5,
			3,
			'#1E2B1A',
			0.45
		);
		ok('edgeOutline paints neighbors', bumps.length > 0, `${bumps.length} bumps`);
		ok(
			'edgeOutline never paints filled cells',
			bumps.every(([x, y]) => block[y][x] === 0)
		);
		const again = [];
		edgeOutline(
			(x, y) => x >= 0 && y >= 0 && x < 5 && y < 5 && block[y][x] === 1,
			(x, y, c) => again.push([x, y, c]),
			5,
			5,
			3,
			'#1E2B1A',
			0.45
		);
		ok('edgeOutline deterministic', JSON.stringify(bumps) === JSON.stringify(again));
	}
	// Bushes: solid decor on expected cells.
	{
		const spots = [[36, 12], [22, 18], [30, 28], [40, 20], [15, 9]];
		const g = createGame();
		enterMap(g, 'pallet', 35, 12, 'right');
		g.state = 'walk';
		update(g, K({ right: true }), P()); // (36,12) is a bush
		ok('bush blocks', !g.moving, `${g.mx},${g.my}`);
		ok(
			'bush spots planted',
			spots.every(([x, y]) => MAPS.pallet.grid[y][x] === 'b'),
			spots.filter(([x, y]) => MAPS.pallet.grid[y][x] !== 'b').map(([x, y]) => `${x},${y}`).join(';')
		);
	}
	// River vs lake: river cells are 'r' and flow (4 frames), lake shimmers (2).
	{
		const river = [[24, 1], [25, 10], [24, 20], [25, 27]];
		ok(
			'river cells are r',
			river.every(([x, y]) => MAPS.pallet.grid[y][x] === 'r'),
			river.filter(([x, y]) => MAPS.pallet.grid[y][x] !== 'r').map(([x, y]) => `${x},${y}`).join(';')
		);
		ok('bridge over river', MAPS.pallet.grid[16][24] === '+' && MAPS.pallet.grid[16][25] === '+');
		ok('pond stays lake', MAPS.pallet.grid[29][22] === '~' && MAPS.pallet.grid[30][24] === 'o' && MAPS.pallet.grid[31][26] === 'o');
		ok(
			'river flows (8 frames)',
			[0, 15, 30, 45].map((f) => tileFrame('r', 24, 5, f)).join(',') === '0,1,2,3' &&
				[0, 15, 30, 45, 60, 75, 90, 105].map((f) => tileFrame('r', 24, 5, f)).join(',') === '0,1,2,3,4,5,6,7'
		);
		ok(
			'lake shimmers (2 frames)',
			[0, 30, 60].map((f) => tileFrame('~', 21, 29, f)).join(',') === '0,1,0'
		);
	}
	// Two-pass layers: every cell resolves to a painted ground tile + layers.
	{
		const seen = new Set();
		for (const m of Object.values(MAPS))
			for (let y = 0; y < m.grid.length; y++)
				for (let x = 0; x < m.grid[y].length; x++) seen.add(normCell(m, x, y).ground);
		const fullCover = new Set([',', '.', '~', ':', 'L', 'r', 'R', 'H', 'w', 'Y', 'V', 'B', 'M', 'N']);
		const noground = [...seen].filter((gch) => !fullCover.has(gch));
		ok('all tiles have ground coverage', noground.length === 0, noground.join(','));
		for (const [ch, gch] of Object.entries(GROUND)) {
			ok(`ground ${ch}->${gch} is a ground layer`, fullCover.has(gch) && !GROUND[gch], `${ch}->${gch}`);
		}
	}
	for (const [name, look] of Object.entries(LOOKS)) {
		for (const dir of ['down', 'up', 'side']) {
			const seqs = [];
			for (const phase of [0, 1]) {
				const [calls, ops] = stubOps();
				paintActor(ops, look, dir, phase);
				ok(`${name}/${dir}/f${phase} paints`, calls.area > 300, `${calls.area}px`);
				seqs.push(calls.seq.join(';'));
			}
			ok(`${name}/${dir} walk frames differ`, seqs[0] !== seqs[1]);
		}
		const calls = (() => {
			const [c, o] = stubOps();
			paintActor(o, look, 'down', 0);
			return c;
		})();
		if (look.shirtTex) ok(`${name} coats ${look.shirtTex}`, calls.coat.includes(look.shirtTex), calls.coat.join(','));
		ok(`${name} shaded`, calls.shade > 0);
	}
}

// 10. texture recipes all produce coverage (pure, no DOM)
{
	const dead = Object.entries(TEX).filter(([, r]) => {
		let n = 0;
		for (let y = 0; y < r.size; y++) for (let x = 0; x < r.size; x++) if (r.fn(x, y) > 0) n++;
		return n === 0;
	});
	ok('all textures have coverage', dead.length === 0, dead.map(([k]) => k).join(','));
}

// 12. every painter runs without DOM and only emits valid colors
{
	const colors = [];
	const ctxStub = new Proxy(
		{},
		{
			get: (t, p) => {
				if (p === 'createPattern') return () => ({});
				if (p === 'canvas') return {};
				return (...a) => undefined;
			},
			set: (t, p, v) => {
				if (p === 'fillStyle') colors.push(v);
				t[p] = v;
				return true;
			}
		}
	);
	globalThis.document = {
		createElement: () => ({ width: 0, height: 0, getContext: () => ctxStub })
	};
	const tiles = await import('../src/lib/tiles.js');
	let threw = null;
	try {
		const art = tiles.buildArt();
		for (const [ch, frames] of Object.entries(art.tiles)) {
			if (!Array.isArray(frames) || frames.length === 0) throw new Error('no frames for ' + ch);
		}
		for (const m of Object.values(MAPS)) {
			for (const row of m.grid) {
				for (const ch of row) {
					const frames = art.tiles[ch] ?? art.tiles[GROUND[ch] ?? ','];
					if (!frames || !frames.length) throw new Error(`no art for grid char '${ch}'`);
				}
			}
		}
		for (const roof of ['red', 'blue', 'gray']) {
			for (const w of [5, 7]) {
				const c = { getContext: () => ctxStub };
				tiles.paintHouse(c.getContext(), 0, 0, w, roof, 2);
			}
		}
		for (const style of ['lab', 'ash', 'gary']) {
			tiles.paintBuilding(ctxStub, 7, 4, { style });
		}
		for (let i = 1; i < 16; i++) {
			tiles.paintFoam(ctxStub, 0, 0, { n: i & 1, s: i & 2, e: i & 4, w: i & 8 });
		}
	} catch (e) {
		threw = e;
	}
	delete globalThis.document;
	ok('all painters run headless', !threw, threw ? threw.message : `${colors.length} fills`);
	ok(
		'building heights defined',
		BUILDING_H.lab === 128 && BUILDING_H.ash === 128 && BUILDING_H.gary === 128 && BUILDING_H.house > 0
	);
	const bad = colors.filter(
		(c) => typeof c === 'string' && !/^(#[0-9a-f]{6}|rgba?\([^)]*\))$/i.test(c)
	);
	ok('all paint colors valid', bad.length === 0, bad.slice(0, 3).join(','));
}{
	const errs = [];
	for (const [id, m] of Object.entries(MAPS)) {
		const W = m.grid[0].length;
		const H = m.grid.length;
		for (const p of m.props ?? []) {
			if (p.type !== 'house') continue;
			if (p.x < 0 || p.y < 0 || p.x + p.w > W || p.y + 4 > H) errs.push(`${id}: prop out of bounds`);
			for (let dx = 0; dx < p.w; dx++) {
				for (let dy = 0; dy < 4; dy++) {
					const ch = m.grid[p.y + dy][p.x + dx];
					if (!'RHWwd'.includes(ch)) errs.push(`${id}: (${p.x + dx},${p.y + dy})='${ch}' not house`);
				}
			}
			const doorCh = m.grid[p.y + 3][p.x + p.door];
			if (doorCh !== 'd') errs.push(`${id}: door tile is '${doorCh}'`);
		}
		const covered = new Set();
		for (const p of m.props ?? []) {
			if (p.type !== 'house') continue;
			for (let dx = 0; dx < p.w; dx++) for (let dy = 0; dy < 4; dy++) covered.add(`${p.x + dx},${p.y + dy}`);
		}
		if (covered.has(`${m.spawn.x},${m.spawn.y}`)) errs.push(`${id}: spawn inside house`);
		for (const n of m.npcs) if (covered.has(`${n.x},${n.y}`)) errs.push(`${id}: npc ${n.id} inside house`);
	}
	ok('house props sane', errs.length === 0, errs.join('; '));
}

// 13b. styled buildings (lab / ash / gary) + 2-story gray house + field prop
{
	ok(
		'lab building styled',
		MAPS.pallet.props.some((p) => p.type === 'house' && p.style === 'lab' && p.x === 5 && p.y === 4 && p.w === 7)
	);
	ok(
		'ash + gary buildings styled',
		MAPS.pallet.props.some((p) => p.type === 'house' && p.style === 'ash' && p.x === 32 && p.w === 5) &&
			MAPS.pallet.props.some((p) => p.type === 'house' && p.style === 'gary' && p.x === 38 && p.w === 6)
	);
	ok(
		'gray house is 2-story',
		MAPS.pallet.props.some((p) => p.type === 'house' && p.roof === 'gray' && p.floors === 2)
	);
	const g = createGame();
	ok('lab building described', describeAt(g, 6, 5) === 'pallet house (5-11,4-7) style=lab door=8', describeAt(g, 6, 5));
	ok('ash building described', describeAt(g, 33, 11) === 'pallet house (32-36,8-11) style=ash door=33', describeAt(g, 33, 11));
	ok(
		'flower field prop covers V region',
		MAPS.pallet.props.some((p) => p.type === 'field' && p.x === 37 && p.y === 25 && p.w === 9 && p.h === 8)
	);
	ok(
		'sunflower prop present',
		MAPS.pallet.props.some((p) => p.type === 'sunflower' && p.x === 17 && p.y === 24 && p.count === 3)
	);
}

console.log(fails ? `${fails} FAILURES` : 'ALL GREEN');
process.exit(fails ? 1 : 0);
