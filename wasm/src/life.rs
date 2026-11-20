//! Village life (Kaigen `st_life.c`, the part that is about the villagers): Pip walks his loop around
//! the pond, pauses now and then, stops and turns to you when you come within 6 studs; toasts (the short
//! messages at the top). Positions in studs.
//!
//! Hooks for later parts: [`PipMode`] already has the hide and seek / tag / football modes of the Kaigen
//! port; `Life::pip_mode` switches them and `Life::step_pip` is where their movement goes (the hide and
//! seek part, the ball). `Life::toast` shows a message (e.g. "You found Pip! +5 studs"). The town's
//! colliders (`Life::blocks`) are kept for Pip's hiding spots and fleeing.

#![allow(dead_code)] // hooks for later parts (see the module doc)

use crate::town::Rect;
use engine::prelude::*;

/// Pip's loop around the pond, never through it (the west bank between the water and the garden fence).
const PIP_WAYPOINTS: [[f32; 2]; 6] = [[-8.0, 13.2], [-3.0, 13.8], [-2.8, 23.0], [-11.0, 23.2], [-12.9, 22.9], [-12.9, 14.0]];
/// Studs per second: Pip strolling (the three.js speed; the Kaigen port scaled it to its faster walk).
pub const PIP_WALK: f32 = 3.2;
/// Within this many studs Pip stops and looks at you (and you can talk to him).
pub const PIP_TALK: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum PipMode {
    /// His loop around the pond.
    Walk,
    /// Chasing the ball (ball part).
    Football,
    /// Hide and seek: hidden somewhere (hide and seek part).
    Hiding,
    /// Found, or the game is over: waves until you walk off.
    Wave,
    /// Tag: runs from you.
    Flee,
}

pub struct Life {
    seed: u32,
    pub pip: Vec2,
    pub pip_yaw: f32,
    pub pip_mode: PipMode,
    pub pip_wp: usize,
    pub pip_pause: f32,
    /// Pip's speed this step (studs/s) for his walk cycle.
    pub pip_speed: f32,
    /// Tests: routines hold still.
    pub frozen: bool,
    /// The message at the top and its seconds left.
    pub toast: String,
    pub toast_t: f32,
    pub blocks: Vec<Rect>,
}

impl Default for Life {
    fn default() -> Self {
        Self {
            seed: 4242,
            pip: Vec2::new(PIP_WAYPOINTS[0][0], PIP_WAYPOINTS[0][1]),
            pip_yaw: 0.0,
            pip_mode: PipMode::Walk,
            pip_wp: 0,
            pip_pause: 0.0,
            pip_speed: 0.0,
            frozen: false,
            toast: String::new(),
            toast_t: 0.0,
            blocks: vec![],
        }
    }
}

/// Wraps an angle to -pi..pi.
pub fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
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

    /// Pip back at the start of his loop (tests).
    pub fn pip_home(&mut self) {
        self.pip = Vec2::new(PIP_WAYPOINTS[0][0], PIP_WAYPOINTS[0][1]);
        self.pip_wp = 0;
        self.pip_mode = PipMode::Walk;
        self.pip_pause = 0.0;
        self.pip_yaw = 0.0;
    }

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

    /// One fixed step of Pip (Kaigen `life_pip`, the walking part). `p` = you (studs), `talking` = a
    /// dialog with him is open.
    pub fn step_pip(&mut self, p: Vec2, dt: f32, outside: bool, talking: bool) {
        self.pip_speed = 0.0;
        let dp = (p - self.pip).length();
        let face_you = |s: &mut Self, rate: f32| {
            let want = (p.x - s.pip.x).atan2(p.y - s.pip.y);
            s.pip_yaw += wrap(want - s.pip_yaw) * (dt * rate).min(1.0);
        };
        match self.pip_mode {
            PipMode::Wave => {
                face_you(self, 8.0);
                if dp > 24.0 || !outside {
                    self.pip_mode = PipMode::Walk;
                }
            }
            PipMode::Hiding | PipMode::Flee | PipMode::Football => {} // later parts
            PipMode::Walk => {
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
    }

    pub fn update(&mut self, dt: f32) {
        self.toast_t = (self.toast_t - dt).max(0.0);
    }
}
