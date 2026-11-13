// Converts the game's .bin models back to OBJ (with vertex colours) e.g. to look at them in Blender.
//   node scripts/bin-to-obj.mjs out.obj static/models/a.bin [static/models/b.bin ...]
import { readFileSync, writeFileSync } from 'node:fs';
const [, , out, ...files] = process.argv;
const lines = ['# exported from the game models (scripts/bin-to-obj.mjs)'];
let base = 0;
for (const f of files) {
	const buf = readFileSync(f);
	const ab = buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength);
	const [n, ni, flags] = new Uint32Array(ab, 0, 3);
	let o = 12;
	const pos = new Float32Array(ab, o, n * 3); o += n * 12;
	o += n * 12; // normals (Blender recomputes them)
	let col = null;
	if (flags & 1) { col = new Uint8Array(ab, o, n * 4); o += n * 4; }
	if (flags & 2) o += n * 8;
	const idx = new Uint32Array(ab.slice(o, o + ni * 4));
	lines.push(`o ${f.split('/').pop().replace('.bin', '')}`);
	for (let i = 0; i < n; i++) {
		const p = [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]].map((v) => v.toFixed(5)).join(' ');
		lines.push(col ? `v ${p} ${(col[i * 4] / 255).toFixed(3)} ${(col[i * 4 + 1] / 255).toFixed(3)} ${(col[i * 4 + 2] / 255).toFixed(3)}` : `v ${p}`);
	}
	for (let i = 0; i < ni; i += 3) lines.push(`f ${idx[i] + 1 + base} ${idx[i + 1] + 1 + base} ${idx[i + 2] + 1 + base}`);
	base += n;
	console.log(f, n, 'verts', ni / 3, 'tris');
}
writeFileSync(out, lines.join('\n'));
console.log('written', out);
