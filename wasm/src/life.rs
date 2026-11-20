//! Village life (Kaigen `st_life.c`): golden studs, the duck family (and feeding it), the jumping fish,
//! butterflies, fireflies at night, the ball and the goal (Pip plays along), the treasure chest, rain and
//! puddles, the glowing lamps, the park bench, dancing, and Pip: his walk around the pond, hide and seek,
//! tag and football. Positions in studs (y too), unless named `_m`.
//!
//! [`Life`] is the simulation (engine-free, its own generator, run in the play screen's fixed step:
//! `Life::step`); it queues its sounds in `Life::sfx` and says `dirty` when the save changed.
//! [`LifeView`] owns the entities (meshes made once: [`LifeArt`] in `TownAssets`) and places them from
//! the state (`LifeView::sync`). Pip's figure is the play screen's (npc "kid"): it reads `pip`, `pip_yaw`,
//! `pip_speed` and `pip_pose`.

use crate::figure::Pose;
use crate::save::GameSave;
use crate::sound::Sfx;
use crate::town::{self, rooms, Rect, TownAssets, STUD};
use engine::geom::MeshBuilder;
use engine::mesh::MeshData;
use engine::prelude::*;
use serde_json::{json, Value};

pub const GOLDEN_COUNT: usize = 15;
const DUCKS: usize = 4;
const CRUMBS: usize = 5;
const FLIES: usize = 8;
const FIREFLIES: usize = 30;
pub const RAIN_DROPS: usize = 160;
const PUDDLES: usize = 14;

/// The golden studs (id, x, z): outside, and one in each room.
pub const GOLDEN: [(&str, f32, f32); GOLDEN_COUNT] = [
    ("g-nw", -22.0, -24.0),
    ("g-pond", -14.0, 22.5),
    ("g-garden", -23.0, 12.0),
    ("g-park", 15.0, -12.0),
    ("g-route1", 0.0, -40.0),
    ("g-route2", -7.5, -30.0),
    ("g-route3", 6.5, -36.0),
    ("g-gap", 24.5, -14.0),
    ("g-behind", 64.5, -20.0),
    ("g-yard", 60.0, 22.8),
    ("g-lamp", -25.0, 0.5),
    ("g-home", 191.0, 6.0),
    ("g-dex", 256.0, -5.0),
    ("g-lab", 342.0, 8.5),
    ("g-bedroom", 194.5, 44.6),
];
fn golden_base(x: f32) -> f32 {
    1.4 + if x > 150.0 { 0.4 } else { 0.0 }
}

/// Pip's loop around the pond, never through it (the west bank between the water and the garden fence).
const PIP_WAYPOINTS: [[f32; 2]; 6] = [[-8.0, 13.2], [-3.0, 13.8], [-2.8, 23.0], [-11.0, 23.2], [-12.9, 22.9], [-12.9, 14.0]];
/// Studs per second: Pip strolling (the three.js speed; the Kaigen port scaled its speeds by 1.24 to its faster walk,
/// this port walks at the three.js speed, so its speeds are the three.js ones).
pub const PIP_WALK: f32 = 3.2;
const PIP_FLEE: f32 = 9.6;
const PIP_BALL: f32 = 6.5;
/// Within this many studs Pip stops and looks at you (and you can talk to him).
pub const PIP_TALK: f32 = 6.0;
/// The park bench: where you sit (studs) and the target in front of it.
pub const BENCH_SEAT: [f32; 2] = [15.0, -8.3];

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum PipMode {
    /// His loop around the pond.
    Walk,
    /// Chasing the ball.
    Football,
    /// Hide and seek: hidden somewhere.
    Hiding,
    /// Found, or the game is over: waves until you walk off.
    Wave,
    /// Tag: runs from you.
    Flee,
}

#[derive(Clone, Copy, Default)]
struct Crumb {
    active: bool,
    landed: bool,
    t: f32,
    from: Vec3,
    to: Vec3,
    /// where it is drawn (studs)
    at: Vec3,
    spin: f32,
}

/// What the play screen tells the life step about you and the world.
pub struct LifeIn {
    /// your feet (studs)
    pub p: Vec3,
    /// your walking direction (world x/z, unit or zero)
    pub move_dir: Vec2,
    /// your speed (m/s)
    pub speed_m: f32,
    /// sprinting: the run key (or the stick pushed far) and a move wish
    pub sprinting: bool,
    pub outside: bool,
    /// the engine's night 0..1
    pub night: f32,
    /// days passed (the play screen counts the clock's wraps)
    pub day: u32,
    /// a dialog with Pip is open
    pub talking_pip: bool,
}

pub struct Life {
    seed: u32,
    pub time: f32,
    pub blocks: Vec<Rect>,
    // ---- golden studs (drawn from the save's mask) ----
    // ---- ducks ----
    pub duck_p: [Vec3; DUCKS],
    duck_yaw: [f32; DUCKS],
    duck_goal: Vec2,
    duck_timer: f32,
    quack_t: f32,
    crumbs: [Crumb; CRUMBS],
    feed_cd: f32,
    // ---- fish ----
    pub fish_next: f32,
    fish_jump: f32,
    fish_at: Vec3,
    fish_rot: Quat,
    // ---- butterflies and fireflies ----
    fly_gone: [f32; FLIES],
    fly_catch: [f32; FLIES],
    pub fly_at: [(Vec3, f32, f32); FLIES],
    ff_home: [Vec3; FIREFLIES],
    ff_gone: [f32; FIREFLIES],
    ff_catch: [f32; FIREFLIES],
    pub ff_at: [(Vec3, f32); FIREFLIES],
    // ---- the ball ----
    pub ball_p: Vec3,
    pub ball_v: Vec3,
    kick_cd: f32,
    goal_t: f32,
    goal_armed: bool,
    // ---- the chest ----
    pub chest_p: Vec3,
    pub chest_live: bool,
    pub chest_open: bool,
    chest_t: f32,
    pub chest_next: f32,
    pub chest_day: Option<u32>,
    chest_draw: (f32, f32, f32),
    // ---- weather ----
    drop_p: Vec<Vec3>,
    pub rain: f32,
    pub rain_target: f32,
    pub rain_timer: f32,
    /// the rain you hear and see (0 inside)
    pub rain_level: f32,
    puddle_p: Vec<Vec3>,
    pub wet: f32,
    splash_cd: f32,
    pub splashes: u32,
    pub night: f32,
    day: u32,
    // ---- your doings ----
    pub sitting: bool,
    pub dancing: bool,
    // ---- Pip ----
    pub pip: Vec2,
    pub pip_yaw: f32,
    pub pip_mode: PipMode,
    pub pip_wp: usize,
    pub pip_pause: f32,
    /// Pip's speed this step (studs/s) for his walk cycle.
    pub pip_speed: f32,
    pub pip_pose: Pose,
    pip_t: f32,
    pub pip_cd: f32,
    hint_t: f32,
    /// > 0 while the countdown runs (eyes closed)
    pub seek_count: f32,
    /// seconds of hide and seek left
    pub seek_left: f32,
    /// the screen goes dark (the hide and seek countdown)
    pub fade: bool,
    /// Tests: Pip's walk holds still.
    pub frozen: bool,
    // ---- out ----
    /// The message at the top and its seconds left.
    pub toast: String,
    pub toast_t: f32,
    /// sounds to play (effect, pitch, volume); the fish's splash clip
    pub sfx: Vec<(Sfx, f32, f32)>,
    pub fish_splash: bool,
    /// the save changed
    pub dirty: bool,
}

