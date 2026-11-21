//! The island's towns (OPEN_WORLD.md, the places like Pokémon Red / Blue): built from bricks on their flat pads
//! (`island::PADS`) the first time you come near, then kept.
//!
//! * **Brickton**, the city: an avenue from the south gate road and a cross street meet on a plaza; skyscrapers in
//!   the north (taller toward the middle), the big mall north-east of the plaza, low shops and offices in the south.
//! * **Port Saltwick**, the harbour: two rows of colourful houses along the street from the west, a pier into the
//!   sea in the east with a boat at it (later the way to the next island), a lighthouse.
//! * **Dewmere**, the quiet village: cottages round a green, a pond, trees.
//!
//! Every building is its own entity of parts per colour (it fades while it hides you, `fade.rs`) and a box collider.
//! The middle of each pad (where the road arrives, where a teleport lands) stays free. Bricks overlap by a few
//! hundredths so no two faces share a plane (as in the village).

use crate::fade::Fader;
use crate::island::{self, Island, PADS};
use crate::town::{self, m, OccluderSpec, TownAssets, STUD};
use engine::geom::MeshBuilder;
use engine::physics::rapier::prelude::ColliderBuilder;
use engine::prelude::*;
use serde_json::{json, Value};

/// How near (studs from a pad's middle) a town is built.
const BUILD_R: f32 = 330.0;

/// One building as it is built: parts per colour, its footprint and height (studs, for the collider and the fade).
struct Bld {
    mb: MeshBuilder,
    lo: Vec3,
    hi: Vec3,
    n: u32,
    solid: bool,
    /// the part that blocks you and fades (a house without its garden); None: all of it
    body: Option<(Vec3, Vec3)>,
}

impl Bld {
    fn new() -> Self {
        Self { mb: MeshBuilder::new(), lo: Vec3::splat(f32::MAX), hi: Vec3::splat(f32::MIN), n: 0, solid: true, body: None }
    }
    /// A box (studs) min..max in colour `hex`; every box a little larger or smaller than the last (no shared planes).
    fn bx(&mut self, hex: u32, a: Vec3, b: Vec3) {
        self.n += 1;
        let e = 0.011 * ((self.n % 7) as f32 - 3.0);
        let (a, b) = (a - Vec3::splat(e), b + Vec3::splat(e));
        self.mb.part(&format!("c{hex:06x}")).cuboid(m(a.x, a.y, a.z), m(b.x, b.y, b.z));
        self.lo = self.lo.min(a);
        self.hi = self.hi.max(b);
    }
    fn cyl(&mut self, hex: u32, c: Vec3, r: f32, h: f32) {
        let t = Mat4::from_translation(m(c.x, c.y, c.z));
        self.mb.part(&format!("c{hex:06x}"));
        self.mb.with(t, |mb| {
            mb.cylinder(r * STUD, h * STUD, 12);
        });
        self.lo = self.lo.min(c - Vec3::new(r, 0.0, r));
        self.hi = self.hi.max(c + Vec3::new(r, h, r));
    }
}

struct Rng(u32);
impl Rng {
    fn f(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
    fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[((self.f() * v.len() as f32) as usize).min(v.len() - 1)]
    }
}

fn shade(hex: u32, k: f32) -> u32 {
    let c = |s: u32| (((hex >> s) & 255) as f32 * k).min(255.0) as u32;
    (c(16) << 16) | (c(8) << 8) | c(0)
}

// ------------------------------------------------------------------------------------------------ buildings

/// Windows round a box (x0..x1, z0..z1) in bands from y0 up every `floor` studs: glass a little proud of the walls,
/// inset from the corners.
fn window_bands(b: &mut Bld, glass: u32, x0: f32, x1: f32, z0: f32, z1: f32, y0: f32, y1: f32, floor: f32, band: f32) {
    let o = 0.08;
    let mut y = y0 + (floor - band) * 0.6;
    while y + band < y1 - 0.4 {
        b.bx(glass, Vec3::new(x0 + 0.6, y, z1), Vec3::new(x1 - 0.6, y + band, z1 + o));
        b.bx(glass, Vec3::new(x0 + 0.6, y, z0 - o), Vec3::new(x1 - 0.6, y + band, z0));
        b.bx(glass, Vec3::new(x0 - o, y, z0 + 0.6), Vec3::new(x0, y + band, z1 - 0.6));
        b.bx(glass, Vec3::new(x1, y, z0 + 0.6), Vec3::new(x1 + o, y + band, z1 - 0.6));
        y += floor;
    }
}

