//! The first deck's quest and the pond cave (Kaigen `st_dive.c`). In the lab: Prof. Quill's old diving suit (hidden
//! in a corner; taken through `talk::Use::Suit`). With the suit the pond takes you down into the flooded cave
//! (`cave.rs`): you swim in first person - on a phone hold a finger to swim forward and move it to steer (as
//! abyssal-garden), pinch to swim backwards, spread to swim faster; with keys and mouse W/S A/D, Space / Q up and
//! down, Shift fast, drag to look - between swaying kelp, fish schools, mantas, rising bubbles, marine snow and glowing
//! plants, the water darker the deeper you go. Eight pearls to find, and the Stud Core: down the trench, through the
//! tunnel at its bottom, at the far end of the deep grotto.
//! Swim up into the hole (or the Surface button / Esc) to leave.
//!
//! The dive has its own camera (`ctx.set_main_camera`, drawing only `cave::CAVE_LAYER`, a wide lens) and owns the
//! fog / light settings while it lasts (put back when you surface). `PlayScreen` starts it (`start`) after the
//! sinking, calls `update` / `draw` every frame while `on` and moves you to the pond's north bank when it ends.

use crate::cave::{self, floor, hash, world, CAVE_HOLE, CAVE_LAYER, CAVE_R, CAVE_TOP};
use crate::life::Life;
use crate::save::GameSave;
use crate::sound::Sfx;
use crate::town::{self, rooms, TownAssets, STUD};
use crate::Data;
use engine::geom::MeshBuilder;
use engine::prelude::*;
use engine::renderer::{srgb_hex, Material, MaterialId, MeshId};
use serde_json::{json, Value};

const KELP: usize = 36;
const FISH: usize = 42; // three schools of 14
const MANTAS: usize = 2;
const BUBBLES: usize = 48;
const TUFTS: usize = 40;
pub const PEARLS: usize = 8;
/// marine snow: specks drifting around the diver (wrapped in a box around you)
const SNOW: usize = 96;
/// Where the Stud Core lies (cave studs): at the far end of the deep grotto, past the tunnel from the trench.
const CORE_AT: Vec3 = cave::DEEP_CORE;
/// Where the suit stands in the lab (studs) and its yaw (Kaigen).
const SUIT_AT: Vec3 = Vec3::new(314.2, 0.4, -9.4);

/// A small multi-colour model: (mesh, material) per part, shared by all its copies (one draw per part).
/// (`.1`: the parts' scale, for the unit-diameter builtin sphere)
#[derive(Clone, Default)]
struct Shape(Vec<(MeshId, MaterialId)>, f32);

/// Builds a shape at the origin in studs (bottom-centred boxes like `town_box`).
struct Sb(MeshBuilder);
impl Sb {
    fn bx(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32) -> &mut Self {
        self.0.part(&format!("c{hex:06x}")).block(town::m(x, y, z), town::m(w, h, d));
        self
    }
    fn rot(&mut self, hex: u32, size: Vec3, c: Vec3, euler: Vec3) -> &mut Self {
        let t = Mat4::from_rotation_translation(Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z), town::m(c.x, c.y, c.z));
        let s = town::m(size.x, size.y, size.z) * 0.5;
        self.0.part(&format!("c{hex:06x}")).with(t, |b| {
            b.cuboid(-s, s);
        });
        self
    }
    fn cyl(&mut self, hex: u32, x: f32, y: f32, z: f32, rb: f32, rt: f32, h: f32, seg: u32) -> &mut Self {
        let profile = [Vec2::new(0.0, 0.0), Vec2::new(rb * STUD, 0.0), Vec2::new(rt * STUD, h * STUD), Vec2::new(0.0, h * STUD)];
        self.0.part(&format!("c{hex:06x}")).with(Mat4::from_translation(town::m(x, y, z)), |b| {
            b.lathe(&profile, seg);
        });
        self
    }
}

fn shape(ctx: &mut Ctx, name: &str, f: impl FnOnce(&mut Sb), mut mat: impl FnMut(&mut Ctx, u32) -> MaterialId) -> Shape {
    let mut b = Sb(MeshBuilder::new());
    f(&mut b);
    let mut out = vec![];
    for (p, md) in b.0.finish() {
        let hex = u32::from_str_radix(&p[1..], 16).unwrap_or(0xffffff);
        let mesh = ctx.renderer.add_mesh(&format!("{name}:{p}"), &md);
        out.push((mesh, mat(ctx, hex)));
    }
    Shape(out, 1.0)
}

/// A copy of a shape (hidden far below until placed), on `layer`.
fn spawn(ctx: &mut Ctx, s: &Shape, name: &str, layer: u8) -> Entity {
    let root = ctx.world.spawn((Transform::from_xyz(0.0, -150.0, 0.0), Name(name.into())));
    for (mesh, mat) in &s.0 {
        let mut mr = MeshRenderer::new(*mesh, *mat);
        mr.cast_shadows = false;
        ctx.world.spawn((Transform::default().with_scale(Vec3::splat(s.1)), Parent(root), mr));
    }
    ctx.set_layer(root, layer);
    root
}

fn place(ctx: &mut Ctx, e: Entity, at: Vec3, rot: Quat, scale: f32) {
    if let Ok(mut t) = ctx.world.get::<&mut Transform>(e) {
        t.translation = at;
        t.rotation = rot;
        t.scale = Vec3::splat(scale);
    }
}

fn hide(ctx: &mut Ctx, e: Entity) {
    place(ctx, e, Vec3::new(0.0, -150.0, 0.0), Quat::IDENTITY, 0.01);
}

/// A glowing material: linear colour `c`, emissive `c * k` (Kaigen `life_glow_material`).
fn glow(ctx: &mut Ctx, c: Vec3, k: f32) -> MaterialId {
    ctx.renderer.add_material(Material { base_color: (c * 0.3).extend(1.0), emissive: c * k, roughness: 0.5, ..Material::default() })
}

/// The sharks' circles (cave studs: centre x, y, z, radius; speed rad/s, its sign the way round): three in the basin
/// at different depths, one in the deep grotto.
const SHARKS: [[f32; 5]; 4] = [[0.0, 22.0, 0.0, 24.0, 0.16], [-6.0, 44.0, 4.0, 18.0, -0.2], [10.0, 8.0, -8.0, 14.0, 0.24], [0.0, -20.0, 92.0, 9.0, -0.3]];
/// The shark's size: the model is ~1 m long, so ~6.7 studs.
const SHARK_SCALE: f32 = 2.55;

/// The yaw that turns a shape's -z (its head) to the direction `v`.
fn head_to(v: Vec3) -> f32 {
    (-v.x).atan2(-v.z)
}

/// The fog, light and shadow settings the dive takes over (given back when you surface).
#[derive(Clone, Copy)]
struct Look {
    fog_color: Option<Vec3>,
    fog_density: f32,
    fog_start: f32,
    fog_max: f32,
    fog_height_falloff: f32,
    sun_scale: f32,
    ambient_scale: f32,
    shadows: bool,
}

