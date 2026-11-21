//! The island north of Stonecrest (OPEN_WORLD.md, milestone 1): its "height law". One pure function
//! ([`natural`]) gives the natural height, cliffiness, forest density, climate and coast distance for any
//! (x, z) in studs; [`Island::bake`] samples it once into a coarse lattice (`CELL` studs per cell), and the
//! fine height of a stud ([`Island::cell`]) is the lattice interpolated, terraced into cliffs, blended into
//! the flat site pads and the graded main road, and rounded to whole plates (LEGO steps).
//!
//! Units are studs like the rest of the town (x east, z south, y up); heights are counted in plates
//! (`PLATE` = 0.4 studs). The island lies north of the north plate's edge (`SOUTH_Z`); Brickhollow, Route 1
//! and Stonecrest are its south tip. The chunks, studs, colliders and the sea are `islandview.rs`, the map
//! `worldmap.rs`.

use engine::prelude::*;

/// A plate's height (studs): the step of every island height.
pub const PLATE: f32 = 0.4;
/// The sea's surface (studs; between two plate steps, so no top lies in its plane).
pub const SEA_Y: f32 = -1.4;
/// The island begins north of the north plate (the strip's ground ends at z -194).
pub const SOUTH_Z: f32 = -194.0;
/// The lattice: x X0..X1, z Z0..Z1, `CELL` studs per cell. It reaches south past the strip (Brickhollow, Route 1,
/// Stonecrest): the land round the strip's plates and the sea beyond (the plates themselves are holes, `hole`).
pub const X0: f32 = -704.0;
pub const X1: f32 = 704.0;
pub const Z0: f32 = -1538.0;
pub const Z1: f32 = 254.0;
pub const CELL: f32 = 4.0;
pub const NX: usize = ((X1 - X0) / CELL) as usize;
pub const NZ: usize = ((Z1 - Z0) / CELL) as usize;
/// Chunks (studs per side) and their count.
pub const CHUNK: f32 = 64.0;
pub const CX: i32 = ((X1 - X0) / CHUNK) as i32;
pub const CZ: i32 = ((Z1 - Z0) / CHUNK) as i32;
/// The deep sea floor, the bottom every chunk's skirt reaches, and the deepest top you can wade on (plates).
pub const DEEP: i32 = -15;
pub const FLOOR: i32 = -20;
pub const WADE_MIN: i32 = -6;
/// Render layers: the chunks near you, the chunks further out (drawn only while the camera looks far), the sea.
pub const LAYER_NEAR: u8 = 11;
pub const LAYER_FAR: u8 = 12;
pub const LAYER_SEA: u8 = 13;
/// The north gate in Stonecrest's north wall (studs): centre x, the wall's z, half its opening.
pub const GATE_X: f32 = 35.0;
pub const GATE_Z: f32 = -190.5;
pub const GATE_HALF: f32 = 4.0;

// ---- the palette (sRGB): every top / side / stud colour of the island ----
pub const GRASS: u8 = 0;
pub const GRASS_LIGHT: u8 = 1;
pub const GRASS_LIME: u8 = 2;
pub const GRASS_DARK: u8 = 3;
pub const FOREST: u8 = 4;
pub const COOL_GRASS: u8 = 5;
pub const SAND: u8 = 6;
pub const SAND_DARK: u8 = 7;
pub const WET_SAND: u8 = 8;
pub const TRAIL: u8 = 9;
pub const TRAIL_EDGE: u8 = 10;
pub const ROCK_LIGHT: u8 = 11;
pub const ROCK_MID: u8 = 12;
pub const ROCK_DARK: u8 = 13;
pub const SNOW: u8 = 14;
pub const SNOW_SHADE: u8 = 15;
pub const SEA_FLOOR: u8 = 16;
pub const DEEP_FLOOR: u8 = 17;
pub const PAD_CITY: u8 = 18;
pub const PAD_HARBOUR: u8 = 19;
pub const PAD_VILLAGE: u8 = 20;
pub const SANDSTONE: u8 = 21;
pub const SANDSTONE_DARK: u8 = 22;
pub const DIRT: u8 = 23;
pub const PALETTE: [u32; 24] = [
    0x4b9f4a, 0x58ab41, 0x7fc04a, 0x3c8c3a, 0x237841, 0x6b9a73, 0xe4cd9e, 0xd2b67f, 0xbfa071, 0xc8a46a, 0xa07a4a, 0xa0a5a9, 0x878b8a, 0x6c6e68, 0xf4f4f4,
    0xdde6ec, 0xa89a78, 0x2e5e74, 0xc4c8cb, 0xd9bb7b, 0x8cc65a, 0xd9a066, 0xa86f3c, 0x8a5a32,
];
/// ±3 % brightness steps per brick (`palette entry = colour * JITTER.len() + step`).
pub const JITTER: [f32; 5] = [0.97, 0.985, 1.0, 1.015, 1.03];