impl Default for Life {
    fn default() -> Self {
        let mut l = Self {
            seed: 4242,
            time: 0.0,
            blocks: vec![],
            duck_p: std::array::from_fn(|i| Vec3::new(-8.0, 0.3, 18.5 + i as f32 * 0.9)),
            duck_yaw: [0.0; DUCKS],
            duck_goal: Vec2::new(-8.0, 18.5),
            duck_timer: 3.0,
            quack_t: 0.0,
            crumbs: [Crumb::default(); CRUMBS],
            feed_cd: 0.0,
            fish_next: 8.0,
            fish_jump: -1.0,
            fish_at: Vec3::new(-7.0, 0.1, 16.5),
            fish_rot: Quat::IDENTITY,
            fly_gone: [0.0; FLIES],
            fly_catch: [0.0; FLIES],
            fly_at: [(Vec3::ZERO, 0.0, 1.0); FLIES],
            ff_home: [Vec3::ZERO; FIREFLIES],
            ff_gone: [0.0; FIREFLIES],
            ff_catch: [0.0; FIREFLIES],
            ff_at: [(Vec3::ZERO, 0.0); FIREFLIES],
            ball_p: Vec3::new(15.0, 0.6, -6.2),
            ball_v: Vec3::ZERO,
            kick_cd: 0.0,
            goal_t: 0.0,
            goal_armed: true,
            chest_p: Vec3::ZERO,
            chest_live: false,
            chest_open: false,
            chest_t: 0.0,
            chest_next: 50.0,
            chest_day: None,
            chest_draw: (0.0, 0.0, 0.0),
            drop_p: vec![Vec3::new(0.0, -1.0, 0.0); RAIN_DROPS],
            rain: 0.0,
            rain_target: 0.0,
            rain_timer: 0.0,
            rain_level: 0.0,
            puddle_p: vec![],
            wet: 0.0,
            splash_cd: 0.0,
            splashes: 0,
            night: 0.0,
            day: 0,
            sitting: false,
            dancing: false,
            pip: Vec2::new(PIP_WAYPOINTS[0][0], PIP_WAYPOINTS[0][1]),
            pip_yaw: 0.0,
            pip_mode: PipMode::Walk,
            pip_wp: 0,
            pip_pause: 0.0,
            pip_speed: 0.0,
            pip_pose: Pose::Normal,
            pip_t: 0.0,
            pip_cd: 0.0,
            hint_t: 0.0,
            seek_count: 0.0,
            seek_left: 0.0,
            fade: false,
            frozen: false,
            toast: String::new(),
            toast_t: 0.0,
            sfx: vec![],
            fish_splash: false,
            dirty: false,
        };
        // (the same order of rolls as Kaigen life_init: fireflies, the rain timer, the puddles)
        for i in 0..FIREFLIES {
            l.ff_home[i] = if i < 14 {
                Vec3::new(l.range(-8.0, 8.0), 0.0, l.range(-40.0, -29.0))
            } else if i < 24 {
                Vec3::new(l.range(-13.0, -3.0), 0.0, l.range(14.0, 23.0))
            } else {
                Vec3::new(l.range(10.0, 20.0), 0.0, l.range(-15.0, -8.0))
            };
        }
        l.rain_timer = l.range(150.0, 240.0);
        // puddles on the village roads, at least 5 apart, the whole puddle on the road
        let roads: Vec<Rect> = town::ROADS.iter().copied().filter(|r| r.z0 > town::NORTH_EDGE).collect();
        let mut tries = 0;
        while tries < 400 && l.puddle_p.len() < PUDDLES {
            tries += 1;
            let rd = roads[(l.rand() * roads.len() as f32) as usize % roads.len()];
            let rr = l.range(0.9, 1.8);
            let x = l.range(rd.x0 + rr, rd.x1 - rr);
            let z = l.range(rd.z0 + rr, rd.z1 - rr);
            if rd.x1 - rd.x0 < 2.0 * rr || rd.z1 - rd.z0 < 2.0 * rr {
                continue;
            }
            if l.puddle_p.iter().any(|q| Vec2::new(q.x - x, q.z - z).length() <= 5.0) {
                continue;
            }
            l.puddle_p.push(Vec3::new(x, rr, z));
        }
        l
    }
}

/// Wraps an angle to -pi..pi.
pub fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// The yaw that turns a mesh facing +x towards `d` (x, z).
fn yaw_to(d: Vec2) -> f32 {
    (-d.y).atan2(d.x)
}

pub fn in_pond(x: f32, z: f32, m: f32) -> bool {
    x > -12.0 + m && x < -4.0 - m && z > 15.0 + m && z < 22.0 - m
}

fn on_road(x: f32, z: f32) -> bool {
    town::ROADS.iter().any(|r| x >= r.x0 && x <= r.x1 && z >= r.z0 && z <= r.z1)
}