#[derive(Default)]
pub struct Dive {
    // ---- the lab ----
    suit: Option<Entity>,
    suit_shown: Option<bool>,
    // ---- the dive ----
    pub on: bool,
    leaving: bool,
    /// seconds under water; the black fade between the pond and the cave (1 = black)
    pub t: f32,
    pub fade: f32,
    /// the diver, cave studs; the view (yaw 0 looks towards -z)
    pub p: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    vel: Vec3,
    /// the finger that swims, where it went down (the stick's centre) and where it was last frame
    touch: Option<i32>,
    touch_at: Vec2,
    touch_last: Vec2,
    /// fingers seen last frame (a new id = a finger that just came down)
    known: Vec<i32>,
    /// two fingers down: pinching them together swims backwards, spreading them swims faster
    pinch_on: bool,
    pinch_d0: f32,
    squeeze: f32,
    /// the swim input of the last frame (tests): forward (-1..1), fast
    pub az: f32,
    pub fast: bool,
    click_surface: bool,
    /// since the Stud Core was taken (-1: not this dive)
    core_t: f32,
    /// seconds before a pipe takes you again; pipe journeys (for tests)
    warp_cd: f32,
    pub warps: u32,
    /// pearls taken on this dive
    pub pearls_now: u32,
    dives: u32,
    // ---- the living cave ----
    built: bool,
    solids: Vec<cave::CaveBox>,
    cave_tris: usize,
    kelp: Vec<(Entity, Vec3, f32)>,
    fish: Vec<Entity>,
    manta: Vec<Entity>,
    /// the LEGO sharks (assets/props/shark.glb, the user's model repaired: one closed mesh of 6000 triangles)
    sharks: Vec<Entity>,
    bubble: Vec<Entity>,
    pearl: Vec<(Entity, Vec3)>,
    snow: Vec<Entity>,
    core: Option<Entity>,
    /// the cave's lights (entity, full intensity) and the helmet lamp
    lights: Vec<(Entity, f32)>,
    lamp: Option<Entity>,
    cam: Option<Entity>,
    fov: f32,
    look: Option<Look>,
    hud: String,
}