/// A flat site pad for a later place (milestone 2+): name, kind, centre and half size (studs).
pub struct Pad {
    pub name: &'static str,
    pub kind: &'static str,
    pub x: f32,
    pub z: f32,
    pub hx: f32,
    pub hz: f32,
    pub colour: u8,
}

pub const PADS: [Pad; 3] = [
    Pad { name: "Brickton", kind: "city", x: -80.0, z: -820.0, hx: 90.0, hz: 75.0, colour: PAD_CITY },
    Pad { name: "Port Saltwick", kind: "harbour", x: 470.0, z: -620.0, hx: 40.0, hz: 36.0, colour: PAD_HARBOUR },
    Pad { name: "Dewmere", kind: "village", x: -300.0, z: -470.0, hx: 50.0, hz: 45.0, colour: PAD_VILLAGE },
];
/// The soft ring around a pad where the land eases to its height (studs).
const PAD_RING: f32 = 28.0;

/// The main road: the trunk from the north gate to the city, branches to the village and the harbour
/// (control points, studs). A branch starts on the trunk (its first point lies on it).
pub const ROADS: [&[[f32; 2]]; 3] = [
    &[[GATE_X, SOUTH_Z], [GATE_X, -235.0], [15.0, -290.0], [5.0, -360.0], [-20.0, -440.0], [-40.0, -540.0], [-60.0, -640.0], [-80.0, -745.0]],
    &[[5.0, -360.0], [-80.0, -390.0], [-170.0, -430.0], [-250.0, -465.0]],
    &[[-40.0, -540.0], [60.0, -560.0], [170.0, -585.0], [280.0, -610.0], [380.0, -618.0], [430.0, -620.0]],
];
/// Where each road ends (index into `PADS`).
const ROAD_TO: [usize; 3] = [0, 2, 1];
/// The road: its walkable half width, the edge band, the valley the land eases into it over (studs).
pub const ROAD_HALF: f32 = 3.0;
pub const ROAD_EDGE: f32 = 4.0;
const ROAD_VALLEY: f32 = 16.0;
/// The steepest the graded road rises (studs per stud): below a plate per stud, so every step is walkable.
const ROAD_SLOPE: f32 = 0.25;

/// The island's capsules (a, b, radius): the land is their smooth union, warped by noise.
const CAPSULES: [[f32; 5]; 10] = [
    [35.0, -260.0, 35.0, -300.0, 150.0],
    [35.0, -300.0, -40.0, -700.0, 190.0],
    [35.0, -260.0, -300.0, -430.0, 170.0],
    [-300.0, -430.0, -80.0, -820.0, 220.0],
    [35.0, -260.0, 350.0, -560.0, 160.0],
    [350.0, -560.0, -80.0, -820.0, 200.0],
    [-80.0, -820.0, -250.0, -1230.0, 220.0],
    [-80.0, -820.0, 250.0, -1180.0, 220.0],
    [-250.0, -1230.0, 250.0, -1180.0, 180.0],
    [350.0, -560.0, 450.0, -640.0, 110.0],
];
/// Bays cut out of the coast (centre, radius).
const BAYS: [[f32; 3]; 2] = [[-500.0, -660.0, 120.0], [360.0, -960.0, 110.0]];

// ------------------------------------------------------------------------- noise (pure, seeded)