impl Life {
    pub fn rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed >> 8) as f32 / 16777216.0
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.rand()
    }

    pub fn toast(&mut self, text: &str) {
        self.toast = text.to_string();
        self.toast_t = 2.6;
    }

    fn say(&mut self, text: &str) {
        self.toast(&format!("Pip: \"{text}\""));
    }

    fn snd(&mut self, s: Sfx, pitch: f32, vol: f32) {
        self.sfx.push((s, pitch, vol));
    }

    /// Is (x, z) in a town collider (grown by r)?
    fn blocked(&self, x: f32, z: f32, r: f32) -> bool {
        self.blocks.iter().any(|b| x > b.x0 - r && x < b.x1 + r && z > b.z0 - r && z < b.z1 + r)
    }

    /// Pip back at the start of his loop (tests).
    pub fn pip_home(&mut self) {
        self.pip = Vec2::new(PIP_WAYPOINTS[0][0], PIP_WAYPOINTS[0][1]);
        self.pip_wp = 0;
        self.pip_mode = PipMode::Walk;
        self.pip_pause = 0.0;
        self.pip_yaw = 0.0;
        self.seek_count = 0.0;
        self.seek_left = 0.0;
        self.fade = false;
        self.pip_cd = 0.0;
    }

    /// Every frame: the toast fades.
    pub fn update(&mut self, dt: f32) {
        self.toast_t = (self.toast_t - dt).max(0.0);
    }

    /// One fixed step of the whole village life (Kaigen `life_update`).
    pub fn step(&mut self, s: &mut GameSave, i: &LifeIn, dt: f32) {
        self.time += dt;
        self.night = i.night;
        self.day = i.day;
        let walking = i.speed_m > 0.5;
        self.golden(s, i.p);
        self.ducks(i.p, dt, i.outside);
        self.fish(i.p, dt, i.outside);
        self.butterflies(s, i.p, dt, i.outside);
        self.fireflies(s, i.p, dt, i.outside, walking);
        self.ball(s, i.p, i.move_dir, i.speed_m, dt, i.outside);
        self.chest(i.p, dt, i.outside);
        self.weather(i.p, dt, i.outside, walking);
        self.step_pip(s, Vec2::new(i.p.x, i.p.z), dt, i.outside, i.sprinting, i.talking_pip);
    }

    // ------------------------------------------------------------------ the parts

    fn golden(&mut self, s: &mut GameSave, p: Vec3) {
        for (k, g) in GOLDEN.iter().enumerate() {
            if (s.golden_mask >> k) & 1 == 1 {
                continue;
            }
            let (dx, dz) = (g.1 - p.x, g.2 - p.z);
            if dx * dx + dz * dz < 1.4 * 1.4 * 1.3 {
                s.golden_mask |= 1 << k;
                s.studs += 5;
                let n = s.golden_count();
                self.snd(Sfx::Coin, 1.0, 0.9);
                self.toast(&format!("Golden stud! {n}/15 found \u{b7} +5 studs"));
                if n as usize == GOLDEN_COUNT && !s.legendary {
                    s.legendary = true;
                    self.snd(Sfx::Fanfare, 1.0, 0.9);
                    self.toast("All golden studs found! A legendary card appears\u{2026}");
                }
                self.dirty = true;
            }
        }
    }

    fn ducks(&mut self, p: Vec3, dt: f32, outside: bool) {
        // crumbs fly and land
        for c in 0..CRUMBS {
            let mut cr = self.crumbs[c];
            if !cr.active {
                continue;
            }
            cr.t += dt;
            if !cr.landed {
                let k = (cr.t / 0.6).clamp(0.0, 1.0);
                let mut q = cr.from.lerp(cr.to, k);
                q.y += (k * std::f32::consts::PI).sin() * 1.6;
                cr.at = q;
                cr.spin = cr.t * 5.0;
                if k >= 1.0 {
                    cr.landed = true;
                    self.snd(Sfx::Splash, 1.6, 0.25);
                }
            } else {
                cr.at = Vec3::new(cr.to.x, 0.3 + (self.time * 3.0 + c as f32).sin() * 0.03, cr.to.z);
                cr.spin = 0.0;
            }
            self.crumbs[c] = cr;
        }
        // the mother: a crumb, you, or a wandering goal
        self.duck_timer -= dt;
        let close_by = outside && p.x > -16.5 && p.x < 0.5 && p.z > 10.5 && p.z < 26.5;
        let mut speed = 1.1;
        if self.duck_timer <= 0.0 {
            self.duck_goal = Vec2::new(self.range(-11.0, -5.0), self.range(16.0, 21.0));
            self.duck_timer = self.range(5.0, 11.0);
        }
        if close_by {
            self.duck_goal = Vec2::new(p.x.clamp(-11.0, -5.0), p.z.clamp(16.0, 21.0));
            speed = 2.2;
        }
        for i in 0..DUCKS {
            let mut goal = self.duck_goal;
            let mut sp = speed;
            if i > 0 {
                // a line behind the one in front
                let lead = self.duck_p[i - 1];
                let fwd = Vec2::new(self.duck_yaw[i - 1].cos(), -self.duck_yaw[i - 1].sin());
                let gap = if i == 1 { 1.15 } else { 0.75 };
                goal = Vec2::new(lead.x - fwd.x * gap, lead.z - fwd.y * gap);
                sp = 2.6;
            }
            // a crumb in the water wins
            let mut best = 1e9f32;
            let mut bi = None;
            for (c, cr) in self.crumbs.iter().enumerate() {
                if cr.active && cr.landed {
                    let d = Vec2::new(cr.to.x - self.duck_p[i].x, cr.to.z - self.duck_p[i].z).length();
                    if d < best {
                        best = d;
                        bi = Some(c);
                    }
                }
            }
            if let Some(c) = bi {
                goal = Vec2::new(self.crumbs[c].to.x, self.crumbs[c].to.z);
                sp = if i == 0 { 2.4 } else { 2.9 };
                let scale = if i == 0 { 0.95 } else { 0.52 };
                if best < 0.45 * (scale + 0.4) * 1.6 {
                    self.crumbs[c].active = false;
                    self.snd(Sfx::Quack, if i == 0 { 1.0 } else { 1.7 }, 0.5);
                }
            }
            let d = goal - Vec2::new(self.duck_p[i].x, self.duck_p[i].z);
            let len = d.length();
            if len > 0.15 {
                let step = len.min(sp * len.min(1.0) * dt * 1.4);
                self.duck_p[i].x += d.x / len * step;
                self.duck_p[i].z += d.y / len * step;
                self.duck_yaw[i] += wrap(yaw_to(d) - self.duck_yaw[i]) * (dt * 3.0).min(1.0);
            } else if i == 0 && close_by {
                let w = yaw_to(Vec2::new(p.x - self.duck_p[i].x, p.z - self.duck_p[i].z));
                self.duck_yaw[i] += wrap(w - self.duck_yaw[i]) * (dt * 3.0).min(1.0);
            }
            self.duck_p[i].x = self.duck_p[i].x.clamp(-11.4, -4.6);
            self.duck_p[i].z = self.duck_p[i].z.clamp(15.6, 21.4);
        }
        // quacks while you are close by
        self.quack_t -= dt;
        if close_by && self.quack_t <= 0.0 {
            self.snd(Sfx::Quack, 1.0, 0.45);
            if self.rand() < 0.6 {
                let pitch = self.range(1.6, 1.9);
                self.snd(Sfx::Quack, pitch, 0.3);
            }
            self.quack_t = self.range(2.5, 5.5);
        }
        self.feed_cd = (self.feed_cd - dt).max(0.0);
    }

    /// The pond's feeding spot: five crumbs thrown into the water close by you (Kaigen `life_feed`).
    pub fn feed(&mut self, s: &mut GameSave, p: Vec3) {
        if self.crumbs.iter().any(|c| c.active) || self.feed_cd > 0.0 {
            return;
        }
        self.feed_cd = 3.0;
        self.snd(Sfx::Quack, 1.0, 0.5);
        for i in 0..CRUMBS {
            let tx = (p.x - 2.4 + self.range(-1.75, 1.75)).clamp(-11.2, -4.8);
            let tz = (p.z + self.range(-1.75, 1.75)).clamp(15.8, 21.2);
            self.crumbs[i] = Crumb { active: true, landed: false, t: -(i as f32) * 0.08, from: Vec3::new(p.x, 2.6, p.z), to: Vec3::new(tx, 0.3, tz), at: Vec3::new(p.x, 2.6, p.z), spin: 0.0 };
        }
        s.duck_feeds += 1;
        if s.duck_feeds == 1 {
            self.toast("Quack! The ducks love bread.");
        } else if s.duck_feeds % 10 == 0 {
            self.toast(&format!("You've fed the ducks {} times!", s.duck_feeds));
        }
        self.dirty = true;
    }

    fn fish(&mut self, p: Vec3, dt: f32, outside: bool) {
        let a = self.time * 0.6;
        let mut at = Vec3::new(-8.0 + a.cos() * 3.0, 0.12, 18.5 + a.sin() * 2.0);
        let yaw = yaw_to(Vec2::new(-a.sin() * 3.0, a.cos() * 2.0));
        let mut pitch = 0.0;
        if self.fish_jump >= 0.0 {
            self.fish_jump += dt / 0.9;
            let k = self.fish_jump.min(1.0);
            at.y = 0.08 + (k * std::f32::consts::PI).sin() * 1.9;
            pitch = (0.5 - k) * 2.4;
            if self.fish_jump >= 1.0 {
                self.fish_jump = -1.0;
                self.snd(Sfx::Splash, 1.0, 0.35);
            }
        } else {
            self.fish_next -= dt;
            let d = Vec2::new(p.x + 8.0, p.z - 18.5).length();
            if self.fish_next <= 0.0 && outside && d < 14.0 {
                self.fish_jump = 0.0;
                self.fish_next = self.range(6.0, 14.0);
                self.fish_splash = true;
            }
        }
        self.fish_at = at;
        self.fish_rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(pitch);
    }

    fn butterflies(&mut self, s: &mut GameSave, p: Vec3, dt: f32, outside: bool) {
        const C: [[f32; 3]; FLIES] = [[-19.0, 4.0, 12.0], [-17.0, 3.0, 11.0], [15.0, 4.0, -11.0], [0.0, 5.0, -36.0], [-4.0, 4.0, -32.0], [5.0, 4.0, -38.0], [45.0, 5.0, 16.0], [-8.0, 3.0, 19.0]];
        for i in 0..FLIES {
            let c = C[i];
            let a = self.time * (0.4 + 0.05 * i as f32) + i as f32 * 1.7;
            let mut q = Vec3::new(c[0] + a.sin() * c[1], 2.2 + (2.3 * a).sin() * 0.8, c[2] + (1.7 * a).sin() * c[1] * 0.6);
            if self.fly_catch[i] > 0.0 {
                // spiralling away
                self.fly_catch[i] += dt / 1.6;
                let k = self.fly_catch[i];
                q.x += (k * 9.0).cos() * k * 2.0;
                q.z += (k * 9.0).sin() * k * 2.0;
                q.y += k * k * 3.5;
                if k >= 1.0 {
                    self.fly_catch[i] = 0.0;
                    self.fly_gone[i] = self.range(25.0, 35.0);
                }
            } else if self.fly_gone[i] > 0.0 {
                self.fly_gone[i] -= dt;
                q.y = -50.0;
            } else if outside && Vec2::new(q.x - p.x, q.z - p.z).length() < 1.3 * 1.3 {
                self.fly_catch[i] = 0.001;
                s.studs += 1;
                s.butterflies += 1;
                self.snd(Sfx::Pop, 1.6, 0.4);
                self.toast(&format!("A butterfly! ({} caught) \u{b7} +1 stud", s.butterflies));
                self.dirty = true;
            }
            let flap = 0.25 + (self.time * 14.0 + i as f32).sin().abs() * 0.75;
            self.fly_at[i] = (q, a * 2.0, flap);
        }
    }

    fn fireflies(&mut self, s: &mut GameSave, p: Vec3, dt: f32, outside: bool, walking: bool) {
        let show = outside && self.night > 0.25;
        for i in 0..FIREFLIES {
            let h = self.ff_home[i];
            let a = self.time * (0.5 + 0.03 * i as f32) + i as f32;
            let mut q = Vec3::new(h.x + a.cos() * 1.4, 1.2 + (a * 1.3).sin() * 0.6, h.z + a.sin() * 1.4);
            let mut sc = 0.6 + 0.4 * (self.time * 2.0 + i as f32 * 1.7).sin().abs();
            if self.ff_catch[i] > 0.0 {
                self.ff_catch[i] += dt / 0.6;
                let k = self.ff_catch[i].min(1.0);
                q = q.lerp(Vec3::new(p.x, 3.0, p.z), k);
                sc *= 1.0 + k;
                if self.ff_catch[i] >= 1.0 {
                    self.ff_catch[i] = 0.0;
                    self.ff_gone[i] = 30.0;
                }
            } else if self.ff_gone[i] > 0.0 {
                self.ff_gone[i] -= dt;
                sc = 0.0;
            } else if show && self.night > 0.5 && walking && Vec2::new(q.x - p.x, q.z - p.z).length() < 1.2 * 1.4 {
                self.ff_catch[i] = 0.001;
                s.fireflies += 1;
                s.studs += 1;
                self.snd(Sfx::Coin, 1.5, 0.35);
                self.toast(&format!("A firefly! ({} caught) \u{b7} +1 stud", s.fireflies));
                self.dirty = true;
            }
            if !show {
                q.y = -50.0;
            }
            self.ff_at[i] = (q, sc);
        }
    }

    fn ball(&mut self, s: &mut GameSave, p: Vec3, move_dir: Vec2, speed_m: f32, dt: f32, outside: bool) {
        self.kick_cd -= dt;
        let d = Vec2::new(self.ball_p.x - p.x, self.ball_p.z - p.z);
        let dist = d.length();
        let ps = speed_m / STUD; // studs/s
        if outside && dist < 0.6 + 1.0 && ps > 1.0 && self.kick_cd <= 0.0 {
            let away = if dist > 0.01 { d / dist } else { move_dir };
            let dir = (away * 0.6 + move_dir * 0.4).normalize_or(away);
            let power = (3.0 + ps * 0.9).min(14.0);
            self.ball_v = Vec3::new(dir.x * power, 0.0, dir.y * power);
            self.kick_cd = 0.25;
            self.snd(Sfx::Kick, 1.0, (power / 14.0).min(1.0));
        }
        if dist < 1.3 && dist > 0.01 && outside {
            // never inside you
            self.ball_p.x = p.x + d.x / dist * 1.3;
            self.ball_p.z = p.z + d.y / dist * 1.3;
        }
        // roll, bounce off the colliders (70 % of the speed)
        let sp = Vec2::new(self.ball_v.x, self.ball_v.z).length();
        if sp > 0.02 {
            let (nx, nz) = (self.ball_p.x + self.ball_v.x * dt, self.ball_p.z + self.ball_v.z * dt);
            let bx = self.blocked(nx, self.ball_p.z, 0.5) || in_pond(nx, self.ball_p.z, -0.6);
            let bz = self.blocked(self.ball_p.x, nz, 0.5) || in_pond(self.ball_p.x, nz, -0.6);
            if bx {
                self.ball_v.x *= -0.7;
            } else {
                self.ball_p.x = nx;
            }
            if bz {
                self.ball_v.z *= -0.7;
            } else {
                self.ball_p.z = nz;
            }
            if (bx || bz) && sp > 2.0 {
                self.snd(Sfx::Kick, 1.5, 0.3);
            }
            let k = (-1.4 * dt).exp();
            self.ball_v.x *= k;
            self.ball_v.z *= k;
        } else {
            self.ball_v = Vec3::ZERO;
        }
        // the goal
        let in_goal = self.ball_p.x > 25.0 && self.ball_p.x < 27.0 && self.ball_p.z > -7.6 && self.ball_p.z < -4.4;
        if in_goal && self.goal_armed {
            self.goal_armed = false;
            self.goal_t = 0.0;
            s.goals += 1;
            s.studs += 1;
            self.snd(Sfx::Fanfare, 1.0, 0.7);
            self.toast(&format!("GOAL! ({} scored) \u{b7} +1 stud", s.goals));
            self.dirty = true;
        }
        if !self.goal_armed {
            self.goal_t += dt;
            if self.ball_p.x < 24.2 {
                self.goal_armed = true;
            } else if self.goal_t > 2.0 && sp < 1.5 {
                self.ball_v = Vec3::new(-9.0, 0.0, 0.0);
            }
        }
    }

    fn chest(&mut self, p: Vec3, dt: f32, outside: bool) {
        if self.chest_live {
            self.chest_t += dt;
            let mut shrink = 1.0;
            if self.chest_open && self.chest_t > 3.0 {
                shrink = 1.0 - ((self.chest_t - 3.0) / 0.6).min(1.0);
                if shrink <= 0.0 {
                    self.chest_live = false;
                    self.chest_next = 30.0;
                }
            }
            let jiggle = if self.chest_open { 0.0 } else { (self.time * 9.0).sin() * 0.04 * if (self.time * 1.3).sin() > 0.6 { 1.0 } else { 0.0 } };
            let lid = if self.chest_open { -(self.chest_t / 0.4).min(1.0) * 1.6 } else { 0.0 };
            self.chest_draw = (1.4 * shrink, jiggle, lid);
            return;
        }
        self.chest_next -= dt;
        if self.chest_next > 0.0 || !outside || self.chest_day == Some(self.day) {
            return;
        }
        for _ in 0..80 {
            let x = self.range(-24.0, 64.0);
            let z = self.range(-23.0, 21.0);
            let d = Vec2::new(x - p.x, z - p.z).length();
            if !(12.0..=50.0).contains(&d) || on_road(x, z) || self.blocked(x, z, 1.8) || in_pond(x, z, -1.5) {
                continue;
            }
            self.chest_p = Vec3::new(x, 0.0, z);
            self.chest_live = true;
            self.chest_open = false;
            self.chest_t = 0.0;
            self.chest_day = Some(self.day);
            self.chest_draw = (1.4, 0.0, 0.0);
            self.snd(Sfx::Coin, 0.7, 0.5);
            self.toast("Something sparkles somewhere in the village\u{2026}");
            return;
        }
        self.chest_next = 20.0;
    }

    /// E at the chest (Kaigen `life_open_chest`): 3..8 studs.
    pub fn open_chest(&mut self, s: &mut GameSave) {
        if !self.chest_live || self.chest_open {
            return;
        }
        self.chest_open = true;
        self.chest_t = 0.0;
        let n = 3 + (self.rand() * 6.0) as u32;
        s.chests += 1;
        s.studs += n;
        self.snd(Sfx::Fanfare, 1.2, 0.7);
        self.toast(&format!("Treasure! You found {n} studs."));
        self.dirty = true;
    }

    /// The chest stands there and can be opened (its target is in reach).
    pub fn chest_ready(&self) -> bool {
        self.chest_live && !self.chest_open
    }

    fn weather(&mut self, p: Vec3, dt: f32, outside: bool, walking: bool) {
        self.rain_timer -= dt;
        if self.rain_timer <= 0.0 {
            if self.rain_target > 0.5 {
                self.rain_target = 0.0;
                self.rain_timer = self.range(180.0, 300.0);
            } else {
                self.rain_target = 1.0;
                self.rain_timer = self.range(35.0, 55.0);
            }
        }
        self.rain += (self.rain_target - self.rain) * (dt * 0.35).min(1.0);
        let level = if outside { self.rain } else { 0.0 };
        self.rain_level = level;
        let n = (level * RAIN_DROPS as f32) as usize;
        for i in 0..RAIN_DROPS {
            if i >= n {
                self.drop_p[i].y = -1.0;
                continue;
            }
            let mut d = self.drop_p[i];
            d.y -= (18.0 + (i % 7) as f32) * dt;
            if d.y < 0.0 || Vec2::new(d.x - p.x, d.z - p.z + 4.0).length() > 26.0 {
                d = Vec3::new(p.x + self.range(-20.0, 20.0), self.range(8.0, 20.0), p.z - 4.0 + self.range(-16.0, 16.0));
            }
            self.drop_p[i] = d;
        }
        // puddles: wet while it rains hard, dry slowly
        if self.rain > 0.3 {
            self.wet = (self.wet + dt * 0.04 * self.rain).min(1.0);
        } else {
            self.wet = (self.wet - dt * 0.016).max(0.0);
        }
        self.splash_cd -= dt;
        for i in 0..self.puddle_p.len() {
            let pd = self.puddle_p[i];
            let r = pd.y * (0.35 + 0.65 * self.wet);
            if self.wet >= 0.25 && walking && self.splash_cd <= 0.0 && Vec2::new(p.x - pd.x, p.z - pd.z).length() < r * 0.9 {
                self.splash_cd = 0.35;
                self.splashes += 1;
                self.snd(Sfx::Splash, 1.1, 0.45);
            }
        }
    }

    /// Rain now (tests, `life {rain}`): 1 starts a shower at once, 0 dries it up.
    pub fn set_rain(&mut self, on: bool) {
        self.rain_target = if on { 1.0 } else { 0.0 };
        self.rain = self.rain_target;
        self.rain_timer = if on { self.range(35.0, 55.0) } else { self.range(180.0, 300.0) };
    }

    /// The TV's weather: 2 rain, 1 night, 0 day (Kaigen app_update).
    pub fn weather_code(&self) -> u8 {
        if self.rain > 0.5 {
            2
        } else if self.night > 0.5 {
            1
        } else {
            0
        }
    }

    /// The park bench: sit (time runs 24x faster) until you move (Kaigen `life_sit`). Returns whether it changed.
    pub fn sit(&mut self, s: &mut GameSave, on: bool) -> bool {
        if on == self.sitting {
            return false;
        }
        self.sitting = on;
        if on {
            s.bench_sits += 1;
            self.dirty = true;
            self.snd(Sfx::Blip, 1.0, 0.7);
            self.toast("Time flies on the bench\u{2026} (move to get up)");
        }
        true
    }

    // ------------------------------------------------------------------ Pip

    /// Moves Pip toward `to` at `speed`; returns the remaining distance.
    fn pip_move(&mut self, to: Vec2, speed: f32, dt: f32) -> f32 {
        let d = to - self.pip;
        let len = d.length();
        if len > 0.05 {
            let step = len.min(speed * dt);
            self.pip += d / len * step;
            self.pip_yaw = d.x.atan2(d.y); // figures face +z at yaw 0
            self.pip_speed = speed;
        } else {
            self.pip_speed = 0.0;
        }
        len
    }

    /// Hide and seek: you found Pip (walked up to him, or pressed E at him).
    pub fn pip_found(&mut self, s: &mut GameSave) {
        self.pip_mode = PipMode::Wave;
        s.seek_wins += 1;
        s.seek_played += 1;
        s.studs += 5;
        self.dirty = true;
        self.snd(Sfx::Fanfare, 1.0, 0.7);
        self.toast("You found Pip! +5 studs");
    }

    /// Hiding and the countdown is over: E at him finds him (not his chat).
    pub fn pip_hidden(&self) -> bool {
        self.pip_mode == PipMode::Hiding && self.seek_count <= 0.0
    }

    /// The dialog's "Hide and seek": eyes closed, the screen goes dark, Pip hides.
    pub fn hide_seek_start(&mut self) {
        self.seek_count = 3.0;
        self.fade = true;
        self.snd(Sfx::Blip, 1.0, 0.7);
    }

    fn find_hiding_spot(&mut self, p: Vec2) -> bool {
        for _ in 0..400 {
            let x = self.range(-24.0, 64.0);
            let z = self.range(-23.5, 22.0);
            let d = Vec2::new(x - p.x, z - p.y).length();
            if !(16.0..=45.0).contains(&d) || on_road(x, z) || self.blocked(x, z, 0.7) || in_pond(x, z, -1.0) {
                continue;
            }
            if !self.blocked(x, z, 1.6) {
                continue; // close behind something
            }
            self.pip = Vec2::new(x, z);
            return true;
        }
        false
    }

    /// One fixed step of Pip (Kaigen `life_pip`). `p` = you (studs), `talking` = a dialog with him is open.
    pub fn step_pip(&mut self, s: &mut GameSave, p: Vec2, dt: f32, outside: bool, sprinting: bool, talking: bool) {
        self.pip_speed = 0.0;
        self.pip_pose = Pose::Normal;
        self.pip_cd -= dt;
        let dp = (p - self.pip).length();
        // hide and seek countdown (eyes closed)
        if self.seek_count > 0.0 {
            let before = self.seek_count;
            self.seek_count -= dt / 0.9;
            if before > 2.0 && self.seek_count <= 2.0 {
                if !self.find_hiding_spot(p) {
                    self.seek_count = 0.0;
                    self.fade = false;
                    self.toast("Pip found no hiding spot. Try again outside!");
                    return;
                }
                self.pip_mode = PipMode::Hiding;
            }
            if before as i32 != self.seek_count as i32 && self.seek_count > 0.0 {
                self.snd(Sfx::Blip, 1.0, 0.7);
            }
            if self.seek_count <= 0.0 {
                self.fade = false;
                self.seek_left = 60.0;
                self.hint_t = 9.0;
                self.snd(Sfx::Alert, 1.0, 0.7);
                self.toast("Ready or not! Find Pip within 60 seconds.");
            }
        }
        let face_you = |l: &mut Self, rate: f32| {
            let want = (p.x - l.pip.x).atan2(p.y - l.pip.y);
            l.pip_yaw += wrap(want - l.pip_yaw) * (dt * rate).min(1.0);
        };
        match self.pip_mode {
            PipMode::Hiding => {
                self.pip_pose = Pose::Crouch;
                if self.seek_count > 0.0 {
                    return;
                }
                self.seek_left -= dt;
                self.hint_t -= dt;
                if dp < 3.2 {
                    self.pip_found(s);
                } else if self.seek_left <= -3.0 {
                    self.pip_mode = PipMode::Wave;
                    s.seek_played += 1;
                    self.dirty = true;
                    self.toast("Time is up! Pip was hiding really well.");
                } else if self.hint_t <= 0.0 {
                    self.hint_t = 8.0;
                    let k = (self.rand() * 3.0) as usize % 3;
                    let hot = ["Hot! HOT!", "Uh oh, so close!", "Hot!!"];
                    let warm = ["Warm...", "Getting warmer!", "Warm!"];
                    let cold = ["Cold!", "Brrr, ice cold!", "Freezing!"];
                    self.say(if dp < 8.0 { hot[k] } else if dp < 16.0 { warm[k] } else { cold[k] });
                }
            }
            PipMode::Wave => {
                self.pip_pose = Pose::Wave;
                self.pip_yaw = (p.x - self.pip.x).atan2(p.y - self.pip.y);
                if dp > 24.0 || !outside {
                    self.pip_mode = PipMode::Walk;
                }
            }
            PipMode::Flee => {
                self.pip_t += dt;
                let mut away = self.pip - p;
                let al = away.length();
                away = if al > 0.01 { away / al } else { Vec2::X };
                let wob = (self.pip_t * 3.0).sin() * 0.5;
                let mut best = away;
                for off in [0.0f32, 0.5, -0.5, 1.0, -1.0, 1.5, -1.5] {
                    // dodge obstacles
                    let a = off + wob;
                    let dir = Vec2::new(away.x * a.cos() - away.y * a.sin(), away.x * a.sin() + away.y * a.cos());
                    let (tx, tz) = (self.pip.x + dir.x * 2.0, self.pip.y + dir.y * 2.0);
                    if !self.blocked(tx, tz, 0.6) && !in_pond(tx, tz, -0.8) && tx > -25.5 && tx < 65.5 && tz > -24.8 && tz < 22.6 {
                        best = dir;
                        break;
                    }
                }
                let to = self.pip + best * 3.0;
                self.pip_move(to, PIP_FLEE, dt);
                if dp < 2.3 && self.pip_t > 0.8 {
                    self.pip_mode = PipMode::Wave;
                    self.pip_cd = 25.0;
                    s.studs += 3;
                    s.tag_wins += 1;
                    self.dirty = true;
                    self.snd(Sfx::Fanfare, 1.0, 0.6);
                    self.toast("You tagged Pip! +3 studs");
                } else if self.pip_t > 10.0 {
                    self.pip_mode = PipMode::Walk;
                    self.pip_cd = 15.0;
                    self.say("Too slow! Hehe!");
                }
            }
            PipMode::Football => {
                self.pip_t += dt;
                let ball = Vec2::new(self.ball_p.x, self.ball_p.z);
                let db = (ball - self.pip).length();
                let pb = (p - ball).length();
                if self.pip_t > 8.0 || pb < 1.8 {
                    self.pip_mode = PipMode::Walk;
                    self.pip_cd = 4.0;
                    return;
                }
                self.pip_move(ball, PIP_BALL, dt);
                if db < 1.35 {
                    // shoot at the goal, or pass to you
                    let (aim, power) = if self.rand() < 0.35 && self.ball_p.x > 12.0 {
                        (Vec2::new(26.0 - ball.x, -6.0 - ball.y), 11.0)
                    } else {
                        let a = p - ball;
                        (a, (3.5 + a.length() * 0.55).min(10.0))
                    };
                    let aim = aim.normalize_or_zero();
                    self.ball_v = Vec3::new(aim.x * power, 0.0, aim.y * power);
                    self.snd(Sfx::Kick, 1.0, 0.6);
                    self.pip_mode = PipMode::Walk;
                    self.pip_cd = 5.0;
                }
            }
            PipMode::Walk => {
                let bs = Vec2::new(self.ball_v.x, self.ball_v.z).length();
                let ball = Vec2::new(self.ball_p.x, self.ball_p.z);
                let bp = (ball - self.pip).length();
                let pbp = (p - ball).length();
                if outside && sprinting && dp < 2.3 && self.pip_cd <= 0.0 && !talking && !self.frozen {
                    self.pip_mode = PipMode::Flee;
                    self.pip_t = 0.0;
                    self.snd(Sfx::Alert, 1.0, 0.6);
                    self.toast("Tag with Pip! Catch him again within 10 seconds (sprint with Shift)");
                    return;
                }
                if outside && self.pip_cd <= 0.0 && bs < 2.5 && self.goal_armed && bp < 16.0 && dp < 20.0 && pbp > 3.0 && !talking && !self.frozen {
                    self.pip_mode = PipMode::Football;
                    self.pip_t = 0.0;
                    self.say("My turn!");
                    return;
                }
                if dp < PIP_TALK || talking {
                    face_you(self, 4.0);
                    return;
                }
                if self.frozen {
                    return; // tests: he stands where he is (but still looks at you)
                }
                if self.pip_pause > 0.0 {
                    self.pip_pause -= dt;
                    return;
                }
                let w = PIP_WAYPOINTS[self.pip_wp];
                if self.pip_move(Vec2::new(w[0], w[1]), PIP_WALK, dt) < 0.1 {
                    self.pip_wp = (self.pip_wp + 1) % PIP_WAYPOINTS.len();
                    if self.rand() < 0.5 {
                        self.pip_pause = self.range(1.5, 3.5);
                    }
                }
            }
        }
        if matches!(self.pip_mode, PipMode::Hiding | PipMode::Wave) {
            self.pip_speed = 0.0;
        }
    }

    /// The HUD (Kaigen `life_draw`): the studs and golden studs, the hide and seek clock, the countdown's dark screen.
    pub fn hud(&self, s: &GameSave, narrow: bool) -> (String, String) {
        let seek = if self.seek_left > 0.0 && self.pip_mode == PipMode::Hiding && self.seek_count <= 0.0 {
            format!("<span class='bh-seek'>Find Pip: {} s</span>", self.seek_left.max(0.0).ceil() as u32)
        } else {
            String::new()
        };
        let studs = format!("<div class='bh-studs{}'><span>&#9679; {} studs &nbsp; &#9733; {}/15</span>{seek}</div>", if narrow { " narrow" } else { "" }, s.studs, s.golden_count());
        let fade = if self.fade { format!("<div class='bh-seekfade'>Pip is hiding\u{2026} {}</div>", self.seek_count.ceil().max(1.0) as u32) } else { String::new() };
        (studs, fade)
    }

    pub fn describe(&self, s: &GameSave) -> Value {
        let r = |v: f32| (v * 100.0).round() / 100.0;
        json!({
            "golden": s.golden_count(), "legendary": s.legendary, "studs": s.studs,
            "ducks": self.duck_p.iter().map(|d| [r(d.x), r(d.z)]).collect::<Vec<_>>(),
            "crumbs": self.crumbs.iter().filter(|c| c.active).count(), "duck_feeds": s.duck_feeds,
            "fish_jump": r(self.fish_jump), "fish_next": r(self.fish_next),
            "butterflies": s.butterflies, "flies": self.fly_at.iter().map(|f| [r(f.0.x), r(f.0.y), r(f.0.z)]).collect::<Vec<_>>(),
            "fireflies": s.fireflies, "fireflies_shown": self.ff_at.iter().filter(|f| f.0.y > -10.0 && f.1 > 0.0).count(),
            "ball": [r(self.ball_p.x), r(self.ball_p.z)], "ball_speed": r(Vec2::new(self.ball_v.x, self.ball_v.z).length()), "goals": s.goals, "goal_armed": self.goal_armed,
            "chest": {"live": self.chest_live, "open": self.chest_open, "x": r(self.chest_p.x), "z": r(self.chest_p.z), "next": r(self.chest_next)}, "chests": s.chests,
            "rain": r(self.rain), "rain_target": self.rain_target, "rain_level": r(self.rain_level), "rain_timer": r(self.rain_timer),
            "drops": self.drop_p.iter().filter(|d| d.y >= 0.0).count(), "wet": r(self.wet), "puddles": self.puddle_p.len(), "splashes": self.splashes,
            "night": r(self.night), "day": self.day, "sitting": self.sitting, "dancing": self.dancing, "bench_sits": s.bench_sits,
            "seek_count": r(self.seek_count), "seek_left": r(self.seek_left), "fade": self.fade, "hidden": self.pip_hidden(),
            "seek_played": s.seek_played, "seek_wins": s.seek_wins, "tag_wins": s.tag_wins, "pip_cd": r(self.pip_cd), "pip_pose": self.pip_pose,
        })
    }
}

