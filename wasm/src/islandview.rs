//! The island on screen (OPEN_WORLD.md milestone 1): brick terrain in chunks of `island::CHUNK` studs, built while
//! you walk, the near studs, the colliders and the sea.
//!
//! * **Chunks.** A fine version (1 stud per cell, the stud texture on its tops, colliders) near you, a coarse one
//!   (4 studs per cell) further out and while the fine one is not built yet, so there is never a hole. Tops are
//!   greedy rectangles of one height and colour (one part per colour: the baseplate's stud texture tinted); the
//!   risers are runs merged along each line, cut into bricks with a ±3 % brightness each, all in one part whose
//!   colours come from a small palette texture (constant UVs per face: no extra materials per shade). Big drops are
//!   rock (sandstone in the warm south) with strata per brick row and the top colour on the lip; plate risers
//!   take the top's colour. Each chunk closes its borders with skirts down to `island::FLOOR`, set 0.01 studs
//!   inside, so a neighbour of the other detail never leaves a crack and no two faces share a plane.
//! * **Streaming.** Sampling the law runs as a job in rows under a per-frame millisecond budget; meshing, the GPU
//!   upload and the colliders happen when a chunk's samples are complete. The chunk under you is built at once
//!   (a teleport). Chunks inside the view are on `LAYER_NEAR`, the others on `LAYER_FAR` (drawn only while the
//!   camera looks far: a close-up, a fight).
//! * **Studs.** Real stud geometry only within `STUD_R` studs of you (8x8 tiles, one mesh each, palette colours),
//!   the texture beyond.
//! * **Colliders.** Per fine chunk the greedy rectangles of one height as boxes down to the floor; water deeper than
//!   `island::WADE_MIN` is a tall invisible wall, so you wade in the shallows and stop at the drop.
//! * **Sea.** The engine's ocean (follows the camera) at `island::SEA_Y` on `LAYER_SEA`.

use crate::island::{self, Cell, Island, CHUNK, FLOOR, PALETTE, PLATE, SEA_Y, WADE_MIN, X0, Z0};
use crate::town::{m, TownAssets, STUD};
use engine::mesh::{MeshData, Vertex};
use engine::physics::rapier::prelude::{ColliderBuilder, ColliderHandle};
use engine::prelude::*;
use engine::renderer::{Material, MaterialId, MeshId};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Studs within which the tops get real studs (desktop / phone).
const STUD_R: f32 = 12.0;
const STUD_R_PHONE: f32 = 10.0;
const TILE: i32 = 8;
/// The sea's radius around the camera (metres).
const SEA_RADIUS: f32 = 170.0;
/// The palette texture: 16 x 16 entries of 4 x 4 texels.
const PAL_N: u32 = 64;

/// Rectangles around you (studs: west, east, north, south of you).
const FINE_R: [f32; 4] = [96.0, 96.0, 96.0, 64.0];
const COARSE_R: [f32; 4] = [256.0, 256.0, 256.0, 128.0];
/// What the play camera can see (zoomed out, plus the shadows reaching in): these chunks go on `LAYER_NEAR`.
const VIEW_R: [f32; 4] = [64.0, 64.0, 56.0, 36.0];

fn rect_hits(c: (i32, i32), p: Vec2, r: [f32; 4], grow: f32) -> bool {
    let (x0, z0) = (X0 + c.0 as f32 * CHUNK, Z0 + c.1 as f32 * CHUNK);
    x0 < p.x + r[1] + grow && x0 + CHUNK > p.x - r[0] - grow && z0 < p.y + r[3] + grow && z0 + CHUNK > p.y - r[2] - grow
}

