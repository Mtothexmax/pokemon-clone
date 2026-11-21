//! The pond cave (Kaigen `st_cave.c`): under the pond lies a flooded basin of LEGO rock, built far east of
//! everything (x 600..680 studs) on its own render layer ([`CAVE_LAYER`]) and drawn only while you dive.
//! Inspired by abyssal-garden (github.com/Noniv/abyssal-garden): a kelp forest, a purple grotto under an
//! overhang, coral and sponges on the ledges, a stone whale tail, a sunken boat and a deep trench; from its bottom a
//! tunnel runs through the north wall into the deep grotto, where the Stud Core lies. The living parts (kelp, fish,
//! mantas, bubbles, glowing plants, pearls, lights) and the swimming are `dive.rs`. Cave coordinates are studs from (CAVE_X, 0, CAVE_Z).

use crate::town::{self, TownAssets, STUD};
use engine::geom::MeshBuilder;
use engine::prelude::*;
use engine::renderer::MaterialId;
use std::collections::HashMap;

pub const CAVE_X: f32 = 640.0;
pub const CAVE_Z: f32 = 0.0;
/// The basin: x and z in [-R, R].
pub const CAVE_R: f32 = 40.0;
/// The ceiling; the hole up to the pond in its middle (|x|, |z| < CAVE_HOLE).
pub const CAVE_TOP: f32 = 80.0;
pub const CAVE_HOLE: f32 = 5.0;
const CAVE_CELL: f32 = 8.0;
/// The tunnel from the bottom of the trench through the north wall (x within TUN_W / 2, y TUN_Y0..TUN_Y1, z from
/// CAVE_R to DEEP_Z0) into the deep grotto (x within DEEP_W / 2, y DEEP_Y0..DEEP_Y1, z DEEP_Z0..DEEP_Z1), where the
/// Stud Core waits on a rock column at the far end (`DEEP_CORE`).
pub const TUN_W: f32 = 8.0;
pub const TUN_Y0: f32 = -24.0;
pub const TUN_Y1: f32 = -15.0;
pub const DEEP_W: f32 = 36.0;
pub const DEEP_Y0: f32 = -34.0;
pub const DEEP_Y1: f32 = -6.0;
pub const DEEP_Z0: f32 = 76.0;
pub const DEEP_Z1: f32 = 116.0;
/// The green warp pipe at the grotto's far end (where the core's column stood): swim down into its opening and you
/// come out of the pipe in the core room (`CORE_ROOM_*`), a small cave below and beyond, where the Stud Core lies;
/// its own pipe takes you back.
pub const PIPE_AT: Vec3 = Vec3::new(0.0, DEEP_Y0, 104.0);
pub const PIPE_TOP: f32 = -22.0;
pub const PIPE_R: f32 = 3.2;
pub const CR_W: f32 = 22.0;
pub const CR_Y0: f32 = -66.0;
pub const CR_Y1: f32 = -46.0;
pub const CR_Z0: f32 = 124.0;
pub const CR_Z1: f32 = 148.0;
pub const PIPE2_AT: Vec3 = Vec3::new(0.0, CR_Y0, 129.0);
pub const PIPE2_TOP: f32 = CR_Y0 + 6.0;
pub const DEEP_CORE: Vec3 = Vec3::new(0.0, CR_Y0 + 5.0, 142.0);

/// Where the diver is (dive.rs collision, the debug state).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Zone {
    Basin,
    Tunnel,
    Deep,
    /// the small cave behind the warp pipe, with the Stud Core
    Core,
}

/// The cave's render layer (village 1, rooms 2..5, the battle 29, hidden 31).
pub const CAVE_LAYER: u8 = 8;

/// Something solid the diver bumps into (cave studs).
#[derive(Clone, Copy, Debug)]
pub struct CaveBox {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub z0: f32,
    pub z1: f32,
}