fn hash(ix: i32, iz: i32, seed: u32) -> u32 {
    let mut h = (ix as u32).wrapping_mul(0x27d4_eb2d) ^ (iz as u32).wrapping_mul(0x1656_67b1) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

/// 0..1 from integer coordinates (per brick jitter, patches).
pub fn hash01(ix: i32, iz: i32, seed: u32) -> f32 {
    (hash(ix, iz, seed) >> 8) as f32 / 16_777_216.0
}

/// Smooth value noise, -1..1.
fn vnoise(x: f32, z: f32, seed: u32) -> f32 {
    let (fx, fz) = (x.floor(), z.floor());
    let (ix, iz) = (fx as i32, fz as i32);
    let (tx, tz) = (x - fx, z - fz);
    let (u, v) = (tx * tx * (3.0 - 2.0 * tx), tz * tz * (3.0 - 2.0 * tz));
    let c = |a: i32, b: i32| hash01(ix + a, iz + b, seed) * 2.0 - 1.0;
    let top = c(0, 0) + (c(1, 0) - c(0, 0)) * u;
    let bot = c(0, 1) + (c(1, 1) - c(0, 1)) * u;
    top + (bot - top) * v
}

/// Fractal value noise, about -1..1.
fn fbm(x: f32, z: f32, oct: u32, seed: u32) -> f32 {
    let (mut s, mut a, mut f, mut n) = (0.0, 1.0, 1.0, 0.0);
    for i in 0..oct {
        s += vnoise(x * f + i as f32 * 17.3, z * f - i as f32 * 9.1, seed + i) * a;
        n += a;
        a *= 0.5;
        f *= 2.03;
    }
    s / n * 1.6
}

/// Ridged noise, 0..1 (sharp crests: the mountains).
fn ridged(x: f32, z: f32, oct: u32, seed: u32) -> f32 {
    let (mut s, mut a, mut f, mut n) = (0.0, 1.0, 1.0, 0.0);
    for i in 0..oct {
        let r = 1.0 - vnoise(x * f + i as f32 * 5.7, z * f + i as f32 * 3.3, seed + i).abs();
        s += r * r * a;
        n += a;
        a *= 0.5;
        f *= 2.1;
    }
    s / n
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

fn sd_capsule(p: Vec2, a: Vec2, b: Vec2, r: f32) -> f32 {
    let (pa, ba) = (p - a, b - a);
    let h = (pa.dot(ba) / ba.dot(ba).max(1e-6)).clamp(0.0, 1.0);
    (pa - ba * h).length() - r
}

/// Signed distance to a rectangle (centre, half size): < 0 inside.
fn sd_box(p: Vec2, c: Vec2, half: Vec2) -> f32 {
    let q = (p - c).abs() - half;
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
}

// ------------------------------------------------------------------------- the law

/// The strip's plates (village; Route 1 and Stonecrest) as a signed distance (studs; < 0 on them).
pub fn strip_sd(x: f32, z: f32) -> f32 {
    let p = Vec2::new(x, z);
    sd_box(p, Vec2::new(20.0, -9.0), Vec2::new(52.0, 38.0)).min(sd_box(p, Vec2::new(5.0, -120.5), Vec2::new(51.0, 73.5)))
}

/// How far the land reaches round the strip's plates (studs, before the coast's noise).
const STRIP_LAND: f32 = 55.0;

/// Where the island has no ground of its own: the strip's plates (their own ground lies there) and the rooms far
/// east of the village (interiors on their own layers).
pub fn hole(x: f32, z: f32) -> bool {
    strip_sd(x, z) <= 0.0 || (x >= 140.0 && z > -150.0)
}

/// The coast as a signed distance (studs; < 0 on land): capsules between the regions' centres, smoothly
/// united, warped by noise, bays cut out; the strip's table south of the island is always land.
pub fn coast(x: f32, z: f32) -> f32 {
    let w = Vec2::new(x + fbm(x / 220.0, z / 220.0, 3, 11) * 45.0, z + fbm(x / 220.0 + 5.3, z / 220.0, 3, 12) * 45.0);
    let mut d = f32::MAX;
    for c in CAPSULES {
        let s = sd_capsule(w, Vec2::new(c[0], c[1]), Vec2::new(c[2], c[3]), c[4]);
        d = if d == f32::MAX { s } else { smin(d, s, 70.0) };
    }
    for b in BAYS {
        let cut = b[2] - (w - Vec2::new(b[0], b[1])).length();
        d = -smin(-d, -cut, 30.0);
    }
    d += fbm(x / 60.0, z / 60.0, 3, 13) * 9.0;
    // the south coast curves north away from the strip (the land reaches the island's edge only where the strip is)
    let zline = SOUTH_Z + 30.0 - (((x - 20.0).abs() - 215.0).max(0.0) * 0.9) + fbm(x / 70.0, 3.7, 2, 14) * 10.0;
    d = -smin(-d, -(z - zline) * 0.9, 25.0);
    // the strip's table (x -180..220) ends at the island's south edge: land there, so it never shows
    let d = smin(d, sd_box(Vec2::new(x, z), Vec2::new(20.0, -200.0), Vec2::new(215.0, 50.0)), 30.0);
    // and land round the strip's plates (meadows, then a beach), so the village never ends in the void
    smin(d, strip_sd(x, z) - STRIP_LAND + fbm(x / 45.0, z / 45.0, 3, 15) * 14.0, 30.0)
}

/// How far north (0 in the warm south .. 1 in the cold north).
pub fn north(z: f32) -> f32 {
    smoothstep(-800.0, -1200.0, z)
}

/// Terrace step (studs) of the cliffs: 2 bricks in the middle (a jump gets you up), 3 in the north.
fn terrace_step(z: f32) -> f32 {
    2.4 + 1.2 * north(z)
}

/// The natural land at (x, z) before terraces, pads and roads.
#[derive(Clone, Copy, Debug, Default)]
pub struct Nat {
    /// height (studs), cliffiness 0..1, forest density 0..1, warmth 0 (north) .. 1 (south), coast distance
    pub h: f32,
    pub cliff: f32,
    pub forest: f32,
    pub warm: f32,
    pub coast: f32,
}

/// The height law (pure).
pub fn natural(x: f32, z: f32) -> Nat {
    let d = coast(x, z);
    let n = north(z);
    let warm = (smoothstep(-1300.0, -500.0, z) + fbm(x / 300.0, z / 300.0, 2, 91) * 0.1).clamp(0.0, 1.0);
    if d > 0.0 {
        // the sea: a shallow shelf you can wade on, then the drop into the deep
        let dd = d + fbm(x / 40.0, z / 40.0, 2, 21) * 4.0;
        let h = if dd < 6.0 {
            -2.0
        } else if dd < 13.0 {
            -2.4
        } else if dd < 17.0 {
            -3.6
        } else {
            -6.0
        };
        return Nat { h, cliff: 0.0, forest: 0.0, warm, coast: d };
    }
    let inland = -d;
    let rise = smoothstep(0.0, 90.0, inland);
    let hills = fbm(x / 150.0, z / 150.0, 4, 31);
    let lumps = fbm(x / 55.0, z / 55.0, 2, 32);
    let mut h = -1.0 + rise * (4.0 + 6.0 * hills + 1.5 * lumps);
    // the north: mountains on ridges, a rocky coast that rises at once
    let ridge = ridged(x / 170.0, z / 170.0, 4, 41);
    let mtn = n * smoothstep(30.0, 160.0, inland) * ridge * ridge * 55.0;
    let rocky = n * smoothstep(0.0, 10.0, inland) * 4.0;
    // the middle: flat-topped mesas with cliffs
    let mesa = smoothstep(0.25, 0.4, fbm(x / 230.0 + 3.1, z / 230.0 - 1.7, 3, 51)) * smoothstep(40.0, 120.0, inland) * (1.0 - n);
    h += mtn + rocky + mesa * 7.0;
    let cliff = smoothstep(0.08, 0.3, n * ridge * smoothstep(20.0, 60.0, inland)).max(mesa).max(n * smoothstep(0.0, 4.0, inland) * 0.8);
    let forest = smoothstep(0.1, 0.4, fbm(x / 120.0, z / 120.0, 3, 61)) * smoothstep(15.0, 40.0, inland) * (1.0 - (mtn / 25.0).min(1.0));
    Nat { h: h.max(-1.2), cliff, forest, warm, coast: d }
}

/// Cliffs: the height snapped to terraces of `step` studs (a narrow ramp at each riser, which the plates turn
/// into a wall), mixed in by the cliffiness.
fn terrace(h: f32, c: f32, step: f32) -> f32 {
    if c <= 0.01 {
        return h;
    }
    let f = (h / step).floor() * step;
    let t = f + step * smoothstep(0.9, 1.0, (h - f) / step);
    h + (t - h) * c
}

/// What a fine stud of the island is: its height (plates), top colour, and whether it is road / pad.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub plates: i32,
    pub colour: u8,
    pub warm: bool,
    /// 0 none, 1 the road, 2 its edge
    pub road: u8,
    /// the pad it lies on (index into `PADS`)
    pub pad: Option<u8>,
    /// no ground here (`hole`): no top, no sides, no collider
    pub hole: bool,
}

/// One graded road: its points, the distance along it at each point, the height every stud along it.
struct Road {
    pts: Vec<Vec2>,
    cum: Vec<f32>,
    prof: Vec<f32>,
    bbox: (Vec2, Vec2),
}

/// The self-tests of the bake (debug builds; `game_state.state.island.selftest`).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SelfTest {
    pub ok: bool,
    pub problems: Vec<String>,
    pub road_min_y: f32,
    pub road_max_step: i32,
}

