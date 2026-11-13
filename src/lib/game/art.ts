// Card portraits: Openmon pixel-art sprites (static/sprites), with procedural brick creatures as fallback.
import { CARD_MAP, KIND_INFO } from './cards';
import { MODELS } from '../voxelmon/index';
import { drawPixelPicture } from '../voxelmon/draw.js';
import { voxelPortrait } from '../three/voxel';

const cache = new Map<string, string>();

function shade(hex: string, amt: number) {
	const n = parseInt(hex.slice(1), 16);
	const r = Math.max(0, Math.min(255, (n >> 16) + amt));
	const g = Math.max(0, Math.min(255, ((n >> 8) & 255) + amt));
	const b = Math.max(0, Math.min(255, (n & 255) + amt));
	return `rgb(${r},${g},${b})`;
}

/** A brick with studs on top. */
function brick(x: number, y: number, w: number, h: number, color: string, studs = true) {
	let s = `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="3" fill="${color}" stroke="${shade(color, -60)}" stroke-width="1.5"/>`;
	s += `<rect x="${x + 2}" y="${y + 2}" width="${w - 4}" height="${Math.min(4, h / 4)}" rx="2" fill="#fff" opacity=".25"/>`;
	if (studs) {
		const n = Math.max(1, Math.round(w / 14));
		const step = w / n;
		for (let i = 0; i < n; i++) {
			const cx = x + step * (i + 0.5);
			s += `<rect x="${cx - 4.5}" y="${y - 4}" width="9" height="5" rx="1.5" fill="${color}" stroke="${shade(color, -60)}" stroke-width="1.2"/>`;
		}
	}
	return s;
}