/// Kaigen `cave_hash`: 0..1 from two ints.
pub fn hash(a: i32, b: i32) -> f32 {
    let mut h = (a.wrapping_mul(73856093) as u32) ^ (b.wrapping_mul(19349663) as u32) ^ 0x9e3779b9;
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

/// The floor under (x, z): low hills of sand and rock, the trench in the north (x within 12, z past 14) far below.
pub fn floor(x: f32, z: f32) -> f32 {
    let i = ((x + CAVE_R) / CAVE_CELL).floor() as i32;
    let k = ((z + CAVE_R) / CAVE_CELL).floor() as i32;
    let cx = -CAVE_R + (i as f32 + 0.5) * CAVE_CELL;
    let cz = -CAVE_R + (k as f32 + 0.5) * CAVE_CELL;
    if cx.abs() < 12.0 && cz > 14.0 {
        return if cz > 30.0 { -24.0 } else { -14.0 }; // the trench, in two steps
    }
    let rim = if cx.abs().max(cz.abs()) > 30.0 { 4.0 } else { 0.0 }; // the floor rises towards the walls
    1.0 + 3.0 * hash(i, k) + rim
}

/// Cave studs -> world metres.
pub fn world(c: Vec3) -> Vec3 {
    town::m(CAVE_X + c.x, c.y, CAVE_Z + c.z)
}

/// The cave being built: one mesh part per colour, the solids.
struct Build {
    mb: MeshBuilder,
    solids: Vec<CaveBox>,
    bricks: u32,
}

impl Build {
    /// A brick in cave coordinates, its bottom at y (Kaigen `town_box`).
    fn bx(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32) {
        // touching bricks would share faces: each a few hundredths of a stud off (as the town does)
        self.bricks += 1;
        let e = 0.01 * ((self.bricks % 7) as i32 - 3) as f32;
        self.mb.part(&format!("c{hex:06x}")).block(world(Vec3::new(x, y, z)), town::m(w + e, h + e, d - e));
    }
    /// A solid brick (Kaigen `cave_rock`).
    fn rock(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32, solid: bool) {
        self.bx(hex, w, h, d, x, y, z);
        if solid {
            self.solid(x - w * 0.5, x + w * 0.5, y, y + h, z - d * 0.5, z + d * 0.5);
        }
    }
    fn solid(&mut self, x0: f32, x1: f32, y0: f32, y1: f32, z0: f32, z1: f32) {
        self.solids.push(CaveBox { x0, x1, y0, y1, z0, z1 });
    }
    /// A rotated brick about its centre (cave studs, euler XYZ).
    fn rot(&mut self, hex: u32, size: Vec3, c: Vec3, euler: Vec3) {
        let t = Mat4::from_rotation_translation(Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z), world(c));
        let s = town::m(size.x, size.y, size.z) * 0.5;
        self.mb.part(&format!("c{hex:06x}")).with(t, |b| {
            b.cuboid(-s, s);
        });
    }
    /// A cylinder standing on y (cave studs).
    fn cyl(&mut self, hex: u32, x: f32, y: f32, z: f32, rb: f32, rt: f32, h: f32, seg: u32) {
        let profile = [Vec2::new(0.0, 0.0), Vec2::new(rb * STUD, 0.0), Vec2::new(rt * STUD, h * STUD), Vec2::new(0.0, h * STUD)];
        self.mb.part(&format!("c{hex:06x}")).with(Mat4::from_translation(world(Vec3::new(x, y, z))), |b| {
            b.lathe(&profile, seg);
        });
    }
    /// Coral: a stem and branches going up and out.
    fn coral(&mut self, x: f32, y: f32, z: f32, hex: u32, s: f32) {
        self.bx(hex, 0.5 * s, 1.6 * s, 0.5 * s, x, y, z);
        for b in 0..4 {
            let a = b as f32 * 1.57 + hash((x * 7.0) as i32, b) * 0.8;
            let len = (1.0 + hash(b, (z * 5.0) as i32)) * s;
            let c = Vec3::new(x + a.sin() * len * 0.4, y + 1.4 * s + len * 0.4, z + a.cos() * len * 0.4);
            self.rot(hex, Vec3::new(0.35 * s, len, 0.35 * s), c, Vec3::new(a.cos() * 0.6, 0.0, -a.sin() * 0.6));
            self.bx(hex, 0.55 * s, 0.55 * s, 0.55 * s, c.x + a.sin() * len * 0.3, c.y + len * 0.4, c.z + a.cos() * len * 0.3);
        }
    }
    /// A barrel sponge: a tube with a dark throat.
    fn sponge(&mut self, x: f32, y: f32, z: f32, h: f32, hex: u32) {
        self.cyl(hex, x, y, z, 1.1, 1.5, h, 10);
        self.cyl(0x2a120a, x, y + h - 0.2, z, 1.0, 1.0, 0.25, 10);
    }
}

const CORALS: [u32; 4] = [0xd63a3a, 0xff7aa8, 0xff9a3c, 0x9b4dd6];

