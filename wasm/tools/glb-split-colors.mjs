#!/usr/bin/env node
// Turns vertex-coloured .glb models (COLOR_0, e.g. the LEGO props) into models with one primitive
// per colour and a plain material each. The wasmOnly glTF loader reads materials, not vertex
// colours, and LEGO parts are flat-coloured anyway, so nothing is lost and every colour batches.
//
//   node tools/glb-split-colors.mjs <in.glb> <out.glb> [--quant 24]
//   node tools/glb-split-colors.mjs --batch <in-dir> <out-dir>
//
// Triangles are grouped by the (quantised) colour of their first vertex. Models without COLOR_0
// are copied unchanged. Zero npm dependencies.

import fs from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(n); return i >= 0 ? Number(args[i + 1]) : d; };
const QUANT = opt('--quant', 24); // colour bucket size in 0..255 steps

const COMP = { 5120: Int8Array, 5121: Uint8Array, 5122: Int16Array, 5123: Uint16Array, 5125: Uint32Array, 5126: Float32Array };
const NCOMP = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 };

function readGlb(buf) {
  if (buf.readUInt32LE(0) !== 0x46546c67) throw new Error('not a glb');
  let off = 12, json = null, bin = null;
  while (off < buf.length) {
    const len = buf.readUInt32LE(off), type = buf.readUInt32LE(off + 4);
    const chunk = buf.subarray(off + 8, off + 8 + len);
    if (type === 0x4e4f534a) json = JSON.parse(chunk.toString('utf8'));
    else if (type === 0x004e4942) bin = chunk;
    off += 8 + len;
  }
  return { json, bin };
}

function accessor(g, i) {
  const a = g.json.accessors[i];
  const bv = g.json.bufferViews[a.bufferView];
  const T = COMP[a.componentType];
  const n = NCOMP[a.type];
  const stride = bv.byteStride ? bv.byteStride / T.BYTES_PER_ELEMENT : n;
  const base = g.bin.byteOffset + (bv.byteOffset ?? 0) + (a.byteOffset ?? 0);
  const src = new T(g.bin.buffer, base, (a.count - 1) * stride + n);
  const out = new Float64Array(a.count * n);
  for (let k = 0; k < a.count; k++)
    for (let c = 0; c < n; c++) {
      let v = src[k * stride + c];
      if (a.normalized) v /= T === Uint8Array ? 255 : T === Uint16Array ? 65535 : 1;
      out[k * n + c] = v;
    }
  return { data: out, n, count: a.count };
}

const toLinear = (x) => (x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4);
const toSrgb = (x) => (x <= 0.0031308 ? x * 12.92 : 1.055 * x ** (1 / 2.4) - 0.055);

