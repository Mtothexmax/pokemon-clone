//! Swipe walking for phones (Kaigen `st_swipe.c`, the three.js "Swipe" controls, the default on touch): swipe once
//! anywhere and the figure keeps walking that way, around obstacles (it picks one side and keeps it, towards a door
//! when one is near); a tap stops it, and so does a wall it cannot get past for 1.5 s. Two fingers still pinch to zoom;
//! the Use button works as before. Swipes snap to the 4 screen directions (Kaigen f058de7).
//!
//! The engine's joystick is switched off in this mode (`set_stick_enabled(false)`): every finger is a look finger, the
//! swipe is read from `ctx.input.touch.touches` (start and current position) every frame, and swipes that start and
//! end between two frames from `touch.swipes`. The menu's CONTROLS row picks Swipe or Stick (`GameSave::touch_mode`).

use crate::town::{rooms, STUD};
use engine::prelude::*;

/// Finger travel that makes a swipe (less is a tap), CSS px.
const SWIPE_MIN_PX: f32 = 22.0;

#[derive(Default)]
pub struct Swipe {
    pub on: bool,
    /// world x/z, unit
    pub dir: Vec2,
    best: f32,
    gained: f32,
    /// 0 not decided, +1 / -1: the side it walks around an obstacle
    side: i32,
    area: Option<rooms::RoomId>,
    tracking: Option<i32>,
    start: Vec2,
    swiped: bool,
    /// touch ids seen last frame
    seen: Vec<i32>,
    /// swipes started (tests)
    pub started: u32,
    /// the last step went around something (tests)
    pub dodging: bool,
}

/// A screen direction (x right, y down) to a world direction for the fixed camera (yaw 0: up is north), snapped to
/// one of 4 ways on screen.
pub fn world_dir(d: Vec2) -> Vec2 {
    let step = std::f32::consts::FRAC_PI_2;
    let ang = (d.y.atan2(d.x) / step + 0.5).floor() * step;
    Vec2::new(ang.cos(), ang.sin()).normalize_or_zero()
}

impl Swipe {
    pub fn stop(&mut self) {
        self.on = false;
    }

    fn start_walk(&mut self, dir: Vec2, pos: Vec3, now: f32, area: Option<rooms::RoomId>) {
        self.on = true;
        self.dir = dir;
        self.side = 0;
        self.best = pos.x * dir.x + pos.z * dir.y;
        self.gained = now;
        self.area = area;
        self.started += 1;
    }

    /// Every frame: the newest finger becomes the swipe; its way follows the finger; a tap stops (Kaigen `swipe_touch`).
    pub fn touch(&mut self, ctx: &Ctx, pos: Vec3, now: f32, area: Option<rooms::RoomId>) {
        let g = &ctx.input.touch;
        let ids: Vec<i32> = g.touches.iter().map(|t| t.id).collect();
        let pinching = g.touches.len() >= 2;
        for t in &g.touches {
            if !self.seen.contains(&t.id) {
                self.tracking = Some(t.id);
                self.start = t.start;
                self.swiped = false;
            }
        }
        if pinching {
            self.tracking = None; // two fingers zoom
        }
        if let Some(id) = self.tracking {
            match g.touches.iter().find(|t| t.id == id) {
                Some(t) => {
                    let d = t.pos - self.start;
                    if d.length() > SWIPE_MIN_PX {
                        let wd = world_dir(d);
                        if !self.on || wd.dot(self.dir) < 0.9995 {
                            self.start_walk(wd, pos, now, area);
                        }
                        self.swiped = true;
                    }
                }
                None => {
                    // lifted: a tap stops (a quick flick was a swipe: in `swipes`)
                    let flick = g.swipes.iter().find(|s| s.from.distance(self.start) < 1.0);
                    if let Some(s) = flick {
                        self.start_walk(world_dir(s.dir), pos, now, area);
                    } else if !self.swiped {
                        self.stop();
                    }
                    self.tracking = None;
                }
            }
        } else if !pinching {
            // a whole swipe or tap between two frames
            if let Some(s) = g.swipes.first() {
                self.start_walk(world_dir(s.dir), pos, now, area);
            } else if !g.taps.is_empty() {
                self.stop();
            }
        }
        self.seen = ids;
    }

