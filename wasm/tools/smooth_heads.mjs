#!/usr/bin/env node
// Smooth-shaded minifig heads: the minifig parts come flat shaded (a normal per facet), which shows every facet of the
// round head. This gives each corner the normal of the facets around it within `--smooth` degrees (auto smooth, as
// Blender's), welds equal corners and writes the glb again (the same triangles; images kept).
//
//   node tools/smooth_heads.mjs                  assets/minifig/*/head_flat.glb -> head.glb (head_flat.glb is the
//                                                source: change that one and rerun; it is made from head.glb once)
//   node tools/smooth_heads.mjs <in.glb> <out.glb> [--smooth 46]
//
// Zero npm dependencies.

import fs from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2).filter((a, i, all) => a !== '--smooth' && all[i - 1] !== '--smooth');
const opt = (n, d) => { const all = process.argv.slice(2); const i = all.indexOf(n); return i >= 0 ? all[i + 1] : d; };

function readGlb(file) {
  const b = fs.readFileSync(file);
  if (b.readUInt32LE(0) !== 0x46546c67) throw new Error(`${file}: not a glb`);
  let o = 12, json = null, bin = null;
  while (o < b.length) {
    const len = b.readUInt32LE(o), type = b.readUInt32LE(o + 4);
    const data = b.subarray(o + 8, o + 8 + len);
    if (type === 0x4e4f534a) json = JSON.parse(data.toString('utf8'));
    else if (type === 0x004e4942) bin = data;
    o += 8 + len;
  }
  return { json, bin };
}

function accessor(g, i) {
  const a = g.json.accessors[i];
  const bv = g.json.bufferViews[a.bufferView];
  const comps = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 }[a.type];
  const T = { 5126: Float32Array, 5125: Uint32Array, 5123: Uint16Array, 5121: Uint8Array }[a.componentType];
  const size = T.BYTES_PER_ELEMENT;
  const stride = bv.byteStride || comps * size;
  const base = (bv.byteOffset || 0) + (a.byteOffset || 0);
  const out = [];
  for (let k = 0; k < a.count; k++) {
    const v = [];
    for (let c = 0; c < comps; c++) {
      const off = g.bin.byteOffset + base + k * stride + c * size;
      const dv = new DataView(g.bin.buffer, off, size);
      v.push(size === 4 ? (T === Float32Array ? dv.getFloat32(0, true) : dv.getUint32(0, true)) : size === 2 ? dv.getUint16(0, true) : dv.getUint8(0));
    }
    out.push(comps === 1 ? v[0] : v);
  }
  return out;
}

const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const add = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const mul = (a, s) => [a[0] * s, a[1] * s, a[2] * s];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const len = (a) => Math.hypot(a[0], a[1], a[2]);
const norm = (a) => { const l = len(a) || 1; return mul(a, 1 / l); };
const key = (p) => p.map((x) => Math.round(x * 1e5)).join(',');

/** Auto smooth (as Blender's): the minifig parts come flat shaded (a normal per facet, some zero), which shows every
 *  facet of a round part. Each corner gets the area-weighted normal of the triangles around its position whose facet
 *  normal is within `SMOOTH_DEG` of its own: the facets of a round part (~11 degrees apart) blend, a rim (90) stays. */
const SMOOTH_DEG = Number(opt('--smooth', 46));
function smoothCorners(P, UV, I) {
  const fn = [];
  for (let t = 0; t < I.length; t += 3) fn.push(cross(sub(P[I[t + 1]], P[I[t]]), sub(P[I[t + 2]], P[I[t]])));
  const around = new Map();
  for (let t = 0; t < I.length; t += 3) for (let e = 0; e < 3; e++) {
    const k = key(P[I[t + e]]);
    if (!around.has(k)) around.set(k, []);
    around.get(k).push(t / 3);
  }
  const cos = Math.cos((SMOOTH_DEG * Math.PI) / 180);
  const P2 = [], N2 = [], UV2 = UV ? [] : null, I2 = [];
  for (let t = 0; t < I.length; t += 3) {
    const own = norm(fn[t / 3]);
    for (let e = 0; e < 3; e++) {
      const v = I[t + e];
      let n = [0, 0, 0];
      for (const o of around.get(key(P[v]))) if (dot(norm(fn[o]), own) > cos) n = add(n, fn[o]);
      I2.push(P2.length);
      P2.push(P[v]);
      N2.push(len(n) > 1e-12 ? norm(n) : own);
      if (UV) UV2.push(UV[v]);
    }
  }
  return { P: P2, N: N2, UV: UV2, I: I2 };
}

/** Welds equal corners (position, normal, uv) into shared vertices. */
function weld({ P, N, UV, I }) {
  const outP = [], outN = [], outUV = [], outI = [];
  const ids = new Map();
  for (const v of I) {
    const p = P[v], n = norm(N[v]), uv = UV ? UV[v] : [0, 0];
    const k = `${key(p)}/${n.map((x) => Math.round(x * 1e3)).join(',')}/${uv.map((x) => Math.round(x * 1e5)).join(',')}`;
    let id = ids.get(k);
    if (id === undefined) {
      id = outP.length;
      ids.set(k, id);
      outP.push(p);
      outN.push(n);
      outUV.push(uv);
    }
    outI.push(id);
  }
  return { P: outP, N: outN, UV: outUV, I: outI };
}