// ====================================================================== the entities

/// The life meshes (made once, kept in `TownAssets`): one mesh per colour part.
pub struct LifeArt {
    golden: Vec<(MeshId, MaterialId)>,
    mother: Vec<(MeshId, MaterialId)>,
    duckling: Vec<(MeshId, MaterialId)>,
    crumb: Vec<(MeshId, MaterialId)>,
    fish: Vec<(MeshId, MaterialId)>,
    wings: Vec<Vec<(MeshId, MaterialId)>>,
    firefly: Vec<(MeshId, MaterialId)>,
    ball: Vec<(MeshId, MaterialId)>,
    chest: Vec<(MeshId, MaterialId)>,
    lid: Vec<(MeshId, MaterialId)>,
    drop: Vec<(MeshId, MaterialId)>,
    puddle: Vec<(MeshId, MaterialId)>,
    bulb: Vec<(MeshId, MaterialId)>,
    pub bulb_mat: MaterialId,
}

fn hex_of(part: &str) -> u32 {
    part.strip_prefix('c').and_then(|h| u32::from_str_radix(h, 16).ok()).unwrap_or(0xffffff)
}

/// A shape: the builder's colour parts (`c` + hex) as meshes with the town's LEGO materials.
fn shape(ctx: &mut Ctx, assets: &mut TownAssets, name: &str, f: impl FnOnce(&mut MeshBuilder)) -> Vec<(MeshId, MaterialId)> {
    let mut mb = MeshBuilder::new();
    f(&mut mb);
    mb.finish()
        .into_iter()
        .map(|(p, md)| {
            let mesh = ctx.renderer.add_mesh(&format!("life {name}:{p}"), &md);
            (mesh, assets.material(ctx, hex_of(&p)))
        })
        .collect()
}

