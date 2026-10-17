/**
 * chargen.js — procedural character generator for every actor.
 *
 * A look is pure data: {skin, hair, style, hat, shirt, shirtTex,
 * pants, skirt, shoes}. `paintActor(ops, look, dir, phase)` draws a
 * 16x16 sprite against abstract ops ({rect, px, coat, shade}), so the
 * headless test can verify invariants without a canvas. The browser
 * binds ops to a 2d context in `buildActors()`.
 *
 * Directions: down/up/side (side is flipped for left). Phases 0/1 are
 * the walk cycle (legs spread, arms swing); standing uses phase 0.
 */

import { coat as coatTex } from './texture.js';
import { ASH_PAL, buildAshFrames } from './ash.js';

/** Cast looks (NPCs stay procedural; the player uses the Ash matrices). */
export const LOOKS = {
	oak: { skin: '#e0ac7e', hair: '#dcdcdc', style: 'bald', shirt: '#f7f7f7', shirtTex: 'weave', pants: '#6b5f4e', shoes: '#3e2723' },
	mom: { skin: '#f1c9a5', hair: '#5d4037', style: 'long', shirt: '#e74c3c', shirtTex: 'dots', skirt: true, shoes: '#4e342e' },
	farmer: { skin: '#c68863', hair: '#3e2723', style: 'cap', hat: '#e8c86b', shirt: '#2e9e5b', shirtTex: 'weave', pants: '#5d4037', shoes: '#3e2723' },
	lass: { skin: '#f1c9a5', hair: '#e8c86b', style: 'long', shirt: '#ec7fa9', shirtTex: 'dots', skirt: true, shoes: '#4e342e' }
};

const EYE = '#202020';

function hairdo(o, L, dir) {
	if (L.style === 'cap') {
		o.rect(8, 1, 16, 6, L.hat); // crown
		o.rect(8, 6, 16, 2, L.hair); // underside shade
		o.rect(8, 1, 16, 1, L.shirt); // crown highlight band
		if (dir === 'down') {
			o.rect(13, 3, 6, 4, '#f7f7f7'); // white front panel
			o.rect(8, 7, 16, 2, L.hat); // brim front
			o.rect(8, 8, 16, 1, L.hair);
		} else if (dir === 'side') {
			o.rect(14, 3, 4, 4, '#f7f7f7');
			o.rect(20, 3, 6, 4, L.hat); // brim points +x
			o.rect(20, 6, 6, 1, L.hair);
		} else {
			o.px(15, 0, L.hair); // cap button from behind
			o.px(16, 0, L.hair);
		}
		if (dir !== 'side') {
			o.rect(10, 8, 2, 6, L.hair); // side hair L
			o.rect(20, 8, 2, 6, L.hair); // side hair R
		} else {
			o.rect(8, 6, 4, 10, L.hair); // hair mass on the eyeless back side
		}
	} else if (L.style === 'long') {
		if (dir === 'side') {
			o.rect(10, 2, 12, 4, L.hair);
			o.rect(8, 4, 5, 13, L.hair); // heavy back curtain, face stays open
		} else {
			o.rect(10, 2, 12, 4, L.hair);
			o.rect(8, 4, 4, 13, L.hair); // falls past shoulders
			o.rect(20, 4, 4, 13, L.hair);
			if (dir === 'up') {
				o.rect(10, 6, 12, 10, L.hair); // back of head — no skin showing
				o.shade(18, 6, 4, 10, 0.14); // strand depth
			}
		}
	} else if (L.style === 'bald') {
		o.rect(10, 5, 12, 2, L.hair); // fringe
		if (dir === 'side') {
			o.px(10, 8, L.hair);
		} else {
			o.px(10, 8, L.hair);
			o.px(21, 8, L.hair);
		}
	} else {
		// short
		o.rect(10, 2, 12, 4, L.hair);
		if (dir === 'side') {
			o.rect(10, 6, 3, 4, L.hair); // back only, face open
		} else {
			o.rect(10, 6, 2, 4, L.hair);
			o.rect(20, 6, 2, 4, L.hair);
		}
	}
}

