//! Brickhollow, the LEGO village, built in code. A port of the three.js `town.ts` (via the Kaigen
//! port `st_town.c`): every coordinate below is in STUDS like the original (1 stud = [`STUD`] m,
//! a brick 1.2 studs high, a plate 0.4); x east, z south, house fronts face +z.
//!
//! Static bricks go into one `MeshBuilder` per 32x32-stud chunk with one part per colour, so each
//! chunk is a few meshes that the GPU culls on its own. Studs are instanced entities (one mesh),
//! LEGO plant / sign models are glTF instances (cached per URL), colliders are rapier cuboids.

use engine::geom::MeshBuilder;
use engine::physics::rapier::prelude::ColliderBuilder;
use engine::prelude::*;
use engine::renderer::{AlphaMode, Material, MaterialId, MeshId};
use std::collections::{BTreeMap, HashMap};

#[path = "rooms.rs"]
pub mod rooms;
#[path = "north.rs"]
mod north;

/// Metres per stud: the minifig (4.9 studs) is then 1.86 m.
pub const STUD: f32 = 0.38;
const STUD_H: f32 = 0.17;
const STUD_R: f32 = 0.3;
const CHUNK: f32 = 32.0;
/// The HLOD cells of the village's bricks (m; two chunks) and the camera distance beyond which a cell is its one
/// simplified mesh (m from the cell's centre, times the LOD distance scale).
const HLOD_CELL: f32 = 2.0 * CHUNK * STUD;
const HLOD_DISTANCE: f32 = 60.0;

// the LEGO palette (sRGB)
pub const C_GREEN: u32 = 0x4b9f4a;
pub const C_DARK_GREEN: u32 = 0x237841;
pub const C_GRAY: u32 = 0xa0a5a9;
pub const C_DARK_GRAY: u32 = 0x6c6e68;
pub const C_RED: u32 = 0xc91a09;
pub const C_BLUE: u32 = 0x0055bf;
pub const C_ORANGE: u32 = 0xfe8a18;
pub const C_TAN: u32 = 0xe4cd9e;
pub const C_WHITE: u32 = 0xf4f4f4;
pub const C_BROWN: u32 = 0x583927;
pub const C_RED_BROWN: u32 = 0x82422a;
pub const C_YELLOW: u32 = 0xf2cd37;
pub const C_GLASS: u32 = 0x9fd8ef;
pub const C_WATER: u32 = 0x2a7fd4;
pub const C_BLACK: u32 = 0x1b2a34;
pub const C_PINK: u32 = 0xff8ac0;

/// Studs -> metres.
pub fn m(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x * STUD, y * STUD, z * STUD)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub x1: f32,
    pub z0: f32,
    pub z1: f32,
}

impl Rect {
    pub const fn new(x0: f32, x1: f32, z0: f32, z1: f32) -> Self {
        Self { x0, x1, z0, z1 }
    }
    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.x0 && x <= self.x1 && z >= self.z0 && z <= self.z1
    }
}

/// The grey road plates (0.4 high, studs on top), as the three.js `roads` list.
pub const ROADS: [Rect; 10] = [
    Rect::new(-3.0, 3.0, -27.0, 24.0),
    Rect::new(-26.0, 29.0, -2.0, 3.0),
    Rect::new(26.0, 29.0, 3.0, 10.0),
    Rect::new(29.0, 66.0, 7.0, 10.0),
    Rect::new(-16.0, -14.0, -6.0, -2.0),
    Rect::new(3.0, 15.0, 16.5, 19.5),
    Rect::new(-20.0, -18.0, 3.0, 7.0),
    // Stonecrest: the main street, the cross street, the square around the fountain
    Rect::new(-3.0, 3.0, -148.0, -111.0),
    Rect::new(-30.0, 40.0, -127.0, -122.0),
    Rect::new(-10.0, 10.0, -147.0, -133.0),
];

/// All tall grass: the first patch north of the village and three more along Route 1.
pub const GRASSES: [Rect; 4] = [
    Rect::new(-8.6, 8.6, -42.0, -28.2),
    Rect::new(-17.0, -4.5, -64.0, -50.0),
    Rect::new(4.5, 18.0, -92.0, -80.0),
    Rect::new(-16.0, -4.5, -106.0, -94.0),
];

/// The fountain on Stonecrest's square (studs): centre, the basin's inner radius, its water level, the bowl on top
/// (rim radius, water level), the nozzle's top.
pub const FOUNTAIN: [f32; 2] = [0.0, -140.0];
pub const FOUNTAIN_IN: f32 = 2.7;
pub const FOUNTAIN_WATER: f32 = 1.28;
pub const BOWL_R: f32 = 1.3;
pub const BOWL_WATER: f32 = 4.0;
pub const NOZZLE_TOP: f32 = 4.5;

pub const POND: Rect = Rect::new(-12.0, -4.0, 15.0, 22.0);

// ---- Route 1 north and Stonecrest (the three.js round 2: town.ts buildNorth, Kaigen st_north.c) ----
/// The ground plate of the route and the town (the village's plate ends at z -47).
pub const NORTH_PLATE: Rect = Rect::new(-46.0, 56.0, -194.0, -47.0);
/// The creek across the route; only the bridge crosses it.
pub const CREEK: Rect = Rect::new(-46.0, 56.0, -75.0, -71.0);
pub const BRIDGE: Rect = Rect::new(-3.4, 3.4, -76.2, -69.8);
/// The dirt trail from the first patch of grass to the town.
pub const TRAIL: Rect = Rect::new(-2.5, 2.5, -111.0, -42.0);
/// The trail from Stonecrest's square around the castle to the north gate (east, then north).
pub const GATE_TRAILS: [Rect; 2] = [Rect::new(10.0, 33.0, -146.0, -143.0), Rect::new(33.0, 37.0, -194.0, -143.0)];
/// Stonecrest begins north of here, Route 1 north of -27 (three.js `zoneLabel`).
pub const STONECREST_Z: f32 = -111.0;
pub const ROUTE_Z: f32 = -27.0;
/// The zone you are in outside: 0 Brickhollow, 1 Route 1, 2 Stonecrest, 3 the island north of it.
pub fn zone_at(z: f32) -> usize {
    if z < crate::island::SOUTH_Z {
        3
    } else if z < STONECREST_Z {
        2
    } else if z < ROUTE_Z {
        1
    } else {
        0
    }
}
pub const ZONE_NAMES: [&str; 4] = ["Brickhollow", "Route 1", "Stonecrest", "Brick Isle"];

/// Render layers of the land north of the village, in bands of z (studs, south edge first): only the bands
/// near you are drawn (`bands_near`), the village's own layer stops at the plate's edge (z -47).
pub const NORTH_BANDS: [(f32, f32, u8); 4] = [(-79.0, -47.0, 6), (-111.0, -79.0, 7), (-143.0, -111.0, 9), (-210.0, -143.0, 10)];
pub const NORTH_EDGE: f32 = -47.0;
/// The render layer of something outside at (x, z) studs (the rooms far east keep theirs: groups).
pub fn layer_at(x: f32, z: f32) -> u8 {
    if z < crate::island::SOUTH_Z {
        return crate::island::LAYER_NEAR;
    }
    if x >= 150.0 || z >= NORTH_EDGE {
        return rooms::VILLAGE_LAYER;
    }
    NORTH_BANDS.iter().find(|b| z >= b.0).map(|b| b.2).unwrap_or(NORTH_BANDS[3].2)
}
/// The outdoor layers to draw with you at z (studs): every band within `north` studs ahead (north) and
/// `south` behind (the top-down camera sees ~22 studs ahead and ~14 behind you; shadows reach further).
pub fn bands_near(z: f32, north: f32, south: f32) -> u32 {
    let (lo, hi) = (z - north, z + south);
    let mut m = if hi >= NORTH_EDGE - 1.0 { 1u32 << rooms::VILLAGE_LAYER } else { 0 };
    for b in NORTH_BANDS {
        if b.1 >= lo && b.0 <= hi {
            m |= 1 << b.2;
        }
    }
    m
}
/// Every outdoor layer (the camera at your eyes looks far).
pub fn all_bands() -> u32 {
    NORTH_BANDS.iter().fold(1u32 << rooms::VILLAGE_LAYER, |m, b| m | (1 << b.2))
}
pub fn on_road(x: f32, z: f32) -> bool {
    ROADS.iter().any(|r| r.contains(x, z))
}

/// A villager standing in the town (three.js `NPCS`): id, figure style folder, studs, yaw.
pub struct NpcDef {
    pub id: &'static str,
    pub style: &'static str,
    pub x: f32,
    pub z: f32,
    pub rot: f32,
}

pub const NPCS: [NpcDef; 5] = [
    NpcDef { id: "rival", style: "dex", x: 52.5, z: -3.4, rot: -std::f32::consts::FRAC_PI_2 },
    NpcDef { id: "kid", style: "pip", x: -8.0, z: 13.2, rot: 0.0 },
    // Prof. Quill guarding the way north (until you have a Voxelmon)
    NpcDef { id: "gate-prof", style: "quill", x: 0.0, z: -24.4, rot: 0.0 },
    // Stonecrest: Sir Bricksworth on the square below the castle, Marla by the bakery
    NpcDef { id: "knight", style: "knight", x: 7.0, z: -146.2, rot: 0.0 },
    NpcDef { id: "baker", style: "baker", x: -8.0, z: -110.6, rot: 0.0 },
];
/// Where Prof. Quill stands at the gap north: in the middle while it is closed, aside once it is open (studs, yaw).
pub const GATE_PROF_CLOSED: [f32; 3] = [0.0, -24.4, 0.0];
pub const GATE_PROF_OPEN: [f32; 3] = [-5.4, -24.2, 0.4];

/// Deterministic random stream (the three.js version used Math.random; the layout only needs to look random).
pub struct TownRng(u32);
impl TownRng {
    pub fn f(&mut self) -> f32 {
        crate::lcg_f(&mut self.0)
    }
}

/// One glTF model instance to spawn.
pub struct Prop {
    pub path: String,
    pub pos: Vec3,
    pub scale: f32,
    pub rot: f32,
    /// -1 the village, else the room (`rooms::RoomId`) it stands in.
    pub area: i8,
    /// > 0: automatic LODs, their switch distances times this (the forest beside Route 1); 0: full detail
    pub lod: f32,
}

/// A static collider: rect in studs, height in metres (from the ground).
#[derive(Clone, Copy)]
pub struct Solid {
    pub r: Rect,
    pub y0: f32,
    pub h: f32,
}