fn col(mb: &mut MeshBuilder, hex: u32) -> &mut MeshBuilder {
    mb.part(&format!("c{hex:06x}"))
}

/// A box w x h x d studs, its bottom centre at (x, y, z) studs (Kaigen `town_box`).
fn bx(mb: &mut MeshBuilder, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32) {
    col(mb, hex).block(town::m(x, y, z), town::m(w, h, d));
}

fn sphere(mb: &mut MeshBuilder, hex: u32, t: Mat4, seg: u32, rings: u32) {
    let mut md = MeshData::sphere(1.0, seg, rings);
    engine::geom::smooth_normals(&mut md, 80.0);
    col(mb, hex).with(t, |b| {
        b.add(&md);
    });
}

/// The three.js duck (Kaigen `town_duck`): facing +x, feet at 0, size s.
fn duck(mb: &mut MeshBuilder, s: f32, hex: u32) {
    sphere(mb, hex, Mat4::from_scale_rotation_translation(town::m(0.75 * s, 0.45 * s, 0.5 * s), Quat::IDENTITY, town::m(0.0, 0.45 * s, 0.0)), 12, 8);
    let head = Vec3::new(0.5 * s, 1.0 * s, 0.0);
    sphere(mb, hex, Mat4::from_scale_rotation_translation(town::m(0.33 * s, 0.33 * s, 0.33 * s), Quat::IDENTITY, town::m(head.x, head.y, head.z)), 10, 6);
    bx(mb, town::C_ORANGE, 0.4 * s, 0.12 * s, 0.25 * s, head.x + 0.35 * s, head.y - 0.11 * s, 0.0);
    let tail = (((hex >> 16) & 255) as f32 * 0.9) as u32 * 65536 + (((hex >> 8) & 255) as f32 * 0.9) as u32 * 256 + ((hex & 255) as f32 * 0.9) as u32;
    bx(mb, tail, 0.3 * s, 0.12 * s, 0.3 * s, -0.7 * s, 0.64 * s, 0.0);
}

