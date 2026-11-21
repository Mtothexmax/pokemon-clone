//! Route 1 north and Stonecrest (the three.js round 2, `town.ts` buildNorth / buildFountain, via the Kaigen port
//! `st_north.c`): the dirt trail through the forest, the creek with its bridge, three more patches of tall grass, the
//! town below the old castle with its six houses, the fountain on the square, Sir Bricksworth and Marla's spots, the
//! route and town signs, the walls along the edges. Coordinates in studs (x east, z south) as in `town.rs`.
//!
//! Everything here lies north of the village plate (z < -47) and goes onto the band layers of `town::layer_at`, so
//! only the bands near you are drawn (`town::bands_near`).

use super::*;

/// LOD distance factor of the forest trees behind the walls (`Prop::lod`).
const FOREST_LOD: f32 = 0.2;

/// Kaigen `north_near_creek`: no trees right at the creek.
fn near_creek(z: f32) -> bool {
    z > CREEK.z0 - 2.5 && z < CREEK.z1 + 2.5
}

impl Town {
    /// A road plate (0.4 high, solid so you step onto it) with its studs, none where an earlier road has them.
    pub(crate) fn road(&mut self, i: usize) {
        let rd = ROADS[i];
        self.here((rd.x0 + rd.x1) * 0.5, (rd.z0 + rd.z1) * 0.5);
        self.b(C_GRAY).block(m((rd.x0 + rd.x1) * 0.5, 0.0, (rd.z0 + rd.z1) * 0.5), m(rd.x1 - rd.x0, 0.4, rd.z1 - rd.z0));
        self.solids.push(Solid { r: rd, y0: 0.0, h: 0.4 * STUD });
        let mut x = rd.x0 + 0.5;
        while x < rd.x1 {
            let mut z = rd.z0 + 0.5;
            while z < rd.z1 {
                if !ROADS[..i].iter().any(|o| x > o.x0 && x < o.x1 && z > o.z0 && z < o.z1) {
                    self.stud(C_GRAY, x, 0.4, z);
                }
                z += 1.0;
            }
            x += 1.0;
        }
    }

    /// A sign on a post with its painted board (as the Route 1 sign by the village gap).
    fn north_sign(&mut self, x: f32, z: f32, board: u32, glb: &str) {
        self.brick(C_BROWN, 0.35, 3.2, 0.35, x, 0.0, z - 0.2, false);
        self.sign_board(x, z, 3.4, 1.8, 1.3, board);
        self.prop(glb, x, z + crate::town::SIGN_FACE, 2.2, 1.0, 0.0);
        self.block(x - 0.5, x + 0.5, z - 0.7, z + 0.3);
    }

