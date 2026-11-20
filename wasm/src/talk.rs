//! Talking to the villagers (Kaigen `st_world.c` targets + the talk parts of `starting_template.c`):
//! who is in reach (the "E Talk to Dex" prompt, the touch Use button), the camera that goes to your eyes
//! and looks at whom you talk to (and back), you and them turning to each other, the goal pill top left
//! and the "!" / "…" markers over the villagers' heads (Kaigen `cg_quest_draw`, `cg_markers_draw`).
//!
//! Hooks for later parts:
//! * [`TARGETS`] lists every talk spot (studs) with its [`DialogId`]; the rooms part adds the indoor
//!   villagers to `PlayScreen::npcs` (their target spots are already here), Stonecrest adds the knight
//!   and the baker. Pip's spot follows him (`Life::pip`).
//! * [`goal_text`] is the quest line from the save flags. [`CamZoom`] eases the camera to any board
//!   ([`Board`]: a sign, a TV screen, a face) and back — the sign close-ups can use it.

#![allow(dead_code)] // hooks for later parts (see the module doc)

use crate::dialog::DialogId;
use crate::save::GameSave;
use crate::town::STUD;
use engine::prelude::*;

/// A talk spot (studs): who you talk to there. `npc` = the villager's id in `PlayScreen::npcs`.
pub struct Target {
    pub x: f32,
    pub z: f32,
    pub verb: &'static str,
    pub label: &'static str,
    pub dialog: DialogId,
    pub npc: &'static str,
}

/// Talk spots (Kaigen `world_target`). Indoor ones (x > 150) wait for the rooms part.
pub const TARGETS: [Target; 7] = [
    Target { x: 193.4, z: -2.8, verb: "Talk to", label: "Mom", dialog: DialogId::Mom, npc: "mom" },
    Target { x: 335.4, z: -1.2, verb: "Talk to", label: "Prof. Quill", dialog: DialogId::Prof, npc: "lab-prof" },
    Target { x: 52.5, z: -1.8, verb: "Talk to", label: "Dex", dialog: DialogId::Dex, npc: "rival" },
    Target { x: 256.1, z: 2.0, verb: "Talk to", label: "Lumi", dialog: DialogId::Lumi, npc: "lumi" },
    // Prof. Quill at the gap north (Kaigen g_gate_prof_x: 0 while the way is closed)
    Target { x: 0.0, z: -22.6, verb: "Talk to", label: "Prof. Quill", dialog: DialogId::Gate, npc: "gate-prof" },
    Target { x: 341.6, z: -6.4, verb: "Look at", label: "the Deck-o-Matic", dialog: DialogId::Machine, npc: "" },
    // Pip: moved to where he is every frame
    Target { x: -8.0, z: 13.2, verb: "Talk to", label: "Pip", dialog: DialogId::Pip, npc: "kid" },
];
pub const PIP_TARGET: usize = 6;
/// Reach (studs): 4.2 for everyone, 6 for Pip (he stops walking within 6).
pub fn reach(i: usize) -> f32 {
    if i == PIP_TARGET {
        crate::life::PIP_TALK
    } else {
        4.2
    }
}

/// The current goal (the three.js quest line, Kaigen `cg_quest_draw`).
pub fn goal_text(s: &GameSave) -> String {
    let owned = (s.collection & ((1u64 << crate::dialog::CARD_POOL) - 1)).count_ones();
    if !s.has_deck && !s.quest_part {
        "Visit Prof. Quill in her lab (south-east)".into()
    } else if !s.has_deck && !s.has_part && !s.has_suit {
        "Find Prof. Quill's old diving suit somewhere in the lab".into()
    } else if !s.has_deck && !s.has_part {
        "Dive into the pond and find the Stud Core, deep in the cave".into()
    } else if !s.has_deck {
        "Bring the Stud Core to Prof. Quill".into()
    } else if s.rival_wins == 0 {
        "Beat Dex \u{2014} he waits at his big house (east)".into()
    } else if s.voxelmon == 0 {
        format!("Win card battles for EXP and upgrade a 6-energy card into a Voxelmon \u{b7} {} EXP", s.exp)
    } else if !s.gate_open {
        "Show your Voxelmon to Prof. Quill at the village exit (north)".into()
    } else if owned < crate::dialog::CARD_POOL {
        format!("Battle wild Voxelmon on Route 1 \u{b7} {owned}/{} cards", crate::dialog::CARD_POOL)
    } else {
        "Collection complete! You are a Brick Beast master.".into()
    }
}

