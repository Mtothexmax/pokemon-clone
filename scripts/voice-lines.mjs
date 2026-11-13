// Collects every character dialog line in the game that should have a voice.
// Shared by scripts/generate-voices.mjs (makes the audio) and tests/voice-lines.test.mjs (finds lines without audio).
// Paths are relative to the project root (run from there).
import { readFileSync } from 'node:fs';

export const OUT = 'static/voices';

/** speaker name -> Fish Audio voice model (keep in sync with src/lib/game/voice.ts) */
export const VOICE_MODELS = {
	You: '803ea6222c1d434ba29fb7bd057e2514',
	Dex: 'adb8cf792d014175bfd0934698360cea',
	'Prof. Hazel Quill': 'e5b455367c1c463198ab6dc04c411b28',
	Mom: '4610e0629d3142eda3e2bda78e328372',
	Lumi: 'ef6d0310ff384baa87bc37f943402244',
	Pip: '116eacdfd57e4ea0b193b93a76184499',
	// people on TV (see src/lib/game/tvScripts.ts)
	'BBN Anchor': '41184a9e4cd84301b8bd59f4eed99635',
	'BBN Reporter': '7b6a7866c24d4f8ca1940c8ced3929ab',
	Narrator: 'c3958f083c2d407982263c77cfa831e3'
};
// keep in sync with src/lib/game/voice.ts
export const speakable = (text) =>
	text
		.replace(/\*([^*]+)\*/g, '[$1]') // *yawn* -> [yawn]: Fish Audio sound tag
		.replace(/[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}]/gu, '')
		.replace(/\s*[—–]\s*/g, ', ')
		.replace(/\s+/g, ' ')
		.trim();
