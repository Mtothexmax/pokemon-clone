// Interiors are built once at startup far away from the village (x >= 200).
// Entering a house is just a teleport + camera snap -> zero loading time.
// Rooms are deliberately much bigger than the houses look from outside.
import * as THREE from 'three';
import type { Town } from './town';
import { cardArt } from '../game/art';

export type AreaId = 'outside' | 'home' | 'dex' | 'lab' | 'bedroom';

export interface Rect {
	x0: number;
	x1: number;
	z0: number;
	z1: number;
}

export interface Room {
	id: AreaId;
	label: string;
	ox: number;
	oz: number;
	W: number;
	D: number;
	walk: Rect;
	/** Outside door: x, z of the door face (doors face +z / the camera). */
	door: { x: number; z: number };
	/** reached by the stairs inside a house, no outside door */
	upstairs?: boolean;
}

export const ROOMS: Room[] = [
	mkRoom('home', 'Home', 200, 0, 26, 18, { x: -15, z: -6 }),
	mkRoom('dex', "Dex & Lumi's House", 260, 0, 26, 18, { x: 47, z: -11.2 }),
	mkRoom('lab', 'Quill Lab', 330, 0, 34, 22, { x: 13, z: 16 }),
	{ ...mkRoom('bedroom', 'Your Room', 200, 40, 18, 14, { x: 0, z: 0 }), upstairs: true }
];

/** walkable stairs in the home (ascending towards -z); the top leads to the bedroom */
export const HOME_STAIRS = { x0: 209.1, x1: 212.1, z0: -4.4, z1: 2.1 };
/** stairwell in the bedroom (descending towards +z); walking into it leads back down */
export const BED_WELL = { x0: 205.5, x1: 208.5, z0: 42, z1: 46 };

function mkRoom(id: AreaId, label: string, ox: number, oz: number, W: number, D: number, door: { x: number; z: number }): Room {
	return {
		id, label, ox, oz, W, D, door,
		walk: { x0: ox - W / 2 + 0.9, x1: ox + W / 2 - 0.9, z0: oz - D / 2 + 1.0, z1: oz + D / 2 - 0.6 }
	};
}
export const roomById = (id: AreaId) => ROOMS.find((r) => r.id === id)!;