/// Which villagers get the big "!" (they matter for the goal); the others get "…" (Kaigen updateMarkers).
pub fn marker_big(id: &str, s: &GameSave, mom_talked: bool, kid_talked: bool) -> bool {
    match id {
        "rival" => s.has_deck && s.rival_wins == 0,
        "kid" => s.golden_count() < 15 && !kid_talked,
        "gate-prof" => s.voxelmon > 0 && !s.gate_open,
        "mom" => !s.has_deck && !mom_talked,
        "lumi" => s.has_deck && s.wins == 0,
        "lab-prof" => !s.has_deck && (!s.quest_part || s.has_part),
        _ => false,
    }
}

/// Something the camera looks at squarely: centre (metres), facing yaw (0 = +z), size (studs) and the
/// farthest the camera may be (metres, 0 = no limit), as the Kaigen `SignBoard`.
#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub c: Vec3,
    pub yaw: f32,
    pub w: f32,
    pub h: f32,
    pub max_dist: f32,
}

/// Eases the camera from where the orbit camera has it to a [`Board`] and back (0.6 s, smoothstep).
/// While active the `ThirdPersonCamera` has no target (the engine leaves the camera alone) and gets it
/// back when the camera has returned.
#[derive(Default)]
pub struct CamZoom {
    pub t: f32,
    pub active: bool,
    from: Option<Transform>,
    target: Option<Entity>,
    board: Option<Board>,
}

impl CamZoom {
    /// Every frame (after moving things, before the engine's camera update). Returns true while it owns the camera.
    pub fn update(&mut self, ctx: &mut Ctx, cam: Entity, want: Option<Board>, dt: f32) -> bool {
        if want.is_some() && !self.active {
            let Ok(t) = ctx.world.get::<&Transform>(cam).map(|t| *t) else { return false };
            self.active = true;
            self.t = 0.0;
            self.from = Some(t);
            if let Ok(mut c) = ctx.world.get::<&mut ThirdPersonCamera>(cam) {
                self.target = c.target.take();
            }
        }
        if !self.active {
            return false;
        }
        if want.is_some() {
            self.board = want;
        }
        let dir = if want.is_some() { 1.0 } else { -1.0 };
        self.t = (self.t + dir * dt / 0.6).clamp(0.0, 1.0);
        let from = self.from.unwrap_or_default();
        if self.t <= 0.0 {
            self.active = false;
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(cam) {
                *t = from;
            }
            if let Ok(mut c) = ctx.world.get::<&mut ThirdPersonCamera>(cam) {
                c.target = self.target.take();
            }
            return false;
        }
        let Some(bd) = self.board else { return true };
        let fov = ctx.world.get::<&Camera>(cam).map(|c| c.fov_y_deg).unwrap_or(38.0);
        let (w, h) = ctx.renderer.size();
        let aspect = (w as f32 / h.max(1) as f32).max(0.5);
        let tan_half = (fov.to_radians() * 0.5).tan();
        let mut dist = (bd.h * 1.3 * 0.5 / tan_half).max(bd.w * 1.15 * 0.5 / (tan_half * aspect)) * STUD;
        if bd.max_dist > 0.0 {
            dist = dist.min(bd.max_dist);
        }
        let normal = Vec3::new(bd.yaw.sin(), 0.0, bd.yaw.cos());
        let to = Transform::from_translation(bd.c + normal * dist).looking_at(bd.c, Vec3::Y);
        let k = self.t * self.t * (3.0 - 2.0 * self.t);
        if let Ok(mut t) = ctx.world.get::<&mut Transform>(cam) {
            t.translation = from.translation.lerp(to.translation, k);
            t.rotation = from.rotation.slerp(to.rotation, k);
        }
        true
    }
}

/// The board for talking to someone standing at `npc` (feet, metres) while you stand at `me`: their
/// head and shoulders from your eyes (never farther than you stand). Kaigen aimed at the chin (3.3 studs
/// up) with a wider lens; with this camera's 38 degrees that cut the face off, so it aims at the face (4 studs).
pub fn face_board(npc: Vec3, me: Vec3) -> Board {
    let to_me = Vec2::new(me.x - npc.x, me.z - npc.z);
    let yaw = if to_me.length() > 0.01 { to_me.x.atan2(to_me.y) } else { 0.0 };
    Board { c: npc + Vec3::Y * 4.0 * STUD, yaw, w: 4.6, h: 5.6, max_dist: (to_me.length() - 0.15).max(1.3) }
}