/// The baked island: the lattice of the natural land, the pads' heights, the graded roads.
pub struct Island {
    /// per lattice point ((NX + 1) x (NZ + 1), row by row from z0): height, coast distance
    h: Vec<f32>,
    coast: Vec<f32>,
    /// cliffiness, forest, warmth (0..255)
    cliff: Vec<u8>,
    forest: Vec<u8>,
    warm: Vec<u8>,
    pub pad_h: [f32; 3],
    roads: Vec<Road>,
    pub bake_ms: f64,
    pub selftest: SelfTest,
}

impl Island {
    /// Samples the law into the lattice, sets the pads' heights and grades the roads (once, at the start).
    pub fn bake() -> Self {
        let t0 = engine::log::now_ms();
        let n = (NX + 1) * (NZ + 1);
        let mut me = Self { h: Vec::with_capacity(n), coast: Vec::with_capacity(n), cliff: Vec::with_capacity(n), forest: Vec::with_capacity(n), warm: Vec::with_capacity(n), pad_h: [0.0; 3], roads: vec![], bake_ms: 0.0, selftest: SelfTest::default() };
        for k in 0..=NZ {
            let z = Z0 + k as f32 * CELL;
            for i in 0..=NX {
                let s = natural(X0 + i as f32 * CELL, z);
                me.h.push(s.h);
                me.coast.push(s.coast);
                me.cliff.push((s.cliff * 255.0) as u8);
                me.forest.push((s.forest * 255.0) as u8);
                me.warm.push((s.warm * 255.0) as u8);
            }
        }
        for (i, p) in PADS.iter().enumerate() {
            me.pad_h[i] = ((me.lerp(p.x, p.z).h / PLATE).round() * PLATE).max(0.8);
        }
        // the roads: the land (with the pads) along them, smoothed, graded to a walkable slope between fixed ends
        for (ri, pts) in ROADS.iter().enumerate() {
            let pts: Vec<Vec2> = pts.iter().map(|p| Vec2::new(p[0], p[1])).collect();
            let mut cum = vec![0.0];
            for w in pts.windows(2) {
                cum.push(cum.last().unwrap() + (w[1] - w[0]).length());
            }
            let len = *cum.last().unwrap();
            let n = len.ceil() as usize + 1;
            let at = |s: f32| -> Vec2 {
                let j = cum.iter().rposition(|&c| c <= s).unwrap_or(0).min(pts.len() - 2);
                let t = ((s - cum[j]) / (cum[j + 1] - cum[j]).max(1e-6)).clamp(0.0, 1.0);
                pts[j] + (pts[j + 1] - pts[j]) * t
            };
            let ground: Vec<f32> = (0..n).map(|i| {
                let p = at(i as f32);
                me.base(p.x, p.y)
            }).collect();
            let mut prof: Vec<f32> = (0..n).map(|i| {
                let (a, b) = (i.saturating_sub(6), (i + 6).min(n - 1));
                ground[a..=b].iter().sum::<f32>() / (b - a + 1) as f32
            }).collect();
            let start = if ri == 0 { 0.0 } else { me.road_height(pts[0]).unwrap_or(prof[0]) };
            let end = me.pad_h[ROAD_TO[ri]];
            prof[0] = start;
            prof[n - 1] = end;
            for i in 1..n {
                prof[i] = prof[i].clamp(prof[i - 1] - ROAD_SLOPE, prof[i - 1] + ROAD_SLOPE);
            }
            prof[n - 1] = end;
            for i in (0..n - 1).rev() {
                prof[i] = prof[i].clamp(prof[i + 1] - ROAD_SLOPE, prof[i + 1] + ROAD_SLOPE);
            }
            let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
            for p in &pts {
                lo = lo.min(*p);
                hi = hi.max(*p);
            }
            me.roads.push(Road { pts, cum, prof, bbox: (lo - Vec2::splat(ROAD_VALLEY), hi + Vec2::splat(ROAD_VALLEY)) });
        }
        me.bake_ms = engine::log::now_ms() - t0;
        if cfg!(debug_assertions) {
            me.selftest = me.self_test();
            if !me.selftest.ok {
                log_warn!("island self-test: {:?}", me.selftest.problems);
            }
        }
        log_info!("island: {}x{} lattice baked in {:.1} ms", NX + 1, NZ + 1, me.bake_ms);
        me
    }