impl LifeArt {
    pub fn new(ctx: &mut Ctx, a: &mut TownAssets) -> Self {
        use town::*;
        let gold = ctx.renderer.add_material(Material { name: "golden stud".into(), emissive: Vec3::new(1.0, 0.62, 0.1) * 0.55, ..Material::color(0xffcc33).rough(0.25) });
        let glow = ctx.renderer.add_material(Material { name: "firefly".into(), emissive: Vec3::new(2.2, 2.0, 0.9), ..Material::color(0xfff07a) });
        let bulb_mat = ctx.renderer.add_material(Material { name: "lamp bulb".into(), emissive: Vec3::new(1.0, 0.86, 0.5) * 0.1, ..Material::color(0xfff5c0) });
        let mut golden = shape(ctx, a, "golden", |mb| {
            // standing on its edge, facing z
            let up = Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2);
            col(mb, 0xffcc33).with(up * Mat4::from_translation(Vec3::new(0.0, -0.16 * STUD, 0.0)), |b| {
                b.cylinder(0.55 * STUD, 0.32 * STUD, 20);
            });
            col(mb, 0xffcc33).with(up * Mat4::from_translation(Vec3::new(0.0, 0.1 * STUD, 0.0)), |b| {
                b.cylinder(0.42 * STUD, 0.14 * STUD, 20);
            });
        });
        for p in &mut golden {
            p.1 = gold;
        }
        let mother = shape(ctx, a, "duck", |mb| duck(mb, 0.95 * 1.3, C_WHITE));
        let duckling = shape(ctx, a, "duckling", |mb| duck(mb, 0.52 * 1.3, C_YELLOW));
        let crumb = shape(ctx, a, "crumb", |mb| bx(mb, 0xd9a35c, 0.3, 0.2, 0.3, 0.0, 0.0, 0.0));
        let fish = shape(ctx, a, "fish", |mb| {
            bx(mb, C_ORANGE, 0.8, 0.3, 0.35, 0.0, -0.15, 0.0);
            bx(mb, 0xf06d00, 0.25, 0.35, 0.08, -0.5, -0.17, 0.0);
        });
        let wings = [C_PINK, C_YELLOW, 0x5aa9ff, C_WHITE, C_ORANGE]
            .iter()
            .map(|&c| {
                shape(ctx, a, "wing", |mb| {
                    bx(mb, c, 0.55, 0.04, 0.4, 0.3, 0.0, 0.0);
                    bx(mb, c, 0.55, 0.04, 0.4, -0.3, 0.0, 0.0);
                    bx(mb, C_BLACK, 0.1, 0.08, 0.5, 0.0, 0.0, 0.0);
                })
            })
            .collect();
        let mut firefly = shape(ctx, a, "firefly", |mb| sphere(mb, 0xfff07a, Mat4::from_scale(Vec3::splat(0.12 * STUD)), 6, 4));
        for p in &mut firefly {
            p.1 = glow;
        }
        let ball = shape(ctx, a, "ball", |mb| {
            sphere(mb, C_WHITE, Mat4::from_scale(Vec3::splat(0.6 * STUD)), 14, 10);
            // the red band: a ring round the middle
            let ring: Vec<Vec2> = (0..=8).map(|k| {
                let t = k as f32 / 8.0 * std::f32::consts::TAU;
                Vec2::new(0.6 * STUD + t.cos() * 0.06 * STUD, t.sin() * 0.06 * STUD)
            }).collect();
            col(mb, C_RED).lathe(&ring, 18);
        });
        let chest = shape(ctx, a, "chest", |mb| {
            bx(mb, C_RED_BROWN, 1.6, 0.9, 1.1, 0.0, 0.0, 0.0);
            bx(mb, C_YELLOW, 1.65, 0.12, 1.15, 0.0, 0.75, 0.0);
            bx(mb, C_YELLOW, 0.25, 0.3, 0.06, 0.0, 0.45, 0.57);
        });
        // hinged at its back edge
        let lid = shape(ctx, a, "chest lid", |mb| bx(mb, C_BROWN, 1.6, 0.45, 1.1, 0.0, 0.0, 0.55));
        let drop = shape(ctx, a, "rain drop", |mb| bx(mb, 0xb8d4ff, 0.06, 1.1, 0.06, 0.0, 0.0, 0.0));
        let puddle = shape(ctx, a, "puddle", |mb| {
            col(mb, 0x5c7f9c).cylinder(STUD, 0.03 * STUD, 16);
        });
        let mut bulb = shape(ctx, a, "bulb", |mb| bx(mb, 0xfff5c0, 0.7, 0.8, 0.7, 0.0, 0.0, 0.0));
        for p in &mut bulb {
            p.1 = bulb_mat;
        }
        Self { golden, mother, duckling, crumb, fish, wings, firefly, ball, chest, lid, drop, puddle, bulb, bulb_mat }
    }
}

