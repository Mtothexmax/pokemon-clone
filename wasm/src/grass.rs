//! The tall grass moves: every tuft's blades are an instance of one mesh (`town::tuft_mesh`) turned about the
//! plate it stands on. Whoever walks through (you, the villagers, your Voxelmon, a wild one) pushes the tufts
//! near them away; a damped spring per tuft brings them back with a little wobble (in `fixed_update`, so a
//! stepped test is repeatable), and a slow wind sways the tufts in view (per frame, travelling gusts).
//!
//! Cheap on purpose: the engine has no per-instance vertex shader hook for games, so the bend is a rotation
//! of each tuft's transform (one instance record each). Only tufts near a pusher run the spring, only tufts
//! in view sway; the rest keep their last pose and cost nothing (static instance records, static shadows).

use crate::town::{TuftSpec, GRASSES, STUD};
use engine::prelude::*;
use serde_json::{json, Value};

/// Lean (radians) a pusher right at a tuft gives it; the most a tuft leans.
const PUSH_BEND: f32 = 0.95;
const MAX_BEND: f32 = 1.05;
/// Reach of a push beyond the pusher's radius (metres).
const REACH: f32 = 0.55;
/// Spring stiffness while pushed / recovering (1/s^2) and the damping ratio (a little overshoot).
const K_PUSH: f32 = 160.0;
const K_BACK: f32 = 55.0;
const ZETA: f32 = 0.32;
/// Wind: sway amplitude (radians), direction (XZ), gust speed.
const WIND_AMP: f32 = 0.075;
const WIND_DIR: Vec2 = Vec2::new(0.83, 0.55);
/// Tufts within this rectangle around you sway (studs: x either side, north, south).
const VIEW: (f32, f32, f32) = (36.0, 32.0, 22.0);

struct Tuft {
    e: Entity,
    /// pivot (metres)
    p: Vec3,
    yaw: Quat,
    scale: f32,
    /// lean: the direction the tip goes (XZ) times the angle (radians), and its rate
    bend: Vec2,
    vel: Vec2,
    phase: f32,
    awake: bool,
    /// in view last frame (swaying)
    swaying: bool,
}

#[derive(Default)]
pub struct Grass {
    tufts: Vec<Tuft>,
    /// per patch (`GRASSES`): the tufts' index range
    patches: Vec<(usize, usize)>,
    /// whoever pushes this step: feet (metres) and radius (metres)
    pushers: Vec<(Vec3, f32)>,
    t: f32,
    pub wind: f32,
    /// tufts with a running spring / in view, for the debug state
    awake: usize,
    swaying: usize,
    pub writes: usize,
}

impl Grass {
    pub fn new(list: Vec<(Entity, TuftSpec)>) -> Self {
        let mut tufts: Vec<Tuft> = list
            .into_iter()
            .map(|(e, s)| Tuft {
                e,
                p: s.pos,
                yaw: Quat::from_rotation_y(-s.yaw),
                scale: s.scale,
                bend: Vec2::ZERO,
                vel: Vec2::ZERO,
                phase: s.yaw * 1.7,
                awake: false,
                swaying: false,
            })
            .collect();
        // sorted by patch so a patch with nobody near is skipped in one test
        let patch = |p: Vec3| GRASSES.iter().position(|g| p.x / STUD > g.x0 - 1.0 && p.x / STUD < g.x1 + 1.0 && p.z / STUD > g.z0 - 1.0 && p.z / STUD < g.z1 + 1.0).unwrap_or(GRASSES.len());
        tufts.sort_by_key(|t| patch(t.p));
        let mut patches = vec![];
        for g in 0..=GRASSES.len() {
            let a = tufts.iter().position(|t| patch(t.p) == g).unwrap_or(tufts.len());
            let n = tufts.iter().filter(|t| patch(t.p) == g).count();
            patches.push((a, a + n));
        }
        Self { tufts, patches, wind: 1.0, ..Default::default() }
    }

    /// Who walks this step (feet in metres, radius in metres); call before `step`.
    pub fn set_pushers(&mut self, p: Vec<(Vec3, f32)>) {
        self.pushers = p;
    }