function convert(inPath, outPath) {
  const g = readGlb(fs.readFileSync(inPath));
  const hasColor = g.json.meshes.some((m) => m.primitives.some((p) => p.attributes.COLOR_0 !== undefined));
  if (!hasColor) {
    fs.copyFileSync(inPath, outPath);
    return { file: path.basename(inPath), copied: true };
  }
  // output buffers
  const chunks = [];
  let binLen = 0;
  const views = [], accs = [], mats = [], matKey = new Map();
  const push = (arr, target, type, minmax) => {
    const bytes = Buffer.from(arr.buffer, arr.byteOffset, arr.byteLength);
    const pad = (4 - (binLen % 4)) % 4;
    if (pad) { chunks.push(Buffer.alloc(pad)); binLen += pad; }
    views.push({ buffer: 0, byteOffset: binLen, byteLength: bytes.length, ...(target ? { target } : {}) });
    chunks.push(bytes);
    binLen += bytes.length;
    const acc = { bufferView: views.length - 1, componentType: arr instanceof Uint32Array ? 5125 : 5126, count: arr.length / NCOMP[type], type };
    if (minmax) Object.assign(acc, minmax);
    accs.push(acc);
    return accs.length - 1;
  };
  const material = (rgb, src) => {
    const key = rgb.join(',') + '|' + (src?.alphaMode ?? '') + (src?.doubleSided ? 'd' : '');
    if (!matKey.has(key)) {
      const f = src?.pbrMetallicRoughness?.baseColorFactor ?? [1, 1, 1, 1];
      mats.push({
        name: `c${rgb.map((v) => v.toString(16).padStart(2, '0')).join('')}`,
        pbrMetallicRoughness: {
          baseColorFactor: [toLinear(rgb[0] / 255) * f[0], toLinear(rgb[1] / 255) * f[1], toLinear(rgb[2] / 255) * f[2], f[3]],
          metallicFactor: src?.pbrMetallicRoughness?.metallicFactor ?? 0,
          roughnessFactor: src?.pbrMetallicRoughness?.roughnessFactor ?? 0.35,
        },
        ...(src?.doubleSided ? { doubleSided: true } : {}),
        ...(src?.alphaMode && src.alphaMode !== 'OPAQUE' ? { alphaMode: src.alphaMode, alphaCutoff: src.alphaCutoff } : {}),
      });
      matKey.set(key, mats.length - 1);
    }
    return matKey.get(key);
  };
  const meshes = g.json.meshes.map((m) => {
    const prims = [];
    for (const p of m.primitives) {
      const pos = accessor(g, p.attributes.POSITION);
      const nrm = p.attributes.NORMAL !== undefined ? accessor(g, p.attributes.NORMAL) : null;
      const uv = p.attributes.TEXCOORD_0 !== undefined ? accessor(g, p.attributes.TEXCOORD_0) : null;
      const col = p.attributes.COLOR_0 !== undefined ? accessor(g, p.attributes.COLOR_0) : null;
      const idx = p.indices !== undefined ? accessor(g, p.indices).data : Float64Array.from({ length: pos.count }, (_, i) => i);
      const srcMat = p.material !== undefined ? g.json.materials[p.material] : null;
      // group triangles by colour
      const groups = new Map();
      for (let t = 0; t < idx.length; t += 3) {
        let rgb = [255, 255, 255];
        if (col) {
          const v = idx[t] * col.n;
          // COLOR_0 is linear in glTF; bucket in sRGB so the steps match what the eye sees
          rgb = [0, 1, 2].map((c) => Math.min(255, Math.round(Math.round((toSrgb(col.data[v + c]) * 255) / QUANT) * QUANT)));
        }
        const k = rgb.join(',');
        if (!groups.has(k)) groups.set(k, { rgb, tris: [] });
        groups.get(k).tris.push(idx[t], idx[t + 1], idx[t + 2]);
      }
      for (const { rgb, tris } of groups.values()) {
        // compact the vertices used by this group
        const remap = new Map();
        const P = [], N = [], U = [], I = new Uint32Array(tris.length);
        tris.forEach((v, i) => {
          if (!remap.has(v)) {
            remap.set(v, remap.size);
            P.push(pos.data[v * 3], pos.data[v * 3 + 1], pos.data[v * 3 + 2]);
            if (nrm) N.push(nrm.data[v * 3], nrm.data[v * 3 + 1], nrm.data[v * 3 + 2]);
            if (uv) U.push(uv.data[v * 2], uv.data[v * 2 + 1]);
          }
          I[i] = remap.get(v);
        });
        const Pf = Float32Array.from(P);
        const mn = [Infinity, Infinity, Infinity], mx = [-Infinity, -Infinity, -Infinity];
        for (let i = 0; i < Pf.length; i += 3) for (let c = 0; c < 3; c++) { mn[c] = Math.min(mn[c], Pf[i + c]); mx[c] = Math.max(mx[c], Pf[i + c]); }
        const attributes = { POSITION: push(Pf, 34962, 'VEC3', { min: mn, max: mx }) };
        if (nrm) attributes.NORMAL = push(Float32Array.from(N), 34962, 'VEC3');
        if (uv) attributes.TEXCOORD_0 = push(Float32Array.from(U), 34962, 'VEC2');
        prims.push({ attributes, indices: push(I, 34963, 'SCALAR'), material: material(rgb, srcMat) });
      }
    }
    return { name: m.name, primitives: prims };
  });
  const json = {
    asset: { version: '2.0', generator: 'glb-split-colors' },
    scene: 0,
    scenes: g.json.scenes,
    nodes: g.json.nodes,
    meshes,
    materials: mats,
    accessors: accs,
    bufferViews: views,
    buffers: [{ byteLength: binLen }],
  };
  let jb = Buffer.from(JSON.stringify(json), 'utf8');
  jb = Buffer.concat([jb, Buffer.alloc((4 - (jb.length % 4)) % 4, 0x20)]);
  let bb = Buffer.concat(chunks);
  bb = Buffer.concat([bb, Buffer.alloc((4 - (bb.length % 4)) % 4)]);
  const head = Buffer.alloc(12);
  head.writeUInt32LE(0x46546c67, 0); head.writeUInt32LE(2, 4); head.writeUInt32LE(12 + 8 + jb.length + 8 + bb.length, 8);
  const ch = (len, type) => { const b = Buffer.alloc(8); b.writeUInt32LE(len, 0); b.writeUInt32LE(type, 4); return b; };
  fs.writeFileSync(outPath, Buffer.concat([head, ch(jb.length, 0x4e4f534a), jb, ch(bb.length, 0x004e4942), bb]));
  return { file: path.basename(inPath), colors: mats.length, primitives: meshes.reduce((s, m) => s + m.primitives.length, 0), bytes: 12 + 16 + jb.length + bb.length };
}

if (args[0] === '--batch') {
  const [, inDir, outDir] = args;
  fs.mkdirSync(outDir, { recursive: true });
  const out = [];
  for (const f of fs.readdirSync(inDir).filter((f) => f.endsWith('.glb'))) out.push(convert(path.join(inDir, f), path.join(outDir, f)));
  console.log(JSON.stringify(out));
} else if (args.length >= 2) {
  console.log(JSON.stringify(convert(args[0], args[1])));
} else {
  console.error('usage: node tools/glb-split-colors.mjs <in.glb> <out.glb> | --batch <in-dir> <out-dir>');
  process.exit(2);
}