    /// The fountain on the square (buildFountain): a hollow stone basin with a tiled floor, a pillar, a bowl on top
    /// with its nozzle. The water, the jet, the falling streams, splashes and ripples are `fountain.rs`.
    fn fountain(&mut self, x: f32, z: f32) {
        let ring = |r0: f32, r1: f32, r2: f32, h: f32| [Vec2::new(r0, 0.0), Vec2::new(r1, 0.0), Vec2::new(r2, h), Vec2::new(r0, h), Vec2::new(r0, 0.0)];
        // the basin's wall and its lighter rim (studs -> metres in the lathe)
        let s = |v: [Vec2; 5]| v.map(|p| p * STUD);
        self.here(x, z);
        self.b(C_GRAY).with(Mat4::from_translation(m(x, 0.4, z)), |b| {
            b.lathe(&s(ring(FOUNTAIN_IN, 3.2, 3.05, 1.0)), 28);
        });
        self.b(0xc4c8cb).with(Mat4::from_translation(m(x, 1.4, z)), |b| {
            b.lathe(&s(ring(FOUNTAIN_IN - 0.08, 3.25, 3.25, 0.3)), 28);
        });
        // the floor inside: sand-blue tiles the water tints
        self.cyl(0x5b86a6, x, 0.4, z, FOUNTAIN_IN + 0.02, FOUNTAIN_IN + 0.02, 0.35, 28);
        self.cyl(C_GRAY, x, 0.75, z, 0.6, 0.45, 2.8, 16);
        // the bowl: hollow, the water in it overflows its rim
        self.b(C_GRAY).with(Mat4::from_translation(m(x, 3.55, z)), |b| {
            b.lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.6, 0.0), Vec2::new(BOWL_R, 0.5), Vec2::new(BOWL_R - 0.14, 0.5), Vec2::new(0.55, 0.25), Vec2::new(0.0, 0.25)].map(|p| p * STUD), 20);
        });
        // the nozzle in the middle of the bowl
        self.cyl(C_DARK_GRAY, x, 3.75, z, 0.28, 0.2, 0.75, 10);
        self.block(x - 3.3, x + 3.3, z - 3.3, z + 3.3);
    }

    /// The north gate (OPEN_WORLD.md milestone 1): two stone pillars and a lintel in Stonecrest's north wall where the
    /// trail leaves for the island. The doors and the blocker between the pillars are the play screen's (they open).
    fn north_gate(&mut self) {
        use crate::island::{GATE_HALF, GATE_X, GATE_Z};
        for s in [-1.0f32, 1.0] {
            let x = GATE_X + s * (GATE_HALF + 1.0);
            self.brick(0x8a8f93, 2.2, 0.4, 2.2, x, 0.0, GATE_Z, false);
            for k in 0..6 {
                let hex = if k % 2 == 0 { C_GRAY } else { 0x8a8f93 };
                self.brick(hex, 2.0, 1.2, 2.0, x, 0.4 + k as f32 * 1.2, GATE_Z, false);
            }
            self.brick(C_DARK_GRAY, 2.4, 0.4, 2.4, x, 7.6, GATE_Z, true);
            self.block(x - 1.1, x + 1.1, GATE_Z - 1.1, GATE_Z + 1.1);
        }
        // the lintel over the opening, with a red banner plate on its south face
        self.brick(C_GRAY, GATE_HALF * 2.0 + 0.4, 1.2, 1.6, GATE_X, 6.4, GATE_Z, false);
        self.brick(C_RED, 3.0, 0.8, 0.2, GATE_X, 6.6, GATE_Z + 0.95, false);
    }

    pub(crate) fn build_north(&mut self) {
        // ---- the three patches of tall grass up the route (the first one is the village's) ----
        let greens = [0x58ab41, C_GREEN, 0x7fc04a, 0x3c8c3a];
        let mut n = 0;
        for r in GRASSES.iter().skip(1) {
            let mut x = r.x0 + 0.5;
            while x < r.x1 {
                let mut z = r.z0 + 0.5;
                while z < r.z1 {
                    let (jx, jz, s) = ((self.rand() - 0.5) * 0.4, (self.rand() - 0.5) * 0.4, 0.8 + self.rand() * 0.5);
                    self.grass_tuft(x + jx, z + jz, s, greens[n % 4]);
                    n += 1;
                    z += 1.05;
                }
                x += 1.05;
            }
        }

        // ---- the creek across the forest: a dark bed, stone banks (the water is a surface: `build`); only the
        // bridge crosses it ----
        let (cx, cz) = ((CREEK.x0 + CREEK.x1) * 0.5, (CREEK.z0 + CREEK.z1) * 0.5);
        let cw = CREEK.x1 - CREEK.x0;
        self.brick(0x1d4a5e, cw, 0.02, CREEK.z1 - CREEK.z0, cx, 0.0, cz, false);
        self.brick(C_GRAY, cw, 0.35, 0.6, cx, 0.0, CREEK.z0 - 0.3, true);
        self.brick(C_GRAY, cw, 0.35, 0.6, cx, 0.0, CREEK.z1 + 0.3, true);
        self.block(CREEK.x0, BRIDGE.x0 + 0.4, CREEK.z0 - 0.6, CREEK.z1 + 0.6);
        self.block(BRIDGE.x1 - 0.4, CREEK.x1, CREEK.z0 - 0.6, CREEK.z1 + 0.6);
        // the bridge: a deck you walk onto (0.5 high, solid as the roads), planks, posts and rails
        {
            let (bw, bd, bz) = (BRIDGE.x1 - BRIDGE.x0, BRIDGE.z1 - BRIDGE.z0, (BRIDGE.z0 + BRIDGE.z1) * 0.5);
            self.brick(0x8a5a32, bw, 0.5, bd, 0.0, 0.0, bz, false);
            self.solids.push(Solid { r: BRIDGE, y0: 0.0, h: 0.5 * STUD });
            let mut z = BRIDGE.z0 + 0.5;
            while z < BRIDGE.z1 {
                self.brick(0x6b4426, bw - 0.2, 0.06, 0.12, 0.0, 0.5, z, false);
                z += 1.0;
            }
            for s in [-1.0f32, 1.0] {
                let x = s * (bw * 0.5 - 0.2);
                for z in [BRIDGE.z0 + 0.3, bz, BRIDGE.z1 - 0.3] {
                    self.brick(0x6b4426, 0.35, 1.6, 0.35, x, 0.5, z, false);
                }
                self.brick(0x8a5a32, 0.3, 0.25, bd, x, 1.75, bz, false);
                self.block(x - 0.25, x + 0.25, BRIDGE.z0, BRIDGE.z1); // the rails keep you on the deck
            }
        }

        // ---- the two signs ----
        self.north_sign(5.5, -49.2, 0xfff6c8, "assets/signs/north.glb");
        self.north_sign(5.5, -108.6, C_WHITE, "assets/signs/stonecrest.glb");

        // ---- the forest along the route: spruces and round trees beside it, a scatter of trees and bushes inside ----
        // (beyond the walls: simpler LOD levels from a few metres on; the camera looks down on them from ~16 m)
        self.prop_lod = FOREST_LOD;
        let mut z = -48.5f32;
        while z >= -110.0 {
            let mut ax = 21.5f32;
            while ax <= 55.0 {
                for s in [-1.0f32, 1.0] {
                    let x = s * ax + (self.rand() - 0.5) * 1.2;
                    if x < NORTH_PLATE.x0 + 1.0 || near_creek(z) || (ax > 30.0 && self.rand() < 0.25) {
                        continue;
                    }
                    let zz = z + (self.rand() - 0.5) * 1.2;
                    if self.rand() < 0.3 {
                        let sc = 6.0 + self.rand() * 1.5;
                        self.round_tree(x, zz, sc);
                    } else {
                        let sc = 8.5 + self.rand() * 2.5;
                        self.spruce(x, zz, sc);
                    }
                }
                ax += if ax < 30.0 { 3.6 } else { 4.6 };
            }
            z -= 3.6;
        }
        self.prop_lod = 0.0;
        {
            // (the three.js seeded scatter: Park-Miller from 7)
            let mut seed: u64 = 7;
            let mut r01 = || {
                seed = seed * 16807 % 2147483647;
                seed as f32 / 2147483647.0
            };
            let mut z = -50.0f32;
            while z >= -108.0 {
                let mut x = -17.0f32;
                while x <= 17.0 {
                    let px = x + (r01() - 0.5) * 2.0;
                    let pz = z + (r01() - 0.5) * 2.0;
                    x += 5.5;
                    if px.abs() < 6.0 || near_creek(pz) {
                        continue;
                    }
                    let by_grass = GRASSES[1..].iter().any(|g| px > g.x0 - 2.5 && px < g.x1 + 2.5 && pz > g.z0 - 2.5 && pz < g.z1 + 2.5);
                    if by_grass || ((px - 5.5).abs() < 3.0 && ((pz + 49.2).abs() < 4.0 || (pz + 108.6).abs() < 4.0)) {
                        continue;
                    }
                    if r01() < 0.45 {
                        continue;
                    }
                    let k = r01();
                    if k < 0.5 {
                        self.round_tree(px, pz, 6.0 + k * 1.5);
                        self.block(px - 1.3, px + 1.3, pz - 1.3, pz + 1.3);
                    } else {
                        self.bush(px, pz, 2.2 + k * 0.6);
                        self.block(px - 1.1, px + 1.1, pz - 1.1, pz + 1.1);
                    }
                }
                z -= 5.5;
            }
        }
        // around Stonecrest (behind its walls: simpler LODs too)
        self.prop_lod = FOREST_LOD;
        let mut z = -112.0f32;
        while z >= -192.0 {
            let (x, s) = (-44.2 + (self.rand() - 0.5) * 1.2, 8.5 + self.rand() * 2.5);
            self.spruce(x, z, s);
            let (x, s) = (53.5 + (self.rand() - 0.5) * 1.2, 8.5 + self.rand() * 2.5);
            self.spruce(x, z + 1.8, s);
            z -= 3.6;
        }
        // (the row's random numbers are drawn for the trees left out at the north gate too, so the rest stays as it was)
        let at_gate = |x: f32, front: bool| (x - crate::island::GATE_X).abs() < crate::island::GATE_HALF + if front { 6.0 } else { 3.2 };
        let mut x = -40.0f32;
        while x <= 50.0 {
            let (xx, s) = (x + (self.rand() - 0.5) * 1.2, 8.5 + self.rand() * 2.5);
            if !at_gate(xx, false) {
                self.spruce(xx, -191.5, s);
            } else {
                let _ = self.rot(); // (the tree's turn)
            }
            if !(-24.0..=24.0).contains(&x) {
                let (xx, s) = (x + 1.8 + (self.rand() - 0.5) * 1.2, 8.5 + self.rand() * 2.5);
                if !at_gate(xx, true) {
                    self.spruce(xx, -187.5, s);
                } else {
                    let _ = self.rot();
                }
            }
            x += 3.6;
        }
        self.north_gate();

        self.prop_lod = 0.0;

        // ---- Stonecrest: the main street, the cross street, the square ----
        for i in 7..ROADS.len() {
            self.road(i);
        }
        // six houses (their doors lead nowhere), the fountain, the old castle
        let houses: [(f32, f32, f32, f32, u32, u32, u32, &str); 6] = [
            (-17.0, -116.0, 12.0, 8.0, C_RED, C_TAN, C_BLUE, "bakery"),
            (17.0, -116.0, 12.0, 8.0, C_BLUE, C_WHITE, C_RED, "inn"),
            (33.0, -116.0, 10.0, 8.0, C_DARK_GREEN, C_YELLOW, C_RED_BROWN, "shop"),
            (-22.0, -137.0, 12.0, 9.0, C_ORANGE, C_WHITE, C_DARK_GREEN, "market"),
            (22.0, -137.0, 12.0, 9.0, C_RED_BROWN, C_GRAY, C_YELLOW, "smith"),
            (36.0, -137.0, 9.0, 8.0, C_BLUE, C_TAN, C_RED, "post"),
        ];
        for (x, z, w, d, roof, wall, door, sign) in houses {
            let path = format!("assets/signs/house_{sign}.glb");
            self.group(|t| t.house(x, z, w, d, roof, wall, door, C_RED, &path));
        }
        self.fountain(FOUNTAIN[0], FOUNTAIN[1]);
        {
            // the castle (the three.js "lego city" voxel model), turned by 90 degrees
            let (cx, cz, sc) = (0.0, -166.0, 1.6);
            let (hx, hz) = (27.7 * sc * 0.5, 21.9 * sc * 0.5);
            self.prop("assets/props/castle.glb", cx, cz, 0.0, sc, std::f32::consts::FRAC_PI_2);
            self.block_h(cx - hx, cx + hx, cz - hz, cz + hz, 12.0);
        }

        // ---- where you can walk: Route 1 (x -20..20) and Stonecrest (x -42..48), walls along their edges ----
        for w in [
            [-21.0, -20.0, -112.0, -41.6], // Route 1, west
            [20.0, 21.0, -112.0, -41.6],   //          east
            [-20.0, -9.4, -42.6, -41.6],   //          south, beside the first patch
            [9.4, 20.0, -42.6, -41.6],
            [-43.0, -42.0, -191.0, -111.0], // Stonecrest, west
            [48.0, 49.0, -191.0, -111.0],   //             east
            [-42.0, crate::island::GATE_X - crate::island::GATE_HALF - 1.0, -191.0, -190.0], // north, beside the gate
            [crate::island::GATE_X + crate::island::GATE_HALF + 1.0, 48.0, -191.0, -190.0],
            [-42.0, -20.0, -112.0, -111.0], //             south, beside the route
            [20.0, 48.0, -112.0, -111.0],
        ] {
            self.block(w[0], w[1], w[2], w[3]);
        }
    }
}
