//! Your Voxelmon and the real-time fights in the tall grass (Kaigen `st_vox.c`, the three.js `voxelmon.ts`): the card
//! you raised to its final stage walks behind you as a 3D voxel beast (`assets/beasts/<id>.glb`, the Kaigen models with
//! one material per colour: `tools/glb-split-colors.mjs`); in the grass wild ones jump out, yours fights with its two
//! skills (keys 1 and 2 or the icon tiles; they recharge) while the wild one lunges on its own. The camera leaves you
//! and watches the middle between the two beasts. A new Voxelmon rises in 3D out of two glowing rings during the
//! upgrade moment (`showcase`). Positions are in studs (x east, z south, y up), as in the Kaigen port.
//!
//! The fight stays in the village (no screen of its own, as Kaigen): `PlayScreen` calls `update` every frame (it says
//! whether a fight holds you), `camera` after the talk zoom, `draw` with the HUD; `walk` / `face_yaw` steer you back
//! from the arena in `fixed_update`.

use crate::cards::{self, VOXEL_LEVEL, WILD_EXP};
use crate::life::{self, Life};
use crate::sound::{Sfx, Track};
use crate::town::rooms::{self, RoomId};
use crate::town::{self, GRASSES, STUD};
use crate::Data;
use engine::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::f32::consts::PI;

const CUBES: usize = 10;
/// The voxel cubes (volleys), the ring (a wave), the mark on the ground and the bolt (a strike).
const PROJ: usize = CUBES + 3;
/// A damage number floats this long.
const NUM_T: f32 = 0.9;
/// The upgrade moment (deck.rs): the card bursts, the 3D beast appears.
pub const UP_BURST: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum SkillKind {
    Dash,
    Volley,
    Wave,
    Strike,
}

#[derive(Clone, Copy, Debug)]
pub struct Skill {
    pub name: &'static str,
    pub kind: SkillKind,
    pub power: f32,
    pub cooldown: f32,
    pub color: u32,
}

const fn sk(name: &'static str, kind: SkillKind, power: f32, cooldown: f32, color: u32) -> Skill {
    Skill { name, kind, power, cooldown, color }
}

/// Your Voxelmon's two skills (Kaigen `vox_skill`): the three legendary ones have their own, every other a Tackle and
/// a Brick Blast.
pub fn skill(id: usize, k: usize) -> Skill {
    use SkillKind::*;
    match (id, k) {
        (cards::TITANOAK, 0) => sk("Root Slam", Dash, 14.0, 2.2, 0x8a5a2b),
        (cards::TITANOAK, _) => sk("Leaf Storm", Volley, 30.0, 7.0, 0x58ab41),
        (cards::STORMAW, 0) => sk("Thunder Bite", Dash, 13.0, 2.0, 0xffd23d),
        (cards::STORMAW, _) => sk("Storm Call", Strike, 32.0, 7.5, 0xffe14d),
        (cards::ABYSSQUID, 0) => sk("Ink Shot", Volley, 12.0, 2.2, 0x6a4c93),
        (cards::ABYSSQUID, _) => sk("Tidal Whirl", Wave, 30.0, 7.0, 0x3fa9f5),
        (_, 0) => sk("Tackle", Dash, 10.0, 2.0, 0xffffff),
        _ => sk("Brick Blast", Volley, 22.0, 6.0, 0xf2cd37),
    }
}

/// Each skill's icon (Kaigen tools/make_screen_art.py `sk_*`: Lucide-style lines on a 24 grid).
fn skill_icon(s: &Skill) -> &'static str {
    match s.name {
        "Root Slam" => "<path d='M12 3v9'/><path d='M12 12c-3 1.5-5.5 4-6.5 8'/><path d='M12 12c3 1.5 5.5 4 6.5 8'/><path d='M12 12v9'/><path d='M9 6l3-3 3 3'/>",
        "Leaf Storm" => "<path d='M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z'/><path d='M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12'/>",
        "Storm Call" => "<path d='M6 16.3A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 .5 9'/><path d='M13 12l-3 5h4l-3 5'/>",
        "Thunder Bite" => "<path d='M13 2L4 14h7l-1 8 9-12h-7z'/>",
        "Tidal Whirl" => "<path d='M2 6c.6.5 1.2 1 2.5 1C7 7 7 5 9.5 5c2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1'/><path d='M2 12c.6.5 1.2 1 2.5 1C7 13 7 11 9.5 11c2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1'/><path d='M2 18c.6.5 1.2 1 2.5 1C7 19 7 17 9.5 17c2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1'/>",
        "Ink Shot" => "<path d='M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z'/>",
        "Tackle" => "<path d='M5 17l5-5-5-5'/><path d='M12 17l5-5-5-5'/>",
        _ if s.kind == SkillKind::Dash => "<path d='M5 17l5-5-5-5'/><path d='M12 17l5-5-5-5'/>",
        _ => "<rect x='5' y='12' width='14' height='9' rx='2'/><path d='M8.5 12V9.5h3V12M12.5 12V9.5h3V12'/><path d='M3 5l2 2M21 5l-2 2M12 2v3'/>",
    }
}
const ICON_BACK: &str = "<path d='M19 12H5'/><path d='M12 19l-7-7 7-7'/>";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub enum Act {
    #[default]
    Idle,
    Lunge,
    Dash,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Fighter {
    pub id: Option<usize>,
    pub p: Vec3,
    pub home: Vec3,
    pub yaw: f32,
    /// studs high
    pub size: f32,
    pub hp: f32,
    pub max: f32,
    pub shiny: bool,
    pub tired: bool,
    pub act: Act,
    act_t: f32,
    dmg: f32,
    act_hit: bool,
    next_atk: f32,
    knock: Vec3,
    /// a hit squashes it for a moment
    squash: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Proj {
    live: bool,
    /// 0 cube, 1 ring, 2 mark, 3 bolt
    kind: u8,
    t0: f32,
    dur: f32,
    dmg: f32,
    arc: f32,
    from: Vec3,
    mine: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize)]
pub enum Phase {
    #[default]
    None,
    Intro,
    Fight,
    End,
}

/// 0 won, 1 lost, 2 fled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum FightResult {
    Won,
    Lost,
    Fled,
}

struct Num {
    e: Entity,
    p: Vec3,
    t: f32,
}

#[derive(Default)]
pub struct Vox {
    seed: u32,
    /// the beasts' models (spawned on first use) and whether each is shown
    models: HashMap<usize, (Entity, bool)>,
    pub me: Fighter,
    pub wild: Fighter,
    /// the companion shown (card), where it was last (a door: it comes along at once)
    me_id: Option<usize>,
    me_area: Option<Option<RoomId>>,
    pub phase: Phase,
    phase_t: f32,
    now: f32,
    pub cool: [f32; 2],
    proj: [Proj; PROJ],
    proj_e: Vec<Entity>,
    ring_mesh: Option<MeshId>,
    glow: HashMap<u32, MaterialId>,
    nums: Vec<Num>,
    meter: f32,
    rustle_cd: f32,
    /// the camera is at your eyes (a close-up): your Voxelmon steps out of the view
    pub hide_me: bool,
    last_p: Option<Vec3>,
    /// where you walk (studs, a direction)
    face: Vec2,
    pub result: Option<FightResult>,
    /// the arena's centre
    pub center: Vec3,
    /// you step back to see the fight: a world direction, 0..1
    pub walk: Vec2,
    reward_at: f32,
    reward_card: Option<usize>,
    pending_toast: Option<String>,
    click: [bool; 2],
    click_run: bool,
    // ---- the upgrade to a Voxelmon: the beast rising in front of the camera ----
    show_id: Option<usize>,
    show_ring: Vec<Entity>,
    show_size: f32,
    // ---- the fight camera ----
    pub cam_on: bool,
    pub cam_t: f32,
    fight_cam: Option<Entity>,
    cam_pos: Vec3,
    cam_look: Vec3,
    cam_side: f32,
    // ---- tests ----
    /// a wild one now (debug `vox {force}`), which (None: the pool) and shiny
    force: Option<(Option<usize>, bool)>,
    /// the chance per 3 studs walked in the grass (Kaigen 0.22)
    pub rate: f32,
    /// the wild one never lunges (tests)
    pub wild_idle: bool,
    pub fights: u32,
    pub skills_used: u32,
    pub hits: u32,
    pub rustles: u32,
}