    /// The lattice at (x, z), bilinear (outside: the deep sea).
    pub fn lerp(&self, x: f32, z: f32) -> Nat {
        let fx = (x - X0) / CELL;
        let fz = (z - Z0) / CELL;
        if fx < 0.0 || fz < 0.0 || fx >= NX as f32 || fz >= NZ as f32 {
            let inside = fx >= -0.01 && fz >= -0.01 && fx <= NX as f32 + 0.01 && fz <= NZ as f32 + 0.01;
            if !inside {
                return Nat { h: -6.0, cliff: 0.0, forest: 0.0, warm: 0.5, coast: 999.0 };
            }
        }
        let (fx, fz) = (fx.clamp(0.0, NX as f32 - 1e-3), fz.clamp(0.0, NZ as f32 - 1e-3));
        let (i, k) = (fx as usize, fz as usize);
        let (tx, tz) = (fx - i as f32, fz - k as f32);
        let a = k * (NX + 1) + i;
        let (b, c, d) = (a + 1, a + NX + 1, a + NX + 2);
        let bl = |v: &[f32]| {
            let top = v[a] + (v[b] - v[a]) * tx;
            let bot = v[c] + (v[d] - v[c]) * tx;
            top + (bot - top) * tz
        };
        let bu = |v: &[u8]| {
            let f = |j: usize| v[j] as f32 / 255.0;
            let top = f(a) + (f(b) - f(a)) * tx;
            let bot = f(c) + (f(d) - f(c)) * tx;
            top + (bot - top) * tz
        };
        Nat { h: bl(&self.h), coast: bl(&self.coast), cliff: bu(&self.cliff), forest: bu(&self.forest), warm: bu(&self.warm) }
    }