/// What the town build produced (counts for `debug_state` and tests).
#[derive(Default, Clone, serde::Serialize)]
pub struct TownStats {
    pub chunks: usize,
    pub meshes: usize,
    pub triangles: usize,
    pub studs: usize,
    pub props: usize,
    pub colliders: usize,
    pub colours: usize,
    pub occluders: usize,
    /// the HLOD cells of the bricks and their triangles
    pub hlods: usize,
    pub hlod_triangles: usize,
}

/// Something tall that fades out while it hides the hero (see `fade.rs`).
pub enum OccluderSpec {
    /// Built bricks (a house, the lab): their root, extra entities (studs) and world bounds.
    Bricks { root: Entity, extra: Vec<Entity>, min: Vec3, max: Vec3 },
    /// A glTF model (a tree): its root; bounds come from its meshes once loaded.
    Model { root: Entity },
}

pub struct Town {
    rng: TownRng,
    chunks: BTreeMap<(i32, i32, u8), MeshBuilder>,
    /// chunk the next geometry goes to (of the current thing's centre) and its render layer
    at: (i32, i32, u8),
    used: Vec<Rect>,
    pub solids: Vec<Solid>,
    pub props: Vec<Prop>,
    pub studs: Vec<(u32, Vec3, Option<usize>)>,
    box_n: u32,
    /// Buildings with their own meshes (so they can fade), and the one being built.
    groups: Vec<MeshBuilder>,
    group: Option<usize>,
    /// The area (-1 village, else `rooms::RoomId`) of each group, and of what is built now.
    group_area: Vec<i8>,
    area: i8,
    /// the LOD factor of the props placed now (`Prop::lod`)
    prop_lod: f32,
    /// the tall grass tufts' blades (instanced, see grass.rs)
    pub tufts: Vec<TuftSpec>,
}

/// One tuft of tall grass: its blades' pivot (metres, the top of the round plate), turn, size, colour.
#[derive(Clone, Copy, Debug)]
pub struct TuftSpec {
    pub pos: Vec3,
    pub yaw: f32,
    pub scale: f32,
    pub hex: u32,
}

/// The blades of a LEGO grass stem (six leaning blades and a centre one) around (0, 0, 0), turn 0, size 1
/// (studs as `grass_tuft` built them; the plate stays in the static bricks).
pub fn tuft_mesh() -> engine::mesh::MeshData {
    let mut mb = MeshBuilder::new();
    mb.part("blades");
    for i in 0..7 {
        let a = i as f32 * 0.9;
        let lean = if i == 6 { 0.0 } else { 0.55 + 0.15 * (i % 2) as f32 };
        let h = if i == 6 { 2.4 } else { 2.0 } - 0.25;
        let tip = Vec3::new(a.cos() * lean, h, a.sin() * lean) * STUD;
        let profile = [Vec2::new(0.17 * STUD, 0.0), Vec2::new(0.0, tip.length())];
        mb.with(Mat4::from_quat(y_to(tip)), |b| {
            b.lathe(&profile, 3);
        });
    }
    mb.finish().into_iter().next().unwrap().1
}

fn part(hex: u32) -> String {
    format!("c{hex:06x}")
}

fn shade(hex: u32, k: f32) -> u32 {
    let c = |s: u32| (((hex >> s) & 255) as f32 * k).min(255.0) as u32;
    (c(16) << 16) | (c(8) << 8) | c(0)
}

/// Rotation that turns +Y into `dir`.
fn y_to(dir: Vec3) -> Quat {
    Quat::from_rotation_arc(Vec3::Y, dir.normalize_or(Vec3::Y))
}

impl Town {
    fn new() -> Self {
        Self { rng: TownRng(20261114), chunks: BTreeMap::new(), at: (0, 0, rooms::VILLAGE_LAYER), used: vec![], solids: vec![], props: vec![], studs: vec![], box_n: 0, groups: vec![], group: None, group_area: vec![], area: -1, prop_lod: 0.0, tufts: vec![] }
    }

    fn rand(&mut self) -> f32 {
        self.rng.f()
    }
    fn rot(&mut self) -> f32 {
        self.rand() * std::f32::consts::TAU
    }

    fn here(&mut self, x: f32, z: f32) {
        self.at = ((x / CHUNK).floor() as i32, (z / CHUNK).floor() as i32, layer_at(x, z));
    }

    fn b(&mut self, hex: u32) -> &mut MeshBuilder {
        let mb = match self.group {
            Some(g) => &mut self.groups[g],
            None => self.chunks.entry(self.at).or_insert_with(MeshBuilder::new),
        };
        mb.part(&part(hex));
        mb
    }

    /// One stud on top of a brick at (x, y, z), y = the brick's top.
    fn stud(&mut self, hex: u32, x: f32, y: f32, z: f32) {
        self.studs.push((hex, m(x, y, z), self.group));
    }

    /// Builds `f` into its own group of meshes (a building that fades while it hides the hero).
    fn group(&mut self, f: impl FnOnce(&mut Self)) {
        self.groups.push(MeshBuilder::new());
        self.group_area.push(self.area);
        self.group = Some(self.groups.len() - 1);
        f(self);
        self.group = None;
    }

    /// Studs on a brick's top face, centred like real LEGO (one per full stud of its size).
    fn stud_rect(&mut self, hex: u32, x0: f32, x1: f32, z0: f32, z1: f32, y: f32) {
        if x1 - x0 < 0.7 || z1 - z0 < 0.7 {
            return;
        }
        let nx = (x1 - x0 + 0.05).floor().max(1.0) as i32;
        let nz = (z1 - z0 + 0.05).floor().max(1.0) as i32;
        let (ox, oz) = (x0 + (x1 - x0 - nx as f32) * 0.5 + 0.5, z0 + (z1 - z0 - nz as f32) * 0.5 + 0.5);
        for i in 0..nx {
            for k in 0..nz {
                self.stud(hex, ox + i as f32, y, oz + k as f32);
            }
        }
    }

    /// A brick: w x h x d studs, centred on x/z, its BOTTOM at y (`box()` in the three.js town).
    pub fn brick(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32, studs: bool) {
        // bricks that touch would share faces exactly and flicker: every brick is a few hundredths of a
        // stud smaller or larger than the last, so no two faces lie in one plane (a step of 0.012 studs moves
        // a face 2.3 mm: more than the z-fighting check's 2 mm)
        self.box_n += 1;
        let e = 0.012 * ((self.box_n % 7) as i32 - 3) as f32;
        let e2 = 0.012 * ((self.box_n * 3 % 7) as i32 - 3) as f32;
        self.here(x, z);
        self.b(hex).block(m(x, y + e2 * 0.5, z), m(w + e, h + e2, d - e));
        if studs {
            self.stud_rect(hex, x - w * 0.5, x + w * 0.5, z - d * 0.5, z + d * 0.5, y + h);
        }
    }

