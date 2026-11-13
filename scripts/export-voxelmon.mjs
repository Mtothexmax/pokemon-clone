// Writes every Brick Beast's pixel creature as an editable Voxelmon model: src/lib/voxelmon/models/<id>.js
// (palette + rows of characters). The game draws the 3D Voxelmon from these files (see src/lib/voxelmon/draw.js).
//
//   node scripts/export-voxelmon.mjs              (all beasts; existing model files are kept)
//   node scripts/export-voxelmon.mjs titanoak     (just these)
//   --force                                       overwrite existing (hand-edited!) model files
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'node:fs';
import { decodePng } from './png.mjs';
import { CARD_MAP } from '../src/lib/game/cards.ts';

// which sprite belongs to which card: read from the SPRITES table in art.ts (id: ['sheet', column, row])
const artSrc = readFileSync('src/lib/game/art.ts', 'utf8');
const table = artSrc.slice(artSrc.indexOf('export const SPRITES'), artSrc.indexOf('};', artSrc.indexOf('export const SPRITES')));
const SPRITES = Object.fromEntries([...table.matchAll(/(\w+): \['(\w+)', (\d+), (\d+)\]/g)].map((m) => [m[1], [m[2], +m[3], +m[4]]]));

const OUT = 'src/lib/voxelmon/models';

const sheets = new Map();
const sheet = (id) => (sheets.has(id) ? sheets.get(id) : sheets.set(id, decodePng(`static/sprites/${id}.png`)).get(id));
// characters for the palette ('.' is empty)
const KEYS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#$%&*+-=@~^!?:;<>/|';

const only = process.argv.slice(2).filter((a) => !a.startsWith('--'));
const force = process.argv.includes('--force');
mkdirSync(OUT, { recursive: true });
let written = 0, kept = 0;
for (const [id, [sheetId, col, row]] of Object.entries(SPRITES)) {
	if (only.length && !only.includes(id)) continue;
	const file = `${OUT}/${id}.js`;
	// hand-made models (build functions) are never overwritten, not even with --force
	if (existsSync(file) && (!force || readFileSync(file, 'utf8').includes('build(v)'))) {
		kept++;
		continue;
	}
	const s = sheet(sheetId);
	// the 64x64 cell, cropped to the creature (same as the card art)
	const at = (x, y) => ((row * 64 + y) * s.w + col * 64 + x) * 4;
	let x0 = 64, y0 = 64, x1 = -1, y1 = -1;
	for (let y = 0; y < 64; y++)
		for (let x = 0; x < 64; x++)
			if (s.rgba[at(x, y) + 3] > 100) [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
	if (x1 < 0) continue;
	const palette = new Map();
	const rows = [];
	for (let y = y0; y <= y1; y++) {
		let line = '';
		for (let x = x0; x <= x1; x++) {
			const p = at(x, y);
			if (s.rgba[p + 3] <= 100) {
				line += '.';
				continue;
			}
			const hex = '#' + [0, 1, 2].map((k) => s.rgba[p + k].toString(16).padStart(2, '0')).join('');
			if (!palette.has(hex)) palette.set(hex, KEYS[palette.size] ?? '?');
			line += palette.get(hex);
		}
		rows.push(line.replace(/\.+$/, ''));
	}
	const name = CARD_MAP[id]?.name ?? id;
	const pal = [...palette].map(([hex, k]) => `\t\t${k}: '${hex}'`).join(',\n');
	const src = `// ${name} as a Voxelmon. The 3D model is drawn from this pixel picture (see ../draw.js):
// every character is one column of voxels, '.' = empty; columns get thicker towards the middle of the body.
// Edit the pixels or colours freely. Optional: \`depth\` = rows of digits 0-9 (half thickness per pixel) to shape it by hand.
export default {
	name: '${name.replace(/'/g, "\\'")}',
	palette: {
${pal}
	},
	pixels: [
${rows.map((r) => `\t\t'${r}'`).join(',\n')}
	]
};
`;
	writeFileSync(file, src);
	written++;
}
console.log(`${written} model files written, ${kept} kept (use --force to overwrite)`);