    /// The land with its terraces and the pads, before the road (studs), and the pad weight / index.
    fn base_pads(&self, x: f32, z: f32, n: &Nat) -> (f32, Option<u8>) {
        let mut h = terrace(n.h, n.cliff, terrace_step(z));
        let mut on = None;
        let p = Vec2::new(x, z);
        for (i, pad) in PADS.iter().enumerate() {
            let d = sd_box(p, Vec2::new(pad.x, pad.z), Vec2::new(pad.hx, pad.hz));
            if d < PAD_RING {
                let w = 1.0 - smoothstep(0.0, PAD_RING, d);
                h += (self.pad_h[i] - h) * w;
                if d <= 0.0 {
                    on = Some(i as u8);
                }
            }
        }
        (h, on)
    }

    fn base(&self, x: f32, z: f32) -> f32 {
        let n = self.lerp(x, z);
        self.base_pads(x, z, &n).0
    }

    /// The nearest road within its valley: (distance, graded height there).
    fn road_at(&self, p: Vec2) -> Option<(f32, f32)> {
        let mut best: Option<(f32, f32)> = None;
        for r in &self.roads {
            if p.x < r.bbox.0.x || p.y < r.bbox.0.y || p.x > r.bbox.1.x || p.y > r.bbox.1.y {
                continue;
            }
            for j in 0..r.pts.len() - 1 {
                let (a, b) = (r.pts[j], r.pts[j + 1]);
                let ab = b - a;
                let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
                let d = (p - (a + ab * t)).length();
                if d < ROAD_VALLEY && best.is_none_or(|b| d < b.0) {
                    let s = r.cum[j] + ab.length() * t;
                    let i = (s.floor() as usize).min(r.prof.len() - 1);
                    let i2 = (i + 1).min(r.prof.len() - 1);
                    let f = s - i as f32;
                    best = Some((d, r.prof[i] + (r.prof[i2] - r.prof[i]) * f));
                }
            }
        }
        best
    }

    fn road_height(&self, p: Vec2) -> Option<f32> {
        self.road_at(p).filter(|r| r.0 < 1.0).map(|r| r.1)
    }

