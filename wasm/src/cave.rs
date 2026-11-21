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
pub const DEEP_CORE: Vec3 = Vec3::new(0.0, -19.6, 110.0);

/// Where the diver is (dive.rs collision, the debug state).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Zone {
    Basin,
    Tunnel,
    Deep,
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
        // the Stud Core's column at the far end, at the tunnel's height
        let (cx, cz) = (DEEP_CORE.x, DEEP_CORE.z);
        t.rock(0x55606c, 6.0, 3.0, 6.0, cx, DEEP_Y0, cz, true);
        t.rock(0x4a5361, 4.0, DEEP_CORE.y - 4.4 - DEEP_Y0, 4.0, cx, DEEP_Y0 + 3.0, cz, true);
        t.rock(0x6a7582, 5.0, 2.0, 5.0, cx, DEEP_CORE.y - 3.4, cz, true);
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