/// A skyscraper: a body, window bands per floor, a setback crown, a roof edge, sometimes an antenna.
fn tower(r: &mut Rng, b: &mut Bld, c: Vec3, w: f32, d: f32, h: f32) {
    let body = r.pick(&[0xa0a5a9, 0x6c6e68, 0xe4cd9e, 0xf4f4f4, 0x0a3463, 0x5a93db, 0x9ba19d]);
    let glass = r.pick(&[0x9fd8ef, 0x6fb6e0, 0xbfe6f4, 0x3b7fb8]);
    let (x0, x1, z0, z1) = (c.x - w * 0.5, c.x + w * 0.5, c.z - d * 0.5, c.z + d * 0.5);
    b.bx(shade(body, 0.8), Vec3::new(x0 - 0.4, c.y, z0 - 0.4), Vec3::new(x1 + 0.4, c.y + 1.2, z1 + 0.4));
    if h > 40.0 || r.f() < 0.4 {
        // a glass tower: the glass body, pillars of the frame colour up every side, a band every three floors
        b.bx(glass, Vec3::new(x0, c.y, z0), Vec3::new(x1, c.y + h, z1));
        let o = 0.12;
        let cols = |len: f32| ((len / 3.2).round() as i32).max(2);
        for side in 0..4 {
            let (len, along_x) = if side < 2 { (w, true) } else { (d, false) };
            let n = cols(len);
            for i in 0..=n {
                let t = -len * 0.5 + len * i as f32 / n as f32;
                let (a, bb) = match side {
                    0 => (Vec3::new(c.x + t - 0.3, c.y, z1), Vec3::new(c.x + t + 0.3, c.y + h, z1 + o)),
                    1 => (Vec3::new(c.x + t - 0.3, c.y, z0 - o), Vec3::new(c.x + t + 0.3, c.y + h, z0)),
                    2 => (Vec3::new(x0 - o, c.y, c.z + t - 0.3), Vec3::new(x0, c.y + h, c.z + t + 0.3)),
                    _ => (Vec3::new(x1, c.y, c.z + t - 0.3), Vec3::new(x1 + o, c.y + h, c.z + t + 0.3)),
                };
                let _ = along_x;
                b.bx(body, a, bb);
            }
        }
        let mut y = c.y + 7.2;
        while y < c.y + h - 1.0 {
            b.bx(body, Vec3::new(x0 - o * 0.5, y, z0 - o * 0.5), Vec3::new(x1 + o * 0.5, y + 0.5, z1 + o * 0.5));
            y += 7.2;
        }
    } else {
        b.bx(body, Vec3::new(x0, c.y, z0), Vec3::new(x1, c.y + h, z1));
        window_bands(b, glass, x0, x1, z0, z1, c.y + 1.2, c.y + h, 2.4, 1.3);
    }
    // the lobby's glass front (south, toward the street) and the crown
    b.bx(glass, Vec3::new(c.x - w * 0.3, c.y + 0.2, z1), Vec3::new(c.x + w * 0.3, c.y + 2.2, z1 + 0.15));
    let top = c.y + h;
    b.bx(shade(body, 0.85), Vec3::new(x0 - 0.3, top, z0 - 0.3), Vec3::new(x1 + 0.3, top + 0.6, z1 + 0.3));
    if h > 30.0 {
        // a setback crown (or two on the tallest), a spire on some
        let mut y = top + 0.6;
        let mut k = 0.6;
        for _ in 0..if h > 90.0 { 2 } else { 1 } {
            let (iw, id) = (w * k, d * k);
            b.bx(body, Vec3::new(c.x - iw * 0.5, y, c.z - id * 0.5), Vec3::new(c.x + iw * 0.5, y + 6.0, c.z + id * 0.5));
            window_bands(b, glass, c.x - iw * 0.5, c.x + iw * 0.5, c.z - id * 0.5, c.z + id * 0.5, y, y + 6.0, 2.4, 1.3);
            y += 6.0;
            k *= 0.65;
        }
        if r.f() < 0.6 {
            b.bx(0xc91a09, Vec3::new(c.x - 0.25, y, c.z - 0.25), Vec3::new(c.x + 0.25, y + 6.0 + h * 0.08, c.z + 0.25));
        }
    } else {
        // roof boxes (air conditioning, a water tank)
        b.bx(0x6c6e68, Vec3::new(c.x - 1.5, top + 0.6, c.z - 1.0), Vec3::new(c.x + 1.5, top + 2.0, c.z + 1.0));
    }
}

/// A low city building: a shop floor with an awning and a big window, an office floor or two above.
fn shop(r: &mut Rng, b: &mut Bld, c: Vec3, w: f32, d: f32, floors: u32, front: f32) {
    let wall = r.pick(&[0xe4cd9e, 0xf4f4f4, 0xc91a09, 0x0055bf, 0xfe8a18, 0x82422a, 0xa0a5a9, 0x4b9f4a]);
    let awning = r.pick(&[0xc91a09, 0xf2cd37, 0x0055bf, 0x237841, 0xff8ac0]);
    let (x0, x1, z0, z1) = (c.x - w * 0.5, c.x + w * 0.5, c.z - d * 0.5, c.z + d * 0.5);
    let h = 4.0 + floors as f32 * 3.2;
    b.bx(wall, Vec3::new(x0, c.y, z0), Vec3::new(x1, c.y + h, z1));
    // the front faces the street (+z or -z)
    let (zf, out) = if front > 0.0 { (z1, 1.0) } else { (z0, -1.0) };
    let fz = |t: f32| zf + out * t;
    let span = |a: f32, b2: f32| if out > 0.0 { (a, b2) } else { (-b2, -a) };
    let (sa, sb) = span(0.0, 0.15);
    b.bx(0x9fd8ef, Vec3::new(x0 + 1.0, c.y + 0.4, zf + sa), Vec3::new(x1 - 1.0, c.y + 3.0, zf + sb));
    let (aa, ab) = span(0.0, 2.0);
    b.bx(awning, Vec3::new(x0 + 0.5, c.y + 3.2, zf + aa), Vec3::new(x1 - 0.5, c.y + 3.6, zf + ab));
    let _ = fz;
    window_bands(b, 0x9fd8ef, x0, x1, z0, z1, c.y + 4.0, c.y + h, 3.2, 1.6);
    b.bx(shade(wall, 0.8), Vec3::new(x0 - 0.2, c.y + h, z0 - 0.2), Vec3::new(x1 + 0.2, c.y + h + 0.5, z1 + 0.2));
}

