// Tiny synthesized sound effects (no audio files needed).
import { assetUrl } from '../asset-url';
let ctx: AudioContext | null = null;
let enabled = true;
/** false while the browser tab has no focus: no game sound at all */
let focused = true;
export function setSfxFocused(on: boolean) {
	focused = on;
	if (!ctx) return;
	if (on) ctx.resume().catch(() => {});
	else ctx.suspend().catch(() => {});
}

export function setSfxEnabled(on: boolean) {
	enabled = on;
}

function ac() {
	if (!ctx) {
		try {
			ctx = new AudioContext();
		} catch {
			return null;
		}
	}
	if (!focused) return null;
	if (ctx.state === 'suspended') ctx.resume().catch(() => {});
	return ctx;
}

function tone(freq: number, start: number, dur: number, type: OscillatorType = 'sine', vol = 0.15) {
	const a = ac();
	if (!a || !enabled) return;
	const t = a.currentTime + start;
	const o = a.createOscillator();
	const g = a.createGain();
	o.type = type;
	o.frequency.setValueAtTime(freq, t);
	g.gain.setValueAtTime(0, t);
	g.gain.linearRampToValueAtTime(vol, t + 0.01);
	g.gain.exponentialRampToValueAtTime(0.001, t + dur);
	o.connect(g).connect(a.destination);
	o.start(t);
	o.stop(t + dur + 0.05);
}