    /// A stud of the island (its cell from x..x+1, z..z+1 sampled at (x, z) = the cell's centre).
    pub fn cell(&self, x: f32, z: f32) -> Cell {
        let n = self.lerp(x, z);
        let (mut h, pad) = self.base_pads(x, z, &n);
        let mut road = 0;
        if let Some((d, rh)) = self.road_at(Vec2::new(x, z)) {
            let w = 1.0 - smoothstep(ROAD_EDGE, ROAD_VALLEY, d);
            h += (rh - h) * w;
            road = if d < ROAD_HALF { 1 } else if d < ROAD_EDGE { 2 } else { 0 };
        }
        // the edge to the strip: the road meets the plate's ground at 0, the land beside the plates lies a plate below
        // them (a plate never shares a plane with the land) and rises to low hills further out; the sea stays sea
        if z > -224.0 {
            let k = smoothstep(-224.0, -197.0, z);
            let target = if road != 0 { 0.0 } else if n.coast > 0.0 { h } else { -PLATE + (h.min(2.4) + PLATE).max(0.0) * smoothstep(6.0, 45.0, strip_sd(x, z)) };
            h += (target - h) * k;
        }
        let plates = (h / PLATE).round() as i32;
        let colour = self.colour(x, z, plates, &n, road, pad);
        Cell { plates, colour, warm: n.warm > 0.55, road, pad, hole: hole(x, z) }
    }

    /// A top's colour: under water sand or the deep, the road, the pads, the beach, snow and rock on the heights,
    /// the forest floor, the meadows (low-frequency patches of a few greens, cooler in the north).
    fn colour(&self, x: f32, z: f32, plates: i32, n: &Nat, road: u8, pad: Option<u8>) -> u8 {
        let y = plates as f32 * PLATE;
        if y < SEA_Y {
            return if plates <= DEEP { DEEP_FLOOR } else if plates >= -5 { WET_SAND } else { SEA_FLOOR };
        }
        if road == 1 {
            return TRAIL;
        }
        if road == 2 {
            return TRAIL_EDGE;
        }
        if let Some(p) = pad {
            return PADS[p as usize].colour;
        }
        let patch = vnoise(x / 23.0, z / 23.0, 71) + vnoise(x / 7.0, z / 7.0, 72) * 0.25;
        if y <= -0.4 && n.coast > -30.0 {
            return if n.warm > 0.45 { if patch > 0.3 { SAND_DARK } else { SAND } } else if patch > 0.0 { ROCK_LIGHT } else { SAND_DARK };
        }
        let snow_line = 23.0 + 14.0 * n.warm;
        if y > snow_line && n.warm < 0.5 {
            return if patch > 0.35 { SNOW_SHADE } else { SNOW };
        }
        if y > snow_line - 7.0 || (n.cliff > 0.6 && y > 5.0 && n.warm < 0.5) {
            return if patch > 0.2 { ROCK_MID } else { ROCK_LIGHT };
        }
        if n.forest > 0.5 {
            return if patch > 0.1 { FOREST } else { GRASS_DARK };
        }
        if n.warm < 0.45 {
            return if patch > 0.3 { COOL_GRASS } else if patch < -0.3 { GRASS_DARK } else { GRASS };
        }
        if patch > 0.35 {
            GRASS_LIME
        } else if patch < -0.25 {
            GRASS_LIGHT
        } else {
            GRASS
        }
    }

    /// The ground's top at (x, z) in studs (the stud under the point).
    pub fn ground_y(&self, x: f32, z: f32) -> f32 {
        self.cell(x.floor() + 0.5, z.floor() + 0.5).plates as f32 * PLATE
    }

    /// The road polylines (for the map).
    pub fn road_lines(&self) -> Vec<Vec<Vec2>> {
        self.roads.iter().map(|r| r.pts.clone()).collect()
    }