/// Distance from p to a chunk's rect (studs).
fn chunk_dist(c: (i32, i32), p: Vec2) -> f32 {
    let (x0, z0) = (X0 + c.0 as f32 * CHUNK, Z0 + c.1 as f32 * CHUNK);
    let dx = (x0 - p.x).max(p.x - x0 - CHUNK).max(0.0);
    let dz = (z0 - p.y).max(p.y - z0 - CHUNK).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

fn entry(colour: u8, jit: usize) -> u16 {
    colour as u16 * island::JITTER.len() as u16 + jit.min(island::JITTER.len() - 1) as u16
}

fn entry_uv(e: u16) -> [f32; 2] {
    let n = (PAL_N / 4) as u16;
    [((e % n) as f32 * 4.0 + 2.0) / PAL_N as f32, ((e / n) as f32 * 4.0 + 2.0) / PAL_N as f32]
}

/// Entities and meshes of one built version of a chunk.
#[derive(Default)]
struct Part {
    ents: Vec<Entity>,
    meshes: Vec<MeshId>,
    tris: usize,
}

impl Part {
    fn drop(self, ctx: &mut Ctx) {
        for e in self.ents {
            ctx.despawn(e);
        }
        for m in self.meshes {
            ctx.renderer.remove_mesh(m);
        }
    }
}

/// The fine heights (plates) and colours of a chunk, kept for its studs.
struct Grid {
    h: Vec<i16>,
    col: Vec<u8>,
}

#[derive(Default)]
struct Chunk {
    coarse: Option<Part>,
    fine: Option<Part>,
    grid: Option<Grid>,
    colliders: Vec<ColliderHandle>,
    layer: u8,
}

/// Sampling the law for one chunk version, row by row.
struct Job {
    key: (i32, i32),
    fine: bool,
    cells: Vec<Cell>,
}

impl Job {
    fn n(&self) -> usize {
        if self.fine {
            CHUNK as usize
        } else {
            CHUNK as usize / 4
        }
    }
    fn cs(&self) -> f32 {
        if self.fine {
            1.0
        } else {
            4.0
        }
    }
    fn done(&self) -> bool {
        self.cells.len() == (self.n() + 2) * (self.n() + 2)
    }
    /// Samples one more row (with the border ring).
    fn row(&mut self, law: &Island) {
        let (n, cs) = (self.n(), self.cs());
        let w = n + 2;
        let b = self.cells.len() / w;
        let (x0, z0) = (X0 + self.key.0 as f32 * CHUNK, Z0 + self.key.1 as f32 * CHUNK);
        let z = z0 + (b as f32 - 1.0 + 0.5) * cs;
        for a in 0..w {
            self.cells.push(law.cell(x0 + (a as f32 - 1.0 + 0.5) * cs, z));
        }
    }
}

/// The meshes of one chunk version: tops per colour, the sides, the collider boxes (min, max studs).
struct Built {
    tops: Vec<(u8, MeshData)>,
    sides: MeshData,
    boxes: Vec<(Vec3, Vec3)>,
    grid: Option<Grid>,
    tris: usize,
}

fn quad(md: &mut MeshData, p: [Vec3; 4], n: Vec3, uv: [[f32; 2]; 4]) {
    let base = md.vertices.len() as u32;
    for i in 0..4 {
        md.vertices.push(Vertex { pos: p[i].into(), normal: n.into(), uv: uv[i] });
    }
    md.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// The colour segments of one riser (plates, bottom..top): the top's colour for plate steps, rock with strata
/// per brick row below a lip of the top colour for big drops, one dark slab under the sea.
fn riser(out: &mut Vec<(i32, i32, u16)>, top: i32, bot: i32, c: &Cell, along: i32, line: i32) {
    if top <= bot {
        return;
    }
    let drop = top - bot;
    if (top as f32) * PLATE < SEA_Y {
        out.push((bot, top, entry(if drop >= 3 { island::ROCK_DARK } else { c.colour }, 2)));
        return;
    }
    if drop < 3 {
        let j = (island::hash01(line, along.div_euclid(8), 5) * 5.0) as usize;
        out.push((bot, top, entry(c.colour, j)));
        return;
    }
    out.push((top - 1, top, entry(c.colour, 2)));
    let strata: [u8; 4] = if c.warm { [island::SANDSTONE, island::SANDSTONE, island::SANDSTONE_DARK, island::DIRT] } else { [island::ROCK_LIGHT, island::ROCK_MID, island::ROCK_LIGHT, island::ROCK_DARK] };
    let mut cur = top - 1;
    while cur > bot {
        let r = (cur - 1).div_euclid(3);
        let sb = bot.max(r * 3);
        let brick = (along + (r & 1) * 2).div_euclid(4);
        let j = (island::hash01(r, brick * 31 + line, 7) * 5.0) as usize;
        out.push((sb, cur, entry(strata[r.rem_euclid(4) as usize], j)));
        cur = sb;
    }
}

/// Meshes a sampled chunk version (see the module docs).
fn mesh(job: &Job) -> Built {
    let (n, cs) = (job.n(), job.cs());
    let w = n + 2;
    let (x0, z0) = (X0 + job.key.0 as f32 * CHUNK, Z0 + job.key.1 as f32 * CHUNK);
    let c = |a: i32, b: i32| &job.cells[(b + 1) as usize * w + (a + 1) as usize];
    let y = |p: i32| p as f32 * PLATE;
    // ---- tops: greedy rectangles of one height and colour ----
    let mut tops: HashMap<u8, MeshData> = HashMap::new();
    let mut seen = vec![false; n * n];
    let ni = n as i32;
    for b in 0..ni {
        for a in 0..ni {
            if seen[(b * ni + a) as usize] {
                continue;
            }
            let k = c(a, b);
            let key = (k.plates, k.colour);
            let mut a1 = a + 1;
            while a1 < ni && !seen[(b * ni + a1) as usize] && (c(a1, b).plates, c(a1, b).colour) == key {
                a1 += 1;
            }
            let mut b1 = b + 1;
            'grow: while b1 < ni {
                for aa in a..a1 {
                    if seen[(b1 * ni + aa) as usize] || (c(aa, b1).plates, c(aa, b1).colour) != key {
                        break 'grow;
                    }
                }
                b1 += 1;
            }
            for bb in b..b1 {
                for aa in a..a1 {
                    seen[(bb * ni + aa) as usize] = true;
                }
            }
            let (xa, xb) = (x0 + a as f32 * cs, x0 + a1 as f32 * cs);
            let (za, zb) = (z0 + b as f32 * cs, z0 + b1 as f32 * cs);
            let yy = y(key.0);
            // world UVs in studs: one stud of the texture per stud, centred like the geometry studs
            quad(tops.entry(key.1).or_default(), [m(xa, yy, zb), m(xb, yy, zb), m(xb, yy, za), m(xa, yy, za)], Vec3::Y, [[xa, -zb], [xb, -zb], [xb, -za], [xa, -za]]);
        }
    }
    // ---- sides: per direction and line, runs of equal segments merged along the line ----
    let mut sides = MeshData::default();
    let mut segs: Vec<(i32, i32, u16)> = vec![];
    // open runs: (y0, y1, entry, start along)
    let mut open: Vec<(i32, i32, u16, i32)> = vec![];
    let mut next: Vec<(i32, i32, u16, i32)> = vec![];
    for dir in 0..4 {
        for line in 0..ni {
            open.clear();
            for along in 0..=ni {
                segs.clear();
                if along < ni {
                    let (a, b) = if dir < 2 { (line, along) } else { (along, line) };
                    let (da, db) = [(1, 0), (-1, 0), (0, 1), (0, -1)][dir];
                    let me = c(a, b);
                    let nb = c(a + da, b + db);
                    let border = a + da < 0 || a + da >= ni || b + db < 0 || b + db >= ni;
                    // (brick ids from absolute studs, so neighbouring chunks agree)
                    let abs_along = if dir < 2 { (z0 + along as f32 * cs) as i32 } else { (x0 + along as f32 * cs) as i32 };
                    let abs_line = if dir < 2 { (x0 + a as f32 * cs) as i32 * 4 + dir as i32 } else { (z0 + b as f32 * cs) as i32 * 4 + dir as i32 };
                    if border {
                        riser(&mut segs, me.plates, nb.plates.max(FLOOR), me, abs_along, abs_line);
                        let hidden = me.plates.min(nb.plates);
                        if hidden > FLOOR {
                            segs.push((FLOOR, hidden, entry(island::ROCK_DARK, 2)));
                        }
                    } else if me.plates > nb.plates && dir != 3 {
                        // (no north-facing risers inside a chunk: the camera looks north and never sees their front,
                        // the sun lights them from the south; a quarter of the sides saved)
                        riser(&mut segs, me.plates, nb.plates, me, abs_along, abs_line);
                    }
                }
                next.clear();
                for s in &segs {
                    let start = open.iter().find(|o| o.0 == s.0 && o.1 == s.1 && o.2 == s.2).map(|o| o.3).unwrap_or(along);
                    next.push((s.0, s.1, s.2, start));
                }
                for o in &open {
                    if !next.iter().any(|q| q.0 == o.0 && q.1 == o.1 && q.2 == o.2 && q.3 == o.3) {
                        // emit the run [o.3, along)
                        let (s0, s1) = (o.3 as f32 * cs, along as f32 * cs);
                        let (y0, y1) = (y(o.0), y(o.1));
                        let uv = entry_uv(o.2);
                        let uv4 = [uv; 4];
                        let skirt = |at_edge: bool| if at_edge { 0.01 } else { 0.0 };
                        match dir {
                            0 => {
                                let p = x0 + (line + 1) as f32 * cs - skirt(line == ni - 1);
                                let (za, zb) = (z0 + s0, z0 + s1);
                                quad(&mut sides, [m(p, y0, zb), m(p, y0, za), m(p, y1, za), m(p, y1, zb)], Vec3::X, uv4);
                            }
                            1 => {
                                let p = x0 + line as f32 * cs + skirt(line == 0);
                                let (za, zb) = (z0 + s0, z0 + s1);
                                quad(&mut sides, [m(p, y0, za), m(p, y0, zb), m(p, y1, zb), m(p, y1, za)], -Vec3::X, uv4);
                            }
                            2 => {
                                let p = z0 + (line + 1) as f32 * cs - skirt(line == ni - 1);
                                let (xa, xb) = (x0 + s0, x0 + s1);
                                quad(&mut sides, [m(xa, y0, p), m(xb, y0, p), m(xb, y1, p), m(xa, y1, p)], Vec3::Z, uv4);
                            }
                            _ => {
                                let p = z0 + line as f32 * cs + skirt(line == 0);
                                let (xa, xb) = (x0 + s0, x0 + s1);
                                quad(&mut sides, [m(xb, y0, p), m(xa, y0, p), m(xa, y1, p), m(xb, y1, p)], -Vec3::Z, uv4);
                            }
                        }
                    }
                }
                std::mem::swap(&mut open, &mut next);
            }
        }
    }
    // ---- fine: the collider boxes (greedy by height; deep water a tall wall) and the grid for the studs ----
    let mut boxes = vec![];
    let mut grid = None;
    if job.fine {
        let key = |k: &Cell| if k.plates < WADE_MIN { i32::MIN } else { k.plates };
        let mut seen = vec![false; n * n];
        for b in 0..ni {
            for a in 0..ni {
                if seen[(b * ni + a) as usize] {
                    continue;
                }
                let kk = key(c(a, b));
                let mut a1 = a + 1;
                while a1 < ni && !seen[(b * ni + a1) as usize] && key(c(a1, b)) == kk {
                    a1 += 1;
                }
                let mut b1 = b + 1;
                'g: while b1 < ni {
                    for aa in a..a1 {
                        if seen[(b1 * ni + aa) as usize] || key(c(aa, b1)) != kk {
                            break 'g;
                        }
                    }
                    b1 += 1;
                }
                for bb in b..b1 {
                    for aa in a..a1 {
                        seen[(bb * ni + aa) as usize] = true;
                    }
                }
                let top = if kk == i32::MIN { SEA_Y + 10.0 } else { y(kk) };
                boxes.push((Vec3::new(x0 + a as f32, y(FLOOR), z0 + b as f32), Vec3::new(x0 + a1 as f32, top, z0 + b1 as f32)));
            }
        }
        let mut g = Grid { h: Vec::with_capacity(n * n), col: Vec::with_capacity(n * n) };
        for b in 0..ni {
            for a in 0..ni {
                g.h.push(c(a, b).plates as i16);
                g.col.push(c(a, b).colour);
            }
        }
        grid = Some(g);
    }
    let mut tops: Vec<(u8, MeshData)> = tops.into_iter().collect();
    tops.sort_by_key(|t| t.0);
    let tris = tops.iter().map(|t| t.1.indices.len() / 3).sum::<usize>() + sides.indices.len() / 3;
    Built { tops, sides, boxes, grid, tris }
}

