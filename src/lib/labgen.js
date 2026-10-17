/**
 * labgen.js — procedural laboratory-equipment generator (pure, no DOM).
 *
 * Ported from the reference "Prozeduraler Laborbedarf-Generator": a small
 * machine/item drawn onto a 32px tile — one of poketable / mainframe /
 * analyzer / incubator / default (CRT desk) — with optional cables, papers,
 * books and beakers scattered around it. `ctx` is passed in so headless tests
 * can drive it.
 *
 * params: { seed, palette | paletteIndex, itemWidth, itemHeight,
 *   itemType(poketable|mainframe|analyzer|incubator|desk), deskStyle(wood|tech|metal),
 *   showGroundShadow, showPokeballs, showScreenGlow, cablesLevel,
 *   showPapers, showBooks, showBeakers }
 */
import { createPRNG } from './flowerfield.js';

/** Reference palettes (Retro Pokémon GBA lab look). */
export const LAB_PALETTES = [
	{
		id: 'oak-classic', name: 'Kanto Lab (Retro Oak)',
		metalLight: '#cbd5e1', metalMid: '#64748b', metalDark: '#334155', deskTop: '#94a3b8',
		woodLight: '#d97706', woodDark: '#78350f', screenBg: '#064e3b', screenGlow: '#34d399',
		cable1: '#ef4444', cable2: '#3b82f6', cable3: '#eab308', liquid: '#06b6d4', outline: '#0f172a'
	},
	{
		id: 'cyber-green', name: 'High-Tech Plasma',
		metalLight: '#e2e8f0', metalMid: '#475569', metalDark: '#1e293b', deskTop: '#64748b',
		woodLight: '#b45309', woodDark: '#451a03', screenBg: '#022c22', screenGlow: '#10b981',
		cable1: '#ec4899', cable2: '#10b981', cable3: '#f59e0b', liquid: '#a855f7', outline: '#020617'
	},
	{
		id: 'vintage-wood', name: "Prof. Oak's Study",
		metalLight: '#94a3b8', metalMid: '#475569', metalDark: '#1e293b', deskTop: '#b45309',
		woodLight: '#d97706', woodDark: '#451a03', screenBg: '#1e1b4b', screenGlow: '#818cf8',
		cable1: '#dc2626', cable2: '#2563eb', cable3: '#16a34a', liquid: '#f43f5e', outline: '#1c1917'
	},
	{
		id: 'silph-co', name: 'Silph Co. Prototype',
		metalLight: '#f1f5f9', metalMid: '#94a3b8', metalDark: '#475569', deskTop: '#e2e8f0',
		woodLight: '#92400e', woodDark: '#451a03', screenBg: '#172554', screenGlow: '#60a5fa',
		cable1: '#f97316', cable2: '#06b6d4', cable3: '#84cc16', liquid: '#10b981', outline: '#090d16'
	}
];

