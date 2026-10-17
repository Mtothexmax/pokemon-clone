/**
 * core.js — framebuffer helpers, 5x7 pixel font, input, tiny audio.
 *
 * No DOM access at import time, so Node can import this for headless tests.
 * The game draws on a 160x144 canvas (Game Boy resolution); CSS scales it
 * up with crisp pixels.
 */

export const TILE = 32;

/**
 * Viewport in game pixels (multiples of TILE). Not fixed: `fitScreen` derives
 * the visible tile count from the window size so the canvas fills the window
 * at the window's own proportions (crisp pixels, ~2x on-screen tile size).
 * Module live bindings keep game.js in sync.
 */
export let VW = 320;
export let VH = 288;

/** On-screen pixels per game pixel. */
const VIEW_SCALE = 2;

/**
 * Snap the canvas to the window. The viewport is as many whole 32px tiles as
 * fill the window (never more than the current map, so no black void shows),
 * then the canvas is stretched to the full window so there are never black
 * bars — the game always uses whatever space/proportion the window gives it.
 */
export function fitScreen(canvas, mapW = Infinity, mapH = Infinity) {
	if (typeof window === 'undefined') return;
	const cols = Math.max(10, Math.ceil(window.innerWidth / (TILE * VIEW_SCALE)));
	const rows = Math.max(9, Math.ceil(window.innerHeight / (TILE * VIEW_SCALE)));
	VW = Math.min(cols * TILE, mapW);
	VH = Math.min(rows * TILE, mapH);
	canvas.width = VW;
	canvas.height = VH;
	canvas.style.width = window.innerWidth + 'px';
	canvas.style.height = window.innerHeight + 'px';
}

/** 5x7 pixel font, uppercase only (like the Game Boy originals). */
const GLYPHS = {
	A: ['.###.', '#...#', '#...#', '#####', '#...#', '#...#', '#...#'],
	B: ['####.', '#...#', '#...#', '####.', '#...#', '#...#', '####.'],
	C: ['.####', '#....', '#....', '#....', '#....', '#....', '.####'],
	D: ['####.', '#...#', '#...#', '#...#', '#...#', '#...#', '####.'],
	E: ['#####', '#....', '#....', '####.', '#....', '#....', '#####'],
	F: ['#####', '#....', '#....', '####.', '#....', '#....', '#....'],
	G: ['.####', '#....', '#....', '#.###', '#...#', '#...#', '.###.'],
	H: ['#...#', '#...#', '#...#', '#####', '#...#', '#...#', '#...#'],
	I: ['#####', '..#..', '..#..', '..#..', '..#..', '..#..', '#####'],
	J: ['..###', '...#.', '...#.', '...#.', '...#.', '#..#.', '.##..'],
	K: ['#...#', '#..#.', '#.#..', '##...', '#.#..', '#..#.', '#...#'],
	L: ['#....', '#....', '#....', '#....', '#....', '#....', '#####'],
	M: ['#...#', '##.##', '#.#.#', '#.#.#', '#...#', '#...#', '#...#'],
	N: ['#...#', '##..#', '##..#', '#.#.#', '#..##', '#..##', '#...#'],
	O: ['.###.', '#...#', '#...#', '#...#', '#...#', '#...#', '.###.'],
	P: ['####.', '#...#', '#...#', '####.', '#....', '#....', '#....'],
	Q: ['.###.', '#...#', '#...#', '#...#', '#.#.#', '#..#.', '.##.#'],
	R: ['####.', '#...#', '#...#', '####.', '#.#..', '#..#.', '#...#'],
	S: ['.####', '#....', '#....', '.###.', '....#', '....#', '####.'],
	T: ['#####', '..#..', '..#..', '..#..', '..#..', '..#..', '..#..'],
	U: ['#...#', '#...#', '#...#', '#...#', '#...#', '#...#', '.###.'],
	V: ['#...#', '#...#', '#...#', '#...#', '#...#', '..#.#', '...#.'],
	W: ['#...#', '#...#', '#...#', '#.#.#', '#.#.#', '##.##', '#...#'],
	X: ['#...#', '#...#', '..#.#', '...#.', '..#.#', '#...#', '#...#'],
	Y: ['#...#', '#...#', '..#.#', '...#.', '..#..', '..#..', '..#..'],
	Z: ['#####', '....#', '...#.', '..#..', '.#...', '#....', '#####'],
	'0': ['.###.', '#..##', '#.#.#', '#.#.#', '##..#', '#...#', '.###.'],
	'1': ['..#..', '.##..', '..#..', '..#..', '..#..', '..#..', '.###.'],
	'2': ['.###.', '#...#', '....#', '...#.', '..#..', '.#...', '#####'],
	'3': ['#####', '...#.', '..#..', '...#.', '....#', '#...#', '.###.'],
	'4': ['...#.', '..##.', '.#.#.', '#..#.', '#####', '...#.', '...#.'],
	'5': ['#####', '#....', '####.', '....#', '....#', '#...#', '.###.'],
	'6': ['..##.', '.#...', '#....', '####.', '#...#', '#...#', '.###.'],
	'7': ['#####', '....#', '...#.', '..#..', '.#...', '.#...', '.#...'],
	'8': ['.###.', '#...#', '#...#', '.###.', '#...#', '#...#', '.###.'],
	'9': ['.###.', '#...#', '#...#', '.####', '....#', '...#.', '.##..'],
	' ': ['.....', '.....', '.....', '.....', '.....', '.....', '.....'],
	'.': ['.....', '.....', '.....', '.....', '.....', '.##..', '.##..'],
	',': ['.....', '.....', '.....', '.....', '.##..', '.##..', '.#...'],
	'!': ['..#..', '..#..', '..#..', '..#..', '..#..', '.....', '..#..'],
	'?': ['.###.', '#...#', '....#', '...#.', '..#..', '.....', '..#..'],
	"'": ['..#..', '..#..', '.....', '.....', '.....', '.....', '.....'],
	':': ['.....', '.##..', '.##..', '.....', '.##..', '.##..', '.....'],
	';': ['.....', '.##..', '.##..', '.....', '.##..', '.##..', '.#...'],
	'-': ['.....', '.....', '.....', '#####', '.....', '.....', '.....'],
	'/': ['....#', '....#', '...#.', '..#..', '.#...', '#....', '#....'],
	'(': ['...#.', '..#..', '.#...', '.#...', '.#...', '..#..', '...#.'],
	')': ['.#...', '..#..', '...#.', '...#.', '...#.', '..#..', '.#...'],
	'%': ['##..#', '##.#.', '..#..', '.#...', '#..##', '#..##', '.....']
};

