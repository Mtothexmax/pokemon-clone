//! Talking to the villagers (Kaigen `st_world.c` targets + the talk parts of `starting_template.c`):
//! who is in reach (the "E Talk to Dex" prompt, the touch Use button), the camera that goes to your eyes
//! and looks at whom you talk to (and back), you and them turning to each other, the goal pill top left
//! and the "!" / "…" markers over the villagers' heads (Kaigen `cg_quest_draw`, `cg_markers_draw`).
//!
//! Hooks for later parts:
//! * [`TARGETS`] lists everything you can use (studs) with what it does ([`Use`]: talk with its
//!   [`DialogId`], read a sign, the TV, the piano, the computers, the bed); `PlayScreen::use_target`
//!   dispatches. Stonecrest adds the knight, the baker and its signs. The lab quest's diving suit is
//!   `Use::Suit` (`SUIT_TARGET`, only while it stands there). Pip's spot follows him (`Life::pip`), the chest's
//!   the chest (`CHEST_TARGET`, only while it stands there); the ducks' feeding spot and the park bench.
//! * [`goal_text`] is the quest line from the save flags. [`CamZoom`] eases the camera to any board
//!   ([`Board`]: a sign, a TV screen, a face; `tilt` for screens leaning back) and back.

#![allow(dead_code)] // hooks for later parts (see the module doc)

use crate::dialog::DialogId;
use crate::save::GameSave;
use crate::town::STUD;
use engine::prelude::*;

/// A talk spot (studs): who you talk to there. `npc` = the villager's id in `PlayScreen::npcs`.
pub struct Target {
    pub kind: Use,
    pub x: f32,
    pub z: f32,
    pub verb: &'static str,
    pub label: &'static str,
    pub dialog: DialogId,
    pub npc: &'static str,
}

/// What a target does (Kaigen `TargetKind`): talk (its `dialog`), read a sign (`readable`), watch the TV,
/// play the piano, use a computer, sleep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    Talk,
    Read(usize),
    Tv,
    Piano,
    Pc,
    Lab,
    Sleep,
    /// Prof. Quill's old diving suit in the lab corner (only while it is there: `dive::suit_here`)
    Suit,
    /// throw bread to the ducks (life.rs `Life::feed`)
    Feed,
    /// sit on the park bench (time flies)
    Sit,
    /// open the treasure chest (its spot moves: `Life::chest_p`, only while it stands there)
    Chest,
}

const fn thing(kind: Use, x: f32, z: f32, verb: &'static str, label: &'static str) -> Target {
    Target { kind, x, z, verb, label, dialog: DialogId::IntroKeys, npc: "" }
}

/// Everything you can do something with (Kaigen `world_target`): the villagers, the signs and the
/// mailbox, the TVs, the piano, the computers, your bed. Pip's spot follows him.
pub const TARGETS: [Target; 27] = [
    Target { kind: Use::Talk, x: 193.4, z: -2.8, verb: "Talk to", label: "Mom", dialog: DialogId::Mom, npc: "mom" },
    Target { kind: Use::Talk, x: 335.4, z: -1.2, verb: "Talk to", label: "Prof. Quill", dialog: DialogId::Prof, npc: "lab-prof" },
    Target { kind: Use::Talk, x: 52.5, z: -1.8, verb: "Talk to", label: "Dex", dialog: DialogId::Dex, npc: "rival" },
    Target { kind: Use::Talk, x: 256.1, z: 2.0, verb: "Talk to", label: "Lumi", dialog: DialogId::Lumi, npc: "lumi" },
    // Prof. Quill at the gap north (Kaigen g_gate_prof_x: 0 while the way is closed)
    Target { kind: Use::Talk, x: 0.0, z: -22.6, verb: "Talk to", label: "Prof. Quill", dialog: DialogId::Gate, npc: "gate-prof" },
    Target { kind: Use::Talk, x: 341.6, z: -6.4, verb: "Look at", label: "the Deck-o-Matic", dialog: DialogId::Machine, npc: "" },
    // Pip: moved to where he is every frame
    Target { kind: Use::Talk, x: -8.0, z: 13.2, verb: "Talk to", label: "Pip", dialog: DialogId::Pip, npc: "kid" },
    // the signs with a board (the camera flies to it) and the mailbox (a card in the middle)
    thing(Use::Read(0), 5.9, 21.8, "Read", "the village sign"),
    thing(Use::Read(1), 5.0, -22.8, "Read", "the route sign"),
    thing(Use::Read(2), -21.4, 8.9, "Read", "the garden sign"),
    thing(Use::Read(3), 13.0, 18.6, "Read", "the lab sign"),
    thing(Use::Read(4), -18.2, -2.4, "Read", "the mailbox"),
    // in the rooms
    thing(Use::Tv, 205.0, -4.6, "Watch", "TV"),
    thing(Use::Tv, 269.0, -4.4, "Watch", "TV"),
    thing(Use::Piano, 251.0, -4.4, "Play", "the piano"),
    thing(Use::Pc, 201.4, 36.6, "Use", "your PC"),
    thing(Use::Sleep, 196.2, 38.6, "Sleep in", "your bed"),
    thing(Use::Lab, 343.2, -2.2, "Use", "the lab computer"),
    thing(Use::Lab, 343.2, 3.3, "Use", "the lab computer"),
    // the lab quest: the suit behind the bookshelves (Kaigen st_world.c case 15)
    thing(Use::Suit, 315.6, -7.2, "Take", "the old diving suit"),
    // Route 1 north and Stonecrest (Kaigen st_world.c cases 17..20)
    thing(Use::Read(5), 5.5, -48.0, "Read", "the route sign"),
    thing(Use::Read(6), 5.5, -107.4, "Read", "the town sign"),
    Target { kind: Use::Talk, x: 7.0, z: -144.8, verb: "Talk to", label: "Sir Bricksworth", dialog: DialogId::Knight, npc: "knight" },
    Target { kind: Use::Talk, x: -8.0, z: -109.2, verb: "Talk to", label: "Marla", dialog: DialogId::Baker, npc: "baker" },
    // village life (Kaigen st_world.c: the pond's feeding spot, the park bench; the chest moves)
    thing(Use::Feed, -2.7, 18.5, "Feed", "the ducks"),
    thing(Use::Sit, 15.0, -7.4, "Sit on", "the park bench"),
    thing(Use::Chest, 0.0, 0.0, "Open", "the treasure chest"),
];
pub const CHEST_TARGET: usize = 26;
pub const SIT_TARGET: usize = 25;
pub const FEED_TARGET: usize = 24;
pub const SUIT_TARGET: usize = 19;
/// Prof. Quill at the gap: her spot moves aside once the way is open (`town::GATE_PROF_OPEN`).
pub const GATE_TARGET: usize = 4;
pub const PIP_TARGET: usize = 6;
/// Reach (studs): 4.2 for everyone, 6 for Pip (he stops walking within 6).
pub fn reach(i: usize) -> f32 {
    if i == PIP_TARGET {
        crate::life::PIP_TALK
    } else if TARGETS[i].kind == Use::Talk {
        4.2
    } else {
        3.6
    }
}

