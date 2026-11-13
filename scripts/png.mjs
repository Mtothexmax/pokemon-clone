// Minimal PNG reader for the scripts (no dependencies): 8-bit RGBA / RGB / palette, not interlaced.
import { readFileSync } from 'node:fs';
import { inflateSync } from 'node:zlib';

/** -> { w, h, rgba } */
export function decodePng(file) {
	const d = readFileSync(file);
	let i = 8, w = 0, h = 0, type = 0, plte = null, trns = null;
	const idat = [];
	while (i < d.length) {
		const n = d.readUInt32BE(i), t = d.toString('ascii', i + 4, i + 8), c = d.subarray(i + 8, i + 8 + n);
		if (t === 'IHDR') {
			w = c.readUInt32BE(0);
			h = c.readUInt32BE(4);
			type = c[9];
			if (c[8] !== 8 || c[12] !== 0) throw new Error(`${file}: only 8-bit, non-interlaced PNGs`);
		} else if (t === 'PLTE') plte = c;
		else if (t === 'tRNS') trns = c;
		else if (t === 'IDAT') idat.push(c);
		i += 12 + n;
	}
	const bpp = type === 6 ? 4 : type === 2 ? 3 : type === 3 ? 1 : 0;
	if (!bpp) throw new Error(`${file}: colour type ${type} not supported`);
	const raw = inflateSync(Buffer.concat(idat));
	const stride = w * bpp, px = Buffer.alloc(h * stride);
	for (let y = 0; y < h; y++) {
		const f = raw[y * (stride + 1)], src = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
		for (let x = 0; x < stride; x++) {
			const a = x >= bpp ? px[y * stride + x - bpp] : 0, b = y ? px[(y - 1) * stride + x] : 0, c = x >= bpp && y ? px[(y - 1) * stride + x - bpp] : 0;
			const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
			const pred = f === 1 ? a : f === 2 ? b : f === 3 ? (a + b) >> 1 : f === 4 ? (pa <= pb && pa <= pc ? a : pb <= pc ? b : c) : 0;
			px[y * stride + x] = (src[x] + pred) & 255;
		}
	}
	const rgba = Buffer.alloc(w * h * 4);
	for (let k = 0; k < w * h; k++) {
		if (bpp === 4) px.copy(rgba, k * 4, k * 4, k * 4 + 4);
		else if (bpp === 3) (px.copy(rgba, k * 4, k * 3, k * 3 + 3), (rgba[k * 4 + 3] = 255));
		else {
			const j = px[k];
			plte.copy(rgba, k * 4, j * 3, j * 3 + 3);
			rgba[k * 4 + 3] = trns && j < trns.length ? trns[j] : 255;
		}
	}
	return { w, h, rgba };
}

