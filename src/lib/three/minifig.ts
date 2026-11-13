import * as THREE from 'three';
import { loadModel } from './models';

export interface FigStyle {
	torso: string;
	legs: string;
	skin?: string;
	hair: string;
	hairStyle: 'cap' | 'spiky' | 'ponytail' | 'bob' | 'gray' | 'short';
	cap?: string;
	coat?: boolean; // lab coat overlay
	decal?: 'zip' | 'apron' | 'tie' | 'star' | 'stripe';
	decalColor?: string;
	female?: boolean;
}

const SKIN = '#f5c72c';
/** arms are held slightly forward at rest, like a real minifig */
const ARM_REST = -0.22;

function m(color: string, opts: Partial<THREE.MeshStandardMaterialParameters> = {}) {
	return new THREE.MeshStandardMaterial({ color, roughness: 0.32, metalness: 0, ...opts });
}

function torsoCanvas(style: FigStyle) {
	const c = document.createElement('canvas');
	c.width = 128;
	c.height = 128;
	const g = c.getContext('2d')!;
	g.fillStyle = style.coat ? '#f4f4f4' : style.torso;
	g.fillRect(0, 0, 128, 128);
	const dc = style.decalColor ?? '#ffffff';
	if (style.coat) {
		// shirt + tie visible between lapels
		g.fillStyle = style.torso;
		g.beginPath();
		g.moveTo(44, 0);
		g.lineTo(84, 0);
		g.lineTo(64, 70);
		g.closePath();
		g.fill();
		g.fillStyle = dc;
		g.fillRect(60, 6, 8, 46);
		g.strokeStyle = '#999';
		g.lineWidth = 3;
		g.beginPath();
		g.moveTo(44, 0);
		g.lineTo(64, 70);
		g.lineTo(64, 128);
		g.moveTo(84, 0);
		g.lineTo(64, 70);
		g.stroke();
		g.fillStyle = '#3d9bff';
		g.fillRect(84, 60, 18, 4);
	}
	switch (style.decal) {
		case 'zip':
			g.fillStyle = dc;
			g.fillRect(61, 0, 6, 128);
			g.fillRect(14, 30, 28, 8);
			g.fillRect(86, 30, 28, 8);
			break;
		case 'apron':
			g.fillStyle = dc;
			g.beginPath();
			g.roundRect(30, 36, 68, 92, 10);
			g.fill();
			g.fillStyle = '#e65a8a';
			for (let i = 0; i < 3; i++) {
				g.beginPath();
				g.arc(48 + i * 16, 70, 5, 0, Math.PI * 2);
				g.fill();
			}
			break;
		case 'tie':
			g.fillStyle = dc;
			g.beginPath();
			g.moveTo(58, 0);
			g.lineTo(70, 0);
			g.lineTo(74, 70);
			g.lineTo(64, 84);
			g.lineTo(54, 70);
			g.fill();
			break;
		case 'star': {
			g.fillStyle = dc;
			g.beginPath();
			for (let i = 0; i < 10; i++) {
				const r = i % 2 ? 10 : 24;
				const a = (i / 10) * Math.PI * 2 - Math.PI / 2;
				g.lineTo(64 + Math.cos(a) * r, 60 + Math.sin(a) * r);
			}
			g.fill();
			break;
		}
		case 'stripe':
			g.fillStyle = dc;
			g.fillRect(0, 44, 128, 18);
			g.fillRect(0, 76, 128, 8);
			break;
	}
	return c;
}

// ---------- detailed minifigure model (split into parts by scripts/convert-models.mjs) ----------
type PartName = 'head' | 'torso' | 'armL' | 'armR' | 'legL' | 'legR';
const PARTS: PartName[] = ['head', 'torso', 'armL', 'armR', 'legL', 'legR'];
// pivots in figure space (feet at y=0, 4.9 tall, facing +z)
const PIVOT: Record<PartName, [number, number, number]> = {
	head: [0, 3.95, 0],
	torso: [0, 0, 0],
	armL: [-0.86, 3.12, -0.1],
	armR: [0.86, 3.12, -0.1],
	legL: [-0.4, 1.55, 0],
	legR: [0.4, 1.55, 0]
};
let FIG: { geo: Record<PartName, THREE.BufferGeometry> } | null = null;
let figPromise: Promise<void> | null = null;

/** Loads the minifigure parts once. Call before creating any Minifig. */
export function preloadFigure() {
	figPromise ??= (async () => {
		const geos = await Promise.all(PARTS.map((n) => loadModel(`/models/fig-${n}.bin`)));
		const geo = {} as Record<PartName, THREE.BufferGeometry>;
		PARTS.forEach((n, i) => {
			const g = geos[i].clone();
			const [x, y, z] = PIVOT[n];
			g.translate(-x, -y, -z);
			geo[n] = g;
		});
		FIG = { geo };
	})();
	return figPromise;
}

