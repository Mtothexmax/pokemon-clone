//! The tall grass moves: every tuft's blades are an instance of one mesh (`town::tuft_mesh`, origin at its foot)
//! with a `Material::bend` material (`TownAssets::tuft_material`). The engine's vertex shader leans them away from
//! the *pushers* set every frame (`ctx.renderer.set_pushers`): you, the villagers outside, your Voxelmon and a wild
//! one, the nearest `MAX_PUSHERS - WIND_PUSHERS` of them. No tuft transform is ever written again: no per-frame
//! instance upload, the tufts stay static shadow casters (their shadows stay upright, as the engine documents).
//!
//! Wind: the engine's bend path has no wind of its own (only its GPU grass sways), so two far *wind pushers*
//! stand upwind of you with radii that reach the field; moving them a little lets the whole field lean and breathe
//! with the gusts (a gradient across the view, stronger upwind) at no GPU or upload cost.

use crate::town::{TuftSpec, GRASSES, STUD};
use engine::prelude::*;
use serde_json::{json, Value};

/// `Material::bend` of the tufts: horizontal lean per metre of height at a full push (capped by the engine at 0.85).
pub const TUFT_BEND: f32 = 1.1;
/// Reach of a push beyond the pusher's radius (metres).
const REACH: f32 = 0.55;
/// Wind: direction (XZ, towards which the tufts lean), the wind pushers' reach (metres).
const WIND_DIR: Vec2 = Vec2::new(0.83, 0.55);
const WIND_R: f32 = 300.0;
const WIND_PUSHERS: usize = 2;
const MAX_PUSHERS: usize = 16;
/// Your wake: where you were (a point every `WAKE_EVERY` s, `WAKE_N` of them) keeps pushing with a radius that shrinks
/// to nothing over `WAKE_LIFE` s, so the tufts behind you rise again instead of snapping up (the old spring's look).
const WAKE_N: usize = 3;
const WAKE_EVERY: f32 = 0.12;
const WAKE_LIFE: f32 = 0.45;

struct Tuft {
    /// foot (metres)
    p: Vec3,
}

#[derive(Default)]
pub struct Grass {
    tufts: Vec<Tuft>,
    /// per patch (`GRASSES`): the tuft count
    patches: Vec<usize>,
    /// whoever pushes (fixed step): feet (metres) and radius (metres, the reach included)
    pushers: Vec<(Vec3, f32)>,
    /// what went to the renderer last frame (walkers + wind)
    sent: Vec<(Vec3, f32)>,
    /// your wake: feet, radius, age (s)
    wake: Vec<(Vec3, f32, f32)>,
    wake_t: f32,
    t: f32,
    pub wind: f32,
}

impl Grass {
    pub fn new(list: Vec<(Entity, TuftSpec)>) -> Self {
        let tufts: Vec<Tuft> = list.into_iter().map(|(_, s)| Tuft { p: s.pos }).collect();
        let patch = |p: Vec3| GRASSES.iter().position(|g| p.x / STUD > g.x0 - 1.0 && p.x / STUD < g.x1 + 1.0 && p.z / STUD > g.z0 - 1.0 && p.z / STUD < g.z1 + 1.0).unwrap_or(GRASSES.len());
        let patches = (0..=GRASSES.len()).map(|g| tufts.iter().filter(|t| patch(t.p) == g).count()).collect();
        Self { tufts, patches, wind: 1.0, ..Default::default() }
    }

    /// Who walks this step (feet in metres, radius in metres; you first). Feet high above the grass (a jump) push
    /// nothing. Fixed step: also ages your wake.
    pub fn set_pushers(&mut self, p: Vec<(Vec3, f32)>, dt: f32) {
        self.pushers = p.into_iter().filter(|(q, _)| q.y < 1.2).map(|(q, r)| (q, r + REACH)).collect();
        for w in &mut self.wake {
            w.2 += dt;
        }
        self.wake.retain(|w| w.2 < WAKE_LIFE);
        self.wake_t += dt;
        if let Some(&(me, r)) = self.pushers.first() {
            if self.wake_t >= WAKE_EVERY && self.wake.last().is_none_or(|w| (w.0 - me).length() > 0.15) {
                self.wake_t = 0.0;
                self.wake.push((me, r, 0.0));
                if self.wake.len() > WAKE_N {
                    self.wake.remove(0);
                }
            }
        }
    }