/// The mall: a long hall with a glass front and entrance, a coloured band with the sign, a glass roof light.
fn mall(b: &mut Bld, c: Vec3, w: f32, d: f32) {
    let (x0, x1, z0, z1) = (c.x - w * 0.5, c.x + w * 0.5, c.z - d * 0.5, c.z + d * 0.5);
    let h = 26.0;
    b.bx(0xf4f4f4, Vec3::new(x0, c.y, z0), Vec3::new(x1, c.y + h, z1));
    // two glass storeys on the front (south), white pillars, the entrance two storeys tall in the middle
    b.bx(0x6fb6e0, Vec3::new(x0 + 2.0, c.y + 0.4, z1), Vec3::new(x1 - 2.0, c.y + 9.0, z1 + 0.2));
    b.bx(0x6fb6e0, Vec3::new(x0 + 2.0, c.y + 10.2, z1), Vec3::new(x1 - 2.0, c.y + 15.6, z1 + 0.2));
    for k in 0..7 {
        let x = x0 + 2.0 + (w - 4.0) * k as f32 / 6.0;
        b.bx(0xf4f4f4, Vec3::new(x - 0.25, c.y + 0.4, z1 + 0.2), Vec3::new(x + 0.25, c.y + 15.6, z1 + 0.35));
    }
    b.bx(0x1b2a34, Vec3::new(c.x - 5.0, c.y + 0.4, z1 + 0.35), Vec3::new(c.x + 5.0, c.y + 8.6, z1 + 0.5));
    b.bx(0x9fd8ef, Vec3::new(c.x - 4.6, c.y + 0.4, z1 + 0.5), Vec3::new(c.x + 4.6, c.y + 8.2, z1 + 0.6));
    b.bx(0x1b2a34, Vec3::new(c.x - 0.15, c.y + 0.4, z1 + 0.6), Vec3::new(c.x + 0.15, c.y + 8.2, z1 + 0.7));
    // the sign band high up: red with yellow "letters" (blocks)
    b.bx(0xc91a09, Vec3::new(x0 + 1.0, c.y + 17.0, z1), Vec3::new(x1 - 1.0, c.y + 22.4, z1 + 0.4));
    let mut x = c.x - 13.0;
    for wl in [3.2, 1.2, 3.2, 1.2, 2.2, 1.2, 2.2, 1.2, 3.2] {
        if wl > 1.5 {
            b.bx(0xf2cd37, Vec3::new(x, c.y + 18.0, z1 + 0.4), Vec3::new(x + wl, c.y + 21.4, z1 + 0.6));
        }
        x += wl + 0.6;
    }
    // the entrance canopy above the doors: a glass roof on a white frame out from the wall, a yellow edge, two struts
    let cy = c.y + 9.4;
    b.bx(0xf4f4f4, Vec3::new(c.x - 7.0, cy, z1), Vec3::new(c.x + 7.0, cy + 0.5, z1 + 5.0));
    b.bx(0x9fd8ef, Vec3::new(c.x - 6.6, cy + 0.5, z1 + 0.2), Vec3::new(c.x + 6.6, cy + 0.65, z1 + 4.7));
    b.bx(0xf2cd37, Vec3::new(c.x - 7.1, cy - 0.3, z1 + 5.0), Vec3::new(c.x + 7.1, cy + 0.7, z1 + 5.3));
    for s in [-1.0, 1.0] {
        b.bx(0xa0a5a9, Vec3::new(c.x + s * 6.0 - 0.2, cy + 0.5, z1 + 0.1), Vec3::new(c.x + s * 6.0 + 0.2, cy + 4.0, z1 + 0.5));
    }
    // roof: edge, a long glass light, a machine room
    b.bx(0xa0a5a9, Vec3::new(x0 - 0.3, c.y + h, z0 - 0.3), Vec3::new(x1 + 0.3, c.y + h + 0.6, z1 + 0.3));
    b.bx(0x9fd8ef, Vec3::new(x0 + 6.0, c.y + h + 0.6, c.z - 3.0), Vec3::new(x1 - 6.0, c.y + h + 2.4, c.z + 3.0));
    b.bx(0x6c6e68, Vec3::new(x0 + 3.0, c.y + h + 0.6, z0 + 3.0), Vec3::new(x0 + 9.0, c.y + h + 3.6, z0 + 8.0));
}

/// A house's look (Brick Ages: a few parameters make every house different).
struct HouseStyle {
    /// walls, roof, accent (shutters, door, flower boxes): a curated triple
    colours: (u32, u32, u32),
    storeys: u32,
    /// false: a stepped gable along x; true: a hip roof stepping in on all sides
    hip: bool,
    /// the door's side: +1 south, -1 north
    front: f32,
    /// a garden round it: a picket fence, a path to the door, flower beds
    garden: bool,
}

/// A wall of bricks in running bond (rows of 1.2 studs, bricks of 2 or 4, each row's joints offset), a touch of
/// brightness jitter per brick, holes left for the openings (along0, along1, bottom, top). `along_x`: the wall runs
/// along x from `a0` to `a1`, its outer face at `face`, `out` the outside's sign.
fn brick_wall(b: &mut Bld, hex: u32, along_x: bool, a0: f32, a1: f32, face: f32, out: f32, y0: f32, y1: f32, holes: &[(f32, f32, f32, f32)], seed: u32) {
    let t = 1.0;
    let (f0, f1) = if out > 0.0 { (face - t, face) } else { (face, face + t) };
    let mut row = 0u32;
    let mut y = y0;
    while y < y1 - 0.05 {
        let yt = (y + 1.2).min(y1);
        let mut a = a0;
        let mut k = 0u32;
        let mut next = a0 + if row % 2 == 1 { 2.0 } else { 4.0 };
        while a < a1 - 0.01 {
            let e = next.min(a1);
            // the brick, cut round the holes in this row
            let mut segs = vec![(a, e)];
            for &(h0, h1, hy0, hy1) in holes {
                if hy1 <= y + 0.01 || hy0 >= yt - 0.01 {
                    continue;
                }
                let mut nsegs = vec![];
                for (s0, s1) in segs {
                    if h1 <= s0 || h0 >= s1 {
                        nsegs.push((s0, s1));
                        continue;
                    }
                    if h0 - s0 > 0.05 {
                        nsegs.push((s0, h0));
                    }
                    if s1 - h1 > 0.05 {
                        nsegs.push((h1, s1));
                    }
                }
                segs = nsegs;
            }
            let jit = [0.96, 1.0, 1.03, 0.985][(seed.wrapping_mul(31).wrapping_add(row * 17 + k * 7) % 4) as usize];
            let col = shade(hex, jit);
            for (s0, s1) in segs {
                if along_x {
                    b.bx(col, Vec3::new(s0, y, f0), Vec3::new(s1, yt, f1));
                } else {
                    b.bx(col, Vec3::new(f0, y, s0), Vec3::new(f1, yt, s1));
                }
            }
            a = e;
            next = a + if (seed + row + k) % 3 == 0 { 2.0 } else { 4.0 };
            k += 1;
        }
        y = yt;
        row += 1;
    }
}