fn spawn(ctx: &mut Ctx, name: &str, parts: &[(MeshId, MaterialId)], layer: u8, shadow: bool) -> Entity {
    let root = ctx.world.spawn((Transform::from_xyz(0.0, -50.0, 0.0), Name(name.to_string())));
    for &(mesh, mat) in parts {
        let mut mr = MeshRenderer::new(mesh, mat);
        mr.cast_shadows = shadow;
        ctx.world.spawn((Transform::default(), Parent(root), mr, Name(name.to_string())));
    }
    ctx.set_layer(root, layer);
    root
}

/// The life's entities (spawned with the play screen).
#[derive(Default)]
pub struct LifeView {
    golden: Vec<Entity>,
    ducks: Vec<Entity>,
    crumbs: Vec<Entity>,
    fish: Option<Entity>,
    flies: Vec<Entity>,
    fireflies: Vec<Entity>,
    ball: Option<Entity>,
    chest: Option<(Entity, Entity)>,
    drops: Vec<Entity>,
    puddles: Vec<Entity>,
    bulbs: Vec<Entity>,
    bulb_mat: Option<MaterialId>,
    bulb_glow: f32,
    /// the rain loop (on `Bus::Ambience`, its volume is the bus's)
    pub rain_voice: Option<VoiceId>,
    /// the fish's splash clip
    pub fish_clip: Option<SoundId>,
}