/// A green warp pipe (Super Mario Land) standing on `y0` at (x, z), its top (the rim's upper face) at `top`: a body,
/// a wider rim, the dark opening, a light stripe down its front. Solid up to its top.
fn pipe(t: &mut Build, x: f32, y0: f32, top: f32, z: f32) {
    let (green, rim, light, dark) = (0x1e9e3a, 0x26b847, 0x7fe08f, 0x061a0c);
    t.cyl(green, x, y0, z, PIPE_R, PIPE_R, top - 2.0 - y0, 20);
    t.cyl(rim, x, top - 2.0, z, PIPE_R + 0.8, PIPE_R + 0.8, 2.0, 20);
    t.cyl(dark, x, top - 0.02, z, PIPE_R - 0.4, PIPE_R - 0.4, 0.06, 20);
    // the shine: a pale stripe down the body and the rim (toward the tunnel, -z)
    t.bx(light, 0.7, top - 2.4 - y0, 0.3, x - PIPE_R * 0.45, y0 + 0.2, z - PIPE_R * 0.9);
    t.bx(light, 0.8, 1.6, 0.3, x - (PIPE_R + 0.8) * 0.45, top - 1.8, z - (PIPE_R + 0.8) * 0.9);
    t.solid(x - PIPE_R - 0.8, x + PIPE_R + 0.8, y0, top, z - PIPE_R - 0.8, z + PIPE_R + 0.8);
}