/// A window in a wall's opening: glass set back, a white frame with a cross, shutters in the accent colour, a flower
/// box below (ground floor fronts).
fn window(r: &mut Rng, b: &mut Bld, along_x: bool, at: f32, face: f32, out: f32, y0: f32, w: f32, h: f32, accent: u32, flowers: bool) {
    let p = |al: f32, o: f32, y: f32| if along_x { Vec3::new(al, y, face + out * o) } else { Vec3::new(face + out * o, y, al) };
    let bx = |b: &mut Bld, col: u32, a0: f32, a1: f32, o0: f32, o1: f32, y0: f32, y1: f32| {
        let (pa, pb) = (p(a0, o0, y0), p(a1, o1, y1));
        b.bx(col, pa.min(pb), pa.max(pb));
    };
    let (a0, a1) = (at - w * 0.5, at + w * 0.5);
    bx(b, 0x9fd8ef, a0, a1, -0.6, -0.4, y0, y0 + h);
    bx(b, 0xf4f4f4, a0 - 0.2, a1 + 0.2, -0.1, 0.25, y0 - 0.25, y0);
    bx(b, 0xf4f4f4, a0 - 0.2, a1 + 0.2, -0.1, 0.15, y0 + h, y0 + h + 0.2);
    bx(b, 0xf4f4f4, a0 - 0.2, a0, -0.4, 0.15, y0, y0 + h);
    bx(b, 0xf4f4f4, a1, a1 + 0.2, -0.4, 0.15, y0, y0 + h);
    bx(b, 0xf4f4f4, at - 0.08, at + 0.08, -0.45, -0.3, y0, y0 + h);
    bx(b, 0xf4f4f4, a0, a1, -0.45, -0.3, y0 + h * 0.55, y0 + h * 0.55 + 0.12);
    for s in [-1.0f32, 1.0] {
        let (sa, sb) = if s < 0.0 { (a0 - 0.2 - w * 0.5, a0 - 0.2) } else { (a1 + 0.2, a1 + 0.2 + w * 0.5) };
        bx(b, accent, sa, sb, 0.0, 0.15, y0 - 0.05, y0 + h + 0.05);
        bx(b, shade(accent, 0.8), sa + 0.15, sb - 0.15, 0.15, 0.22, y0 + h * 0.3, y0 + h * 0.36);
    }
    if flowers {
        bx(b, 0x583927, a0 - 0.1, a1 + 0.1, 0.25, 0.9, y0 - 0.85, y0 - 0.3);
        let n = ((w / 0.7) as i32).max(2);
        for i in 0..n {
            let fa = a0 + 0.35 + (w - 0.7) * i as f32 / (n - 1) as f32;
            let col = r.pick(&[0xc91a09, 0xf2cd37, 0xff8ac0, 0xf4f4f4, 0xfe8a18]);
            bx(b, 0x4b9f4a, fa - 0.25, fa + 0.25, 0.35, 0.8, y0 - 0.3, y0 - 0.05);
            bx(b, col, fa - 0.18, fa + 0.18, 0.4, 0.75, y0 - 0.05, y0 + 0.2);
        }
    }
}