function smoothGlb(inFile, outFile) {
  const g = readGlb(inFile);
  const j = g.json;
  const chunks = [];
  let offset = 0;
  const views = [];
  const pushView = (buf, target) => {
    const pad = (4 - (offset % 4)) % 4;
    if (pad) { chunks.push(Buffer.alloc(pad)); offset += pad; }
    views.push({ buffer: 0, byteOffset: offset, byteLength: buf.length, ...(target ? { target } : {}) });
    chunks.push(buf);
    offset += buf.length;
    return views.length - 1;
  };
  // images keep their bytes (new views)
  const imageViews = (j.images || []).map((im) => {
    if (im.bufferView === undefined) return undefined;
    const bv = j.bufferViews[im.bufferView];
    return pushView(Buffer.from(g.bin.subarray(bv.byteOffset || 0, (bv.byteOffset || 0) + bv.byteLength)));
  });
  const accessors = [];
  let before = 0, after = 0;
  for (const mesh of j.meshes) {
    for (const prim of mesh.primitives) {
      const P = accessor(g, prim.attributes.POSITION), N = accessor(g, prim.attributes.NORMAL);
      const UV = prim.attributes.TEXCOORD_0 !== undefined ? accessor(g, prim.attributes.TEXCOORD_0) : null;
      const I = prim.indices !== undefined ? accessor(g, prim.indices) : P.map((_, k) => k);
      const r = weld(smoothCorners(P, UV, I));
      before += I.length / 3;
      after += r.I.length / 3;
      const f32 = (arr) => Buffer.from(new Float32Array(arr.flat()).buffer);
      const min = [0, 1, 2].map((c) => Math.min(...r.P.map((p) => p[c])));
      const max = [0, 1, 2].map((c) => Math.max(...r.P.map((p) => p[c])));
      const attrs = {};
      accessors.push({ bufferView: pushView(f32(r.P), 34962), componentType: 5126, count: r.P.length, type: 'VEC3', min, max });
      attrs.POSITION = accessors.length - 1;
      accessors.push({ bufferView: pushView(f32(r.N), 34962), componentType: 5126, count: r.N.length, type: 'VEC3' });
      attrs.NORMAL = accessors.length - 1;
      if (UV) {
        accessors.push({ bufferView: pushView(f32(r.UV), 34962), componentType: 5126, count: r.UV.length, type: 'VEC2' });
        attrs.TEXCOORD_0 = accessors.length - 1;
      }
      const small = r.P.length < 65536;
      const ib = small ? Buffer.from(new Uint16Array(r.I).buffer) : Buffer.from(new Uint32Array(r.I).buffer);
      accessors.push({ bufferView: pushView(ib, 34963), componentType: small ? 5123 : 5125, count: r.I.length, type: 'SCALAR' });
      prim.attributes = attrs;
      prim.indices = accessors.length - 1;
    }
  }
  (j.images || []).forEach((im, k) => { if (imageViews[k] !== undefined) im.bufferView = imageViews[k]; });
  j.accessors = accessors;
  j.bufferViews = views;
  const bin = Buffer.concat(chunks);
  const binPad = Buffer.concat([bin, Buffer.alloc((4 - (bin.length % 4)) % 4)]);
  j.buffers = [{ byteLength: binPad.length }];
  let js = Buffer.from(JSON.stringify(j), 'utf8');
  js = Buffer.concat([js, Buffer.alloc((4 - (js.length % 4)) % 4, 0x20)]);
  const head = Buffer.alloc(12);
  head.writeUInt32LE(0x46546c67, 0); head.writeUInt32LE(2, 4); head.writeUInt32LE(12 + 8 + js.length + 8 + binPad.length, 8);
  const ch = (type, data) => { const h = Buffer.alloc(8); h.writeUInt32LE(data.length, 0); h.writeUInt32LE(type, 4); return Buffer.concat([h, data]); };
  fs.writeFileSync(outFile, Buffer.concat([head, ch(0x4e4f534a, js), ch(0x004e4942, binPad)]));
  console.log(`${path.relative(process.cwd(), outFile)}: ${before} -> ${after} triangles, ${fs.statSync(outFile).size} bytes`);
}

if (args.length === 0) {
  const root = path.join(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1')), '..', 'assets', 'minifig');
  for (const style of fs.readdirSync(root)) {
    const flat = path.join(root, style, 'head_flat.glb'), f = path.join(root, style, 'head.glb');
    if (!fs.existsSync(f)) continue;
    if (!fs.existsSync(flat)) fs.copyFileSync(f, flat);
    smoothGlb(flat, f);
  }
} else if (args.length === 2) {
  smoothGlb(args[0], args[1]);
} else {
  console.error('usage: node tools/smooth_heads.mjs [<in.glb> <out.glb>] [--smooth 46]');
  process.exit(1);
}