/// The real studs of one 8x8 tile (palette colours; 8 sides, a flat top).
fn stud_tile(grid: &Grid, x0: f32, z0: f32, a0: usize, b0: usize) -> MeshData {
    let mut md = MeshData::default();
    let (r, h) = (0.3 * STUD, (0.17 - 0.01) * STUD);
    let ring: Vec<(f32, f32)> = (0..8).map(|k| {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        (a.cos(), a.sin())
    }).collect();
    for b in b0..b0 + TILE as usize {
        for a in a0..a0 + TILE as usize {
            let i = b * CHUNK as usize + a;
            let y = grid.h[i] as f32 * PLATE;
            if y < SEA_Y + 0.3 {
                continue;
            }
            let uv = entry_uv(entry(grid.col[i], 2));
            let c = m(x0 + a as f32 + 0.5, y, z0 + b as f32 + 0.5);
            let base = md.vertices.len() as u32;
            for &(cx, sz) in &ring {
                let n = [cx, 0.0, sz];
                md.vertices.push(Vertex { pos: [c.x + cx * r, c.y, c.z + sz * r], normal: n, uv });
                md.vertices.push(Vertex { pos: [c.x + cx * r, c.y + h, c.z + sz * r], normal: n, uv });
            }
            for k in 0..8u32 {
                let k1 = (k + 1) % 8;
                let (bk, tk, bk1, tk1) = (base + k * 2, base + k * 2 + 1, base + k1 * 2, base + k1 * 2 + 1);
                md.indices.extend_from_slice(&[bk, tk, tk1, bk, tk1, bk1]);
            }
            let cap = md.vertices.len() as u32;
            for &(cx, sz) in &ring {
                md.vertices.push(Vertex { pos: [c.x + cx * r, c.y + h, c.z + sz * r], normal: [0.0, 1.0, 0.0], uv });
            }
            for k in 1..7u32 {
                md.indices.extend_from_slice(&[cap, cap + k + 1, cap + k]);
            }
        }
    }
    md
}