// ---------------- per-card attack sounds (retro handheld style, all synthesized) ----------------
type Out = AudioNode;
function hashStr(s: string) {
	let h = 2166136261;
	for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
	return h >>> 0;
}
/** small seeded PRNG so every card always gets the same (but its own) sound */
function rng(seed: number) {
	return () => {
		seed = (seed + 0x6d2b79f5) | 0;
		let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
		t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}
/** oscillator gliding from f0 to f1, optional vibrato */
function sweep(out: Out, f0: number, f1: number, start: number, dur: number, type: OscillatorType, vol: number, vibHz = 0, vibDepth = 0) {
	const a = out.context as AudioContext;
	const t = a.currentTime + start;
	const o = a.createOscillator();
	const g = a.createGain();
	o.type = type;
	o.frequency.setValueAtTime(Math.max(20, f0), t);
	o.frequency.exponentialRampToValueAtTime(Math.max(20, f1), t + dur);
	if (vibHz) {
		const l = a.createOscillator();
		const lg = a.createGain();
		l.frequency.value = vibHz;
		lg.gain.value = vibDepth;
		l.connect(lg).connect(o.frequency);
		l.start(t);
		l.stop(t + dur + 0.05);
	}
	g.gain.setValueAtTime(0, t);
	g.gain.linearRampToValueAtTime(vol, t + 0.008);
	g.gain.exponentialRampToValueAtTime(0.001, t + dur);
	o.connect(g).connect(out);
	o.start(t);
	o.stop(t + dur + 0.05);
}
/** filtered noise burst, the filter frequency glides from f0 to f1 */
function noise(out: Out, start: number, dur: number, vol: number, ftype: BiquadFilterType, f0: number, f1: number, q = 1) {
	const a = out.context as AudioContext;
	const t = a.currentTime + start;
	const buf = a.createBuffer(1, Math.max(1, Math.floor(a.sampleRate * dur)), a.sampleRate);
	const d = buf.getChannelData(0);
	for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
	const src = a.createBufferSource();
	src.buffer = buf;
	const f = a.createBiquadFilter();
	f.type = ftype;
	f.Q.value = q;
	f.frequency.setValueAtTime(Math.max(30, f0), t);
	f.frequency.exponentialRampToValueAtTime(Math.max(30, f1), t + dur);
	const g = a.createGain();
	g.gain.setValueAtTime(vol, t);
	g.gain.exponentialRampToValueAtTime(0.001, t + dur);
	src.connect(f).connect(g).connect(out);
	src.start(t);
}
/** semitone steps -> frequency */
const st = (base: number, n: number) => base * Math.pow(2, n / 12);

type Theme = 'fire' | 'water' | 'electric' | 'plant' | 'boom' | 'wind' | 'rock' | 'ghost' | 'light' | 'ice' | 'mech' | 'jump' | 'gold';
const THEMES: Record<string, Theme> = {
	flicker: 'fire', cinderpup: 'fire', emberhorn: 'fire', magmaw: 'fire',
	drizzlet: 'water', tidalotl: 'water', coralyth: 'water', abyssquid: 'water',
	zapling: 'electric', voltail: 'electric', sparkmole: 'electric', thundrake: 'electric', stormaw: 'electric',
	sproutle: 'plant', mosscub: 'plant', fungoo: 'plant', bramblewolf: 'plant', titanoak: 'plant',
	boomshroom: 'boom',
	gustling: 'wind', zephyrrow: 'wind',
	pebbit: 'rock', rockodile: 'rock', cragoliath: 'rock', buildbeaver: 'rock', brickling: 'rock',
	gloomoth: 'ghost', phantomule: 'ghost', echowl: 'ghost',
	glimmerling: 'light', lumifox: 'light',
	glaciomoth: 'ice',
	tickbot: 'mech', clockcrab: 'mech',
	quickhop: 'jump',
	goldbrick: 'gold'
};
const ALL_THEMES: Theme[] = ['fire', 'water', 'electric', 'plant', 'wind', 'rock', 'ghost', 'light', 'mech', 'jump'];

/** the signature attack sound of a card: themed by its creature, varied by its id, lower for bigger (pricier) cards */
function attackSound(id: string, cost: number, support: boolean) {
	const a = ac();
	if (!a || !enabled) return;
	const seed = hashStr(id);
	const r = rng(seed);
	const theme = THEMES[id] ?? ALL_THEMES[seed % ALL_THEMES.length];
	const p = (1.3 - Math.min(6, cost) * 0.09) * (0.82 + r() * 0.36) * (support ? 1.12 : 1);
	const out = a.createGain();
	out.gain.value = 0.9;
	out.connect(a.destination);
	const wave = (): OscillatorType => (['square', 'sawtooth', 'triangle'] as const)[Math.floor(r() * 3)];
	switch (theme) {
		case 'fire': {
			noise(out, 0, 0.32 + r() * 0.15, 0.13, 'bandpass', 2400 * p, 500 * p, 0.9);
			const pops = 3 + Math.floor(r() * 4);
			for (let i = 0; i < pops; i++) noise(out, 0.04 + r() * 0.28, 0.03, 0.12, 'highpass', 3000, 2000);
			sweep(out, 420 * p, 110 * p, 0.02, 0.3, 'sawtooth', 0.05);
			break;
		}
		case 'water': {
			const n = 3 + Math.floor(r() * 4);
			const gap = 0.045 + r() * 0.03;
			for (let i = 0; i < n; i++) {
				const f = (260 + r() * 260) * p;
				sweep(out, f, f * (2.2 + r()), i * gap, 0.07, 'sine', 0.11);
			}
			noise(out, n * gap, 0.18, 0.07, 'lowpass', 1400, 300);
			break;
		}
		case 'electric': {
			sweep(out, 950 * p, 280 * p, 0, 0.26 + r() * 0.1, 'square', 0.055, 30 + r() * 30, 260 * p);
			sweep(out, 1500 * p, 500 * p, 0.12 + r() * 0.06, 0.16, 'sawtooth', 0.04, 55, 120);
			noise(out, 0, 0.22, 0.05, 'highpass', 4000, 2500);
			break;
		}
		case 'plant': {
			const scale = [0, 2, 4, 7, 9, 12, 14, 16];
			const n = 3 + Math.floor(r() * 3);
			const base = 330 * p;
			const w = r() < 0.5 ? 'triangle' : 'square';
			let k = Math.floor(r() * 2);
			for (let i = 0; i < n; i++) {
				const f = st(base, scale[Math.min(scale.length - 1, k)]);
				sweep(out, f, f, i * 0.065, 0.1, w, w === 'square' ? 0.045 : 0.09);
				k += 1 + Math.floor(r() * 2);
			}
			noise(out, 0, 0.2, 0.04, 'highpass', 5000, 3000);
			break;
		}
		case 'boom': {
			noise(out, 0.05, 0.7, 0.28, 'lowpass', 1600, 90, 0.7);
			sweep(out, 140 * p, 35, 0.05, 0.55, 'square', 0.09);
			sweep(out, 600 * p, 900 * p, 0, 0.06, 'square', 0.04); // fuse
			break;
		}
		case 'wind': {
			const d = 0.4 + r() * 0.2;
			noise(out, 0, d, 0.16, 'bandpass', 350 * p, 2600 * p, 2.5 + r() * 3);
			sweep(out, 200 * p, 520 * p, 0.05, d * 0.8, 'sine', 0.04, 6, 30);
			break;
		}
		case 'rock': {
			sweep(out, 170 * p, 48 * p, 0, 0.22, 'square', 0.1);
			noise(out, 0, 0.18, 0.14, 'lowpass', 700 * p, 120, 0.8);
			const hits = 1 + Math.floor(r() * 2);
			for (let i = 1; i <= hits; i++) sweep(out, 130 * p, 45 * p, i * (0.09 + r() * 0.04), 0.16, 'square', 0.07);
			break;
		}
		case 'ghost': {
			// eerie wobble with a small echo
			const delay = a.createDelay(0.5);
			delay.delayTime.value = 0.12 + r() * 0.08;
			const fb = a.createGain();
			fb.gain.value = 0.35;
			const wet = a.createGain();
			wet.gain.value = 0.5;
			out.connect(delay).connect(fb).connect(delay);
			delay.connect(wet).connect(a.destination);
			sweep(out, 760 * p, 230 * p, 0, 0.5, r() < 0.5 ? 'sine' : 'triangle', 0.09, 6 + r() * 4, 45 * p);
			sweep(out, 380 * p, 160 * p, 0.12, 0.35, 'sine', 0.05, 9, 20);
			break;
		}
		case 'light': {
			const n = 4 + Math.floor(r() * 3);
			const base = 1050 * p;
			for (let i = 0; i < n; i++) sweep(out, st(base, Math.floor(r() * 15)), st(base, Math.floor(r() * 15) + 12), i * 0.045, 0.09, 'triangle', 0.07);
			sweep(out, st(base, 12), st(base, 24), n * 0.045, 0.2, 'sine', 0.06);
			break;
		}
		case 'ice': {
			noise(out, 0, 0.35, 0.07, 'highpass', 6000, 9000);
			for (let i = 0; i < 4; i++) sweep(out, st(1400 * p, i * 5), st(1400 * p, i * 5 + 1), i * 0.06, 0.12, 'sine', 0.07);
			sweep(out, 320 * p, 180 * p, 0.05, 0.3, 'triangle', 0.06);
			break;
		}
		case 'mech': {
			const ticks = 3 + Math.floor(r() * 3);
			for (let i = 0; i < ticks; i++) sweep(out, i % 2 ? 900 * p : 1300 * p, i % 2 ? 880 * p : 1250 * p, i * 0.07, 0.03, 'square', 0.06);
			sweep(out, 1760 * p, 1760 * p, ticks * 0.07 + 0.02, 0.25, 'triangle', 0.08); // ding
			break;
		}
		case 'jump': {
			sweep(out, 220 * p, 900 * p, 0, 0.12, wave(), 0.06);
			sweep(out, 500 * p, 1300 * p, 0.12, 0.12, 'square', 0.05);
			break;
		}
		case 'gold': {
			[0, 4, 7, 12, 16].forEach((n, i) => sweep(out, st(988 * p, n), st(988 * p, n), i * 0.06, 0.16, 'square', 0.05));
			break;
		}
	}
}

// ---------------- recorded samples (played through the same AudioContext, so focus / mute rules apply) ----------------
const buffers = new Map<string, Promise<AudioBuffer | null>>();
function sample(url: string, vol: number, rateJitter = 0, offset = 0) {
	const a = ac();
	if (!a || !enabled) return;
	let p = buffers.get(url);
	if (!p) {
		p = fetch(assetUrl(url))
			.then((r) => r.arrayBuffer())
			.then((b) => a.decodeAudioData(b))
			.catch(() => null);
		buffers.set(url, p);
	}
	p.then((buf) => {
		if (!buf || !focused || !enabled) return;
		const src = a.createBufferSource();
		src.buffer = buf;
		src.playbackRate.value = 1 + (Math.random() * 2 - 1) * rateJitter;
		const g = a.createGain();
		g.gain.value = vol;
		src.connect(g).connect(a.destination);
		src.start(0, offset);
	});
}
let lastFoil = 0;
let lastTear = -1;
let rainNode: { src: AudioBufferSourceNode; g: GainNode } | null = null;

export const sfx = {
	/** touching a sealed foil pack (hover) - "Foil Crinkle" by DavidGallie, Pixabay Content License */
	foilTouch() {
		const now = performance.now();
		if (now - lastFoil < 220) return;
		lastFoil = now;
		sample('/audio/foil-hover.mp3', 0.75, 0.08);
	},
	/** the unique attack sound of a card (on attack / boost) */
	attack(id: string, cost: number, support = false) {
		attackSound(id, cost, support);
	},
	/** golden stud pickup */
	/** foil wrapper being torn open: one of three short lo-fi slices of a plastic-wrap recording
	 *  ("bubble wrap plastic" by dinamakan, Pixabay), each time a little different (piece, pitch, volume) */
	crinkle() {
		let i = Math.floor(Math.random() * 3);
		if (i === lastTear) i = (i + 1) % 3;
		lastTear = i;
		sample(`/audio/foil-tear-${i + 1}.mp3`, 0.6 + (Math.random() * 2 - 1) * 0.09, 0.12);
	},
	/** rewarding 8-bit "you got a card!" melody; rare = longer fanfare with a bass line */
	reward(rare = false) {
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = 0.8;
		out.connect(a.destination);
		const note = (n: number) => 523.25 * Math.pow(2, n / 12); // semitones from C5
		// a cheerful 8-bit "you got something!" tune: da da da da dada da da da da da da daaa
		// [semitones from C5, length in beats]
		const beat = 0.105;
		const melody: [number, number][] = [
			[-5, 1], [-5, 1], [0, 1], [4, 1], [2, 0.5], [4, 0.5], [7, 1], [4, 1], [5, 1], [9, 1], [7, 1], [11, 1], [12, 3]
		];
		const finale: [number, number][] = rare ? [[16, 0.75], [19, 0.75], [24, 4]] : [];
		let at = 0;
		for (const [n, len] of [...melody, ...finale]) {
			const d = len * beat;
			sweep(out, note(n), note(n), at, d * 0.92, 'square', 0.045);
			// second voice a third/sixth below on the longer notes
			if (len >= 1) sweep(out, note(n - (n % 12 === 0 ? 5 : 3)), note(n - (n % 12 === 0 ? 5 : 3)), at, d * 0.9, 'triangle', 0.035);
			at += d;
		}
		// bass: C - G - F - G - C (and a big C chord for rare cards)
		const bass: [number, number][] = [[-24, 4], [-17, 3], [-19, 2], [-17, 2], [-24, rare ? 9.5 : 4]];
		let bt = 0;
		for (const [n, len] of bass) {
			sweep(out, note(n), note(n), bt, len * beat * 0.95, 'triangle', 0.09);
			bt += len * beat;
		}
		if (rare) for (const n of [12, 16, 19]) sweep(out, note(n), note(n), at - 4 * beat, 4 * beat, 'triangle', 0.03);
	},
	coin() {
		tone(988, 0, 0.12, 'square', 0.08);
		tone(1319, 0.08, 0.3, 'square', 0.08);
	},
	/** wild encounter "!" */
	alert() {
		tone(660, 0, 0.1, 'sawtooth', 0.07);
		tone(880, 0.1, 0.1, 'sawtooth', 0.07);
		tone(1175, 0.2, 0.25, 'sawtooth', 0.07);
	},
	/** UI blip (dialog advance, card select) */
	blip() {
		tone(740, 0, 0.06, 'triangle', 0.06);
	},
	/** card placed */
	place() {
		tone(330, 0, 0.08, 'square', 0.06);
		tone(220, 0.05, 0.1, 'square', 0.05);
	},
	/** power buff */
	buff() {
		tone(660, 0, 0.1, 'triangle', 0.09);
		tone(990, 0.07, 0.18, 'triangle', 0.09);
	},
	/** power debuff / hit */
	hit() {
		tone(180, 0, 0.16, 'sawtooth', 0.09);
		tone(120, 0.06, 0.2, 'square', 0.07);
	},
	/** soft footstep (different per surface) */
	step(surface: 'road' | 'grass' | 'wood') {
		const a = ac();
		if (!a || !enabled) return;
		const len = 0.06;
		const buf = a.createBuffer(1, Math.floor(a.sampleRate * len), a.sampleRate);
		const d = buf.getChannelData(0);
		for (let i = 0; i < d.length; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / d.length, 3);
		const src = a.createBufferSource();
		src.buffer = buf;
		const f = a.createBiquadFilter();
		f.type = 'lowpass';
		f.frequency.value = surface === 'grass' ? 900 : surface === 'wood' ? 600 : 1800;
		const g = a.createGain();
		g.gain.value = surface === 'grass' ? 0.05 : 0.07;
		src.playbackRate.value = 0.9 + Math.random() * 0.2;
		src.connect(f).connect(g).connect(a.destination);
		src.start();
		if (surface === 'wood') tone(140, 0, 0.05, 'sine', 0.05);
	},
	/** a card shatters */
	shatter() {
		const a = ac();
		if (!a || !enabled) return;
		// short noise burst + falling tone
		const len = 0.35;
		const buf = a.createBuffer(1, a.sampleRate * len, a.sampleRate);
		const d = buf.getChannelData(0);
		for (let i = 0; i < d.length; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / d.length, 2);
		const src = a.createBufferSource();
		src.buffer = buf;
		const g = a.createGain();
		g.gain.value = 0.18;
		src.connect(g).connect(a.destination);
		src.start();
		tone(520, 0, 0.12, 'square', 0.06);
		tone(260, 0.05, 0.25, 'square', 0.05);
	},
	/** soft rain noise; level 0..1 (0 = off) */
	rain(level: number) {
		const a = ac();
		if (!a) return;
		if (!rainNode && level > 0.01) {
			const len = 2 * a.sampleRate;
			const buf = a.createBuffer(1, len, a.sampleRate);
			const d = buf.getChannelData(0);
			for (let i = 0; i < len; i++) d[i] = Math.random() * 2 - 1;
			const src = a.createBufferSource();
			src.buffer = buf;
			src.loop = true;
			const hp = a.createBiquadFilter();
			hp.type = 'highpass';
			hp.frequency.value = 400;
			const lp = a.createBiquadFilter();
			lp.type = 'lowpass';
			lp.frequency.value = 2200;
			const g = a.createGain();
			g.gain.value = 0;
			src.connect(hp).connect(lp).connect(g).connect(a.destination);
			src.start();
			rainNode = { src, g };
		}
		if (!rainNode) return;
		rainNode.g.gain.setTargetAtTime(enabled ? level * 0.045 : 0, a.currentTime, 0.4);
		if (level <= 0.005) {
			const n = rainNode;
			rainNode = null;
			setTimeout(() => n.src.stop(), 1500);
		}
	},
	/** rubber ball kick / bounce: a soft "boing" */
	kick(strength = 0.5) {
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = 0.4 + strength * 0.6;
		out.connect(a.destination);
		sweep(out, 260 + strength * 120, 120, 0, 0.16, 'sine', 0.14, 22, 30);
		noise(out, 0, 0.06, 0.06, 'lowpass', 900, 200, 0.7);
	},
	/** the pond fish jumps: a recorded splash out of and back into the water (fits the 0.9 s jump) */
	fishSplash() {
		sample('/audio/fish-splash.mp3', 0.8, 0.06, 0.12);
	},
	/** water splash (puddles); loud = a bigger one */
	splash(loud = false) {
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = loud ? 1 : 0.6;
		out.connect(a.destination);
		noise(out, 0, 0.35, 0.16, 'bandpass', 1800, 350, 0.8);
		sweep(out, 700, 180, 0, 0.18, 'sine', 0.12);
	},
	/** a LEGO duck quacks; pitch 1 = mother duck, higher = duckling */
	quack(pitch = 1) {
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = pitch > 1 ? 0.35 : 0.5;
		out.connect(a.destination);
		// nasal: a buzzy sawtooth through a narrow band, falling at the end
		const f = a.createBiquadFilter();
		f.type = 'bandpass';
		f.frequency.value = 1100 * pitch;
		f.Q.value = 3;
		f.connect(out);
		sweep(f, 420 * pitch, 300 * pitch, 0, 0.17, 'sawtooth', 0.5, 18, 12);
	},
	/** a firefly is caught: a soft, glassy twinkle */
	twinkle() {
		[0, 7, 12, 19].forEach((n, i) => tone(1568 * Math.pow(2, n / 12), i * 0.06, 0.25, 'sine', 0.045));
	},
	/** one piano note (a soft triangle with a few overtones, long decay) */
	piano(freq: number) {
		tone(freq, 0, 1.6, 'triangle', 0.17);
		tone(freq * 2, 0, 0.7, 'sine', 0.05);
		tone(freq * 3, 0, 0.3, 'sine', 0.025);
	},
	/** camera shutter: a quick click-clack */
	shutter() {
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = 0.7;
		out.connect(a.destination);
		noise(out, 0, 0.03, 0.25, 'highpass', 3000, 2500, 0.8);
		noise(out, 0.07, 0.045, 0.2, 'bandpass', 1800, 1200, 1.5);
		tone(2400, 0.12, 0.05, 'square', 0.02);
	},
	/** a bouncy little dance tune (plays once when you start dancing) */
	dance() {
		const mel = [0, 4, 7, 12, 7, 4, 9, 7, 5, 4, 2, 0];
		mel.forEach((n, i) => tone(523 * Math.pow(2, n / 12), i * 0.14, 0.12, 'square', 0.035));
		[0, 0.56, 1.12].forEach((s) => tone(131, s, 0.2, 'triangle', 0.08));
	},
	/** a butterfly flutters away */
	flutter() {
		[0, 4, 7, 12, 16].forEach((n, i) => tone(1046 * Math.pow(2, n / 12), i * 0.04, 0.09, 'triangle', 0.05));
	},
	/** fanfare (complete collection / victory) */
	fanfare() {
		[523, 659, 784, 1047].forEach((f, i) => tone(f, i * 0.12, 0.35, 'square', 0.07));
	},
	/** door / area change */
	door() {
		// a soft wooden door: short creak, a whoosh of air, then a muffled thud as it closes
		// (the old two-tone chime sounded like hitting metal)
		const a = ac();
		if (!a || !enabled) return;
		const out = a.createGain();
		out.gain.value = 0.8;
		out.connect(a.destination);
		sweep(out, 210, 260, 0, 0.16, 'sawtooth', 0.018, 28, 18); // creak
		noise(out, 0.02, 0.22, 0.05, 'bandpass', 500, 1400, 1.2); // air
		sweep(out, 110, 55, 0.2, 0.14, 'sine', 0.16); // thud
		noise(out, 0.2, 0.06, 0.05, 'lowpass', 600, 200); // latch
	}
};