/** Draw uppercase text (unknown chars become space). s = pixel scale. */
export function drawText(g, str, x, y, color = '#fff', s = 1) {
	g.fillStyle = color;
	let cx = x;
	for (const raw of String(str).toUpperCase()) {
		const gl = GLYPHS[raw] ?? GLYPHS[' '];
		for (let r = 0; r < 7; r++) {
			const row = gl[r];
			for (let c = 0; c < 5; c++) {
				if (row[c] === '#') g.fillRect(cx + c * s, y + r * s, s, s);
			}
		}
		cx += 6 * s;
	}
	return cx - x - s;
}

/** Word-wrap for the dialogue box (~24 chars/line at 2x scale). */
export function wrap(text, max = 24) {
	const words = String(text).toUpperCase().split(' ');
	const lines = [];
	let line = '';
	for (const w of words) {
		const next = line ? line + ' ' + w : w;
		if (next.length > max) {
			if (line) lines.push(line);
			line = w;
		} else {
			line = next;
		}
	}
	if (line) lines.push(line);
	return lines;
}

/** Classic dialogue box: white fill, dark border. */
export function drawBox(g, x, y, w, h) {
	g.fillStyle = '#f8f8f8';
	g.fillRect(x, y, w, h);
	g.fillStyle = '#202020';
	g.fillRect(x, y, w, 2);
	g.fillRect(x, y + h - 2, w, 2);
	g.fillRect(x, y, 2, h);
	g.fillRect(x + w - 2, y, 2, h);
}

// ------------------------------------------------------------- input

/** Live key state. Keys: up/down/left/right/a/b/start (+pressed edge flags). */
export const keys = {
	up: false, down: false, left: false, right: false,
	a: false, b: false, start: false, debug: false
};
/** Edge-triggered: true for one frame when the key goes down. */
export const pressed = {
	up: false, down: false, left: false, right: false,
	a: false, b: false, start: false, debug: false
};

const KEYMAP = {
	ArrowUp: 'up', KeyW: 'up',
	ArrowDown: 'down', KeyS: 'down',
	ArrowLeft: 'left', KeyA: 'left',
	ArrowRight: 'right', KeyD: 'right',
	KeyZ: 'a', KeyJ: 'a', Space: 'a',
	KeyX: 'b', KeyK: 'b', Escape: 'b', ShiftLeft: 'b', ShiftRight: 'b',
	Enter: 'start',
	Backquote: 'debug'
};

/**
 * Set a logical key state. Used by the keyboard binding and by the on-screen
 * touch pad, so both drive the exact same `keys`/`pressed` state.
 */
export function setKey(name, down) {
	if (!(name in keys)) return;
	if (down && !keys[name]) pressed[name] = true;
	keys[name] = down;
}

/** Release every held key (e.g. when the touch pad is dismissed). */
export function releaseAll() {
	for (const k of Object.keys(keys)) keys[k] = false;
}

/** Bind keyboard. Call once from the browser entry point. */
export function bindInput() {
	window.addEventListener('keydown', (e) => {
		const name = KEYMAP[e.code];
		if (name) {
			setKey(name, true);
			e.preventDefault();
		}
	});
	window.addEventListener('keyup', (e) => {
		const name = KEYMAP[e.code];
		if (name) setKey(name, false);
	});
	window.addEventListener('blur', () => {
		for (const k of Object.keys(keys)) keys[k] = false;
	});
}

/** Clear edge flags at the end of each frame. */
export function endFrame() {
	for (const k of Object.keys(pressed)) pressed[k] = false;
}

// ------------------------------------------------------------- audio

let audioCtx = null;
/** Tiny square-wave blip (text advance, bumps, warps). Silent until first input. */
export function blip(freq = 660, dur = 0.05) {
	try {
		audioCtx ??= new (window.AudioContext || window.webkitAudioContext)();
		if (audioCtx.state === 'suspended') void audioCtx.resume();
		const t = audioCtx.currentTime;
		const o = audioCtx.createOscillator();
		const g = audioCtx.createGain();
		o.type = 'square';
		o.frequency.setValueAtTime(freq, t);
		g.gain.setValueAtTime(0.04, t);
		g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
		o.connect(g).connect(audioCtx.destination);
		o.start(t);
		o.stop(t + dur + 0.01);
	} catch {
		/* no audio — game still works */
	}
}