#[derive(Default, Clone, serde::Serialize)]
pub struct ViewStats {
    pub builds: u32,
    pub fine_builds: u32,
    pub coarse_builds: u32,
    /// the most time one frame spent building (ms), and the last frame's
    pub max_frame_ms: f32,
    pub last_frame_ms: f32,
    /// the slowest single fine / coarse mesh + upload step (ms)
    pub max_fine_ms: f32,
    pub max_coarse_ms: f32,
    /// of those: meshing (CPU), GPU upload + spawn
    pub max_mesh_ms: f32,
    pub max_upload_ms: f32,
    pub unloads: u32,
}

/// The island's chunks, studs, colliders and sea in the play screen.
#[derive(Default)]
pub struct IslandView {
    tops: Vec<MaterialId>,
    bricks: Option<MaterialId>,
    sea_mat: Option<MaterialId>,
    sea_mesh: Option<MeshId>,
    sea: Option<Entity>,
    chunks: HashMap<(i32, i32), Chunk>,
    tiles: HashMap<(i32, i32), (Entity, MeshId, usize)>,
    job: Option<Job>,
    /// the last place the layers were sorted for (8-stud cell) and whether the camera looked far
    sorted: Option<((i32, i32), bool)>,
    pub stats: ViewStats,
    /// the frame budget for building (ms)
    pub budget_ms: f32,
    pub phone: bool,
    pub active: bool,
}