export function drawLabEquipment(ctx, width, height, params) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, width, height);

	const prng = createPRNG(params.seed);
	const palette = params.palette || LAB_PALETTES[params.paletteIndex ?? 0] || LAB_PALETTES[0];

	const centerX = Math.floor(width / 2);
	const groundY = height - 8;

	const itemW = params.itemWidth;
	const itemH = params.itemHeight;

	const itemLeft = centerX - Math.floor(itemW / 2);
	const itemRight = itemLeft + itemW;
	const itemTop = groundY - itemH;

	// 1. Ground shadow.
	if (params.showGroundShadow) {
		ctx.fillStyle = 'rgba(15, 23, 42, 0.45)';
		ctx.fillRect(itemLeft - 3, groundY + 1, itemW + 6, 4);
		ctx.fillRect(itemLeft - 1, groundY + 5, itemW + 2, 2);
	}

	// 2. Base frame / housing / desk.
	const drawBaseTable = (topY, tableH) => {
		const isWood = params.deskStyle === 'wood';
		const isTech = params.deskStyle === 'tech';
		const primaryColor = isWood ? palette.woodDark : isTech ? '#e2e8f0' : palette.metalMid;
		const lightColor = isWood ? palette.woodLight : isTech ? '#ffffff' : palette.metalLight;
		const darkColor = isWood ? '#271003' : palette.metalDark;

		ctx.fillStyle = darkColor;
		ctx.fillRect(itemLeft + 2, topY + tableH, 3, groundY - (topY + tableH));
		ctx.fillRect(itemRight - 5, topY + tableH, 3, groundY - (topY + tableH));

		ctx.fillStyle = primaryColor;
		ctx.fillRect(itemLeft, topY, itemW, tableH);

		ctx.fillStyle = lightColor;
		ctx.fillRect(itemLeft, topY, itemW, 2);

		ctx.fillStyle = palette.outline;
		ctx.fillRect(itemLeft - 1, topY - 1, itemW + 2, 1);
		ctx.fillRect(itemLeft - 1, topY, 1, tableH + 1);
		ctx.fillRect(itemRight, topY, 1, tableH + 1);
	};

	// 3. Equipment types.
	if (params.itemType === 'poketable') {
		drawBaseTable(itemTop + 8, itemH - 8);
		const trayW = itemW - 8;
		const trayX = itemLeft + 4;
		const trayY = itemTop + 4;
		ctx.fillStyle = palette.metalDark;
		ctx.fillRect(trayX, trayY + 4, trayW, 4);
		ctx.fillStyle = palette.metalLight;
		ctx.fillRect(trayX, trayY + 4, trayW, 1);
		if (params.showPokeballs) {
			const ballPositions = [
				trayX + Math.floor(trayW * 0.2),
				trayX + Math.floor(trayW * 0.5),
				trayX + Math.floor(trayW * 0.8)
			];
			ballPositions.forEach((bx) => {
				const by = trayY - 1;
				ctx.fillStyle = palette.metalLight;
				ctx.fillRect(bx - 3, by + 4, 6, 2);
				ctx.fillStyle = '#ef4444';
				ctx.fillRect(bx - 3, by - 2, 6, 3);
				ctx.fillStyle = '#f8fafc';
				ctx.fillRect(bx - 3, by + 1, 6, 3);
				ctx.fillStyle = palette.outline;
				ctx.fillRect(bx - 4, by - 1, 1, 5);
				ctx.fillRect(bx + 3, by - 1, 1, 5);
				ctx.fillRect(bx - 3, by - 3, 6, 1);
				ctx.fillRect(bx - 3, by + 4, 6, 1);
				ctx.fillRect(bx - 3, by, 6, 1);
				ctx.fillStyle = '#ffffff';
				ctx.fillRect(bx - 1, by, 2, 1);
			});
		}
	} else if (params.itemType === 'mainframe') {
		ctx.fillStyle = palette.metalDark;
		ctx.fillRect(itemLeft, itemTop, itemW, itemH);
		for (let y = itemTop + 4; y < groundY - 6; y += 6) {
			ctx.fillStyle = palette.metalMid;
			ctx.fillRect(itemLeft + 3, y, itemW - 6, 4);
			for (let x = itemLeft + 5; x < itemRight - 6; x += 4) {
				const ledState = prng() > 0.4;
				ctx.fillStyle = ledState ? (prng() > 0.5 ? '#22c55e' : '#ef4444') : '#1e293b';
				ctx.fillRect(x, y + 1, 2, 2);
			}
		}
		const scW = itemW - 10;
		const scH = 10;
		const scX = itemLeft + 5;
		const scY = itemTop + 6;
		ctx.fillStyle = palette.screenBg;
		ctx.fillRect(scX, scY, scW, scH);
		if (params.showScreenGlow) {
			ctx.fillStyle = palette.screenGlow;
			ctx.fillRect(scX + 2, scY + 2, scW - 6, 1);
			ctx.fillRect(scX + 2, scY + 5, Math.floor(scW * 0.6), 1);
			ctx.fillRect(scX + 2, scY + 7, Math.floor(scW * 0.4), 1);
		}
		ctx.fillStyle = palette.outline;
		ctx.fillRect(itemLeft - 1, itemTop - 1, itemW + 2, 1);
		ctx.fillRect(itemLeft - 1, itemTop + itemH, itemW + 2, 1);
		ctx.fillRect(itemLeft - 1, itemTop - 1, 1, itemH + 2);
		ctx.fillRect(itemRight, itemTop - 1, 1, itemH + 2);
	} else if (params.itemType === 'analyzer') {
		drawBaseTable(itemTop + 10, itemH - 10);
		const devX = centerX - 8;
		const devY = itemTop + 2;
		ctx.fillStyle = palette.metalDark;
		ctx.fillRect(devX, devY, 16, 10);
		ctx.fillStyle = palette.screenGlow;
		ctx.fillRect(devX + 2, devY + 4, 12, 2);
		ctx.fillStyle = 'rgba(255,255,255,0.8)';
		ctx.fillRect(devX + 12, devY - 4, 3, 6);
		ctx.fillStyle = palette.liquid;
		ctx.fillRect(devX + 12, devY - 2, 3, 4);
	} else if (params.itemType === 'incubator') {
		drawBaseTable(groundY - 8, 8);
		const glassW = itemW - 12;
		const glassH = itemH - 10;
		const glassX = itemLeft + 6;
		const glassY = itemTop + 2;
		ctx.fillStyle = palette.metalDark;
		ctx.fillRect(glassX - 2, glassY - 2, glassW + 4, 3);
		ctx.fillRect(glassX - 2, glassY + glassH - 1, glassW + 4, 3);
		ctx.fillStyle = palette.screenBg;
		ctx.fillRect(glassX, glassY, glassW, glassH);
		ctx.fillStyle = palette.liquid;
		ctx.fillRect(glassX + 1, glassY + 3, glassW - 2, glassH - 4);
		const eggX = centerX;
		const eggY = glassY + Math.floor(glassH / 2);
		ctx.fillStyle = '#fef08a';
		ctx.fillRect(eggX - 3, eggY - 4, 6, 8);
		ctx.fillRect(eggX - 4, eggY - 2, 8, 5);
		ctx.fillStyle = '#f97316';
		ctx.fillRect(eggX - 2, eggY - 2, 2, 2);
		ctx.fillRect(eggX + 1, eggY + 1, 2, 2);
		if (params.showScreenGlow) {
			ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
			ctx.fillRect(glassX + 1, glassY + 1, 2, glassH - 2);
			ctx.fillRect(glassX + 4, glassY + 1, 1, glassH - 2);
		}
	} else {
		drawBaseTable(itemTop + 12, itemH - 12);
		const monW = 14;
		const monH = 11;
		const monX = itemLeft + 4;
		const monY = itemTop + 1;
		ctx.fillStyle = palette.metalDark;
		ctx.fillRect(monX, monY, monW, monH);
		ctx.fillStyle = palette.screenBg;
		ctx.fillRect(monX + 2, monY + 2, monW - 4, monH - 4);
		if (params.showScreenGlow) {
			ctx.fillStyle = palette.screenGlow;
			ctx.fillRect(monX + 5, monY + 4, 4, 1);
			ctx.fillRect(monX + 5, monY + 6, 4, 1);
		}
	}

	// 4. Cable mess.
	if (params.cablesLevel > 0) {
		const cableColors = [palette.cable1, palette.cable2, palette.cable3];
		const count = params.cablesLevel * 3;
		for (let i = 0; i < count; i++) {
			const cColor = cableColors[i % cableColors.length];
			const startX = itemLeft + Math.floor(prng() * itemW);
			const startY = groundY - Math.floor(prng() * 10);
			ctx.fillStyle = cColor;
			let cx = startX;
			let cy = startY;
			const len = 8 + Math.floor(prng() * 12);
			for (let step = 0; step < len; step++) {
				ctx.fillRect(cx, cy, 1, 1);
				if (prng() > 0.5) cx += prng() > 0.5 ? 1 : -1;
				else cy += prng() > 0.3 ? 1 : 0;
			}
		}
	}

	// 5. Papers.
	if (params.showPapers) {
		const paperCount = 3 + Math.floor(prng() * 4);
		for (let i = 0; i < paperCount; i++) {
			const px = itemLeft - 6 + Math.floor(prng() * (itemW + 12));
			const py = groundY - 2 + Math.floor(prng() * 6);
			ctx.fillStyle = '#f8fafc';
			ctx.fillRect(px, py, 4, 3);
			ctx.fillStyle = '#94a3b8';
			ctx.fillRect(px + 1, py + 1, 2, 1);
			ctx.fillStyle = palette.outline;
			ctx.fillRect(px - 1, py, 1, 3);
		}
	}

	// 6. Books.
	if (params.showBooks) {
		const bx = itemRight - 10;
		const by = groundY - 3;
		const bookColors = ['#dc2626', '#2563eb', '#16a34a', '#d97706'];
		for (let b = 0; b < 3; b++) {
			const bookY = by - b * 3;
			ctx.fillStyle = bookColors[(b + Math.floor(prng() * 4)) % bookColors.length];
			ctx.fillRect(bx - (b % 2), bookY, 8, 2);
			ctx.fillStyle = '#f8fafc';
			ctx.fillRect(bx - (b % 2) + 7, bookY, 1, 2);
			ctx.fillStyle = palette.outline;
			ctx.fillRect(bx - (b % 2) - 1, bookY, 1, 2);
		}
	}

	// 7. Chemistry beakers.
	if (params.showBeakers) {
		const bkX = itemLeft + 2;
		const bkY = itemTop + 4;
		ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
		ctx.fillRect(bkX, bkY + 2, 4, 4);
		ctx.fillRect(bkX + 1, bkY, 2, 2);
		ctx.fillStyle = palette.liquid;
		ctx.fillRect(bkX + 1, bkY + 3, 2, 3);
		ctx.fillStyle = palette.outline;
		ctx.fillRect(bkX - 1, bkY + 2, 1, 4);
		ctx.fillRect(bkX + 4, bkY + 2, 1, 4);
	}
}
