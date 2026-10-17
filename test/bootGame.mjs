/**
 * bootGame.mjs — full startGame() boot + frames headless with stub DOM.
 * Catches runtime errors in drawGame/update that unit stubs would miss.
 */
function strictCtx() {
	return {
		canvas: {},
		fillStyle: '#000',
		globalAlpha: 1,
		globalCompositeOperation: 'source-over',
		lineWidth: 1,
		fillRect() {},
		clearRect() {},
		beginPath() {},
		rect() {},
		clip() {},
		ellipse() {},
		arc() {},
		fill() {},
		stroke() {},
		save() {},
		restore() {},
		translate() {},
		scale() {},
		drawImage() {},
		createPattern: () => ({}),
		setLineDash() {},
		measureText: () => ({ width: 0 }),
		getImageData: () => ({ data: [] })
	};
}
globalThis.window = {
	innerWidth: 1600,
	innerHeight: 1000,
	addEventListener() {},
	removeEventListener() {},
	__game: null
};
globalThis.document = {
	createElement: () => {
		const c = { width: 32, height: 32 };
		c.getContext = () => strictCtx();
		return c;
	}
};
globalThis.performance = { now: () => Date.now() };
let rafCb = null;
globalThis.requestAnimationFrame = (cb) => {
	rafCb = cb;
	return 1;
};
globalThis.cancelAnimationFrame = () => {
	rafCb = null;
};

let fails = 0;
const ok = (n, c, e = '') => {
	console.log(`${c ? '  ok  ' : ' FAIL '} ${n}${e ? ' — ' + e : ''}`);
	if (!c) fails++;
};

try {
	const { startGame } = await import('../src/lib/main.js');
	const { keys } = await import('../src/lib/core.js');
	const canvas = { width: 300, height: 150, style: {}, getContext: () => strictCtx(), addEventListener() {}, getBoundingClientRect: () => ({ left: 0, top: 0, width: 300, height: 150 }) };
	const { dispose } = startGame(canvas, {});
	ok('startGame boots', !!window.__game);
	// Walk east with a clock consistent with startGame's Date.now base.
	keys.right = true;
	let t = Date.now();
	for (let i = 0; i < 90; i++) {
		t += 1000 / 60;
		const cb = rafCb;
		rafCb = null;
		cb(t);
	}
	keys.right = false;
	const g = window.__game;
	ok('90 frames without throw', true);
	ok('player walked east', g.mx > 34, `mx=${g.mx} my=${g.my} map=${g.mapId}`);
	dispose();
} catch (e) {
	ok('bootGame', false, e.stack.split('\n').slice(0, 3).join(' | '));
}
console.log(fails ? `${fails} FAILURES` : 'ALL GREEN');
process.exit(fails ? 1 : 0);