    /// A rotated brick (centre, euler radians XYZ).
    fn brick_rot(&mut self, hex: u32, size: Vec3, c: Vec3, euler: Vec3) {
        self.here(c.x, c.z);
        let t = Mat4::from_rotation_translation(Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z), m(c.x, c.y, c.z));
        let s = m(size.x, size.y, size.z) * 0.5;
        self.b(hex).with(t, |b| {
            b.cuboid(-s, s);
        });
    }

    /// A cylinder standing on y (bottom), radii and height in studs.
    fn cyl(&mut self, hex: u32, x: f32, y: f32, z: f32, rb: f32, rt: f32, h: f32, seg: u32) {
        self.here(x, z);
        let profile = [Vec2::new(0.0, 0.0), Vec2::new(rb * STUD, 0.0), Vec2::new(rt * STUD, h * STUD), Vec2::new(0.0, h * STUD)];
        self.b(hex).with(Mat4::from_translation(m(x, y, z)), |b| {
            b.lathe(&profile, seg);
        });
    }

    /// A cone from `base` (radius r, metres) to `tip`.
    fn cone(&mut self, hex: u32, base: Vec3, tip: Vec3, r: f32, seg: u32) {
        let d = tip - base;
        let profile = [Vec2::new(0.0, 0.0), Vec2::new(r, 0.0), Vec2::new(0.0, d.length())];
        self.b(hex).with(Mat4::from_rotation_translation(y_to(d), base), |b| {
            b.lathe(&profile, seg);
        });
    }

    fn occupy(&mut self, x0: f32, x1: f32, z0: f32, z1: f32) {
        self.used.push(Rect::new(x0, x1, z0, z1));
    }

    /// A block `h` metres high nobody walks through (buildings are as high as their roofs).
    fn block_h(&mut self, x0: f32, x1: f32, z0: f32, z1: f32, h: f32) {
        self.occupy(x0, x1, z0, z1);
        self.solids.push(Solid { r: Rect::new(x0, x1, z0, z1), y0: 0.0, h });
    }

    /// A 2 m high blocker (the three.js "blockers").
    fn block(&mut self, x0: f32, x1: f32, z0: f32, z1: f32) {
        self.block_h(x0, x1, z0, z1, 2.0);
    }

    // ------------------------------------------------------------------------------- props

    /// The strip in front of each readable sign board, where the camera stands for the close-up:
    /// nothing grows there.
    fn in_sign_view(x: f32, z: f32) -> bool {
        let boards = [[5.9, 20.41, 0.0, 3.8], [5.0, -24.09, 0.0, 3.4], [-22.91, 8.9, std::f32::consts::FRAC_PI_2, 2.8], [13.0, 16.17, 0.0, 5.2], [5.5, -48.99, 0.0, 3.4], [5.5, -108.39, 0.0, 3.4]];
        boards.iter().any(|b: &[f32; 4]| {
            let (dx, dz, yaw) = (x - b[0], z - b[1], b[2]);
            let along = dx * yaw.sin() + dz * yaw.cos();
            let side = dx * yaw.cos() - dz * yaw.sin();
            along > -0.5 && along < 9.0 && side.abs() < b[3] * 0.5 + 1.2
        })
    }

    /// One model instance at (x, z) studs, standing on y studs, scaled by s, turned by rot.
    fn prop(&mut self, path: &str, x: f32, z: f32, y: f32, s: f32, rot: f32) {
        if !path.starts_with("assets/signs/") && y < 1.0 && Self::in_sign_view(x, z) {
            return;
        }
        self.props.push(Prop { path: path.to_string(), pos: m(x, y, z), scale: s, rot, area: self.area, lod: self.prop_lod });
    }

    fn petal(hex: u32) -> &'static str {
        match hex {
            C_RED => "red",
            C_YELLOW => "yellow",
            C_PINK => "pink",
            C_WHITE => "white",
            C_ORANGE => "orange",
            _ => "purple",
        }
    }

    /// The LEGO flower plant (2x2 leaves + 1x1 flower).
    fn flower(&mut self, x: f32, z: f32, hex: u32, y: f32) {
        let r = self.rot();
        self.prop(&format!("assets/props/flower_{}.glb", Self::petal(hex)), x, z, y, 1.0, r);
        self.occupy(x - 0.6, x + 0.6, z - 0.6, z + 0.6);
    }

    /// A "Lego Frog" plant: 0 leafy a, 1 leafy b, 2 sprout, 3 bouquet, 4 / 5 the flat stems.
    fn frog_plant(&mut self, variant: u32, x: f32, z: f32, petal: u32) {
        let s = 0.9 + self.rand() * 0.25;
        let r = self.rot();
        let colours = ["red", "yellow", "pink", "white", "orange", "purple"];
        let c = colours[petal as usize % 6];
        let path = match variant {
            0 => "assets/props/frog_leafy_a.glb".to_string(),
            1 => "assets/props/frog_leafy_b.glb".to_string(),
            2 => format!("assets/props/frog_sprout_{c}.glb"),
            3 => format!("assets/props/frog_bouquet_{c}.glb"),
            4 => format!("assets/props/frog_stem_a_{c}.glb"),
            _ => format!("assets/props/frog_stem_b_{c}.glb"),
        };
        self.prop(&path, x, z, 0.0, s, r);
    }

    /// LEGO bamboo leaves.
    fn leaves(&mut self, x: f32, z: f32, s: f32) {
        let r = self.rot();
        self.prop("assets/props/bamboo.glb", x, z, 0.0, s, r);
        self.occupy(x - 0.8, x + 0.8, z - 0.8, z + 0.8);
    }

    /// The LEGO flower pot (4x4), or only its flowers (the park).
    fn flower_pot(&mut self, x: f32, z: f32, pot: bool) {
        let r = self.rot();
        self.prop(if pot { "assets/props/flowerpot.glb" } else { "assets/props/flower_bunch.glb" }, x, z, 0.0, 1.0, r);
        self.block(x - 1.8, x + 1.8, z - 1.8, z + 1.8);
    }

    fn spruce(&mut self, x: f32, z: f32, s: f32) {
        let r = self.rot();
        self.prop("assets/props/spruce.glb", x, z, 0.0, s, r);
    }
    fn round_tree(&mut self, x: f32, z: f32, s: f32) {
        let r = self.rot();
        self.prop("assets/props/round_tree.glb", x, z, 0.0, s, r);
    }
    fn bush(&mut self, x: f32, z: f32, s: f32) {
        let r = self.rot();
        self.prop("assets/props/bush.glb", x, z, 0.0, s, r);
    }

    /// Is (x, z) free for a plant of radius r: inside the village, off the roads, every used rect and placed plant.
    fn free(&self, x: f32, z: f32, r: f32, taken: &[Vec3]) -> bool {
        if x < -26.2 + r || x > 66.2 - r || z < -25.6 + r || z > 23.4 - r {
            return false;
        }
        let hit = |q: &Rect| x > q.x0 - r && x < q.x1 + r && z > q.z0 - r && z < q.z1 + r;
        if ROADS.iter().any(hit) || self.used.iter().any(hit) {
            return false;
        }
        !taken.iter().any(|t| (t.x - x).powi(2) + (t.z - z).powi(2) < (t.y + r).powi(2))
    }

    /// The three.js `placeFrogPlants`: plants along the village's edge ring, then cypresses, ferns,
    /// small pines and reeds wherever there is room, outer areas first.
    fn wild_plants(&mut self) {
        let mut taken: Vec<Vec3> = vec![];
        for f in [[-3.0, 21.5, 0.0], [-13.4, 18.4, 3.0], [-24.2, 16.2, 1.0], [-24.0, 10.5, 2.0]] {
            if self.free(f[0], f[1], 1.4, &taken) {
                let p = (self.rand() * 6.0) as u32 % 6;
                self.frog_plant(f[2] as u32, f[0], f[1], p);
                taken.push(Vec3::new(f[0], 1.4, f[1]));
            }
        }
        // the edge ring, shuffled
        let mut cand = vec![];
        let mut x = -26.2 + 1.0;
        while x < 66.2 - 1.0 {
            let mut z = -25.6 + 1.0;
            while z < 23.4 - 1.0 {
                let edge = (x + 26.2f32).min(66.2 - x).min((z + 25.6f32).min(23.4 - z));
                if edge > 0.8 && edge < 7.0 {
                    let (jx, jz) = ((self.rand() - 0.5) * 0.6, (self.rand() - 0.5) * 0.6);
                    cand.push(Vec3::new(x + jx, 0.0, z + jz));
                }
                z += 1.3;
            }
            x += 1.3;
        }
        for i in (1..cand.len()).rev() {
            let j = (self.rand() * (i + 1) as f32) as usize % (i + 1);
            cand.swap(i, j);
        }
        let mut plants = 0;
        for c in cand.clone() {
            if plants >= 30 {
                break;
            }
            let v = (self.rand() * 4.0) as u32 % 4;
            let r = if v < 2 { 1.9 } else { 1.3 };
            if !self.free(c.x, c.z, r, &taken) {
                continue;
            }
            let p = (self.rand() * 6.0) as u32 % 6;
            self.frog_plant(v, c.x, c.z, p);
            taken.push(Vec3::new(c.x, r, c.z));
            plants += 1;
        }
        // reeds standing in the pond's corners
        for r in [[-11.2, 15.8], [-4.6, 15.9], [-11.3, 21.2], [-4.7, 21.3]] {
            let (s, rot) = (0.45 + self.rand() * 0.1, self.rot());
            self.prop("assets/props/reed.glb", r[0], r[1], 0.0, s, rot);
        }
        // the whole village, outer areas first: a key of edge distance plus noise (in y), taken in bands
        let mut cand = vec![];
        let mut x = -26.2 + 1.0;
        while x < 66.2 - 1.0 {
            let mut z = -25.6 + 1.0;
            while z < 23.4 - 1.0 {
                let edge = (x + 26.2f32).min(66.2 - x).min((z + 25.6f32).min(23.4 - z));
                let (jx, key, jz) = ((self.rand() - 0.5) * 0.5, edge + self.rand() * 4.0, (self.rand() - 0.5) * 0.5);
                cand.push(Vec3::new(x + jx, key, z + jz));
                z += 1.1;
            }
            x += 1.1;
        }
        // per kind: how many, radius, scale, collider half-size (0: walk through), shadow
        let kinds = [(6, 1.7, 0.8, 0.8, "cypress"), (4, 2.2, 0.3, 0.0, "fern"), (8, 1.6, 0.75, 0.6, "pine_small"), (10, 1.1, 0.5, 0.0, "reed")];
        for (want, r, sc, bl, name) in kinds {
            let mut k = 0;
            let mut band = 0.0;
            while band < 40.0 && k < want {
                for c in &cand {
                    if k >= want {
                        break;
                    }
                    if c.y < band || c.y >= band + 0.5 || !self.free(c.x, c.z, r, &taken) {
                        continue;
                    }
                    let (s, rot) = (sc * (0.9 + self.rand() * 0.2), self.rot());
                    self.prop(&format!("assets/props/{name}.glb"), c.x, c.z, 0.0, s, rot);
                    taken.push(Vec3::new(c.x, r, c.z));
                    if bl > 0.0 {
                        self.block(c.x - bl, c.x + bl, c.z - bl, c.z + bl);
                    }
                    k += 1;
                }
                band += 0.5;
            }
        }
    }

    /// A LEGO grass stem: a round 1x1 plate with six flat blades and a centre blade.
    fn grass_tuft(&mut self, x: f32, z: f32, s: f32, hex: u32) {
        if Self::in_sign_view(x, z) {
            return;
        }
        self.cyl(hex, x, 0.0, z, 0.4, 0.4, 0.3, 6);
        // the blades are one instanced mesh per tuft (`tuft_mesh`) that the engine bends (grass.rs)
        let a0 = self.rand() * std::f32::consts::TAU;
        self.tufts.push(TuftSpec { pos: m(x, 0.25, z), yaw: a0, scale: s, hex });
    }

    // ------------------------------------------------------------------------------- buildings

    /// A window with a white frame, glass and a mullion; `along_z` = in a wall facing +-z.
    fn window(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, along_z: bool, out: f32) {
        if along_z {
            self.brick(C_WHITE, w + 0.4, h + 0.4, 0.3, x, y - 0.2, z + out * 0.05, false);
            self.brick(C_GLASS, w - 0.2, h - 0.2, 0.3, x, y + 0.1, z + out * 0.2, false);
            self.brick(C_WHITE, 0.15, h, 0.2, x, y, z + out * 0.3, false);
        } else {
            self.brick(C_WHITE, 0.3, h + 0.4, w + 0.4, x + out * 0.05, y - 0.2, z, false);
            self.brick(C_GLASS, 0.3, h - 0.2, w - 0.2, x + out * 0.2, y + 0.1, z, false);
            self.brick(C_WHITE, 0.2, h, 0.15, x + out * 0.3, y, z, false);
        }
    }

    /// A brick wall (bottom on the 0.4 foundation) with a thin groove every brick (1.2 studs).
    fn wall(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, z: f32) {
        self.brick(hex, w, h, d, x, 0.4, z, false);
        let groove = shade(hex, 0.86);
        let mut y = 0.4 + 1.2;
        while y < 0.4 + h - 0.3 {
            self.brick(groove, w + 0.06, 0.07, d + 0.06, x, y - 0.035, z, false);
            y += 1.2;
        }
    }

    /// A two-storey village house (`buildHouse`).
    fn house(&mut self, cx: f32, cz: f32, w: f32, d: f32, roof: u32, wall: u32, door: u32, mailbox: u32, sign: &str) {
        let (x0, x1, z0, z1) = (cx - w * 0.5, cx + w * 0.5, cz - d * 0.5, cz + d * 0.5);
        self.block_h(x0 - 0.6, x1 + 0.6, z0 - 0.6, z1 + 0.5, 11.0 * STUD);
        self.brick(C_GRAY, w + 1.0, 0.4, d + 1.0, cx, 0.0, cz, false);
        const H: f32 = 7.2;
        self.wall(wall, w, H, 1.0, cx, z1 - 0.5);
        self.wall(wall, w, H, 1.0, cx, z0 + 0.5);
        self.wall(wall, 1.0, H, d - 2.0, x0 + 0.5, cz);
        self.wall(wall, 1.0, H, d - 2.0, x1 - 0.5, cz);
        self.brick(wall, w - 2.0, 0.4, d - 2.0, cx, H, cz, false);
        self.brick(C_WHITE, w + 0.2, 0.4, d + 0.2, cx, 3.8, cz, false);
        // door (front faces +z)
        self.brick(C_WHITE, 2.2, 4.0, 0.4, cx, 0.4, z1 + 0.1, false);
        self.brick(door, 1.8, 3.6, 0.4, cx, 0.4, z1 + 0.25, false);
        self.brick(C_YELLOW, 0.3, 0.3, 0.3, cx + 0.55, 2.05, z1 + 0.5, false);
        self.brick(C_DARK_GRAY, 3.0, 0.4, 1.2, cx, 0.4, z1 + 0.7, true);
        for s in [-1.0, 1.0] {
            let wx = cx + s * (w * 0.5 - 2.3);
            self.window(wx, 1.6, z1, 2.4, 1.8, true, 1.0);
            self.window(wx, 5.0, z1, 1.9, 1.6, true, 1.0);
            self.window(cx + s * w * 0.5, 5.0, cz, 2.4, 1.6, false, s);
            self.window(wx, 5.0, z0, 1.9, 1.6, true, -1.0);
        }
        // stepped brick roof
        let ov = 0.8;
        for i in 0..6 {
            let dd = d + ov * 2.0 - i as f32 * 1.9;
            if dd <= 0.6 {
                break;
            }
            let c = if i % 2 == 0 { roof } else { shade(roof, 0.92) };
            self.brick(c, w + ov * 2.0, 0.6, dd, cx, 0.4 + H + i as f32 * 0.6, cz, false);
        }
        self.brick(roof, w + ov * 2.0, 0.4, 1.0, cx, 0.4 + H + 6.0 * 0.6, cz, true);
        self.brick(C_RED_BROWN, 1.6, 3.4, 1.6, cx + w * 0.5 - 2.2, 0.4 + H + 0.6, cz - 1.5, true);
        // the sign board above the door
        let sw = 3.4f32.min(w - 7.4);
        let sh = sw / 4.0;
        self.brick(C_WHITE, sw + 0.3, sh + 0.3, 0.2, cx, 5.8 - sh * 0.5 - 0.15, z1, false);
        self.brick(mailbox, sw, sh, 0.1, cx, 5.8 - sh * 0.5, z1 + 0.12, false);
        self.prop(sign, cx, z1 + 0.19, 5.8, 1.0, 0.0);
        // mailbox
        let (mx, mz) = (cx - 3.2, z1 + 2.2);
        self.brick(C_DARK_GRAY, 0.4, 1.6, 0.4, mx, 0.0, mz, false);
        self.brick(mailbox, 1.0, 0.9, 1.4, mx, 1.6, mz, true);
        self.block(mx - 0.6, mx + 0.6, mz - 0.8, mz + 0.8);
    }

    /// Prof. Quill's lab (`buildLab`).
    fn lab(&mut self, cx: f32, cz: f32, w: f32, d: f32) {
        let (x0, x1, z0, z1) = (cx - w * 0.5, cx + w * 0.5, cz - d * 0.5, cz + d * 0.5);
        self.block_h(x0 - 0.6, x1 + 0.6, z0 - 0.6, z1 + 0.5, 8.6 * STUD);
        self.brick(C_GRAY, w + 1.0, 0.4, d + 1.0, cx, 0.0, cz, false);
        const H: f32 = 7.2;
        self.wall(C_TAN, w, H, 1.0, cx, z1 - 0.5);
        self.wall(C_TAN, w, H, 1.0, cx, z0 + 0.5);
        self.wall(C_TAN, 1.0, H, d - 2.0, x0 + 0.5, cz);
        self.wall(C_TAN, 1.0, H, d - 2.0, x1 - 0.5, cz);
        let fz = z1;
        self.brick(C_WHITE, 2.4, 4.2, 0.4, cx, 0.4, fz + 0.1, false);
        self.brick(C_DARK_GREEN, 2.0, 3.8, 0.4, cx, 0.4, fz + 0.25, false);
        self.brick(C_GLASS, 0.6, 1.2, 0.2, cx, 2.6, fz + 0.45, false);
        self.brick(C_DARK_GRAY, 3.4, 0.4, 1.4, cx, 0.4, fz + 0.8, true);
        for s in [-1.0, 1.0] {
            let wx = cx + s * 4.6;
            self.window(wx, 1.6, fz, 5.0, 1.6, true, 1.0);
            for i in 0..3 {
                let c = if i % 2 == 1 { C_WHITE } else { C_RED };
                self.brick(c, 5.4, 0.3, 0.6, wx, 3.6 + i as f32 * 0.25, fz + 0.4 + i as f32 * 0.35, false);
            }
        }
        // round windows
        for rx in [-6.0, -3.5, 3.5, 6.0] {
            let up = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            self.here(cx + rx, fz);
            for (hex, r, dz) in [(C_DARK_GRAY, 0.85, 0.1), (0x40b8d8, 0.65, 0.18)] {
                let profile = [Vec2::new(0.0, -0.15 * STUD), Vec2::new(r * STUD, -0.15 * STUD), Vec2::new(r * STUD, 0.15 * STUD), Vec2::new(0.0, 0.15 * STUD)];
                self.b(hex).with(Mat4::from_rotation_translation(up, m(cx + rx, 5.8, fz + dz)), |b| {
                    b.lathe(&profile, 14);
                });
            }
        }
        // flat roof with white tiles and a rim with studs
        self.brick(C_DARK_GRAY, w + 1.0, 0.6, d + 1.0, cx, 0.4 + H, cz, false);
        let mut x = x0 + 1.0;
        while x < x1 - 0.5 {
            let mut z = z0 + 1.0;
            while z < z1 - 0.5 {
                self.brick(C_WHITE, 2.3, 0.25, 2.3, x + 1.15, 0.4 + H + 0.6, z + 1.15, false);
                z += 2.5;
            }
            x += 2.5;
        }
        self.brick(C_GRAY, w + 1.0, 0.8, 0.6, cx, 0.4 + H + 0.6, z0 - 0.2, true);
        self.brick(C_GRAY, w + 1.0, 0.8, 0.6, cx, 0.4 + H + 0.6, z1 + 0.2, true);
        self.brick(C_GRAY, 0.6, 0.8, d, x0 - 0.2, 0.4 + H + 0.6, cz, false);
        self.brick(C_GRAY, 0.6, 0.8, d, x1 + 0.2, 0.4 + H + 0.6, cz, false);
        // solar panels at the back of the roof
        for i in 0..3 {
            let px = x0 + 3.0 + i as f32 * 3.4;
            self.brick_rot(0x1d3f8a, Vec3::new(3.0, 0.2, 2.0), Vec3::new(px, 0.4 + H + 1.6, z0 + 2.0), Vec3::new(0.45, 0.0, 0.0));
            self.brick(C_DARK_GRAY, 0.3, 0.9, 0.3, px, 0.4 + H + 0.8, z0 + 1.6, false);
        }
        // chimney with a hole
        self.brick(C_RED_BROWN, 2.6, 2.4, 2.6, x1 - 2.8, 0.4 + H + 0.6, z0 + 2.8, false);
        self.brick(C_BLACK, 1.4, 0.1, 1.4, x1 - 2.8, 0.4 + H + 3.0, z0 + 2.8, false);
        // satellite dish on a pole
        self.brick(C_DARK_GRAY, 0.4, 1.6, 0.4, cx + 1.0, 0.4 + H + 0.6, cz, false);
        self.here(cx + 1.0, cz);
        self.cone(C_WHITE, m(cx + 1.0, 0.4 + H + 3.0, cz + 0.6), m(cx + 1.0, 0.4 + H + 2.3, cz - 0.2), 1.4 * STUD, 14);
        // the lab sign: a blue board with a white frame over the door
        self.brick(C_WHITE, 5.4, 2.1, 0.15, cx, 4.7, fz + 0.02, false);
        self.brick(0x1e5aa8, 5.0, 1.7, 0.1, cx, 4.9, fz + 0.1, false);
        self.prop("assets/signs/lab.glb", cx, fz + 0.17, 5.75, 1.0, 0.0);
    }

    /// A white picket fence from (x0, z0) to (x1, z1): a post every stud, two rails, one collider.
    fn fence(&mut self, x0: f32, z0: f32, x1: f32, z1: f32) {
        let len = ((x1 - x0).powi(2) + (z1 - z0).powi(2)).sqrt();
        let n = len.round().max(1.0) as i32;
        for i in 0..=n {
            let k = i as f32 / n as f32;
            self.brick(C_WHITE, 0.22, 1.5, 0.22, x0 + (x1 - x0) * k, 0.0, z0 + (z1 - z0) * k, false);
        }
        let along_x = (x1 - x0).abs() > (z1 - z0).abs();
        let (mx, mz) = ((x0 + x1) * 0.5, (z0 + z1) * 0.5);
        for r in 0..2 {
            let (w, d) = if along_x { (len, 0.18) } else { (0.18, len) };
            self.brick(C_WHITE, w, 0.25, d, mx, 0.5 + 0.6 * r as f32, mz, false);
        }
        self.block(x0.min(x1) - 0.35, x0.max(x1) + 0.35, z0.min(z1) - 0.35, z0.max(z1) + 0.35);
    }

    /// A sign: a framed board facing +z.
    /// (the painted board's front is z + 0.2 plus up to 0.018 of the bricks' size steps; its picture, a glTF plane, hangs at
    /// z + `SIGN_FACE`: at 0.21 the Stonecrest board poked through its middle and only the frame showed)
    fn sign_board(&mut self, x: f32, z: f32, w: f32, h: f32, y: f32, board: u32) {
        self.brick(C_WHITE, w + 0.2, h + 0.2, 0.25, x, y - 0.1, z, false);
        self.brick(board, w - 0.2, h - 0.2, 0.1, x, y + 0.1, z + 0.15, false);
    }

    /// A lamp post with a cap with studs.
    fn lamp(&mut self, x: f32, z: f32) {
        self.brick(C_BLACK, 0.35, 5.0, 0.35, x, 0.0, z, false);
        self.brick(C_BLACK, 0.9, 0.2, 0.9, x, 5.0, z, false);
        self.brick(C_BLACK, 1.0, 0.25, 1.0, x, 6.0, z, true);
        self.block(x - 0.4, x + 0.4, z - 0.4, z + 0.4);
    }

    /// A pile of loose LEGO bricks (real brick sizes, studs on top), resting on each other.
    fn brick_pile(&mut self, seed: u32, cx: f32, cz: f32, color: u32) {
        let mut rng = TownRng(seed.wrapping_mul(9973).wrapping_add(17));
        let sizes: [[f32; 2]; 8] = [[2.0, 4.0], [2.0, 2.0], [2.0, 4.0], [1.0, 2.0], [1.0, 4.0], [2.0, 3.0], [2.0, 2.0], [1.0, 1.0]];
        let extra = [C_WHITE, C_YELLOW, C_BLUE, C_RED, C_GREEN, C_ORANGE];
        let mut placed: Vec<[f32; 4]> = vec![]; // x, z, radius, top
        const R: f32 = 3.6;
        for _ in 0..16 {
            let si = (rng.f() * 8.0) as usize % 8;
            let (w, d) = (sizes[si][0], sizes[si][1]);
            let col = if rng.f() < 0.7 { color } else { extra[(rng.f() * 6.0) as usize % 6] };
            let my_r = (w * w + d * d).sqrt() * 0.5;
            let (mut x, mut z, mut y) = (0.0f32, 0.0f32, 99.0f32);
            let mut tries = 0;
            while tries < 8 && y > 1.6 {
                let (ang, dist) = (rng.f() * std::f32::consts::TAU, rng.f().sqrt() * R);
                x = ang.cos() * dist;
                z = ang.sin() * dist;
                y = 0.0;
                for p in &placed {
                    if ((p[0] - x).powi(2) + (p[1] - z).powi(2)).sqrt() < (p[2] + my_r) * 0.85 {
                        y = y.max(p[3]);
                    }
                }
                tries += 1;
            }
            if y > 2.0 {
                continue;
            }
            let q = Quat::from_rotation_y(rng.f() * std::f32::consts::PI);
            self.here(cx + x, cz + z);
            let half = m(w, 1.2, d) * 0.5;
            self.b(col).with(Mat4::from_rotation_translation(q, m(cx + x, y + 0.6, cz + z)), |b| {
                b.cuboid(-half, half);
            });
            for sx in 0..w as i32 {
                for sz in 0..d as i32 {
                    let local = q * Vec3::new(sx as f32 - (w - 1.0) * 0.5, 0.0, sz as f32 - (d - 1.0) * 0.5);
                    self.stud(col, cx + x + local.x, y + 1.2, cz + z + local.z);
                }
            }
            placed.push([x, z, my_r, y + 1.2 + STUD_H]);
        }
        self.block(cx - R + 0.3, cx + R - 0.3, cz - R + 0.3, cz + R - 0.3);
    }

    /// A LEGO cloud: stacked white plates.
    fn cloud(&mut self, x: f32, y: f32, z: f32, s: f32) {
        self.brick(C_WHITE, 8.0 * s, 1.2 * s, 4.0 * s, x, y, z, false);
        self.brick(C_WHITE, 5.0 * s, 1.2 * s, 3.0 * s, x - 0.8 * s, y + 1.2 * s, z, false);
        self.brick(C_WHITE, 3.0 * s, 1.0 * s, 2.0 * s, x + 1.2 * s, y + 2.4 * s, z, false);
        self.brick(C_WHITE, 4.0 * s, 0.8 * s, 2.6 * s, x + 4.6 * s, y + 0.2 * s, z + 0.4 * s, false);
    }

    // ------------------------------------------------------------------------------- the village

    fn build_village(&mut self) {
        // ---- roads: grey plates with studs, solid so the figure steps up onto them ----
        for (i, rd) in ROADS.iter().enumerate().take(7) {
            self.here((rd.x0 + rd.x1) * 0.5, (rd.z0 + rd.z1) * 0.5);
            self.b(C_GRAY).block(m((rd.x0 + rd.x1) * 0.5, 0.0, (rd.z0 + rd.z1) * 0.5), m(rd.x1 - rd.x0, 0.4, rd.z1 - rd.z0));
            self.solids.push(Solid { r: *rd, y0: 0.0, h: 0.4 * STUD });
            let mut x = rd.x0 + 0.5;
            while x < rd.x1 {
                let mut z = rd.z0 + 0.5;
                while z < rd.z1 {
                    // no studs where a road crosses another (each crossing has them once)
                    if !ROADS[..i].iter().any(|o| x > o.x0 && x < o.x1 && z > o.z0 && z < o.z1) {
                        self.stud(C_GRAY, x, 0.4, z);
                    }
                    z += 1.0;
                }
                x += 1.0;
            }
        }

        // ---- the houses: yours (west, blue roof) and Dex & Lumi's (east, red roof, yellow walls) ----
        self.group(|t| t.house(-15.0, -11.0, 12.0, 10.0, C_BLUE, C_WHITE, C_RED_BROWN, C_BLUE, "assets/signs/house_home.glb"));
        self.group(|t| t.house(47.0, -12.0, 16.0, 12.0, C_RED, C_YELLOW, C_BLUE, C_RED, "assets/signs/house_dex.glb"));
        // Dex's garage next to it, with a grey door and a driveway plate
        self.brick(C_GRAY, 9.0, 0.4, 9.0, 35.0, 0.0, -10.0, false);
        self.group(|t| {
            t.brick(C_WHITE, 8.0, 5.6, 8.0, 35.0, 0.4, -10.0, false);
            t.brick(C_DARK_GRAY, 6.0, 4.0, 0.3, 35.0, 0.4, -5.9, false);
            for i in 0..4 {
                t.brick(C_GRAY, 6.0, 0.1, 0.35, 35.0, 1.2 + i as f32 * 0.9, -5.75, false);
            }
            t.brick(C_RED, 9.0, 0.6, 9.0, 35.0, 6.0, -10.0, true);
        });
        self.block_h(30.4, 39.6, -14.6, -5.5, 6.6 * STUD);
        self.brick(C_GRAY, 6.0, 0.4, 1.0, 35.0, 0.0, -5.0, false);

        // ---- the little park: trees, flowers, a bench ----
        {
            let (cx, cz) = (15.0, -11.0);
            for tr in [[-4.5, -2.5], [4.5, -3.0], [0.0, -4.5]] {
                let s = 5.5 + self.rand();
                self.round_tree(cx + tr[0], cz + tr[1], s);
                self.block(cx + tr[0] - 0.9, cx + tr[0] + 0.9, cz + tr[1] - 0.9, cz + tr[1] + 0.9);
            }
            self.flower_pot(cx - 5.5, cz + 3.0, false);
            self.flower_pot(cx + 5.5, cz + 3.0, false);
            self.brick(C_RED_BROWN, 3.2, 0.4, 1.2, cx, 1.0, cz + 1.5, true);
            self.brick(C_RED_BROWN, 3.2, 1.2, 0.3, cx, 1.4, cz + 0.95, false);
            for s in [-1.0, 1.0] {
                self.brick(C_DARK_GRAY, 0.4, 1.0, 1.0, cx + s * 1.3, 0.0, cz + 1.5, false);
            }
            self.block(cx - 1.7, cx + 1.7, cz + 0.8, cz + 2.2);
        }

        // ---- loose LEGO brick piles ----
        self.brick_pile(0, 45.0, 16.5, C_RED);
        self.brick_pile(2, 57.5, 16.5, C_YELLOW);
        self.brick_pile(3, 33.5, 17.0, C_BLUE);
        self.brick_pile(1, -17.0, -21.5, C_ORANGE);

        // ---- the lab ----
        self.group(|t| t.lab(13.0, 11.0, 16.0, 10.0));

        // ---- the garden: a white fence with a gate, rows of flowers, a sign ----
        {
            let (x0, x1, z0, z1) = (-25.0, -13.0, 7.0, 17.0);
            self.fence(x0, z0, -21.2, z0);
            self.fence(-16.8, z0, x1, z0);
            self.fence(x0, z1, x1, z1);
            self.fence(x0, z0, x0, z1);
            self.fence(x1, z0, x1, z1);
            self.brick(C_WHITE, 0.8, 2.0, 0.8, -21.4, 0.0, z0, true);
            self.brick(C_WHITE, 0.8, 2.0, 0.8, -16.6, 0.0, z0, true);
            let cols = [C_RED, C_RED, C_YELLOW, C_PINK, C_WHITE, C_ORANGE];
            let mut x = x0 + 1.6;
            while x < x1 - 1.0 {
                let mut z = z0 + 1.9;
                while z < z1 - 1.0 {
                    if self.rand() >= 0.15 && !(z < 10.5 && x < -14.0) {
                        let (jx, jz) = ((self.rand() - 0.5) * 0.3, (self.rand() - 0.5) * 0.3);
                        let c = cols[(self.rand() * 6.0) as usize % 6];
                        self.flower(x + jx, z + jz, c, 0.0);
                    }
                    z += 2.2;
                }
                x += 2.2;
            }
            self.brick(C_BROWN, 0.3, 2.6, 0.3, -23.2, 0.0, 8.9, false);
            self.brick(C_TAN, 0.15, 1.5, 2.8, -23.0, 1.35, 8.9, false);
            self.prop("assets/signs/garden.glb", -22.91, 8.9, 2.1, 1.0, std::f32::consts::FRAC_PI_2);
            self.block(-23.4, -22.9, 7.6, 10.2);
        }

        // ---- the pond: a brown border with studs, a dark bed, lily pads ----
        {
            let (x0, x1, z0, z1) = (POND.x0, POND.x1, POND.z0, POND.z1);
            self.block_h(x0 - 0.4, x1 + 0.4, z0 - 0.4, z1 + 0.4, 0.5);
            self.brick(C_RED_BROWN, x1 - x0 + 1.0, 0.4, 0.6, (x0 + x1) * 0.5, 0.0, z0 - 0.2, true);
            self.brick(C_RED_BROWN, x1 - x0 + 1.0, 0.4, 0.6, (x0 + x1) * 0.5, 0.0, z1 + 0.2, true);
            self.brick(C_RED_BROWN, 0.6, 0.4, z1 - z0, x0 - 0.2, 0.0, (z0 + z1) * 0.5, true);
            self.brick(C_RED_BROWN, 0.6, 0.4, z1 - z0, x1 + 0.2, 0.0, (z0 + z1) * 0.5, true);
            self.brick(0x1d4a5e, x1 - x0, 0.05, z1 - z0, (x0 + x1) * 0.5, 0.0, (z0 + z1) * 0.5, false);
            for p in [[-10.0, 17.0], [-6.5, 20.0], [-8.0, 18.8]] {
                self.cyl(C_GREEN, p[0], 0.3, p[1], 0.6, 0.6, 0.08, 12);
            }
            self.flower(-6.5, 20.0, C_PINK, 0.38);
            self.brick(C_RED_BROWN, 3.0, 0.3, 1.0, -7.5, 0.9, 22.4, false); // bench near the pond
            self.brick(C_DARK_GRAY, 0.4, 0.9, 0.8, -8.7, 0.0, 22.4, false);
            self.brick(C_DARK_GRAY, 0.4, 0.9, 0.8, -6.3, 0.0, 22.4, false);
        }

        // ---- signs ----
        self.brick(C_BROWN, 0.35, 3.2, 0.35, 4.4, 0.0, 20.0, false);
        self.brick(C_BROWN, 0.35, 3.2, 0.35, 7.4, 0.0, 20.0, false);
        self.sign_board(5.9, 20.2, 3.8, 2.0, 1.3, C_WHITE);
        self.prop("assets/signs/village.glb", 5.9, 20.2 + SIGN_FACE, 2.3, 1.0, 0.0);
        self.block(4.0, 7.8, 19.6, 20.6);
        self.brick(C_BROWN, 0.35, 3.2, 0.35, 5.0, 0.0, -24.5, false);
        self.sign_board(5.0, -24.3, 3.4, 1.8, 1.3, 0xfff6c8);
        self.prop("assets/signs/route.glb", 5.0, -24.3 + SIGN_FACE, 2.2, 1.0, 0.0);
        self.block(4.5, 5.5, -25.0, -24.0);

        // ---- lamp posts ----
        for l in LAMPS {
            self.lamp(l[0], l[1]);
        }

        // ---- round trees inside the village ----
        for t in [[22.0, 20.0], [-24.0, -20.0], [8.0, -20.0], [-9.0, -20.0], [24.0, 4.0], [21.0, -21.0]] {
            self.round_tree(t[0], t[1], 6.5);
            self.block(t[0] - 0.9, t[0] + 0.9, t[1] - 0.9, t[1] + 0.9);
        }

        // ---- bamboo leaves around the pond, along the walls, by the garden and the exit ----
        let lv: [[f32; 3]; 24] = [
            [-13.0, 15.5, 1.6], [-13.0, 21.0, 1.7], [-3.4, 17.0, 1.5], [-12.6, 23.6, 1.6], [-8.0, 23.8, 1.6],
            [-20.6, -5.2, 1.2], [-18.6, -5.2, 1.2], [-11.6, -5.2, 1.2],
            [35.0, 10.6, 1.5], [39.0, 10.6, 1.5], [59.0, 10.6, 1.5], [62.0, 10.6, 1.5],
            [6.0, 17.0, 1.25], [18.0, 17.0, 1.25], [20.0, 17.0, 1.25],
            [-24.0, 8.5, 1.3], [-24.0, 15.6, 1.3], [-14.0, 15.6, 1.3],
            [-25.5, -4.0, 1.8], [25.5, 8.0, 1.8], [-25.5, 18.0, 1.8], [25.5, -18.0, 1.8], [6.5, -25.6, 1.8],
            [-6.5, -25.6, 1.8],
        ];
        for l in lv {
            self.leaves(l[0], l[1], l[2]);
        }

        // ---- flower beds, single flowers and flower pots ----
        for x in [4.2, 6.3] {
            let c = if self.rand() < 0.5 { C_RED } else { C_YELLOW };
            self.flower(x, 18.6, c, 0.0);
        }
        for x in [19.7, 21.8] {
            let c = if self.rand() < 0.5 { C_RED } else { C_YELLOW };
            self.flower(x, 18.6, c, 0.0);
        }
        for x in [-11.5, -9.4, -7.3, -5.2] {
            let c = if self.rand() < 0.5 { C_PINK } else { C_WHITE };
            self.flower(x, 23.7, c, 0.0);
        }
        for f in [[8.0, -9.5], [8.0, -16.0], [22.0, -16.0]] {
            self.flower(f[0], f[1], C_ORANGE, 0.0);
        }
        // (by your own front door the flowers stand without their pots; the others keep them)
        for p in [[-11.8, -4.3], [-22.5, -4.3]] {
            self.flower_pot(p[0], p[1], false);
        }
        for p in [[23.5, 12.0], [55.5, -4.6]] {
            self.flower_pot(p[0], p[1], true);
        }
        // crates by the lab
        self.brick(C_BROWN, 1.6, 1.6, 1.6, 22.6, 0.0, 9.0, true);
        self.brick(C_TAN, 1.6, 1.6, 1.6, 22.6, 1.6, 9.0, true);
        self.block(21.6, 23.6, 8.0, 10.0);

        // ---- tall grass: the first patch north of the village gap ----
        {
            let greens = [0x58ab41, C_GREEN, 0x7fc04a, 0x3c8c3a];
            let mut n = 0;
            let r = GRASSES[0];
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

        // ---- trees: the borders around the village (two staggered rows) and the line to the route ----
        {
            let step = 3.6;
            let mut z = -45.0;
            while z <= 27.0 {
                for (k, x0) in [-30.3, -27.4, 70.3, 67.4].into_iter().enumerate() {
                    let x = x0 + (self.rand() - 0.5) * 0.8;
                    let zz = z + if k % 2 == 1 { step * 0.5 } else { 0.0 };
                    if self.rand() < 0.25 {
                        let s = 6.0 + self.rand() * 1.5;
                        self.round_tree(x, zz, s);
                    } else {
                        let s = 8.5 + self.rand() * 2.5;
                        self.spruce(x, zz, s);
                    }
                }
                z += step;
            }
            // the south row: each tree and bush blocks itself
            let mut x = -24.0;
            while x <= 64.0 {
                let tx = x + (self.rand() - 0.5) * 0.8;
                let s = 3.0 + self.rand() * 0.6;
                self.round_tree(tx, 27.2, s);
                self.block(tx - 0.7, tx + 0.7, 26.5, 27.9);
                let s = 1.6 + self.rand() * 0.5;
                self.bush(x + step * 0.5, 25.0, s);
                self.block(x + step * 0.5 - 0.8, x + step * 0.5 + 0.8, 24.2, 25.8);
                x += step;
            }
            // the village's south edge: a dense hedge and low round trees
            let mut x = -28.0;
            while x <= 68.0 {
                let (jx, jz, s) = ((self.rand() - 0.5) * 0.4, (self.rand() - 0.5) * 0.5, 2.2 + self.rand() * 0.6);
                self.bush(x + jx, 29.9 + jz, s);
                x += 1.7;
            }
            let mut x = -27.0;
            while x <= 68.0 {
                let (jx, jz, s) = ((self.rand() - 0.5) * 1.0, (self.rand() - 0.5) * 0.8, 3.4 + self.rand() * 0.8);
                self.round_tree(x + jx, 32.0 + jz, s);
                x += 3.0;
            }
            let mut x = -25.0f32;
            while x <= 25.0 {
                if x.abs() > 6.0 {
                    let s = 8.5 + self.rand() * 2.5;
                    self.spruce(x, -27.6, s);
                }
                if (x + step * 0.5).abs() > 11.0 {
                    let s = 8.5 + self.rand() * 2.5;
                    self.spruce(x + step * 0.5, -30.2, s);
                }
                x += step;
            }
            let mut x = 28.0;
            while x <= 66.0 {
                let s = 9.0 + self.rand() * 2.0;
                self.spruce(x, -29.6, s);
                let s = 9.0 + self.rand() * 2.0;
                self.spruce(x + step * 0.5, -32.4, s);
                x += step;
            }
            let mut x = 11.5;
            while x <= 27.0 {
                let mut z = -40.9;
                while z <= -32.0 {
                    let s = 8.5 + self.rand() * 2.5;
                    self.spruce(x, z, s);
                    let s = 8.5 + self.rand() * 2.5;
                    self.spruce(-x, z, s);
                    z += step;
                }
                x += step;
            }
            for b in [[-7.2, -26.2], [7.2, -26.2], [-24.0, 22.0], [24.0, 22.0], [-24.0, -24.0], [-9.0, 22.5], [18.0, 22.5], [-26.0, 6.0], [24.0, 14.0], [64.0, 14.0], [64.0, -24.0]] {
                self.bush(b[0], b[1], 2.4);
                self.block(b[0] - 1.1, b[0] + 1.1, b[1] - 1.1, b[1] + 1.1);
            }
        }

        // ---- the villagers who stand still block their spot ----
        for n in &NPCS {
            // (Pip walks; Prof. Quill steps aside once the way north is open: her own capsule blocks)
            if n.id != "kid" && n.id != "gate-prof" {
                let (x, z) = (n.x, n.z);
                self.block(x - 0.9, x + 0.9, z - 0.7, z + 0.7);
            }
        }

        // ---- the little football goal east of the park ----
        {
            let (x0, x1, z0, z1) = (24.5, 27.0, -7.6, -4.4);
            for z in [z0, z1] {
                self.brick(C_WHITE, 0.3, 2.6, 0.3, x0, 0.0, z, false);
                self.brick(C_WHITE, 0.3, 2.6, 0.3, x1, 0.0, z, false);
                self.brick(C_WHITE, x1 - x0, 0.25, 0.25, (x0 + x1) * 0.5, 2.5, z, false);
                let mut x = x0 + 0.5;
                while x < x1 {
                    self.brick(0xdfe6ea, 0.05, 2.5, 0.05, x, 0.0, z, false);
                    x += 0.5;
                }
                self.block(x0, x1, z - 0.2, z + 0.2);
            }
            self.brick(C_WHITE, 0.25, 0.25, z1 - z0, x0, 2.5, (z0 + z1) * 0.5, false);
            let mut z = z0 + 0.4;
            while z < z1 {
                self.brick(0xdfe6ea, 0.05, 2.5, 0.05, x1, 0.0, z, false);
                z += 0.4;
            }
            let mut y = 0.5;
            while y < 2.5 {
                self.brick(0xdfe6ea, 0.05, 0.05, z1 - z0, x1, y, (z0 + z1) * 0.5, false);
                y += 0.5;
            }
            self.block(x1 - 0.2, x1 + 0.2, z0, z1);
        }

        // ---- the wild LEGO plants wherever there is room (the garden inside its fence stays as it is) ----
        self.occupy(-25.0, -13.0, 7.0, 17.0);
        self.wild_plants();

        // ---- LEGO clouds ----
        // the LEGO clouds hang beside and beyond the diorama at the camera's height: the eye-level views (close-ups,
        // fights) see them in the sky, the top-down camera never has them below it (they used to float over Route 1
        // under the camera: pale slabs and shadows on the route)
        self.cloud(-92.0, 32.0, 4.0, 1.6);
        self.cloud(-98.0, 36.0, -78.0, 2.0);
        self.cloud(-90.0, 30.0, -150.0, 1.5);
        self.cloud(125.0, 34.0, -12.0, 1.8);
        self.cloud(118.0, 31.0, -105.0, 1.5);
        self.cloud(18.0, 38.0, -252.0, 2.2);

        // ---- where you can walk: the village, the gap north and the grass patch (walls along the edges) ----
        for w in [
            [-26.2, 66.2, 28.6, 29.6],
            [-27.2, -26.2, -25.6, 28.6],
            [66.2, 67.2, -25.6, 28.6],
            [-26.2, -3.6, -26.6, -25.6],
            [3.6, 66.2, -26.6, -25.6],
            [-4.6, -3.6, -28.6, -25.6],
            [3.6, 4.6, -28.6, -25.6],
            [-8.4, -3.6, -29.6, -28.6],
            [3.6, 8.4, -29.6, -28.6],
            [-9.4, -8.4, -41.6, -28.6],
            [8.4, 9.4, -41.6, -28.6],
        ] {
            self.block(w[0], w[1], w[2], w[3]);
        }

        // ---- Route 1 north and Stonecrest (last: the village above keeps its random layout) ----
        self.build_north();
    }
}

