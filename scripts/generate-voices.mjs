// Generates character voice lines with Fish Audio (model s2.1-pro).
// Reads FISH_API_KEY from .env, collects the fixed dialog lines from the game code and writes
// static/voices/<id>.mp3 + static/voices/manifest.json. Already generated lines are skipped.
//
//   node scripts/generate-voices.mjs            (generate missing lines)
//   node scripts/generate-voices.mjs --list     (only print the lines)
//   --model=s2.1-pro-free   other Fish model (default s2.1-pro-free)   --force   regenerate existing files
import { readFileSync, writeFileSync, existsSync, mkdirSync, readdirSync } from 'node:fs';
import { OUT, VOICE_MODELS, collectVoiceLines } from './voice-lines.mjs';

// the lines are collected in scripts/voice-lines.mjs (also used by the unit test that finds lines without audio)
const { lines } = await collectVoiceLines();
const all = [...lines.entries()];
console.log(`${all.length} voice lines`);
if (process.argv.includes('--list')) {
	for (const [id, l] of all) console.log(id, l.speaker.padEnd(18), l.text);
	process.exit(0);
}

// ---------- generate ----------
const env = existsSync('.env') ? readFileSync('.env', 'utf8') : '';
const KEY = process.env.FISH_API_KEY || env.match(/^FISH_API_KEY=(.+)$/m)?.[1]?.trim();
if (!KEY) {
	console.error('FISH_API_KEY missing (.env)');
	process.exit(1);
}
mkdirSync(OUT, { recursive: true });
const MODEL = process.argv.find((a) => a.startsWith('--model='))?.slice(8) || 's2.1-pro-free';
const FORCE = process.argv.includes('--force');
console.log(`model ${MODEL}`);
const todo = all.filter(([id]) => FORCE || !existsSync(`${OUT}/${id}.mp3`));
console.log(`${todo.length} to generate`);

async function tts(text, reference_id, attempt = 0) {
	const res = await fetch('https://api.fish.audio/v1/tts', {
		method: 'POST',
		headers: { Authorization: `Bearer ${KEY}`, 'Content-Type': 'application/json', model: MODEL },
		body: JSON.stringify({ text, reference_id, format: 'mp3', mp3_bitrate: 128, latency: 'normal', normalize: true })
	});
	if ((res.status === 429 || res.status === 503) && attempt < 5) {
		await new Promise((r) => setTimeout(r, 1000 * 2 ** attempt));
		return tts(text, reference_id, attempt + 1);
	}
	if (!res.ok) throw new Error(`${res.status} ${await res.text()}`);
	return Buffer.from(await res.arrayBuffer());
}

let done = 0, failed = 0;
const queue = [...todo];
async function worker() {
	while (queue.length) {
		const [id, l] = queue.shift();
		try {
			const mp3 = await tts(l.text, VOICE_MODELS[l.speaker]);
			writeFileSync(`${OUT}/${id}.mp3`, mp3);
			done++;
			console.log(`ok  ${l.speaker}: ${l.text.slice(0, 60)}`);
		} catch (e) {
			failed++;
			console.error(`ERR ${l.speaker}: ${l.text.slice(0, 40)} -> ${e.message.slice(0, 200)}`);
		}
	}
}
await Promise.all([worker(), worker(), worker()]);
const manifest = readdirSync(OUT).filter((f) => f.endsWith('.mp3')).map((f) => f.replace('.mp3', ''));
writeFileSync(`${OUT}/manifest.json`, JSON.stringify(manifest));
console.log(`done: ${done} generated, ${failed} failed, ${manifest.length} in manifest`);
