import { readFileSync, writeFileSync } from 'node:fs';
import * as THREE from 'three';
import { OBJLoader } from 'three/addons/loaders/OBJLoader.js';
import { mergeVertices } from 'three/addons/utils/BufferGeometryUtils.js';
for (const n of [0, 1, 2, 3]) {
	const o = new OBJLoader().parse(readFileSync(`models-src/bricks/bricks_${n}.obj`, 'utf8'));
	let g; o.traverse((m) => m.isMesh && (g = m.geometry));
	g.rotateX(-Math.PI / 2);
	g.computeBoundingBox();
	const b = g.boundingBox;
	g.translate(-(b.min.x + b.max.x) / 2, -b.min.y, -(b.min.z + b.max.z) / 2);
	g.deleteAttribute('uv');
	g = mergeVertices(g, 1e-4);
	const idx = g.index.array, nV = g.attributes.position.count;
	const buf = Buffer.concat([new Uint32Array([nV, idx.length, 0]).buffer, new Float32Array(g.attributes.position.array).buffer, new Float32Array(g.attributes.normal.array).buffer, new Uint32Array(idx).buffer].map((x) => Buffer.from(x)));
	writeFileSync(`static/models/_prev${n}.bin`, buf);
	console.log(n, nV, (buf.length / 1024).toFixed(0) + 'KB');
}