impl IslandView {
    /// The materials (once per play screen): a stud-textured plastic per top colour, the palette for the risers
    /// and studs, the sea.
    pub fn init(&mut self, ctx: &mut Ctx, town: &TownAssets) {
        self.budget_ms = 3.0;
        for (i, hex) in PALETTE.iter().enumerate() {
            let mat = Material { name: format!("isle top {i} {hex:06x}"), texture: town.stud_tex, normal_texture: town.stud_nrm, normal_scale: 1.0, ..Material::color(*hex).rough(crate::PLASTIC_ROUGH) };
            self.tops.push(ctx.renderer.add_material(mat));
        }
        let mut px = vec![0u8; (PAL_N * PAL_N * 4) as usize];
        let per = PAL_N / 4;
        for (ci, hex) in PALETTE.iter().enumerate() {
            for (j, k) in island::JITTER.iter().enumerate() {
                let e = entry(ci as u8, j) as u32;
                let (ex, ey) = ((e % per) * 4, (e / per) * 4);
                let ch = |s: u32| ((((hex >> s) & 255) as f32) * k).round().clamp(0.0, 255.0) as u8;
                for yy in ey..ey + 4 {
                    for xx in ex..ex + 4 {
                        let o = ((yy * PAL_N + xx) * 4) as usize;
                        px[o..o + 4].copy_from_slice(&[ch(16), ch(8), ch(0), 255]);
                    }
                }
            }
        }
        let pal = ctx.renderer.add_texture_rgba(PAL_N, PAL_N, &px, "island palette").ok();
        self.bricks = Some(ctx.renderer.add_material(Material { name: "isle bricks".into(), texture: pal, ..Material::color(0xffffff).rough(crate::PLASTIC_ROUGH) }));
        // the sea: clear over the sand shelf, deep blue further out
        self.sea_mat = Some(ctx.renderer.add_material(Material {
            name: "isle sea water".into(),
            wave_height: 0.03,
            wave_scale: 1.2,
            water_chop: 0.35,
            water_clarity: 1.6,
            water_foam: 0.15,
            water_vertex_spacing: 0.35,
            ..Material::water(0x15506c, 0x3fa7c9)
        }));
    }

    /// The camera's island layers: the near chunks and the sea, the far chunks too while it looks far.
    pub fn layers(&self, far: bool) -> u32 {
        if !self.active {
            return 0;
        }
        (1 << island::LAYER_NEAR) | (1 << island::LAYER_SEA) | if far { 1 << island::LAYER_FAR } else { 0 }
    }

    fn spawn_built(&mut self, ctx: &mut Ctx, key: (i32, i32), b: Built, fine: bool) -> Part {
        let mut part = Part { tris: b.tris, ..Default::default() };
        let name = if fine { "isle fine" } else { "isle coarse" };
        for (colour, md) in &b.tops {
            let mesh = ctx.renderer.add_mesh(&format!("{name} {},{} top {colour}", key.0, key.1), md);
            let e = ctx.spawn_mesh(&format!("{name} {},{}", key.0, key.1), mesh, self.tops[*colour as usize], Transform::default());
            part.ents.push(e);
            part.meshes.push(mesh);
        }
        if !b.sides.indices.is_empty() {
            let mesh = ctx.renderer.add_mesh(&format!("{name} {},{} sides", key.0, key.1), &b.sides);
            let e = ctx.spawn_mesh(&format!("{name} {},{}", key.0, key.1), mesh, self.bricks.unwrap(), Transform::default());
            part.ents.push(e);
            part.meshes.push(mesh);
        }
        let layer = self.chunks.get(&key).map(|c| c.layer).unwrap_or(island::LAYER_FAR);
        for e in &part.ents {
            ctx.set_layer(*e, layer);
        }
        part
    }