function faceCanvas(style: FigStyle) {
	const c = document.createElement('canvas');
	c.width = 256;
	c.height = 256;
	const g = c.getContext('2d')!;
	g.fillStyle = style.skin ?? SKIN;
	g.fillRect(0, 0, 256, 256);
	const ink = '#1b1b1b';
	// eyes
	g.fillStyle = ink;
	for (const x of [98, 158]) {
		g.beginPath();
		g.ellipse(x, 120, 9, 13, 0, 0, Math.PI * 2);
		g.fill();
	}
	g.fillStyle = '#ffffff';
	for (const x of [101, 161]) {
		g.beginPath();
		g.arc(x, 114, 3.5, 0, Math.PI * 2);
		g.fill();
	}
	// brows
	g.strokeStyle = ink;
	g.lineCap = 'round';
	g.lineWidth = 6;
	g.beginPath();
	g.moveTo(82, 90);
	g.lineTo(112, 86);
	g.moveTo(144, 86);
	g.lineTo(174, 90);
	g.stroke();
	if (style.female) {
		// lashes + lips
		g.lineWidth = 4;
		g.beginPath();
		g.moveTo(86, 112);
		g.lineTo(80, 106);
		g.moveTo(170, 112);
		g.lineTo(176, 106);
		g.stroke();
		g.fillStyle = '#c0392b';
		g.beginPath();
		g.ellipse(128, 176, 20, 9, 0, 0, Math.PI);
		g.fill();
		g.strokeStyle = ink;
		g.lineWidth = 4;
		g.beginPath();
		g.moveTo(106, 176);
		g.lineTo(150, 176);
		g.stroke();
	} else {
		g.lineWidth = 7;
		g.beginPath();
		g.arc(128, 150, 32, 0.18 * Math.PI, 0.82 * Math.PI);
		g.stroke();
	}
	return c;
}

function canvasTex(c: HTMLCanvasElement) {
	const t = new THREE.CanvasTexture(c);
	t.colorSpace = THREE.SRGBColorSpace;
	t.anisotropy = 4;
	return t;
}

/** Returns a LEGO-like minifigure (approx. 4.9 units tall, feet at y=0). */
export class Minifig {
	group = new THREE.Group();
	legL = new THREE.Group();
	legR = new THREE.Group();
	armL = new THREE.Group();
	armR = new THREE.Group();
	head = new THREE.Group();
	walkT = 0;