    /// Is a step of the figure's width this way free? Rays at chest height (low steps and curbs do not count) from
    /// your middle, ignoring your own capsule (`me`).
    fn free(ctx: &Ctx, pos: Vec3, d: Vec2, me: Option<Entity>) -> bool {
        let dir = Vec3::new(d.x, 0.0, d.y);
        let side = Vec3::new(-d.y, 0.0, d.x);
        let skip: Vec<Entity> = me.into_iter().collect();
        (-1..=1).all(|k| {
            let a = pos + Vec3::Y * 0.6 + side * 0.2 * k as f32;
            ctx.physics.raycast_excluding(a, dir, 0.75, &skip).is_none()
        })
    }

    /// The door nearest to you (village: the house doors; inside: the way out), metres.
    fn near_door(pos: Vec3, area: Option<rooms::RoomId>) -> Option<Vec2> {
        let p = Vec2::new(pos.x, pos.z) / STUD;
        match area {
            None => {
                let mut best = 14.0;
                let mut out = None;
                for id in [rooms::RoomId::Home, rooms::RoomId::Dex, rooms::RoomId::Lab] {
                    let r = rooms::room_def(id);
                    let dd = Vec2::new(r.door_x - p.x, r.door_z + 0.8 - p.y).length();
                    if dd < best {
                        best = dd;
                        out = Some(Vec2::new(r.door_x, r.door_z + 0.8) * STUD);
                    }
                }
                out
            }
            Some(rooms::RoomId::Bedroom) => None,
            Some(id) => {
                let r = rooms::room_def(id);
                Some(Vec2::new(r.ox, r.oz + r.d * 0.5) * STUD)
            }
        }
    }

    /// The way to go now (world x/z), or None: stuck or through a door, stop (Kaigen `swipe_step`). `pos` = feet, m.
    pub fn step(&mut self, ctx: &Ctx, pos: Vec3, me: Option<Entity>, now: f32, area: Option<rooms::RoomId>) -> Option<Vec2> {
        self.dodging = false;
        if area != self.area {
            return None; // through a door: stand still on the other side
        }
        let a = self.dir;
        // no progress for 1.5 s: a wall, stop instead of searching back and forth
        let along = pos.x * a.x + pos.z * a.y;
        if along > self.best + 0.12 {
            self.best = along;
            self.gained = now;
        } else if now - self.gained > 1.5 {
            return None;
        }
        if Self::free(ctx, pos, a, me) {
            return Some(a);
        }
        // around the obstacle: one side the first time (towards a door nearby), then keep it
        let mut order = [1, -1];
        if let Some(door) = Self::near_door(pos, area) {
            let to = door - Vec2::new(pos.x, pos.z);
            let rot = |s: f32| Vec2::new(a.x * s.cos() - a.y * s.sin(), a.x * s.sin() + a.y * s.cos());
            if rot(-1.0).dot(to) > rot(1.0).dot(to) {
                order = [-1, 1];
            }
        }
        for ang in [0.45f32, 0.8, 1.15, std::f32::consts::FRAC_PI_2] {
            let n = if self.side != 0 { 1 } else { 2 };
            for o in order.iter().take(n) {
                let sd = if self.side != 0 { self.side } else { *o };
                let (c, s) = ((ang * sd as f32).cos(), (ang * sd as f32).sin());
                let d = Vec2::new(a.x * c - a.y * s, a.x * s + a.y * c);
                if Self::free(ctx, pos, d, me) {
                    self.side = sd;
                    self.dodging = true;
                    return Some(d);
                }
            }
        }
        None
    }
}