    /// Finishes a sampled job: meshes, uploads, colliders; the fine version replaces the coarse one.
    fn finish(&mut self, ctx: &mut Ctx, job: Job) {
        let t0 = engine::log::now_ms();
        let key = job.key;
        let fine = job.fine;
        let mut b = mesh(&job);
        let t1 = engine::log::now_ms();
        let grid = b.grid.take();
        let boxes = std::mem::take(&mut b.boxes);
        let part = self.spawn_built(ctx, key, b, fine);
        let t2 = engine::log::now_ms();
        self.stats.max_mesh_ms = self.stats.max_mesh_ms.max((t1 - t0) as f32);
        self.stats.max_upload_ms = self.stats.max_upload_ms.max((t2 - t1) as f32);
        let ch = self.chunks.entry(key).or_default();
        if fine {
            for (lo, hi) in boxes {
                let half = (hi - lo) * 0.5 * STUD;
                let h = ctx.physics.add_static(ColliderBuilder::cuboid(half.x, half.y, half.z), (lo + hi) * 0.5 * STUD);
                ch.colliders.push(h);
            }
            ch.grid = grid;
            ch.fine = Some(part);
            if let Some(c) = ch.coarse.take() {
                c.drop(ctx);
            }
        } else {
            ch.coarse = Some(part);
        }
        let ms = (engine::log::now_ms() - t0) as f32;
        self.stats.builds += 1;
        if fine {
            self.stats.fine_builds += 1;
            self.stats.max_fine_ms = self.stats.max_fine_ms.max(ms);
        } else {
            self.stats.coarse_builds += 1;
            self.stats.max_coarse_ms = self.stats.max_coarse_ms.max(ms);
        }
        self.sorted = None;
    }

    fn drop_fine(&mut self, ctx: &mut Ctx, key: (i32, i32)) {
        let Some(ch) = self.chunks.get_mut(&key) else { return };
        if let Some(p) = ch.fine.take() {
            p.drop(ctx);
        }
        for h in ch.colliders.drain(..) {
            ctx.physics.world.remove_collider(h);
        }
        ch.grid = None;
        let gone: Vec<(i32, i32)> = self.tiles.keys().filter(|t| t.0 * TILE / CHUNK as i32 == key.0 && t.1 * TILE / CHUNK as i32 == key.1).copied().collect();
        for t in gone {
            if let Some((e, mesh, _)) = self.tiles.remove(&t) {
                ctx.despawn(e);
                ctx.renderer.remove_mesh(mesh);
            }
        }
        self.stats.unloads += 1;
    }

    fn drop_all(&mut self, ctx: &mut Ctx, key: (i32, i32)) {
        self.drop_fine(ctx, key);
        if let Some(mut ch) = self.chunks.remove(&key) {
            if let Some(p) = ch.coarse.take() {
                p.drop(ctx);
            }
        }
    }

    /// Builds the chunk under (x, z) at once (a teleport, a run faster than the streaming).
    pub fn ensure_at(&mut self, ctx: &mut Ctx, law: &Island, x: f32, z: f32) {
        let key = (((x - X0) / CHUNK).floor() as i32, ((z - Z0) / CHUNK).floor() as i32);
        if key.0 < 0 || key.1 < 0 || key.0 >= island::CX || key.1 >= island::CZ || self.chunks.get(&key).is_some_and(|c| c.fine.is_some()) {
            return;
        }
        if self.job.as_ref().is_some_and(|j| j.key == key && j.fine) {
            let mut j = self.job.take().unwrap();
            while !j.done() {
                j.row(law);
            }
            self.finish(ctx, j);
            return;
        }
        let mut j = Job { key, fine: true, cells: vec![] };
        while !j.done() {
            j.row(law);
        }
        self.finish(ctx, j);
    }

