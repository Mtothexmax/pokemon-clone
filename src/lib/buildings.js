/**
 * buildings.js — whole-building facade generators (pure, no DOM).
 *
 * Each is authored 1:1 in its FINAL pixel size so every line is exactly one
 * pixel (no fractional scaling -> no thicker-on-one-side seams). They fill the
 * map's house box:
 *   - drawCyberLab   224x128  (Oak's palette, "Cyberpunk Oak's Lab")
 *   - drawAshHouse   160x128  (Ash's house, "Ash's House")
 *   - drawGaryHouse  192x128  (Gary's house, "Gary's House", blue roof)
 *
 * Background is left transparent so the caller's grass shows through.
 */

/** Tiny pixel/rect drawing helpers bound to a context. */
function mk(ctx) {
	return {
		r(x, y, w, h, c) {
			ctx.fillStyle = c;
			ctx.fillRect(x, y, w, h);
		},
		p(x, y, c) {
			ctx.fillStyle = c;
			ctx.fillRect(x, y, 1, 1);
		}
	};
}

// ------------------------------------------------------------- cyber lab

const LAB_CYBER = {
	roofDark: '#4a044e', roofMid: '#701a75', roofLight: '#a21caf', roofHighlight: '#f472b6',
	wallDark: '#0f172a', wallMid: '#1e293b', wallLight: '#334155', wallPanelLine: '#020617',
	trimDark: '#0e7490', trimMid: '#06b6d4', trimLight: '#67e8f9',
	neonCyan: '#22d3ee', neonPink: '#f43f5e', neonGreen: '#10b981', neonYellow: '#facc15',
	glassBg: '#0284c7', glassLight: '#38bdf8', glassGleam: '#e0f2fe',
	doorFrame: '#111827', doorGlass: '#06b6d4', doorHolo: '#38bdf8',
	groundShadow: 'rgba(2,6,23,0.55)'
};