/// The user's LEGO coral (`createCoralModel`): a dark tan base plate (12 x 10) with studs round its edge, a sand plate
/// (8 x 6), sand rocks, coloured 1x1 plates, and on it the green branching coral element (closed, rounded tips) with
/// four small companions in coral, pink, gold and aqua. `at`: the base's centre bottom (cave studs); `k` cave studs
/// per model unit (the model's stud is 0.8 units).
fn lego_coral(t: &mut Build, at: Vec3, k: f32) {
    const U: f32 = 0.8;
    const BRICK_H: f32 = 0.96;
    const PLATE_H: f32 = 0.32;
    let (coral_green, coral, pink, gold, aqua, dark_tan, sand, light_sand) = (0x2fbf48, 0xff5e5b, 0xff4fa3, 0xf2b705, 0x1cc6c6, 0x8f7a56, 0xd9c08a, 0xe8d6a8);
    let pos = |x: f32, y: f32, z: f32| at + Vec3::new(x, y, z) * k;
    // a brick on the stud grid (gx, gz its corner in studs, gy its bottom), real studs where `studs` says
    let brick = |t: &mut Build, gx: f32, gz: f32, gy: f32, w: f32, d: f32, h: f32, hex: u32, studs: &dyn Fn(i32, i32) -> bool| {
        let c = pos((gx + w * 0.5) * U, gy, (gz + d * 0.5) * U);
        t.bx(hex, (w * U - 0.016) * k, h * k, (d * U - 0.016) * k, c.x, c.y, c.z);
        for i in 0..w as i32 {
            for j in 0..d as i32 {
                if studs(i, j) {
                    let s = pos((gx + i as f32 + 0.5) * U, gy + h, (gz + j as f32 + 0.5) * U);
                    t.cyl(hex, s.x, s.y, s.z, 0.24 * k, 0.24 * k, 0.17 * k, 10);
                }
            }
        }
    };
    let all = |_: i32, _: i32| true;
    brick(t, -6.0, -5.0, 0.0, 12.0, 10.0, PLATE_H, dark_tan, &|i, j| !((2..10).contains(&i) && (2..8).contains(&j)));
    brick(t, -4.0, -3.0, PLATE_H, 8.0, 6.0, PLATE_H, sand, &|i, j| !((3..5).contains(&i) && (2..4).contains(&j)));
    brick(t, -4.0, -3.0, PLATE_H * 2.0, 2.0, 2.0, BRICK_H * 0.5, light_sand, &all);
    brick(t, -4.0, -3.0, PLATE_H * 2.0 + BRICK_H * 0.5, 2.0, 2.0, PLATE_H, dark_tan, &|i, j| !(i == 1 && j == 1));
    brick(t, 2.0, 1.0, PLATE_H * 2.0, 2.0, 2.0, PLATE_H, light_sand, &all);
    brick(t, 2.0, 1.0, PLATE_H * 3.0, 1.0, 2.0, PLATE_H, dark_tan, &|_, j| j == 0);
    brick(t, -1.0, 2.0, PLATE_H * 2.0, 1.0, 1.0, PLATE_H, gold, &all);
    brick(t, -6.0, -5.0, PLATE_H, 1.0, 1.0, PLATE_H, pink, &all);
    brick(t, 5.0, -5.0, PLATE_H, 1.0, 1.0, PLATE_H, aqua, &all);
    brick(t, 5.0, 4.0, PLATE_H, 2.0, 1.0, PLATE_H, coral, &all);
    brick(t, -6.0, 4.0, PLATE_H, 1.0, 1.0, PLATE_H, gold, &all);
    // the coral element: branches from the stem to closed tips (picture pixels -> units, the foot at 0)
    let p = |x: f32, y: f32, z: f32| Vec3::new((x - 68.0) * 0.04, (132.0 - y) * 0.04, z);
    let branches: [(f32, Vec<Vec3>); 5] = [
        (0.21, vec![p(68.0, 130.0, 0.0), p(67.0, 108.0, 0.0), p(58.0, 88.0, 0.1), p(55.0, 66.0, 0.0), p(62.0, 42.0, -0.1), p(76.0, 22.0, 0.0), p(82.0, 9.0, 0.0)]),
        (0.2, vec![p(62.0, 96.0, 0.0), p(50.0, 86.0, 0.15), p(40.0, 74.0, 0.0), p(36.0, 60.0, 0.1), p(40.0, 46.0, 0.0)]),
        (0.2, vec![p(66.0, 92.0, 0.0), p(80.0, 86.0, -0.15), p(94.0, 80.0, 0.0), p(105.0, 70.0, -0.1), p(108.0, 58.0, 0.0)]),
        (0.19, vec![p(67.0, 120.0, 0.0), p(54.0, 126.0, 0.1), p(38.0, 128.0, 0.0), p(28.0, 122.0, 0.0), p(26.0, 110.0, 0.0)]),
        (0.17, vec![p(57.0, 76.0, 0.0), p(68.0, 68.0, 0.12), p(74.0, 56.0, 0.0), p(72.0, 46.0, 0.0)]),
    ];
    // Catmull-Rom through the points (tension 0.5), 8 samples per span
    let spline = |pts: &[Vec3]| -> Vec<Vec3> {
        let n = pts.len();
        let g = |i: i32| pts[i.clamp(0, n as i32 - 1) as usize];
        let mut out = vec![];
        for s in 0..n - 1 {
            let (p0, p1, p2, p3) = (g(s as i32 - 1), g(s as i32), g(s as i32 + 1), g(s as i32 + 2));
            for q in 0..8 {
                let u = q as f32 / 8.0;
                let (u2, u3) = (u * u, u * u * u);
                out.push(0.5 * ((2.0 * p1) + (-p0 + p2) * u + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2 + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u3));
            }
        }
        out.push(pts[n - 1]);
        out
    };
    let piece = |t: &mut Build, gx: f32, gz: f32, gy: f32, hex: u32, scale: f32, rot: f32, skip: &[usize]| {
        let base = pos((gx + 0.5) * U, gy, (gz + 0.5) * U);
        let tr = |v: Vec3| base + Quat::from_rotation_y(rot) * (v * scale * k);
        // the sleeve on its stud and the ring at its foot
        t.cyl(hex, base.x, base.y, base.z, 0.3 * scale * k, 0.3 * scale * k, 0.34 * scale * k, 16);
        t.cyl(hex, base.x, base.y, base.z, 0.36 * scale * k, 0.36 * scale * k, 0.07 * scale * k, 16);
        for (bi, (r, pts)) in branches.iter().enumerate() {
            if skip.contains(&bi) {
                continue;
            }
            let path: Vec<Vec3> = spline(pts).into_iter().map(|v| world(tr(v))).collect();
            let rr = r * scale * k * STUD;
            let ring: Vec<Vec2> = (0..10).map(|i| {
                let a = -(i as f32) / 10.0 * std::f32::consts::TAU;
                Vec2::new(a.cos() * rr, a.sin() * rr)
            }).collect();
            t.mb.part(&format!("c{hex:06x}")).sweep(&path, &ring, true);
            // the rounded, closed ends
            for end in [pts[0], pts[pts.len() - 1]] {
                let c = world(tr(end));
                let prof: Vec<Vec2> = (0..=6).map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 6.0 * std::f32::consts::PI;
                    Vec2::new(a.cos() * rr, a.sin() * rr)
                }).collect();
                t.mb.part(&format!("c{hex:06x}")).with(Mat4::from_translation(c), |b| {
                    b.lathe(&prof, 10);
                });
            }
        }
    };
    let top = PLATE_H * 2.0;
    piece(t, -1.0, -1.0, top, coral_green, 1.0, 0.0, &[]);
    piece(t, -3.0, 1.0, top, coral, 0.5, 0.6, &[]);
    piece(t, 3.0, -2.0, top, pink, 0.55, std::f32::consts::PI, &[]);
    piece(t, 1.0, 2.0, top + PLATE_H, gold, 0.42, -0.8, &[3]);
    piece(t, -3.0, -3.0, top + PLATE_H + BRICK_H * 0.5, aqua, 0.4, 2.2, &[4]);
    // (the base is solid; the coral's branches you swim through)
    let (lo, hi) = (pos(-6.0 * U, 0.0, -5.0 * U), pos(6.0 * U, PLATE_H * 2.0, 5.0 * U));
    t.solid(lo.x, hi.x, lo.y, hi.y, lo.z, hi.z);
}

