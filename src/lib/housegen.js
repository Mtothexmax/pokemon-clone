/**
 * housegen.js — procedural house/facade generator (pure, no DOM).
 *
 * Ported from the reference "Prozeduraler Häuser-Generator": a gabled /
 * gambrel / domed roof over a patterned wall, with arched/square windows,
 * a panelled door and optional awning, porch, windmill wheel, ladder and
 * side annex. Sizes are driven by a single integer `scale` S so the whole
 * facade stays crisp at the game's 32px tile grid, while the wall spans the
 * full footprint width.
 *
 * `drawHouse` deliberately does NOT clear the canvas: the caller (main.js)
 * paints grass underneath first, so the transparent roof-triangle corners
 * don't punch a black hole through the covered map tiles.
 */
import { createPRNG } from './bigtree.js';

/** Five reference palettes (classic GBA town look) + driveway variants. */
export const HOUSE_PALETTES = [
	{
		id: 'red-roof-classic', name: 'Classic Red Roof',
		roofLight: '#e05252', roofMid: '#c0392b', roofDark: '#8e2a20',
		wallLight: '#f5e6ca', wallMid: '#e6d3b3', wallDark: '#b8a383',
		trim: '#ffffff', door: '#a0522d', glass: '#64b5f6', glassHighlight: '#e3f2fd',
		outline: '#2c1810'
	},
	{
		id: 'blue-siding', name: 'Blue Roof & Pale Wood',
		roofLight: '#4a90e2', roofMid: '#2c3e50', roofDark: '#1a252f',
		wallLight: '#e8ecef', wallMid: '#bdc3c7', wallDark: '#7f8c8d',
		trim: '#ffffff', door: '#d35400', glass: '#4fc3f7', glassHighlight: '#ffffff',
		outline: '#111827'
	},
	{
		id: 'barn-red', name: 'Red Wood Barn',
		roofLight: '#cf4545', roofMid: '#9b2c2c', roofDark: '#631717',
		wallLight: '#8d4004', wallMid: '#6a2e02', wallDark: '#451c00',
		trim: '#ffffff', door: '#ffffff', glass: '#334155', glassHighlight: '#94a3b8',
		outline: '#1e0c00'
	},
	{
		id: 'yellow-plaster', name: 'Yellow Plaster & Mill',
		roofLight: '#d35400', roofMid: '#a04000', roofDark: '#6e2c00',
		wallLight: '#fef08a', wallMid: '#fde047', wallDark: '#ca8a04',
		trim: '#ffffff', door: '#5c3a21', glass: '#38bdf8', glassHighlight: '#e0f2fe',
		outline: '#1f1300'
	},
	{
		id: 'stone-manor', name: 'Stone Manor',
		roofLight: '#8b5cf6', roofMid: '#6d28d9', roofDark: '#4c1d95',
		wallLight: '#cbd5e1', wallMid: '#94a3b8', wallDark: '#475569',
		trim: '#e2e8f0', door: '#451a03', glass: '#a5f3fc', glassHighlight: '#ffffff',
		outline: '#0f172a'
	},
	{
		id: 'slate-cottage', name: 'Slate Cottage',
		roofLight: '#8a94a6', roofMid: '#5e6675', roofDark: '#3c4250',
		wallLight: '#f5efe0', wallMid: '#d8cbb0', wallDark: '#b0a184',
		trim: '#ffffff', door: '#8d5524', glass: '#3f7fc8', glassHighlight: '#cfe8ff',
		outline: '#1e2b1a'
	}
];

/**
 * params: { seed, palette | paletteIndex, scale, groundY, inset,
 *   wallHeight, roofHeight, roofType(gabled|gambrel|rounded),
 *   wallStyle(wood|brick|siding), doorType(wood|double|barn),
 *   windowType(arched|square|gba), windowFracs, doorFrac, floors(1|2),
 *   showChimney, porchRoof(small-gable), showAwning, showPorch,
 *   showWindmill, showLadder, showExtension, showGroundShadow }
 */