pub const LAMPS: [[f32; 2]; 6] = [[-4.0, -4.0], [4.0, 4.0], [-4.0, 10.0], [4.0, -14.0], [-22.0, 4.0], [22.0, -3.5]];

/// GPU resources shared by every town build (created once).
pub struct TownAssets {
    /// Stonecrest's models (the castle alone is 4 MB) load after the village is up (`load_deferred`): their roots
    /// (placed, on their layer, already occluders) and what to load under each
    pub deferred: Vec<(Entity, String, f32)>,
    pub materials: HashMap<u32, MaterialId>,
    pub stud: MeshId,
    pub ground: MaterialId,
    /// the baseplate's stud pattern (colour shading, normal map): the island's tops use it too
    pub stud_tex: Option<engine::renderer::TextureId>,
    pub stud_nrm: Option<engine::renderer::TextureId>,
    /// the tan dirt trail of Route 1 (the baseplate's studs in tan)
    pub trail: MaterialId,
    pub water: MaterialId,
    /// the creek on Route 1 (coarser waves: a long strip)
    pub creek: MaterialId,
    /// the blades of a tall grass tuft (`tuft_mesh`) and their bending materials per colour
    pub tuft: MeshId,
    pub tuft_mats: HashMap<u32, MaterialId>,
    /// the village life's meshes (life.rs, made on the first play screen)
    pub life: Option<crate::life::LifeArt>,
}