/** Cyberpunk Oak's Lab, drawn 1:1 into a 224x128 canvas. opts: { ticks }. */
export function drawCyberLab(ctx, W = 224, H = 128, opts = {}) {
	const C = LAB_CYBER;
	const { r, p } = mk(ctx);
	const ticks = opts.ticks ?? 0;
	const pulseCyan = Math.sin(ticks * 0.1) > 0 ? C.neonCyan : C.trimDark;
	const pulsePink = Math.sin(ticks * 0.1 + 1.5) > 0 ? C.neonPink : C.roofDark;
	const pulseYellow = ticks % 20 < 10 ? C.neonYellow : '#854d0e';

	// Ground shadow under the building.
	r(28, 118, 168, 8, C.groundShadow);
	r(40, 123, 144, 4, 'rgba(0,0,0,0.30)');

	// --- roof (trapezoid, ridge centered) ---
	const rx = 16, rw = 192, ryTop = 10, ryBot = 54, rh = ryBot - ryTop;
	for (let row = 0; row < rh; row++) {
		const inset = Math.floor((row * (rw / 2)) / rh);
		const curX = rx + inset;
		const curW = rw - inset * 2;
		const curY = ryBot - row;
		let col = C.roofMid;
		if (row % 4 === 0) col = C.roofDark;
		else if (row % 4 === 1) col = C.roofLight;
		r(curX, curY, curW, 1, col);
		if (row % 6 === 2) r(curX + 4, curY, Math.max(0, curW - 8), 1, C.trimDark);
	}
	for (let row = 0; row <= rh; row++) {
		const inset = Math.floor((row * (rw / 2)) / rh);
		const curY = ryBot - row;
		p(rx + inset, curY, C.trimLight);
		p(rx + inset - 1, curY, C.trimDark);
		p(rx + rw - inset - 1, curY, C.trimLight);
		p(rx + rw - inset, curY, C.trimDark);
	}
	r(rx + rw / 2 - 16, ryTop, 32, 3, C.trimLight);
	r(rx + rw / 2 - 14, ryTop + 1, 28, 1, pulseCyan);

	// --- chimneys: tips reach the ceiling (y = 0) ---
	r(80, 0, 14, 26, C.wallDark);
	r(78, 0, 18, 3, C.trimDark);
	r(79, 2, 16, 1, C.trimMid);
	r(82, 8, 10, 1, C.wallPanelLine);
	p(88, 12, pulseCyan); p(88, 18, pulseCyan);
	r(130, 0, 14, 26, C.wallDark);
	r(128, 0, 18, 3, C.trimMid);
	r(129, 1, 16, 1, pulsePink);
	r(132, 10, 10, 1, C.wallPanelLine);
	r(136, 0, 2, 8, C.trimLight);

	// --- central turbine dome ---
	r(104, 14, 16, 16, C.trimDark);
	r(105, 15, 14, 14, C.glassBg);
	r(109, 17, 6, 10, C.glassLight);
	r(103, 14, 18, 2, C.trimMid);
	r(112, 20, 1, 6, pulseYellow);
	r(109, 22, 7, 1, pulseYellow);

	// --- main body ---
	const bx = 24, by = 54, bw = 176, bh = 68;
	r(bx, by, bw, bh, C.wallMid);
	for (let y = by + 6; y < by + bh; y += 8) r(bx, y, bw, 1, C.wallPanelLine);
	r(bx + 52, by, 1, bh, C.wallDark);
	r(bx + 124, by, 1, bh, C.wallDark);

	// side pillars
	r(bx - 5, by - 6, 9, bh + 6, C.trimDark);
	r(bx - 4, by - 6, 5, bh + 6, C.trimMid);
	r(bx + bw - 5, by - 6, 9, bh + 6, C.trimDark);
	r(bx + bw - 4, by - 6, 5, bh + 6, C.trimMid);
	for (let cy = by; cy < by + bh - 4; cy += 4) {
		p(bx, cy, (cy + ticks) % 8 < 4 ? C.neonCyan : C.trimDark);
		p(bx + bw - 1, cy, (cy + ticks) % 8 < 4 ? C.neonCyan : C.trimDark);
	}

	// --- left window (specimen pod) ---
	const wx = bx + 12, wy = by + 12, ww = 36, wh = 26;
	r(wx - 1, wy - 1, ww + 2, wh + 2, C.trimDark);
	r(wx, wy, ww, wh, C.wallDark);
	r(wx + 1, wy + 1, ww - 2, wh - 2, C.glassBg);
	r(wx + 4, wy + 5, ww - 12, wh - 10, C.glassLight);
	r(wx + 15, wy + 10, 6, 6, C.neonPink);
	r(wx + 2, wy + 2, 5, wh - 4, 'rgba(255,255,255,0.25)');

	// --- right terminal (server matrix), BELOW the left window ---
	const tx = bx + bw - 56, ty = by + 40, tw = 44, th = 24;
	r(tx - 1, ty - 1, tw + 2, th + 2, C.trimDark);
	r(tx, ty, tw, th, C.glassBg);
	for (let ly = ty + 3; ly < ty + th - 3; ly += 4) {
		const barW = ((ly * 7 + ticks) % 22) + 6;
		r(tx + 3, ly, Math.min(barW, tw - 6), 2, C.neonGreen);
	}
	r(tx + 2, ty + 2, 4, th - 4, 'rgba(255,255,255,0.22)');

	// --- holographic entrance (recessed, deep portal) ---
	const dw = 52, dh = 54, dx = 112 - dw / 2, dy = 68;
	r(dx - 4, dy - 4, dw + 8, dh + 4, C.trimDark);
	r(dx - 3, dy - 3, dw + 6, dh + 3, C.trimMid);
	r(dx - 2, dy - 2, dw + 4, dh + 2, C.trimLight);
	r(dx, dy, dw, dh, C.wallPanelLine);
	r(dx + 2, dy + 2, dw - 4, dh - 2, C.wallDark);
	r(dx + 2, dy + 2, dw - 4, 3, C.wallPanelLine);
	r(dx + 4, dy + 5, dw - 8, dh - 5, C.doorFrame);
	r(dx + 6, dy + 8, 18, dh - 12, C.doorGlass);
	r(dx + 28, dy + 8, 18, dh - 12, C.doorGlass);
	r(dx + 6, dy + 8, 18, 2, C.doorHolo);
	r(dx + 28, dy + 8, 18, 2, C.doorHolo);
	r(dx + 4, dy + dh - 3, dw - 8, 3, C.trimMid);
	const hx = dx + dw / 2;
	r(hx - 8, dy - 18, 16, 12, 'rgba(6,182,212,0.20)');
	r(hx - 5, dy - 16, 10, 8, C.neonPink);
	r(hx - 5, dy - 13, 10, 5, C.trimLight);
	r(hx, dy - 15, 3, 3, C.neonYellow);
	r(dx - 8, dy + dh, dw + 16, 4, C.trimDark);
	r(dx - 6, dy + dh + 2, dw + 12, 2, C.trimMid);
}

