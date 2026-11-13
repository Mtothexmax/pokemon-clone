// Character voice lines (pre-generated with Fish Audio by scripts/generate-voices.mjs).
// The API key is only used by that script; the game just plays the generated MP3s.

/** speaker name in dialogs -> Fish Audio voice model id */
export const VOICE_MODELS: Record<string, string> = {
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

/** text as it is sent to the voice model: *yawn* -> [yawn] (Fish Audio sound tag), no emoji etc. */
export function speakable(text: string) {
	return text
		.replace(/\*([^*]+)\*/g, '[$1]')
		.replace(/[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}]/gu, '')
		.replace(/\s*[—–]\s*/g, ', ')
		.replace(/\s+/g, ' ')
		.trim();
}

/** stable id for a line (FNV-1a 32 bit) — must match scripts/generate-voices.mjs */
export function voiceId(speaker: string, text: string) {
	const s = `${VOICE_MODELS[speaker] ?? speaker}|${speakable(text)}`;
	let h = 0x811c9dc5;
	for (let i = 0; i < s.length; i++) {
		h ^= s.charCodeAt(i);
		h = Math.imul(h, 0x01000193);
	}
	return (h >>> 0).toString(16).padStart(8, '0');
}

let available: Set<string> | null = null;
let current: HTMLAudioElement | null = null;
let enabled = true;
try {
	enabled = localStorage.getItem('brickhollow-voices') !== 'off';
} catch {
	/* storage unavailable */
}

export async function loadVoices() {
	try {
		const r = await fetch(`/voices/manifest.json?t=${Date.now()}`);
		available = new Set(r.ok ? ((await r.json()) as string[]) : []);
	} catch {
		available = new Set();
	}
}

export function voicesEnabled() {
	return enabled;
}
export function setVoicesEnabled(on: boolean) {
	enabled = on;
	if (!on) stopVoice();
	try {
		localStorage.setItem('brickhollow-voices', on ? 'on' : 'off');
	} catch {
		/* ignore */
	}
}

let focused = true;
/** mutes a running voice line while the tab has no focus (and skips new ones) */
export function setVoiceFocused(on: boolean) {
	focused = on;
	if (current) current.muted = !on;
}

export function stopVoice() {
	current?.pause();
	current = null;
}

/** lines whose shown text contains a changing number are spoken as one fixed version */
const SPOKEN_AS: [RegExp, string][] = [[/^\d+ wins? already\? I'm so proud of you!$/, "So many wins already? I'm so proud of you!"]];
export function spokenText(text: string) {
	for (const [re, fixed] of SPOKEN_AS) if (re.test(text)) return fixed;
	return text;
}

/**
 * plays the generated line if it exists; silently does nothing otherwise.
 * Returns true if it plays; `onEnd` then runs when the line is over (not when it is stopped).
 */
export function playVoice(speaker: string, text: string, onEnd?: () => void) {
	stopVoice();
	if (!enabled || !focused || !available || !VOICE_MODELS[speaker]) return false;
	const id = voiceId(speaker, spokenText(text));
	if (!available.has(id)) return false;
	const a = new Audio(`/voices/${id}.mp3`);
	a.volume = 0.95;
	a.onended = () => {
		if (current !== a) return;
		current = null;
		onEnd?.();
	};
	a.play().catch(() => {
		// autoplay blocked: still move on after a moment
		if (onEnd) setTimeout(() => current === a && onEnd(), 2500);
	});
	current = a;
	return true;
}
