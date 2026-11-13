import { assetUrl } from '../asset-url';
import * as THREE from 'three';
import { ASSET_VERSION } from './assets-version';

const cache = new Map<string, Promise<THREE.BufferGeometry>>();

/** Loads a pre-converted binary model (see scripts/convert-models.mjs). */
export function loadModel(url: string): Promise<THREE.BufferGeometry> {
	let p = cache.get(url);
	if (!p) {
		p = fetch(`${assetUrl(url)}?v=${ASSET_VERSION}`, { cache: 'no-cache' }) // revalidate so re-converted models are picked up
			.then((r) => r.arrayBuffer())
			.then((buf) => {
				const [n, m, flags] = new Uint32Array(buf, 0, 3);
				let o = 12;
				const g = new THREE.BufferGeometry();
				g.setAttribute('position', new THREE.BufferAttribute(new Float32Array(buf, o, n * 3), 3));
				o += n * 12;
				g.setAttribute('normal', new THREE.BufferAttribute(new Float32Array(buf, o, n * 3), 3));
				o += n * 12;
				if (flags & 1) {
					// stored as sRGB bytes; three.js expects linear vertex colors
					const src = new Uint8Array(buf, o, n * 4);
					const lin = new Float32Array(n * 3);
					const c = new THREE.Color();
					for (let i = 0; i < n; i++) {
						c.setRGB(src[i * 4] / 255, src[i * 4 + 1] / 255, src[i * 4 + 2] / 255, THREE.SRGBColorSpace);
						lin[i * 3] = c.r;
						lin[i * 3 + 1] = c.g;
						lin[i * 3 + 2] = c.b;
					}
					g.setAttribute('color', new THREE.BufferAttribute(lin, 3));
					o += n * 4;
				}
				if (flags & 2) {
					g.setAttribute('uv', new THREE.BufferAttribute(new Float32Array(buf, o, n * 2), 2));
					o += n * 8;
				}
				g.setIndex(new THREE.BufferAttribute(new Uint32Array(buf, o, m), 1));
				o += m * 4;
				if (flags & 4) {
					// material groups: count, then (start, count, materialIndex) triples
					const [gc] = new Uint32Array(buf, o, 1);
					const gr = new Uint32Array(buf, o + 4, gc * 3);
					for (let i = 0; i < gc; i++) g.addGroup(gr[i * 3], gr[i * 3 + 1], gr[i * 3 + 2]);
				}
				g.computeBoundingSphere();
				return g;
			});
		cache.set(url, p);
	}
	return p;
}