	constructor(style: FigStyle) {
		if (!FIG) throw new Error('preloadFigure() must finish before creating a Minifig');
		const hairM = m(style.hair);
		// double-sided: the source mesh has a few open spots (legs/hips) you could otherwise look through
		const plastic = (color: string) => new THREE.MeshStandardMaterial({ color, roughness: 0.28, metalness: 0, side: THREE.DoubleSide });
		const torsoMat = plastic(style.coat ? '#f4f4f4' : style.torso);
		const mats = [
			torsoMat,
			plastic(style.legs),
			plastic(style.skin ?? SKIN),
			new THREE.MeshStandardMaterial({ map: canvasTex(torsoCanvas(style)), roughness: 0.28, side: THREE.DoubleSide }),
			new THREE.MeshStandardMaterial({ map: canvasTex(faceCanvas(style)), roughness: 0.28, side: THREE.DoubleSide })
		];
		const groups: Record<PartName, THREE.Object3D> = {
			head: this.head,
			torso: this.group,
			armL: this.armL,
			armR: this.armR,
			legL: this.legL,
			legR: this.legR
		};
		for (const n of PARTS) {
			const mesh = new THREE.Mesh(FIG.geo[n], mats);
			mesh.name = n;
			const holder = groups[n];
			if (n === 'torso') {
				mesh.position.set(0, 0, 0);
				this.group.add(mesh);
			} else {
				holder.position.set(...PIVOT[n]);
				holder.add(mesh);
				this.group.add(holder);
			}
		}

		// hair / hats
		const hair = new THREE.Group();
		switch (style.hairStyle) {
			case 'cap': {
				const capM = m(style.cap ?? '#d22');
				const dome = new THREE.Mesh(new THREE.SphereGeometry(0.72, 24, 12, 0, Math.PI * 2, 0, Math.PI / 2), capM);
				dome.position.y = 0.32;
				dome.scale.y = 0.8;
				// brim: a crescent that is widest at the front and runs out to nothing at the sides,
				// so it blends into the cap instead of ending in a hard edge
				const shape = new THREE.Shape();
				const R = 0.7; // where the brim meets the cap
				const N = 32;
				for (let i = 0; i <= N; i++) {
					const a = -Math.PI / 2 + (i / N) * Math.PI; // -90° (side) .. 90° (other side), 0 = front
					const r = R + 0.5 * Math.pow(Math.cos(a), 1.5);
					const p = [Math.sin(a) * r, Math.cos(a) * r];
					if (i === 0) shape.moveTo(p[0], p[1]);
					else shape.lineTo(p[0], p[1]);
				}
				for (let i = N; i >= 0; i--) {
					const a = -Math.PI / 2 + (i / N) * Math.PI;
					shape.lineTo(Math.sin(a) * (R - 0.08), Math.cos(a) * (R - 0.08));
				}
				const brimGeo = new THREE.ExtrudeGeometry(shape, { depth: 0.08, bevelEnabled: true, bevelThickness: 0.02, bevelSize: 0.02, bevelSegments: 2, curveSegments: 4 });
				brimGeo.rotateX(Math.PI / 2); // lay it flat; shape +y becomes +z (front)
				const brim = new THREE.Mesh(brimGeo, new THREE.MeshStandardMaterial({ color: style.cap ?? '#d22', roughness: 0.32, side: THREE.DoubleSide }));
				brim.position.set(0, 0.38, 0);
				const back = new THREE.Mesh(new THREE.CylinderGeometry(0.7, 0.66, 0.5, 24, 1, true, Math.PI * 0.6, Math.PI * 0.8), hairM);
				back.position.y = 0.1;
				hair.add(dome, brim, back);
				break;
			}
			case 'spiky': {
				const base = new THREE.Mesh(new THREE.SphereGeometry(0.74, 20, 10, 0, Math.PI * 2, 0, Math.PI / 2), hairM);
				base.position.y = 0.3;
				hair.add(base);
				for (let i = 0; i < 9; i++) {
					const sp = new THREE.Mesh(new THREE.ConeGeometry(0.22, 0.7, 6), hairM);
					const a = (i / 9) * Math.PI * 2;
					sp.position.set(Math.sin(a) * 0.5, 0.75, Math.cos(a) * 0.45 - 0.1);
					sp.rotation.set(Math.cos(a) * 0.9, 0, -Math.sin(a) * 0.9);
					hair.add(sp);
				}
				const back = new THREE.Mesh(new THREE.CylinderGeometry(0.72, 0.68, 0.7, 24, 1, true, Math.PI * 0.55, Math.PI * 0.9), hairM);
				hair.add(back);
				break;
			}
			case 'ponytail':
			case 'bob': {
				const base = new THREE.Mesh(new THREE.SphereGeometry(0.75, 24, 12, 0, Math.PI * 2, 0, Math.PI / 2), hairM);
				base.position.y = 0.3;
				const side = new THREE.Mesh(new THREE.CylinderGeometry(0.76, 0.78, style.hairStyle === 'bob' ? 1.0 : 0.6, 24, 1, true, Math.PI * 0.35, Math.PI * 1.3), hairM);
				side.position.y = style.hairStyle === 'bob' ? -0.15 : 0.05;
				hair.add(base, side);
				if (style.hairStyle === 'ponytail') {
					const tail = new THREE.Mesh(new THREE.CapsuleGeometry(0.25, 0.9, 6, 12), hairM);
					tail.position.set(0, -0.2, -0.85);
					tail.rotation.x = 0.35;
					hair.add(tail);
				}
				break;
			}
			case 'gray': {
				const side = new THREE.Mesh(new THREE.CylinderGeometry(0.7, 0.7, 0.6, 24, 1, true, Math.PI * 0.45, Math.PI * 1.1), hairM);
				side.position.y = 0.15;
				const tuft = new THREE.Mesh(new THREE.SphereGeometry(0.66, 20, 10, 0, Math.PI * 2, 0, Math.PI / 2), hairM);
				tuft.position.set(0, 0.42, -0.08);
				tuft.scale.set(1, 0.5, 0.9);
				hair.add(side, tuft);
				break;
			}
			case 'short': {
				const base = new THREE.Mesh(new THREE.SphereGeometry(0.73, 24, 12, 0, Math.PI * 2, 0, Math.PI / 2), hairM);
				base.position.y = 0.32;
				const fringe = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.25, 0.3), hairM);
				fringe.position.set(0.1, 0.5, 0.55);
				fringe.rotation.z = -0.2;
				const back = new THREE.Mesh(new THREE.CylinderGeometry(0.71, 0.7, 0.55, 24, 1, true, Math.PI * 0.55, Math.PI * 0.9), hairM);
				back.position.y = 0.1;
				hair.add(base, fringe, back);
				break;
			}
		}
		hair.scale.set(1.04, 1.2, 1.04);
		hair.position.y = 0.32;
		this.head.add(hair);

		this.group.traverse((o) => {
			if ((o as THREE.Mesh).isMesh) {
				o.castShadow = true;
				o.receiveShadow = true;
			}
		});
	}

	animate(dt: number, moving: boolean, speed = 1) {
		if (moving) {
			this.walkT += dt * 9 * speed;
			const s = Math.sin(this.walkT) * 0.7;
			this.legL.rotation.x = s;
			this.legR.rotation.x = -s;
			this.armL.rotation.x = ARM_REST - s * 0.8;
			this.armR.rotation.x = ARM_REST + s * 0.8;
		} else {
			const k = Math.min(1, dt * 10);
			for (const p of [this.legL, this.legR]) p.rotation.x *= 1 - k;
			for (const p of [this.armL, this.armR]) p.rotation.x += (ARM_REST - p.rotation.x) * k;
		}
	}

	wave(t: number) {
		this.armR.rotation.x = -2.6 + Math.sin(t * 8) * 0.3;
	}
}