/// A house in the Brick Ages manner: a stone foundation, brick walls in running bond with real openings, framed
/// windows with shutters and flower boxes, a door with its frame, a step and a lamp, a stepped gable or hip roof
/// with a ridge and a chimney; with a garden: a white picket fence, a path to the door, flower beds.
fn house(r: &mut Rng, b: &mut Bld, c: Vec3, w: f32, d: f32, st: &HouseStyle) {
    let (wall, roof, accent) = st.colours;
    let (x0, x1, z0, z1) = (c.x - w * 0.5, c.x + w * 0.5, c.z - d * 0.5, c.z + d * 0.5);
    let base = c.y + 0.6;
    let storey = 4.8;
    let top = base + storey * st.storeys as f32;
    b.bx(0x6c6e68, Vec3::new(x0 - 0.4, c.y, z0 - 0.4), Vec3::new(x1 + 0.4, base, z1 + 0.4));
    let (fz, fo) = if st.front > 0.0 { (z1, 1.0) } else { (z0, -1.0) };
    let (bz, bo) = (if st.front > 0.0 { z0 } else { z1 }, -fo);
    let (ww, wh) = (2.0, 2.4);
    let mut front_holes = vec![(c.x - 1.0, c.x + 1.0, base, base + 3.6)];
    let mut back_holes = vec![];
    let mut side_holes = vec![];
    let wxs = [c.x - w * 0.5 + 2.6, c.x + w * 0.5 - 2.6];
    for k in 0..st.storeys {
        let wy = base + 1.2 + k as f32 * storey;
        for &wx in &wxs {
            front_holes.push((wx - ww * 0.5, wx + ww * 0.5, wy, wy + wh));
            back_holes.push((wx - ww * 0.5, wx + ww * 0.5, wy, wy + wh));
        }
        side_holes.push((c.z - ww * 0.5, c.z + ww * 0.5, wy, wy + wh));
    }
    let seed = (c.x.abs() * 7.0 + c.z.abs() * 13.0) as u32;
    brick_wall(b, wall, true, x0, x1, fz, fo, base, top, &front_holes, seed);
    brick_wall(b, wall, true, x0, x1, bz, bo, base, top, &back_holes, seed + 1);
    brick_wall(b, wall, false, z0 + 1.0, z1 - 1.0, x0, -1.0, base, top, &side_holes, seed + 2);
    brick_wall(b, wall, false, z0 + 1.0, z1 - 1.0, x1, 1.0, base, top, &side_holes, seed + 3);
    // (inside: one dark block, nothing to see through the glass)
    b.bx(shade(wall, 0.6), Vec3::new(x0 + 1.0, base, z0 + 1.0), Vec3::new(x1 - 1.0, top - 0.2, z1 - 1.0));
    for k in 0..st.storeys {
        let wy = base + 1.2 + k as f32 * storey;
        for &wx in &wxs {
            window(r, b, true, wx, fz, fo, wy, ww, wh, accent, k == 0);
            window(r, b, true, wx, bz, bo, wy, ww, wh, accent, false);
        }
        window(r, b, false, c.z, x0, -1.0, wy, ww, wh, accent, false);
        window(r, b, false, c.z, x1, 1.0, wy, ww, wh, accent, false);
    }
    // the door: the door with a little window and a knob, a white frame, a step, a lamp over it
    let zz = |a: f32, bb: f32| if fo > 0.0 { (fz + a, fz + bb) } else { (fz - bb, fz - a) };
    let (a, bb) = zz(-0.6, -0.4);
    b.bx(accent, Vec3::new(c.x - 1.0, base, a), Vec3::new(c.x + 1.0, base + 3.6, bb));
    let (a, bb) = zz(-0.4, -0.3);
    b.bx(0x9fd8ef, Vec3::new(c.x - 0.4, base + 2.4, a), Vec3::new(c.x + 0.4, base + 3.2, bb));
    let (a, bb) = zz(-0.4, -0.2);
    b.bx(0xf2cd37, Vec3::new(c.x + 0.55, base + 1.6, a), Vec3::new(c.x + 0.75, base + 1.8, bb));
    let (a, bb) = zz(-0.1, 0.2);
    b.bx(0xf4f4f4, Vec3::new(c.x - 1.3, base + 3.6, a), Vec3::new(c.x + 1.3, base + 4.0, bb));
    for s in [-1.0f32, 1.0] {
        b.bx(0xf4f4f4, Vec3::new(c.x + s * 1.15 - 0.15, base, a), Vec3::new(c.x + s * 1.15 + 0.15, base + 3.6, bb));
    }
    let (a, bb) = zz(0.0, 1.4);
    b.bx(0xa0a5a9, Vec3::new(c.x - 1.6, c.y, a), Vec3::new(c.x + 1.6, base - 0.1, bb));
    let (a, bb) = zz(0.2, 0.7);
    b.bx(0x1b2a34, Vec3::new(c.x - 0.25, base + 4.2, a), Vec3::new(c.x + 0.25, base + 4.9, bb));
    let (a, bb) = zz(0.3, 0.6);
    b.bx(0xf2cd37, Vec3::new(c.x - 0.18, base + 4.25, a), Vec3::new(c.x + 0.18, base + 4.6, bb));
    // the roof: a cornice, then steps (along x for a gable, in on all sides for a hip), a ridge, a chimney
    b.bx(shade(wall, 0.85), Vec3::new(x0 - 0.3, top, z0 - 0.3), Vec3::new(x1 + 0.3, top + 0.4, z1 + 0.3));
    let ov = 0.8;
    let step = 1.4;
    let mut i = 0;
    loop {
        let dd = d + ov * 2.0 - i as f32 * step;
        let wd = if st.hip { w + ov * 2.0 - i as f32 * step } else { w + ov * 2.0 };
        if dd <= 1.2 || wd <= 1.2 {
            break;
        }
        let col = if i % 2 == 0 { roof } else { shade(roof, 0.9) };
        let y = top + 0.4 + i as f32 * 0.7;
        b.bx(col, Vec3::new(c.x - wd * 0.5, y, c.z - dd * 0.5), Vec3::new(c.x + wd * 0.5, y + 0.7, c.z + dd * 0.5));
        i += 1;
    }
    let ridge_y = top + 0.4 + i as f32 * 0.7;
    let rw = if st.hip { (w + ov * 2.0 - (i as f32 - 1.0) * step).max(1.2) } else { w + ov * 2.0 };
    b.bx(shade(roof, 0.75), Vec3::new(c.x - rw * 0.5, ridge_y, c.z - 0.5), Vec3::new(c.x + rw * 0.5, ridge_y + 0.4, c.z + 0.5));
    let chx = if r.f() < 0.5 { x0 + 2.0 } else { x1 - 2.0 };
    b.bx(0x82422a, Vec3::new(chx - 0.8, top, c.z - 2.2), Vec3::new(chx + 0.8, ridge_y + 1.6, c.z - 0.8));
    b.bx(0x6c6e68, Vec3::new(chx - 1.0, ridge_y + 1.6, c.z - 2.4), Vec3::new(chx + 1.0, ridge_y + 2.0, c.z - 0.6));
    b.body = Some((Vec3::new(x0 - 0.5, c.y, z0 - 0.5), Vec3::new(x1 + 0.5, ridge_y + 2.0, z1 + 0.5)));
    if st.garden {
        // a plot round the house: a white picket fence with a gap at the path, flower beds by the front wall
        let mg = 3.0;
        let (px0, px1, pz0, pz1) = (x0 - mg, x1 + mg, z0 - mg, z1 + mg);
        let fzf = if st.front > 0.0 { pz1 } else { pz0 };
        let post = |b: &mut Bld, x: f32, z: f32| b.bx(0xf4f4f4, Vec3::new(x - 0.15, c.y, z - 0.15), Vec3::new(x + 0.15, c.y + 1.6, z + 0.15));
        let mut x = px0;
        while x <= px1 + 0.01 {
            for zr in [pz0, pz1] {
                if !((zr - fzf).abs() < 0.01 && (x - c.x).abs() < 1.5) {
                    post(b, x, zr);
                }
            }
            x += 1.0;
        }
        let mut z = pz0 + 1.0;
        while z < pz1 - 0.01 {
            post(b, px0, z);
            post(b, px1, z);
            z += 1.0;
        }
        for zr in [pz0, pz1] {
            let gap = (zr - fzf).abs() < 0.01;
            let spans = if gap { vec![(px0, c.x - 1.6), (c.x + 1.6, px1)] } else { vec![(px0, px1)] };
            for (ra, rb) in spans {
                b.bx(0xf4f4f4, Vec3::new(ra, c.y + 0.9, zr - 0.08), Vec3::new(rb, c.y + 1.15, zr + 0.08));
            }
        }
        for xr in [px0, px1] {
            b.bx(0xf4f4f4, Vec3::new(xr - 0.08, c.y + 0.9, pz0), Vec3::new(xr + 0.08, c.y + 1.15, pz1));
        }
        let (pa, pb) = if st.front > 0.0 { (z1 + 1.4, fzf) } else { (fzf, z0 - 1.4) };
        b.bx(0xe4cd9e, Vec3::new(c.x - 1.0, c.y, pa.min(pb)), Vec3::new(c.x + 1.0, c.y + 0.15, pa.max(pb)));
        for s in [-1.0f32, 1.0] {
            let (ba, bb) = if st.front > 0.0 { (z1 + 0.6, z1 + 2.0) } else { (z0 - 2.0, z0 - 0.6) };
            let (xa, xb) = if s < 0.0 { (x0 + 0.5, c.x - 2.0) } else { (c.x + 2.0, x1 - 0.5) };
            if xb - xa < 1.0 {
                continue;
            }
            b.bx(0x583927, Vec3::new(xa, c.y, ba), Vec3::new(xb, c.y + 0.4, bb));
            let zm = (ba + bb) * 0.5;
            let mut fx = xa + 0.5;
            while fx < xb - 0.3 {
                let col = r.pick(&[0xc91a09, 0xf2cd37, 0xff8ac0, 0xf4f4f4, 0x5a93db]);
                b.bx(0x4b9f4a, Vec3::new(fx - 0.2, c.y + 0.4, zm - 0.2), Vec3::new(fx + 0.2, c.y + 0.9, zm + 0.2));
                b.bx(col, Vec3::new(fx - 0.3, c.y + 0.9, zm - 0.3), Vec3::new(fx + 0.3, c.y + 1.3, zm + 0.3));
                fx += 1.0;
            }
        }
    }
}