fn place(ctx: &mut Ctx, e: Entity, studs: Vec3, rot: Quat, scale: Vec3) {
    if let Ok(mut t) = ctx.world.get::<&mut Transform>(e) {
        *t = Transform { translation: town::m(studs.x, studs.y, studs.z), rotation: rot, scale };
    }
}

impl LifeView {
    pub fn spawn(ctx: &mut Ctx, assets: &mut TownAssets, life: &Life) -> Self {
        if assets.life.is_none() {
            assets.life = Some(LifeArt::new(ctx, assets));
        }
        let art = assets.life.take().unwrap();
        let v = town::rooms::VILLAGE_LAYER;
        let mut me = Self { bulb_mat: Some(art.bulb_mat), bulb_glow: -1.0, ..Default::default() };
        for g in GOLDEN {
            let layer = rooms::area_at(g.1, g.2).map(|r| rooms::layer_of(Some(r))).unwrap_or(v);
            me.golden.push(spawn(ctx, g.0, &art.golden, layer, true));
        }
        for i in 0..DUCKS {
            me.ducks.push(spawn(ctx, if i == 0 { "duck" } else { "duckling" }, if i == 0 { &art.mother } else { &art.duckling }, v, true));
        }
        for _ in 0..CRUMBS {
            me.crumbs.push(spawn(ctx, "crumb", &art.crumb, v, false));
        }
        me.fish = Some(spawn(ctx, "fish", &art.fish, v, false));
        for i in 0..FLIES {
            me.flies.push(spawn(ctx, "butterfly", &art.wings[i % 5], v, false));
        }
        for _ in 0..FIREFLIES {
            me.fireflies.push(spawn(ctx, "firefly", &art.firefly, v, false));
        }
        me.ball = Some(spawn(ctx, "football", &art.ball, v, true));
        me.chest = Some((spawn(ctx, "treasure chest", &art.chest, v, true), spawn(ctx, "chest lid", &art.lid, v, true)));
        // (the rain follows you north too: layer 0 is drawn by every camera that draws you)
        for _ in 0..RAIN_DROPS {
            me.drops.push(spawn(ctx, "rain drop", &art.drop, 0, false));
        }
        for _ in 0..life.puddle_p.len() {
            me.puddles.push(spawn(ctx, "puddle", &art.puddle, v, false));
        }
        for l in town::LAMPS {
            let b = spawn(ctx, "lamp bulb", &art.bulb, v, false);
            place(ctx, b, Vec3::new(l[0], 5.2, l[1]), Quat::IDENTITY, Vec3::ONE);
            me.bulbs.push(b);
        }
        assets.life = Some(art);
        me.fish_clip = Some(ctx.load_sound("assets/music/fish-splash.mp3"));
        me
    }

    /// Places everything from the state (every fixed step).
    pub fn sync(&self, ctx: &mut Ctx, l: &Life, s: &GameSave) {
        let t = l.time;
        let hidden = Vec3::new(0.0, -50.0, 0.0);
        for (k, (&e, g)) in self.golden.iter().zip(GOLDEN.iter()).enumerate() {
            if (s.golden_mask >> k) & 1 == 1 {
                place(ctx, e, hidden, Quat::IDENTITY, Vec3::ONE);
            } else {
                place(ctx, e, Vec3::new(g.1, golden_base(g.1) + (t * 3.0 + g.1).sin() * 0.2, g.2), Quat::from_rotation_y(t * 2.2), Vec3::ONE);
            }
        }
        for (i, &e) in self.ducks.iter().enumerate() {
            let d = l.duck_p[i];
            let y = 0.2 + (t * 2.4 + i as f32 * 1.3).sin() * 0.03;
            let q = Quat::from_rotation_y(l.duck_yaw[i]) * Quat::from_rotation_x((t * 1.8 + i as f32).sin() * 0.05);
            place(ctx, e, Vec3::new(d.x, y, d.z), q, Vec3::ONE);
        }
        for (c, &e) in l.crumbs.iter().zip(self.crumbs.iter()) {
            place(ctx, e, if c.active && c.t >= 0.0 { c.at } else { hidden }, Quat::from_rotation_y(c.spin), Vec3::ONE);
        }
        if let Some(f) = self.fish {
            place(ctx, f, l.fish_at, l.fish_rot, Vec3::ONE);
        }
        for (&e, f) in self.flies.iter().zip(l.fly_at.iter()) {
            place(ctx, e, f.0, Quat::from_rotation_y(f.1), Vec3::new(f.2, 1.0, 1.0));
        }
        for (&e, f) in self.fireflies.iter().zip(l.ff_at.iter()) {
            place(ctx, e, if f.1 > 0.0 { f.0 } else { hidden }, Quat::IDENTITY, Vec3::splat(f.1.max(0.01)));
        }
        if let Some(b) = self.ball {
            let p = l.ball_p;
            // it rolls: turned about the axis across its motion
            place(ctx, b, p, Quat::from_euler(EulerRot::XYZ, p.z * 1.6, 0.0, -p.x * 1.6), Vec3::ONE);
        }
        if let Some((c, lid)) = self.chest {
            if l.chest_live {
                let (sc, jiggle, open) = l.chest_draw;
                let p = l.chest_p;
                place(ctx, c, p, Quat::from_rotation_y(jiggle), Vec3::splat(sc));
                let hinge = Vec3::new(p.x, p.y + 0.9 * sc, p.z - 0.55 * sc);
                place(ctx, lid, hinge, Quat::from_euler(EulerRot::YXZ, jiggle, open, 0.0), Vec3::splat(sc));
            } else {
                place(ctx, c, hidden, Quat::IDENTITY, Vec3::ONE);
                place(ctx, lid, hidden, Quat::IDENTITY, Vec3::ONE);
            }
        }
        for (&e, d) in self.drops.iter().zip(l.drop_p.iter()) {
            place(ctx, e, if d.y >= 0.0 { *d } else { hidden }, Quat::IDENTITY, Vec3::ONE);
        }
        for (i, (&e, pd)) in self.puddles.iter().zip(l.puddle_p.iter()).enumerate() {
            let r = pd.y * (0.35 + 0.65 * l.wet);
            let show = l.wet > 0.02;
            place(ctx, e, Vec3::new(pd.x, if show { 0.62 } else { -50.0 }, pd.z), Quat::from_rotation_y(i as f32), Vec3::new(r, 1.0, r * 0.7));
        }
    }

    /// Every frame: the lamps glow at night, the rain's volume (Kaigen `life_lamps`, `life_weather`).
    pub fn frame(&mut self, ctx: &mut Ctx, l: &Life) {
        let glow = 0.1 + l.night * 2.0;
        if (glow - self.bulb_glow).abs() > 0.01 {
            self.bulb_glow = glow;
            if let Some(m) = self.bulb_mat.and_then(|m| ctx.renderer.material_mut(m)) {
                m.emissive = Vec3::new(1.0, 0.86, 0.5) * glow;
            }
        }
        if l.rain_level > 0.01 && self.rain_voice.is_none() {
            self.rain_voice = ctx.audio.play_named(Sfx::Rain.name(), Play::new().looped().volume(1.0).bus(Bus::Ambience));
        }
        ctx.audio.set_bus_volume(Bus::Ambience, l.rain_level * 0.35);
    }

    /// Leaving the play screen: the rain loop stops.
    pub fn stop(&mut self, ctx: &mut Ctx) {
        if let Some(v) = self.rain_voice.take() {
            ctx.audio.stop(v);
        }
    }

    /// Debug picker: the moving things as spheres (name, centre studs, radius studs).
    pub fn pick_spheres(l: &Life, s: &GameSave) -> Vec<(&'static str, Vec3, f32)> {
        let mut v = vec![];
        for (i, d) in l.duck_p.iter().enumerate() {
            v.push((if i == 0 { "Duck (mother)" } else { "Duckling" }, *d, 1.0));
        }
        v.push(("Football", l.ball_p, 0.9));
        for (k, g) in GOLDEN.iter().enumerate() {
            if (s.golden_mask >> k) & 1 == 0 {
                v.push(("Golden stud", Vec3::new(g.1, golden_base(g.1), g.2), 0.9));
            }
        }
        if l.chest_live {
            v.push(("Treasure chest", l.chest_p, 1.5));
        }
        v
    }
}