/// A sign's text (Kaigen `readable`): title, lines, a note, background and ink colours.
pub struct Readable {
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub note: Option<&'static str>,
    pub bg: &'static str,
    pub fg: &'static str,
}

pub fn readable(i: usize) -> Readable {
    let (paper, ink, wood, brown) = ("#f4f4f4", "#1b2a34", "#fff6c8", "#6b3f1d");
    match i {
        0 => Readable { title: "BRICKHOLLOW", lines: &["\"A village where every brick has a story.\"", "Population: 6 minifigs, lots of Beasts, countless studs."], note: None, bg: paper, fg: ink },
        1 => Readable { title: "ROUTE 1", lines: &["Tall grass ahead!", "Wild Brick Beasts love to hide in it. Keep your deck ready."], note: None, bg: wood, fg: brown },
        2 => Readable { title: "GARDEN", lines: &["Please do not step on the flowers."], note: Some("Pip, that means YOU."), bg: "#e4cd9e", fg: brown },
        3 => Readable { title: "QUILL LAB", lines: &["Brick Beast research since 1978.", "Opening hours: whenever the Professor remembers to unlock the door."], note: None, bg: paper, fg: "#1e5aa8" },
        5 => Readable { title: "STONECREST \u{25b2}", lines: &["Route 1 goes on through the forest.", "Mind the creek, use the bridge!"], note: None, bg: wood, fg: brown },
        6 => Readable { title: "STONECREST", lines: &["\"The town below the old castle.\"", "Bakery \u{b7} Inn \u{b7} Market \u{b7} Smith"], note: None, bg: paper, fg: "#6c6e68" },
        _ => Readable { title: "MAILBOX", lines: &["A postcard: \"Welcome home! Drop by the lab when you are ready.\"", "- Prof. Quill"], note: None, bg: paper, fg: ink },
    }
}

/// The painted board of readables 0..3, 5, 6 (metres; the camera flies to it), as the town places the signs.
pub fn sign_board(i: usize) -> Option<Board> {
    let b = |x: f32, y: f32, z: f32, yaw: f32, w: f32, h: f32| Some(Board { c: Vec3::new(x, y, z) * STUD, yaw, w, h, max_dist: 0.0, tilt: 0.0 });
    match i {
        0 => b(5.9, 2.3, 20.41, 0.0, 3.8, 2.0),
        1 => b(5.0, 2.2, -24.09, 0.0, 3.4, 1.8),
        2 => b(-22.91, 2.1, 8.9, std::f32::consts::FRAC_PI_2, 2.8, 1.5),
        3 => b(13.0, 5.75, 16.17, 0.0, 5.2, 1.9),
        5 => b(5.5, 2.2, -48.99, 0.0, 3.4, 1.8),
        6 => b(5.5, 2.2, -108.39, 0.0, 3.4, 1.8),
        _ => None,
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
    /// leaning back (radians; the lab computers' screens): the camera looks down at it
    pub tilt: f32,
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
        let normal = Vec3::new(bd.yaw.sin() * bd.tilt.cos(), bd.tilt.sin(), bd.yaw.cos() * bd.tilt.cos());
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
    Board { c: npc + Vec3::Y * 4.0 * STUD, yaw, w: 4.6, h: 5.6, max_dist: (to_me.length() - 0.15).max(1.3), tilt: 0.0 }
}