// ------------------------------------------------------------- Ash's house

const ASH_C = {
	outline: '#1e241e',
	roofLight: '#de6243', roofMid: '#c6432b', roofDark: '#8b2014', roofShadow: '#521008', roofRidge: '#f08265',
	sidingHighlight: '#dce5ec', sidingLight: '#b8cbdb', sidingMid: '#9bb1c4', sidingDark: '#758d9f', sidingShadow: '#4e6272',
	frameWhite: '#eaf0f6', frameShadow: '#8ea2b3', glassLight: '#8fbafd', glassMid: '#5b8eee', glassDark: '#2c59ca', glassGlint: '#ffffff',
	doorBody: '#ad4b28', doorLight: '#ca5d34', doorDark: '#6d2a13',
	doorHandle: '#f5c638', doorHandleLight: '#fff2a8', doorHandleDark: '#a17a14'
};

/** Ash's Pallet-Town house, drawn 1:1 into a 160x128 canvas. */
export function drawAshHouse(ctx, W = 160, H = 128, _opts = {}) {
	const C = ASH_C;
	const { r, p } = mk(ctx);

	// ground shadow
	r(22, 118, 116, 9, 'rgba(2,6,23,0.28)');
	r(32, 122, 96, 4, 'rgba(0,0,0,0.22)');

	// gabled roof (apex centered)
	const eaveL = 12, eaveR = 148, ryTop = 22, ryBot = 62, ryH = ryBot - ryTop;
	for (let row = 0; row < ryH; row++) {
		const inset = Math.floor((row * ((eaveR - eaveL) / 2)) / ryH);
		const curX = eaveL + inset;
		const curW = (eaveR - eaveL) - inset * 2;
		let col = C.roofMid;
		if (row % 5 === 0) col = C.roofDark;
		else if (row % 5 === 1) col = C.roofLight;
		r(curX, ryTop + row, curW, 1, col);
	}
	for (let row = 0; row <= ryH; row++) {
		const inset = Math.floor((row * ((eaveR - eaveL) / 2)) / ryH);
		const curY = ryTop + row;
		p(eaveL + inset, curY, C.roofRidge);
		p(eaveL + inset + 1, curY, C.roofLight);
		p(eaveR - inset - 1, curY, C.roofDark);
		p(eaveR - inset, curY, C.roofShadow);
	}
	r(eaveL - 2, ryBot - 2, (eaveR - eaveL) + 4, 3, C.roofShadow);
	r(eaveL - 1, ryBot, (eaveR - eaveL) + 2, 2, C.roofDark);

	// dormer windows punched into the roof
	for (const dx0 of [40, 92]) {
		r(dx0 - 2, 38, 28, 3, C.outline);
		r(dx0 - 1, 40, 26, 22, C.sidingMid);
		r(dx0 - 1, 40, 2, 22, C.sidingHighlight);
		r(dx0 + 23, 40, 2, 22, C.sidingDark);
		r(dx0 + 3, 44, 18, 15, C.outline);
		r(dx0 + 4, 45, 16, 13, C.frameWhite);
		r(dx0 + 5, 46, 14, 11, C.glassDark);
		r(dx0 + 5, 46, 14, 3, C.glassMid);
		r(dx0 + 11, 46, 2, 11, C.frameWhite);
		p(dx0 + 6, 47, C.glassGlint); p(dx0 + 7, 47, C.glassGlint); p(dx0 + 6, 48, C.glassGlint);
		p(dx0 + 15, 52, C.glassGlint); p(dx0 + 16, 52, C.glassGlint);
	}

	// walls
	const wx = 24, wy = 62, ww = 112, wh = 60;
	r(wx, wy, ww, wh, C.outline);
	r(wx + 1, wy + 1, ww - 2, wh - 2, C.sidingMid);
	for (let sy = wy + 4; sy < wy + wh - 2; sy += 6) {
		r(wx + 1, sy, ww - 2, 1, C.sidingHighlight);
		if (sy + 3 < wy + wh - 2) r(wx + 1, sy + 3, ww - 2, 1, C.sidingDark);
	}
	r(wx + 1, wy + 1, 2, wh - 2, C.sidingHighlight);
	r(wx + ww - 3, wy + 1, 2, wh - 2, C.sidingDark);

	// front door (left, aligns with the walkable "d" tile)
	r(35, 83, 26, 39, C.outline);
	r(36, 84, 24, 38, C.doorBody);
	r(36, 84, 24, 1, C.doorLight);
	r(36, 84, 1, 38, C.doorLight);
	r(59, 84, 1, 38, C.doorDark);
	r(40, 88, 16, 10, C.outline);
	r(41, 89, 14, 8, C.glassDark);
	r(41, 89, 14, 2, C.glassMid);
	p(44, 90, C.glassGlint); p(45, 90, C.glassGlint); p(44, 91, C.glassGlint);
	r(38, 106, 5, 5, C.doorHandleDark);
	r(38, 106, 4, 4, C.doorHandle);
	p(38, 106, C.doorHandleLight);

	// first-floor windows (right)
	for (const wx0 of [76, 108]) {
		r(wx0 - 1, 71, 26, 26, C.outline);
		r(wx0, 72, 24, 24, C.frameWhite);
		r(wx0 + 2, 74, 20, 20, C.glassDark);
		r(wx0 + 2, 74, 20, 4, C.glassMid);
		r(wx0 + 11, 74, 2, 20, C.frameWhite);
		r(wx0 + 2, 82, 20, 2, C.frameWhite);
		r(wx0 + 2, 84, 20, 1, C.frameShadow);
		p(wx0 + 4, 76, C.glassGlint); p(wx0 + 5, 76, C.glassGlint); p(wx0 + 4, 77, C.glassGlint);
		p(wx0 + 15, 87, C.glassGlint); p(wx0 + 16, 87, C.glassGlint);
	}
}