export function drawHouse(ctx, width, height, params = {}) {
	ctx.imageSmoothingEnabled = false;
	const palette = params.palette || HOUSE_PALETTES[params.paletteIndex ?? 0] || HOUSE_PALETTES[0];
	const S = params.scale ?? 3;
	const centerX = Math.floor(width / 2);
	const groundY = params.groundY ?? height - 6;
	const inset = params.inset ?? 4;
	const houseLeft = inset;
	const houseRight = width - inset;
	const houseW = houseRight - houseLeft;
	const wallH = params.wallHeight ?? 62;
	const roofH = params.roofHeight ?? 62;
	const wallTop = groundY - wallH;
	const roofApexY = wallTop - roofH;
	const outline = palette.outline;

	const line = (x, y, w, h, c) => {
		ctx.fillStyle = c;
		ctx.fillRect(x, y, w, h);
	};

	// 1. Ground shadow.
	if (params.showGroundShadow ?? true) {
		line(houseLeft - 2, groundY, houseW + 4, 3, 'rgba(15,23,42,0.4)');
		line(houseLeft - 1, groundY + 3, houseW + 2, 1, 'rgba(15,23,42,0.4)');
	}

	// 2. Side annex (optional).
	if (params.showExtension) {
		const extW = Math.floor(houseW * 0.4);
		const extH = Math.floor(wallH * 0.7);
		const extLeft = houseRight - 2;
		const extTop = groundY - extH;
		line(extLeft, extTop, extW, extH, palette.wallMid);
		line(extLeft + extW - 1, extTop, 1, extH, palette.wallDark);
		line(extLeft - 1, extTop - 4, extW + 2, 4, palette.roofMid);
		line(extLeft - 1, extTop - 4, extW + 2, 1, palette.roofLight);
		line(extLeft - 1, extTop - 5, extW + 3, 1, outline);
	}

	// 2.5 Chimney (behind the wall + roof so it only pokes above the slope).
	if (params.showChimney) {
		const chimneyW = Math.max(4, Math.round(S * 2));
		const chimneyX = houseRight - chimneyW - 1;
		const chimneyTopY = roofApexY - 2;
		const chimneyH = groundY - chimneyTopY;
		line(chimneyX, chimneyTopY, chimneyW, chimneyH, palette.wallDark);
		line(chimneyX + 1, chimneyTopY + 1, chimneyW - 2, chimneyH, palette.wallMid);
		line(chimneyX - 1, chimneyTopY, chimneyW + 2, 2, palette.trim);
		line(chimneyX - 1, chimneyTopY - 1, chimneyW + 2, 1, outline);
		line(chimneyX - 2, chimneyTopY, 1, 2, outline);
		line(chimneyX + chimneyW + 1, chimneyTopY, 1, 2, outline);
	}

	// 3. Facade / wall body + pattern.
	line(houseLeft, wallTop, houseW, wallH, palette.wallMid);
	const wallStyle = params.wallStyle ?? 'wood';
	const pitch = Math.max(4, Math.round(S * 2));
	if (wallStyle === 'wood') {
		for (let y = wallTop; y < groundY; y += pitch) {
			line(houseLeft, y, houseW, 1, palette.wallDark);
			line(houseLeft, y + 1, houseW, 1, palette.wallLight);
		}
	} else if (wallStyle === 'brick') {
		for (let y = wallTop; y < groundY; y += pitch) {
			line(houseLeft, y, houseW, 1, palette.wallDark);
			const rowOffset = (Math.floor((y - wallTop) / pitch) % 2) * pitch;
			for (let x = houseLeft + rowOffset; x < houseRight; x += pitch * 2) {
				line(x, y, 1, pitch, palette.wallDark);
			}
		}
	} else {
		line(houseLeft + 2, wallTop + 1, houseW - 4, wallH - 1, palette.wallLight);
	}
	// Floor divider for two-storey facades.
	const floors = params.floors ?? 1;
	const floorY = wallTop + Math.floor(wallH / 2);
	if (floors === 2) {
		line(houseLeft, floorY, houseW, 1, palette.wallDark);
		line(houseLeft, floorY + 1, houseW, 1, palette.trim);
	}
	// Corner trims + outlines.
	const tw = Math.max(2, Math.round(S * 0.7));
	line(houseLeft, wallTop, tw, wallH, palette.trim);
	line(houseRight - tw, wallTop, tw, wallH, palette.trim);
	line(houseLeft - 1, wallTop, 1, wallH, outline);
	line(houseRight, wallTop, 1, wallH, outline);

	// 4. Roof.
	const roofType = params.roofType ?? 'gabled';
	const overhang = inset;
	const spanHalf = Math.floor((houseW + overhang * 2) / 2);
	if (roofType === 'gabled') {
		for (let y = wallTop; y >= roofApexY; y--) {
			const progress = (wallTop - y) / roofH;
			const currentW = Math.round((houseW + overhang * 2) * (1 - progress));
			const x = centerX - Math.floor(currentW / 2);
			line(x, y, Math.max(1, currentW), 1, palette.roofMid);
			line(x, y, Math.max(1, Math.floor(currentW * 0.45)), 1, palette.roofLight);
			line(x + Math.floor(currentW * 0.75), y, Math.max(1, Math.floor(currentW * 0.25)), 1, palette.roofDark);
			line(x - 1, y, 1, 1, outline);
			line(x + currentW, y, 1, 1, outline);
		}
		line(centerX - 1, roofApexY - 1, 2, 1, outline);
	} else if (roofType === 'gambrel') {
		const breakY = wallTop - Math.floor(roofH * 0.6);
		for (let y = wallTop; y >= breakY; y--) {
			const progress = (wallTop - y) / Math.max(1, wallTop - breakY);
			const currentW = Math.round(houseW + overhang * 2 - progress * (houseW * 0.3));
			const x = centerX - Math.floor(currentW / 2);
			line(x, y, currentW, 1, palette.roofMid);
			line(x, y, Math.max(1, Math.floor(currentW * 0.5)), 1, palette.roofLight);
			line(x - 1, y, 1, 1, outline);
			line(x + currentW, y, 1, 1, outline);
		}
		for (let y = breakY; y >= roofApexY; y--) {
			const progress = (breakY - y) / Math.max(1, breakY - roofApexY);
			const currentW = Math.round(houseW * 0.7 * (1 - progress) + 4);
			const x = centerX - Math.floor(currentW / 2);
			line(x, y, Math.max(1, currentW), 1, palette.roofMid);
			line(x, y, Math.max(1, Math.floor(currentW * 0.5)), 1, palette.roofLight);
			line(x - 1, y, 1, 1, outline);
			line(x + currentW, y, 1, 1, outline);
		}
	} else {
		const radiusX = Math.floor(houseW / 2) + overhang;
		for (let y = wallTop; y >= roofApexY; y--) {
			const dy = (wallTop - y) / roofH;
			const dx = Math.sqrt(Math.max(0, 1 - dy * dy));
			const currentW = Math.round(radiusX * 2 * dx);
			const x = centerX - Math.floor(currentW / 2);
			line(x, y, Math.max(1, currentW), 1, palette.roofMid);
			line(x + 2, y, Math.max(1, Math.floor(currentW * 0.4)), 1, palette.roofLight);
			if (currentW > 3) line(x - 1, y, 1, 1, outline);
			if (currentW > 3) line(x + currentW, y, 1, 1, outline);
		}
	}
	// Shingle courses + gable trim + eave shadow.
	for (let y = wallTop - Math.round(roofH * 0.25); y >= roofApexY + 2; y -= Math.max(4, S)) {
		const progress = (wallTop - y) / roofH;
		const currentW = Math.round((houseW + overhang * 2) * (1 - progress));
		const x = centerX - Math.floor(currentW / 2);
		if (roofType === 'gabled') line(x + 1, y, Math.max(1, currentW - 2), 1, palette.roofDark);
	}
	line(houseLeft, wallTop - 1, houseW, 1, palette.wallDark);
	line(houseLeft, wallTop, houseW, 2, 'rgba(15,23,42,0.25)');

	// 5. Windmill wheel (optional).
	if (params.showWindmill) {
		const mx = centerX;
		const my = roofApexY - 4;
		line(mx - 3, my - 3, 6, 6, palette.trim);
		ctx.strokeStyle = outline;
		ctx.lineWidth = 1;
		ctx.strokeRect(mx - 3.5, my - 3.5, 7, 7);
		line(mx - 2, my - 2, 4, 4, '#facc15');
		line(mx - 1, my - 12, 2, 10, '#fde047');
		line(mx - 1, my + 2, 2, 10, '#fde047');
		line(mx - 12, my - 1, 10, 2, '#fde047');
		line(mx + 2, my - 1, 10, 2, '#fde047');
		line(mx - 2, my - 12, 1, 10, outline);
		line(mx + 1, my - 12, 1, 10, outline);
		line(mx - 12, my - 2, 10, 1, outline);
		line(mx - 12, my + 1, 10, 1, outline);
	}

	// 6. Windows.
	const windowW = Math.round(6 * S);
	const windowH = Math.round(8 * S);
	const drawWindow = (wx, y) => {
		line(wx - 1, y - 1, windowW + 2, windowH + 2, outline);
		line(wx, y, windowW, windowH, palette.trim);
		line(wx + 1, y + 1, windowW - 2, windowH - 2, palette.glass);
		if ((params.windowType ?? 'arched') === 'arched') line(wx + 2, y, windowW - 4, 1, palette.glass);
		line(wx + 1, y + 1, Math.max(1, Math.round(windowW * 0.35)), 2, palette.glassHighlight);
		line(wx + Math.floor(windowW / 2), y + 1, 1, windowH - 2, outline);
		line(wx + 1, y + Math.floor(windowH / 2), windowW - 2, 1, outline);
		line(wx - 1, y + windowH + 1, windowW + 2, 2, palette.wallDark);
		line(wx - 1, y + windowH + 1, windowW + 2, 1, palette.trim);
	};
	if (floors === 2) {
		// Upper + lower rows, hugging the edges so the centred door stays clear.
		const topY = wallTop + 3;
		const botY = floorY + 4;
		const leftX = houseLeft + Math.round(houseW * 0.22) - Math.floor(windowW / 2);
		const rightX = houseRight - Math.round(houseW * 0.22) - Math.floor(windowW / 2);
		drawWindow(leftX, topY);
		drawWindow(rightX, topY);
		drawWindow(leftX, botY);
		drawWindow(rightX, botY);
	} else {
		const windowY = wallTop + Math.round(wallH * 0.13);
		const fracs =
			params.windowFracs ?? (houseW <= 180 ? [0.24, 0.76] : [0.17, 0.39, 0.61, 0.83]);
		for (const f of fracs) {
			const wx = houseLeft + Math.round(f * houseW) - Math.floor(windowW / 2);
			drawWindow(wx, windowY);
		}
	}

	// 7. Door (centred by default — matches the map's middle `d` tile).
	const doorType = params.doorType ?? 'wood';
	const doorW = Math.round((doorType === 'double' || doorType === 'barn' ? 11 : 8) * S);
	const doorH = Math.round(12 * S);
	const doorFrac = params.doorFrac ?? 0.5;
	const doorX =
		doorFrac === 0.5
			? centerX - Math.floor(doorW / 2)
			: houseLeft + Math.round(doorFrac * houseW) - Math.floor(doorW / 2);
	const doorY = groundY - doorH;
	line(doorX - 1, doorY - 1, doorW + 2, doorH + 1, outline);
	line(doorX - 3, doorY - 3, doorW + 6, 2, palette.wallDark);
	line(doorX, doorY, doorW, doorH, palette.door);
	if (doorType === 'barn') {
		line(doorX, doorY, 1, doorH, palette.trim);
		line(doorX + doorW - 1, doorY, 1, doorH, palette.trim);
		line(doorX + Math.floor(doorW / 2), doorY, 1, doorH, palette.trim);
		line(doorX + 1, doorY + 1, doorW - 2, 1, palette.trim);
		line(doorX + 1, doorY + doorH - 2, doorW - 2, 1, palette.trim);
	} else if (doorType === 'double') {
		line(doorX + 1, doorY + 1, Math.floor(doorW / 2) - 1, doorH - 2, palette.glass);
		line(doorX + Math.floor(doorW / 2) + 1, doorY + 1, Math.floor(doorW / 2) - 1, doorH - 2, palette.glass);
		line(doorX + Math.floor(doorW / 2), doorY, 1, doorH, outline);
		line(doorX + 2, doorY + 2, 2, 3, palette.glassHighlight);
		line(doorX + Math.floor(doorW / 2) + 2, doorY + 2, 2, 3, palette.glassHighlight);
		line(doorX + Math.floor(doorW / 2) - 1, doorY + Math.floor(doorH / 2), 1, 2, '#facc15');
		line(doorX + Math.floor(doorW / 2) + 1, doorY + Math.floor(doorH / 2), 1, 2, '#facc15');
	} else {
		line(doorX + 1, doorY + 1, doorW - 2, 1, palette.wallDark);
		const pw = Math.max(2, Math.floor(doorW / 3));
		line(doorX + 2, doorY + 2, pw, Math.round(doorH * 0.35), palette.wallDark);
		line(doorX + doorW - 2 - pw, doorY + 2, pw, Math.round(doorH * 0.35), palette.wallDark);
		line(doorX + 2, doorY + Math.round(doorH * 0.5), pw, Math.round(doorH * 0.4), palette.wallDark);
		line(doorX + doorW - 2 - pw, doorY + Math.round(doorH * 0.5), pw, Math.round(doorH * 0.4), palette.wallDark);
		const kw = Math.max(2, Math.round(doorW * 0.12));
		line(doorX + doorW - 2 - kw, doorY + Math.floor(doorH * 0.5), kw, kw, '#facc15');
	}
	line(doorX - 1, groundY, doorW + 2, 2, palette.wallDark);
	line(doorX - 2, groundY + 2, doorW + 4, 2, palette.wallLight);

	// 7.5 Gable porch roof over the door.
	if (params.porchRoof === 'small-gable') {
		const pRoofW = doorW + Math.round(2 * S);
		const pRoofH = Math.round(2.3 * S);
		const pRoofX = doorX - Math.round(S);
		const pRoofTopY = doorY - pRoofH + 1;
		for (let y = doorY; y >= pRoofTopY; y--) {
			const prog = (doorY - y) / pRoofH;
			const w = Math.round(pRoofW - prog * pRoofW);
			const px = doorX + Math.floor(doorW / 2) - Math.floor(w / 2);
			line(px, y, Math.max(1, w), 1, palette.roofMid);
			line(px, y, Math.max(1, Math.floor(w * 0.5)), 1, palette.roofLight);
			line(px - 1, y, 1, 1, outline);
			line(px + w, y, 1, 1, outline);
		}
		line(pRoofX + 1, doorY - 1, 2, doorH + 1, palette.trim);
		line(pRoofX + pRoofW - 3, doorY - 1, 2, doorH + 1, palette.trim);
		line(pRoofX, doorY - 1, 1, doorH + 1, outline);
		line(pRoofX + pRoofW - 1, doorY - 1, 1, doorH + 1, outline);
		line(pRoofX - 1, groundY - 1, pRoofW + 2, 2, '#cbd5e1');
		line(pRoofX - 1, groundY + 1, pRoofW + 2, 1, outline);
	}

	// 8. Awning (shop style).
	if (params.showAwning) {
		const awningY = doorY - 4;
		const awningW = houseW - 4;
		const awningX = houseLeft + 2;
		for (let ax = awningX; ax < awningX + awningW; ax++) {
			const stripe = Math.floor((ax - awningX) / 3) % 2 === 0;
			line(ax, awningY, 1, 4, stripe ? '#ef4444' : '#ffffff');
			line(ax, awningY + 4, 1, 1, stripe ? '#ef4444' : '#ffffff');
		}
		line(awningX, awningY + 5, awningW, 2, 'rgba(15,23,42,0.3)');
		line(awningX - 1, awningY - 1, awningW + 2, 1, outline);
	}

	// 9. Porch / railing + step.
	if (params.showPorch) {
		const porchY = groundY - 6;
		const porchW = Math.floor(houseW * 0.5);
		const porchX = centerX - Math.floor(porchW / 2);
		line(porchX, porchY, porchW, 2, palette.trim);
		for (let x = porchX; x <= porchX + porchW; x += 3) line(x, porchY, 1, 6, palette.trim);
		line(centerX - 3, groundY - 2, 6, 2, palette.wallDark);
		line(centerX - 4, groundY - 1, 8, 1, palette.wallDark);
		line(centerX - 5, groundY, 10, 1, outline);
	}

	// 10. Ladder (barn/bonus).
	if (params.showLadder) {
		const ladderX = houseLeft + 4;
		const ladderTopY = wallTop + 2;
		const ladderH = groundY - ladderTopY;
		line(ladderX, ladderTopY, 1, ladderH, palette.door);
		line(ladderX + 5, ladderTopY, 1, ladderH, palette.door);
		for (let y = ladderTopY + 2; y < groundY; y += 3) line(ladderX, y, 6, 1, palette.door);
		line(ladderX - 1, ladderTopY, 1, ladderH, outline);
		line(ladderX + 6, ladderTopY, 1, ladderH, outline);
	}

	// 11. Tiny deterministic wall weathering (keeps it from looking flat).
	const prng = createPRNG(params.seed ?? 1337);
	const n = Math.max(3, Math.round(houseW / 40));
	for (let i = 0; i < n; i++) {
		const x = houseLeft + 2 + Math.floor(prng() * (houseW - 4));
		const y = wallTop + 2 + Math.floor(prng() * (wallH - 4));
		line(x, y, 1, 1, palette.wallDark);
	}
}