/** Procedural brick-creature portrait (fallback when no sprite is assigned). */
function brickArt(id: string): string {
	const hit = cache.get(id);
	if (hit) return hit;
	const def = CARD_MAP[id];
	const a = def.art;
	const t = { bg: KIND_INFO[def.kind].color };
	const big = a.big ? 1.12 : 1;
	const W = 120, H = 100;
	const bw = 56 * big, bh = 30 * big;
	const bx = (W - bw) / 2, by = 92 - bh - 8;
	const hw = 46 * big, hh = 28 * big;
	const hx = (W - hw) / 2, hy = by - hh - 2;
	const dark = shade(a.body, -50);

	let back = '';
	let front = '';
	// features
	switch (a.feature) {
		case 'ears':
			back += brick(hx + 2, hy - 18, 12, 20, a.body, false) + brick(hx + hw - 14, hy - 18, 12, 20, a.body, false);
			back += `<rect x="${hx + 5}" y="${hy - 14}" width="6" height="12" rx="2" fill="${a.accent}"/><rect x="${hx + hw - 11}" y="${hy - 14}" width="6" height="12" rx="2" fill="${a.accent}"/>`;
			break;
		case 'horns':
			back += `<polygon points="${hx + 6},${hy + 2} ${hx - 4},${hy - 20} ${hx + 16},${hy + 2}" fill="${a.accent}" stroke="${shade(a.accent, -60)}" stroke-width="1.5"/>`;
			back += `<polygon points="${hx + hw - 6},${hy + 2} ${hx + hw + 4},${hy - 20} ${hx + hw - 16},${hy + 2}" fill="${a.accent}" stroke="${shade(a.accent, -60)}" stroke-width="1.5"/>`;
			break;
		case 'flame':
			back += `<path d="M${W / 2 - 12},${hy + 4} C${W / 2 - 20},${hy - 14} ${W / 2 - 4},${hy - 18} ${W / 2},${hy - 30} C${W / 2 + 6},${hy - 16} ${W / 2 + 20},${hy - 12} ${W / 2 + 12},${hy + 4} Z" fill="#ffb020" stroke="#d9361b" stroke-width="2"/>`;
			break;
		case 'leaf':
			back += `<path d="M${W / 2},${hy + 2} C${W / 2 - 24},${hy - 6} ${W / 2 - 22},${hy - 26} ${W / 2 - 4},${hy - 22} C${W / 2 - 2},${hy - 10} ${W / 2},${hy - 4} ${W / 2},${hy + 2}Z" fill="${a.accent}" stroke="${shade(a.accent, -50)}" stroke-width="1.5"/>`;
			back += `<path d="M${W / 2},${hy + 2} C${W / 2 + 22},${hy - 4} ${W / 2 + 22},${hy - 22} ${W / 2 + 6},${hy - 20} C${W / 2 + 2},${hy - 10} ${W / 2},${hy - 4} ${W / 2},${hy + 2}Z" fill="${shade(a.accent, 25)}" stroke="${shade(a.accent, -50)}" stroke-width="1.5"/>`;
			break;
		case 'fins':
			back += `<polygon points="${bx - 2},${by + 6} ${bx - 18},${by - 4} ${bx - 12},${by + 20}" fill="${a.accent}"/><polygon points="${bx + bw + 2},${by + 6} ${bx + bw + 18},${by - 4} ${bx + bw + 12},${by + 20}" fill="${a.accent}"/>`;
			back += `<polygon points="${W / 2 - 8},${hy + 2} ${W / 2},${hy - 16} ${W / 2 + 8},${hy + 2}" fill="${a.accent}"/>`;
			break;
		case 'wings':
			back += `<path d="M${bx + 4},${by + 6} Q${bx - 34},${by - 40} ${bx - 26},${by + 18} Z" fill="${a.accent}" opacity=".9" stroke="${shade(a.accent, -50)}" stroke-width="1.5"/>`;
			back += `<path d="M${bx + bw - 4},${by + 6} Q${bx + bw + 34},${by - 40} ${bx + bw + 26},${by + 18} Z" fill="${a.accent}" opacity=".9" stroke="${shade(a.accent, -50)}" stroke-width="1.5"/>`;
			break;
		case 'antenna':
			back += `<line x1="${W / 2 - 10}" y1="${hy}" x2="${W / 2 - 18}" y2="${hy - 18}" stroke="${a.accent}" stroke-width="3"/><circle cx="${W / 2 - 18}" cy="${hy - 20}" r="5" fill="${a.accent}"/>`;
			back += `<line x1="${W / 2 + 10}" y1="${hy}" x2="${W / 2 + 18}" y2="${hy - 18}" stroke="${a.accent}" stroke-width="3"/><circle cx="${W / 2 + 18}" cy="${hy - 20}" r="5" fill="${a.accent}"/>`;
			break;
		case 'spikes':
			for (let i = 0; i < 4; i++) {
				const x = hx + 6 + i * ((hw - 12) / 3);
				back += `<polygon points="${x - 6},${hy + 3} ${x},${hy - 12} ${x + 6},${hy + 3}" fill="${a.accent}" stroke="${shade(a.accent, -50)}" stroke-width="1.2"/>`;
			}
			break;
		case 'ghost':
			back += `<path d="M${bx - 8},${by + bh} Q${bx - 16},${by - 20} ${W / 2},${hy - 16} Q${bx + bw + 16},${by - 20} ${bx + bw + 8},${by + bh} l-8,-8 l-8,8 l-8,-8 l-8,8 l-8,-8 l-8,8 Z" fill="${a.belly}" opacity=".55"/>`;
			break;
		case 'shell':
			back += `<path d="M${bx - 10},${by + bh} Q${W / 2},${by - 34} ${bx + bw + 10},${by + bh} Z" fill="${a.accent}" stroke="${shade(a.accent, -40)}" stroke-width="2"/>`;
			break;
		case 'tentacles':
			for (let i = 0; i < 4; i++) {
				const x = bx + 6 + i * ((bw - 12) / 3);
				front += `<path d="M${x},${by + bh - 4} q${i % 2 ? 8 : -8},10 0,18" stroke="${a.body}" stroke-width="7" fill="none" stroke-linecap="round"/>`;
			}
			back += `<circle cx="${hx}" cy="${hy + 6}" r="7" fill="${a.accent}"/><circle cx="${hx + hw}" cy="${hy + 6}" r="7" fill="${a.accent}"/>`;
			break;
		case 'crest':
			back += `<path d="M${hx + 8},${hy + 2} Q${W / 2},${hy - 28} ${hx + hw - 8},${hy + 2} Z" fill="${a.accent}" stroke="${shade(a.accent, -50)}" stroke-width="1.5"/>`;
			for (let i = 0; i < 3; i++) back += `<circle cx="${hx + 14 + i * ((hw - 28) / 2)}" cy="${hy - 8}" r="3" fill="#fff" opacity=".8"/>`;
			break;
	}

	// legs
	const legs = brick(bx + 4, by + bh - 2, 14, 10, dark, false) + brick(bx + bw - 18, by + bh - 2, 14, 10, dark, false);
	// body + belly
	let body = brick(bx, by, bw, bh, a.body);
	body += `<rect x="${bx + bw * 0.28}" y="${by + 7}" width="${bw * 0.44}" height="${bh - 11}" rx="3" fill="${a.belly}"/>`;
	// head
	let head = brick(hx, hy, hw, hh, a.body);
	// face
	const ey = hy + hh * 0.48;
	const ex1 = W / 2 - 10, ex2 = W / 2 + 10;
	switch (a.eyes ?? 'round') {
		case 'angry':
			head += `<circle cx="${ex1}" cy="${ey}" r="4.5" fill="#111"/><circle cx="${ex2}" cy="${ey}" r="4.5" fill="#111"/>`;
			head += `<line x1="${ex1 - 7}" y1="${ey - 9}" x2="${ex1 + 4}" y2="${ey - 5}" stroke="#111" stroke-width="2.5"/><line x1="${ex2 + 7}" y1="${ey - 9}" x2="${ex2 - 4}" y2="${ey - 5}" stroke="#111" stroke-width="2.5"/>`;
			break;
		case 'sleepy':
			head += `<path d="M${ex1 - 5},${ey} q5,4 10,0" stroke="#111" stroke-width="2.5" fill="none"/><path d="M${ex2 - 5},${ey} q5,4 10,0" stroke="#111" stroke-width="2.5" fill="none"/>`;
			break;
		case 'happy':
			head += `<path d="M${ex1 - 5},${ey + 2} q5,-7 10,0" stroke="#111" stroke-width="2.8" fill="none"/><path d="M${ex2 - 5},${ey + 2} q5,-7 10,0" stroke="#111" stroke-width="2.8" fill="none"/>`;
			break;
		default:
			head += `<circle cx="${ex1}" cy="${ey}" r="5" fill="#111"/><circle cx="${ex2}" cy="${ey}" r="5" fill="#111"/><circle cx="${ex1 + 1.5}" cy="${ey - 1.5}" r="1.6" fill="#fff"/><circle cx="${ex2 + 1.5}" cy="${ey - 1.5}" r="1.6" fill="#fff"/>`;
	}
	head += `<path d="M${W / 2 - 5},${ey + 8} q5,4 10,0" stroke="#111" stroke-width="2" fill="none"/>`;
	head += `<circle cx="${ex1 - 9}" cy="${ey + 7}" r="3" fill="#ff6a8a" opacity=".5"/><circle cx="${ex2 + 9}" cy="${ey + 7}" r="3" fill="#ff6a8a" opacity=".5"/>`;

	const bg = `<defs><radialGradient id="g-${id}" cx="50%" cy="40%" r="75%"><stop offset="0" stop-color="${shade(t.bg, 60)}"/><stop offset="1" stop-color="${t.bg}"/></radialGradient></defs>
<rect width="${W}" height="${H}" fill="url(#g-${id})"/>
<g opacity=".18">${Array.from({ length: 8 }, (_, i) => `<circle cx="${8 + i * 15}" cy="${H - 6}" r="4" fill="#000"/>`).join('')}</g>
<ellipse cx="${W / 2}" cy="${H - 7}" rx="${bw * 0.6}" ry="4" fill="#000" opacity=".25"/>`;

	const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="240" height="200" viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid slice">${bg}${back}${legs}${body}${head}${front}</svg>`;
	const url = `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
	cache.set(id, url);
	return url;
}

// ---------------- Openmon sprites ----------------
// [sheet, column, row] in 64x64 cells (see static/sprites/*.png)
type SheetId = 'set1' | 'set3' | 'set4' | 'set5' | 'set6' | 'shibu';
export const SPRITES: Record<string, [SheetId, number, number]> = {
	pebbit: ['set1', 1, 0],
	flicker: ['set5', 0, 5],
	drizzlet: ['set4', 1, 0],
	sproutle: ['set1', 2, 2],
	zapling: ['set1', 1, 1],
	quickhop: ['shibu', 0, 0],
	voltail: ['shibu', 3, 0],
	cinderpup: ['set6', 2, 4],
	mosscub: ['set1', 3, 1],
	gloomoth: ['set6', 2, 2],
	gustling: ['set5', 3, 5],
	sparkmole: ['set6', 0, 0],
	tickbot: ['set1', 3, 0],
	tidalotl: ['set6', 5, 3],
	rockodile: ['set6', 2, 1],
	fungoo: ['shibu', 0, 3],
	zephyrrow: ['set5', 4, 5],
	glimmerling: ['set5', 4, 4],
	echowl: ['set5', 0, 2],
	buildbeaver: ['set5', 0, 3],
	thundrake: ['set4', 2, 1],
	coralyth: ['set6', 4, 2],
	emberhorn: ['set6', 3, 4],
	bramblewolf: ['shibu', 2, 1],
	phantomule: ['set4', 3, 1],
	boomshroom: ['set3', 2, 2],
	glaciomoth: ['set5', 2, 4],
	magmaw: ['set6', 2, 3],
	lumifox: ['shibu', 2, 2],
	cragoliath: ['set5', 1, 3],
	clockcrab: ['set1', 4, 3],
	titanoak: ['set6', 1, 4],
	stormaw: ['set5', 3, 2],
	abyssquid: ['set5', 1, 0],
	goldbrick: ['set3', 5, 1],
	brickling: ['set5', 4, 1]
};

const sheets = new Map<SheetId, HTMLImageElement>();
let sheetPromise: Promise<void> | null = null;

/** Loads all sprite sheets once (call before showing cards). */
export function preloadSprites() {
	sheetPromise ??= Promise.all(
		(['set1', 'set3', 'set4', 'set5', 'set6', 'shibu'] as SheetId[]).map(
			(id) =>
				new Promise<void>((res) => {
					const img = new Image();
					img.onload = () => {
						sheets.set(id, img);
						res();
					};
					img.onerror = () => res();
					img.src = `/sprites/${id}.png`;
				})
		)
	).then(() => {
		cache.clear();
	});
	return sheetPromise;
}

function spriteArt(id: string): string | null {
	const spec = SPRITES[id];
	if (!spec) return null;
	const img = sheets.get(spec[0]);
	if (!img) return null;
	const [, col, row] = spec;
	// tight bounds of the sprite inside its 64x64 cell
	const cell = document.createElement('canvas');
	cell.width = cell.height = 64;
	const cg = cell.getContext('2d', { willReadFrequently: true })!;
	cg.drawImage(img, col * 64, row * 64, 64, 64, 0, 0, 64, 64);
	const px = cg.getImageData(0, 0, 64, 64).data;
	let x0 = 64, y0 = 64, x1 = -1, y1 = -1;
	for (let y = 0; y < 64; y++)
		for (let x = 0; x < 64; x++)
			if (px[(y * 64 + x) * 4 + 3] > 20) {
				x0 = Math.min(x0, x);
				x1 = Math.max(x1, x);
				y0 = Math.min(y0, y);
				y1 = Math.max(y1, y);
			}
	if (x1 < 0) return null;
	const W = 240, H = 200;
	const c = document.createElement('canvas');
	c.width = W;
	c.height = H;
	const g = c.getContext('2d')!;
	const kind = KIND_INFO[CARD_MAP[id].kind];
	const grd = g.createRadialGradient(W / 2, H * 0.42, 10, W / 2, H * 0.5, W * 0.75);
	grd.addColorStop(0, '#ffffff');
	grd.addColorStop(0.35, kind.color + 'aa');
	grd.addColorStop(1, kind.color);
	g.fillStyle = grd;
	g.fillRect(0, 0, W, H);
	// baseplate studs along the bottom
	g.fillStyle = 'rgba(0,0,0,0.12)';
	for (let i = 0; i < 9; i++) {
		g.beginPath();
		g.arc(14 + i * 27, H - 10, 8, 0, Math.PI * 2);
		g.fill();
	}
	const sw = x1 - x0 + 1, sh = y1 - y0 + 1;
	const scale = Math.max(1, Math.floor(Math.min((W * 0.86) / sw, (H * 0.8) / sh)));
	const dw = sw * scale, dh = sh * scale;
	const dx = Math.round((W - dw) / 2), dy = Math.round(H - 18 - dh);
	g.fillStyle = 'rgba(0,0,0,0.22)';
	g.beginPath();
	g.ellipse(W / 2, H - 16, dw * 0.38, 7, 0, 0, Math.PI * 2);
	g.fill();
	g.imageSmoothingEnabled = false;
	g.drawImage(cell, x0, y0, sw, sh, dx, dy, dw, dh);
	return c.toDataURL('image/png');
}

/** the card's picture window: background and baseplate (the 3D Voxelmon stands on it) */
function cardBackdrop(id: string) {
	const W = 240, H = 200;
	const c = document.createElement('canvas');
	c.width = W;
	c.height = H;
	const g = c.getContext('2d')!;
	const kind = KIND_INFO[CARD_MAP[id].kind];
	const grd = g.createRadialGradient(W / 2, H * 0.42, 10, W / 2, H * 0.5, W * 0.75);
	grd.addColorStop(0, '#ffffff');
	grd.addColorStop(0.35, kind.color + 'aa');
	grd.addColorStop(1, kind.color);
	g.fillStyle = grd;
	g.fillRect(0, 0, W, H);
	// baseplate studs along the bottom and a soft shadow
	g.fillStyle = 'rgba(0,0,0,0.12)';
	for (let i = 0; i < 9; i++) {
		g.beginPath();
		g.arc(14 + i * 27, H - 10, 8, 0, Math.PI * 2);
		g.fill();
	}
	g.fillStyle = 'rgba(0,0,0,0.22)';
	g.beginPath();
	g.ellipse(W / 2, H - 18, W * 0.3, 9, 0, 0, Math.PI * 2);
	g.fill();
	return { c, g, W, H };
}
/**
 * draws the beast into a card picture: straight from the front as pixel art, or (once the card is a real
 * Voxelmon) rendered in 3D from the front. false if there is no voxel model.
 */
function drawBeast(id: string, g: CanvasRenderingContext2D, W: number, H: number, as3d: boolean) {
	const model = MODELS[id];
	if (!model?.build) return false;
	if (!as3d) {
		drawPixelPicture(model, g, W, H);
		return true;
	}
	const pic = voxelPortrait(id);
	if (!pic) return false;
	// same box as the pixel picture: centred, feet just above the bottom edge
	const s = Math.min((W * 0.86) / pic.width, (H * 0.84) / pic.height);
	const w = pic.width * s, h = pic.height * s;
	g.imageSmoothingEnabled = true;
	g.drawImage(pic, (W - w) / 2, H - 14 - h, w, h);
	return true;
}
function voxelArt(id: string, as3d: boolean): string | null {
	const { c, g, W, H } = cardBackdrop(id);
	return drawBeast(id, g, W, H, as3d) ? c.toDataURL('image/png') : null;
}
const creatureCache = new Map<string, string | null>();
/**
 * only the beast, on a transparent background, at the same place as in cardArt: cards draw it above their
 * holo / glitter, so the shine is on the background and not on the beast. null for beasts without a voxel model.
 */
export function cardArtCreature(id: string, as3d = false): string | null {
	const key = as3d ? `${id}|3d` : id;
	if (creatureCache.has(key)) return creatureCache.get(key)!;
	const c = document.createElement('canvas');
	c.width = 240;
	c.height = 200;
	const url = drawBeast(id, c.getContext('2d')!, 240, 200, as3d) ? c.toDataURL('image/png') : null;
	creatureCache.set(key, url);
	return url;
}
const bgCache = new Map<string, string>();
/** just the empty picture window (for the 3D card, where the real Voxelmon stands on it) */
export function cardArtBackdrop(id: string): string {
	if (!bgCache.has(id)) bgCache.set(id, cardBackdrop(id).c.toDataURL('image/png'));
	return bgCache.get(id)!;
}

/** the card's picture; `as3d`: the card is a real Voxelmon, so its picture may show the 3D cubes */
export function cardArt(id: string, as3d = false): string {
	const key = as3d ? `${id}|3d` : id;
	const hit = cache.get(key);
	if (hit) return hit;
	const url = voxelArt(id, as3d) ?? spriteArt(id) ?? brickArt(id);
	cache.set(key, url);
	return url;
}