/// The harbour's pier east into the sea, a boat moored at it.
fn pier(b: &mut Bld, c: Vec3, len: f32) {
    let deck = c.y + 0.2;
    b.bx(0x82422a, Vec3::new(c.x, deck - 0.4, c.z - 2.0), Vec3::new(c.x + len, deck, c.z + 2.0));
    let mut x = c.x + 2.0;
    while x < c.x + len {
        for s in [-1.0, 1.0] {
            b.bx(0x583927, Vec3::new(x - 0.3, island::SEA_Y - 2.0, c.z + s * 1.9 - 0.3), Vec3::new(x + 0.3, deck + 0.8, c.z + s * 1.9 + 0.3));
        }
        x += 5.0;
    }
}

fn boat(b: &mut Bld, c: Vec3) {
    let y = island::SEA_Y - 0.4;
    // hull (two steps, narrower below), a white cabin, a mast with a sail
    b.bx(0xc91a09, Vec3::new(c.x - 5.0, y - 0.6, c.z - 1.4), Vec3::new(c.x + 5.0, y + 0.4, c.z + 1.4));
    b.bx(0xf4f4f4, Vec3::new(c.x - 6.0, y + 0.4, c.z - 2.0), Vec3::new(c.x + 6.0, y + 1.6, c.z + 2.0));
    b.bx(0x0055bf, Vec3::new(c.x + 6.0, y + 0.8, c.z - 1.2), Vec3::new(c.x + 7.2, y + 1.6, c.z + 1.2));
    b.bx(0xe4cd9e, Vec3::new(c.x - 5.6, y + 1.6, c.z - 1.6), Vec3::new(c.x + 5.6, y + 1.9, c.z + 1.6));
    b.bx(0xf4f4f4, Vec3::new(c.x - 3.0, y + 1.9, c.z - 1.3), Vec3::new(c.x + 0.6, y + 4.3, c.z + 1.3));
    b.bx(0x9fd8ef, Vec3::new(c.x - 3.1, y + 2.9, c.z - 1.0), Vec3::new(c.x - 3.0, y + 3.9, c.z + 1.0));
    b.bx(0x583927, Vec3::new(c.x + 2.0, y + 1.9, c.z - 0.2), Vec3::new(c.x + 2.4, y + 12.0, c.z + 0.2));
    b.bx(0xf4f4f4, Vec3::new(c.x + 2.4, y + 3.0, c.z - 0.06), Vec3::new(c.x + 6.4, y + 11.0, c.z + 0.06));
}

fn lighthouse(b: &mut Bld, c: Vec3) {
    b.bx(0xa0a5a9, Vec3::new(c.x - 3.0, c.y, c.z - 3.0), Vec3::new(c.x + 3.0, c.y + 1.2, c.z + 3.0));
    for i in 0..6 {
        let col = if i % 2 == 0 { 0xf4f4f4 } else { 0xc91a09 };
        let r = 2.2 - i as f32 * 0.15;
        b.cyl(col, Vec3::new(c.x, c.y + 1.2 + i as f32 * 3.0, c.z), r, 3.0);
    }
    let top = c.y + 1.2 + 18.0;
    b.bx(0x1b2a34, Vec3::new(c.x - 1.8, top, c.z - 1.8), Vec3::new(c.x + 1.8, top + 0.4, c.z + 1.8));
    b.cyl(0xf2cd37, Vec3::new(c.x, top + 0.4, c.z), 1.1, 2.0);
    b.cyl(0xc91a09, Vec3::new(c.x, top + 2.4, c.z), 1.4, 0.8);
}

// ------------------------------------------------------------------------------------------------ the towns

/// What one town is made of: buildings (solid, fade), decorations (no collider), trees (models), water.
#[derive(Default)]
struct Plan {
    blds: Vec<Bld>,
    trees: Vec<(&'static str, Vec3, f32)>,
    ponds: Vec<(Vec3, Vec2)>,
}

/// Brickton: lots in the four quadrants round the plaza (avenue x = pad x, cross street z = pad z).
fn brickton(y: f32) -> Plan {
    let p = &PADS[0];
    let mut r = Rng(7001);
    let mut plan = Plan::default();
    let street = 7.0;
    let plaza = 18.0;
    // the mall: north-east of the plaza, its front toward the cross street
    let mall_c = Vec3::new(p.x + 34.0, y, p.z - 36.0);
    let (mw, md) = (52.0, 30.0);
    let mut b = Bld::new();
    mall(&mut b, mall_c, mw, md);
    plan.blds.push(b);
    let lot = 17.0;
    for (qx, qz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let (xa, xb) = if qx < 0.0 { (p.x - p.hx + 3.0, p.x - street) } else { (p.x + street, p.x + p.hx - 3.0) };
        let (za, zb) = if qz < 0.0 { (p.z - p.hz + 3.0, p.z - street) } else { (p.z + street, p.z + p.hz - 3.0) };
        let mut z = za;
        while z + lot <= zb + 0.01 {
            let mut x = xa;
            while x + lot <= xb + 0.01 {
                let c = Vec3::new(x + lot * 0.5, y, z + lot * 0.5);
                x += lot + 3.0;
                // the plaza and the mall keep their room
                if (c.x - p.x).abs() < plaza + lot * 0.5 && (c.z - p.z).abs() < plaza + lot * 0.5 {
                    continue;
                }
                if (c.x - mall_c.x).abs() < mw * 0.5 + lot * 0.5 + 1.0 && (c.z - mall_c.z).abs() < md * 0.5 + lot * 0.5 + 6.0 {
                    continue;
                }
                let mut b = Bld::new();
                let w = lot - 2.0 - r.f() * 3.0;
                let d = lot - 2.0 - r.f() * 3.0;
                if qz < 0.0 {
                    // north: towers, taller toward the middle of the city
                    let mid = 1.0 - ((c.x - p.x).abs() / p.hx).min(1.0) * 0.6 - ((c.z - p.z).abs() / p.hz).min(1.0) * 0.3;
                    let h = 48.0 + mid * 95.0 + r.f() * 25.0;
                    tower(&mut r, &mut b, c, w, d, h);
                } else if r.f() < 0.35 {
                    let h = 40.0 + r.f() * 40.0;
                    tower(&mut r, &mut b, c, w, d, h);
                } else {
                    // shops face the street they lie on (the cross street north of them)
                    let front = if (c.z - p.z).abs() < 25.0 { -1.0 } else { 1.0 };
                    let floors = 3 + (r.f() * 4.0) as u32;
                    shop(&mut r, &mut b, c, w, d, floors, front);
                }
                plan.blds.push(b);
            }
            z += lot + 3.0;
        }
    }
    // trees along the avenue and round the plaza
    let mut z = p.z - p.hz + 6.0;
    while z < p.z + p.hz - 4.0 {
        if (z - p.z).abs() > plaza + 2.0 {
            for s in [-1.0, 1.0] {
                plan.trees.push(("assets/props/round_tree.glb", Vec3::new(p.x + s * (street - 1.5), y, z), 3.0));
            }
        }
        z += 12.0;
    }
    for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        plan.trees.push(("assets/props/round_tree.glb", Vec3::new(p.x + dx * (plaza - 3.0), y, p.z + dz * (plaza - 3.0)), 4.0));
    }
    plan.ponds.push((Vec3::new(p.x, y, p.z - 9.0), Vec2::new(8.0, 5.0)));
    plan
}