impl Dive {
    /// The suit in the lab and the cave with its living things (built once per play screen; hidden until you dive).
    pub fn init(&mut self, ctx: &mut Ctx, assets: &mut TownAssets) {
        // ---- the lab: the suit in the corner behind the bookshelves ----
        let suit = shape(
            ctx,
            "diving suit",
            |t| {
                let (brass, yel, dark, glass) = (0xc9a227, 0xe8b923, 0x2b2f36, 0x2a5d8a);
                t.bx(0x6b4a2a, 2.6, 0.4, 2.0, 0.0, 0.0, 0.0); // the wooden stand
                t.bx(dark, 0.9, 0.8, 1.0, -0.55, 0.4, 0.0).bx(dark, 0.9, 0.8, 1.0, 0.55, 0.4, 0.0); // boots
                t.bx(yel, 0.85, 1.5, 0.85, -0.5, 1.2, 0.0).bx(yel, 0.85, 1.5, 0.85, 0.5, 1.2, 0.0); // legs
                t.bx(yel, 2.1, 1.8, 1.1, 0.0, 2.7, 0.0); // the body
                t.bx(brass, 2.2, 0.3, 1.2, 0.0, 3.0, 0.0); // the belt
                t.bx(yel, 0.6, 1.6, 0.7, -1.4, 2.9, 0.0).bx(yel, 0.6, 1.6, 0.7, 1.4, 2.9, 0.0); // arms
                t.bx(brass, 1.8, 0.4, 1.6, 0.0, 4.5, 0.0); // the collar
                t.cyl(brass, 0.0, 4.9, 0.0, 1.0, 1.0, 1.5, 16).cyl(brass, 0.0, 6.4, 0.0, 1.0, 0.55, 0.35, 16); // the helmet
                t.bx(glass, 1.0, 0.9, 0.3, 0.0, 5.2, 0.95); // the window
                t.bx(brass, 1.2, 0.15, 0.35, 0.0, 6.1, 0.98).bx(brass, 1.2, 0.15, 0.35, 0.0, 5.05, 0.98);
                t.bx(dark, 0.35, 0.35, 1.4, 0.0, 3.6, -0.9); // the air hose
            },
            |ctx, hex| assets.material(ctx, hex),
        );
        let e = spawn(ctx, &suit, "diving suit", rooms::HIDDEN_LAYER);
        place(ctx, e, town::m(SUIT_AT.x, SUIT_AT.y, SUIT_AT.z), Quat::from_rotation_y(0.6), 1.0);
        for d in engine::scene::descendants(&ctx.world, e) {
            if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(d) {
                mr.cast_shadows = true;
            }
        }
        self.suit = Some(e);
        self.suit_shown = None;

        // ---- the cave ----
        let c = cave::build(ctx, assets);
        self.solids = c.solids;
        self.cave_tris = c.triangles;
        let plain = |ctx: &mut Ctx, hex: u32| ctx.renderer.add_material(Material { name: format!("cave {hex:06x}"), ..Material::color(hex).rough(0.55) });
        let kelp: Vec<Shape> = [(0x7a9a2e, 0.7), (0x2f8a4a, 0.6), (0x2a9a9a, 0.5)]
            .into_iter()
            .map(|(hex, d)| shape(ctx, "kelp", |t| { t.bx(hex, 0.2, 16.0, d, 0.0, 0.0, 0.0); }, plain))
            .collect();
        for i in 0..KELP {
            // the kelp forest in the south-west, a few strands elsewhere
            let ii = i as i32;
            let x = if i < 30 { -36.0 + hash(ii, 11) * 28.0 } else { -30.0 + hash(ii, 12) * 60.0 };
            let z = if i < 30 { -36.0 + hash(ii, 13) * 30.0 } else { -30.0 + hash(ii, 14) * 40.0 };
            let s = 1.4 + hash(ii, 15) * 1.6;
            let e = spawn(ctx, &kelp[i % 3], "kelp", CAVE_LAYER);
            self.kelp.push((e, Vec3::new(x, floor(x, z), z), s));
        }
        let fish_b = shape(ctx, "fish", |t| { t.bx(0x7fb0cc, 0.4, 0.4, 1.0, 0.0, -0.2, 0.0).bx(0x355c80, 0.08, 0.45, 0.35, 0.0, -0.22, 0.65); }, plain);
        let fish_y = shape(ctx, "fish yellow", |t| { t.bx(0xf2cc4d, 0.4, 0.4, 1.0, 0.0, -0.2, 0.0).bx(0xb08a20, 0.08, 0.45, 0.35, 0.0, -0.22, 0.65); }, plain);
        for i in 0..FISH {
            let s = if i % 7 == 3 { &fish_y } else { &fish_b };
            let e = spawn(ctx, s, "fish", CAVE_LAYER);
            self.fish.push(e);
        }
        let manta = shape(
            ctx,
            "manta",
            |t| {
                t.bx(0x0f1a2a, 2.0, 0.35, 3.0, 0.0, -0.17, 0.0);
                t.rot(0x12203a, Vec3::new(3.4, 0.2, 2.0), Vec3::new(-2.4, 0.0, 0.2), Vec3::new(0.0, 0.0, 0.12));
                t.rot(0x12203a, Vec3::new(3.4, 0.2, 2.0), Vec3::new(2.4, 0.0, 0.2), Vec3::new(0.0, 0.0, -0.12));
                t.bx(0x8a96a8, 1.4, 0.1, 2.0, 0.0, 0.18, 0.0); // the pale stripe
                t.bx(0x0f1a2a, 0.15, 0.15, 3.0, 0.0, -0.07, 2.8); // the tail
            },
            plain,
        );
        for _ in 0..MANTAS {
            let e = spawn(ctx, &manta, "manta", CAVE_LAYER);
            self.manta.push(e);
        }
        for _ in 0..SHARKS.len() {
            let opts = engine::gltf_loader::GltfOpts { auto_lod: true, ..Default::default() };
            let e = ctx.load_gltf_with("assets/props/shark.glb", Transform::from_xyz(0.0, -200.0, 0.0), opts);
            ctx.set_layer(e, CAVE_LAYER);
            self.sharks.push(e);
        }
        // bubbles and snow: unit cubes (scaled), glowing softly
        let bub_mat = ctx.renderer.add_material(Material { base_color: Vec4::new(0.5, 0.7, 0.85, 1.0), emissive: Vec3::new(0.3, 0.45, 0.6), roughness: 0.1, ..Material::default() }.transparent(0.45));
        // (spheres one stud across: bubbles and specks are round)
        let ball = ctx.renderer.builtin.sphere;
        let bub = Shape(vec![(ball, bub_mat)], STUD * 0.5);
        for _ in 0..BUBBLES {
            let e = spawn(ctx, &bub, "bubble", CAVE_LAYER);
            self.bubble.push(e);
        }
        let snow_mat = ctx.renderer.add_material(Material { base_color: Vec4::new(0.1, 0.12, 0.14, 1.0), emissive: Vec3::new(0.55, 0.65, 0.7), roughness: 1.0, ..Material::default() });
        let snow = Shape(vec![(ball, snow_mat)], STUD);
        for _ in 0..SNOW {
            let e = spawn(ctx, &snow, "snow", CAVE_LAYER);
            self.snow.push(e);
        }
        // glowing tufts: magenta and violet in the grotto, cyan and gold here and there on the ledges and the floor
        let tuft_mats = [
            glow(ctx, Vec3::new(0.85, 0.05, 0.4), 3.5),
            glow(ctx, Vec3::new(0.6, 0.1, 0.8), 3.5),
            glow(ctx, Vec3::new(0.1, 0.85, 0.9), 2.2),
            glow(ctx, Vec3::new(1.0, 0.75, 0.2), 2.2),
        ];
        let tufts: Vec<Shape> = tuft_mats
            .iter()
            .map(|m| {
                let m = *m;
                shape(
                    ctx,
                    "tuft",
                    |t| {
                        for i in 0..5 {
                            let a = i as f32 * 1.256;
                            t.rot(0xffffff, Vec3::new(0.25, 1.6 + 0.3 * (i % 2) as f32, 0.25), Vec3::new(a.sin() * 0.35, 0.8, a.cos() * 0.35), Vec3::new(a.cos() * 0.35, 0.0, -a.sin() * 0.35));
                        }
                    },
                    |_, _| m,
                )
            })
            .collect();
        for i in 0..TUFTS {
            let ii = i as i32;
            let grotto = i < 22;
            let x = if grotto { 14.0 + hash(ii, 21) * 24.0 } else { -36.0 + hash(ii, 22) * 72.0 };
            let z = if grotto { -38.0 + hash(ii, 23) * 20.0 } else { -36.0 + hash(ii, 24) * 72.0 };
            let s = 0.8 + hash(ii, 25) * 0.9;
            let e = spawn(ctx, &tufts[if grotto { i % 2 } else { 2 + i % 2 }], "tuft", CAVE_LAYER);
            place(ctx, e, world(Vec3::new(x, floor(x, z), z)), Quat::from_rotation_y(hash(ii, 26) * 6.28), s);
        }
        // the pearls: on ledges, in the grotto, by the boat, under the arch, in the kelp, by the whale tail
        let pearls = [[-34.0, 0.0, -30.0], [28.0, 0.0, -32.0], [-26.0, 0.0, 22.0], [-20.0, 36.0, 0.0], [2.0, 0.0, -14.0], [34.0, 0.0, 30.0], [-8.0, 0.0, 34.0], [18.0, 0.0, 2.0]];
        let pearl_mat = glow(ctx, Vec3::new(0.85, 0.95, 1.0), 3.5);
        let pearl = shape(ctx, "pearl", |t| { t.cyl(0xffffff, 0.0, -0.4, 0.0, 0.7, 0.7, 0.8, 12).cyl(0xffffff, 0.0, 0.4, 0.0, 0.4, 0.4, 0.3, 12); }, |_, _| pearl_mat);
        for p in pearls {
            let mut p = Vec3::from(p);
            if p.y == 0.0 {
                p.y = floor(p.x, p.z) + 1.4;
            }
            let e = spawn(ctx, &pearl, "pearl", CAVE_LAYER);
            self.pearl.push((e, p));
        }
        let core_mat = glow(ctx, Vec3::new(1.0, 0.78, 0.15), 1.6);
        let core = shape(ctx, "stud core", |t| { t.bx(0xffffff, 2.0, 1.2, 2.0, 0.0, -0.6, 0.0).cyl(0xffffff, 0.0, 0.6, 0.0, 0.62, 0.62, 0.35, 12); }, |_, _| core_mat);
        self.core = Some(spawn(ctx, &core, "stud core", CAVE_LAYER));
        let surf_mat = glow(ctx, Vec3::new(0.55, 0.85, 1.0), 2.5);
        let surf = shape(ctx, "cave surface", |t| { t.bx(0xffffff, 12.0, 0.4, 12.0, 0.0, 0.0, 0.0); }, |_, _| surf_mat);
        let e = spawn(ctx, &surf, "cave surface", CAVE_LAYER);
        place(ctx, e, world(Vec3::new(0.0, CAVE_TOP + 19.0, 0.0)), Quat::IDENTITY, 1.0);
        // the lights: daylight down the hole, the grotto, the kelp, the trench, the core, the boat; and your helmet lamp
        let lights: [([f32; 3], u32, f32, f32); 10] = [
            ([0.0, CAVE_TOP - 4.0, 0.0], 0x8cd9ff, 40.0, 90.0),
            ([24.0, 14.0, -28.0], 0xd940d9, 22.0, 60.0),
            ([-22.0, 22.0, -22.0], 0x4df2b3, 22.0, 50.0),
            ([0.0, -6.0, 26.0], 0x3399ff, 18.0, 45.0),
            // the warp pipe's green glow; the core room's warm light over the core
            ([0.0, -16.0, 100.0], 0x5cff7a, 16.0, 50.0),
            ([-24.0, 10.0, 18.0], 0x66ccff, 18.0, 45.0),
            // the tunnel (a faint blue glow half way) and the deep grotto (violet and cyan over its crystals)
            ([0.0, -19.0, 58.0], 0x3d7bff, 14.0, 35.0),
            ([-10.0, -12.0, 88.0], 0xa64dff, 28.0, 90.0),
            ([11.0, -14.0, 102.0], 0x40d9ff, 28.0, 90.0),
            ([0.0, cave::CR_Y0 + 12.0, 139.0], 0xffbf40, 24.0, 80.0),
        ];
        for (p, hex, r, k) in lights {
            let e = ctx.world.spawn((Transform::from_translation(world(Vec3::from(p))), PointLight { color: srgb_hex(hex).truncate(), intensity: 0.0, range: r * STUD }));
            self.lights.push((e, k));
        }
        self.lamp = Some(ctx.world.spawn((Transform::from_xyz(0.0, -150.0, 0.0), PointLight { color: Vec3::new(1.0, 0.96, 0.85), intensity: 0.0, range: 14.0 * STUD })));
        self.built = true;
    }