export function paintActor(o, L, dir, phase) {
	const spread = phase ? 2 : 0; // legs apart on stride frame
	const swing = phase ? 2 : 0; // arms swing opposite

	if (dir === 'side') {
		// Profile view (+x): narrow torso, one front arm, offset legs.
		const backLX = 10 - spread;
		const frontLX = 14 + spread;
		const legC = L.skirt ? L.skin : L.pants;
		const legH = L.skirt ? 4 : 6;
		o.rect(backLX, 24, 4, legH, legC);
		o.rect(frontLX, 24, 4, legH, legC);
		o.rect(backLX, 30, 4, 2, L.shoes);
		o.rect(frontLX, 30, 4, 2, L.shoes);
		o.rect(backLX, 31, 4, 1, '#241a12');
		o.rect(frontLX, 31, 4, 1, '#241a12');
		// Slim torso, shade on the back edge.
		o.rect(10, 14, 12, 10, L.shirt);
		if (L.shirtTex) o.coat(L.shirtTex, 10, 14, 12, 10, 0.7);
		o.shade(10, 14, 2, 10, 0.14);
		if (L.skirt) {
			o.rect(9, 22, 14, 6, L.shirt);
			if (L.shirtTex) o.coat(L.shirtTex, 9, 22, 14, 6, 0.7);
			o.shade(9, 22, 2, 6, 0.14);
		}
		// Front arm only, swinging with the stride.
		o.rect(20, 14 + (2 - swing), 4, 7, L.shirt);
		o.rect(20, 21 + (2 - swing), 4, 2, L.skin);
	} else {
	const legLX = 9 - spread;
	const legRX = 18 + spread;

	// Legs + shoes (skin when skirted).
	const legC = L.skirt ? L.skin : L.pants;
	const legY = 24;
	const legH = L.skirt ? 4 : 6;
	o.rect(legLX, legY, 5, legH, legC);
	o.rect(legRX, legY, 5, legH, legC);
	o.rect(legLX, 30, 5, 2, L.shoes);
	o.rect(legRX, 30, 5, 2, L.shoes);
	o.rect(legLX, 31, 5, 1, '#241a12'); // undersole
	o.rect(legRX, 31, 5, 1, '#241a12');

	// Torso with fabric texture + right-edge shade.
	o.rect(8, 14, 16, 10, L.shirt);
	if (L.shirtTex) o.coat(L.shirtTex, 8, 14, 16, 10, 0.7);
	o.shade(21, 14, 3, 10, 0.14);
	if (L.skirt) {
		o.rect(6, 22, 20, 6, L.shirt);
		if (L.shirtTex) o.coat(L.shirtTex, 6, 22, 20, 6, 0.7);
		o.shade(22, 22, 4, 6, 0.14);
	}

	// Arms: sleeves + skin hands, swinging with the stride.
	o.rect(4, 14 + swing, 4, 7, L.shirt);
	o.rect(24, 14 + (2 - swing), 4, 7, L.shirt);
	o.rect(4, 21 + swing, 4, 2, L.skin);
	o.rect(24, 21 + (2 - swing), 4, 2, L.skin);
	}

	// Head + face toward `dir`.
	o.rect(10, 6, 12, 10, L.skin);
	o.shade(20, 6, 2, 10, 0.1);
	hairdo(o, L, dir);
	if (dir === 'down') {
		o.rect(13, 10, 2, 2, EYE);
		o.rect(18, 10, 2, 2, EYE);
		o.rect(15, 14, 2, 1, '#8e3b3b'); // mouth
		o.rect(11, 12, 2, 1, '#e8656b'); // blush L
		o.rect(20, 12, 2, 1, '#e8656b'); // blush R
	} else if (dir === 'side') {
		o.rect(19, 10, 2, 2, EYE);
	}
}

/** Browser binding: paint every look × dir × phase into outlined canvases.
 * Frames are 34x34 (1px dark outline baked around the 32x32 sprite,
 * same as the Ash matrices); draw them at (x-1, y-1).
 * `shadow` is a shared ground blob. */
export function buildActors() {
	const OUTLINE = '#1e2b1a';
	const opsFor = (g) => ({
		rect: (x, y, w, h, col) => {
			g.fillStyle = col;
			g.fillRect(x, y, w, h);
		},
		px: (x, y, col) => {
			g.fillStyle = col;
			g.fillRect(x, y, 1, 1);
		},
		coat: (id, x, y, w, h, a) => coatTex(g, id, x, y, w, h, a),
		shade: (x, y, w, h, a) => {
			g.fillStyle = `rgba(0,0,20,${a})`;
			g.fillRect(x, y, w, h);
		}
	});
	const monoOps = (g) => ({
		rect: (x, y, w, h) => {
			g.fillStyle = OUTLINE;
			g.fillRect(x, y, w, h);
		},
		px: (x, y) => {
			g.fillStyle = OUTLINE;
			g.fillRect(x, y, 1, 1);
		},
		coat: () => {},
		shade: () => {}
	});
	const out = {};
	// Player: blit Ash matrices (outline baked in, 2px margin like NPCs).
	for (const [dir, frames] of Object.entries(buildAshFrames())) {
		out.player ??= {};
		out.player[dir] = frames.map((mat) => {
			const c = document.createElement('canvas');
			c.width = 34;
			c.height = 34;
			const g = c.getContext('2d');
			for (let y = 0; y < 32; y++) {
				for (let x = 0; x < 32; x++) {
					const v = mat[y][x];
					if (!v) continue;
					g.fillStyle = ASH_PAL[v];
					// +3 vertical: feet land on the tile bottom like NPCs.
					g.fillRect(x + 1, y + 3, 1, 1);
				}
			}
			return c;
		});
	}
	for (const [name, look] of Object.entries(LOOKS)) {
		out[name] = {};
		for (const dir of ['down', 'up', 'side']) {
			out[name][dir] = [0, 1].map((phase) => {
				const body = document.createElement('canvas');
				body.width = 32;
				body.height = 32;
				paintActor(opsFor(body.getContext('2d')), look, dir, phase);
				const sil = document.createElement('canvas');
				sil.width = 32;
				sil.height = 32;
				paintActor(monoOps(sil.getContext('2d')), look, dir, phase);
				const c = document.createElement('canvas');
				c.width = 34;
				c.height = 34;
				const g = c.getContext('2d');
				g.drawImage(sil, 0, 1);
				g.drawImage(sil, 2, 1);
				g.drawImage(sil, 1, 0);
				g.drawImage(sil, 1, 2);
				g.drawImage(body, 1, 1);
				return c;
			});
		}
	}
	const sh = document.createElement('canvas');
	sh.width = 26;
	sh.height = 9;
	const sg = sh.getContext('2d');
	sg.fillStyle = 'rgba(0,0,20,0.28)';
	sg.beginPath();
	sg.ellipse(13, 4, 12, 4, 0, 0, 7);
	sg.fill();
	out.shadow = sh;
	return out;
}