/// How far in front of a sign board's centre its picture hangs (studs; `Town::sign_board`).
pub const SIGN_FACE: f32 = 0.26;

/// Loads the deferred models (`TownAssets::deferred`) under their roots, once nothing else is loading (call every frame;
/// does nothing after the first time). Returns how many it started.
pub fn load_deferred(ctx: &mut Ctx, assets: &mut TownAssets) -> usize {
    if assets.deferred.is_empty() || ctx.assets_pending() > 0 {
        return 0;
    }
    let list = std::mem::take(&mut assets.deferred);
    for (root, path, lod) in &list {
        let opts = prop_opts(ctx, path, *lod);
        let e = ctx.load_gltf_with(path, Transform::default(), opts);
        let _ = ctx.world.insert_one(e, Parent(*root));
        let layer = ctx.world.get::<&engine::scene::RenderLayer>(*root).map(|l| l.0).unwrap_or(0);
        ctx.set_layer(e, layer);
    }
    list.len()
}

/// A tree model (the forests round the village: most of what a PC camera tilted low draws).
pub fn is_tree(path: &str) -> bool {
    ["tree", "spruce", "cypress", "pine", "palm", "bush"].iter().any(|k| path.contains(k))
}

/// How far (times the model's radius) a tree stays a model: beyond, it is the engine's impostor, one baked card
/// (2 triangles, lit with baked normals, turned toward the camera). 60 x radius is ~20 px on a 1350 px PC view.
pub const TREE_IMPOSTOR: f32 = 30.0;