export const voiceId = (speaker, text) => {
	const s = `${VOICE_MODELS[speaker] ?? speaker}|${speakable(text)}`;
	let h = 0x811c9dc5;
	for (let i = 0; i < s.length; i++) {
		h ^= s.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return (h >>> 0).toString(16).padStart(8, '0');
};

/**
 * @returns {Promise<{
 *   lines: Map<string, { speaker: string; text: string; where: string }>;
 *   dynamic: { speaker: string; template: string; sample: string; where: string }[];
 * }>}
 * `lines`: fixed lines (id -> line). `dynamic`: lines whose text changes at runtime (e.g. "${wins} wins"),
 * which can only be voiced through a fixed spoken version (SPOKEN_AS in voice.ts).
 */
export async function collectVoiceLines() {
	// game constants that may appear inside template strings (${...}); resolved so those lines get a voice too
	const cards = await import('../src/lib/game/cards.ts');
	const townSrc = readFileSync('src/lib/three/town.ts', 'utf8');
	const studsAt = townSrc.indexOf('= [', townSrc.indexOf('export const GOLDEN_STUDS'));
	const CONSTS = {
		'POOL.length': cards.POOL.length,
		'STARTER_COLLECTION.length': cards.STARTER_COLLECTION.length,
		'GOLDEN_STUDS.length': (townSrc.slice(studsAt, townSrc.indexOf('];', studsAt)).match(/\{\s*id:/g) || []).length
	};
	/** reads a JS string literal starting at i; returns { value (null if it has unknown ${}), sample, end } */
	function readString(src, i) {
		const q = src[i];
		let j = i + 1, out = '', sample = '', dyn = false;
		while (j < src.length && src[j] !== q) {
			if (src[j] === '\\') {
				const ch = src[j + 1] === 'n' ? '\n' : src[j + 1];
				out += ch;
				sample += ch;
				j += 2;
				continue;
			}
			if (q === '`' && src[j] === '$' && src[j + 1] === '{') {
				const close = src.indexOf('}', j);
				const expr = src.slice(j + 2, close).trim();
				if (expr in CONSTS) {
					out += CONSTS[expr];
					sample += CONSTS[expr];
				} else {
					dyn = true;
					// a plausible runtime value: ternaries (plural endings etc.) -> '', anything else -> 2
					sample += expr.includes('?') ? '' : '2';
				}
				j = close + 1;
				continue;
			}
			out += src[j];
			sample += src[j];
			j++;
		}
		return { value: dyn ? null : out, sample, raw: src.slice(i, j + 1), end: j + 1 };
	}
	const lines = new Map();
	const dynamic = [];
	const lineNo = (src, i) => src.slice(0, i).split('\n').length;
	const add = (speaker, text, where) => {
		if (!VOICE_MODELS[speaker] || !speakable(text)) return;
		lines.set(voiceId(speaker, text), { speaker, text: speakable(text), where });
	};
	/** string literals inside the call starting at `open` (index of "("), skipping nested say(...) calls and object keys */
	function literalsInCall(src, open, speaker, file) {
		let depth = 0, i = open;
		while (i < src.length) {
			const c = src[i];
			if (c === "'" || c === '"' || c === '`') {
				const before = src.slice(Math.max(0, i - 12), i);
				const r = readString(src, i);
				const isKey = /(label|icon|title|action)\s*:\s*$/.test(before) || /\(\s*$/.test(before);
				if (!isKey && depth > 0) {
					const where = `${file}:${lineNo(src, i)}`;
					if (r.value !== null) add(speaker, r.value, where);
					else if (VOICE_MODELS[speaker]) dynamic.push({ speaker, template: r.raw, sample: r.sample, where });
				}
				i = r.end;
				continue;
			}
			if (depth > 0 && src.startsWith('say(', i) && i !== open - 3) {
				// nested say(): handled on its own
				let d = 0, k = i + 3;
				for (; k < src.length; k++) {
					if (src[k] === '(') d++;
					else if (src[k] === ')' && --d === 0) break;
				}
				i = k + 1;
				continue;
			}
			if (c === '(') depth++;
			else if (c === ')' && --depth === 0) break;
			i++;
		}
	}
	const PAGE = 'src/routes/+page.svelte';
	const page = readFileSync(PAGE, 'utf8');
	for (const m of page.matchAll(/say\(\s*'([^']+)'\s*,/g)) literalsInCall(page, m.index + 3, m[1], PAGE);

	// battle taunts: rival = Dex, practice = Lumi
	const BV = 'src/lib/components/BattleView.svelte';
	const battle = readFileSync(BV, 'utf8');
	for (const [key, speaker] of [['rival', 'Dex'], ['practice', 'Lumi']]) {
		const start = battle.indexOf(`\t\t${key}: {`);
		if (start < 0) continue;
		const end = battle.indexOf('\n\t\t}', start);
		for (let i = start; i < end; i++) {
			if (battle[i] === "'" || battle[i] === '`') {
				const r = readString(battle, i);
				const where = `${BV}:${lineNo(battle, i)}`;
				if (r.value !== null) {
					if (!/^(start|turn|snap|win|lose)$/.test(r.value)) add(speaker, r.value, where);
				} else dynamic.push({ speaker, template: r.raw, sample: r.sample, where });
				i = r.end - 1;
			}
		}
	}

	// lines that depend on player progress: one variant per possible value (keep in sync with +page.svelte).
	// Their templates count as voiced, because every variant is in `lines`.
	const VARIANT_TEMPLATES = [/^`You have \$\{owned\} of \$\{POOL\.length\} Brick Beast cards\./];
	for (let k = dynamic.length - 1; k >= 0; k--) if (VARIANT_TEMPLATES.some((re) => re.test(dynamic[k].template))) dynamic.splice(k, 1);
	const total = CONSTS['POOL.length'];
	for (let owned = 0; owned <= total; owned++)
		add('Prof. Hazel Quill', `You have ${owned} of ${total} Brick Beast cards. ${owned >= total ? 'A complete collection — incredible!' : 'Keep exploring Route 1!'}`, 'collection count variants');
	// the people on TV
	const tv = await import('../src/lib/game/tvScripts.ts');
	for (const [show, script] of Object.entries(tv.TV_SCRIPTS)) for (const l of script) add(l.speaker, l.text, `tvScripts.ts (${show})`);
	// fixed spoken versions of lines with a changing number (see SPOKEN_AS in src/lib/game/voice.ts)
	add('Mom', "So many wins already? I'm so proud of you!", 'SPOKEN_AS (voice.ts)');

	return { lines, dynamic };
}