/// Port Saltwick: houses either side of the street from the west road, the pier and the boat east, a lighthouse.
fn saltwick(y: f32) -> Plan {
    let p = &PADS[1];
    let mut r = Rng(7002);
    let mut plan = Plan::default();
    // (wall, roof, shutters and door): bright harbour colours
    let triples = [
        (0x5a93db, 0xc91a09, 0xf4f4f4), (0xf2cd37, 0x0a3463, 0x0055bf), (0xff8ac0, 0x6c6e68, 0xf4f4f4),
        (0xf4f4f4, 0x0a3463, 0x5a93db), (0x9fd8ef, 0xc91a09, 0x0a3463), (0xfe8a18, 0x237841, 0xf4f4f4),
        (0xe4cd9e, 0x82422a, 0x237841),
    ];
    let mut x = p.x - p.hx + 15.0;
    while x < p.x + p.hx - 9.0 {
        for (s, front) in [(-1.0, 1.0), (1.0, -1.0)] {
            let w = 12.4 + r.f() * 1.6;
            let d = 8.0 + r.f() * 2.0;
            let c = Vec3::new(x, y, p.z + s * (5.0 + d * 0.5 + 2.0));
            let mut b = Bld::new();
            let st = HouseStyle { colours: r.pick(&triples), storeys: 2 + (r.f() < 0.4) as u32, hip: r.f() < 0.3, front, garden: false };
            house(&mut r, &mut b, c, w, d, &st);
            plan.blds.push(b);
        }
        x += 16.0;
    }
    let mut b = Bld::new();
    pier(&mut b, Vec3::new(p.x + p.hx - 2.0, y, p.z), 34.0);
    b.solid = false;
    plan.blds.push(b);
    let mut b = Bld::new();
    boat(&mut b, Vec3::new(p.x + p.hx + 24.0, y, p.z + 5.6));
    b.solid = false;
    plan.blds.push(b);
    let mut b = Bld::new();
    lighthouse(&mut b, Vec3::new(p.x + p.hx - 8.0, y, p.z - p.hz + 8.0));
    plan.blds.push(b);
    plan
}

/// Dewmere: cottages round a green with a pond, trees between.
fn dewmere(y: f32) -> Plan {
    let p = &PADS[2];
    let mut r = Rng(7003);
    let mut plan = Plan::default();
    // (wall, roof, shutters and door): warm cottage colours
    let triples = [
        (0xf4f4f4, 0x82422a, 0x237841), (0xe4cd9e, 0x583927, 0xc91a09), (0xf2cd37, 0x82422a, 0x0055bf),
        (0xffe0b0, 0x237841, 0x82422a), (0xf4f4f4, 0xc91a09, 0x0a3463), (0xe4cd9e, 0x0a3463, 0xfe8a18),
    ];
    let n = 9;
    for i in 0..n {
        let a = std::f32::consts::TAU * (i as f32 + 0.5) / n as f32;
        // (the east, where the road arrives, stays open)
        if a.cos() > 0.75 {
            continue;
        }
        let c = Vec3::new(p.x + a.cos() * (p.hx - 14.0), y, p.z + a.sin() * (p.hz - 14.0));
        let mut b = Bld::new();
        let front = if a.sin() > 0.0 { -1.0 } else { 1.0 };
        let (w, d) = (12.4 + r.f() * 1.6, 8.0 + r.f() * 1.5);
        let st = HouseStyle { colours: r.pick(&triples), storeys: 1 + (r.f() < 0.35) as u32, hip: r.f() < 0.4, front, garden: true };
        house(&mut r, &mut b, c, w, d, &st);
        plan.blds.push(b);
    }
    plan.ponds.push((Vec3::new(p.x - 18.0, y, p.z + 14.0), Vec2::new(14.0, 10.0)));
    for _ in 0..26 {
        let (tx, tz) = (p.x + (r.f() - 0.5) * 2.0 * (p.hx - 4.0), p.z + (r.f() - 0.5) * 2.0 * (p.hz - 4.0));
        let (ex, ez) = ((tx - p.x) / (p.hx - 14.0), (tz - p.z) / (p.hz - 14.0));
        let ring = (ex * ex + ez * ez).sqrt();
        // between the houses and the pad's edge, or a few on the green; never on the road in
        if (0.75..0.9).contains(&ring) || ring < 0.35 && (tx - p.x).abs() > 8.0 || (tz - p.z).abs() < 6.0 && tx > p.x {
            continue;
        }
        let path = if r.f() < 0.5 { "assets/props/round_tree.glb" } else { "assets/props/spruce.glb" };
        plan.trees.push((path, Vec3::new(tx, y, tz), if path.contains("spruce") { 7.0 + r.f() * 2.0 } else { 3.5 + r.f() }));
    }
    plan
}