    /// Every frame: loads and unloads around you (`hero` in studs) within the budget, sorts the layers, the studs.
    pub fn update(&mut self, ctx: &mut Ctx, law: &Island, hero: Vec2, far: bool) {
        let t0 = engine::log::now_ms();
        let near_island = hero.y < island::SOUTH_Z + 60.0;
        if !near_island {
            if self.active {
                self.active = false;
                let keys: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
                for k in keys {
                    self.drop_all(ctx, k);
                }
                self.job = None;
                if let Some(s) = self.sea.take() {
                    ctx.despawn(s);
                }
            }
            return;
        }
        self.active = true;
        if self.sea.is_none() {
            if let Some(mat) = self.sea_mat {
                // the engine's ocean mesh, but small: `spawn_ocean` reaches 5 km with 256 segments (~145k triangles,
                // more than the phone's whole budget); the camera sees ~60 m and fog takes the rest. The `Ocean`
                // component keeps it under the camera as with spawn_ocean
                let mesh = *self.sea_mesh.get_or_insert_with(|| ctx.renderer.add_mesh("isle sea", &engine::water::ocean_mesh(SEA_RADIUS, 0.6, 48)));
                let s = ctx.spawn_mesh("isle sea", mesh, mat, Transform::from_xyz(0.0, SEA_Y * STUD, 0.0));
                let _ = ctx.world.insert_one(s, engine::water::Ocean { snap: 2.4 });
                ctx.set_layer(s, island::LAYER_SEA);
                if let Ok(mut r) = ctx.world.get::<&mut MeshRenderer>(s) {
                    r.cast_shadows = false;
                }
                self.sea = Some(s);
            }
        }
        // ---- unload what is far ----
        let keys: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        let mut converted = false;
        for k in keys {
            let ch = &self.chunks[&k];
            if !rect_hits(k, hero, COARSE_R, 64.0) {
                self.drop_all(ctx, k);
            } else if ch.fine.is_some() && !rect_hits(k, hero, FINE_R, 32.0) && !converted {
                // (one per frame: a fine chunk a little too far does no harm meanwhile)
                converted = true;
                // (the coarse version first, so it never leaves a hole)
                let mut j = Job { key: k, fine: false, cells: vec![] };
                while !j.done() {
                    j.row(law);
                }
                self.drop_fine(ctx, k);
                self.finish(ctx, j);
            }
        }
        // ---- the chunk you stand in, at once ----
        self.ensure_at(ctx, law, hero.x, hero.y);
        // ---- build within the budget: holes in view first, then fine near you, then coarse further out ----
        // (a sampled fine chunk is meshed, uploaded and given its colliders in a frame of its own, ~3 ms; the cheap coarse
        // ones finish right after their samples while the budget lasts)
        let left = |t0: f64, budget: f32| budget - (engine::log::now_ms() - t0) as f32;
        loop {
            if left(t0, self.budget_ms) <= 0.0 {
                break;
            }
            if self.job.is_none() {
                self.job = self.next_job(hero);
            }
            let Some(mut j) = self.job.take() else { break };
            if j.done() {
                let fine = j.fine;
                self.finish(ctx, j);
                if fine {
                    break;
                }
                continue;
            }
            while !j.done() && left(t0, self.budget_ms) > 0.0 {
                j.row(law);
            }
            if j.done() && !j.fine && left(t0, self.budget_ms) > 1.5 {
                self.finish(ctx, j);
                continue;
            }
            self.job = Some(j);
            break;
        }
        // ---- layers: near chunks drawn, the rest only while the camera looks far ----
        let cell = ((hero.x / 8.0).floor() as i32, (hero.y / 8.0).floor() as i32);
        if self.sorted != Some((cell, far)) {
            self.sorted = Some((cell, far));
            for (k, ch) in self.chunks.iter_mut() {
                let layer = if rect_hits(*k, hero, VIEW_R, 0.0) { island::LAYER_NEAR } else { island::LAYER_FAR };
                if layer != ch.layer {
                    ch.layer = layer;
                    for p in [&ch.fine, &ch.coarse].into_iter().flatten() {
                        for e in &p.ents {
                            ctx.set_layer(*e, layer);
                        }
                    }
                }
            }
            self.studs(ctx, hero);
        }
        let ms = (engine::log::now_ms() - t0) as f32;
        self.stats.last_frame_ms = ms;
        self.stats.max_frame_ms = self.stats.max_frame_ms.max(ms);
    }

    fn next_job(&self, hero: Vec2) -> Option<Job> {
        let lo = (((hero.x - COARSE_R[0] - X0) / CHUNK).floor() as i32).max(0);
        let hi = (((hero.x + COARSE_R[1] - X0) / CHUNK).floor() as i32).min(island::CX - 1);
        let zlo = (((hero.y - COARSE_R[2] - Z0) / CHUNK).floor() as i32).max(0);
        let zhi = (((hero.y + COARSE_R[3] - Z0) / CHUNK).floor() as i32).min(island::CZ - 1);
        let mut best: Option<(f32, (i32, i32), bool)> = None;
        for ck in zlo..=zhi {
            for ci in lo..=hi {
                let k = (ci, ck);
                if !rect_hits(k, hero, COARSE_R, 0.0) {
                    continue;
                }
                let ch = self.chunks.get(&k);
                let has_fine = ch.is_some_and(|c| c.fine.is_some());
                let has_any = has_fine || ch.is_some_and(|c| c.coarse.is_some());
                let d = chunk_dist(k, hero);
                let in_view = rect_hits(k, hero, VIEW_R, 16.0);
                let cand = if !has_any && in_view {
                    Some((d - 1000.0, false)) // a hole in view: the cheap version first
                } else if !has_fine && rect_hits(k, hero, FINE_R, 0.0) {
                    Some((d, true))
                } else if !has_any {
                    Some((d + 500.0, false))
                } else {
                    None
                };
                if let Some((p, fine)) = cand {
                    if best.is_none_or(|b| p < b.0) {
                        best = Some((p, k, fine));
                    }
                }
            }
        }
        best.map(|(_, key, fine)| Job { key, fine, cells: vec![] })
    }