/// The static cave (its entity is on [`CAVE_LAYER`]): the solids, triangles.
pub struct Cave {
    pub solids: Vec<CaveBox>,
    pub triangles: usize,
}

/// Builds the cave into the current scene (Kaigen `cave_build`).
pub fn build(ctx: &mut Ctx, assets: &mut TownAssets) -> Cave {
    let mut t = Build { mb: MeshBuilder::new(), solids: vec![], bricks: 0 };
    let (rock_a, rock_b, rock_c, sand_a, sand_b) = (0x3a4452, 0x2b3440, 0x4a5361, 0xb9a370, 0x8f8058);
    // ---- the floor: a column per cell, sand on the low middle, rock towards the walls ----
    let mut x = -CAVE_R;
    while x < CAVE_R - 0.1 {
        let mut z = -CAVE_R;
        while z < CAVE_R - 0.1 {
            let (cx, cz) = (x + CAVE_CELL * 0.5, z + CAVE_CELL * 0.5);
            let fh = floor(cx, cz);
            let deep = fh < 0.0;
            let c = if deep {
                if fh < -20.0 { 0x1f2733 } else { rock_b }
            } else if cx.abs().max(cz.abs()) > 28.0 {
                rock_c
            } else if hash(cx as i32, cz as i32) > 0.5 {
                sand_a
            } else {
                sand_b
            };
            t.bx(c, CAVE_CELL, fh + 30.0, CAVE_CELL, cx, -30.0, cz);
            if !deep && hash(cz as i32, cx as i32) > 0.7 {
                // a loose brick or two on the floor
                t.rock(if hash(cx as i32, 3) > 0.5 { rock_c } else { sand_b }, 2.0, 1.2, 4.0, cx + 1.0, fh, cz - 1.0, true);
            }
            z += CAVE_CELL;
        }
        x += CAVE_CELL;
    }
    // ---- the walls: banded rock, with ledges sticking out ----
    for side in 0..4 {
        for band in 0..10 {
            let y = -30.0 + band as f32 * 11.6;
            let c = if band % 2 == 1 { rock_a } else { rock_b };
            let len = 2.0 * CAVE_R + 12.0;
            let s = if side % 2 == 1 { 1.0 } else { -1.0 };
            if side == 1 && band < 2 {
                // the north wall's two lowest bands leave the tunnel's mouth open (x within TUN_W / 2, up to TUN_Y1)
                let hw = TUN_W * 0.5;
                let piece = (len * 0.5 - hw).max(1.0);
                t.bx(c, piece, 11.6, 6.0, -(hw + piece * 0.5), y, CAVE_R + 3.0);
                t.bx(c, piece, 11.6, 6.0, hw + piece * 0.5, y, CAVE_R + 3.0);
                if band == 1 {
                    t.bx(c, TUN_W, y + 11.6 - TUN_Y1, 6.0, 0.0, TUN_Y1, CAVE_R + 3.0);
                }
            } else if side < 2 {
                t.bx(c, len, 11.6, 6.0, 0.0, y, s * (CAVE_R + 3.0));
            } else {
                let s = if side == 3 { 1.0 } else { -1.0 };
                t.bx(c, 6.0, 11.6, len, s * (CAVE_R + 3.0), y, 0.0);
            }
        }
        for l in 0..7 {
            let along = -32.0 + l as f32 * 10.5 + hash(side, l) * 4.0;
            let y = 10.0 + hash(l, side) * 58.0;
            let out = 3.0 + hash(side * 7, l) * 3.0;
            let w = 6.0 + hash(l * 3, side) * 5.0;
            let (x, z, sx, sz) = if side < 2 {
                (along, if side == 1 { 1.0 } else { -1.0 } * (CAVE_R - out * 0.5), w, out)
            } else {
                (if side == 3 { 1.0 } else { -1.0 } * (CAVE_R - out * 0.5), along, out, w)
            };
            t.rock(if l % 2 == 1 { rock_c } else { rock_a }, sx, 2.0, sz, x, y, z, true);
            // something grows on most ledges
            if hash(l, side * 5) > 0.35 {
                t.coral(x, y + 2.0, z, CORALS[((side + l) % 4) as usize], 0.8 + hash(l, l) * 0.5);
            } else {
                t.sponge(x, y + 2.0, z, 2.0 + hash(side, l * 9) * 2.0, if l % 2 == 1 { 0x9a3a14 } else { 0xa35a1a });
            }
        }
    }
    // ---- the ceiling around the hole up to the pond, and the short shaft above it ----
    let (r, h) = (CAVE_R + 6.0, CAVE_HOLE);
    t.bx(rock_b, r - h, 6.0, 2.0 * r, -(r + h) * 0.5, CAVE_TOP, 0.0);
    t.bx(rock_b, r - h, 6.0, 2.0 * r, (r + h) * 0.5, CAVE_TOP, 0.0);
    t.bx(rock_a, 2.0 * h, 6.0, r - h, 0.0, CAVE_TOP, -(r + h) * 0.5);
    t.bx(rock_a, 2.0 * h, 6.0, r - h, 0.0, CAVE_TOP, (r + h) * 0.5);
    for k in 0..4 {
        // the shaft's walls, a brick wider than the hole
        let (sx, sz) = if k < 2 { (2.0 * h + 4.0, 2.0) } else { (2.0, 2.0 * h + 4.0) };
        let (ox, oz) = if k < 2 { (0.0, if k == 0 { -1.0 } else { 1.0 } * (h + 1.0)) } else { (if k == 2 { -1.0 } else { 1.0 } * (h + 1.0), 0.0) };
        t.bx(rock_c, sx, 14.0, sz, ox, CAVE_TOP + 6.0, oz);
    }
    // ---- rock pillars ----
    let pil: [[f32; 3]; 7] = [[-20.0, -8.0, 52.0], [-20.0, 8.0, 46.0], [22.0, 10.0, 60.0], [-6.0, -30.0, 38.0], [30.0, 30.0, 30.0], [8.0, 22.0, 44.0], [-30.0, 28.0, 56.0]];
    for (i, p) in pil.iter().enumerate() {
        let (px, pz, top) = (p[0], p[1], p[2]);
        let i = i as i32;
        let mut y = floor(px, pz);
        let mut w = 5.0f32;
        while y < top {
            let hh = 5.0 + hash(i, y as i32) * 4.0;
            let jx = (hash(y as i32, i) - 0.5) * 1.6;
            let jz = (hash(i * 3, y as i32) - 0.5) * 1.6;
            t.rock(if (y as i32 / 5) % 2 != 0 { rock_a } else { rock_c }, w, hh, w, px + jx, y, pz + jz, true);
            y += hh;
            w = (w - 0.4 + hash(i, hh as i32) * 0.8).max(3.0);
        }
    }
    t.rock(rock_c, 6.0, 4.0, 22.0, -20.0, 40.0, 0.0, true); // the arch over the kelp forest's way
    // ---- the grotto: an overhang in the south-east, its roof on two pillars ----
    t.rock(rock_b, 28.0, 4.0, 26.0, 26.0, 26.0, -27.0, true);
    t.rock(rock_a, 30.0, 2.0, 28.0, 26.0, 30.0, -27.0, false);
    for i in 0..2 {
        let px = 14.0 + i as f32 * 24.0;
        let f = floor(px, -16.0);
        t.rock(rock_c, 4.0, 26.0 - f, 4.0, px, f, -16.0, true);
    }
    // ---- the stone whale tail rising out of the sand ----
    {
        let (x, z) = (2.0, -18.0);
        let y = floor(x, z);
        let (stone, streak) = (0x9aa3a6, 0x6b7477);
        t.rock(stone, 3.0, 10.0, 3.0, x, y, z, true);
        t.bx(streak, 3.1, 0.6, 3.1, x, y + 3.0, z);
        t.rot(stone, Vec3::new(9.0, 1.2, 4.0), Vec3::new(x - 4.0, y + 12.0, z), Vec3::new(0.0, 0.0, 0.45));
        t.rot(stone, Vec3::new(9.0, 1.2, 4.0), Vec3::new(x + 4.0, y + 12.0, z), Vec3::new(0.0, 0.0, -0.45));
        t.solid(x - 8.0, x + 8.0, y + 9.0, y + 15.0, z - 2.0, z + 2.0);
    }
    // ---- a sunken boat of brown bricks, on its side ----
    {
        let (x, z) = (-26.0, 18.0);
        let y = floor(x, z);
        let (wood, dark) = (0x5c3a1e, 0x3a2412);
        t.rot(wood, Vec3::new(14.0, 1.0, 6.0), Vec3::new(x, y + 1.0, z), Vec3::new(0.0, 0.3, 0.25));
        t.rot(wood, Vec3::new(14.0, 4.0, 1.0), Vec3::new(x - 0.6, y + 3.0, z + 3.0), Vec3::new(0.0, 0.3, 0.25));
        t.rot(dark, Vec3::new(14.0, 3.0, 1.0), Vec3::new(x + 0.6, y + 2.0, z - 3.0), Vec3::new(0.0, 0.3, 0.25));
        t.rot(dark, Vec3::new(0.8, 14.0, 0.8), Vec3::new(x + 2.0, y + 7.0, z), Vec3::new(0.0, 0.0, 0.5)); // the mast
        t.solid(x - 7.0, x + 7.0, y, y + 5.0, z - 3.5, z + 3.5);
    }
    // ---- sponges and coral on the floor ----
    for i in 0..18 {
        let x = -34.0 + hash(i, 1) * 68.0;
        let z = -34.0 + hash(i, 2) * 68.0;
        if x.abs() < 13.0 && z > 13.0 {
            continue; // not in the trench
        }
        let y = floor(x, z);
        if i % 3 == 0 {
            t.sponge(x, y, z, 2.0 + hash(i, 3) * 3.0, if i % 2 == 1 { 0x9a3a14 } else { 0xa35a1a });
        } else {
            t.coral(x, y, z, CORALS[(i % 4) as usize], 0.9 + hash(i, 4) * 0.6);
        }
    }
    // ---- the tunnel: from the bottom of the trench north through the wall, a low passage of dark rock ----
    {
        let (z0, z1) = (CAVE_R, DEEP_Z0);
        let (zc, d) = ((z0 + z1) * 0.5, z1 - z0);
        let hw = TUN_W * 0.5;
        let dark = 0x1f2733;
        t.bx(dark, TUN_W + 4.0, 2.0, d, 0.0, TUN_Y0 - 2.0, zc); // floor
        t.bx(rock_b, TUN_W + 4.0, 2.0, d, 0.0, TUN_Y1, zc); // roof
        for sx in [-1.0, 1.0] {
            t.bx(if sx < 0.0 { rock_a } else { rock_c }, 2.0, TUN_Y1 - TUN_Y0, d, sx * (hw + 1.0), TUN_Y0, zc);
        }
        // ribs every 8 studs, so the way reads as a passage (and the lamp catches them)
        let mut z = z0 + 8.0;
        while z < z1 - 2.0 {
            t.bx(rock_c, TUN_W + 1.0, 1.0, 1.2, 0.0, TUN_Y1 - 1.0, z);
            z += 8.0;
        }
    }
    // ---- the deep grotto: a darker room past the tunnel, the Stud Core at its far end ----
    {
        let (hw, z0, z1) = (DEEP_W * 0.5, DEEP_Z0, DEEP_Z1);
        let (zc, d) = ((z0 + z1) * 0.5, z1 - z0);
        let h = DEEP_Y1 - DEEP_Y0;
        let (wall_a, wall_b, deep_floor) = (0x232b38, 0x2e3846, 0x1a212b);
        t.bx(deep_floor, DEEP_W + 6.0, 4.0, d + 6.0, 0.0, DEEP_Y0 - 4.0, zc);
        t.bx(wall_b, DEEP_W + 6.0, 3.0, d + 6.0, 0.0, DEEP_Y1, zc); // the roof
        for sx in [-1.0, 1.0] {
            t.bx(if sx < 0.0 { wall_a } else { wall_b }, 3.0, h, d + 6.0, sx * (hw + 1.5), DEEP_Y0, zc);
        }
        t.bx(wall_a, DEEP_W + 6.0, h, 3.0, 0.0, DEEP_Y0, z1 + 1.5); // the far wall
        // the near wall, around the tunnel's end
        let tw = TUN_W * 0.5;
        let piece = hw + 3.0 - tw;
        for sx in [-1.0, 1.0] {
            t.bx(wall_b, piece, h, 3.0, sx * (tw + piece * 0.5), DEEP_Y0, z0 - 1.5);
        }
        t.bx(wall_a, TUN_W, TUN_Y0 - DEEP_Y0, 3.0, 0.0, DEEP_Y0, z0 - 1.5);
        t.bx(wall_a, TUN_W, DEEP_Y1 - TUN_Y1, 3.0, 0.0, TUN_Y1, z0 - 1.5);
        // a ramp of steps down from the tunnel's floor to the grotto's
        for k in 0..4 {
            let y = TUN_Y0 - 2.5 * (k as f32 + 1.0);
            t.rock(0x2b3440, TUN_W + 2.0, 2.5, 3.0, 0.0, y, z0 + 1.5 + k as f32 * 3.0, true);
        }
        // crystals (pale violet and cyan, lit by the grotto's lights), rocks and sponges on the floor
        for i in 0..14 {
            let x = -hw + 3.0 + hash(i, 21) * (DEEP_W - 6.0);
            let z = z0 + 10.0 + hash(i, 22) * (d - 16.0);
            if x.abs() < 6.0 && z > z1 - 14.0 {
                continue; // the way to the core stays clear
            }
            let hh = 3.0 + hash(i, 23) * 7.0;
            let col = if i % 2 == 0 { 0xb58cff } else { 0x7fe6ff };
            t.rot(col, Vec3::new(1.4, hh, 1.4), Vec3::new(x, DEEP_Y0 + hh * 0.45, z), Vec3::new((hash(i, 24) - 0.5) * 0.6, hash(i, 25) * 3.0, (hash(i, 26) - 0.5) * 0.6));
            if i % 3 == 0 {
                t.rock(wall_b, 4.0, 2.0, 3.0, x + 2.5, DEEP_Y0, z - 1.0, true);
            }
            if i % 4 == 1 {
                t.coral(x - 2.0, DEEP_Y0, z + 2.0, CORALS[(i % 4) as usize], 0.9);
            }
        }
        // the warp pipe at the far end, where the core's column stood
        pipe(&mut t, PIPE_AT.x, DEEP_Y0, PIPE_TOP, PIPE_AT.z);
    }
    // ---- the core room: a small cave of warm rock below and beyond the grotto, the way in its own pipe ----
    {
        let (hw, z0, z1) = (CR_W * 0.5, CR_Z0, CR_Z1);
        let (zc, d) = ((z0 + z1) * 0.5, z1 - z0);
        let h = CR_Y1 - CR_Y0;
        let (wall_a, wall_b, floor_c) = (0x4a3a2e, 0x5c4636, 0x3a2c22);
        t.bx(floor_c, CR_W + 6.0, 4.0, d + 6.0, 0.0, CR_Y0 - 4.0, zc);
        t.bx(wall_b, CR_W + 6.0, 3.0, d + 6.0, 0.0, CR_Y1, zc);
        for sx in [-1.0, 1.0] {
            t.bx(if sx < 0.0 { wall_a } else { wall_b }, 3.0, h, d + 6.0, sx * (hw + 1.5), CR_Y0, zc);
        }
        t.bx(wall_a, CR_W + 6.0, h, 3.0, 0.0, CR_Y0, z0 - 1.5);
        t.bx(wall_b, CR_W + 6.0, h, 3.0, 0.0, CR_Y0, z1 + 1.5);
        // banded ledges on the walls, gold crystals, a little coral
        for i in 0..6 {
            let sx = if i % 2 == 0 { -1.0 } else { 1.0 };
            let z = z0 + 3.0 + i as f32 * 3.6;
            t.rock(if i % 3 == 0 { wall_a } else { 0x6e5440 }, 2.5, 1.5, 3.0, sx * (hw - 1.25), CR_Y0 + 4.0 + hash(i, 31) * 9.0, z, true);
        }
        for i in 0..10 {
            let x = -hw + 2.0 + hash(i, 41) * (CR_W - 4.0);
            let z = z0 + 2.0 + hash(i, 42) * (d - 4.0);
            if x.abs() < 6.0 {
                continue; // the way from the pipe to the core stays clear
            }
            let hh = 2.0 + hash(i, 43) * 5.0;
            let col = if i % 2 == 0 { 0xffd23d } else { 0xffa63d };
            t.rot(col, Vec3::new(1.2, hh, 1.2), Vec3::new(x, CR_Y0 + hh * 0.45, z), Vec3::new((hash(i, 44) - 0.5) * 0.6, hash(i, 45) * 3.0, (hash(i, 46) - 0.5) * 0.6));
        }
        t.coral(-7.0, CR_Y0, z1 - 4.0, CORALS[1], 0.9);
        t.coral(7.5, CR_Y0, z1 - 6.0, CORALS[2], 0.8);
        // the core's rock: a low cushion of stone where it lies
        t.rock(0x6a7582, 6.0, 1.5, 6.0, DEEP_CORE.x, CR_Y0, DEEP_CORE.z, true);
        t.rock(0x8a95a2, 4.0, 1.0, 4.0, DEEP_CORE.x, CR_Y0 + 1.5, DEEP_CORE.z, true);
        // the pipe you came out of
        pipe(&mut t, PIPE2_AT.x, CR_Y0, PIPE2_TOP, PIPE2_AT.z);
    }
    // ---- the LEGO coral (the user's model) on the sand east of the hole, clear of the kelp ----
    {
        let (x, z) = (16.0, 2.0);
        lego_coral(&mut t, Vec3::new(x, floor(x, z), z), 2.2);
    }

    let triangles = t.mb.triangle_count();
    let parts = t.mb.finish();
    let mats: HashMap<String, MaterialId> = parts.iter().map(|(p, _)| (p.clone(), assets.material(ctx, u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff)))).collect();
    let root = ctx.spawn_parts("pond cave", &parts, |p| mats[p], Transform::default());
    ctx.set_layer(root, CAVE_LAYER);
    for d in engine::scene::descendants(&ctx.world, root) {
        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(d) {
            mr.cast_shadows = false;
        }
    }
    Cave { solids: t.solids, triangles }
}