    /// The lab follows the quest: the suit stands there until taken (or the deck is printed).
    pub fn lab(&mut self, ctx: &mut Ctx, s: &GameSave) {
        let want = suit_here(s);
        if self.suit_shown != Some(want) {
            self.suit_shown = Some(want);
            if let Some(e) = self.suit {
                ctx.set_layer(e, if want { rooms::layer_of(Some(rooms::RoomId::Lab)) } else { rooms::HIDDEN_LAYER });
            }
        }
    }

    /// Into the water (Kaigen `dive_start`): in the shaft above the hole, looking down, the screen still black.
    pub fn start(&mut self, ctx: &mut Ctx) {
        self.on = true;
        self.leaving = false;
        self.t = 0.0;
        self.fade = 1.0;
        self.p = Vec3::new(0.0, CAVE_TOP + 9.0, 0.0);
        self.yaw = 0.0;
        self.pitch = -1.25;
        self.vel = Vec3::ZERO;
        self.touch = None;
        self.known = ctx.input.touch.touches.iter().map(|t| t.id).collect();
        self.pinch_on = false;
        self.core_t = -1.0;
        self.pearls_now = 0;
        self.dives += 1;
        self.hud.clear();
        // the dive's camera: the cave only, a wide lens
        let cam = ctx.world.spawn((Transform::from_translation(world(self.p)), Camera { fov_y_deg: 90.0, near: 0.05, far: 70.0, layers: 1 << CAVE_LAYER }, Name("dive camera".into())));
        self.cam = Some(cam);
        ctx.set_main_camera(cam);
        ctx.input.touch.set_stick_enabled(false); // every finger swims and steers
        let s = &ctx.renderer.settings;
        self.look = Some(Look {
            fog_color: s.fog_color,
            fog_density: s.fog_density,
            fog_start: s.fog_start,
            fog_max: s.fog_max,
            fog_height_falloff: s.fog_height_falloff,
            sun_scale: s.sun_scale,
            ambient_scale: s.ambient_scale,
            shadows: s.shadows,
        });
        for (e, k) in &self.lights {
            if let Ok(mut l) = ctx.world.get::<&mut PointLight>(*e) {
                l.intensity = *k;
            }
        }
        if let Some(Ok(mut l)) = self.lamp.map(|e| ctx.world.get::<&mut PointLight>(e)) {
            l.intensity = 14.0;
        }
    }

    /// Back up (the fade is black): the village camera and look return, the cave goes dark.
    fn stop(&mut self, ctx: &mut Ctx) {
        self.on = false;
        if let Some(c) = self.cam.take() {
            ctx.despawn(c); // the village camera draws again (no MainCamera marker left, as the fight camera does)
        }
        if let Some(l) = self.look.take() {
            let s = &mut ctx.renderer.settings;
            s.fog_color = l.fog_color;
            s.fog_density = l.fog_density;
            s.fog_start = l.fog_start;
            s.fog_max = l.fog_max;
            s.fog_height_falloff = l.fog_height_falloff;
            s.sun_scale = l.sun_scale;
            s.ambient_scale = l.ambient_scale;
            s.shadows = l.shadows;
        }
        for (e, _) in &self.lights {
            if let Ok(mut l) = ctx.world.get::<&mut PointLight>(*e) {
                l.intensity = 0.0;
            }
        }
        if let Some(Ok(mut l)) = self.lamp.map(|e| ctx.world.get::<&mut PointLight>(e)) {
            l.intensity = 0.0;
        }
        for e in self.fish.iter().chain(&self.snow).chain(&self.manta).chain(&self.bubble).chain(&self.sharks) {
            hide(ctx, *e);
        }
        ctx.input.touch.set_stick_enabled(true);
        ctx.ui.set("dive", "");
        ctx.ui.set("dive_fx", "");
        self.hud.clear();
    }

    /// The screen goes away while diving (the play screen exits): put everything back.
    pub fn abort(&mut self, ctx: &mut Ctx) {
        if self.on {
            self.stop(ctx);
        }
    }

    /// How deep you are, 0 at the hole .. 1 at the bottom of the trench (the water's colour, the light).
    pub fn depth(&self) -> f32 {
        ((CAVE_TOP - self.p.y) / (CAVE_TOP + 24.0)).clamp(0.0, 1.0)
    }

    /// The view direction.
    fn forward(&self) -> Vec3 {
        let cp = self.pitch.cos();
        Vec3::new(-self.yaw.sin() * cp, self.pitch.sin(), -self.yaw.cos() * cp)
    }

    /// Where `p` is: the basin, the tunnel (past the basin's north wall, in the tunnel's cross-section) or the deep
    /// grotto behind it.
    fn zone(p: Vec3) -> cave::Zone {
        if p.z > cave::DEEP_Z1 + 2.0 && p.y < cave::CR_Y1 {
            cave::Zone::Core
        } else if p.z > cave::DEEP_Z0 {
            cave::Zone::Deep
        } else if p.z > CAVE_R - 1.0 && p.x.abs() < cave::TUN_W * 0.5 && p.y < cave::TUN_Y1 {
            cave::Zone::Tunnel
        } else {
            cave::Zone::Basin
        }
    }