/// Any patch of tall grass (the village's and the three up Route 1).
pub fn in_grass(x: f32, z: f32) -> Option<usize> {
    GRASSES.iter().position(|g| x > g.x0 && x < g.x1 && z > g.z0 && z < g.z1)
}

fn face_to(from: Vec3, to: Vec3) -> f32 {
    (to.x - from.x).atan2(to.z - from.z)
}

fn turn(a: f32, b: f32, k: f32) -> f32 {
    a + life::wrap(b - a) * k.min(1.0)
}

fn move_to(f: &mut Fighter, to: Vec3, dt: f32, speed: f32) {
    let (dx, dz) = (to.x - f.p.x, to.z - f.p.z);
    let d = (dx * dx + dz * dz).sqrt();
    if d < 0.05 {
        return;
    }
    let s = d.min(speed * dt);
    f.p.x += dx / d * s;
    f.p.z += dz / d * s;
}

fn rgb(hex: u32) -> Vec3 {
    engine::renderer::srgb_hex(hex).truncate()
}

fn hp_color(f: f32) -> &'static str {
    if f > 0.5 {
        "#3cbe50"
    } else if f > 0.2 {
        "#f0be28"
    } else {
        "#dc3c28"
    }
}

pub const STYLE: &str = "<style>
#ui [data-slot=vox_wild]{position:fixed;left:0;right:0;top:112px;display:flex;justify-content:center;pointer-events:none}
#ui [data-slot=vox_mine]{position:fixed;left:0;right:0;bottom:calc(18px + env(safe-area-inset-bottom));display:flex;justify-content:center;pointer-events:none}
#ui [data-slot=vox_pill]{position:fixed;right:14px;top:58px;pointer-events:none}
#ui [data-slot=vox_pill].narrow{top:112px}
.vx-box{background:rgba(16,20,26,.84);color:#fff;border-radius:10px;padding:8px 14px 10px;display:flex;flex-direction:column;gap:5px;font-size:16px;font-weight:600}
.vx-box.shiny .vx-name{color:#ffd23d}
.vx-bar{height:10px;border-radius:5px;background:#282c34;overflow:hidden}.vx-bar i{display:block;height:100%;border-radius:5px}
.vx-mine{border-radius:12px;padding:12px;align-items:center;gap:8px;pointer-events:auto}
.vx-skills{display:flex;gap:14px;align-items:flex-start}
.vx-sk{width:76px;display:flex;flex-direction:column;align-items:center;gap:4px}
#ui .vx-tile{position:relative;margin:0;padding:0;width:62px;height:62px;border-radius:16px;background:#2c3038;overflow:hidden;border:2px solid #5a606a;box-shadow:0 2px 8px rgba(0,0,0,.35);cursor:default}
#ui .vx-tile.ready{border-color:rgba(255,255,255,.86);box-shadow:0 2px 8px var(--g);cursor:pointer}
#ui .vx-tile.ready:hover{filter:brightness(1.15)}
.vx-fill{position:absolute;left:0;right:0;bottom:0;background:linear-gradient(180deg,var(--c2),var(--c1));border-top:2px solid rgba(255,255,255,.66)}
.vx-tile.ready .vx-fill{border-top:0;background:linear-gradient(180deg,var(--c3),var(--c1))}
.vx-tile svg{position:absolute;left:13.5px;top:13.5px;width:35px;height:35px;fill:none;stroke:#fff;stroke-width:2.1;stroke-linecap:round;stroke-linejoin:round;filter:drop-shadow(0 1.5px 2px rgba(0,0,0,.55))}
.vx-tile:not(.ready) svg{opacity:.6}
.vx-key{position:absolute;left:5px;top:3px;font-size:11px;font-weight:800;color:rgba(255,255,255,.75);text-shadow:0 0 3px #000}
.vx-sec{position:absolute;left:0;right:0;bottom:3px;text-align:center;font-size:11px;font-weight:800;color:#fff;text-shadow:0 0 3px #000}
.vx-lbl{font-size:11px;font-weight:700;color:#e6eaf0;text-align:center;text-shadow:0 1px 2px rgba(0,0,0,.7);line-height:1.2}
.vx-pill{background:rgba(250,250,246,.88);color:#1b2a34;border-radius:10px;padding:6px 12px 8px;display:flex;flex-direction:column;gap:4px;font-size:14px;font-weight:600}
.vx-pill.night{background:rgba(24,32,42,.88);color:#e8eef4}
#ui .world-label.vx-num{background:none;border:0;padding:0;font-size:24px;font-weight:900;color:#ff5a46;text-shadow:0 2px 0 #000,0 0 4px #000;pointer-events:none}
#ui .world-label.vx-num.mine{font-size:30px;color:#ffe650}
@media (max-width:640px){.vx-sk{width:68px}.vx-skills{gap:8px}}
</style>";

impl Vox {
    fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 8) as f32 / 16777216.0
    }

    pub fn reseed(&mut self, seed: u32) {
        self.seed = seed | 1;
    }

    /// The pieces (once, in the play screen's scene): the glow cubes, rings and the bolt, the showcase's rings.
    pub fn init(&mut self, ctx: &mut Ctx) {
        if self.seed == 0 {
            self.seed = ctx.rng.next_u64() as u32 | 1;
        }
        self.rate = 0.22;
        let mut b = engine::geom::MeshBuilder::new();
        let path: Vec<Vec3> = (0..=24).map(|i| {
            let a = i as f32 / 24.0 * std::f32::consts::TAU;
            Vec3::new(a.cos() * STUD, 0.0, a.sin() * STUD)
        }).collect();
        b.part("ring").sweep(&path, &engine::geom::circle(0.18 * STUD, 6), true);
        let ring = ctx.renderer.add_mesh("vox_ring", &b.finish().into_iter().next().unwrap().1);
        self.ring_mesh = Some(ring);
        let cube = ctx.renderer.builtin.cube;
        let white = self.glow_mat(ctx, 0xffffff);
        for i in 0..PROJ {
            let mesh = if i < CUBES || i == CUBES + 2 { cube } else { ring };
            let e = ctx.spawn_mesh("vox_proj", mesh, white, Transform::from_xyz(0.0, -60.0, 0.0));
            if let Ok(mut m) = ctx.world.get::<&mut MeshRenderer>(e) {
                m.layer = rooms::HIDDEN_LAYER;
                m.cast_shadows = false;
            }
            self.proj_e.push(e);
        }
        let purple = ctx.renderer.add_material(Material::color(0xad73ff).glow(rgb(0xad73ff) * 3.0));
        for _ in 0..2 {
            let e = ctx.spawn_mesh("vox_show_ring", ring, purple, Transform::from_xyz(0.0, -60.0, 0.0));
            if let Ok(mut m) = ctx.world.get::<&mut MeshRenderer>(e) {
                m.layer = rooms::HIDDEN_LAYER;
                m.cast_shadows = false;
            }
            self.show_ring.push(e);
        }
    }

    fn glow_mat(&mut self, ctx: &mut Ctx, hex: u32) -> MaterialId {
        *self.glow.entry(hex).or_insert_with(|| ctx.renderer.add_material(Material::color(hex).glow(rgb(hex) * 2.0)))
    }

    /// A beast's model (loaded on first use, hidden).
    fn model(&mut self, ctx: &mut Ctx, id: usize) -> Entity {
        if let Some((e, _)) = self.models.get(&id) {
            return *e;
        }
        let e = ctx.load_gltf(&format!("assets/beasts/{}.glb", cards::card(id).id), Transform::from_xyz(0.0, -60.0, 0.0));
        ctx.set_layer(e, rooms::HIDDEN_LAYER);
        self.models.insert(id, (e, false));
        e
    }

    fn show_model(&mut self, ctx: &mut Ctx, id: usize, t: Option<Transform>) {
        let e = self.model(ctx, id);
        let on = t.is_some();
        if let Some(t) = t {
            if let Ok(mut tr) = ctx.world.get::<&mut Transform>(e) {
                *tr = t;
            }
        }
        let shown = self.models.get_mut(&id).unwrap();
        if shown.1 != on {
            shown.1 = on;
            ctx.set_layer(e, if on { 0 } else { rooms::HIDDEN_LAYER });
        }
    }

    fn hide(&mut self, ctx: &mut Ctx, id: Option<usize>) {
        if let Some(id) = id {
            if self.models.get(&id).is_some_and(|m| m.1) {
                self.show_model(ctx, id, None);
            }
        }
    }

    fn set_piece(ctx: &mut Ctx, e: Entity, t: Option<Transform>) {
        if let Ok(mut m) = ctx.world.get::<&mut MeshRenderer>(e) {
            m.layer = if t.is_some() { 0 } else { rooms::HIDDEN_LAYER };
        }
        if let (Some(t), Ok(mut tr)) = (t, ctx.world.get::<&mut Transform>(e)) {
            *tr = t;
        }
    }

    /// The fighter's model: knocked over when tired, squashed when hit (Kaigen `vox_pose`).
    fn pose(&mut self, ctx: &mut Ctx, f: &Fighter, grow: f32) {
        let Some(id) = f.id else { return };
        let s = f.size * STUD * grow;
        let sq = 1.0 - f.squash * 0.25;
        let q = Quat::from_rotation_y(f.yaw) * Quat::from_rotation_z(if f.tired { PI / 2.4 } else { 0.0 });
        let t = Transform::from_translation(town::m(f.p.x + f.knock.x, f.p.y, f.p.z + f.knock.z)).with_rotation(q).with_scale(Vec3::new(s * (2.0 - sq), s * sq, s * (2.0 - sq)));
        self.show_model(ctx, id, Some(t));
    }

    fn fighter(&self, mine: bool) -> &Fighter {
        if mine {
            &self.me
        } else {
            &self.wild
        }
    }

    // ------------------------------------------------------------------ the fight

    fn damage(&mut self, ctx: &mut Ctx, data: &mut Data, to_wild: bool, dmg: f32) {
        if self.phase != Phase::Fight {
            return;
        }
        let n = dmg.round().max(1.0) as i32;
        let src = self.fighter(!to_wild).p;
        let f = if to_wild { &mut self.wild } else { &mut self.me };
        f.hp -= n as f32;
        f.squash = 1.0;
        let away = Vec3::new(f.p.x - src.x, 0.0, f.p.z - src.z);
        f.knock = away * (0.6 / away.length().max(0.01));
        let at = Vec3::new(f.p.x, f.p.y + f.size + 0.6, f.p.z);
        let dead = f.hp <= 0.0;
        if dead {
            f.hp = 0.0;
        }
        self.hits += 1;
        let e = ctx.world.spawn((
            Transform::from_translation(town::m(at.x, at.y, at.z)),
            WorldLabel { html: format!("-{n}"), offset: Vec3::ZERO, class: if to_wild { "vx-num mine".into() } else { "vx-num".into() } },
            Name("vox_num".into()),
        ));
        self.nums.push(Num { e, p: at, t: 0.0 });
        data.sound.sfx(ctx, Sfx::Kick, if to_wild { 1.2 } else { 0.85 }, 0.7);
        if dead {
            self.end(ctx, data, if to_wild { FightResult::Won } else { FightResult::Lost });
        }
    }

    fn fire(&mut self, ctx: &mut Ctx, kind: u8, slot: usize, delay: f32, dur: f32, dmg: f32, mine: bool, color: u32) {
        let f = *self.fighter(mine);
        let (r1, r2, r3, r4) = (self.rand(), self.rand(), self.rand(), self.rand());
        self.proj[slot] = Proj {
            live: true,
            kind,
            t0: self.now + delay,
            dur,
            dmg,
            mine,
            from: Vec3::new(f.p.x + (r1 - 0.5) * 1.2, f.p.y + f.size * 0.6 + r2, f.p.z + (r3 - 0.5) * 1.2),
            arc: 1.5 + r4,
        };
        let mat = self.glow_mat(ctx, color);
        if let Ok(mut m) = ctx.world.get::<&mut MeshRenderer>(self.proj_e[slot]) {
            m.material = mat;
        }
    }

    /// Key 1 / 2 or a tile (Kaigen `vox_use_skill`): a dash at the foe, a volley of voxel cubes, a wave, a strike.
    pub fn use_skill(&mut self, ctx: &mut Ctx, data: &mut Data, k: usize) -> bool {
        let Some(id) = self.me.id else { return false };
        if self.phase != Phase::Fight || self.cool[k] > 0.0 || self.me.act != Act::Idle {
            return false;
        }
        let s = skill(id, k);
        self.cool[k] = s.cooldown;
        self.skills_used += 1;
        data.sound.sfx(ctx, Sfx::Pop, 0.8 + 0.01 * cards::card(id).cost as f32, 0.7);
        match s.kind {
            SkillKind::Dash => {
                self.me.act = Act::Dash;
                self.me.act_t = 0.0;
                self.me.act_hit = false;
                self.me.dmg = s.power;
            }
            SkillKind::Volley => {
                for i in 0..5 {
                    if let Some(slot) = (0..CUBES).find(|&j| !self.proj[j].live) {
                        self.fire(ctx, 0, slot, i as f32 * 0.09, 0.45, s.power / 5.0, true, s.color);
                    }
                }
            }
            SkillKind::Wave => self.fire(ctx, 1, CUBES, 0.0, 0.55, s.power, true, s.color),
            SkillKind::Strike => {
                self.fire(ctx, 2, CUBES + 1, 0.0, 0.5, 0.0, true, s.color);
                self.fire(ctx, 3, CUBES + 2, 0.5, 0.25, s.power, true, s.color);
            }
        }
        true
    }

    /// A lunge (the wild one) or a dash (your Tackle): wind up, leap at the foe, hit, hop back.
    fn run_act(&mut self, ctx: &mut Ctx, data: &mut Data, mine: bool, dt: f32) {
        let foe = *self.fighter(!mine);
        let r = self.rand();
        let now = self.now;
        let mut f = *self.fighter(mine);
        if f.act == Act::Idle {
            return;
        }
        f.act_t += dt;
        let (wind, leap, back) = (if f.act == Act::Lunge { 0.55 } else { 0.18 }, 0.16, 0.34);
        let to = Vec3::new(foe.p.x - f.p.x, 0.0, foe.p.z - f.p.z);
        let dir = to / to.length().max(0.01);
        let reach = (foe.size + f.size) * 0.3;
        f.yaw = turn(f.yaw, face_to(f.p, foe.p), dt * 14.0);
        let t = f.act_t;
        let hit = Vec3::new(foe.p.x - dir.x * reach, f.home.y, foe.p.z - dir.z * reach);
        let mut strike = None;
        if t < wind {
            let k = t / wind;
            f.p = Vec3::new(f.home.x - dir.x * k * 0.8, f.home.y, f.home.z - dir.z * k * 0.8);
        } else if t < wind + leap {
            let k = (t - wind) / leap;
            f.p = f.home.lerp(hit, k);
            f.p.y = f.home.y + (k * PI).sin() * 1.2;
        } else {
            if !f.act_hit {
                f.act_hit = true;
                let id = f.id.unwrap_or(0);
                strike = Some(if f.act == Act::Lunge {
                    ((6.0 + cards::card(id).cost as f32 * 1.6) * if f.shiny { 1.25 } else { 1.0 } * (0.85 + r * 0.3)).round()
                } else {
                    f.dmg
                });
            }
            let k = ((t - wind - leap) / back).min(1.0);
            f.p = hit.lerp(f.home, k);
            f.p.y = f.home.y + (k * PI).sin() * 0.6;
            if k >= 1.0 {
                f.act = Act::Idle;
                if !mine {
                    f.next_atk = now + 2.2 + r * 1.4;
                }
            }
        }
        // (the fighter's hp may change in `damage`: write its movement back first)
        let slot = if mine { &mut self.me } else { &mut self.wild };
        let hp = slot.hp;
        *slot = Fighter { hp, ..f };
        if let Some(d) = strike {
            self.damage(ctx, data, mine, d); // (mine: it hits the wild one)
        }
    }

    /// A wild one jumps out (Kaigen `vox_start`): from the pool (cost 5 and below, cheaper ones more often), 1 in 12
    /// shiny; the fight is 7 studs in front of you (where the camera looks), kept inside the patch.
    fn start(&mut self, ctx: &mut Ctx, data: &mut Data, life: &mut Life, p: Vec3, pick: Option<usize>, force_shiny: bool) {
        let total: f32 = (0..cards::CARD_POOL).filter(|&c| cards::card(c).cost <= 5).map(|c| (6 - cards::card(c).cost) as f32).sum();
        let mut r = self.rand() * total;
        let mut id = 0;
        for c in 0..cards::CARD_POOL {
            if cards::card(c).cost > 5 {
                continue;
            }
            r -= (6 - cards::card(c).cost) as f32;
            if r <= 0.0 {
                id = c;
                break;
            }
        }
        let id = pick.unwrap_or(id);
        let shiny = self.rand() < 1.0 / 12.0 || force_shiny;
        let cost = cards::card(id).cost as f32;
        // the camera looks north (the fixed view of the original)
        let (fwd, side) = (Vec2::new(0.0, -1.0), Vec2::new(1.0, 0.0));
        let mut c = Vec3::new(p.x + fwd.x * 7.0, p.y, p.z + fwd.y * 7.0);
        let g = GRASSES[in_grass(p.x, p.z).unwrap_or(0)];
        c.x = c.x.clamp(g.x0 + 3.6, g.x1 - 3.6);
        c.z = c.z.clamp(g.z0 + 3.6, g.z1 - 2.0);
        self.hide(ctx, self.wild.id.filter(|w| Some(*w) != self.me.id));
        let max = ((40.0 + cost * 13.0) * if shiny { 1.4 } else { 1.0 }).round();
        let home = Vec3::new(c.x + side.x * 3.4, c.y, c.z + side.y * 3.4);
        self.wild = Fighter { id: Some(id), size: 2.4 + cost * 0.25, shiny, max, hp: max, home, p: home, ..Default::default() };
        self.me.home = Vec3::new(c.x - side.x * 3.4, c.y, c.z - side.y * 3.4);
        self.me.act = Act::Idle;
        self.center = c;
        self.wild.yaw = face_to(self.wild.p, self.me.home);
        self.phase = Phase::Intro;
        self.phase_t = 0.0;
        self.cool = [0.0; 2];
        self.result = None;
        self.fights += 1;
        self.model(ctx, id);
        let name = cards::card(id).name;
        if shiny {
            data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8);
            life.toast(&format!("A SHINY wild {name} appeared!"));
        } else {
            data.sound.sfx(ctx, Sfx::Alert, 1.0, 0.7);
            life.toast(&format!("A wild {name} jumped out of the grass!"));
        }
        ctx.input.touch.set_stick_enabled(false); // the thumb's taps go to the skill tiles
        data.sound.music(ctx, Track::Wild); // its own tune, not the card battle's
    }

    fn end(&mut self, ctx: &mut Ctx, data: &mut Data, result: FightResult) {
        self.phase = Phase::End;
        self.phase_t = 0.0;
        self.result = Some(result);
        self.me.act = Act::Idle;
        self.wild.act = Act::Idle;
        for i in 0..PROJ {
            if self.proj[i].live {
                self.proj[i].live = false;
                Self::set_piece(ctx, self.proj_e[i], None);
            }
        }
        match result {
            FightResult::Lost => self.me.tired = true,
            FightResult::Won => self.reward(ctx, data),
            FightResult::Fled => {}
        }
    }

    /// The win (Kaigen `vox_reward`): EXP, studs, the wild card (a shiny one: a rare missing card) as a foil pack.
    fn reward(&mut self, ctx: &mut Ctx, data: &mut Data) {
        let shiny = self.wild.shiny;
        let studs = if shiny { 10 } else { 3 };
        let mut seed = self.seed;
        let s = &mut data.save.data;
        s.vox_wins += 1;
        s.exp += WILD_EXP;
        s.studs += studs;
        let mut id = None;
        if shiny {
            id = cards::reward_card(s, &mut seed, true);
        } else if let Some(w) = self.wild.id.filter(|&w| !cards::owns(s, w)) {
            cards::give_card(s, w);
            id = Some(w);
        }
        self.seed = seed | 1;
        data.save.mark();
        data.sound.sfx(ctx, Sfx::Fanfare, if shiny { 1.2 } else { 1.0 }, 0.7);
        self.pending_toast = Some(format!("You won! +{WILD_EXP} EXP \u{b7} +{studs} studs"));
        if id.is_some() {
            self.reward_card = id;
            self.reward_at = 1.4;
        }
    }

    // ------------------------------------------------------------------ every frame

    /// Every frame (Kaigen `vox_update` + `vox_after`). `hero`: your feet in studs. Returns true while a fight holds you.
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, ctx: &mut Ctx, data: &mut Data, life: &mut Life, hero: Vec3, area: Option<RoomId>, busy: bool, actions: &[String], dt: f32) -> bool {
        self.now += dt;
        let p = hero;
        let moved = match self.last_p {
            Some(l) => {
                let step = Vec2::new(p.x - l.x, p.z - l.z);
                let m = step.length();
                if m > 0.01 && m < 3.0 {
                    self.face = self.face.lerp(step / m, (dt * 8.0).min(1.0));
                }
                m
            }
            None => 0.0,
        };
        self.last_p = Some(p);
        self.rustle_cd -= dt;
        if let Some(t) = self.pending_toast.take() {
            life.toast(&t);
        }
        // the damage numbers float up and go
        let mut i = 0;
        while i < self.nums.len() {
            let n = &mut self.nums[i];
            n.t += dt;
            if n.t > NUM_T {
                ctx.despawn(n.e);
                self.nums.swap_remove(i);
                continue;
            }
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.e) {
                t.translation = town::m(n.p.x, n.p.y + n.t * 2.0, n.p.z);
            }
            i += 1;
        }
        // the reward pack after a win
        if self.reward_at > 0.0 {
            self.reward_at -= dt;
            if self.reward_at <= 0.0 {
                if let Some(c) = self.reward_card.take() {
                    data.cards.show_rewards(&format!("{}'s card joined your collection!", cards::card(c).name), &[c], None);
                }
            }
        }
        // the companion: the card you picked in the deck menu
        let s = &data.save.data;
        let want = if s.voxelmon > 0 && cards::level(s, s.voxelmon as usize - 1) >= VOXEL_LEVEL { Some(s.voxelmon as usize - 1) } else { None };
        if want != self.me_id && self.phase == Phase::None {
            let old = self.me.id;
            if old != self.show_id {
                self.hide(ctx, old);
            }
            self.me_id = want;
            self.me = Fighter { id: want, ..Default::default() };
            if let Some(w) = want {
                self.me.size = 4.2;
                self.me.max = 90.0 + cards::card(w).power as f32 * 3.0;
                self.me.hp = self.me.max;
                self.me.p = Vec3::new(p.x + 2.0, p.y, p.z + 3.0);
                self.me_area = Some(area);
                self.model(ctx, w);
            }
        }
        if self.me.id.is_none() {
            return false;
        }
        // hits fade
        for f in [&mut self.me, &mut self.wild] {
            f.squash = (f.squash - dt * 7.0).max(0.0);
            f.knock *= (1.0 - dt * 8.0).max(0.0);
        }
        let clicks = (actions.iter().any(|a| a == "vox_skill:0"), actions.iter().any(|a| a == "vox_skill:1"), actions.iter().any(|a| a == "vox_run"));
        self.click[0] |= clicks.0;
        self.click[1] |= clicks.1;
        self.click_run |= clicks.2;

        if self.phase == Phase::None {
            self.click = [false; 2];
            self.click_run = false;
            let me = &mut self.me;
            // rest: HP comes back, standing up again above half
            me.hp = (me.hp + me.max / 90.0 * dt).min(me.max);
            if me.tired && me.hp > me.max * 0.5 {
                me.tired = false;
            }
            // follow: 3 studs behind you, 1.6 to the side; through a door it comes along at once
            let f = if self.face.length() > 0.1 { self.face.normalize() } else { Vec2::new(0.0, -1.0) };
            let target = Vec3::new(p.x - f.x * 3.0 + f.y * 1.6, p.y, p.z - f.y * 3.0 - f.x * 1.6);
            let dist = (target - me.p).length();
            if dist > 20.0 || self.me_area != Some(area) {
                me.p = target;
                self.me_area = Some(area);
            }
            // (the original's 11 / 6 studs/s scaled by our walk over the three.js one, as Kaigen STUD_PER_THREE)
            let speed = (dist * 3.0).min(if me.tired { 6.0 } else { 11.0 }) * 1.24;
            let before = me.p;
            move_to(me, target, dt, speed);
            let sp = (me.p - before).length() / dt.max(1e-4);
            me.p.y += (p.y - me.p.y) * (dt * 10.0).min(1.0);
            let mut shown = *me;
            if sp > 0.5 {
                me.yaw = turn(me.yaw, (me.p.x - before.x).atan2(me.p.z - before.z), dt * 10.0);
                shown.yaw = me.yaw;
                shown.p.y += (self.now * 10.0).sin().abs() * 0.45;
            } else {
                me.yaw = turn(me.yaw, face_to(me.p, p), dt * 3.0);
                shown.yaw = me.yaw;
            }
            if self.hide_me {
                self.hide(ctx, self.me.id);
            } else if self.show_id != self.me.id {
                // (rising as the new Voxelmon: the showcase poses it)
                self.pose(ctx, &shown, 1.0);
            }
            if let Some((pick, shiny)) = self.force.take() {
                if !self.me.tired && !busy {
                    self.start(ctx, data, life, p, pick, shiny);
                }
                return self.phase != Phase::None;
            }
            // the grass: every 3 studs walked in it a 22 % chance
            if !busy && area.is_none() && in_grass(p.x, p.z).is_some() && moved < 3.0 {
                self.meter += moved;
                if self.meter >= 3.0 {
                    self.meter -= 3.0;
                    if self.rand() < self.rate {
                        if !self.me.tired {
                            self.start(ctx, data, life, p, None, false);
                            return true;
                        } else if self.rustle_cd <= 0.0 {
                            self.rustle_cd = 6.0;
                            self.rustles += 1;
                            life.toast(&format!("The grass rustles\u{2026} but {} is too tired to fight. Let it rest!", cards::card(self.me.id.unwrap()).name));
                        }
                    }
                }
            }
            return false;
        }

        self.phase_t += dt;
        // you back off to 8.5 studs from the arena's centre, so both beasts stand in front of the camera
        self.walk = Vec2::ZERO;
        if self.phase != Phase::End {
            let off = Vec2::new(p.x - self.center.x, p.z - self.center.z);
            let d = off.length();
            if d < 8.0 {
                let away = if d > 0.1 { off / d } else { Vec2::new(0.0, 1.0) };
                self.walk = away * 0.5;
            }
        }
        let t = self.phase_t;
        match self.phase {
            Phase::Intro => {
                let k = (t / 0.5).min(1.0);
                let grow = if k < 1.0 { k * 1.15 } else { 1.0 }.max(0.01);
                self.wild.p.y = self.wild.home.y + ((t / 0.7).min(1.0) * PI).sin() * 1.6;
                let home = self.me.home;
                move_to(&mut self.me, home, dt, 14.0);
                self.me.yaw = turn(self.me.yaw, face_to(self.me.p, self.wild.p), dt * 10.0);
                self.wild.yaw = face_to(self.wild.p, self.me.p);
                let (w, m) = (self.wild, self.me);
                self.pose(ctx, &w, grow);
                self.pose(ctx, &m, 1.0);
                if t > 1.1 {
                    self.phase = Phase::Fight;
                    self.phase_t = 0.0;
                    self.wild.next_atk = self.now + 1.6;
                }
                return true;
            }
            Phase::End => {
                if t > 1.6 {
                    self.phase = Phase::None;
                    self.hide(ctx, self.wild.id.filter(|w| Some(*w) != self.me.id && Some(*w) != self.show_id));
                    self.wild.id = None;
                    data.sound.music(ctx, Track::Home);
                    self.me.home = self.me.p;
                    self.me.p.y = p.y;
                    ctx.input.touch.set_stick_enabled(true);
                    return false;
                }
                if self.result != Some(FightResult::Won) {
                    // it hops away and shrinks
                    let sc = (1.0 - t / 0.9).max(0.01);
                    let mut shown = self.wild;
                    shown.p.y = self.wild.home.y + (t * 9.0).sin().abs() * 0.8;
                    self.pose(ctx, &shown, sc);
                } else {
                    // won: it bursts, yours hops happily
                    if self.wild.id != self.me.id {
                        self.hide(ctx, self.wild.id);
                    }
                    let mut shown = self.me;
                    shown.p.y = self.me.home.y + (t * 10.0).sin().abs() * 0.7;
                    if self.show_id != self.me.id {
                        self.pose(ctx, &shown, 1.0);
                    }
                }
                if self.result == Some(FightResult::Lost) {
                    let m = self.me;
                    self.pose(ctx, &m, 1.0);
                }
                return true;
            }
            _ => {}
        }
        // ---- the fight ----
        for c in &mut self.cool {
            *c = (*c - dt).max(0.0);
        }
        if !busy {
            if ctx.input.key_pressed("Digit1") || self.click[0] {
                self.use_skill(ctx, data, 0);
            }
            if ctx.input.key_pressed("Digit2") || self.click[1] {
                self.use_skill(ctx, data, 1);
            }
            if self.click_run || ctx.input.key_pressed("Escape") {
                life.toast("Got away safely!");
                self.end(ctx, data, FightResult::Fled);
            }
        }
        self.click = [false; 2];
        self.click_run = false;
        if self.phase != Phase::Fight {
            return true;
        }
        self.run_act(ctx, data, true, dt);
        if self.phase == Phase::Fight {
            self.run_act(ctx, data, false, dt);
        }
        let now = self.now;
        if self.me.act == Act::Idle {
            let home = self.me.home;
            move_to(&mut self.me, home, dt, 8.0);
            self.me.yaw = turn(self.me.yaw, face_to(self.me.p, self.wild.p), dt * 10.0);
            self.me.p.y = self.me.home.y + (now * 5.0).sin().abs() * 0.15;
        }
        if self.wild.act == Act::Idle {
            let home = self.wild.home;
            move_to(&mut self.wild, home, dt, 8.0);
            self.wild.yaw = turn(self.wild.yaw, face_to(self.wild.p, self.me.p), dt * 10.0);
            self.wild.p.y = self.wild.home.y + (now * 4.0 + 1.0).sin().abs() * 0.12;
            if now > self.wild.next_atk && self.phase == Phase::Fight && !self.wild_idle {
                self.wild.act = Act::Lunge;
                self.wild.act_t = 0.0;
                self.wild.act_hit = false;
            }
        }
        // the projectiles
        for i in 0..PROJ {
            if self.phase != Phase::Fight {
                break;
            }
            let pr = self.proj[i];
            if !pr.live {
                continue;
            }
            let k = (now - pr.t0) / pr.dur;
            if k < 0.0 {
                continue;
            }
            let foe = *self.fighter(!pr.mine);
            let to = Vec3::new(foe.p.x, foe.p.y + foe.size * 0.45, foe.p.z);
            let e = self.proj_e[i];
            let t = match pr.kind {
                0 => {
                    let mut q = pr.from.lerp(to, k.min(1.0));
                    q.y += (k.min(1.0) * PI).sin() * pr.arc;
                    Transform::from_translation(town::m(q.x, q.y, q.z)).with_rotation(Quat::from_euler(EulerRot::XYZ, k * 9.0, k * 7.0, 0.0)).with_scale(Vec3::splat(0.6 * STUD))
                }
                1 => {
                    let a = Vec3::new(self.me.p.x, self.me.p.y + 0.6, self.me.p.z);
                    let q = a.lerp(to, k.min(1.0));
                    let sc = 1.0 + k * 1.6;
                    Transform::from_translation(town::m(q.x, a.y, q.z)).with_rotation(Quat::from_rotation_y(face_to(self.me.p, self.wild.p))).with_scale(Vec3::splat(sc))
                }
                2 => {
                    let sc = 2.6 - k * 1.6;
                    Transform::from_translation(town::m(foe.p.x, foe.p.y + 0.1, foe.p.z)).with_rotation(Quat::from_rotation_y(now * 3.0)).with_scale(Vec3::new(sc, 0.4, sc))
                }
                _ => Transform::from_translation(town::m(foe.p.x, foe.p.y + 12.0, foe.p.z)).with_scale(Vec3::new((1.0 + k) * 0.5 * STUD, 12.0 * STUD, (1.0 + k) * 0.5 * STUD)),
            };
            Self::set_piece(ctx, e, Some(t));
            if k >= 1.0 {
                self.proj[i].live = false;
                Self::set_piece(ctx, e, None);
                if pr.dmg > 0.0 {
                    self.damage(ctx, data, pr.mine, pr.dmg);
                }
            }
        }
        let m = self.me;
        self.pose(ctx, &m, 1.0);
        if self.phase == Phase::Fight {
            let w = self.wild;
            self.pose(ctx, &w, 1.0);
        }
        true
    }

    /// A debug / test start: a wild one now (with one walking with you), which (None: the pool) and shiny.
    pub fn force(&mut self, pick: Option<usize>, shiny: bool) {
        self.force = Some((pick, shiny));
    }

    /// Run (as the tile / Esc).
    pub fn run(&mut self) {
        self.click_run = true;
    }

    /// A good night: your Voxelmon is fit again (Kaigen: in the sleep action).
    pub fn rest(&mut self) {
        self.me.hp = self.me.max;
        self.me.tired = false;
    }

    /// Facing the fight (the controller's yaw: it turns its -z to it).
    pub fn face_yaw(&self, hero: Vec3) -> Option<f32> {
        if self.phase == Phase::None {
            return None;
        }
        let d = Vec2::new(self.center.x - hero.x, self.center.z - hero.z);
        Some((-d.x).atan2(-d.y))
    }

    // ------------------------------------------------------------------ the upgrade moment

    /// The upgrade to a Voxelmon (deck.rs): once the card has burst, the beast rises out of two glowing rings in front
    /// of you (between you and the camera), spinning and growing, and keeps turning there until the moment is closed.
    pub fn showcase(&mut self, ctx: &mut Ctx, id: Option<usize>, t: f32, cam: Option<Vec3>, hero: Vec3) {
        let (Some(id), Some(cam)) = (id.filter(|_| t >= UP_BURST), cam) else {
            if let Some(s) = self.show_id.take() {
                if Some(s) != self.me.id && Some(s) != self.wild.id {
                    self.hide(ctx, Some(s));
                }
                for e in self.show_ring.clone() {
                    Self::set_piece(ctx, e, None);
                }
            }
            return;
        };
        self.show_id = Some(id);
        let cs = cam / STUD;
        let back = Vec2::new(cs.x - hero.x, cs.z - hero.z).normalize_or_zero();
        let at = Vec3::new(hero.x + back.x * 4.0, hero.y, hero.z + back.y * 4.0);
        let k = ((t - UP_BURST) / 1.2).clamp(0.0, 1.0);
        let grow = 1.0 - (1.0 - k).powi(3);
        let face = (cs.x - at.x).atan2(cs.z - at.z);
        let yaw = face + (t - UP_BURST) * 0.9 + (1.0 - grow) * 6.0;
        let size = 6.0 * STUD * grow.max(0.01);
        self.show_size = 6.0 * grow;
        let y = at.y + (1.0 - grow) * -1.5 + (t * 2.0).sin().abs() * 0.25 * k;
        self.show_model(ctx, id, Some(Transform::from_translation(town::m(at.x, y, at.z)).with_rotation(Quat::from_rotation_y(yaw)).with_scale(Vec3::splat(size))));
        for i in 0..2 {
            let r = (if i == 0 { 3.4 } else { 4.6 + 0.3 * (t * 3.0).sin() }) * (k * 1.6).min(1.0);
            let tr = Transform::from_translation(town::m(at.x, at.y + 0.1 + i as f32 * 0.05, at.z))
                .with_rotation(Quat::from_rotation_y(t * if i == 1 { -1.2 } else { 1.6 }))
                .with_scale(Vec3::new(r.max(0.01), 1.0, r.max(0.01)));
            let e = self.show_ring[i];
            Self::set_piece(ctx, e, Some(tr));
        }
    }

    // ------------------------------------------------------------------ the fight camera

    /// During a fight the camera leaves you and watches the middle between the two beasts from the side away from you
    /// (you are seen behind the fight, watching), far enough for both; it eases there and back to the village camera
    /// afterwards (Kaigen `vox_camera`). The fight has its own camera (`ctx.set_main_camera`); the village camera keeps
    /// following you meanwhile, so the way back ends exactly where it is. Returns true while the fight's camera draws.
    pub fn camera(&mut self, ctx: &mut Ctx, village: Entity, hero_m: Vec3, dt: f32) -> bool {
        let want = self.phase != Phase::None && self.me.id.is_some() && self.wild.id.is_some();
        let ha = town::m(self.me.home.x, self.me.home.y, self.me.home.z);
        let hb = town::m(self.wild.home.x, self.wild.home.y, self.wild.home.z);
        let a = town::m(self.me.p.x, self.me.p.y, self.me.p.z);
        let b = town::m(self.wild.p.x, self.wild.p.y, self.wild.p.z);
        let (mid, hmid) = ((a + b) * 0.5, (ha + hb) * 0.5);
        let axis = Vec2::new(hb.x - ha.x, hb.z - ha.z);
        let hs = axis.length().max(1.0);
        let n = Vec2::new(-axis.y / hs, axis.x / hs); // across the line between them
        let sep = Vec2::new(b.x - a.x, b.z - a.z).length().max(hs);
        let Ok(rig) = ctx.world.get::<&Transform>(village).map(|t| *t) else { return false };
        let Ok(vcam) = ctx.world.get::<&Camera>(village).map(|c| *c) else { return false };
        if want && !self.cam_on {
            // start from where the village camera is; watch from the side away from you
            self.cam_on = true;
            self.cam_t = 0.0;
            self.cam_side = if (hero_m.x - hmid.x) * n.x + (hero_m.z - hmid.z) * n.y >= 0.0 { -1.0 } else { 1.0 };
            self.cam_pos = rig.translation;
            self.cam_look = mid;
            let e = match self.fight_cam {
                Some(e) => e,
                None => {
                    let e = ctx.world.spawn((rig, vcam, Name("fight camera".into())));
                    self.fight_cam = Some(e);
                    e
                }
            };
            ctx.set_main_camera(e);
        }
        if !self.cam_on {
            return false;
        }
        let Some(fc) = self.fight_cam else { return false };
        self.cam_t = (self.cam_t + if want { dt } else { -dt } / 0.9).clamp(0.0, 1.0);
        if self.cam_t <= 0.0 {
            // back: the village camera draws again (the fight's goes: no MainCamera marker left, as the battle screen
            // expects, which takes the cameras' `Camera` off)
            self.cam_on = false;
            ctx.despawn(fc);
            self.fight_cam = None;
            return false;
        }
        // from the side, a little above, far enough that both beasts (and the space between) fit
        // (Kaigen's camera sees 60 degrees high, ours 38: the same framing from further away)
        let tan_v = (vcam.fov_y_deg * 0.5).to_radians().tan();
        let mut dist = (sep * 0.8 + 1.6).max(5.0) * (30f32.to_radians().tan() / tan_v).max(1.0);
        // a phone held upright: both beasts (half a beast's width beyond each) fit across
        let (w, h) = ctx.renderer.size();
        let aspect = w as f32 / h.max(1) as f32;
        dist = dist.max((sep * 0.5 + 1.4) / (tan_v * aspect) / 1.2);
        let look = mid + Vec3::Y * 0.6;
        let pos = look + Vec3::new(n.x * self.cam_side * dist, dist * 0.72, n.y * self.cam_side * dist);
        let kf = 1.0 - (-dt * 3.0).exp(); // eases after the beasts as they move
        if want {
            self.cam_pos = self.cam_pos.lerp(pos, kf);
            self.cam_look = self.cam_look.lerp(look, kf);
        }
        let k = self.cam_t * self.cam_t * (3.0 - 2.0 * self.cam_t);
        let view = Transform::from_translation(self.cam_pos).looking_at(self.cam_look, Vec3::Y);
        let t = Transform::from_translation(rig.translation.lerp(view.translation, k)).with_rotation(rig.rotation.slerp(view.rotation, k));
        if let Ok(mut tr) = ctx.world.get::<&mut Transform>(fc) {
            *tr = t;
        }
        if let Ok(mut c) = ctx.world.get::<&mut Camera>(fc) {
            *c = vcam; // (the layers follow the area)
        }
        true
    }

    // ------------------------------------------------------------------ the HUD

    /// The wild one's name and HP (top centre), yours with the skill tiles and Run (bottom centre); between fights a
    /// small pill with its HP (a moon while it rests). `pill`: outdoors and no dialog.
    pub fn draw(&self, ctx: &mut Ctx, pill: bool, night: bool, narrow: bool) {
        let (Some(me_id), me, wf) = (self.me.id, &self.me, &self.wild) else {
            for s in ["vox_wild", "vox_mine", "vox_pill"] {
                ctx.ui.set(s, "");
            }
            return;
        };
        let bar = |f: f32, w: f32, c: &str| format!("<div class='vx-bar' style='width:{w:.0}px'><i style='width:{:.1}%;background:{c}'></i></div>", (f.clamp(0.0, 1.0) * 100.0));
        if self.phase != Phase::None {
            let wid = wf.id.unwrap_or(0);
            let wfrac = wf.hp / wf.max.max(1.0);
            ctx.ui.set(
                "vox_wild",
                &format!(
                    "<div class='vx-box{}'><span class='vx-name'>{} {} &nbsp; {}/{}</span>{}</div>",
                    if wf.shiny { " shiny" } else { "" },
                    if wf.shiny { "\u{2728} Shiny" } else { "Wild" },
                    cards::card(wid).name,
                    wf.hp as i32,
                    wf.max as i32,
                    bar(wfrac, 240.0, if wf.shiny { "#ffc828" } else { hp_color(wfrac) })
                ),
            );
            let mut tiles = String::new();
            for k in 0..3 {
                let run = k == 2;
                let s = if run { sk("Run", SkillKind::Dash, 0.0, 0.0, 0xa0503c) } else { skill(me_id, k) };
                let ready = run || (self.cool[k] <= 0.0 && self.phase == Phase::Fight);
                let c = rgb(s.color);
                let css = |m: f32| {
                    let (r, g, b) = (lin_to_srgb(c.x) * m, lin_to_srgb(c.y) * m, lin_to_srgb(c.z) * m);
                    format!("rgb({:.0},{:.0},{:.0})", r.min(255.0), g.min(255.0), b.min(255.0))
                };
                let fill = if ready { 1.0 } else if self.cool[k] > 0.0 { 1.0 - self.cool[k] / s.cooldown.max(0.01) } else { 0.0 };
                let (c1, c2, c3) = if ready { (css(210.0), css(210.0), mix_white(&css(210.0))) } else { (css(210.0 * 0.55), css(210.0 * 0.75), String::new()) };
                let action = if run { "vox_run".to_string() } else { format!("vox_skill:{k}") };
                tiles += &format!(
                    "<div class='vx-sk'><button class='vx-tile{}' data-action='{action}' style='--g:{};--c1:{c1};--c2:{c2};--c3:{c3}'>{}<svg viewBox='0 0 24 24'>{}</svg>{}{}</button><div class='vx-lbl'>{}</div></div>",
                    if ready { " ready" } else { "" },
                    if ready && !run { css(210.0).replace("rgb(", "rgba(").replace(')', ",.63)") } else { "rgba(0,0,0,.35)".into() },
                    if fill > 0.0 { format!("<div class='vx-fill' style='height:{:.1}%'></div>", fill * 100.0) } else { String::new() },
                    if run { ICON_BACK } else { skill_icon(&s) },
                    if !ready && self.cool[k] > 0.0 { format!("<span class='vx-sec'>{:.1}</span>", self.cool[k]) } else { String::new() },
                    if run { String::new() } else { format!("<span class='vx-key'>{}</span>", k + 1) },
                    s.name
                );
            }
            let mfrac = me.hp / me.max.max(1.0);
            ctx.ui.set(
                "vox_mine",
                &format!(
                    "<div class='vx-box vx-mine'><span>{} &nbsp; {}/{}</span>{}<div class='vx-skills'>{tiles}</div></div>",
                    cards::card(me_id).name,
                    me.hp as i32,
                    me.max as i32,
                    bar(mfrac, if narrow { 230.0 } else { 300.0 }, hp_color(mfrac))
                ),
            );
            ctx.ui.set("vox_pill", "");
            return;
        }
        ctx.ui.set("vox_wild", "");
        ctx.ui.set("vox_mine", "");
        if !pill {
            ctx.ui.set("vox_pill", "");
            return;
        }
        let f = me.hp / me.max.max(1.0);
        ctx.ui.set(
            "vox_pill",
            &format!(
                "<div class='vx-pill{}' style='margin-top:{}px'><span>{}{} &nbsp;{}/{}</span>{}</div>",
                if night { " night" } else { "" },
                if narrow { 54 } else { 0 },
                if me.tired { "\u{263e} " } else { "" },
                cards::card(me_id).name,
                me.hp as i32,
                me.max as i32,
                bar(f, 150.0, if me.tired { "#7882c8" } else { hp_color(f) })
            ),
        );
    }

    pub fn clear_hud(ctx: &mut Ctx) {
        for s in ["vox_wild", "vox_mine", "vox_pill"] {
            ctx.ui.set(s, "");
        }
    }

    // ------------------------------------------------------------------ tests

    /// Who pushes the tall grass (grass.rs): your Voxelmon while it is shown, the wild one in a fight (feet in
    /// metres, radius in metres).
    pub fn pushers(&self) -> Vec<(Vec3, f32)> {
        let mut v = vec![];
        for (f, on) in [(&self.me, self.me.id.is_some_and(|id| self.models.get(&id).is_some_and(|m| m.1))), (&self.wild, self.phase != Phase::None && self.wild.id.is_some())] {
            if on {
                v.push((f.p * STUD, (f.size * 0.35).clamp(0.6, 3.0) * STUD));
            }
        }
        v
    }

    pub fn describe(&self, ctx: &Ctx) -> Value {
        let r = |v: f32| (v * 100.0).round() / 100.0;
        let fighter = |f: &Fighter| {
            f.id.map(|id| {
                let shown = self.models.get(&id).is_some_and(|m| m.1);
                let scale = self.models.get(&id).and_then(|m| ctx.world.get::<&Transform>(m.0).ok().map(|t| t.scale.y / STUD));
                json!({"id": cards::card(id).id, "hp": f.hp.round(), "max": f.max, "tired": f.tired, "shiny": f.shiny, "x": r(f.p.x), "y": r(f.p.y), "z": r(f.p.z),
                       "act": f.act, "shown": shown, "size": scale.map(r), "children": self.models.get(&id).map(|m| ctx.world.query::<&Parent>().iter().filter(|p| p.0 == m.0).count())})
            })
        };
        json!({
            "phase": self.phase,
            "t": r(self.phase_t),
            "me": fighter(&self.me),
            "wild": fighter(&self.wild),
            "skills": self.me.id.map(|id| (0..2).map(|k| { let s = skill(id, k); json!({"name": s.name, "kind": s.kind, "cooldown": s.cooldown}) }).collect::<Vec<_>>()),
            "cool": [r(self.cool[0]), r(self.cool[1])],
            "result": self.result,
            "center": [r(self.center.x), r(self.center.z)],
            "walk": [r(self.walk.x), r(self.walk.y)],
            "meter": r(self.meter),
            "rate": self.rate,
            "cam_on": self.cam_on,
            "cam_t": r(self.cam_t),
            "projectiles": self.proj.iter().filter(|p| p.live).count(),
            "numbers": self.nums.len(),
            "show": self.show_id.map(|id| json!({"id": cards::card(id).id, "size": r(self.show_size)})),
            "fights": self.fights,
            "skills_used": self.skills_used,
            "hits": self.hits,
            "rustles": self.rustles,
            "models": self.models.len(),
            "reward_pending": self.reward_card.map(|c| cards::card(c).id),
        })
    }

    /// Debug cheats: HP (`me` / `wild`), the wild one never lunges.
    pub fn cheat(&mut self, a: &Value) {
        if let Some(h) = a["hp"]["me"].as_f64() {
            self.me.hp = h as f32;
        }
        if let Some(h) = a["hp"]["wild"].as_f64() {
            self.wild.hp = h as f32;
        }
        if let Some(t) = a["tired"].as_bool() {
            self.me.tired = t;
        }
        if let Some(i) = a["wild_idle"].as_bool() {
            self.wild_idle = i;
        }
        if let Some(r) = a["rate"].as_f64() {
            self.rate = r as f32;
        }
        if let Some(s) = a["seed"].as_u64() {
            self.reseed(s as u32);
        }
    }
}

fn lin_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// A colour mixed 25 % with white (the ready tile's top).
fn mix_white(rgb: &str) -> String {
    let v: Vec<f32> = rgb.trim_start_matches("rgb(").trim_end_matches(')').split(',').filter_map(|x| x.parse().ok()).collect();
    if v.len() != 3 {
        return rgb.to_string();
    }
    format!("rgb({:.0},{:.0},{:.0})", v[0] + (255.0 - v[0]) * 0.25, v[1] + (255.0 - v[1]) * 0.25, v[2] + (255.0 - v[2]) * 0.25)
}