    /// The walkers plus your wake (radius shrinking with age).
    fn walkers(&self) -> Vec<(Vec3, f32)> {
        let mut v = self.pushers.clone();
        v.extend(self.wake.iter().map(|(p, r, age)| (*p, r * (1.0 - age / WAKE_LIFE).max(0.0))).filter(|w| w.1 > 0.05));
        v
    }

    /// Per frame: hands the nearest pushers and the wind to the renderer (`me` = your feet; None indoors: none).
    pub fn update(&mut self, ctx: &mut Ctx, me: Option<Vec3>, dt: f32) {
        self.t += dt;
        self.sent.clear();
        if let Some(me) = me {
            let mut walkers = self.walkers();
            walkers.sort_by(|a, b| (a.0 - me).length_squared().total_cmp(&(b.0 - me).length_squared()));
            self.sent.extend(walkers.into_iter().take(MAX_PUSHERS - WIND_PUSHERS));
            if self.wind > 0.0 {
                self.sent.extend(self.wind_pushers(me));
            }
        }
        ctx.renderer.set_pushers(&self.sent);
    }

    /// Two far pushers upwind of `me`: the field lies at a fraction `x` of their reach from its rim, which gives a
    /// lean of about `TUFT_BEND * smoothstep(x)`; a slow gust and a quicker flutter (from a turned direction) move them.
    fn wind_pushers(&self, me: Vec3) -> [(Vec3, f32); 2] {
        let w = self.wind.min(2.0);
        let dir = WIND_DIR.normalize();
        let side = Vec2::new(dir.x * 0.77 - dir.y * 0.64, dir.x * 0.64 + dir.y * 0.77);
        let gust = 0.13 + 0.05 * (self.t * 1.3).sin() * w;
        let flutter = 0.06 + 0.03 * (self.t * 3.1).sin();
        let at = |d: Vec2, x: f32| {
            let x = (x * w.sqrt()).clamp(0.0, 0.5);
            let q = Vec2::new(me.x, me.z) - d * WIND_R * (1.0 - x);
            (Vec3::new(q.x, me.y, q.y), WIND_R)
        };
        [at(dir, gust), at(side, flutter)]
    }

    /// The engine's lean (`push_offset` in common.wgsl) of a tuft's point 1 m up, as an angle (radians), from `list`.
    fn lean(p: Vec3, list: &[(Vec3, f32)]) -> f32 {
        let mut off = Vec2::ZERO;
        for (q, r) in list {
            let d = Vec2::new(p.x - q.x, p.z - q.z);
            let dist = d.length();
            if dist < *r && (p.y - q.y).abs() < r + 1.0 {
                let x = 1.0 - dist / r;
                let f = x * x * (3.0 - 2.0 * x);
                off += if dist > 1e-4 { d / dist } else { Vec2::X } * f;
            }
        }
        let l = off.length();
        if l < 1e-4 {
            return 0.0;
        }
        (l.min(1.0) * TUFT_BEND).min(0.85).asin()
    }

    /// Debug: counts, the pushers, the lean of the tufts near `at` (walkers only, no wind).
    pub fn describe(&self, at: Option<Vec3>) -> Value {
        let r = |v: f32| (v * 1000.0).round() / 1000.0;
        let walkers = self.walkers();
        let leans: Vec<f32> = self.tufts.iter().map(|t| Self::lean(t.p, &walkers)).collect();
        let max = leans.iter().copied().fold(0.0f32, f32::max);
        let bent = leans.iter().filter(|l| **l > 0.2).count();
        let near = at.map(|p| {
            let mut v: Vec<(f32, f32)> = self.tufts.iter().zip(&leans).map(|(t, l)| ((Vec2::new(t.p.x, t.p.z) - Vec2::new(p.x, p.z)).length(), *l)).filter(|(d, _)| *d < 1.0).collect();
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            v.into_iter().take(6).map(|(d, b)| json!([r(d), r(b)])).collect::<Vec<_>>()
        });
        json!({"tufts": self.tufts.len(), "patches": self.patches, "bent": bent, "max_bend": r(max), "near": near, "pushers": self.pushers.len(), "wake": self.wake.len(),
               "sent": self.sent.len(), "wind_pushers": self.sent.len() - self.sent.len().min(walkers.len()).min(MAX_PUSHERS - WIND_PUSHERS), "wind": self.wind, "bend": TUFT_BEND, "writes": 0})
    }
}