    /// Keep the diver in the water: in the basin (above the floor, below the ceiling but up the hole), in the
    /// tunnel's cross-section, in the deep grotto (its near wall open only at the tunnel), out of the rocks.
    fn collide(&mut self) {
        let r = 1.2;
        let mut p = self.p;
        let hw = cave::TUN_W * 0.5 - r;
        let in_mouth = p.x.abs() < hw + 0.3 && p.y > cave::TUN_Y0 + r - 0.3 && p.y < cave::TUN_Y1 - r + 0.3;
        if Self::zone(self.p) == cave::Zone::Core {
            // the core room (closed: the way out is its pipe, or Surface)
            let cw = cave::CR_W * 0.5 - r;
            p.x = p.x.clamp(-cw, cw);
            p.y = p.y.clamp(cave::CR_Y0 + r, cave::CR_Y1 - r);
            p.z = p.z.clamp(cave::CR_Z0 + r, cave::CR_Z1 - r);
        } else if p.z > cave::DEEP_Z0 + r || (p.z > cave::DEEP_Z0 - r && !in_mouth && Self::zone(self.p) == cave::Zone::Deep) {
            // the deep grotto
            let dw = cave::DEEP_W * 0.5 - r;
            p.x = p.x.clamp(-dw, dw);
            p.y = p.y.clamp(cave::DEEP_Y0 + r, cave::DEEP_Y1 - r);
            p.z = p.z.min(cave::DEEP_Z1 - r);
            if p.z < cave::DEEP_Z0 + r && !in_mouth {
                p.z = cave::DEEP_Z0 + r;
            }
        } else if p.z > CAVE_R - r - 0.5 && (in_mouth || Self::zone(self.p) == cave::Zone::Tunnel) {
            // the tunnel (and its mouth in the north wall)
            p.x = p.x.clamp(-hw, hw);
            p.y = p.y.clamp(cave::TUN_Y0 + r, cave::TUN_Y1 - r);
        } else {
            let in_hole = p.x.abs() < CAVE_HOLE - 1.0 && p.z.abs() < CAVE_HOLE - 1.0;
            p.x = p.x.clamp(-CAVE_R + r + 0.5, CAVE_R - r - 0.5);
            p.z = p.z.clamp(-CAVE_R + r + 0.5, CAVE_R - r - 0.5);
            p.y = p.y.min(if in_hole { CAVE_TOP + 16.0 } else { CAVE_TOP - r });
            p.y = p.y.max(floor(p.x, p.z) + r);
        }
        for b in &self.solids {
            if p.x < b.x0 - r || p.x > b.x1 + r || p.y < b.y0 - r || p.y > b.y1 + r || p.z < b.z0 - r || p.z > b.z1 + r {
                continue;
            }
            let pen = [p.x - (b.x0 - r), (b.x1 + r) - p.x, p.y - (b.y0 - r), (b.y1 + r) - p.y, p.z - (b.z0 - r), (b.z1 + r) - p.z];
            let best = (0..6).min_by(|a, b| pen[*a].total_cmp(&pen[*b])).unwrap_or(0);
            match best {
                0 => p.x = b.x0 - r,
                1 => p.x = b.x1 + r,
                2 => p.y = b.y0 - r,
                3 => p.y = b.y1 + r,
                4 => p.z = b.z0 - r,
                _ => p.z = b.z1 + r,
            }
        }
        self.p = p;
    }