    /// One fixed step of the springs.
    pub fn step(&mut self, dt: f32) {
        let mut awake = 0;
        for (pi, &(a, b)) in self.patches.iter().enumerate() {
            let near: Vec<(Vec2, f32)> = self
                .pushers
                .iter()
                .filter(|(p, r)| {
                    let g = GRASSES.get(pi).copied().unwrap_or(crate::town::Rect::new(-1e9, 1e9, -1e9, 1e9));
                    let m = (r + REACH) / STUD + 1.5;
                    p.x / STUD > g.x0 - m && p.x / STUD < g.x1 + m && p.z / STUD > g.z0 - m && p.z / STUD < g.z1 + m && p.y < 1.2
                })
                .map(|(p, r)| (Vec2::new(p.x, p.z), *r))
                .collect();
            for t in &mut self.tufts[a..b] {
                let base = Vec2::new(t.p.x, t.p.z);
                let mut target = Vec2::ZERO;
                for (q, r) in &near {
                    let d = base - *q;
                    let reach = r + REACH;
                    let l = d.length();
                    if l < reach {
                        // away from the pusher; standing right on it: the tuft's own way
                        let dir = if l > 1e-3 { d / l } else { Vec2::new(t.phase.cos(), t.phase.sin()) };
                        let k = 1.0 - (l / reach);
                        target += dir * PUSH_BEND * (k * (2.0 - k)).min(1.0);
                    }
                }
                if target.length() > MAX_BEND {
                    target = target.normalize() * MAX_BEND;
                }
                let pushed = target != Vec2::ZERO;
                if !pushed && !t.awake {
                    continue;
                }
                let k = if pushed { K_PUSH } else { K_BACK };
                let c = 2.0 * ZETA * k.sqrt();
                t.vel += (k * (target - t.bend) - c * t.vel) * dt;
                t.bend += t.vel * dt;
                t.awake = pushed || t.bend.length() > 0.002 || t.vel.length() > 0.01;
                if !t.awake {
                    t.bend = Vec2::ZERO;
                    t.vel = Vec2::ZERO;
                }
                awake += 1;
            }
        }
        self.awake = awake;
    }

    /// Per frame: the tufts in view around `me` (feet, metres) take their bend plus the wind.
    pub fn update(&mut self, ctx: &mut Ctx, me: Option<Vec3>, dt: f32) {
        self.t += dt;
        let (mut sway, mut writes) = (0, 0);
        let Some(me) = me else { return };
        let (mx, mz) = (me.x / STUD, me.z / STUD);
        let wdir = WIND_DIR.normalize();
        for t in &mut self.tufts {
            let (x, z) = (t.p.x / STUD, t.p.z / STUD);
            let in_view = (x - mx).abs() < VIEW.0 && z > mz - VIEW.1 && z < mz + VIEW.2;
            if !in_view && !t.swaying && !t.awake {
                continue;
            }
            let mut lean = t.bend;
            if in_view {
                // gusts roll along the wind: a slow wave across the field plus each tuft's own flutter
                let along = t.p.x * wdir.x + t.p.z * wdir.y;
                let gust = (self.t * 1.3 - along * 0.55).sin() * 0.65 + 0.35;
                let flutter = (self.t * 3.1 + t.phase).sin() * 0.35;
                lean += wdir * WIND_AMP * self.wind * (gust + flutter + 0.4);
                sway += 1;
            }
            t.swaying = in_view;
            let a = lean.length();
            let tilt = if a > 1e-4 { Quat::from_axis_angle(Vec3::new(lean.y, 0.0, -lean.x) / a, a) } else { Quat::IDENTITY };
            if let Ok(mut tr) = ctx.world.get::<&mut Transform>(t.e) {
                tr.rotation = tilt * t.yaw;
                tr.translation = t.p;
                tr.scale = Vec3::splat(t.scale);
                writes += 1;
            }
        }
        self.swaying = sway;
        self.writes = writes;
    }

    /// Debug: the bends near (x, z) studs.
    pub fn describe(&self, at: Option<Vec3>) -> Value {
        let r = |v: f32| (v * 1000.0).round() / 1000.0;
        let max = self.tufts.iter().map(|t| t.bend.length()).fold(0.0f32, f32::max);
        let bent = self.tufts.iter().filter(|t| t.bend.length() > 0.2).count();
        let near = at.map(|p| {
            let mut v: Vec<(f32, f32)> = self.tufts.iter().map(|t| ((Vec2::new(t.p.x, t.p.z) - Vec2::new(p.x, p.z)).length(), t.bend.length())).filter(|(d, _)| *d < 1.0).collect();
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            v.into_iter().take(6).map(|(d, b)| json!([r(d), r(b)])).collect::<Vec<_>>()
        });
        json!({"tufts": self.tufts.len(), "patches": self.patches.iter().map(|p| p.1 - p.0).collect::<Vec<_>>(), "awake": self.awake, "swaying": self.swaying,
               "writes": self.writes, "bent": bent, "max_bend": r(max), "near": near, "pushers": self.pushers.len(), "wind": self.wind})
    }
}