// ------------------------------------------------------------------------------------------------ in the game

/// The towns in the play screen: built near, kept.
#[derive(Default)]
pub struct Places {
    built: [bool; 3],
    roots: Vec<Entity>,
    colliders: Vec<engine::physics::rapier::prelude::ColliderHandle>,
    pub buildings: usize,
    pub triangles: usize,
    pub trees: usize,
}

impl Places {
    /// Every frame on the island: builds the nearest unbuilt town within reach (one per frame).
    pub fn update(&mut self, ctx: &mut Ctx, assets: &mut TownAssets, law: &Island, fader: &mut Fader, hero: Vec2) {
        for (i, p) in PADS.iter().enumerate() {
            if self.built[i] || Vec2::new(p.x, p.z).distance(hero) > BUILD_R {
                continue;
            }
            self.built[i] = true;
            let y = law.pad_h[i];
            let plan = match i {
                0 => brickton(y),
                1 => saltwick(y),
                _ => dewmere(y),
            };
            self.spawn(ctx, assets, fader, plan, p.name);
            return;
        }
    }

    fn spawn(&mut self, ctx: &mut Ctx, assets: &mut TownAssets, fader: &mut Fader, plan: Plan, name: &str) {
        let t0 = engine::log::now_ms();
        let layer = island::LAYER_NEAR;
        let mut specs = vec![];
        let mut tris = 0;
        for (k, b) in plan.blds.into_iter().enumerate() {
            let (lo, hi) = b.body.unwrap_or((b.lo, b.hi));
            let solid = b.solid;
            tris += b.mb.triangle_count();
            let parts = b.mb.finish();
            let mats: Vec<(String, MaterialIdOf)> = parts.iter().map(|(p, _)| (p.clone(), assets.material(ctx, u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff)))).collect();
            let root = ctx.spawn_parts(&format!("{name} building {k}"), &parts, |p| mats.iter().find(|m| m.0 == p).map(|m| m.1).unwrap_or(mats[0].1), Transform::default());
            ctx.set_layer(root, layer);
            self.roots.push(root);
            if solid {
                let (c, h) = ((lo + hi) * 0.5, (hi - lo) * 0.5);
                let hd = ctx.physics.add_static(ColliderBuilder::cuboid(h.x * STUD, h.y * STUD, h.z * STUD), m(c.x, c.y, c.z));
                self.colliders.push(hd);
                specs.push(OccluderSpec::Bricks { root, extra: vec![], min: m(lo.x, lo.y, lo.z), max: m(hi.x, hi.y, hi.z) });
            }
            self.buildings += 1;
        }
        for (path, at, s) in &plan.trees {
            let opts = town::prop_opts(ctx, path, 0.0);
            let rot = Quat::from_rotation_y((at.x * 12.9898 + at.z * 78.233).sin() * 43758.5 % std::f32::consts::TAU);
            let e = ctx.load_gltf_with(path, Transform::from_translation(m(at.x, at.y, at.z)).with_rotation(rot).with_scale(Vec3::splat(*s)), opts);
            ctx.set_layer(e, layer);
            self.roots.push(e);
            self.trees += 1;
        }
        for (c, half) in &plan.ponds {
            // a raised basin: a dark bed a plate up, the water well above it (a few millimetres of water over the
            // ground flickered: foam and reflections fought over nothing), a stone rim above the water
            let (bed, water, rim) = (c.y + 0.4, c.y + 1.0, c.y + 1.3);
            let w = ctx.spawn_water(&format!("{name} pond"), assets.water, Vec2::new(half.x * 2.0 * STUD, half.y * 2.0 * STUD), Transform::from_translation(m(c.x, water, c.z)));
            ctx.set_layer(w, layer);
            let mut b = Bld::new();
            b.bx(0x1d4a5e, Vec3::new(c.x - half.x, c.y, c.z - half.y), Vec3::new(c.x + half.x, bed, c.z + half.y));
            for (a, bb) in [
                (Vec3::new(c.x - half.x - 0.8, c.y, c.z - half.y - 0.8), Vec3::new(c.x + half.x + 0.8, rim, c.z - half.y)),
                (Vec3::new(c.x - half.x - 0.8, c.y, c.z + half.y), Vec3::new(c.x + half.x + 0.8, rim, c.z + half.y + 0.8)),
                (Vec3::new(c.x - half.x - 0.8, c.y, c.z - half.y), Vec3::new(c.x - half.x, rim, c.z + half.y)),
                (Vec3::new(c.x + half.x, c.y, c.z - half.y), Vec3::new(c.x + half.x + 0.8, rim, c.z + half.y)),
            ] {
                b.bx(0xa0a5a9, a, bb);
            }
            let parts = b.mb.finish();
            let mats: Vec<(String, MaterialIdOf)> = parts.iter().map(|(p, _)| (p.clone(), assets.material(ctx, u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff)))).collect();
            let e = ctx.spawn_parts(&format!("{name} pond rim"), &parts, |p| mats.iter().find(|m| m.0 == p).map(|m| m.1).unwrap_or(mats[0].1), Transform::default());
            ctx.set_layer(e, layer);
            self.roots.push(e);
            self.roots.push(w);
        }
        fader.add(specs);
        self.triangles += tris;
        engine::log_info!("places: {name} built ({} buildings, {tris} triangles) in {:.1} ms", self.buildings, engine::log::now_ms() - t0);
    }

    /// Everything off (leaving the play screen).
    pub fn clear(&mut self, ctx: &mut Ctx) {
        for e in self.roots.drain(..) {
            ctx.despawn(e);
        }
        for h in self.colliders.drain(..) {
            ctx.physics.world.remove_collider(h);
        }
        *self = Self::default();
    }

    pub fn describe(&self) -> Value {
        json!({"built": self.built, "buildings": self.buildings, "triangles": self.triangles, "trees": self.trees})
    }
}

type MaterialIdOf = engine::renderer::MaterialId;