    /// Every frame while diving: steer, swim, collect, animate (Kaigen `dive_update`). Returns true when it is over
    /// (the fade is black: back at the pond).
    pub fn update(&mut self, ctx: &mut Ctx, data: &mut Data, life: &mut Life, actions: &[String], dt: f32) -> bool {
        if !self.on {
            return false;
        }
        let dt = dt.min(0.1);
        self.t += dt;
        self.fade = if self.leaving { (self.fade + dt / 0.45).min(1.0) } else { (self.fade - dt / 0.6).max(0.0) };
        if self.leaving && self.fade >= 1.0 {
            self.stop(ctx);
            return true;
        }
        self.click_surface |= actions.iter().any(|a| a == "dive_surface");
        // ---- steering: a held finger swims forward and steers like a stick from where it went down; the mouse looks ----
        let (sw, sh) = (ctx.canvas.client_width() as f32, ctx.canvas.client_height() as f32);
        let short = sw.min(sh).max(1.0);
        let touches: Vec<(i32, Vec2)> = ctx.input.touch.touches.iter().map(|t| (t.id, t.pos)).collect();
        let fingers = touches.len();
        let mut fwd_touch = false;
        for (id, pos) in &touches {
            if self.touch.is_none() && !self.known.contains(id) {
                let start = ctx.input.touch.touches.iter().find(|t| t.id == *id).map(|t| t.start).unwrap_or(*pos);
                self.touch = Some(*id);
                self.touch_at = start;
                self.touch_last = start;
            }
            if self.touch == Some(*id) {
                fwd_touch = true;
                let now = *pos;
                if fingers >= 2 {
                    // pinching: no steering; after it the stick starts from here
                    self.touch_at = now;
                    self.touch_last = now;
                    continue;
                }
                let jx = ((now.x - self.touch_at.x) / (0.25 * short)).clamp(-1.0, 1.0);
                let jy = ((now.y - self.touch_at.y) / (0.25 * short)).clamp(-1.0, 1.0);
                self.yaw -= 1.5 * jx * dt; // a stick: hold it out to keep turning
                self.pitch -= 1.0 * jy * dt;
                self.yaw -= (now.x - self.touch_last.x) * 2.2 / short; // and the drag turns the view at once
                self.pitch -= (now.y - self.touch_last.y) * 2.2 / short;
                self.touch_last = now;
            }
        }
        if self.touch.is_some_and(|id| !touches.iter().any(|t| t.0 == id)) {
            self.touch = None;
        }
        self.known = touches.iter().map(|t| t.0).collect();
        // the pinch: together (closer than when the second finger came down) swims backwards, apart swims faster
        self.squeeze = 0.0;
        if fingers >= 2 {
            let d = touches[0].1.distance(touches[1].1);
            if !self.pinch_on {
                self.pinch_on = true;
                self.pinch_d0 = d;
            }
            self.squeeze = ((self.pinch_d0 - d) / (0.12 * short)).clamp(-1.0, 1.0);
        } else {
            self.pinch_on = false;
        }
        let fast_touch = self.squeeze < -0.2;
        let back_touch = self.squeeze > 0.2;
        let inp = &ctx.input;
        let look = inp.button(MouseButton::Right) || (inp.button(MouseButton::Left) && !fwd_touch);
        if look && fingers == 0 {
            self.yaw -= inp.pointer_delta.x * 0.0035;
            self.pitch -= inp.pointer_delta.y * 0.0035;
        }
        let k = |c: &str| if inp.key(c) { 1.0 } else { 0.0 };
        self.yaw += (k("ArrowLeft") - k("ArrowRight")) * 1.8 * dt;
        self.pitch += (k("ArrowUp") - k("ArrowDown")) * 1.2 * dt;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
        // ---- swimming: along the view, sideways, straight up and down ----
        let f = self.forward();
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        let ax = k("KeyD") - k("KeyA");
        let mut az = k("KeyW") - k("KeyS");
        let ay = k("Space") - k("KeyQ").max(k("ControlLeft"));
        if back_touch {
            az = -self.squeeze.clamp(0.4, 1.0); // pinched: backwards (stronger the more you pinch)
        } else if fwd_touch && fingers < 2 {
            az = az.max(0.7);
        } else if fast_touch {
            az = 1.0;
        }
        let fast = k("ShiftLeft").max(k("ShiftRight")) > 0.0 || fast_touch;
        self.az = az;
        self.fast = fast;
        let speed = if fast { 22.0 } else { 9.0 };
        let mut want = (f * az + right * ax + Vec3::Y * ay) * speed;
        if want.length() > speed {
            want = want.normalize() * speed;
        }
        if self.fade > 0.6 || self.leaving {
            want = Vec3::ZERO;
        }
        let kf = 1.0 - (-4.5 * dt).exp();
        self.vel = self.vel.lerp(want, kf);
        self.p += self.vel * dt;
        self.collide();
        // a warp pipe: down into its opening and out of the other one (a moment of black, the pipe's sound)
        self.warp_cd = (self.warp_cd - dt).max(0.0);
        if self.warp_cd <= 0.0 && !self.leaving {
            let near = |at: Vec3, top: f32, p: Vec3| Vec2::new(p.x - at.x, p.z - at.z).length() < cave::PIPE_R - 0.4 && p.y < top + 1.6;
            let to = match Self::zone(self.p) {
                cave::Zone::Deep if near(cave::PIPE_AT, cave::PIPE_TOP, self.p) => Some((Vec3::new(cave::PIPE2_AT.x, cave::PIPE2_TOP + 3.0, cave::PIPE2_AT.z), std::f32::consts::PI)),
                cave::Zone::Core if near(cave::PIPE2_AT, cave::PIPE2_TOP, self.p) => Some((Vec3::new(cave::PIPE_AT.x, cave::PIPE_TOP + 3.0, cave::PIPE_AT.z - 2.0), 0.0)),
                _ => None,
            };
            if let Some((at, yaw)) = to {
                self.p = at;
                self.vel = Vec3::ZERO;
                self.yaw = yaw;
                self.pitch = 0.0;
                self.fade = 1.0;
                self.warp_cd = 1.5;
                self.warps += 1;
                data.sound.sfx(ctx, Sfx::Pop, 0.45, 0.9);
            }
        }
        // up the hole: back to the pond
        let esc = ctx.input.key_pressed("Escape");
        if !self.leaving && (self.p.y > CAVE_TOP + 12.0 || self.click_surface || esc) && self.t > 1.0 {
            self.leaving = true;
            data.sound.sfx(ctx, Sfx::Splash, 1.0, 0.7);
        }
        self.click_surface = false;
        // ---- the camera is your eyes, the helmet lamp a little ahead ----
        if let Some(cam) = self.cam {
            let eye = world(self.p);
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(cam) {
                *t = Transform::from_translation(eye).looking_at(eye + f, if f.y.abs() > 0.999 { Vec3::Z } else { Vec3::Y });
            }
            // a wide lens under water: at least 105 degrees across (a phone held upright saw a slit before), 75 up
            // and down, never more than 120 up and down (Kaigen fea1c7b)
            let (w, h) = ctx.renderer.size();
            let aspect = (w as f32 / h.max(1) as f32).max(0.2);
            let across = 105f32.to_radians();
            let v = (2.0 * ((across * 0.5).tan() / aspect).atan()).to_degrees();
            self.fov = v.clamp(75.0, 120.0);
            if let Ok(mut c) = ctx.world.get::<&mut Camera>(cam) {
                c.fov_y_deg = self.fov;
            }
        }
        if let Some(l) = self.lamp {
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(l) {
                t.translation = world(self.p + f * 3.0);
            }
        }
        self.atmosphere(ctx);
        // ---- the living cave ----
        let t = self.t;
        for (i, (e, at, s)) in self.kelp.clone().into_iter().enumerate() {
            // sways about its root, taller strands more
            let fi = i as f32;
            let sw = 0.05 + 0.02 * s;
            let q = Quat::from_euler(EulerRot::YXZ, fi * 1.7, (t * 0.7 + fi).sin() * sw, (t * 0.55 + fi * 0.6).cos() * sw);
            place(ctx, e, world(at), q, s);
        }
        const SCHOOL: [[f32; 5]; 3] = [[-14.0, -18.0, 14.0, 22.0, 0.35], [18.0, 14.0, 18.0, 34.0, -0.28], [-6.0, 10.0, 12.0, 46.0, 0.22]];
        for i in 0..FISH {
            // three schools circling, each fish a little apart and bobbing
            let (sc, j) = (i / 14, (i % 14) as f32);
            let s = SCHOOL[sc];
            let a = t * s[4] + j * 0.09;
            let rr = s[2] + (hash(i as i32, 31) - 0.5) * 4.0;
            let p = Vec3::new(s[0] + a.sin() * rr, s[3] + (hash(i as i32, 32) - 0.5) * 5.0 + (t * 1.3 + j).sin() * 0.6, s[1] + a.cos() * rr);
            let v = Vec3::new(a.cos(), 0.0, -a.sin()) * s[4].signum();
            let yaw = head_to(v) + (t * 6.0 + j).sin() * 0.15;
            place(ctx, self.fish[i], world(p), Quat::from_rotation_y(yaw), 1.4);
        }
        for (i, sh) in SHARKS.iter().enumerate() {
            // the sharks: slow circles (centre x, y, z, radius, speed), the tail swaying the body a little
            let a = t * sh[4] + i as f32 * 2.1;
            let p = Vec3::new(sh[0] + a.sin() * sh[3], sh[1] + (t * 0.5 + i as f32).sin() * 1.5, sh[2] + a.cos() * sh[3]);
            let v = Vec3::new(a.cos(), 0.0, -a.sin()) * sh[4].signum();
            // (the model's head is its -z, as the other shapes')
            let yaw = head_to(v) + (t * 2.2 + i as f32).sin() * 0.12;
            let bank = -0.15 * sh[4].signum();
            if let Some(&e) = self.sharks.get(i) {
                place(ctx, e, world(p), Quat::from_rotation_y(yaw) * Quat::from_rotation_z(bank), SHARK_SCALE);
            }
        }
        for i in 0..MANTAS {
            // two mantas gliding wide circles, wings flapping slowly
            let fi = i as f32;
            let sp = if i == 1 { -0.12 } else { 0.15 };
            let a = t * sp + fi * 2.0;
            let rr = if i == 1 { 26.0 } else { 20.0 };
            let p = Vec3::new(a.sin() * rr, if i == 1 { 56.0 } else { 34.0 } + (t * 0.4).sin() * 3.0, a.cos() * rr);
            let v = Vec3::new(a.cos(), 0.0, -a.sin()) * sp.signum();
            let bank = if i == 1 { 0.28 } else { -0.28 } + (t * 1.2).sin() * 0.18;
            let q = Quat::from_rotation_y(head_to(v)) * Quat::from_rotation_z(bank);
            let flap = 1.0 + 0.12 * (t * 1.6 + fi).sin();
            if let Ok(mut tr) = ctx.world.get::<&mut Transform>(self.manta[i]) {
                tr.translation = world(p);
                tr.rotation = q;
                tr.scale = Vec3::new(1.6 * flap, 1.6, 1.6);
            }
        }
        const VENTS: [[f32; 2]; 6] = [[-10.0, 6.0], [12.0, -6.0], [26.0, -26.0], [-28.0, -14.0], [4.0, 30.0], [30.0, 22.0]];
        for i in 0..BUBBLES {
            // rising from six vents, wobbling
            let (v, ii, fi) = (VENTS[i % 6], i as i32, i as f32);
            let base = floor(v[0], v[1]);
            let rise = (t * (5.0 + hash(ii, 41) * 4.0) + hash(ii, 42) * 60.0).rem_euclid(CAVE_TOP - base);
            let wob = (t * 3.0 + fi).sin() * 0.5;
            let sz = 0.35 + hash(ii, 43) * 0.5;
            place(ctx, self.bubble[i], world(Vec3::new(v[0] + wob, base + rise, v[1] + (t * 2.5 + fi).cos() * 0.5)), Quat::IDENTITY, sz);
        }
        for i in 0..SNOW {
            // marine snow: sinking slowly, wrapped into a 28-stud box around the diver
            const B: f32 = 28.0;
            let ii = i as i32;
            let o = Vec3::new(hash(ii, 51) * B, hash(ii, 52) * B - t * (0.4 + hash(ii, 53) * 0.5), hash(ii, 54) * B);
            let d = o - self.p;
            let rel = Vec3::new(d.x.rem_euclid(B), d.y.rem_euclid(B), d.z.rem_euclid(B));
            let at = self.p + rel - Vec3::splat(B * 0.5);
            let sz = 0.06 + hash(ii, 55) * 0.07;
            place(ctx, self.snow[i], world(at), Quat::from_rotation_y(fi_rot(i, t)), sz);
        }
        // ---- the pearls and the Stud Core ----
        for i in 0..PEARLS {
            let (e, p) = self.pearl[i];
            let s = &mut data.save.data;
            if (s.dive_found >> i) & 1 == 1 {
                hide(ctx, e);
                continue;
            }
            place(ctx, e, world(Vec3::new(p.x, p.y + (t * 2.0 + i as f32).sin() * 0.3, p.z)), Quat::from_rotation_y(t * 1.2), 1.0);
            if (self.p - p).length() < 3.0 {
                s.dive_found |= 1 << i;
                s.studs += 5;
                self.pearls_now += 1;
                let n = (s.dive_found & ((1 << PEARLS) - 1)).count_ones();
                data.save.mark();
                data.sound.sfx(ctx, Sfx::Coin, 1.3, 0.8);
                life.toast(&format!("A glowing pearl! ({n}/{PEARLS}) \u{b7} +5 studs"));
            }
        }
        let s = &mut data.save.data;
        if let Some(core) = self.core {
            if !s.has_part && !s.has_deck {
                let cp = CORE_AT + Vec3::Y * (t * 1.5).sin() * 0.4;
                let pulse = 1.0 + 0.08 * (t * 4.0).sin();
                place(ctx, core, world(cp), Quat::from_rotation_y(t * 0.8), pulse);
                if (self.p - cp).length() < 5.0 {
                    s.has_part = true;
                    data.save.mark();
                    self.core_t = 0.0;
                    data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.9);
                    life.toast("You found the Stud Core! Swim back up and bring it to Prof. Quill.");
                }
            } else {
                hide(ctx, core);
            }
        }
        if self.core_t >= 0.0 {
            self.core_t += dt;
        }
        false
    }

    /// The water: fog and light darker and bluer the deeper you are (Kaigen's `il->water` / `il->depth` and the sun).
    fn atmosphere(&self, ctx: &mut Ctx) {
        let dep = self.depth();
        let shallow = srgb_hex(0x1d6a80).truncate();
        let deep = srgb_hex(0x020c18).truncate();
        let col = shallow.lerp(deep, dep.powf(0.7));
        let s = &mut ctx.renderer.settings;
        s.fog_color = Some(col);
        s.clear_color = col;
        s.sky_zenith = col;
        s.sky_horizon = col;
        s.fog_density = 0.045 + 0.11 * dep;
        s.fog_start = 0.0;
        s.fog_max = 1.0;
        s.fog_height_falloff = 0.0;
        s.sun_scale = 0.3 * (1.0 - dep) + 0.03;
        s.ambient_scale = 0.35 * (1.0 - dep) + 0.06;
        s.shadows = false;
    }

    /// The HUD (Kaigen `dive_draw`): the water's tint and edge, depth and pearls, the hint, the Surface button, the fade.
    pub fn draw(&mut self, ctx: &mut Ctx, s: &GameSave, touch: bool) {
        if !self.on {
            return;
        }
        let dep = self.depth();
        let (sw, _) = (ctx.canvas.client_width() as f32, ctx.canvas.client_height() as f32);
        let narrow = sw < 600.0;
        let n = (s.dive_found & ((1 << PEARLS) - 1)).count_ones();
        let metres = ((CAVE_TOP + 4.0 - self.p.y) * STUD).max(0.0);
        let found = s.has_part || s.has_deck;
        let pill = if narrow {
            format!(
                "<span class='m'>{metres:.0} m</span><span>\u{25c6} {n}/{PEARLS}</span><span class='{}'>{}</span>",
                if found { "ok" } else { "core" },
                if found { "Core \u{2713}" } else { "Core: deep" }
            )
        } else {
            format!(
                "<span class='m'>Depth {metres:.1} m</span><span>\u{25c6} Pearls {n}/{PEARLS}</span><span class='{}'>{}</span>",
                if found { "ok" } else { "core" },
                if found { "Stud Core \u{2713}" } else { "Find the Stud Core (deep down)" }
            )
        };
        let hint = if self.t < 9.0 {
            let a = (9.0 - self.t).clamp(0.0, 1.0);
            format!(
                "<div class='bh-dive-hint' style='opacity:{a:.2}'>{}</div>",
                if touch { "Hold a finger to swim \u{b7} move it to steer \u{b7} pinch: back \u{b7} spread: faster" } else { "W / S swim \u{b7} hold a mouse button and drag to look \u{b7} Space up \u{b7} Q down \u{b7} Shift faster" }
            )
        } else {
            String::new()
        };
        let fade = if self.fade > 0.0 { format!("<div class='bh-dive-fade' style='opacity:{:.2}'></div>", self.fade) } else { String::new() };
        let html = format!(
            "<div class='bh-dive-tint' style='background:linear-gradient(180deg,rgba(41,148,230,{:.2}),rgba(2,16,30,{:.2}))'></div>\
             <div class='bh-dive-edge' style='background:radial-gradient(ellipse at center,rgba(0,0,0,0) 45%,rgba(1,8,16,{:.2}) 100%)'></div>\
             <div class='bh-dive-pill{}'>{pill}</div>{hint}\
             <button class='bh-dive-up' data-action='dive_surface'>\u{25b2} Surface</button>{fade}",
            0.08 * (1.0 - dep),
            0.08 + 0.2 * dep,
            0.35 + 0.25 * dep,
            if narrow { " narrow" } else { "" },
        );
        if html != self.hud {
            ctx.ui.set("dive", &html);
            self.hud = html;
        }
        if self.t < 0.1 {
            // little bubbles from the helmet now and then (CSS animated)
            let mut b = String::new();
            for i in 0..6 {
                let r = 4 + (i % 3) * 3;
                b += &format!(
                    "<i style='left:{}%;width:{}px;height:{}px;animation-delay:{:.2}s'></i>",
                    if i % 2 == 1 { 86 } else { 12 },
                    r * 2,
                    r * 2,
                    -(i as f32) * 0.17 / 0.35
                );
            }
            ctx.ui.set("dive_fx", &format!("<div class='bh-dive-bubbles'>{b}</div>"));
        }
    }

    pub fn describe(&self, ctx: &Ctx, s: &GameSave) -> Value {
        let lit = self.lights.iter().filter(|(e, _)| ctx.world.get::<&PointLight>(*e).map(|l| l.intensity > 0.0).unwrap_or(false)).count();
        let main = self.cam.map(|c| ctx.world.get::<&engine::scene::MainCamera>(c).is_ok());
        let r = |v: f32| (v * 100.0).round() / 100.0;
        json!({
            "on": self.on, "t": r(self.t), "fade": r(self.fade), "leaving": self.leaving,
            "x": r(self.p.x), "y": r(self.p.y), "z": r(self.p.z), "yaw": r(self.yaw), "pitch": r(self.pitch),
            "speed": r(self.vel.length()), "az": r(self.az), "fast": self.fast,
            "depth": r(self.depth()), "metres": r(((CAVE_TOP + 4.0 - self.p.y) * STUD).max(0.0)),
            "touch": self.touch, "pinch": self.pinch_on, "squeeze": r(self.squeeze),
            "pearls": (s.dive_found & ((1 << PEARLS) - 1)).count_ones(), "pearls_now": self.pearls_now,
            "core_taken": self.core_t >= 0.0, "dives": self.dives,
            "fov": r(self.fov), "cam_main": main, "lights_on": lit,
            "fog": r(ctx.renderer.settings.fog_density), "sun_scale": r(ctx.renderer.settings.sun_scale), "ambient": r(ctx.renderer.settings.ambient_scale),
            "suit_shown": self.suit_shown, "built": self.built, "cave_tris": self.cave_tris, "solids": self.solids.len(),
            "near_core": r((self.p - CORE_AT).length()),
            "zone": Self::zone(self.p),
            "warps": self.warps,
            "sharks": self.sharks.len(),
        })
    }

    /// Debug: put the diver somewhere (cave studs) and look somewhere.
    pub fn cheat(&mut self, a: &Value) {
        if let Some(p) = a["at"].as_array() {
            let v: Vec<f32> = p.iter().filter_map(|x| x.as_f64().map(|x| x as f32)).collect();
            if v.len() == 3 {
                self.p = Vec3::new(v[0], v[1], v[2]);
                self.vel = Vec3::ZERO;
            }
        }
        if let Some(y) = a["yaw"].as_f64() {
            self.yaw = y as f32;
        }
        if let Some(p) = a["pitch"].as_f64() {
            self.pitch = p as f32;
        }
        if a["surface"].as_bool() == Some(true) {
            self.click_surface = true;
        }
        if let Some(t) = a["skip_fade"].as_bool() {
            if t {
                self.fade = 0.0;
                self.t = self.t.max(1.1);
            }
        }
    }
}