/** Builds all interiors. Returns emissive materials that should blink. */
export function buildInteriors(t: Town) {
	const blink: THREE.MeshStandardMaterial[] = [];

	const furn = (w: number, h: number, d: number, c: string | THREE.Material, x: number, y: number, z: number, block = true, studs = false) => {
		t.box(w, h, d, c, x, y, z, { studs });
		if (block) t.blockers.push({ x0: x - w / 2, x1: x + w / 2, z0: z - d / 2, z1: z + d / 2 });
	};

	function shell(r: Room, floorA: string, floorB: string, wall: string, trim: string, tiles = false, door = true) {
		const { ox, oz, W, D } = r;
		// floor: wooden planks or checker tiles (smooth "tile" pieces)
		if (tiles) {
			for (let x = -W / 2; x < W / 2; x += 2)
				for (let z = -D / 2; z < D / 2; z += 2)
					t.box(2, 0.4, 2, ((x + z) / 2) % 2 === 0 ? floorA : floorB, ox + x + 1, 0, oz + z + 1, { shadow: false });
		} else {
			for (let x = -W / 2; x < W / 2; x += 1)
				t.box(1, 0.4, D, Math.abs(x) % 2 === 0 ? floorA : floorB, ox + x + 0.5, 0, oz, { shadow: false });
		}
		const H = 9;
		// back wall + wainscot + trim
		t.box(W + 2, H, 1, wall, ox, 0, oz - D / 2 - 0.5);
		t.box(W, 2.4, 0.3, trim, ox, 0.4, oz - D / 2 + 0.1);
		t.box(W + 2, 0.4, 1.4, trim, ox, H, oz - D / 2 - 0.5);
		// side walls (lower towards the front so the camera sees inside)
		for (const s of [-1, 1]) {
			t.box(1, H, D + 1, wall, ox + s * (W / 2 + 0.5), 0, oz - 0.5);
			t.box(0.3, 2.4, D, trim, ox + s * (W / 2 - 0.1), 0.4, oz);
		}
		// low front wall with the door gap
		if (door) {
			const seg = W / 2 - 1.6;
			for (const s of [-1, 1]) t.box(seg + 1, 1.2, 1, wall, ox + s * (1.6 + seg / 2 + 0.5), 0, oz + D / 2 + 0.5, { studs: true });
			// door mat + door frame
			t.box(3, 0.1, 1.6, '#c91a09', ox, 0.4, oz + D / 2 - 0.6, { shadow: false });
			for (const s of [-1, 1]) t.box(0.5, 1.6, 1.2, trim, ox + s * 1.8, 0, oz + D / 2 + 0.5, { studs: true });
		} else t.box(W + 2, 1.2, 1, wall, ox, 0, oz + D / 2 + 0.5, { studs: true });
		// dark void around the room
		const under = new THREE.Mesh(new THREE.PlaneGeometry(160, 160), t.mat('#151f27', { roughness: 1 }));
		under.rotation.x = -Math.PI / 2;
		under.position.set(ox, -0.05, oz);
		t.scene.add(under);
		// warm ceiling light
		const light = new THREE.PointLight('#fff1d6', 120, 40, 1.6);
		light.position.set(ox, 11, oz);
		t.scene.add(light);
	}

	function windowOnBack(r: Room, x: number, w = 4, h = 3) {
		const z = r.oz - r.D / 2;
		t.box(w + 0.6, h + 0.6, 0.3, '#f4f4f4', r.ox + x, 3.2, z + 0.1, { shadow: false });
		t.box(w, h, 0.3, t.mat('#bfe9ff', { emissive: '#9fd8ff', emissiveIntensity: 0.6 }), r.ox + x, 3.5, z + 0.2, { shadow: false });
		t.box(0.2, h, 0.35, '#f4f4f4', r.ox + x, 3.5, z + 0.3, { shadow: false });
	}

	function sign(r: Room, text: string, x: number, y: number, w: number, bg: string, fg: string) {
		const m = new THREE.Mesh(new THREE.PlaneGeometry(w, w / 4), new THREE.MeshStandardMaterial({ map: t.signTexture(text, bg, fg), roughness: 0.4 }));
		m.position.set(r.ox + x, y, r.oz - r.D / 2 + 0.06);
		t.scene.add(m);
	}

	function frame(r: Room, x: number, y: number, c: string) {
		const z = r.oz - r.D / 2;
		t.box(2.2, 1.8, 0.2, '#583927', r.ox + x, y, z + 0.1, { shadow: false });
		t.box(1.8, 1.4, 0.25, c, r.ox + x, y + 0.2, z + 0.15, { shadow: false });
	}

	function plant(x: number, z: number, s = 2.2, bamboo = false) {
		t.box(1.4, 1.2, 1.4, '#82422a', x, 0.4, z, { studs: false });
		if (bamboo) t.bambooSpots.push({ x, y: 1.6, z, s: s * 0.75 });
		else t.bushSpots.push({ x, y: 1.5, z, s });
		t.blockers.push({ x0: x - 0.8, x1: x + 0.8, z0: z - 0.8, z1: z + 0.8 });
	}

	function books(x: number, z: number, w: number, h: number, y0: number) {
		const cols = ['#c91a09', '#0055bf', '#f2cd37', '#237841', '#f4f4f4', '#fe8a18'];
		for (let i = 0; i < Math.floor(w / 0.5); i++) {
			const bh = h * (0.7 + Math.random() * 0.3);
			t.box(0.4, bh, 0.9, cols[(i * 7) % cols.length], x - w / 2 + 0.25 + i * 0.5, y0, z, { shadow: false });
		}
	}

	function bookshelf(x: number, z: number, w: number) {
		furn(w, 7, 1.4, '#583927', x, 0.4, z);
		for (let i = 0; i < 4; i++) {
			t.box(w - 0.4, 0.2, 1.2, '#82422a', x, 1.2 + i * 1.6, z + 0.15, { shadow: false });
			books(x, z + 0.3, w - 0.6, 1.2, 1.4 + i * 1.6);
		}
	}

	function screen(x: number, y: number, z: number, w: number, h: number, color = '#40e0a0') {
		const m = new THREE.MeshStandardMaterial({ color: '#0a1a14', emissive: color, emissiveIntensity: 0.8, roughness: 0.2 });
		blink.push(m);
		t.box(w + 0.3, h + 0.3, 0.3, '#1b2a34', x, y - 0.15, z, { shadow: false });
		t.box(w, h, 0.32, m, x, y, z + 0.03, { shadow: false });
	}

	// ======================= HOME =======================
	t.beginSection('interior:home');
	{
		const r = roomById('home');
		const { ox, oz } = r;
		shell(r, '#c8955c', '#b88550', '#f4e6c8', '#82422a');
		windowOnBack(r, -4);
		windowOnBack(r, 5);
		frame(r, 0.6, 5, '#7fc0ff');
		frame(r, 9.5, 5, '#ffb070');
		// kitchen (back left)
		furn(2.6, 6.4, 2.2, '#f4f4f4', ox - 11.4, 0.4, oz - 7.4); // fridge
		t.box(0.2, 2, 0.2, '#a0a5a9', ox - 10.4, 3.4, oz - 6.2, { shadow: false });
		furn(7, 2.6, 2.2, '#f4f4f4', ox - 6.6, 0.4, oz - 7.4);
		t.box(7.2, 0.3, 2.4, '#6c6e68', ox - 6.6, 3.0, oz - 7.4, { studs: true });
		t.box(2.4, 0.15, 1.6, '#1b2a34', ox - 4.6, 3.3, oz - 7.4, { shadow: false }); // stove
		for (const dx of [-0.6, 0.6]) t.box(0.7, 0.1, 0.7, '#c91a09', ox - 4.6 + dx, 3.45, oz - 7.4, { shadow: false });
		// dining table + chairs
		furn(5, 0.4, 3, '#82422a', ox - 6, 2.4, oz - 1, true, true);
		for (const [dx, dz] of [[-2.1, -1.1], [2.1, -1.1], [-2.1, 1.1], [2.1, 1.1]]) t.box(0.4, 2.0, 0.4, '#583927', ox - 6 + dx, 0.4, oz - 1 + dz);
		for (const dx of [-1.3, 1.3])
			for (const s of [-1, 1]) {
				t.box(1.6, 0.3, 1.4, '#c91a09', ox - 6 + dx, 1.8, oz - 1 + s * 2.4, { studs: true });
				t.box(1.6, 1.8, 0.3, '#c91a09', ox - 6 + dx, 2.1, oz - 1 + s * 3.0);
			}
		// living room: rug, sofa, TV
		t.box(9, 0.12, 7, '#3d6fd1', ox + 5, 0.4, oz + 0.5, { shadow: false });
		t.box(7.6, 0.13, 5.6, '#6fa0ff', ox + 5, 0.41, oz + 0.5, { shadow: false });
		furn(6, 1.4, 2.2, '#c91a09', ox + 5, 0.4, oz + 4.8, true, true);
		t.box(6, 2.2, 0.7, '#c91a09', ox + 5, 1.8, oz + 5.6);
		for (const s of [-1, 1]) t.box(0.7, 2.2, 2.2, '#a01408', ox + 5 + s * 3.2, 0.4, oz + 4.8, { studs: true });
		furn(5, 1.6, 1.6, '#583927', ox + 5, 0.4, oz - 7.4);
		t.box(4.2, 2.6, 0.4, '#1b2a34', ox + 5, 2.0, oz - 7.6);
		screen(ox + 5, 3.3, oz - 7.3, 3.8, 2.1, '#3da0ff');
		{
			// invisible face on the TV: first-person close-up target (the page shows an HTML/SVG TV show on it)
			const face = new THREE.Mesh(new THREE.PlaneGeometry(3.8, 2.1), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false }));
			face.position.set(ox + 5, 4.35, oz - 7.1);
			t.scene.add(face);
			t.registerView('tv:home', face, 3.8, 2.1);
		}
		t.box(1.2, 0.4, 0.8, '#a0a5a9', ox + 5, 2.0, oz - 6.9, { studs: true }); // console
		// coffee table
		furn(3, 1.2, 1.6, '#82422a', ox + 5, 0.4, oz + 1.2, true, true);
		// bookshelf + stairs
		bookshelf(ox + 10.6, oz - 8, 3);
		for (let i = 0; i < 6; i++) t.box(3, 0.6 * (i + 1), 1, '#c8955c', ox + 10.6, 0.4, oz + 7.6 - i * 1 - 6, { studs: true });
		// the stairs are walkable (see HOME_STAIRS); a banister on the open side
		for (let i = 0; i < 6; i++) t.box(0.25, 1.6, 0.25, '#f4f4f4', ox + 9.0, 0.4 + 0.6 * (i + 1), oz + 1.6 - i, { shadow: false });
		{
			const rail = new THREE.Mesh(t.rbox(0.35, 0.3, 6.6), t.mat('#82422a'));
			rail.position.set(ox + 9.0, 0.4 + 0.6 * 3.5 + 1.6, oz - 1.0);
			rail.rotation.x = Math.atan2(0.6, 1);
			t.scene.add(rail);
		}
		t.blockers.push({ x0: ox + 8.7, x1: ox + 9.1, z0: oz - 4.4, z1: oz + 1.1 });
		t.blockers.push({ x0: ox + 9, x1: ox + 12.2, z0: oz - 7.2, z1: oz - 4.4 }); // landing behind the stairs
		// sign so you know where it goes
		{
			const m = new THREE.Mesh(new THREE.PlaneGeometry(2.6, 0.65), new THREE.MeshStandardMaterial({ map: t.signTexture('MY ROOM \u25b2', '#f2cd37', '#1b2a34'), roughness: 0.4 }));
			m.position.set(ox + 12.9, 5.2, oz - 1);
			m.rotation.y = -Math.PI / 2;
			t.scene.add(m);
		}
		plant(ox - 11.6, oz + 6.8);
		plant(ox + 11.6, oz + 6.8, 2.2, true);
		t.flowerPot(ox - 6, oz - 1, 2.8 + 0.2125, false); // on the dining table
		plant(ox - 1.2, oz - 7.6, 1.8);
	}

	t.endSection();
	// ======================= BEDROOM (upstairs at home) =======================
	t.beginSection('interior:bedroom');
	{
		const r = roomById('bedroom');
		const { ox, oz } = r;
		shell(r, '#c8955c', '#b88550', '#d8ecff', '#0055bf', false, false);
		windowOnBack(r, -1.5);
		frame(r, -6.5, 5.2, '#f2cd37');
		frame(r, 3.4, 5.2, '#c91a09');
		// bed (back left) with a blanket in the player's teal
		furn(4, 1.2, 6.6, '#82422a', ox - 6.4, 0.4, oz - 3.4);
		t.box(4, 3, 0.4, '#583927', ox - 6.4, 0.4, oz - 6.7);
		t.box(3.6, 0.6, 5.9, '#f4f4f4', ox - 6.4, 1.6, oz - 3.45, { shadow: false }); // mattress ends behind the blanket front (no z-fighting)
		t.box(3.8, 0.7, 4, '#1b9e8f', ox - 6.4, 1.6, oz - 2.2);
		t.box(2.6, 0.5, 1.3, '#f4f4f4', ox - 6.4, 2.2, oz - 5.6);
		// desk + PC + chair (back middle)
		furn(4.6, 2.4, 2, '#f4f4f4', ox + 1.4, 0.4, oz - 5.9, true, true);
		screen(ox + 1.4, 4.0, oz - 6.5, 2.6, 1.6, '#3da0ff');
		{
			// invisible face on the monitor: target of the first-person close-up (the page draws an HTML desktop on it)
			const face = new THREE.Mesh(new THREE.PlaneGeometry(2.6, 1.6), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false }));
			face.position.set(ox + 1.4, 4.8, oz - 6.3);
			t.scene.add(face);
			t.registerView('pc:bedroom', face, 2.6, 1.6);
		}
		t.box(1.8, 0.15, 0.6, '#1b2a34', ox + 1.4, 2.8, oz - 5.4, { shadow: false });
		t.box(1.5, 1.4, 1.5, '#c91a09', ox + 1.4, 0.4, oz - 3.9, { studs: true });
		t.box(1.5, 1.8, 0.3, '#c91a09', ox + 1.4, 1.8, oz - 3.3);
		// wardrobe (back right)
		furn(3.2, 6.4, 1.8, '#82422a', ox + 6.6, 0.4, oz - 5.8);
		for (const s of [-1, 1]) t.box(0.2, 0.9, 0.2, '#f2cd37', ox + 6.6 + s * 0.35, 3.4, oz - 4.85, { shadow: false });
		// rug + toy shelf with little brick beasts
		t.box(6, 0.12, 4.4, '#f2cd37', ox - 1.2, 0.4, oz + 1.6, { shadow: false });
		t.box(5, 0.13, 3.4, '#fe8a18', ox - 1.2, 0.41, oz + 1.6, { shadow: false });
		furn(1.4, 3.2, 4, '#583927', ox - 8.2, 0.4, oz + 3.4, true, true);
		const toys = ['#c91a09', '#0055bf', '#237841', '#f2cd37', '#7b2fbe'];
		toys.forEach((c, i) => t.box(0.8, 0.8, 0.6, c, ox - 8.2, 3.6, oz + 1.8 + i * 0.8, { studs: true }));
		plant(ox - 3.4, oz - 5.8, 1.8);
		// stairwell down (front right): dark opening, steps fading into the dark, railings
		const { x0, x1, z0, z1 } = BED_WELL;
		const cx = (x0 + x1) / 2;
		t.box(x1 - x0, 0.06, z1 - z0, '#151f27', cx, 0.4, (z0 + z1) / 2, { shadow: false });
		const shades = ['#c8955c', '#a87a48', '#7a5834', '#4d3822'];
		shades.forEach((c, i) => t.box(x1 - x0 - 0.2, 0.05, 0.8, c, cx, 0.43, z0 + 0.5 + i * 0.95, { shadow: false }));
		t.box(0.3, 1.6, z1 - z0 + 0.3, '#f4f4f4', x0 - 0.15, 0.4, (z0 + z1) / 2);
		t.box(x1 - x0 + 0.3, 1.6, 0.3, '#f4f4f4', cx, 0.4, z1 + 0.15);
		t.blockers.push({ x0: x0 - 0.3, x1: x0, z0, z1 }, { x0: x0 - 0.3, x1, z0: z1, z1: z1 + 0.3 });
	}
	t.endSection();
	// ======================= DEX & LUMI =======================
	t.beginSection('interior:dex');
	{
		const r = roomById('dex');
		const { ox, oz } = r;
		shell(r, '#e4cd9e', '#d8c090', '#d8e8f4', '#0055bf');
		windowOnBack(r, -7);
		windowOnBack(r, 7);
		sign(r, 'DEX IS #1', 0, 6.4, 5, '#7b2fbe', '#f2cd37');
		// trophy shelf
		furn(8, 0.4, 1.4, '#583927', ox, 3.6, oz - 7.8, false);
		const gold = t.mat('#f2cd37', { metalness: 0.7, roughness: 0.2 });
		for (let i = 0; i < 4; i++) {
			const cup = new THREE.Mesh(new THREE.CylinderGeometry(0.45, 0.2, 0.9, 14), gold);
			cup.position.set(ox - 3 + i * 2, 4.6, oz - 7.8);
			const base = new THREE.Mesh(new THREE.CylinderGeometry(0.3, 0.35, 0.3, 12), gold);
			base.position.set(ox - 3 + i * 2, 4.15, oz - 7.8);
			cup.castShadow = true;
			t.scene.add(cup, base);
		}
		// piano
		furn(5, 2.6, 2, '#1b2a34', ox - 9, 0.4, oz - 7.2);
		t.box(5, 2.2, 0.6, '#1b2a34', ox - 9, 3.0, oz - 7.9);
		for (let i = 0; i < 9; i++) t.box(0.45, 0.2, 0.8, '#f4f4f4', ox - 11 + i * 0.5, 3.0, oz - 6.5, { shadow: false });
		furn(2.4, 1.2, 1.2, '#1b2a34', ox - 9, 0.4, oz - 5.0, true, true);
		{
			// invisible face over the piano front: first-person close-up target (the page draws playable keys on it)
			const face = new THREE.Mesh(new THREE.PlaneGeometry(4.8, 2.1), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false }));
			face.position.set(ox - 9, 4.1, oz - 7.55);
			t.scene.add(face);
			t.registerView('piano:dex', face, 4.8, 2.1);
		}
		// gaming corner
		furn(5, 0.4, 2.2, '#f4f4f4', ox + 9, 2.6, oz - 7.2, false, true);
		t.blockers.push({ x0: ox + 6.5, x1: ox + 11.5, z0: oz - 8.3, z1: oz - 6.1 });
		for (const s of [-1, 1]) t.box(0.4, 2.2, 2, '#f4f4f4', ox + 9 + s * 2.2, 0.4, oz - 7.2);
		screen(ox + 9, 4.3, oz - 7.8, 3, 1.8, '#ff6ad5');
		{
			// the gaming TV can be watched too (same TV show as at home)
			const face = new THREE.Mesh(new THREE.PlaneGeometry(3, 1.8), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false }));
			face.position.set(ox + 9, 5.2, oz - 7.6);
			t.scene.add(face);
			t.registerView('tv:dex', face, 3, 1.8);
		}
		t.box(1.8, 1.5, 1.8, '#7b2fbe', ox + 9, 0.4, oz - 4.8, { studs: true });
		t.box(1.8, 2, 0.4, '#7b2fbe', ox + 9, 1.9, oz - 4.0);
		// couch, rug, table
		t.box(10, 0.12, 6, '#7b2fbe', ox, 0.4, oz - 0.5, { shadow: false });
		furn(6, 1.4, 2.2, '#237841', ox, 0.4, oz + 3.2, true, true);
		t.box(6, 2.2, 0.7, '#237841', ox, 1.8, oz + 4.0);
		furn(3.4, 1.2, 2, '#f4f4f4', ox, 0.4, oz - 0.9, true, true);
		// beds (two small ones)
		for (const s of [-1, 1]) {
			furn(3, 1.2, 5, s < 0 ? '#ff9ecd' : '#3d6fd1', ox + s * 11, 0.4, oz + 3.5, true, true);
			t.box(2.4, 0.5, 1.2, '#f4f4f4', ox + s * 11, 1.6, oz + 1.6, { shadow: false });
		}
		plant(ox - 4, oz - 7.4, 1.8);
		t.flowerPot(ox, oz - 0.9, 1.6 + 0.2125, false); // on the coffee table
		plant(ox + 4.5, oz - 7.4, 1.8, true);
	}

	t.endSection();
	// ======================= LAB =======================
	t.beginSection('interior:lab');
	{
		const r = roomById('lab');
		const { ox, oz } = r;
		shell(r, '#f4f4f4', '#d8dde0', '#f4f4f4', '#3d6fd1', true);
		sign(r, 'QUILL LAB', 0, 6.6, 7, '#f4f4f4', '#0055bf');
		windowOnBack(r, -4, 3, 2.4);
		windowOnBack(r, 4, 3, 2.4);
		// bookshelves (back left)
		bookshelf(ox - 13, oz - 10, 5);
		bookshelf(ox - 7.6, oz - 10, 5);
		// machines (back right) with blinking lights
		for (let i = 0; i < 3; i++) {
			const x = ox + 8 + i * 3.6;
			furn(3.2, 6, 2.2, '#a0a5a9', x, 0.4, oz - 9.6);
			screen(x, 4.4, oz - 8.4, 2.2, 1.4, ['#40e0a0', '#ffd23d', '#ff6a5a'][i]);
			for (let j = 0; j < 3; j++) {
				const m = new THREE.MeshStandardMaterial({ color: '#222', emissive: ['#ff3a3a', '#3aff6a', '#3a9aff'][j], emissiveIntensity: 1, roughness: 0.3 });
				blink.push(m);
				t.box(0.4, 0.4, 0.2, m, x - 0.8 + j * 0.8, 2.2, oz - 8.4, { shadow: false });
			}
		}
		// aquariums along the left wall
		for (let i = 0; i < 2; i++) {
			const z = oz - 2 + i * 6;
			furn(2.4, 1.6, 4.6, '#1b2a34', ox - 15.4, 0.4, z);
			const glass = new THREE.Mesh(t.rbox(2.2, 3, 4.4), t.mat('#4ab0ff', { transparent: true, opacity: 0.45, roughness: 0.05 }));
			glass.position.set(ox - 15.4, 3.5, z);
			t.scene.add(glass);
			t.box(2.0, 0.6, 4.2, '#e4cd9e', ox - 15.4, 2.0, z, { shadow: false });
			t.fishes.push(t.box(0.7, 0.35, 0.3, '#fe8a18', ox - 15.4, 3.4, z, { shadow: false }));
		}
		// computer desks along the right wall
		for (let i = 0; i < 2; i++) {
			const z = oz - 2 + i * 5.5;
			furn(2.6, 0.4, 4, '#f4f4f4', ox + 15.2, 2.6, z, true, true);
			for (const s of [-1, 1]) t.box(2.4, 2.2, 0.3, '#f4f4f4', ox + 15.2, 0.4, z + s * 1.8);
			// a real LEGO computer (from the office set) stands on the desk, facing the room (loaded with the models)
			t.labComputers.push({ x: ox + 15.2, y: 3.0, z: z - 0.2, s: 1.2 });
		}
		// central table with three starter cards on glowing pedestals
		furn(10, 0.5, 3.6, '#82422a', ox, 2.6, oz - 1.2, true, true);
		for (const [dx, dz] of [[-4.5, -1.4], [4.5, -1.4], [-4.5, 1.4], [4.5, 1.4]]) t.box(0.5, 2.2, 0.5, '#583927', ox + dx, 0.4, oz - 1.2 + dz);
		['flicker', 'drizzlet', 'sproutle'].forEach((id, i) => {
			const x = ox - 3 + i * 3;
			const ped = new THREE.MeshStandardMaterial({ color: '#ddd', emissive: ['#ff7a3d', '#3d9bff', '#5ccb4c'][i], emissiveIntensity: 0.7 });
			blink.push(ped);
			t.box(1.6, 0.6, 1.6, ped, x, 3.1, oz - 1.2, { shadow: false });
			const tex = new THREE.TextureLoader().load(cardArt(id));
			tex.colorSpace = THREE.SRGBColorSpace;
			const card = new THREE.Mesh(new THREE.PlaneGeometry(1.5, 1.25), new THREE.MeshStandardMaterial({ map: tex, side: THREE.DoubleSide, emissive: '#ffffff', emissiveMap: tex, emissiveIntensity: 0.35 }));
			card.position.set(x, 4.6, oz - 1.2);
			card.userData.spin = i;
			t.spinners.push(card);
			t.scene.add(card);
		});
		t.flowerPot(ox - 15.4, oz + 8.4);
		plant(ox + 15.4, oz + 8.6, 2.2, true);
		plant(ox + 1.2, oz - 9.8, 1.8);
	}

	t.endSection();
	return blink;
}
