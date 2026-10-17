/**
 * bootStrict.mjs — run the full art boot path with a STRICT canvas stub
 * (throws on negative dims like a real canvas) to catch runtime errors
 * that the lenient Proxy stub in headless.mjs would swallow.
 */
function strictCtx(log) {
	const canvasStub = { width: 32, height: 32 };
	const g = {
		canvas: canvasStub,
		fillStyle: '#000',
		globalAlpha: 1,
		globalCompositeOperation: 'source-over',
		lineWidth: 1,
		fillRect(x, y, w, h) {
			if (!Number.isFinite(x + y + w + h)) throw new Error(`non-finite fillRect ${x},${y},${w},${h}`);
			if (w < 0 || h < 0) throw new Error(`negative fillRect ${w}x${h}`);
			log.fills++;
		},
		clearRect() {},
		beginPath() {},
		rect() {},
		clip() {},
		ellipse() {},
		arc(x, y, r) {
			if (!(r >= 0)) throw new Error(`bad arc radius ${r}`);
			log.arcs++;
		},
		fill() {},
		stroke() {},
		save() {},
		restore() {},
		translate() {},
		scale() {},
		drawImage() {
			log.blits++;
		},
		createPattern: () => ({}),
		setLineDash() {},
		measureText: () => ({ width: 0 })
	};
	return g;
}

globalThis.document = {
	createElement: () => {
		const c = { width: 32, height: 32 };
		c.getContext = () => strictCtx(globalThis.__log);
		return c;
	}
};
globalThis.__log = { fills: 0, arcs: 0, blits: 0 };

const tiles = await import('../src/lib/tiles.js');
const chargen = await import('../src/lib/chargen.js');
const maps = await import('../src/lib/maps.js');

let fails = 0;
const ok = (n, c, e = '') => {
	console.log(`${c ? '  ok  ' : ' FAIL '} ${n}${e ? ' — ' + e : ''}`);
	if (!c) fails++;
};

try {
	const art = tiles.buildArt();
	let n = 0;
	for (const frames of Object.values(art.tiles)) n += frames.length;
	ok('buildArt strict boot', n > 10, `${n} tile canvases`);
	// painters that main.js calls directly
	const c0 = { getContext: () => strictCtx(globalThis.__log) };
	tiles.paintHouse(c0.getContext(), 0, 0, 7, 'red', 3);
	tiles.paintHouse(c0.getContext(), 0, 0, 5, 'blue', 2);
	for (const style of ['lab', 'ash', 'gary']) tiles.paintBuilding(c0.getContext(), 7, 4, { style });
	tiles.paintField(c0.getContext(), 9 * 32, 8 * 32, 20250);
	for (let i = 1; i < 16; i++) {
		tiles.paintFoam(c0.getContext(), 0, 0, { n: i & 1, s: i & 2, e: i & 4, w: i & 8 });
	}
	const { drawSunflowerPatch } = await import('../src/lib/sunflower.js');
	drawSunflowerPatch(c0.getContext(), 3 * 32, 2 * 32, { count: 3, seed: 4211 });
	ok('houses+fields+buildings strict boot', true);
	const actors = chargen.buildActors();
	let m = 0;
	for (const [k, v] of Object.entries(actors)) {
		if (k === 'shadow') continue;
		for (const d of Object.values(v)) m += d.length;
	}
	ok('buildActors strict boot', m >= 24, `${m} frames`);
} catch (e) {
	ok('strict boot', false, e.message);
}
console.log(`fills=${globalThis.__log.fills} arcs=${globalThis.__log.arcs}`);
console.log(fails ? `${fails} FAILURES` : 'ALL GREEN');
process.exit(fails ? 1 : 0);