    /// The real studs near you: 8x8 tiles within the radius, built from the fine chunks' grids.
    fn studs(&mut self, ctx: &mut Ctx, hero: Vec2) {
        let r = if self.phone { STUD_R_PHONE } else { STUD_R };
        let t = |v: f32, o: f32| ((v - o) / TILE as f32).floor() as i32;
        let (tx0, tx1) = (t(hero.x - r, X0), t(hero.x + r, X0));
        let (tz0, tz1) = (t(hero.y - r, Z0), t(hero.y + r, Z0));
        let want = |k: (i32, i32), grow: f32| {
            // (by the tile's centre: tiles cover about the circle, not every tile it touches)
            let c = Vec2::new(X0 + (k.0 * TILE) as f32, Z0 + (k.1 * TILE) as f32) + Vec2::splat(TILE as f32 * 0.5);
            c.distance(hero) < r + grow
        };
        let gone: Vec<(i32, i32)> = self.tiles.keys().filter(|k| !want(**k, 6.0)).copied().collect();
        for k in gone {
            if let Some((e, mesh, _)) = self.tiles.remove(&k) {
                ctx.despawn(e);
                ctx.renderer.remove_mesh(mesh);
            }
        }
        let per = CHUNK as i32 / TILE;
        for tk in tz0..=tz1 {
            for tx in tx0..=tx1 {
                let k = (tx, tk);
                if tx < 0 || tk < 0 || self.tiles.contains_key(&k) || !want(k, 0.0) {
                    continue;
                }
                let ck = (tx.div_euclid(per), tk.div_euclid(per));
                let Some(g) = self.chunks.get(&ck).and_then(|c| c.grid.as_ref()) else { continue };
                let (cx0, cz0) = (X0 + ck.0 as f32 * CHUNK, Z0 + ck.1 as f32 * CHUNK);
                let md = stud_tile(g, cx0, cz0, (tx.rem_euclid(per) * TILE) as usize, (tk.rem_euclid(per) * TILE) as usize);
                if md.indices.is_empty() {
                    continue;
                }
                let tris = md.indices.len() / 3;
                let mesh = ctx.renderer.add_mesh(&format!("isle studs {tx},{tk}"), &md);
                let e = ctx.spawn_mesh(&format!("isle studs {tx},{tk}"), mesh, self.bricks.unwrap(), Transform::default());
                if let Ok(mut r) = ctx.world.get::<&mut MeshRenderer>(e) {
                    r.cast_shadows = false;
                }
                ctx.set_layer(e, island::LAYER_NEAR);
                self.tiles.insert(k, (e, mesh, tris));
            }
        }
    }

    /// Everything off (leaving the play screen): entities, meshes, colliders.
    pub fn clear(&mut self, ctx: &mut Ctx) {
        let keys: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        for k in keys {
            self.drop_all(ctx, k);
        }
        for (_, (e, mesh, _)) in self.tiles.drain() {
            ctx.despawn(e);
            ctx.renderer.remove_mesh(mesh);
        }
        if let Some(s) = self.sea.take() {
            ctx.despawn(s);
        }
        self.job = None;
        self.active = false;
    }

    pub fn describe(&self) -> Value {
        let fine = self.chunks.values().filter(|c| c.fine.is_some()).count();
        let coarse = self.chunks.values().filter(|c| c.coarse.is_some()).count();
        let near_tris: usize = self.chunks.values().filter(|c| c.layer == island::LAYER_NEAR).map(|c| c.fine.as_ref().or(c.coarse.as_ref()).map(|p| p.tris).unwrap_or(0)).sum();
        json!({
            "active": self.active,
            "fine": fine,
            "coarse": coarse,
            "near": self.chunks.values().filter(|c| c.layer == island::LAYER_NEAR).count(),
            "near_tris": near_tris,
            "stud_tiles": self.tiles.len(),
            "stud_tris": self.tiles.values().map(|t| t.2).sum::<usize>(),
            "colliders": self.chunks.values().map(|c| c.colliders.len()).sum::<usize>(),
            "job": self.job.as_ref().map(|j| json!({"key": [j.key.0, j.key.1], "fine": j.fine, "rows": j.cells.len() / (j.n() + 2)})),
            "sea": self.sea.is_some(),
            "stats": self.stats,
        })
    }
}