fn fi_rot(i: usize, t: f32) -> f32 {
    i as f32 * 0.7 + t * 0.3
}

/// The suit still stands in the lab (Kaigen `g_suit_here`).
pub fn suit_here(s: &GameSave) -> bool {
    !s.has_suit && !s.has_deck
}

/// You wear it: the pond takes you down (Kaigen `g_can_dive`).
pub fn can_dive(s: &GameSave) -> bool {
    s.has_suit || s.has_deck
}

pub const STYLE: &str = "<style>
#ui [data-slot=dive]{position:fixed;inset:0;pointer-events:none;z-index:5}
#ui [data-slot=dive]:empty,#ui [data-slot=dive_fx]:empty{display:none}
#ui [data-slot=dive_fx]{position:fixed;inset:0;pointer-events:none;z-index:6;overflow:hidden}
.bh-dive-tint,.bh-dive-edge{position:fixed;inset:0;pointer-events:none}
.bh-dive-pill{position:fixed;left:50%;transform:translateX(-50%);top:calc(env(safe-area-inset-top) + 14px);display:flex;gap:16px;align-items:center;background:rgba(4,20,36,.6);color:#e8f6ff;border-radius:18px;padding:7px 14px;font-size:15px;font-weight:700;white-space:nowrap}
.bh-dive-pill.narrow{left:calc(env(safe-area-inset-left) + 12px);transform:none;gap:10px;font-size:14px}
.bh-dive-pill .m{color:#9fe0ff}.bh-dive-pill .core{color:#ffd23d;font-size:13px}.bh-dive-pill .ok{color:#8dff9a;font-size:13px}
.bh-dive-hint{position:fixed;left:50%;transform:translateX(-50%);bottom:calc(env(safe-area-inset-bottom) + 26px);max-width:calc(100vw - 32px);box-sizing:border-box;background:rgba(4,20,36,.6);color:#e8f6ff;border-radius:14px;padding:8px 16px;font-size:14px;font-weight:600;text-align:center}
#ui .bh-dive-up{position:fixed;right:calc(env(safe-area-inset-right) + 14px);top:calc(env(safe-area-inset-top) + 14px);margin:0;pointer-events:auto;background:rgba(30,111,224,.9);color:#fff;border-radius:18px;padding:8px 14px;font-size:14px;font-weight:700}
#ui .bh-dive-up:hover{filter:brightness(1.15)}
.bh-dive-fade{position:fixed;inset:0;background:#020a14;pointer-events:none;z-index:90}
.bh-dive-bubbles i{position:absolute;bottom:5%;border:1px solid rgba(207,234,255,.6);background:rgba(255,255,255,.08);border-radius:50%;animation:bh-bub 2.86s linear infinite}
@keyframes bh-bub{0%{transform:translateY(0);opacity:1}60%{transform:translateY(-90vh);opacity:0}100%{transform:translateY(-90vh);opacity:0}}
</style>";
