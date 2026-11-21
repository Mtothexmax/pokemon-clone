#!/usr/bin/env node
// Finer minifig heads for close-ups: every triangle becomes a curved PN-triangle patch (Vlachos et al. 2001),
// tessellated LEVEL x LEVEL, so round parts (the head cylinder, helmets, hair) get round silhouettes while flat
// parts stay flat and hard edges stay sharp. Crack-free: an edge's curve depends only on that edge's data. At a hard
// edge (split normals: the cap and the side of the head) the curve's tangent at each end is perpendicular to both
// normals (cross product), so the rim of a cylinder becomes a circle on both sides of the crease.
// UVs are linear, normals are the vertex normals interpolated (Phong shading), so prints keep their place.
//
//   node tools/subdivide.mjs <in.glb> <out.glb> [--level 4]
//   node tools/subdivide.mjs --heads            (assets/minifig/*/head.glb -> head_hi.glb, and head.glb smooth shaded:
//                                                a copy of the flat-shaded original is kept as head_flat.glb)
//
// The game uses head_hi.glb as LOD 0 of a figure's head when the camera comes close (figure.rs). Zero npm dependencies.

import fs from 'node:fs';
import path from 'node:path';

const args = process.argv.slice(2);
const opt = (n, d) => { const i = args.indexOf(n); return i >= 0 ? args[i + 1] : d; };
let LEVEL = Number(opt('--level', 4));

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

/** One primitive: positions, normals, uvs, indices -> the tessellated arrays. */
function tessellate(P0, N0, UV0, I0) {
  const { P, N, UV, I } = smoothCorners(P0, UV0, I0);
  // which normals meet along each edge (by welded positions): a hard edge has two different ones at an end
  const edgeNormals = new Map(); // "ka|kb" (sorted) -> { [ka]: [normals], [kb]: [normals] }
  const ek = (a, b) => (a < b ? `${a}|${b}` : `${b}|${a}`);
  for (let t = 0; t < I.length; t += 3) {
    for (let e = 0; e < 3; e++) {
      const i = I[t + e], j = I[t + (e + 1) % 3];
      const ki = key(P[i]), kj = key(P[j]);
      const k = ek(ki, kj);
      let m = edgeNormals.get(k);
      if (!m) edgeNormals.set(k, (m = { [ki]: [], [kj]: [] }));
      m[ki].push(N[i]);
      m[kj].push(N[j]);
    }
  }
  // the control point one third along edge i -> j (shared by both triangles of the edge)
  const ctrl = (i, j) => {
    const e = sub(P[j], P[i]);
    const ns = edgeNormals.get(ek(key(P[i]), key(P[j])))[key(P[i])];
    const n0 = norm(ns[0]);
    const other = ns.find((n) => dot(norm(n), n0) < 0.999);
    if (other) {
      // crease: along the line both surfaces share
      let t = norm(cross(n0, norm(other)));
      const d = dot(t, e);
      if (d < 0) t = mul(t, -1);
      if (len(cross(n0, norm(other))) < 1e-4) return add(P[i], mul(e, 1 / 3));
      return add(P[i], mul(t, Math.abs(d) / 3));
    }
    // smooth (also a boundary): the edge projected onto the tangent plane (PN triangles)
    const w = dot(e, n0);
    return mul(add(mul(P[i], 2), sub(P[j], mul(n0, w))), 1 / 3);
  };

  const outP = [], outN = [], outUV = [], outI = [];
  const weld = new Map();
  const vert = (p, n, uv) => {
    const k = `${key(p)}/${n.map((x) => Math.round(x * 1e3)).join(',')}/${uv.map((x) => Math.round(x * 1e5)).join(',')}`;
    let id = weld.get(k);
    if (id === undefined) {
      id = outP.length;
      weld.set(k, id);
      outP.push(p);
      outN.push(n);
      outUV.push(uv);
    }
    return id;
  };
  const L = LEVEL;
  for (let t = 0; t < I.length; t += 3) {
    const [a, b, c] = [I[t], I[t + 1], I[t + 2]];
    const b300 = P[a], b030 = P[b], b003 = P[c];
    const b210 = ctrl(a, b), b120 = ctrl(b, a), b021 = ctrl(b, c), b012 = ctrl(c, b), b102 = ctrl(c, a), b201 = ctrl(a, c);
    const E = mul([b210, b120, b021, b012, b102, b201].reduce(add), 1 / 6);
    const V = mul(add(add(b300, b030), b003), 1 / 3);
    const b111 = add(E, mul(sub(E, V), 0.5));
    const grid = [];
    for (let i = 0; i <= L; i++) {
      grid.push([]);
      for (let j = 0; j <= L - i; j++) {
        const w = i / L, u = j / L, v = 1 - w - u; // w: towards c, u: towards b, v: towards a
        // a point on an edge of the triangle is on that edge's curve: the same for both triangles
        const p = [b300, b030, b003, b210, b120, b021, b012, b102, b201, b111];
        const k = [v ** 3, u ** 3, w ** 3, 3 * v * v * u, 3 * v * u * u, 3 * u * u * w, 3 * u * w * w, 3 * w * w * v, 3 * w * v * v, 6 * u * v * w];
        let pos = [0, 0, 0];
        for (let m = 0; m < 10; m++) pos = add(pos, mul(p[m], k[m]));
        const n = norm(add(add(mul(N[a], v), mul(N[b], u)), mul(N[c], w)));
        const uv = UV ? [UV[a][0] * v + UV[b][0] * u + UV[c][0] * w, UV[a][1] * v + UV[b][1] * u + UV[c][1] * w] : [0, 0];
        grid[i].push(vert(pos, n, uv));
      }
    }
    for (let i = 0; i < L; i++) {
      for (let j = 0; j < L - i; j++) {
        outI.push(grid[i][j], grid[i][j + 1], grid[i + 1][j]);
        if (j < L - i - 1) outI.push(grid[i][j + 1], grid[i + 1][j + 1], grid[i + 1][j]);
      }
    }
  }
  return { P: outP, N: outN, UV: outUV, I: outI };
}

function subdivideGlb(inFile, outFile) {
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
      const r = tessellate(P, N, UV, I);
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

if (args[0] === '--heads') {
  const root = path.join(path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1')), '..', 'assets', 'minifig');
  for (const style of fs.readdirSync(root)) {
    const flat = path.join(root, style, 'head_flat.glb'), f = path.join(root, style, 'head.glb');
    if (!fs.existsSync(f)) continue;
    if (!fs.existsSync(flat)) fs.copyFileSync(f, flat);
    subdivideGlb(flat, path.join(root, style, 'head_hi.glb'));
    // the far head: the same triangles, smooth shaded (level 1: corners only)
    const keep = LEVEL;
    LEVEL = 1;
    subdivideGlb(flat, f);
    LEVEL = keep;
  }
} else if (args.length >= 2) {
  subdivideGlb(args[0], args[1]);
} else {
  console.error('usage: node tools/subdivide.mjs <in.glb> <out.glb> [--level 4] | --heads');
  process.exit(1);
}
