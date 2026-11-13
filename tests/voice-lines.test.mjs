// Finds character dialog lines that have no generated voice audio yet.
//   npm test
// Fix a failure with:  node scripts/generate-voices.mjs   (or give a changing line a SPOKEN_AS entry in voice.ts)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { OUT, VOICE_MODELS, collectVoiceLines, voiceId } from '../scripts/voice-lines.mjs';
import * as gameVoice from '../src/lib/game/voice.ts';

const { lines, dynamic } = await collectVoiceLines();
const manifest = new Set(existsSync(`${OUT}/manifest.json`) ? JSON.parse(readFileSync(`${OUT}/manifest.json`, 'utf8')) : []);
const hasAudio = (id) => manifest.has(id) && existsSync(`${OUT}/${id}.mp3`);
const fmt = (list) => list.map((l) => `\n  - ${l.speaker}: "${l.text}"  (${l.where})`).join('');

test('the game and the generator compute the same voice ids', () => {
	assert.deepEqual(gameVoice.VOICE_MODELS, VOICE_MODELS, 'VOICE_MODELS differ between voice.ts and scripts/voice-lines.mjs');
	for (const [speaker] of Object.entries(VOICE_MODELS)) {
		const text = '*yawn* Hello — world! 🎉';
		assert.equal(gameVoice.voiceId(speaker, text), voiceId(speaker, text), `speakable()/voiceId() differ for ${speaker}`);
	}
});

test('there are voiced dialog lines at all', () => {
	assert.ok(lines.size > 20, `only ${lines.size} lines found: did the dialog code change?`);
});

test('every fixed dialog line has generated audio', () => {
	const missing = [...lines.entries()].filter(([id]) => !hasAudio(id)).map(([, l]) => l);
	assert.equal(missing.length, 0, `${missing.length} dialog line(s) without audio (run: node scripts/generate-voices.mjs):${fmt(missing)}`);
});

test('every dialog line with changing text has a fixed spoken version with audio', () => {
	const problems = [];
	for (const d of dynamic) {
		const spoken = gameVoice.spokenText(d.sample);
		if (spoken === d.sample) problems.push({ ...d, text: `${d.template} (no SPOKEN_AS entry in voice.ts)` });
		else if (!hasAudio(voiceId(d.speaker, spoken))) problems.push({ ...d, text: `${spoken} (spoken version not generated)` });
	}
	assert.equal(problems.length, 0, `${problems.length} changing dialog line(s) without audio:${fmt(problems)}`);
});