    /// Debug builds: the pads don't overlap and lie inland, the roads end at their pads, start at the gate, stay
    /// above the sea with walkable steps.
    fn self_test(&self) -> SelfTest {
        let mut p = vec![];
        for (i, a) in PADS.iter().enumerate() {
            for b in PADS.iter().skip(i + 1) {
                if (a.x - b.x).abs() < a.hx + b.hx + PAD_RING && (a.z - b.z).abs() < a.hz + b.hz + PAD_RING {
                    p.push(format!("pads {} and {} overlap", a.name, b.name));
                }
            }
            for (cx, cz) in [(a.x - a.hx, a.z - a.hz), (a.x + a.hx, a.z - a.hz), (a.x - a.hx, a.z + a.hz), (a.x + a.hx, a.z + a.hz), (a.x, a.z)] {
                if coast(cx, cz) > -4.0 {
                    p.push(format!("pad {} reaches the sea at ({cx}, {cz})", a.name));
                }
            }
            if self.pad_h[i] <= SEA_Y {
                p.push(format!("pad {} below the sea", a.name));
            }
        }
        // the harbour lies by the sea
        let h = &PADS[1];
        if coast(h.x + h.hx + 50.0, h.z) < 0.0 && coast(h.x, h.z - h.hz - 50.0) < 0.0 && coast(h.x, h.z + h.hz + 50.0) < 0.0 {
            p.push("the harbour pad is not by the sea".into());
        }
        let (mut min_y, mut max_step) = (f32::MAX, 0);
        for (ri, r) in self.roads.iter().enumerate() {
            let end = *r.pts.last().unwrap();
            let pad = &PADS[ROAD_TO[ri]];
            if sd_box(end, Vec2::new(pad.x, pad.z), Vec2::new(pad.hx, pad.hz)) > 6.0 {
                p.push(format!("road {ri} ends away from {}", pad.name));
            }
            if ri > 0 && self.roads[0].pts.windows(2).all(|w| sd_capsule(r.pts[0], w[0], w[1], 0.0) > 1.0) && self.roads[1..ri].iter().all(|o| o.pts.windows(2).all(|w| sd_capsule(r.pts[0], w[0], w[1], 0.0) > 1.0)) {
                p.push(format!("road {ri} does not start on a road"));
            }
            // walk it stud by stud on its centre line
            let mut last: Option<i32> = None;
            for (j, w) in r.pts.windows(2).enumerate() {
                let len = (w[1] - w[0]).length();
                let steps = len.ceil() as usize;
                for s in 0..steps {
                    let q = w[0] + (w[1] - w[0]) * (s as f32 / steps as f32);
                    if q.y > SOUTH_Z - 0.5 {
                        continue;
                    }
                    let c = self.cell(q.x.floor() + 0.5, q.y.floor() + 0.5);
                    min_y = min_y.min(c.plates as f32 * PLATE);
                    if coast(q.x, q.y) > -8.0 {
                        p.push(format!("road {ri} segment {j} runs by the sea at ({:.0}, {:.0})", q.x, q.y));
                        break;
                    }
                    if let Some(l) = last {
                        if (c.plates - l).abs() > 1 && (c.plates - l).abs() > max_step {
                            p.push(format!("road {ri} steps {} plates at ({:.0}, {:.0})", c.plates - l, q.x, q.y));
                        }
                        max_step = max_step.max((c.plates - l).abs());
                    }
                    last = Some(c.plates);
                }
            }
        }
        if min_y <= SEA_Y {
            p.push(format!("a road dips into the sea ({min_y})"));
        }
        if max_step > 1 {
            p.push(format!("a road step of {max_step} plates"));
        }
        // the gate: the road starts at it, on the strip's level, on land
        let g = self.roads[0].pts[0];
        if (g.x - GATE_X).abs() > 0.1 || (g.y - SOUTH_Z).abs() > 0.1 || self.cell(GATE_X + 0.5, SOUTH_Z - 1.5).plates != 0 || coast(GATE_X, SOUTH_Z - 2.0) > -10.0 {
            p.push("the gate is not connected to the road at the strip's level".into());
        }
        SelfTest { ok: p.is_empty(), problems: p, road_min_y: if min_y == f32::MAX { 0.0 } else { min_y }, road_max_step: max_step }
    }

    pub fn describe(&self) -> serde_json::Value {
        serde_json::json!({
            "bake_ms": (self.bake_ms * 10.0).round() / 10.0,
            "lattice": [NX + 1, NZ + 1],
            "pads": PADS.iter().zip(self.pad_h).map(|(p, h)| serde_json::json!({"name": p.name, "kind": p.kind, "x": p.x, "z": p.z, "y": h})).collect::<Vec<_>>(),
            "selftest": self.selftest,
            // (round the strip: the village plate is a hole, the land beside it lies a plate below it)
            "hole_village": hole(20.0, -9.0) && !hole(-40.0, -9.0),
            "beside_village_y": (self.ground_y(-36.5, -9.5) * 100.0).round() / 100.0,
        })
    }
}