// ------------------------------------------------------------- Gary's house

/** Gary's house (blue roof / slate siding), drawn 1:1 into a 192x128 canvas. */
export function drawGaryHouse(ctx, W = 192, H = 128, _opts = {}) {
	const C = {
		roofLight: '#4a7fd4', roofMid: '#2f56a0', roofDark: '#1e3a70', roofRidge: '#16294f', roofShadow: '#0e1a33',
		sidingLight: '#dce3ea', sidingMid: '#bdc8d4', sidingDark: '#91a0b0', sidingLine: '#a2b0bf',
		frameWhite: '#f0f4f8', frameShadow: '#91a0b0',
		glassLight: '#8fbbf8', glassMid: '#5c92e8', glassDark: '#3563c4', glassGlint: '#ffffff',
		doorBody: '#8d4022', doorLight: '#a8502d', doorDark: '#5c2713', doorHandle: '#ffcc00',
		porchLight: '#e2e8f0', porchMid: '#cbd5e1', porchDark: '#64748b', railWhite: '#f8fafc', railShadow: '#94a3b8',
		outlineDark: '#1a202c'
	};
	const { r, p } = mk(ctx);

	// ground shadow
	r(46, 118, 108, 9, 'rgba(2,6,23,0.28)');
	r(56, 122, 88, 4, 'rgba(0,0,0,0.22)');

	// gabled roof
	const eaveL = 38, eaveR = 176, ryTop = 20, ryBot = 60, ryH = ryBot - ryTop;
	for (let row = 0; row < ryH; row++) {
		const inset = Math.floor((row * ((eaveR - eaveL) / 2)) / ryH);
		const curX = eaveL + inset;
		const curW = (eaveR - eaveL) - inset * 2;
		let col = C.roofMid;
		if (row % 5 === 0) col = C.roofDark;
		else if (row % 5 === 1) col = C.roofLight;
		r(curX, ryTop + row, curW, 1, col);
	}
	for (let row = 0; row <= ryH; row++) {
		const inset = Math.floor((row * ((eaveR - eaveL) / 2)) / ryH);
		const curY = ryTop + row;
		p(eaveL + inset, curY, C.roofLight);
		p(eaveL + inset + 1, curY, C.roofLight);
		p(eaveR - inset - 1, curY, C.roofDark);
		p(eaveR - inset, curY, C.roofRidge);
	}
	r(eaveL - 2, ryBot - 2, (eaveR - eaveL) + 4, 3, C.roofShadow);
	r(eaveL - 1, ryBot, (eaveR - eaveL) + 2, 2, C.roofMid);

	// flat-roof dormer windows
	for (const dx0 of [58, 112]) {
		r(dx0 - 2, 36, 24, 3, C.roofShadow);
		r(dx0 - 1, 39, 22, 20, C.sidingMid);
		r(dx0 - 1, 39, 2, 20, C.sidingLight);
		r(dx0 + 19, 39, 2, 20, C.sidingDark);
		r(dx0 + 3, 42, 14, 14, C.frameWhite);
		r(dx0 + 4, 43, 12, 12, C.glassDark);
		r(dx0 + 9, 43, 2, 12, C.frameShadow);
		r(dx0 + 3, 48, 14, 1, C.frameShadow);
		p(dx0 + 5, 44, C.glassGlint); p(dx0 + 6, 44, C.glassGlint); p(dx0 + 5, 45, C.glassGlint);
		p(dx0 + 12, 51, C.glassGlint); p(dx0 + 13, 51, C.glassGlint);
	}

	// walls
	const wx = 52, wy = 60, ww = 112, wh = 62;
	r(wx, wy, ww, wh, C.sidingMid);
	for (let sy = wy + 3; sy < wy + wh; sy += 5) r(wx, sy, ww, 1, C.sidingLine);
	r(wx, wy, 2, wh, C.sidingLight);
	r(wx + ww - 2, wy, 2, wh, C.sidingDark);

	// center upper window
	r(88, 68, 22, 16, C.frameWhite);
	r(90, 70, 18, 12, C.glassMid);
	p(92, 72, C.glassGlint); p(93, 72, C.glassGlint); p(92, 73, C.glassGlint);
	p(102, 78, C.glassGlint); p(103, 78, C.glassGlint);
	r(88, 82, 22, 1, C.frameShadow);

	// lower-left bay window grid
	r(60, 84, 30, 28, C.frameWhite);
	r(61, 85, 28, 26, C.glassDark);
	for (let gx = 61; gx <= 89; gx += 5) r(gx, 85, 1, 26, C.frameShadow);
	for (let gy = 85; gy <= 111; gy += 5) r(61, gy, 28, 1, C.frameShadow);
	for (let py = 86; py < 111; py += 5) for (let px = 62; px < 89; px += 5) p(px, py, C.glassGlint);

	// entrance porch + mahogany door (right, aligns with the walkable "d")
	const dx0 = 130, dx1 = 160;
	r(dx0 - 4, 76, (dx1 - dx0) + 8, 3, C.roofMid);
	r(dx0 - 5, 79, (dx1 - dx0) + 10, 3, C.roofDark);
	r(dx0 - 2, 72, (dx1 - dx0) + 4, 3, C.roofLight);
	r(dx0 - 2, 82, 2, 40, C.porchLight);
	r(dx1, 82, 2, 40, C.porchDark);
	r(dx0 - 3, 118, (dx1 - dx0) + 6, 4, C.porchMid);
	r(dx0 - 3, 122, (dx1 - dx0) + 6, 1, C.porchDark);
	r(dx0 + 1, 82, 28, 40, C.doorBody);
	r(dx0 + 1, 82, 28, 1, C.doorLight);
	r(dx0 + 1, 82, 1, 40, C.doorLight);
	r(dx0 + 27, 82, 1, 40, C.doorDark);
	r(dx0 + 5, 88, 14, 10, C.frameWhite);
	r(dx0 + 6, 89, 12, 8, C.glassLight);
	r(dx0 + 6, 93, 12, 1, C.frameShadow);
	r(dx0 + 12, 89, 1, 8, C.frameShadow);
	p(dx0 + 7, 90, C.glassGlint); p(dx0 + 8, 90, C.glassGlint);
	r(dx0 + 3, 104, 3, 3, C.doorHandle);
	p(dx0 + 3, 104, '#ffffff');

	// porch railing
	r(dx0 - 6, 100, 6, 2, C.railWhite);
	r(58, 100, 34, 2, C.railWhite);
	r(58, 96, 34, 1, C.railShadow);
	[58, 68, 78, 88].forEach((rx) => {
		r(rx, 96, 2, 10, C.railWhite);
		p(rx, 96, C.railShadow);
	});
	[dx0 - 6, dx1 + 3].forEach((rx) => {
		r(rx, 96, 2, 10, C.railWhite);
		p(rx, 96, C.railShadow);
	});
}