/// The load options of a prop: its engine LOD levels (`lod` > 0: their switch distance), trees also an impostor.
pub fn prop_opts(ctx: &Ctx, path: &str, lod: f32) -> engine::gltf_loader::GltfOpts {
    let mut o = if lod > 0.0 { engine::gltf_loader::GltfOpts { auto_lod: true, lod_distance: lod, ..Default::default() } } else { ctx.gltf_defaults };
    if is_tree(path) {
        o.impostor = true;
        o.impostor_distance = TREE_IMPOSTOR;
        o.impostor_views = 8;
        o.impostor_res = 128;
    }
    o
}

/// Linear material for an sRGB LEGO colour (shiny plastic, as `mat()` in the three.js town).
fn lego(hex: u32) -> Material {
    Material { name: format!("lego {hex:06x}"), ..Material::color(hex).rough(crate::PLASTIC_ROUGH) }
}

/// The baseplate's stud pattern: one stud per texture tile (base colour shading + normal map), seen from above as a
/// LEGO stud's profile: a flat top (radius 0.272 studs), a narrow bevel down to the wall (0.30), the wall itself a
/// thin ring of sideways normals, and round the foot a soft contact shadow (no hard ring). 4x4 samples per texel, so
/// the rim stays smooth in the mip levels.
fn stud_maps(ctx: &mut Ctx) -> (Option<engine::renderer::TextureId>, Option<engine::renderer::TextureId>) {
    const N: u32 = 128;
    const SS: u32 = 4;
    const R_TOP: f32 = 0.272;
    const R_BASE: f32 = STUD_R;
    const WALL: f32 = 0.012;
    let mut col = vec![0u8; (N * N * 4) as usize];
    let mut nrm = vec![0u8; (N * N * 4) as usize];
    let smooth = |e0: f32, e1: f32, x: f32| {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    for y in 0..N {
        for x in 0..N {
            let (mut n_sum, mut shade_sum) = (Vec3::ZERO, 0.0f32);
            for sy in 0..SS {
                for sx in 0..SS {
                    // (studs: the tile's middle is the stud's middle)
                    let u = (x as f32 + (sx as f32 + 0.5) / SS as f32) / N as f32 - 0.5;
                    let v = (y as f32 + (sy as f32 + 0.5) / SS as f32) / N as f32 - 0.5;
                    let d = (u * u + v * v).sqrt().max(1e-5);
                    let radial = Vec2::new(u / d, v / d);
                    let (n, shade) = if d <= R_TOP {
                        // the top: flat, a touch brighter toward the middle
                        (Vec3::Z, 1.0 + 0.04 * (1.0 - d / R_TOP))
                    } else if d <= R_BASE - WALL {
                        // the bevel: a cone falling outward
                        let t = (d - R_TOP) / (R_BASE - WALL - R_TOP);
                        let tilt = 0.55 + 0.35 * t;
                        (Vec3::new(radial.x * tilt, radial.y * tilt, 1.0).normalize(), 0.98)
                    } else if d <= R_BASE {
                        // the wall, seen from above as a thin ring facing outward
                        (Vec3::new(radial.x, radial.y, 0.25).normalize(), 0.9)
                    } else {
                        // the plate round the foot: a soft contact shadow
                        (Vec3::Z, 0.9 + 0.1 * smooth(0.0, 0.16, d - R_BASE))
                    };
                    n_sum += n;
                    shade_sum += shade;
                }
            }
            let n = n_sum.normalize();
            let shade = shade_sum / (SS * SS) as f32;
            let i = ((y * N + x) * 4) as usize;
            let v = (255.0 * shade.min(1.0)) as u8;
            let hi = (255.0 * (shade - 1.0).max(0.0) * 4.0) as u8;
            col[i..i + 4].copy_from_slice(&[v.saturating_add(hi), v.saturating_add(hi), v.saturating_add(hi), 255]);
            // (texture v runs down the image: its y flips)
            nrm[i..i + 4].copy_from_slice(&[((n.x * 0.5 + 0.5) * 255.0) as u8, ((-n.y * 0.5 + 0.5) * 255.0) as u8, ((n.z * 0.5 + 0.5) * 255.0) as u8, 255]);
        }
    }
    (ctx.renderer.add_texture_rgba(N, N, &col, "baseplate studs").ok(), ctx.renderer.add_texture_rgba_linear(N, N, &nrm, "baseplate studs normal").ok())
}

impl TownAssets {
    pub fn new(ctx: &mut Ctx) -> Self {
        let mut mb = MeshBuilder::new();
        // no bottom disc: a stud always stands on a brick (the disc lay in the plane of any brick stacked on it);
        // 0.01 studs lower than STUD_H: a plate laid "on the studs" (y + STUD_H) does not share the studs' top plane
        let sh = (STUD_H - 0.01) * STUD;
        mb.lathe(&[Vec2::new(STUD_R * STUD, 0.0), Vec2::new(STUD_R * STUD, sh), Vec2::new(0.0, sh)], 10);
        let parts = mb.finish();
        let mut md = parts.into_iter().next().unwrap().1;
        engine::geom::smooth_normals(&mut md, 50.0);
        let stud = ctx.renderer.add_mesh("lego stud", &md);
        let (tex, nrm) = stud_maps(ctx);
        let ground = ctx.renderer.add_material(Material { name: "baseplate".into(), texture: tex, normal_texture: nrm, normal_scale: 1.0, ..lego(C_GREEN) });
        let trail = ctx.renderer.add_material(Material { name: "trail".into(), texture: tex, normal_texture: nrm, normal_scale: 1.0, ..lego(0xc8a46a) });
        // a small, murky village pond: low calm waves, little clarity
        let water = ctx.renderer.add_material(Material { name: "pond water".into(), wave_height: 0.015, wave_scale: 2.0, water_chop: 0.3, water_clarity: 0.8, water_foam: 0.0, water_vertex_spacing: 0.1, ..Material::water(0x1d4a5e, C_WATER) });
        let creek = ctx.renderer.add_material(Material { name: "creek water".into(), wave_height: 0.012, wave_scale: 2.5, water_chop: 0.35, water_clarity: 0.9, water_foam: 0.0, water_vertex_spacing: 0.25, ..Material::water(0x1d4a5e, C_WATER) });
        let _ = AlphaMode::Opaque;
        let tuft = ctx.renderer.add_mesh("grass tuft", &tuft_mesh());
        Self { deferred: vec![], materials: HashMap::new(), stud, ground, stud_tex: tex, stud_nrm: nrm, trail, water, creek, tuft, tuft_mats: HashMap::new(), life: None }
    }

    pub fn material(&mut self, ctx: &mut Ctx, hex: u32) -> MaterialId {
        *self.materials.entry(hex).or_insert_with(|| ctx.renderer.add_material(lego(hex)))
    }

    /// The tall grass blades in `hex`: the LEGO plastic that bends away from the pushers (grass.rs).
    pub fn tuft_material(&mut self, ctx: &mut Ctx, hex: u32) -> MaterialId {
        *self.tuft_mats.entry(hex).or_insert_with(|| ctx.renderer.add_material(Material { name: format!("grass tuft {hex:06x}"), bend: crate::grass::TUFT_BEND, ..lego(hex) }))
    }
}

/// Builds the village into the current scene. Returns the colliders' rects (studs) for walk checks and stats.
/// The grass tufts' blades come back as (entity, spec) for `grass::Grass`.
pub fn build(ctx: &mut Ctx, assets: &mut TownAssets) -> (Vec<Solid>, TownStats, Vec<OccluderSpec>, Vec<(Entity, TuftSpec)>) {
    let t0 = engine::log::now_ms();
    let mut t = Town::new();
    t.build_village();
    t.rooms_build();
    let village = rooms::VILLAGE_LAYER;
    let layer = |a: i8| if a < 0 { village } else { rooms::layer_of(Some(rooms::ROOMS[a as usize])) };
    let mut stats = TownStats::default();

    // ---- the baseplate (village) ----
    {
        let mut mb = MeshBuilder::new().uv_world(1.0 / STUD);
        mb.part("ground").cuboid(m(20.0 - 52.0, -0.8, -9.0 - 38.0), m(20.0 + 52.0, 0.0, -9.0 + 38.0));
        let parts = mb.finish();
        let g = assets.ground;
        let base = ctx.spawn_parts("baseplate", &parts, |_| g, Transform::default());
        ctx.set_layer(base, village);
        ctx.physics.add_static(ColliderBuilder::cuboid(52.0 * STUD, 0.4 * STUD, 38.0 * STUD), m(20.0, -0.4, -9.0));
        // (no table under the plates any more: the island's land and sea lie round them, islandview.rs)
        // ---- the north plate (Route 1 and Stonecrest), one piece per band, and the trail on it ----
        let p = NORTH_PLATE;
        for (i, (z0, z1, layer)) in NORTH_BANDS.into_iter().enumerate() {
            let (pz0, pz1) = (z0.max(p.z0), z1.min(p.z1));
            let mut mb = MeshBuilder::new().uv_world(1.0 / STUD);
            mb.part("ground").cuboid(m(p.x0, -0.8, pz0), m(p.x1, 0.0, pz1));
            let (tz0, tz1) = (z0.max(TRAIL.z0), z1.min(TRAIL.z1));
            if tz1 > tz0 {
                mb.part("trail").cuboid(m(TRAIL.x0, 0.0, tz0), m(TRAIL.x1, 0.04, tz1));
            }
            // (the gate trail's two pieces meet at a corner: tops only, so no two side faces share a plane)
            for tr in GATE_TRAILS {
                let (tz0, tz1) = (z0.max(tr.z0), z1.min(tr.z1));
                if tz1 > tz0 {
                    let y = 0.04;
                    mb.part("trail").quad(m(tr.x0, y, tz1), m(tr.x1, y, tz1), m(tr.x1, y, tz0), m(tr.x0, y, tz0));
                }
            }
            let parts = mb.finish();
            let (g, tr) = (assets.ground, assets.trail);
            let e = ctx.spawn_parts(&format!("north plate {i}"), &parts, |n| if n == "trail" { tr } else { g }, Transform::default());
            ctx.set_layer(e, layer);
        }
        ctx.physics.add_static(ColliderBuilder::cuboid((p.x1 - p.x0) * 0.5 * STUD, 0.4 * STUD, (p.z1 - p.z0) * 0.5 * STUD), m((p.x0 + p.x1) * 0.5, -0.4, (p.z0 + p.z1) * 0.5));
        // the creek's water (on the band of the bridge): one strip (spawn_water tessellates each axis to the spacing)
        let c = CREEK;
        let creek = ctx.spawn_water("creek water", assets.creek, Vec2::new((c.x1 - c.x0) * STUD, (c.z1 - c.z0) * STUD), Transform::from_translation(m((c.x0 + c.x1) * 0.5, 0.12, (c.z0 + c.z1) * 0.5)));
        ctx.set_layer(creek, layer_at(0.0, (c.z0 + c.z1) * 0.5));
        // the pond's water surface
        let pond = ctx.spawn_water("pond water", assets.water, Vec2::new((POND.x1 - POND.x0) * STUD, (POND.z1 - POND.z0) * STUD), Transform::from_translation(m((POND.x0 + POND.x1) * 0.5, 0.25, (POND.z0 + POND.z1) * 0.5)));
        ctx.set_layer(pond, village);
    }

    // ---- the bricks, chunk by chunk, one part per colour ----
    let chunks = std::mem::take(&mut t.chunks);
    let bricks = ctx.world.spawn((Transform::default(), Name("town bricks".into())));
    for ((cx, cz, chunk_layer), mb) in chunks {
        stats.triangles += mb.triangle_count();
        let parts = mb.finish();
        stats.meshes += parts.len();
        let mats: HashMap<String, MaterialId> = parts
            .iter()
            .map(|(p, _)| {
                let hex = u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff);
                (p.clone(), assets.material(ctx, hex))
            })
            .collect();
        let e = ctx.spawn_parts(&format!("town chunk {cx},{cz} (layer {chunk_layer})"), &parts, |p| mats[p], Transform::default());
        ctx.set_layer(e, chunk_layer);
        let _ = ctx.world.insert_one(e, Parent(bricks));
        stats.chunks += 1;
    }
    // far away (a PC camera tilted low) each cell of bricks is one simplified mesh (the engine's HLOD: vertices
    // clustered at a stud, the colours baked in): the near cells keep their bricks
    let rep = ctx.batch_static_with(bricks, HLOD_CELL, Some(engine::batch::Hlod { distance: HLOD_DISTANCE, cluster: STUD }));
    stats.hlods = rep.hlods;
    stats.hlod_triangles = rep.hlod_triangles;

    // ---- buildings: their own meshes, so they can fade while they hide the hero ----
    let mut occluders = vec![];
    let mut group_studs: Vec<Vec<Entity>> = vec![vec![]; t.groups.len()];
    let groups = std::mem::take(&mut t.groups);
    let mut roots = vec![];
    let mut group_layer = vec![];
    for (i, mb) in groups.into_iter().enumerate() {
        stats.triangles += mb.triangle_count();
        let parts = mb.finish();
        stats.meshes += parts.len();
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for (_, md) in &parts {
            for v in &md.vertices {
                let p = Vec3::from(v.pos);
                min = min.min(p);
                max = max.max(p);
            }
        }
        let mats: HashMap<String, MaterialId> = parts
            .iter()
            .map(|(p, _)| (p.clone(), assets.material(ctx, u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff))))
            .collect();
        let root = ctx.spawn_parts(&format!("building {i}"), &parts, |p| mats[p], Transform::default());
        // (a building outside: the band of its middle)
        let gl = if t.group_area[i] < 0 { layer_at((min.x + max.x) * 0.5 / STUD, (min.z + max.z) * 0.5 / STUD) } else { layer(t.group_area[i]) };
        group_layer.push(gl);
        ctx.set_layer(root, gl);
        roots.push((root, min, max, t.group_area[i]));
    }

    // ---- studs: one mesh, instanced ----
    for (hex, p, g) in &t.studs {
        let mat = assets.material(ctx, *hex);
        let e = ctx.spawn_mesh("stud", assets.stud, mat, Transform::from_translation(*p));
        ctx.set_layer(e, g.map(|g| group_layer[g]).unwrap_or_else(|| layer_at(p.x / STUD, p.z / STUD)));
        if let Some(g) = g {
            group_studs[*g].push(e);
        }
    }
    stats.studs = t.studs.len();
    // ---- the tall grass blades: one mesh, instanced, each tuft its own static transform; the engine bends them
    // (`Material::bend`, pushers set in grass.rs); they cast their own (unbent) static shadows ----
    let mut tufts = vec![];
    for spec in &t.tufts {
        let mat = assets.tuft_material(ctx, spec.hex);
        let e = ctx.spawn_mesh("grass tuft", assets.tuft, mat, Transform::from_translation(spec.pos).with_rotation(Quat::from_rotation_y(-spec.yaw)).with_scale(Vec3::splat(spec.scale)));
        ctx.set_layer(e, layer_at(spec.pos.x / STUD, spec.pos.z / STUD));
        tufts.push((e, *spec));
    }
    for ((root, min, max, area), extra) in roots.into_iter().zip(group_studs) {
        // the rooms are drawn alone: nothing in them hides you
        if area >= 0 {
            continue;
        }
        occluders.push(OccluderSpec::Bricks { root, extra, min, max });
    }

    // ---- models (the trees can fade too) ----
    for p in &t.props {
        let t = Transform::from_translation(p.pos).with_rotation(Quat::from_rotation_y(p.rot)).with_scale(Vec3::splat(p.scale));
        // Stonecrest's (far north, out of sight at the start): an empty root now, the model once the village is loaded
        let later = p.area < 0 && zone_at(p.pos.z / STUD) == 2;
        let root = if later {
            let root = ctx.world.spawn((t, Name(format!("later:{}", p.path))));
            assets.deferred.push((root, p.path.clone(), p.lod));
            root
        } else {
            let opts = prop_opts(ctx, &p.path, p.lod);
            ctx.load_gltf_with(&p.path, t, opts)
        };
        ctx.set_layer(root, if p.area < 0 { layer_at(p.pos.x / STUD, p.pos.z / STUD) } else { layer(p.area) });
        if is_tree(&p.path) || p.path.contains("castle") {
            occluders.push(OccluderSpec::Model { root });
        }
    }
    stats.props = t.props.len();
    stats.occluders = occluders.len();

    // ---- colliders ----
    for s in &t.solids {
        let (hx, hz) = ((s.r.x1 - s.r.x0) * 0.5 * STUD, (s.r.z1 - s.r.z0) * 0.5 * STUD);
        ctx.physics.add_static(ColliderBuilder::cuboid(hx, s.h * 0.5, hz), Vec3::new((s.r.x0 + s.r.x1) * 0.5 * STUD, s.y0 + s.h * 0.5, (s.r.z0 + s.r.z1) * 0.5 * STUD));
    }
    stats.colliders = t.solids.len();
    stats.colours = assets.materials.len();
    engine::log_info!("town: {} chunks, {} meshes, {} tris, {} studs, {} props, {} colliders in {:.0} ms", stats.chunks, stats.meshes, stats.triangles, stats.studs, stats.props, stats.colliders, engine::log::now_ms() - t0);
    (t.solids, stats, occluders, tufts)
}
