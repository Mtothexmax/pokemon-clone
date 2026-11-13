// A Voxelmon in the game: the 3D model is drawn from its editable model file
// (src/lib/voxelmon/models/<id>.js: palette + pixel rows) with the drawing method in src/lib/voxelmon/draw.js.
// The model files were made from the card sprites by scripts/export-voxelmon.mjs and can be edited by hand.
import * as THREE from 'three';
import { drawVoxelmon } from '../voxelmon/draw.js';

import { MODELS } from '../voxelmon/index';
/** does this beast have a Voxelmon model file? */
export const hasVoxelModel = (id: string) => !!MODELS[id];

const cache = new Map<string, THREE.BufferGeometry | null>();
function buildGeometry(id: string): THREE.BufferGeometry | null {
	const model = MODELS[id];
	return model ? drawVoxelmon(model) : null;
}

/**
 * A Voxelmon model: a group whose origin is between its feet, facing +z, `height` world units tall.
 * Returns null if the card has no sprite (or the sprite sheets are not loaded yet).
 */
export function buildVoxelmon(id: string, height: number, opts: { shiny?: boolean } = {}) {
	if (!cache.has(id)) cache.set(id, buildGeometry(id));
	const geo = cache.get(id);
	if (!geo) return null;
	const { h } = geo.userData.size as { w: number; h: number };
	const mat = new THREE.MeshStandardMaterial({
		vertexColors: true,
		roughness: 0.5,
		metalness: opts.shiny ? 0.35 : 0,
		color: opts.shiny ? '#ffe08a' : '#ffffff',
		emissive: opts.shiny ? '#6a4a00' : '#000000'
	});
	const mesh = new THREE.Mesh(geo, mat);
	mesh.castShadow = true;
	mesh.receiveShadow = true;
	const s = height / h;
	mesh.scale.setScalar(s);
	const g = new THREE.Group();
	g.name = `voxelmon:${id}`;
	// the body bobs / squashes inside the group; the group itself is placed and turned
	const body = new THREE.Group();
	body.add(mesh);
	g.add(body);
	g.userData.body = body;
	g.userData.mat = mat;
	g.userData.voxel = s;
	return g;
}

// ---------- card portrait: the Voxelmon rendered in 3D, straight from the front ----------
let portraitRenderer: THREE.WebGLRenderer | null = null;
const portraitCache = new Map<string, HTMLCanvasElement | null>();
/**
 * The beast rendered in 3D from the front (camera a little above, light from the front-left), cropped to
 * the beast, on a transparent background. For the picture of cards that are real Voxelmon.
 */
export function voxelPortrait(id: string): HTMLCanvasElement | null {
	if (portraitCache.has(id)) return portraitCache.get(id)!;
	const mon = buildVoxelmon(id, 10);
	if (!mon) {
		portraitCache.set(id, null);
		return null;
	}
	const S = 512;
	const r = (portraitRenderer ??= new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true }));
	r.setSize(S, S, false);
	r.outputColorSpace = THREE.SRGBColorSpace;
	r.setClearColor(0x000000, 0);
	const scene = new THREE.Scene();
	scene.add(new THREE.HemisphereLight('#ffffff', '#6a7a8a', 1.5));
	const sun = new THREE.DirectionalLight('#ffffff', 2.3);
	sun.position.set(-4, 7, 9);
	scene.add(sun);
	scene.add(mon);
	const box = new THREE.Box3().setFromObject(mon);
	const size = box.getSize(new THREE.Vector3()), center = box.getCenter(new THREE.Vector3());
	// straight from the front, a little from above so the tops of the voxels show
	const cam = new THREE.PerspectiveCamera(30, 1, 0.1, 500);
	const up = (12 * Math.PI) / 180;
	const dist = (Math.max(size.x, size.y) / 2 / Math.tan((15 * Math.PI) / 180)) * 1.25 + size.z / 2;
	cam.position.set(center.x, center.y + Math.sin(up) * dist, center.z + Math.cos(up) * dist);
	cam.lookAt(center);
	r.render(scene, cam);
	(mon.userData.mat as THREE.Material).dispose();
	// crop to the beast
	const full = document.createElement('canvas');
	full.width = full.height = S;
	const g = full.getContext('2d', { willReadFrequently: true })!;
	g.drawImage(r.domElement, 0, 0);
	const px = g.getImageData(0, 0, S, S).data;
	let x0 = S, y0 = S, x1 = -1, y1 = -1;
	for (let y = 0; y < S; y++)
		for (let x = 0; x < S; x++)
			if (px[(y * S + x) * 4 + 3] > 8) [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
	if (x1 < 0) {
		portraitCache.set(id, null);
		return null;
	}
	const out = document.createElement('canvas');
	out.width = x1 - x0 + 1;
	out.height = y1 - y0 + 1;
	out.getContext('2d')!.drawImage(full, x0, y0, out.width, out.height, 0, 0, out.width, out.height);
	portraitCache.set(id, out);
	return out;
}
