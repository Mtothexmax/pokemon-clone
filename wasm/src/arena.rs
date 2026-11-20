//! The card battle in 3D (Kaigen `st_cardgame.c` + `st_cardui.c` battle_draw / battle_input, rules in `battle.rs`).
//!
//! A screen pushed over the village: a wooden table with green felt seen from the player's chair, the three
//! locations as thick tiles in a row (they flip over from "?" to their picture when revealed), the opponent's slots
//! beyond them, yours in front. Every card is a thin 3D slab (printed face, card back, cream edge) that is always
//! somewhere: on a deck pile, fanned in your hand at the bottom of the view (camera space), fanned face down at the
//! far edge (the opponent's hand), or lying in a slot. When its place changes it flies there: played cards are
//! lowered onto the slot with a small bounce, cards turned over at the reveal flip around their long axis, drawn
//! cards fly out of the pile. Power changes are 3D number badges (one atlas), location totals lie at the tiles'
//! edges, the leading side's edge glows. Attacks are glowing balls with impact sparks, destroyed cards shatter into
//! six pieces, a win rains LEGO bricks.
//!
//! The camera is fixed (a slight sway) and fitted to the screen every frame: the board fills the space between
//! the top bar and the hand; a phone held upright gets narrower columns and a steeper view. Picking is a ray
//! against each card's current slab and against the table plane. The HUD (energy orb, End Turn, Undo, SNAP,
//! Retreat, the portraits with the 1st / 2nd badge, the taunt bubble, the big card's buttons, the result) is HTML
//! with `data-action` buttons, so everything works by touch.
//!
//! The way in (Kaigen `cg_transition_draw`): bricks slide over the village, YOU vs the opponent, the board is set
//! up under the bricks (`Transition`, drawn by both screens).

#![allow(deprecated)] // Mat4::perspective_rh / look_at_rh (the engine uses them too)

use crate::battle::{self, Battle, BattleKind, Outcome, Phase, SIDE_AI, SIDE_P};
use crate::cardface::Face;
use crate::cards;
use crate::sound::{Sfx, Track};
use crate::table::{self, CARD_H, CARD_T, TILE_H, TILE_T};
use crate::Data;
use engine::particles::{ParticleEmitter, ParticlePreset};
use engine::prelude::*;
use serde_json::{json, Value};
use std::f32::consts::{FRAC_PI_2, PI};

/// Where the table stands: far from the village (no village shadow or light reaches it).
const BASE: Vec3 = Vec3::new(0.0, 0.0, 900.0);
/// The table's render layer (the battle camera draws only this one).
pub const LAYER: u8 = 29;
const HIDDEN: u8 = crate::town::rooms::HIDDEN_LAYER;
/// The gap between slots (the card width on the board is `Lay::bw`).
const GAP: f32 = 0.06;
const FOV: f32 = 40.0;
/// Hand cards and the big card: depth in front of the camera.
const HAND_D: f32 = 2.0;
const INSPECT_D: f32 = 1.3;
const DIM_D: f32 = 1.6;
/// A card dragged over the table floats this high.
const DRAG_H: f32 = 0.5;
/// The way into a battle (s) and the time the board is set up under the bricks.
pub const TRANS: f32 = 2.1;
pub const TRANS_BOARD: f32 = 1.55;

// ---------------------------------------------------------------- the way in (shared with the play screen)

pub struct Transition {
    pub t: f32,
    pub kind: BattleKind,
    #[allow(dead_code)]
    pub opponent: &'static str,
}

/// Starts the way into a battle (the intro fanfare, the bricks).
pub fn start_transition(ctx: &mut Ctx, data: &mut Data, kind: BattleKind) {
    let opponent = match kind {
        BattleKind::Pip => "PIP",
        BattleKind::Rival if data.save.data.rival_wins > 0 => "DEX \u{2605}",
        BattleKind::Rival => "DEX",
        BattleKind::Wild => "???",
        BattleKind::Practice => "LUMI",
    };
    data.trans = Some(Transition { t: 0.0, kind, opponent });
    data.sound.music(ctx, Track::Intro);
    ctx.ui.set("bt_trans", &transition_html(opponent));
}

/// Advances the bricks; true once the board should be set up (every frame, from whichever screen is on top).
pub fn transition_update(ctx: &mut Ctx, data: &mut Data, dt: f32) -> bool {
    let Some(tr) = &mut data.trans else { return false };
    let (t0, t1) = (tr.t, tr.t + dt);
    tr.t = t1;
    if t0 < 0.2 && t1 >= 0.2 {
        data.sound.sfx(ctx, Sfx::Kick, 0.7, 0.5); // the bricks rush in
    }
    if t0 < 1.12 && t1 >= 1.12 {
        data.sound.sfx(ctx, Sfx::Pop, 0.55, 0.9); // the VS lands
    }
    if t1 >= TRANS {
        data.trans = None;
        ctx.ui.set("bt_trans", "");
    }
    t0 < TRANS_BOARD && t1 >= TRANS_BOARD
}

/// The bricks, YOU vs them, the flashes: css animations started once (timed like Kaigen `cg_transition_draw`).
fn transition_html(opp: &str) -> String {
    let colors = ["#c91a09", "#f2cd37", "#0055bf", "#237841", "#fe8a18"];
    let mut rows = String::new();
    for r in 0..7 {
        let dir = if r % 2 == 1 { "r" } else { "l" };
        let mut bricks = String::new();
        for c in 0..12 {
            let col = colors[((r * 7 + c * 13 + r * c) % 5) as usize];
            bricks.push_str(&format!("<i style='background:{col}'><b></b><b></b><b></b><b></b><b></b><b></b><b></b><b></b></i>"));
        }
        let din = 0.18 + 0.045 * r as f32;
        let dout = 1.65 + 0.035 * r as f32;
        rows.push_str(&format!(
            "<div class='btt-row {dir}' style='top:{top:.3}%;animation-delay:{din:.3}s,{dout:.3}s'>{bricks}</div>",
            top = r as f32 * 100.0 / 6.0
        ));
    }
    format!(
        "<div class='btt'>{rows}<div class='btt-you'>YOU</div><div class='btt-opp'>{opp}</div><div class='btt-vs'>VS</div><div class='btt-flash'></div></div>"
    )
}

/// Styles of the battle HUD and the transition.
pub const STYLE: &str = "<style>
#ui [data-slot=bt_trans]{position:fixed;inset:0;z-index:80;pointer-events:none}#ui [data-slot=bt_trans]:empty{display:none}
.btt{position:fixed;inset:0;overflow:hidden}
.btt-row{position:absolute;left:-12vh;height:calc(100% / 6 + 1px);min-height:60px;display:flex;white-space:nowrap;transform:translateX(-120vw);animation:bttin .4s cubic-bezier(.2,.8,.3,1) forwards,bttout .32s ease-in forwards}
.btt-row.r{left:auto;right:-12vh;flex-direction:row-reverse;transform:translateX(120vw);animation-name:bttinr,bttoutr}
@keyframes bttin{from{transform:translateX(-120vw)}to{transform:translateX(0)}}
@keyframes bttout{from{transform:translateX(0)}to{transform:translateX(120vw)}}
@keyframes bttinr{from{transform:translateX(120vw)}to{transform:translateX(0)}}
@keyframes bttoutr{from{transform:translateX(0)}to{transform:translateX(-120vw)}}
.btt-row i{flex:none;width:calc(100vh / 3);min-width:120px;height:100%;box-sizing:border-box;border:2px solid rgba(0,0,0,.35);border-radius:6px;display:grid;grid-template-columns:repeat(4,1fr);grid-template-rows:1fr 1fr;place-items:center;padding:6%;filter:brightness(1)}
.btt-row b{width:56%;aspect-ratio:1;border-radius:50%;background:linear-gradient(135deg,rgba(255,255,255,.35),rgba(0,0,0,.12));box-shadow:2px 3px 3px rgba(0,0,0,.35)}
.btt-you,.btt-opp{position:absolute;top:calc(50% - 10vh);height:20vh;min-height:90px;max-height:170px;width:50%;display:flex;align-items:center;font-weight:800;color:#fff;font-size:clamp(34px,7.6vh,64px);text-shadow:0 3px 6px rgba(0,0,0,.5);box-shadow:0 6px 18px rgba(0,0,0,.45);opacity:0}
.btt-you{left:0;justify-content:flex-end;padding-right:11vh;box-sizing:border-box;background:linear-gradient(90deg,#0b3d91,#1e6fe0);transform:translate(-60vw,-12%) rotate(-4deg);animation:bttyou 1.12s linear .75s forwards}
.btt-opp{right:0;padding-left:11vh;box-sizing:border-box;background:linear-gradient(90deg,#e0301e,#8f1306);transform:translate(60vw,12%) rotate(-4deg);animation:bttopp 1.12s linear .75s forwards}
@keyframes bttyou{0%{opacity:1;transform:translate(-60vw,-12%) rotate(-4deg)}20%{transform:translate(0,-12%) rotate(-4deg)}78%{opacity:1;transform:translate(0,-12%) rotate(-4deg)}100%{opacity:1;transform:translate(-60vw,-12%) rotate(-4deg)}}
@keyframes bttopp{0%{opacity:1;transform:translate(60vw,12%) rotate(-4deg)}20%{transform:translate(0,12%) rotate(-4deg)}78%{opacity:1;transform:translate(0,12%) rotate(-4deg)}100%{opacity:1;transform:translate(60vw,12%) rotate(-4deg)}}
.btt-vs{position:absolute;left:50%;top:50%;width:clamp(100px,23vh,195px);aspect-ratio:1;margin:calc(clamp(100px,23vh,195px) / -2) 0 0 calc(clamp(100px,23vh,195px) / -2);border-radius:50%;background:radial-gradient(circle at 50% 40%,#fff3a0,#f2a705);border:5px solid #fff;box-sizing:border-box;display:grid;place-items:center;font-weight:800;color:#c91a09;font-size:clamp(40px,9.6vh,80px);text-shadow:0 2px 0 #fff;box-shadow:0 6px 20px rgba(0,0,0,.5);opacity:0;animation:bttvs .92s linear .95s forwards}
@keyframes bttvs{0%{opacity:0;transform:scale(2.6) rotate(-8deg)}10%{opacity:1}24%{transform:scale(.9) rotate(0)}32%{transform:scale(1)}73%{opacity:1;transform:scale(1)}100%{opacity:0;transform:scale(.4)}}
.btt-flash{position:absolute;inset:0;background:#fff;opacity:0;animation:bttfl 1.35s linear forwards}
@keyframes bttfl{0%{opacity:0}12%{opacity:.85}24%{opacity:0}86.6%{opacity:0}86.7%{opacity:.55}100%{opacity:0}}
#ui [data-slot^=bt_]{pointer-events:none}
#ui [data-slot=bt_top]{position:fixed;left:0;right:0;top:0;z-index:30}
.bt-top{display:flex;align-items:center;gap:8px;padding:6px max(10px,env(safe-area-inset-right)) 6px max(10px,env(safe-area-inset-left));height:50px;box-sizing:border-box;background:linear-gradient(rgba(5,10,8,.62),rgba(5,10,8,.18));color:#fff;font-weight:700}
.bt-port{position:relative;flex:none;width:38px;height:38px;border-radius:11px;border:3px solid rgba(255,255,255,.6);display:grid;place-items:center;font-size:18px;font-weight:800;box-sizing:border-box}
.bt-port.lit{border-color:#ffd23d;animation:btpulse .8s ease-in-out infinite alternate}
@keyframes btpulse{from{box-shadow:0 0 8px 2px rgba(255,210,61,.55)}to{box-shadow:0 0 18px 5px rgba(255,210,61,.95)}}
.bt-badge{position:absolute;right:-10px;top:-7px;font-size:11px;font-weight:900;padding:1px 4px;border-radius:7px;background:#3a3f4a;border:1px solid #fff;color:#fff;line-height:13px}
.bt-badge.first{background:#ffd23d;color:#2a1a00}
.bt-name{display:flex;flex-direction:column;line-height:1.1;min-width:0}.bt-name b{font-size:17px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.bt-name small{font-size:11px;opacity:.8;font-weight:600}
.bt-sp{flex:1}
.bt-stakes{display:flex;gap:5px;align-items:center;padding:2px 10px;border-radius:14px;border:2px solid rgba(255,255,255,.4);background:rgba(0,0,0,.35);font-size:19px;font-weight:800}
.bt-stakes.hot{background:linear-gradient(#ff9a3d,#d9361b);box-shadow:0 0 14px #ff9a3d}
.bt-stakes i{width:16px;height:10px;background:#f2cd37;border-radius:2px;position:relative;margin-top:3px}.bt-stakes i::before{content:'';position:absolute;left:2px;top:-3px;width:4px;height:3px;background:#f2cd37;border-radius:1px 1px 0 0;box-shadow:7px 0 0 #f2cd37}
.bt-turn{font-size:14px;font-weight:500}.bt-turn b{font-size:19px;font-weight:800}
#ui .bt-btn{pointer-events:auto;margin:0;border:0;border-radius:10px;padding:7px 12px;font-weight:800;font-size:14px;color:#fff;background:rgba(255,255,255,.16);border-bottom:3px solid rgba(0,0,0,.3);cursor:pointer;touch-action:manipulation}
#ui .bt-btn:hover{filter:brightness(1.15);background:rgba(255,255,255,.24)}
#ui .bt-btn:disabled{opacity:.4;pointer-events:none}
#ui .bt-btn.snap{background:linear-gradient(#ffb43d,#e0541e);font-size:16px;letter-spacing:.5px}
#ui .bt-btn.blue{background:linear-gradient(#3d9bff,#1655b0);font-size:20px;padding:10px 16px}
#ui .bt-btn.blue.go{animation:btgo .7s ease-in-out infinite alternate}
@keyframes btgo{from{box-shadow:0 0 0 0 rgba(61,155,255,.6)}to{box-shadow:0 0 0 7px rgba(61,155,255,.15)}}
#ui .bt-btn.green{background:linear-gradient(#4fc06a,#1e7a35);font-size:17px}
#ui [data-slot=bt_bottom]{position:fixed;left:0;right:0;bottom:0;z-index:30}
.bt-bot{position:relative;height:var(--bh);box-sizing:border-box}
.bt-orb{position:absolute;left:max(12px,env(safe-area-inset-left));bottom:max(12px,env(safe-area-inset-bottom));width:var(--orb);height:var(--orb);border-radius:50%;background:radial-gradient(circle at 35% 30%,#7ac0ff,#1655b0);border:4px solid rgba(255,255,255,.7);box-shadow:0 0 20px rgba(61,155,255,.6);display:flex;flex-direction:column;align-items:center;justify-content:center;color:#fff;box-sizing:border-box}
.bt-orb b{font-size:calc(var(--orb) * .42);font-weight:900;line-height:1;text-shadow:0 2px 2px rgba(0,0,0,.3)}.bt-orb b small{font-size:calc(var(--orb) * .17);opacity:.75;font-weight:700}
.bt-orb span{font-size:calc(var(--orb) * .1);font-weight:700;opacity:.85;letter-spacing:.5px}
.bt-orb.spent{filter:saturate(.35) brightness(.8)}
.bt-me{position:absolute;left:calc(max(12px,env(safe-area-inset-left)) + var(--orb) - 10px);bottom:calc(max(12px,env(safe-area-inset-bottom)) + var(--orb) - 22px)}
.bt-ctl{position:absolute;right:max(12px,env(safe-area-inset-right));bottom:max(12px,env(safe-area-inset-bottom));display:flex;flex-direction:column;gap:6px;width:150px}
.bt-bot.phone .bt-ctl{flex-direction:row-reverse;width:auto;left:calc(max(12px,env(safe-area-inset-left)) + var(--orb) + 18px);align-items:stretch}
.bt-bot.phone .bt-ctl .blue{flex:1;font-size:18px}
#ui [data-slot=bt_over]{position:fixed;inset:0;z-index:35}
.bt-hint{position:absolute;left:50%;transform:translateX(-50%);background:rgba(0,0,0,.65);color:#fff;border-radius:99px;padding:5px 14px;font-size:14px;font-weight:600;white-space:nowrap;max-width:calc(100vw - 24px);overflow:hidden;text-overflow:ellipsis}
#ui .bt-suggest{pointer-events:auto;position:absolute;left:50%;transform:translateX(-50%);margin:0;border:3px solid #fff;border-radius:99px;padding:10px 22px;font-size:17px;color:#fff;background:linear-gradient(#3d9bff,#1655b0);box-shadow:0 8px 24px rgba(0,0,0,.4);animation:btgo .8s ease-in-out infinite alternate;white-space:nowrap}
.bt-taunt{position:absolute;left:max(14px,env(safe-area-inset-left));top:58px;max-width:min(70vw,340px);background:#fff;color:#1b2a34;border:3px solid #1b2a34;border-radius:14px;padding:8px 14px;font-size:15px;font-weight:700;box-shadow:0 6px 16px rgba(0,0,0,.35);animation:btpop .25s ease-out}
.bt-taunt::before{content:'';position:absolute;left:16px;top:-8px;width:12px;height:12px;background:#1b2a34;transform:rotate(45deg)}
@keyframes btpop{from{opacity:0;transform:scale(.9) translateY(6px)}to{opacity:1;transform:none}}
.bt-banner{position:absolute;left:50%;top:max(110px,15%);transform:translate(-50%,-50%);background:linear-gradient(#2a3a5a,#1b2a34);border:3px solid rgba(255,255,255,.5);color:#fff;border-radius:16px;padding:10px 26px;font-size:24px;font-weight:900;text-align:center;box-shadow:0 10px 30px rgba(0,0,0,.45);animation:btban 1.5s ease-out forwards;white-space:nowrap}
.bt-banner small{display:block;font-size:14px;font-weight:600;opacity:.85}
@keyframes btban{0%{opacity:0;transform:translate(-50%,-50%) scale(1.6)}12%{opacity:1;transform:translate(-50%,-50%) scale(1)}80%{opacity:1}100%{opacity:0}}
.bt-log{position:absolute;right:8px;top:54px;width:min(320px,86vw);background:rgba(16,28,36,.95);border:2px solid rgba(255,255,255,.25);border-radius:14px;padding:10px 12px;color:#fff;font-size:12px;box-sizing:border-box}.bt-log b{font-size:15px}
.bt-insp{position:absolute;left:0;right:0;display:flex;flex-direction:column;align-items:center;gap:8px;color:#fff;font-size:15px;font-weight:600;text-shadow:0 1px 3px rgba(0,0,0,.6)}
.bt-zones{display:flex;gap:6px;width:min(92vw,420px)}
#ui .bt-zones button{flex:1;margin:0;pointer-events:auto;border:3px solid rgba(0,0,0,.35);border-radius:12px;padding:10px 6px;font-weight:800;font-size:14px;color:#fff;text-shadow:0 1px 2px rgba(0,0,0,.6)}
#ui .bt-zones button:disabled{opacity:.35}
.bt-end{position:absolute;inset:0;display:grid;place-items:center;background:rgba(0,0,0,.35)}
.bt-endp{display:flex;flex-direction:column;align-items:center;gap:14px;padding:22px 30px;border-radius:20px;background:linear-gradient(#2a3a5ae8,#1b2a34ee);border:4px solid rgba(255,255,255,.3);color:#fff;animation:btpop .35s ease-out;max-width:calc(100vw - 24px);box-sizing:border-box}
.bt-endp h2{margin:0;font-size:44px;font-weight:900}.bt-endp h2.win{color:#fde047;text-shadow:0 0 18px rgba(253,224,71,.6)}
.bt-sc{display:flex;gap:18px}.bt-sc div{display:flex;flex-direction:column;align-items:center;gap:2px}.bt-sc small{font-size:12px;opacity:.8}.bt-sc b{font-size:24px}.bt-sc i{font-style:normal;font-size:20px;font-weight:900}
.bt-studs{display:flex;gap:8px;align-items:center;font-size:20px;font-weight:700}.bt-studs i{width:20px;height:20px;border-radius:50%;background:radial-gradient(circle at 35% 30%,#fff3a8,#d9a300)}
.world-label.bt-fx{background:none;padding:0;pointer-events:none}
.bt-fx span{display:block;transform:translateY(50%);font-size:26px;font-weight:900;color:#fff;text-shadow:0 0 6px #000,0 2px 0 #000}
.bt-fx span.up{color:#8dff6a}.bt-fx span.dn{color:#ff6a5a}
</style>";

// ---------------------------------------------------------------- the 3D objects

/// Where a card is, how it is turned and how big: `rot * RotY(flip)` (flip PI = face down, turned over its long axis).
#[derive(Clone, Copy, Debug)]
struct Pose {
    pos: Vec3,
    rot: Quat,
    flip: f32,
    scale: f32,
}

impl Pose {
    fn lerp(a: &Pose, b: &Pose, t: f32) -> Pose {
        Pose { pos: a.pos.lerp(b.pos, t), rot: a.rot.slerp(b.rot, t), flip: a.flip + (b.flip - a.flip) * t, scale: a.scale + (b.scale - a.scale) * t }
    }
    fn quat(&self) -> Quat {
        self.rot * Quat::from_rotation_y(self.flip)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Place {
    Deck(usize),
    Hand(usize, usize),
    Board(usize, usize),
    Drag,
    Gone,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
enum Style {
    /// a short move (the hand making room)
    Plain,
    /// lowered onto a slot: over it, then down with a small bounce
    Lower,
    /// turned over with a little lift
    Flip,
    /// out of the deck pile into the hand
    Draw,
}

#[derive(Clone, Copy, Debug)]
struct Anim {
    from: Pose,
    t: f32,
    dur: f32,
    style: Style,
    landed: bool,
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}
fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Height above the slot while a card is lowered (u: 0..1 of the drop): falls, touches at 0.62, a small bounce.
fn drop_height(u: f32) -> f32 {
    if u < 0.62 {
        0.42 * (1.0 - (u / 0.62).powi(2))
    } else {
        0.05 * (PI * (u - 0.62) / 0.38).sin()
    }
}

impl Anim {
    fn eval(&self, to: &Pose) -> Pose {
        let t = (self.t / self.dur).clamp(0.0, 1.0);
        match self.style {
            Style::Plain => Pose::lerp(&self.from, to, ease_out(t)),
            Style::Flip => {
                let mut p = Pose::lerp(&self.from, to, ease_in_out(t));
                p.pos.y += 0.34 * (PI * t).sin();
                p
            }
            Style::Draw => {
                let mut p = Pose::lerp(&self.from, to, ease_out(t));
                p.pos.y += 0.3 * (PI * t).sin();
                p
            }
            Style::Lower => {
                let hover = Pose { pos: to.pos + Vec3::Y * 0.42, ..*to };
                if t < 0.5 {
                    let e = ease_out(t / 0.5);
                    Pose::lerp(&self.from, &hover, e)
                } else {
                    let u = (t - 0.5) / 0.5;
                    Pose { pos: to.pos + Vec3::Y * drop_height(u), ..*to }
                }
            }
        }
    }
    /// The moment a lowered card touches the felt.
    fn touch_t(&self) -> f32 {
        self.dur * (0.5 + 0.5 * 0.62)
    }
}

struct CardObj {
    root: Entity,
    face: Entity,
    back: Entity,
    badge: Entity,
    glow: Entity,
    face_now: Option<Face>,
    face_shown: bool,
    badge_now: Option<(usize, i32)>,
    glow_now: Option<MaterialId>,
    pose: Pose,
    place: Place,
    flip_to: f32,
    anim: Option<Anim>,
    last_fx: f32,
    last_pulse: f32,
    last_die: f32,
    shattered: bool,
}

struct TileObj {
    root: Entity,
    front: Entity,
    back: Entity,
    glow: Entity,
    bars: [Entity; 2],
    totals: [Entity; 2],
    front_tex: Option<TextureId>,
    shown_rev: bool,
    flip: f32,
    flip_t: f32,
    totals_now: [Option<(usize, i32)>; 2],
}

struct Ball {
    e: Entity,
    from: usize,
    to: usize,
    t: f32,
    buff: bool,
    n: i32,
    kill: bool,
    hit: bool,
}

struct Shard {
    e: Entity,
    vel: Vec3,
    spin: Vec3,
    t: f32,
}

struct Brick {
    e: Entity,
    pos: Vec3,
    vel: Vec3,
    rot: Quat,
    spin: Vec3,
    delay: f32,
    rest: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Inspect {
    Hand(usize),
    Board(usize),
    Zone(usize),
}

/// What a press on the table started.
#[derive(Clone, Copy, Debug)]
enum Press {
    Card { ci: usize, from_hand: bool, start: Vec2, moved: bool },
    Other(Option<usize>, Option<usize>),
    Modal,
}

#[derive(Clone, Copy, Default, Debug)]
struct Lay {
    w: f32,
    h: f32,
    /// card width on the board
    bw: f32,
    portrait: bool,
    phone: bool,
    col: f32,
    tile_w: f32,
    row0: f32,
    step: f32,
    rows_end: f32,
    top_px: f32,
    bottom_px: f32,
    hand_px: f32,
    hand_base_px: f32,
    hand_avail_px: f32,
    orb_px: f32,
}

impl Lay {
    fn tile_d(&self) -> f32 {
        self.tile_w * TILE_H
    }
    fn zone_x(&self, z: usize) -> f32 {
        (z as f32 - 1.0) * self.col
    }
    fn slot(&self, z: usize, side: usize, k: usize) -> Vec3 {
        let (c, r) = ((k % 2) as f32, (k / 2) as f32);
        let x = self.zone_x(z) + (c - 0.5) * (self.bw + GAP);
        let zz = self.row0 + r * self.step;
        BASE + Vec3::new(x, self.bw * CARD_T * 0.5 + 0.008, if side == SIDE_P { zz } else { -zz })
    }
    fn pile(&self, side: usize) -> Vec3 {
        let (x, z) = if self.portrait { (1.5 * self.col - 0.32, self.rows_end + 0.62) } else { (1.5 * self.col + 0.62, self.row0 + 0.3) };
        BASE + if side == SIDE_P { Vec3::new(x, 0.0, z) } else { Vec3::new(-x, 0.0, -z) }
    }
    fn ai_hand(&self) -> Vec3 {
        BASE + Vec3::new(if self.portrait { 0.35 } else { 0.0 }, 0.3, -(self.rows_end + 0.62))
    }
}

/// The result handed to the play screen (studs, EXP, cards, the after-dialogue: `PlayScreen::battle_done`).
#[derive(Clone, Copy, Debug)]
pub struct BattleOut {
    pub kind: BattleKind,
    pub result: Outcome,
    pub stakes: i32,
}

pub struct BattleScreen {
    pub kind: BattleKind,
    pub seed: u32,
    b: Option<Battle>,
    cam: Option<Entity>,
    saved: Option<engine::renderer::RenderSettings>,
    /// the camera that was main before the battle (the village's): main again on exit
    prev_cam: Option<Entity>,
    cards: Vec<CardObj>,
    tiles: Vec<TileObj>,
    ghosts: Vec<(Entity, usize, usize, usize)>,
    backdrop: Option<Entity>,
    dim: Option<Entity>,
    insp_obj: Option<CardObj>,
    insp_tile: Option<Entity>,
    balls: Vec<Ball>,
    shards: Vec<Shard>,
    bricks: Vec<Brick>,
    fx_labels: Vec<(Entity, f32)>,
    lay: Lay,
    cam_target: Vec3,
    cam_dist: f32,
    pitch: f32,
    time: f32,
    /// the battle runs this much faster (tests)
    pub speed: f32,
    // ---- the player's input ----
    selected: Option<usize>,
    inspect: Option<Inspect>,
    press: Option<Press>,
    drag_over: Option<usize>,

    drag_tilt: f32,
    hover: Option<usize>,
    was_down: bool,
    last_pos: Vec2,
    touch_ui: bool,
    log_open: bool,
    // ---- around the battle (Kaigen CardGame) ----
    taunt: Option<(&'static str, f32)>,
    rng: u32,
    last_turn: i32,
    ai_snap_said: bool,
    celebrated: bool,
    confetti_t: f32,
    banner: Option<(String, String, f32)>,
    banner_seq: u32,
    last_phase: Option<Phase>,
    done: bool,
    hud: [String; 3],
    // ---- for tests ----
    lands: u32,
    flips: u32,
    shatters: u32,
    balls_fired: u32,
    plays: u32,
}

impl Default for BattleScreen {
    fn default() -> Self {
        Self::new(BattleKind::Practice, 1)
    }
}

/// Puts an entity on the table's layer or hides it.
fn show(ctx: &mut Ctx, e: Entity, on: bool) {
    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
        mr.layer = if on { LAYER } else { HIDDEN };
    }
}

fn spawn(ctx: &mut Ctx, name: &str, mesh: MeshId, mat: MaterialId, t: Transform, parent: Option<Entity>, shadows: bool) -> Entity {
    let e = ctx.spawn_mesh(name, mesh, mat, t);
    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
        mr.layer = LAYER;
        mr.cast_shadows = shadows;
    }
    if let Some(p) = parent {
        let _ = ctx.world.insert_one(e, Parent(p));
    }
    e
}

fn set_tf(ctx: &mut Ctx, e: Entity, t: Transform) {
    if let Ok(mut tr) = ctx.world.get::<&mut Transform>(e) {
        *tr = t;
    }
}

fn burst(ctx: &mut Ctx, pos: Vec3, preset: ParticlePreset, color: Vec4, n: u32, speed: f32, size: f32, gravity: f32) {
    let mut em = ParticleEmitter::preset(preset).burst(n);
    em.color_start = color;
    em.color_end = Vec4::new(color.x, color.y, color.z, 0.0);
    em.speed = (speed * 0.4, speed);
    em.size = (size * 0.5, size);
    em.lifetime = (0.35, 0.8);
    em.gravity = Vec3::Y * -gravity;
    em.spread = 1.0;
    em.direction = Vec3::Y;
    em.additive = true;
    ctx.spawn_particles("battle fx", pos, em);
}

fn kind_color(k: cards::Kind) -> Vec4 {
    let c = cards::kind_info(k).color;
    let f = |s: u32| ((c >> s) & 255) as f32 / 255.0;
    Vec4::new(f(16) * 2.4, f(8) * 2.4, f(0) * 2.4, 1.0)
}

impl BattleScreen {
    pub fn new(kind: BattleKind, seed: u32) -> Self {
        BattleScreen {
            kind,
            seed,
            b: None,
            cam: None,
            saved: None,
            prev_cam: None,
            cards: vec![],
            tiles: vec![],
            ghosts: vec![],
            backdrop: None,
            dim: None,
            insp_obj: None,
            insp_tile: None,
            balls: vec![],
            shards: vec![],
            bricks: vec![],
            fx_labels: vec![],
            lay: Lay::default(),
            cam_target: BASE,
            cam_dist: 8.0,
            pitch: 0.9,
            time: 0.0,
            speed: 1.0,
            selected: None,
            inspect: None,
            press: None,
            drag_over: None,

            drag_tilt: 0.0,
            hover: None,
            was_down: false,
            last_pos: Vec2::ZERO,
            touch_ui: false,
            log_open: false,
            taunt: None,
            rng: seed.wrapping_mul(2654435761) | 1,
            last_turn: 0,
            ai_snap_said: false,
            celebrated: false,
            confetti_t: -1.0,
            banner: None,
            banner_seq: 0,
            last_phase: None,
            done: false,
            hud: Default::default(),
            lands: 0,
            flips: 0,
            shatters: 0,
            balls_fired: 0,
            plays: 0,
        }
    }

    fn b(&self) -> &Battle {
        self.b.as_ref().unwrap()
    }

    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng >> 8
    }

    /// The opponent says something (Kaigen `cg_say`): the bubble and, for Dex and Lumi, the voice.
    fn say(&mut self, ctx: &mut Ctx, data: &mut Data, what: usize, secs: f32) {
        let r = self.rand();
        let l = battle::line(self.kind, what, r);
        self.taunt = Some((l.text, secs));
        data.sound.voice(ctx, l.voice);
    }

    fn banner(&mut self, title: impl Into<String>, sub: impl Into<String>) {
        self.banner = Some((title.into(), sub.into(), 1.5));
        self.banner_seq += 1;
    }

    // ---------------------------------------------------------------- setup

    fn build(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.art.ensure(ctx);
        let art = &data.art;
        let s = &data.save.data;
        // the battle (Kaigen cg_begin_board)
        let deck = match self.kind {
            BattleKind::Pip => cards::DECK_PIP,
            BattleKind::Rival if s.rival_wins > 0 => cards::DECK_RIVAL_2,
            BattleKind::Rival => cards::DECK_RIVAL,
            _ => cards::DECK_PRACTICE,
        };
        let opp = match self.kind {
            BattleKind::Pip => "Pip",
            BattleKind::Rival if s.rival_wins > 0 => "Dex \u{2605} Rematch",
            BattleKind::Rival => "Dex",
            BattleKind::Wild => "Wild Voxelmon",
            BattleKind::Practice => "Lumi",
        };
        let mut mine: Vec<usize> = s.deck.iter().map(|&c| c as usize).filter(|&c| c < cards::CARD_COUNT).collect();
        if mine.is_empty() {
            mine = cards::DECKS[cards::DECK_STARTER].to_vec();
        }
        let mut b = Battle::new(self.kind, opp, &mine, &cards::DECKS[deck], self.seed);
        if self.kind == BattleKind::Pip {
            // Pip's cards are all a little stronger: +1 Power each
            for c in &mut b.c {
                if c.owner == SIDE_AI {
                    c.modp += 1;
                }
            }
        }
        self.last_turn = b.turn;
        self.b = Some(b);

        // ---- the table: wood, felt, the backdrop ----
        let cube = ctx.renderer.builtin.cube;
        let quad = ctx.renderer.builtin.quad;
        let (wood, felt) = (art.wood.unwrap(), art.felt.unwrap());
                spawn(ctx, "table top", cube, wood, Transform::from_translation(BASE + Vec3::new(0.0, -0.105, 0.0)).with_scale(Vec3::new(8.6, 0.18, 7.6)), None, false);
        spawn(ctx, "table felt", quad, felt, Transform::from_translation(BASE).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)).with_scale(Vec3::new(7.9, 6.9, 1.0)), None, false);
        // the gold rim around the felt
        let rim = art.rim.unwrap();
        for (p, sc) in [
            (Vec3::new(0.0, 0.012, 3.47), Vec3::new(7.98, 0.024, 0.06)),
            (Vec3::new(0.0, 0.012, -3.47), Vec3::new(7.98, 0.024, 0.06)),
            (Vec3::new(3.97, 0.012, 0.0), Vec3::new(0.06, 0.024, 6.86)),
            (Vec3::new(-3.97, 0.012, 0.0), Vec3::new(0.06, 0.024, 6.86)),
        ] {
            spawn(ctx, "table rim", cube, rim, Transform::from_translation(BASE + p).with_scale(sc), None, false);
        }
        let bg = match self.kind {
            BattleKind::Rival => 1,
            BattleKind::Practice => 2,
            _ => 0,
        };
        if let Some(t) = data.art.backdrop_tex[bg] {
            let m = ctx.renderer.add_material(Material { texture: Some(t), unlit: true, ..Material::color(0x8a96a0) });
            self.backdrop = Some(spawn(ctx, "backdrop", quad, m, Transform::from_translation(BASE + Vec3::new(0.0, 3.0, -9.0)).with_scale(Vec3::new(28.0, 15.75, 1.0)), None, false));
        }

        // ---- the location tiles ----
        let (tf, tb, tr) = (data.art.tile_front.unwrap(), data.art.tile_back.unwrap(), data.art.tile_round.unwrap());
        let ghost = data.art.ghost.unwrap();
        let gold = data.art.glow_gold.unwrap();
        let red = data.art.glow_red.unwrap();
        let round = data.art.round.unwrap();
        let num = data.art.num_mat.unwrap();
        for z in 0..3 {
            let hid = data.art.hidden_tex[z].unwrap_or(TextureId(0));
            let hm = data.art.face_mat(ctx, hid);
            let tile = ctx.world.spawn((Transform::from_translation(BASE), Name(format!("tile {z}"))));
            let front = spawn(ctx, "tile front", tf, hm, Transform::default(), Some(tile), true);
            let back = spawn(ctx, "tile back", tb, hm, Transform::default(), Some(tile), true);
            let glow = spawn(ctx, "tile glow", tr, gold, Transform::default(), None, false);
            let bars = [spawn(ctx, "lead bar", cube, gold, Transform::default(), None, false), spawn(ctx, "lead bar", cube, red, Transform::default(), None, false)];
            let n0 = data.art.number(ctx, table::NUM_TOTAL, 0).unwrap();
            let totals = [spawn(ctx, "total", n0, num, Transform::default(), None, false), spawn(ctx, "total", n0, num, Transform::default(), None, false)];
            show(ctx, glow, false);
            show(ctx, bars[0], false);
            show(ctx, bars[1], false);
            self.tiles.push(TileObj { root: tile, front, back, glow, bars, totals, front_tex: None, shown_rev: false, flip: PI, flip_t: 9.0, totals_now: [None; 2] });
            for side in 0..2 {
                for k in 0..4 {
                    let g = spawn(ctx, "slot", round, ghost, Transform::default(), None, false);
                    self.ghosts.push((g, z, side, k));
                }
            }
        }

        // ---- every card: on its deck pile ----
        let n = self.b().c.len();
        for _ in 0..n {
            let o = self.new_card_obj(ctx, data);
            self.cards.push(o);
        }
        let dim = spawn(ctx, "dim", round, data.art.dim.unwrap(), Transform::default(), None, false);
        show(ctx, dim, false);
        self.dim = Some(dim);

        // ---- the camera ----
        let cam = ctx.world.spawn((Transform::from_translation(BASE + Vec3::new(0.0, 6.0, 6.0)).looking_at(BASE, Vec3::Y), Camera { fov_y_deg: FOV, near: 0.3, far: 60.0, layers: 1 << LAYER }, Name("battle camera".into())));
        self.cam = Some(cam);
        ctx.set_main_camera(cam);
    }

    fn new_card_obj(&mut self, ctx: &mut Ctx, data: &mut Data) -> CardObj {
        let art = &mut data.art;
        let back_mat = art.back_mat(ctx, &mut data.faces).unwrap_or(MaterialId(0));
        let (cf, cb, round) = (art.card_front.unwrap(), art.card_back.unwrap(), art.round.unwrap());
        let num = art.num_mat.unwrap();
        let n0 = art.number(ctx, table::NUM_BASE, 0).unwrap();
        let gold = art.glow_gold.unwrap();
        let root = ctx.world.spawn((Transform::from_translation(BASE), Name("card".into())));
        let face = spawn(ctx, "card face", cf, back_mat, Transform::default(), Some(root), true);
        let back = spawn(ctx, "card back", cb, back_mat, Transform::default(), Some(root), true);
        let badge = spawn(ctx, "power", n0, num, Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.0)), Some(root), false);
        let glow = spawn(ctx, "card glow", round, gold, Transform::from_xyz(0.0, 0.0, -CARD_T * 0.5 - 0.01).with_scale(Vec3::new(1.13, 1.1, 1.0)), Some(root), false);
        show(ctx, face, false);
        show(ctx, badge, false);
        show(ctx, glow, false);
        let pose = Pose { pos: self.lay.pile(SIDE_P), rot: Quat::from_rotation_x(-FRAC_PI_2), flip: PI, scale: self.lay.bw };
        CardObj {
            root,
            face,
            back,
            badge,
            glow,
            face_now: None,
            face_shown: false,
            badge_now: None,
            glow_now: Some(gold),
            pose,
            place: Place::Gone,
            flip_to: PI,
            anim: None,
            last_fx: 99.0,
            last_pulse: 99.0,
            last_die: 99.0,
            shattered: false,
        }
    }

    // ---------------------------------------------------------------- layout and camera

    fn layout(&mut self, ctx: &Ctx) {
        let w = ctx.canvas.client_width().max(1) as f32;
        let h = ctx.canvas.client_height().max(1) as f32;
        let portrait = h > w * 1.15;
        let phone = w <= 640.0;
        let col = if portrait { 1.2 } else { 1.6 };
        let tile_w = if portrait { 1.13 } else { 1.45 };
        let tile_d = tile_w * TILE_H;
        let bw = if portrait { 0.5 } else { 0.58 };
        let bh = bw * CARD_H;
        let row0 = tile_d * 0.5 + 0.14 + bh * 0.5;
        let step = bh + GAP;
        let rows_end = row0 + step + bh * 0.5;
        // the hand's card size (Kaigen battle_draw: a tall phone gets bigger cards)
        let mut hand_px = if phone { (w * 0.19).clamp(52.0, 96.0) } else { (w * 0.15).min(h * 0.17).clamp(58.0, 128.0) };
        if portrait && h / w > 1.7 {
            hand_px = ((w - 20.0) / 4.4).min(112.0);
        }
        let orb_px = if phone { 54.0 } else { (hand_px * 1.05).clamp(64.0, 104.0) };
        let hand_base_px = if phone { orb_px + 26.0 } else { 22.0 };
        let bottom_px = hand_base_px + hand_px * 1.4 + 14.0;
        let hand_avail_px = if phone { w - 16.0 } else { w - 2.0 * (orb_px.max(150.0) + 30.0) };
        self.lay = Lay { w, h, bw, portrait, phone, col, tile_w, row0, step, rows_end, top_px: 54.0, bottom_px, hand_px, hand_base_px, hand_avail_px, orb_px };
    }

    /// The camera that fits the board between the top bar and the hand (binary search on the distance, the aim
    /// moved so the board is centred in that band), with a slight sway.
    fn fit_camera(&mut self, ctx: &mut Ctx) {
        let l = self.lay;
        let pitch: f32 = if l.portrait { 63f32.to_radians() } else { 52f32.to_radians() };
        let xr = if l.portrait { 1.5 * l.col + 0.08 } else { 1.5 * l.col + 0.95 };
        let zf = l.rows_end + 1.0;
        let zn = if l.portrait { l.rows_end + 1.12 } else { l.rows_end + 0.12 };
        let pts = [
            Vec3::new(-xr, 0.0, zn),
            Vec3::new(xr, 0.0, zn),
            Vec3::new(-xr, 0.0, -zf),
            Vec3::new(xr, 0.0, -zf),
            Vec3::new(-xr, 0.45, -zf),
            Vec3::new(xr, 0.45, -zf),
        ];
        let aspect = l.w / l.h;
        let proj = Mat4::perspective_rh(FOV.to_radians(), aspect, 0.1, 100.0);
        let dir = Vec3::new(0.0, pitch.sin(), pitch.cos());
        let yt = 1.0 - 2.0 * l.top_px / l.h;
        let yb = -1.0 + 2.0 * l.bottom_px / l.h;
        let project = |d: f32, tz: f32| -> (f32, f32, f32) {
            let target = Vec3::new(0.0, 0.0, tz);
            let view = Mat4::look_at_rh(target + dir * d, target, Vec3::Y);
            let vp = proj * view;
            let (mut x, mut y0, mut y1) = (0.0f32, f32::MAX, f32::MIN);
            for p in pts {
                let c = vp * p.extend(1.0);
                let n = c.truncate() / c.w.max(0.001);
                x = x.max(n.x.abs());
                y0 = y0.min(n.y);
                y1 = y1.max(n.y);
            }
            (x, y0, y1)
        };
        // the aim (z) for a distance: the board's middle at the band's middle (secant steps)
        let centre = |d: f32| -> f32 {
            let want = (yt + yb) * 0.5;
            let mut tz = 0.0;
            for _ in 0..5 {
                let (_, a0, a1) = project(d, tz);
                let (_, b0, b1) = project(d, tz + 0.1);
                let (ca, cb) = ((a0 + a1) * 0.5, (b0 + b1) * 0.5);
                let slope = (cb - ca) / 0.1;
                if slope.abs() < 1e-4 {
                    break;
                }
                tz += (want - ca) / slope;
            }
            tz
        };
        let (mut lo, mut hi) = (2.0f32, 40.0f32);
        for _ in 0..24 {
            let d = (lo + hi) * 0.5;
            let tz = centre(d);
            let (x, y0, y1) = project(d, tz);
            if x <= 0.97 && y1 - y0 <= yt - yb {
                hi = d;
            } else {
                lo = d;
            }
        }
        self.cam_dist = hi;
        self.cam_target = BASE + Vec3::new(0.0, 0.0, centre(hi));
        self.pitch = pitch;
        let sway = Vec3::new((self.time * 0.31).sin() * 0.05, (self.time * 0.23).sin() * 0.03, 0.0);
        let pos = self.cam_target + dir * self.cam_dist + sway;
        if let Some(c) = self.cam {
            set_tf(ctx, c, Transform::from_translation(pos).looking_at(self.cam_target + sway * 0.3, Vec3::Y));
        }
    }

    fn cam_tf(&self, ctx: &Ctx) -> Transform {
        self.cam.and_then(|c| ctx.world.get::<&Transform>(c).ok().map(|t| *t)).unwrap_or_default()
    }

    /// World units per CSS pixel at depth `d` in front of the camera.
    fn px(&self, d: f32) -> f32 {
        2.0 * d * (FOV.to_radians() * 0.5).tan() / self.lay.h
    }

    /// A point `d` in front of the camera at CSS pixel (x, y).
    fn cam_point(&self, cam: &Transform, x: f32, y: f32, d: f32) -> Vec3 {
        let k = self.px(d);
        cam.translation + cam.rotation * Vec3::new((x - self.lay.w * 0.5) * k, (self.lay.h * 0.5 - y) * k, -d)
    }

    fn ray(&self, ctx: &Ctx, p: Vec2) -> Option<(Vec3, Vec3)> {
        ctx.screen_ray(p)
    }

    /// Where a ray meets the table plane at height y.
    fn on_plane(o: Vec3, d: Vec3, y: f32) -> Option<Vec3> {
        if d.y.abs() < 1e-5 {
            return None;
        }
        let t = (BASE.y + y - o.y) / d.y;
        (t > 0.0).then(|| o + d * t)
    }

    // ---------------------------------------------------------------- where each card belongs

    /// Hand cards of a side in the order shown (with the ones still waiting on the pile to fly in).
    fn hand_slots(&self, side: usize) -> Vec<(usize, bool)> {
        let b = self.b();
        let mut fresh = 0;
        let mut v = vec![];
        for &ci in &b.hand[side] {
            let c = &b.c[ci];
            let delay = 0.12 + 0.17 * fresh as f32;
            let mut waiting = false;
            if c.draw_t < delay + 0.52 {
                fresh += 1;
                waiting = c.draw_t < delay;
            }
            v.push((ci, waiting));
        }
        v
    }

    fn hand_pose(&self, cam: &Transform, i: usize, n: usize, ci: usize) -> Pose {
        let l = &self.lay;
        let k = self.px(HAND_D);
        let w = l.hand_px;
        let spacing = if n > 1 { (w * 1.04).min((l.hand_avail_px - w) / (n - 1) as f32) } else { 0.0 };
        let mid = (n as f32 - 1.0) * 0.5;
        let off = i as f32 - mid;
        let x = l.w * 0.5 + off * spacing;
        let mut y = l.h - l.hand_base_px - w * 0.7 + off * off * 1.2;
        let lifted = self.hover == Some(ci) || self.selected == Some(ci);
        if lifted {
            y -= w * 0.22;
        }
        let mut pos = self.cam_point(cam, x, y, HAND_D - i as f32 * 0.012 - if lifted { 0.06 } else { 0.0 });
        let fan = if n > 1 { -off * 0.045 * (spacing / w).min(1.0) } else { 0.0 };
        let scale = w * k * if lifted { 1.1 } else { 1.0 };
        let b = self.b();
        if !b.playable(ci) && b.phase == Phase::Plan {
            pos -= cam.rotation * Vec3::Y * (w * k * 0.06);
        }
        Pose { pos, rot: cam.rotation * Quat::from_rotation_z(fan), flip: 0.0, scale }
    }

    fn target(&self, ci: usize, cam: &Transform, hands: &[Vec<(usize, bool)>; 2], ptr: Option<(Vec3, Vec3)>) -> (Place, Pose) {
        let b = self.b();
        let c = &b.c[ci];
        let l = &self.lay;
        let flat = Quat::from_rotation_x(-FRAC_PI_2);
        let deck_pose = |side: usize, i: usize| -> Pose {
            let h = (i as f32 + 0.5) * l.bw * CARD_T * 1.2 + 0.003;
            let j = ((ci as u32).wrapping_mul(2654435761) >> 24) as f32 / 255.0 - 0.5;
            Pose { pos: l.pile(side) + Vec3::Y * h, rot: Quat::from_rotation_y(j * 0.12) * flat, flip: PI, scale: l.bw }
        };
        if !c.alive {
            return (Place::Gone, Pose { pos: self.cards[ci].pose.pos, rot: self.cards[ci].pose.rot, flip: self.cards[ci].pose.flip, scale: self.cards[ci].pose.scale });
        }
        if let (Some(Press::Card { ci: d, moved: true, .. }), Some((o, dir))) = (self.press, ptr) {
            if d == ci {
                let p = Self::on_plane(o, dir, DRAG_H).unwrap_or(o + dir * 3.0);
                let over_hand = self.drag_over == Some(3);
                let rot = Quat::from_rotation_z(-self.drag_tilt * 0.03) * Quat::from_rotation_x(-FRAC_PI_2 + 0.45);
                let scale = if over_hand { l.bw * 1.0 } else { l.bw * 1.12 };
                return (Place::Drag, Pose { pos: p, rot, flip: 0.0, scale });
            }
        }
        if c.zone >= 0 {
            let z = c.zone as usize;
            let k = b.zone_order(z, c.owner).iter().position(|&x| x == ci).unwrap_or(0);
            let face_down = !c.revealed && !(c.owner == SIDE_P && b.phase == Phase::Plan);
            return (Place::Board(z, k), Pose { pos: l.slot(z, c.owner, k), rot: flat, flip: if face_down { PI } else { 0.0 }, scale: l.bw });
        }
        let side = c.owner;
        if let Some(i) = hands[side].iter().position(|h| h.0 == ci) {
            let n = hands[side].len();
            if hands[side][i].1 {
                return (Place::Deck(b.deck[side].len()), deck_pose(side, b.deck[side].len()));
            }
            if side == SIDE_P {
                return (Place::Hand(i, n), self.hand_pose(cam, i, n, ci));
            }
            let off = i as f32 - (n as f32 - 1.0) * 0.5;
            let sp = (l.bw * 0.62).min(2.6 / n.max(1) as f32);
            let pos = l.ai_hand() + Vec3::new(off * sp, -off.abs() * 0.01, off.abs() * 0.02 - i as f32 * 0.002) + Vec3::Y * (i as f32 * 0.004);
            let rot = Quat::from_rotation_y(off * 0.06) * Quat::from_rotation_x(-FRAC_PI_2 + 0.5) * Quat::from_rotation_z(off * 0.05);
            return (Place::Hand(i, n), Pose { pos, rot, flip: PI, scale: l.bw * 0.9 });
        }
        if let Some(i) = b.deck[side].iter().position(|&d| d == ci) {
            let n = b.deck[side].len();
            return (Place::Deck(n - 1 - i), deck_pose(side, n - 1 - i));
        }
        (Place::Gone, self.cards[ci].pose)
    }

    // ---------------------------------------------------------------- input

    fn hit_card(&self, o: Vec3, d: Vec3, only: impl Fn(usize) -> bool) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for (ci, obj) in self.cards.iter().enumerate() {
            if !only(ci) || obj.place == Place::Gone {
                continue;
            }
            let q = obj.pose.quat();
            let n = q * Vec3::Z;
            let den = d.dot(n);
            if den.abs() < 1e-5 {
                continue;
            }
            let t = (obj.pose.pos - o).dot(n) / den;
            if t <= 0.0 {
                continue;
            }
            let local = q.inverse() * (o + d * t - obj.pose.pos) / obj.pose.scale;
            if local.x.abs() <= 0.5 && local.y.abs() <= CARD_H * 0.5 && best.map_or(true, |b| t < b.0) {
                best = Some((t, ci));
            }
        }
        best.map(|b| b.1)
    }

    /// The column under a table point (your slots or the tile; `any`: the whole column).
    fn zone_at(&self, p: Vec3, any: bool) -> Option<usize> {
        let l = &self.lay;
        let q = p - BASE;
        let z0 = if any { -(l.rows_end + 0.2) } else { -(l.tile_d() * 0.5 + 0.12) };
        if q.z < z0 || q.z > l.rows_end + 0.25 {
            return None;
        }
        (0..3).find(|&z| (q.x - l.zone_x(z)).abs() <= l.col * 0.5)
    }

    fn on_tile(&self, p: Vec3) -> bool {
        let q = p - BASE;
        q.z.abs() <= self.lay.tile_d() * 0.5 + 0.05
    }

    /// One code path for the UI, the touch taps and the `battle_play` command.
    fn play(&mut self, ctx: &mut Ctx, data: &mut Data, ci: usize, z: usize) -> bool {
        let b = self.b.as_mut().unwrap();
        if b.stage(ci, z) {
            self.selected = None;
            self.plays += 1;
            let _ = data;
            let _ = ctx;
            true
        } else {
            false
        }
    }

    fn end_turn(&mut self, ctx: &mut Ctx, data: &mut Data) {
        let b = self.b.as_mut().unwrap();
        if b.phase != Phase::Plan {
            return;
        }
        self.selected = None;
        self.inspect = None;
        b.end_turn();
        // the opponent's cards fly to their slots before the first one turns over
        b.wait = b.wait.max(0.95);
        let first = if b.first == SIDE_P { "You reveal first".to_string() } else { format!("{} reveals first", b.opponent) };
        data.sound.sfx(ctx, Sfx::Blip, 1.0, 0.4);
        self.banner(first, "");
    }

    fn pointer(&mut self, ctx: &mut Ctx) -> (bool, Vec2, Vec<Vec2>) {
        let t = &ctx.input.touch;
        if t.touch_device {
            self.touch_ui = true;
        }
        let mut quick = vec![];
        if let Some(f) = t.touches.first() {
            return (true, f.pos, quick);
        }
        if !self.was_down {
            quick.extend(t.taps.iter().copied());
        }
        let down = ctx.input.button(MouseButton::Left);
        if ctx.input.button_pressed(MouseButton::Left) && !down && !self.was_down {
            quick.push(ctx.input.pointer);
        }
        (down, ctx.input.pointer, quick)
    }

    fn modal(&self) -> bool {
        self.b().phase == Phase::Over
    }

    fn on_press(&mut self, ctx: &Ctx, p: Vec2) {
        if self.modal() {
            self.press = Some(Press::Modal);
            return;
        }
        if self.inspect.is_some() {
            self.press = Some(Press::Modal);
            return;
        }
        let Some((o, d)) = self.ray(ctx, p) else { return };
        let b = self.b();
        let plan = b.phase == Phase::Plan;
        let hit = self.hit_card(o, d, |ci| {
            let c = &b.c[ci];
            c.alive && (c.zone >= 0 || (c.owner == SIDE_P && b.hand[SIDE_P].contains(&ci)))
        });
        if let Some(ci) = hit {
            let c = &b.c[ci];
            if c.owner == SIDE_P && (c.zone < 0 || (c.staged && plan)) {
                self.press = Some(Press::Card { ci, from_hand: c.zone < 0, start: p, moved: false });
                return;
            }
            self.press = Some(Press::Other(Some(ci), None));
            return;
        }
        let z = Self::on_plane(o, d, 0.0).and_then(|q| self.zone_at(q, true));
        self.press = Some(Press::Other(None, z));
    }

    fn on_move(&mut self, ctx: &mut Ctx, data: &mut Data, p: Vec2) {
        let Some(Press::Card { ci, from_hand, start, moved }) = self.press else { return };
        let b = self.b();
        let can_drag = (!from_hand || b.playable(ci)) && b.phase == Phase::Plan;
        if !moved && can_drag && (p - start).length() > 8.0 {
            self.press = Some(Press::Card { ci, from_hand, start, moved: true });
            if from_hand {
                self.selected = Some(ci);
            }
            data.sound.sfx(ctx, Sfx::Blip, 1.3, 0.3);
        }
        if let Some(Press::Card { moved: true, .. }) = self.press {
            let vx = p.x - self.last_pos.x;
            self.drag_tilt = (self.drag_tilt * 0.75 + vx * 0.6).clamp(-14.0, 14.0);
            self.drag_over = None;
            if p.y >= self.lay.h - self.lay.bottom_px {
                self.drag_over = Some(3);
            } else if let Some((o, d)) = self.ray(ctx, p) {
                self.drag_over = Self::on_plane(o, d, 0.0).and_then(|q| self.zone_at(q, false));
            }
        }
    }

    fn on_release(&mut self, ctx: &mut Ctx, data: &mut Data, p: Vec2) {
        let press = self.press.take();
        let over = self.drag_over.take();
        log_info!("battle release at {p:?}: {press:?} over {over:?}");
        match press {
            None => {}
            Some(Press::Modal) => {
                // a tap while the big card is open: on a location you can play it to, play it there; else close
                if let Some(Inspect::Hand(ci)) = self.inspect {
                    if let Some((o, d)) = self.ray(ctx, p) {
                        if let Some(z) = Self::on_plane(o, d, 0.0).and_then(|q| self.zone_at(q, true)) {
                            if self.b().can_play(ci, z) && p.y < self.lay.h - self.lay.bottom_px && !self.over_inspect(p) {
                                self.play(ctx, data, ci, z);
                                self.inspect = None;
                                return;
                            }
                        }
                    }
                }
                if self.inspect.is_some() && !self.over_inspect(p) {
                    self.inspect = None;
                }
            }
            Some(Press::Card { ci, from_hand, moved: false, .. }) => {
                // a tap: the card big (a hand card is also selected)
                self.inspect = Some(if from_hand { Inspect::Hand(ci) } else { Inspect::Board(ci) });
                if from_hand {
                    self.selected = if self.b().phase == Phase::Plan { Some(ci) } else { None };
                }
                data.sound.sfx(ctx, Sfx::Blip, 1.2, 0.35);
            }
            Some(Press::Card { ci, from_hand, moved: true, .. }) => {
                let b = self.b.as_mut().unwrap();
                if from_hand {
                    if let Some(t) = over.filter(|&t| t < 3) {
                        self.play(ctx, data, ci, t);
                    }
                    self.selected = None;
                } else {
                    let orig = b.c[ci].zone as usize;
                    match over {
                        Some(t) if t < 3 && t != orig => {
                            b.unstage(ci);
                            if !b.stage(ci, t) {
                                b.stage(ci, orig); // not allowed there: back where it was
                            }
                        }
                        Some(3) => b.unstage(ci),
                        _ => {}
                    }
                }
            }
            Some(Press::Other(card, zone)) => {
                let b = self.b();
                if let Some(ci) = card {
                    let c = &b.c[ci];
                    if let (true, Some(sel)) = (c.owner == SIDE_P && c.zone >= 0, self.selected) {
                        if b.can_play(sel, c.zone as usize) {
                            let z = c.zone as usize;
                            self.play(ctx, data, sel, z);
                            return;
                        }
                    }
                    if c.revealed || c.owner == SIDE_P {
                        self.inspect = Some(Inspect::Board(ci));
                        data.sound.sfx(ctx, Sfx::Blip, 1.2, 0.35);
                    }
                } else if let Some(z) = zone {
                    if let Some(sel) = self.selected.filter(|&s| b.can_play(s, z)) {
                        self.play(ctx, data, sel, z);
                    } else if let Some((o, d)) = self.ray(ctx, p) {
                        if Self::on_plane(o, d, 0.0).is_some_and(|q| self.on_tile(q)) {
                            self.inspect = Some(Inspect::Zone(z));
                            data.sound.sfx(ctx, Sfx::Blip, 1.2, 0.35);
                        } else {
                            self.selected = None;
                        }
                    }
                } else {
                    self.selected = None;
                }
            }
        }
    }

    /// The pointer is on the big card (taps there do not close it).
    fn over_inspect(&self, p: Vec2) -> bool {
        let (cx, cy, w, h) = self.inspect_rect();
        (p.x - cx).abs() <= w * 0.5 && (p.y - cy).abs() <= h * 0.5
    }

    /// The big card's centre and size in CSS px.
    fn inspect_rect(&self) -> (f32, f32, f32, f32) {
        let l = &self.lay;
        let zone = matches!(self.inspect, Some(Inspect::Zone(_)));
        if zone {
            let w = (l.w * 0.88).min(420.0);
            let h = w * TILE_H;
            return (l.w * 0.5, l.top_px + 30.0 + h * 0.5, w, h);
        }
        let h = (l.h * 0.5).min(l.w * 0.66 * 1.4).min(470.0);
        let w = h / 1.4;
        (l.w * 0.5, l.top_px + 14.0 + h * 0.5, w, h)
    }

    fn input(&mut self, ctx: &mut Ctx, data: &mut Data, actions: &[String]) {
        let (down, pos, quick) = self.pointer(ctx);
        let busy = data.trans.is_some();
        if !busy {
            if down && !self.was_down {
                self.on_press(ctx, pos);
            }
            if down {
                self.on_move(ctx, data, pos);
            }
            if !down && self.was_down {
                let p = self.last_pos;
                self.on_release(ctx, data, p);
            }
            for q in quick {
                self.on_press(ctx, q);
                self.on_release(ctx, data, q);
            }
        }
        self.was_down = down;
        if down {
            self.last_pos = pos;
        }
        // hover (mouse): the hand card under the pointer lifts
        self.hover = None;
        if !down && !self.touch_ui && self.inspect.is_none() && !busy {
            if let Some((o, d)) = self.ray(ctx, ctx.input.pointer) {
                let b = self.b();
                self.hover = self.hit_card(o, d, |ci| b.hand[SIDE_P].contains(&ci));
            }
        }
        // ---- buttons and keys ----
        let phase = self.b().phase;
        for a in actions {
            let a = a.as_str();
            match a {
                "bt_end" | "bt_suggest" => self.end_turn(ctx, data),
                "bt_undo" => {
                    let b = self.b.as_mut().unwrap();
                    for i in 0..b.c.len() {
                        if b.c[i].staged {
                            b.unstage(i);
                        }
                    }
                }
                "bt_snap" => {
                    self.b.as_mut().unwrap().snap();
                    data.sound.sfx(ctx, Sfx::Alert, 1.0, 0.6);
                }
                "bt_retreat" => self.b.as_mut().unwrap().retreat(),
                "bt_log" => self.log_open = !self.log_open,
                "bt_close" => self.inspect = None,
                "bt_take_back" => {
                    if let Some(Inspect::Board(ci)) = self.inspect {
                        self.b.as_mut().unwrap().unstage(ci);
                    }
                    self.inspect = None;
                }
                "bt_activate" => {
                    if let Some(Inspect::Board(ci)) = self.inspect {
                        self.b.as_mut().unwrap().activate(ci);
                        data.sound.sfx(ctx, Sfx::Alert, 1.3, 0.4);
                    }
                    self.inspect = None;
                }
                "bt_cancel_act" => {
                    if let Some(Inspect::Board(ci)) = self.inspect {
                        self.b.as_mut().unwrap().cancel_activate(ci);
                    }
                    self.inspect = None;
                }
                "bt_continue" => {
                    if phase == Phase::Over {
                        self.done = true;
                    }
                }
                _ => {
                    if let Some(z) = a.strip_prefix("bt_play:").and_then(|z| z.parse::<usize>().ok()) {
                        if let Some(Inspect::Hand(ci)) = self.inspect {
                            self.play(ctx, data, ci, z);
                        }
                        self.inspect = None;
                    }
                }
            }
        }
        if ctx.input.key_pressed("Escape") {
            if self.inspect.is_some() {
                self.inspect = None;
            } else if self.log_open {
                self.log_open = false;
            } else {
                self.selected = None;
            }
        }
        if ctx.input.key_pressed("Enter") {
            if phase == Phase::Over {
                self.done = true;
            } else if self.inspect.is_none() && self.press.is_none() {
                self.end_turn(ctx, data);
            }
        }
        if self.b().phase != Phase::Plan {
            if matches!(self.press, Some(Press::Card { .. })) {
                self.press = None;
            }
            self.selected = None;
            if matches!(self.inspect, Some(Inspect::Hand(_))) {
                self.inspect = None;
            }
        }
    }

    // ---------------------------------------------------------------- the battle going on (Kaigen cg_update)

    fn run(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        let b = self.b.as_mut().unwrap();
        let before_turn = b.turn;
        b.update(dt);
        let (reveals, hits, buffs, kills) = (b.sfx_reveals, b.sfx_hits, b.sfx_buffs, b.sfx_kills);
        b.sfx_reveals = 0;
        b.sfx_hits = 0;
        b.sfx_buffs = 0;
        b.sfx_kills = 0;
        let snd = &mut data.sound;
        if reveals > 0 {
            snd.sfx(ctx, Sfx::Pop, 1.1, 0.45);
        }
        if hits > 0 {
            snd.sfx(ctx, Sfx::Kick, 1.15, 0.7);
        }
        if buffs > 0 {
            snd.sfx(ctx, Sfx::Coin, 1.25, 0.4);
        }
        if kills > 0 {
            snd.sfx(ctx, Sfx::Splash, 1.6, 0.6);
            snd.sfx(ctx, Sfx::Kick, 0.7, 0.8);
        }
        let b = self.b();
        let (turn, max, phase, result, ai_snapped) = (b.turn, b.max_turns, b.phase, b.result, b.ai_snapped);
        if turn > self.last_turn {
            self.last_turn = turn;
            if turn < max && self.rand() % 100 < 45 {
                self.say(ctx, data, 1, 2.6);
            }
            data.sound.sfx(ctx, Sfx::Blip, 0.9, 0.4);
            let b = self.b();
            let sub = if turn <= 3 {
                format!("Location revealed: {}", cards::ZONES[b.zone[(turn - 1) as usize]].name)
            } else if turn == max {
                "Final turn! Stakes doubled.".to_string()
            } else {
                String::new()
            };
            let _ = before_turn;
            self.banner(format!("Turn {turn}"), sub);
        }
        if ai_snapped && !self.ai_snap_said {
            self.ai_snap_said = true;
            self.say(ctx, data, 2, 3.2);
            data.sound.sfx(ctx, Sfx::Alert, 1.0, 0.6);
            let opp = self.b().opponent.clone();
            self.banner(format!("{opp} SNAPPED!"), "Stakes doubled");
        }
        if phase == Phase::Over && !self.celebrated {
            self.celebrated = true;
            self.banner = None;
            self.inspect = None;
            self.press = None;
            match result {
                Outcome::Win => {
                    data.sound.music(ctx, Track::Victory);
                    self.confetti_t = 0.0;
                    self.spawn_confetti(ctx, data);
                    data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8);
                    self.say(ctx, data, 4, 4.0);
                }
                Outcome::Lose => {
                    data.sound.music(ctx, Track::Defeat);
                    self.say(ctx, data, 3, 4.0);
                }
                _ => {}
            }
        }
        if phase == Phase::Over && result == Outcome::Retreat {
            self.done = true;
        }
        self.last_phase = Some(phase);
    }

    // ---------------------------------------------------------------- drawing the table

    fn sync_cards(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        // bricklings made during the battle get their objects
        while self.cards.len() < self.b().c.len() {
            let mut o = self.new_card_obj(ctx, data);
            let side = self.b().c[self.cards.len()].owner;
            o.pose.pos = if side == SIDE_P { self.lay.pile(SIDE_P) } else { self.lay.ai_hand() };
            self.cards.push(o);
        }
        let cam = self.cam_tf(ctx);
        let hands = [self.hand_slots(SIDE_P), self.hand_slots(SIDE_AI)];
        let ptr = if matches!(self.press, Some(Press::Card { moved: true, .. })) { self.ray(ctx, self.last_pos) } else { None };
        // attackers lunge at their targets (Kaigen: 22 px), the hit card shakes, a buffed one hops
        let n = self.cards.len();
        let mut offset = vec![Vec3::ZERO; n];
        {
            let b = self.b();
            for j in 0..n {
                let t = &b.c[j];
                let src = t.fx_src;
                let q = t.fx_t.min(t.die_t) / 0.42;
                if src >= 0 && src as usize != j && q < 1.0 {
                    let i = src as usize;
                    let d = (self.cards[j].pose.pos - self.cards[i].pose.pos).normalize_or_zero();
                    let a = if q < 0.25 {
                        -0.3 * q / 0.25
                    } else if q < 0.55 {
                        -0.3 + 1.3 * (q - 0.25) / 0.3
                    } else {
                        1.0 - (q - 0.55) / 0.45
                    };
                    offset[i] += d * 0.16 * a + Vec3::Y * 0.1 * a.max(0.0);
                }
                if t.fx_t > 0.44 && t.fx_t < 0.9 && t.fx_n != 0 {
                    let r = (t.fx_t - 0.44) / 0.46;
                    if t.fx_n < 0 {
                        offset[j] += Vec3::X * (r * 70.0).sin() * 0.035 * (1.0 - r);
                    } else {
                        offset[j] += Vec3::Y * 0.12 * (PI * r).sin();
                    }
                }
            }
        }
        for ci in 0..n {
            let (place, mut to) = self.target(ci, &cam, &hands, ptr);
            let b = self.b();
            let c = b.c[ci].clone();
            let o = &mut self.cards[ci];
            if place != o.place || (to.flip - o.flip_to).abs() > 1.0 {
                let style = match (o.place, place) {
                    (_, Place::Board(..)) if !matches!(o.place, Place::Board(..)) => Style::Lower,
                    (Place::Deck(_), Place::Hand(..)) => Style::Draw,
                    (Place::Board(..), Place::Board(..)) if (to.flip - o.flip_to).abs() > 1.0 => Style::Flip,
                    _ => Style::Plain,
                };
                let dur = match style {
                    Style::Lower => 0.62,
                    Style::Flip => 0.5,
                    Style::Draw => 0.52,
                    Style::Plain => 0.28,
                };
                if style == Style::Flip && to.flip < 1.0 {
                    self.flips += 1;
                }
                if o.place != Place::Gone && place != Place::Drag {
                    o.anim = Some(Anim { from: o.pose, t: 0.0, dur, style, landed: false });
                } else if place != Place::Drag {
                    o.pose = to;
                }
                o.place = place;
                o.flip_to = to.flip;
            }
            // the pulse of a reveal or an activate (a bump), with a burst of its kind's colour
            let pulse = c.pulse_t;
            if pulse < o.last_pulse - 0.01 && c.zone >= 0 {
                let col = kind_color(cards::card(c.id).kind);
                let at = o.pose.pos + Vec3::Y * 0.08;
                burst(ctx, at, ParticlePreset::Magic, col, 26, 1.6, 0.09, 1.5);
            }
            o.last_pulse = pulse;
            let bump = if pulse < 0.5 && c.zone >= 0 { 1.0 + 0.16 * (PI * pulse / 0.5).sin() } else { 1.0 };
            // the new pose
            let mut landed = false;
            if let Some(a) = &mut o.anim {
                let before = a.t;
                a.t += dt;
                if a.style == Style::Lower && !a.landed && before < a.touch_t() && a.t >= a.touch_t() {
                    a.landed = true;
                    landed = true;
                }
                o.pose = a.eval(&to);
                if a.t >= a.dur {
                    o.anim = None;
                }
            } else if place == Place::Drag {
                let k = 1.0 - (-dt * 28.0).exp();
                o.pose = Pose::lerp(&o.pose, &to, k);
            } else if place != Place::Gone {
                let k = 1.0 - (-dt * 14.0).exp();
                o.pose = Pose::lerp(&o.pose, &to, k);
            }
            to.pos += offset[ci];
            let mut shown = o.pose;
            shown.pos += offset[ci];
            shown.scale *= bump;
            // a destroyed card: frozen until the ball hits, then six shards fly
            let dead = !c.alive;
            if dead && c.die_t < o.last_die - 0.01 {
                o.shattered = false;
            }
            o.last_die = c.die_t;
            let gone = dead && c.die_t >= 0.56;
            let cast = !matches!(place, Place::Hand(..)) || c.owner == SIDE_AI;
            for e in [o.face, o.back] {
                if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
                    mr.cast_shadows = cast;
                }
            }
            set_tf(ctx, o.root, Transform::from_translation(shown.pos).with_rotation(shown.quat()).with_scale(Vec3::splat(shown.scale)));
            if landed {
                self.lands += 1;
                data.sound.sfx(ctx, Sfx::Pop, 0.8, 0.6);
                burst(ctx, shown.pos + Vec3::Y * 0.02, ParticlePreset::Dust, Vec4::new(1.0, 0.95, 0.8, 0.7), 14, 0.7, 0.07, 0.3);
            }
            if gone && !self.cards[ci].shattered {
                self.cards[ci].shattered = true;
                self.shatter(ctx, data, ci);
            }
            self.style_card(ctx, data, ci, &c, gone);
        }
    }

    /// The face texture, the power badge and the glow of a card.
    fn style_card(&mut self, ctx: &mut Ctx, data: &mut Data, ci: usize, c: &battle::BCard, gone: bool) {
        let b = self.b.as_ref().unwrap();
        let plan = b.phase == Phase::Plan;
        let level = if c.owner == SIDE_P { cards::level(&data.save.data, c.id) } else { 0 };
        let mut face = Face::new(c.id, level);
        if c.zone >= 0 {
            face.hide_cost = true;
        }
        let o = &self.cards[ci];
        let face_up = o.pose.flip < FRAC_PI_2 + 0.2 || o.flip_to < 1.0;
        let show_face = !gone && face_up && !(c.owner == SIDE_AI && !c.revealed && c.zone >= 0 && o.anim.is_none() && o.flip_to > 1.0);
        let show_face = show_face && !(c.owner == SIDE_AI && c.zone < 0);
        let power = if c.zone >= 0 { b.power(ci) } else { cards::card(c.id).power + c.modp };
        let base = cards::card(c.id).power;
        let badge = if show_face && power != base && (c.revealed || c.zone < 0 || plan) {
            Some((if power > base { table::NUM_UP } else { table::NUM_DOWN }, power))
        } else {
            None
        };
        let glow = if gone || !show_face {
            None
        } else if self.selected == Some(ci) && plan {
            data.art.glow_gold
        } else if b.can_activate(ci) && c.owner == SIDE_P && plan {
            data.art.glow_green
        } else if c.owner == SIDE_P && plan && (c.staged || c.act_order >= 0) {
            data.art.glow_white
        } else if c.zone < 0 && c.owner == SIDE_P && b.playable(ci) && self.hover == Some(ci) {
            data.art.glow_gold
        } else {
            None
        };
        // apply
        if self.cards[ci].face_now != Some(face) && show_face {
            if let Some(t) = data.faces.texture(ctx, face) {
                let m = data.art.face_mat(ctx, t);
                if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(self.cards[ci].face) {
                    mr.material = m;
                }
                self.cards[ci].face_now = Some(face);
            }
        }
        let o = &mut self.cards[ci];
        if o.face_shown != show_face {
            o.face_shown = show_face;
            show(ctx, o.face, show_face);
        }
        let (badge_e, back_e, glow_e) = (o.badge, o.back, o.glow);
        show(ctx, back_e, !gone);
        if self.cards[ci].badge_now != badge {
            self.cards[ci].badge_now = badge;
            match badge {
                Some((st, v)) => {
                    if let Some(m) = data.art.number(ctx, st, v) {
                        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(badge_e) {
                            mr.mesh = m;
                        }
                    }
                    show(ctx, badge_e, true);
                    set_tf(ctx, badge_e, Transform::from_xyz(0.3315, 0.5315, CARD_T * 0.5 + 0.02).with_scale(Vec3::splat(0.245)));
                }
                None => {
                    show(ctx, badge_e, false);
                    set_tf(ctx, badge_e, Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.0)));
                }
            }
        }
        let o = &mut self.cards[ci];
        match glow {
            Some(m) => {
                if o.glow_now != Some(m) {
                    o.glow_now = Some(m);
                    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(glow_e) {
                        mr.material = m;
                    }
                }
                show(ctx, glow_e, true);
                let a = 0.65 + 0.35 * (self.time * 5.0).sin();
                if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(glow_e) {
                    mr.opacity = a;
                }
            }
            None => show(ctx, glow_e, false),
        }
    }

    fn shatter(&mut self, ctx: &mut Ctx, data: &mut Data, ci: usize) {
        self.shatters += 1;
        let o = &self.cards[ci];
        let (pos, q, s) = (o.pose.pos, o.pose.quat(), o.pose.scale);
        let mat = o.face_now.and_then(|f| data.faces.texture(ctx, f)).map(|t| data.art.face_mat(ctx, t));
        let Some(mat) = mat else { return };
        for (k, (cx, cy)) in table::SHARDS.iter().enumerate() {
            let mesh = data.art.shards[k];
            let local = Vec3::new(*cx, *cy, 0.02);
            let p = pos + q * (local * s);
            let h = |m: u32| ((((ci as u32) * 31 + k as u32 * 7 + m).wrapping_mul(2654435761)) >> 24) as f32 / 255.0;
            let out = (q * Vec3::new(*cx, *cy, 0.0)).normalize_or_zero();
            let vel = out * (0.9 + 1.2 * h(1)) + Vec3::Y * (1.6 + 1.2 * h(2));
            let spin = Vec3::new(h(3) - 0.5, h(4) - 0.5, h(5) - 0.5) * 18.0;
            let e = spawn(ctx, "shard", mesh, mat, Transform::from_translation(p).with_rotation(q).with_scale(Vec3::splat(s)), None, true);
            self.shards.push(Shard { e, vel, spin, t: 0.0 });
        }
        burst(ctx, pos + Vec3::Y * 0.1, ParticlePreset::Sparks, Vec4::new(3.0, 1.2, 0.4, 1.0), 40, 3.0, 0.06, 4.0);
    }

    fn sync_tiles(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        let l = self.lay;
        let b = self.b.as_ref().unwrap();
        // where the dragged / selected card may go
        let card = match self.press {
            Some(Press::Card { ci, moved: true, .. }) => Some(ci),
            _ => self.selected,
        };
        let dragging_board = matches!(self.press, Some(Press::Card { moved: true, from_hand: false, .. }));
        let mut fresh_tex = vec![];
        for z in 0..3 {
            let rev = b.zone_rev[z];
            let zid = b.zone[z];
            let tex = data.art.zone_tex[zid];
            fresh_tex.push((z, rev, tex));
        }
        for (z, rev, tex) in fresh_tex {
            let t = &mut self.tiles[z];
            if t.front_tex != tex {
                if let Some(tx) = tex {
                    t.front_tex = Some(tx);
                    let m = data.art.face_mat(ctx, tx);
                    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(t.front) {
                        mr.material = m;
                    }
                }
            }
            if rev != t.shown_rev && (tex.is_some() || t.flip_t > 3.0) {
                t.shown_rev = rev;
                t.flip_t = 0.0;
            }
            t.flip_t += dt;
            let want = if t.shown_rev { 0.0 } else { PI };
            let q = (t.flip_t / 0.7).clamp(0.0, 1.0);
            t.flip = if t.shown_rev { PI * (1.0 - ease_in_out(q)) } else { want };
            let lift = if t.flip_t < 0.7 && t.shown_rev { 0.45 * (PI * q).sin() } else { 0.0 };
            let x = l.zone_x(z);
            let pos = BASE + Vec3::new(x, l.tile_w * TILE_T * 0.5 + 0.012 + lift, 0.0);
            let rot = Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_x(t.flip);
            let tr = Transform::from_translation(pos).with_rotation(rot).with_scale(Vec3::splat(l.tile_w));
            let (root, back, glow, bars, totals) = (t.root, t.back, t.glow, t.bars, t.totals);
            set_tf(ctx, root, tr);
            // the hidden side shows the "?" art (unrevealed) - the back mesh uses the hidden texture
            let _ = back;
            // the glow: where the card can go (gold), the column under the dragged card (white)
            let ok = card.is_some_and(|ci| {
                if dragging_board {
                    b.c[ci].zone != z as i32 && b.zone_open(z, SIDE_P)
                } else {
                    b.can_play(ci, z)
                }
            });
            let over = self.drag_over == Some(z);
            let gm = if over { data.art.glow_white } else { data.art.glow_gold };
            if let (true, Some(m)) = (ok || over, gm) {
                if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(glow) {
                    mr.material = m;
                    mr.layer = LAYER;
                    mr.opacity = 0.55 + 0.4 * (self.time * 6.0).sin().abs();
                }
                let s = if over { 1.12 } else { 1.08 };
                set_tf(ctx, glow, Transform::from_translation(BASE + Vec3::new(x, 0.01, 0.0)).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)).with_scale(Vec3::new(l.tile_w * s, l.tile_w * (s + 0.06), 1.0)));
            } else {
                show(ctx, glow, false);
            }
            // the leading side's edge glows, the totals lie at the tile's edges
            let win = b.zone_winner(z);
            let d = l.tile_d();
            for side in 0..2 {
                let zz = if side == SIDE_P { d * 0.5 + 0.035 } else { -(d * 0.5 + 0.035) };
                let lead = win == Some(side);
                show(ctx, bars[side], lead);
                if lead {
                    let pulse = 1.0 + 0.08 * (self.time * 4.0).sin();
                    set_tf(ctx, bars[side], Transform::from_translation(BASE + Vec3::new(x, 0.014, zz)).with_scale(Vec3::new(l.tile_w * 0.94 * pulse, 0.02, 0.035)));
                }
                let pw = b.zone_power(z, side);
                let style = if lead {
                    if side == SIDE_P {
                        table::NUM_LEAD
                    } else {
                        table::NUM_FOE
                    }
                } else {
                    table::NUM_TOTAL
                };
                let tz = if side == SIDE_P { d * 0.5 + 0.1 } else { -(d * 0.5 + 0.1) };
                let size = if l.portrait { 0.34 } else { 0.38 };
                let top = l.tile_w * TILE_T + 0.03 + lift;
                if self.tiles[z].totals_now[side] != Some((style, pw)) {
                    self.tiles[z].totals_now[side] = Some((style, pw));
                    if let Some(m) = data.art.number(ctx, style, pw) {
                        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(totals[side]) {
                            mr.mesh = m;
                        }
                    }
                }
                let s = if lead { size * (1.0 + 0.06 * (self.time * 4.0).sin()) } else { size };
                set_tf(ctx, totals[side], Transform::from_translation(BASE + Vec3::new(x, top, tz)).with_rotation(Quat::from_rotation_x(-FRAC_PI_2 + 0.35)).with_scale(Vec3::splat(s)));
                show(ctx, totals[side], true);
            }
        }
        // the empty slots, faint
        let ghosts = self.ghosts.clone();
        for (e, z, side, k) in ghosts {
            let cap = b.capacity(z);
            let used = b.zone_order(z, side).len();
            let vis = k < cap && k >= used;
            show(ctx, e, vis);
            if vis {
                set_tf(ctx, e, Transform::from_translation(l.slot(z, side, k) - Vec3::Y * (l.bw * CARD_T * 0.5 + 0.0)).with_rotation(Quat::from_rotation_x(-FRAC_PI_2)).with_scale(Vec3::splat(l.bw * 0.98)));
            }
        }
    }

    /// The big card (or location) in front of the camera, the table dimmed behind it.
    fn sync_inspect(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        let cam = self.cam_tf(ctx);
        let want = self.inspect;
        if let Some(d) = self.dim {
            show(ctx, d, want.is_some());
            if want.is_some() {
                let k = self.px(DIM_D);
                set_tf(ctx, d, Transform::from_translation(cam.translation + cam.rotation * Vec3::new(0.0, 0.0, -DIM_D)).with_rotation(cam.rotation).with_scale(Vec3::splat(self.lay.w.max(self.lay.h) * k * 1.2)));
            }
        }
        let (cx, cy, w, _) = self.inspect_rect();
        let k = self.px(INSPECT_D);
        let centre = self.cam_point(&cam, cx, cy, INSPECT_D);
        match want {
            Some(Inspect::Zone(z)) => {
                if let Some(o) = self.insp_obj.take() {
                    ctx.despawn(o.root);
                }
                if self.insp_tile.is_none() {
                    let zid = self.b().zone[z];
                    let tex = if self.b().zone_rev[z] { data.art.zone_tex[zid] } else { data.art.hidden_tex[z] };
                    if let (Some(t), Some(mesh)) = (tex, data.art.tile_front) {
                        let m = data.art.face_mat(ctx, t);
                        self.insp_tile = Some(spawn(ctx, "inspect tile", mesh, m, Transform::default(), None, false));
                    }
                }
                if let Some(e) = self.insp_tile {
                    set_tf(ctx, e, Transform::from_translation(centre).with_rotation(cam.rotation).with_scale(Vec3::splat(w * k)));
                }
            }
            Some(Inspect::Hand(ci)) | Some(Inspect::Board(ci)) => {
                if let Some(e) = self.insp_tile.take() {
                    ctx.despawn(e);
                }
                let c = self.b().c[ci].clone();
                let level = if c.owner == SIDE_P { cards::level(&data.save.data, c.id) } else { 0 };
                let face = Face::new(c.id, level).big();
                let fresh = self.insp_obj.as_ref().is_none_or(|o| o.face_now != Some(face));
                if fresh {
                    if let Some(o) = self.insp_obj.take() {
                        ctx.despawn(o.root);
                    }
                    let mut o = self.new_card_obj(ctx, data);
                    if let Some(t) = data.faces.texture(ctx, face) {
                        let m = data.art.face_mat(ctx, t);
                        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(o.face) {
                            mr.material = m;
                        }
                        show(ctx, o.face, true);
                    }
                    o.face_now = Some(face);
                    for e in [o.face, o.back, o.badge] {
                        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
                            mr.cast_shadows = false;
                        }
                    }
                    o.pose = self.cards[ci].pose;
                    o.anim = Some(Anim { from: o.pose, t: 0.0, dur: 0.3, style: Style::Plain, landed: false });
                    self.insp_obj = Some(o);
                }
                // the battle power on the big card
                let b = self.b();
                let power = if c.zone >= 0 { b.power(ci) } else { cards::card(c.id).power + c.modp };
                let base = cards::card(c.id).power;
                let badge = (power != base).then(|| (if power > base { table::NUM_UP } else { table::NUM_DOWN }, power));
                let to = Pose { pos: centre, rot: cam.rotation, flip: 0.0, scale: w * k };
                let o = self.insp_obj.as_mut().unwrap();
                if let Some(a) = &mut o.anim {
                    a.t += dt;
                    o.pose = a.eval(&to);
                    if a.t >= a.dur {
                        o.anim = None;
                    }
                } else {
                    o.pose = to;
                }
                let (root, be, pose) = (o.root, o.badge, o.pose);
                set_tf(ctx, root, Transform::from_translation(pose.pos).with_rotation(pose.quat()).with_scale(Vec3::splat(pose.scale)));
                if o.badge_now != badge {
                    o.badge_now = badge;
                    if let Some((st, v)) = badge {
                        if let Some(m) = data.art.number(ctx, st, v) {
                            if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(be) {
                                mr.mesh = m;
                            }
                        }
                    }
                    show(ctx, be, badge.is_some());
                    set_tf(ctx, be, Transform::from_xyz(0.3315, 0.5315, CARD_T * 0.5 + 0.02).with_scale(Vec3::splat(if badge.is_some() { 0.245 } else { 0.0 })));
                }
            }
            None => {
                if let Some(o) = self.insp_obj.take() {
                    ctx.despawn(o.root);
                }
                if let Some(e) = self.insp_tile.take() {
                    ctx.despawn(e);
                }
            }
        }
    }

    /// Balls from attacker to target, the impact, the +N / -N, the shards, the confetti.
    fn sync_effects(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        // new power changes: a ball flies (or the card glows when it buffs itself)
        let n = self.b().c.len().min(self.cards.len());
        for j in 0..n {
            let c = self.b().c[j].clone();
            let at = if c.alive { c.fx_t } else { c.die_t.min(c.fx_t) };
            let fresh = at < self.cards[j].last_fx - 0.01;
            self.cards[j].last_fx = at;
            if !fresh || (c.alive && c.fx_n == 0) {
                continue;
            }
            let src = c.fx_src;
            let kill = !c.alive;
            if src >= 0 && src as usize != j {
                let mat = if c.fx_n > 0 && !kill { data.art.ball_buff } else { data.art.ball_hit };
                let sphere = ctx.renderer.builtin.sphere;
                if let Some(m) = mat {
                    let e = spawn(ctx, "ball", sphere, m, Transform::from_translation(self.cards[src as usize].pose.pos).with_scale(Vec3::splat(0.0)), None, false);
                    self.balls.push(Ball { e, from: src as usize, to: j, t: 0.0, buff: c.fx_n > 0 && !kill, n: c.fx_n, kill, hit: false });
                    self.balls_fired += 1;
                }
            } else {
                // its own power changed (Sproutle, Cinderpup, Clockcrab)
                let p = self.cards[j].pose.pos;
                burst(ctx, p + Vec3::Y * 0.08, ParticlePreset::Magic, if c.fx_n > 0 { Vec4::new(0.6, 3.0, 0.6, 1.0) } else { Vec4::new(3.0, 0.6, 0.4, 1.0) }, 22, 1.4, 0.08, 1.0);
                self.fx_label(ctx, p, c.fx_n);
            }
        }
        // the balls: an arc from the attacker (ease in), the impact
        let mut i = 0;
        while i < self.balls.len() {
            let bl = &mut self.balls[i];
            bl.t += dt;
            let q = (bl.t - 0.12) / 0.32;
            let a = self.cards[bl.from].pose.pos + Vec3::Y * 0.12;
            let bpos = self.cards[bl.to].pose.pos + Vec3::Y * 0.08;
            let (e, buff, n, kill, to) = (bl.e, bl.buff, bl.n, bl.kill, bl.to);
            if q < 0.0 {
                set_tf(ctx, e, Transform::from_translation(a).with_scale(Vec3::splat(0.0)));
            } else if q < 1.0 {
                let ee = q * q;
                let arc = if buff { 0.8 } else { 0.45 };
                let p = a.lerp(bpos, ee) + Vec3::Y * arc * 4.0 * ee * (1.0 - ee);
                let s = if ee < 0.5 { 0.4 + 1.6 * ee } else { 1.2 - 0.6 * (ee - 0.5) };
                set_tf(ctx, e, Transform::from_translation(p).with_scale(Vec3::splat(0.13 * s)));
            } else if !bl.hit {
                bl.hit = true;
                set_tf(ctx, e, Transform::from_translation(bpos).with_scale(Vec3::splat(0.0)));
                if buff {
                    burst(ctx, bpos, ParticlePreset::Magic, Vec4::new(0.8, 3.2, 0.8, 1.0), 30, 1.8, 0.09, 1.0);
                } else {
                    burst(ctx, bpos, ParticlePreset::Sparks, Vec4::new(3.5, 1.4, 0.3, 1.0), 36, 2.6, 0.06, 3.0);
                }
                if !kill {
                    self.fx_label(ctx, bpos, n);
                }
                let _ = to;
            }
            if self.balls[i].t > 0.9 {
                let e = self.balls[i].e;
                ctx.despawn(e);
                self.balls.remove(i);
            } else {
                i += 1;
            }
        }
        // the shards fall and fade
        let mut i = 0;
        while i < self.shards.len() {
            let s = &mut self.shards[i];
            s.t += dt;
            s.vel.y -= 6.0 * dt;
            let e = s.e;
            let (vel, spin, t) = (s.vel, s.spin, s.t);
            if let Ok(mut tr) = ctx.world.get::<&mut Transform>(e) {
                tr.translation += vel * dt;
                tr.rotation = Quat::from_scaled_axis(spin * dt) * tr.rotation;
            }
            if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
                mr.opacity = (1.0 - (t - 0.5) / 0.6).clamp(0.0, 1.0);
            }
            if t > 1.1 {
                ctx.despawn(e);
                self.shards.remove(i);
            } else {
                i += 1;
            }
        }
        // the +N / -N rise and fade
        let mut i = 0;
        while i < self.fx_labels.len() {
            self.fx_labels[i].1 += dt;
            let (e, t) = self.fx_labels[i];
            if let Ok(mut l) = ctx.world.get::<&mut WorldLabel>(e) {
                l.offset = Vec3::Y * (0.2 + t * 0.35);
            }
            if t > 1.1 || self.inspect.is_some() {
                ctx.despawn(e);
                self.fx_labels.remove(i);
            } else {
                i += 1;
            }
        }
        // the confetti: LEGO bricks rain onto the table and stay
        if self.confetti_t >= 0.0 {
            self.confetti_t += dt;
            let ct = self.confetti_t;
            for br in &mut self.bricks {
                if ct < br.delay || br.rest {
                    continue;
                }
                br.vel.y -= 7.0 * dt;
                br.vel *= 1.0 - 0.4 * dt;
                br.pos += br.vel * dt;
                br.rot = Quat::from_scaled_axis(br.spin * dt) * br.rot;
                if br.pos.y < BASE.y + 0.035 {
                    br.pos.y = BASE.y + 0.035;
                    if br.vel.y.abs() < 0.8 {
                        br.rest = true;
                        // settle flat on a random turn
                        let (yaw, _, _) = br.rot.to_euler(EulerRot::YXZ);
                        br.rot = Quat::from_rotation_y(yaw);
                    } else {
                        br.vel.y = -br.vel.y * 0.35;
                        br.vel.x *= 0.6;
                        br.vel.z *= 0.6;
                        br.spin *= 0.5;
                    }
                }
                let (e, p, r) = (br.e, br.pos, br.rot);
                if let Ok(mut tr) = ctx.world.get::<&mut Transform>(e) {
                    tr.translation = p;
                    tr.rotation = r;
                }
            }
            if ct > 0.5 && ct - dt <= 0.5 {
                for k in 0..3 {
                    let x = (k as f32 - 1.0) * self.lay.col;
                    burst(ctx, BASE + Vec3::new(x, 0.4, 0.0), ParticlePreset::Sparks, Vec4::new(3.0, 2.4, 0.6, 1.0), 50, 3.5, 0.07, 2.5);
                }
            }
        }
        let _ = data;
    }

    fn fx_label(&mut self, ctx: &mut Ctx, at: Vec3, n: i32) {
        if self.inspect.is_some() || n == 0 {
            return;
        }
        let (cls, txt) = if n > 0 { ("up", format!("+{n}")) } else { ("dn", n.to_string()) };
        let e = ctx.world.spawn((Transform::from_translation(at), WorldLabel { html: format!("<span class='{cls}'>{txt}</span>"), offset: Vec3::Y * 0.2, class: "bt-fx".into() }));
        self.fx_labels.push((e, 0.0));
    }

    fn spawn_confetti(&mut self, ctx: &mut Ctx, data: &mut Data) {
        let cube = ctx.renderer.builtin.cube;
        let l = self.lay;
        for i in 0..70u32 {
            let h = |m: u32| (((i * 5 + m).wrapping_mul(2654435761)) >> 24) as f32 / 255.0;
            let m = data.art.confetti[(i % 7) as usize];
            let big = h(3) < 0.5;
            let size = if big { Vec3::new(0.16, 0.07, 0.08) } else { Vec3::new(0.08, 0.07, 0.08) };
            let pos = BASE + Vec3::new((h(4) - 0.5) * (3.0 * l.col + 1.2), 3.2 + h(6) * 1.5, (h(7) - 0.5) * 2.0 * l.rows_end);
            let e = spawn(ctx, "confetti", cube, m, Transform::from_translation(pos).with_scale(size), None, true);
            let vel = Vec3::new((h(8) - 0.5) * 1.6, -0.5 - h(9), (h(10) - 0.5) * 1.6);
            let spin = Vec3::new(h(11) - 0.5, h(12) - 0.5, h(13) - 0.5) * 14.0;
            self.bricks.push(Brick { e, pos, vel, rot: Quat::IDENTITY, spin, delay: h(1) * 0.7, rest: false });
        }
    }

    // ---------------------------------------------------------------- the HUD

    fn draw_hud(&mut self, ctx: &mut Ctx, data: &Data) {
        let b = self.b.as_ref().unwrap();
        let l = self.lay;
        let over = b.phase == Phase::Over;
        let plan = b.phase == Phase::Plan;
        // ---- the top bar: the opponent with the 1st / 2nd badge, the stakes, the turn, Log / SNAP / Retreat ----
        let ini = if self.kind == BattleKind::Wild { "?".to_string() } else { b.opponent.chars().next().unwrap_or('?').to_string() };
        let foe_first = b.first == SIDE_AI;
        let badge = |first: bool| if over { String::new() } else { format!("<span class='bt-badge{}'>{}</span>", if first { " first" } else { "" }, if first { "1st" } else { "2nd" }) };
        let foe_col = match self.kind {
            BattleKind::Rival => "#c0392b",
            BattleKind::Pip => "#2e9d5a",
            _ => "#7b2fbe",
        };
        let sub = if l.phone { String::new() } else { format!("<small>Hand {} &middot; Deck {}</small>", b.hand[SIDE_AI].len(), b.deck[SIDE_AI].len()) };
        let top = format!(
            "<div class='bt-top'><div class='bt-port{lit}' style='background:{foe_col}'>{ini}{fb}</div><div class='bt-name'><b>{name}</b>{sub}</div><div class='bt-sp'></div>\
             <div class='bt-stakes{hot}'><i></i>{stakes}</div><div class='bt-turn'>T<b>{turn}</b>/{max}</div><div class='bt-sp'></div>\
             <button class='bt-btn' data-action='bt_log'>Log</button><button class='bt-btn snap' data-action='bt_snap'{snapd}>SNAP</button><button class='bt-btn' data-action='bt_retreat'{retd}>{ret}</button></div>",
            lit = if foe_first && !over { " lit" } else { "" },
            fb = badge(foe_first),
            name = b.opponent,
            hot = if b.stakes > 1 { " hot" } else { "" },
            stakes = b.stakes,
            turn = b.turn,
            max = b.max_turns,
            snapd = if b.p_snapped || over { " disabled" } else { "" },
            retd = if over { " disabled" } else { "" },
            ret = if l.phone { "Flee" } else { "Retreat" },
        );
        // ---- the bottom: the energy orb with your portrait, Undo / End Turn ----
        let no_moves = b.no_moves();
        let me_first = b.first == SIDE_P;
        let bottom = format!(
            "<div class='bt-bot{ph}' style='--bh:{bh:.0}px;--orb:{orb:.0}px'><div class='bt-orb{spent}'><b>{left}<small>/{en}</small></b><span>ENERGY</span></div>\
             <div class='bt-me'><div class='bt-port{lit}' style='background:#1e6fe0;width:34px;height:34px;font-size:10px'>YOU{mb}</div></div>\
             <div class='bt-ctl'><button class='bt-btn blue{go}' data-action='bt_end'{endd}>{endl}</button><button class='bt-btn' data-action='bt_undo'{undod}>Undo</button></div></div>",
            ph = if l.phone { " phone" } else { "" },
            bh = l.bottom_px,
            orb = l.orb_px,
            spent = if b.energy_left == 0 && plan { " spent" } else { "" },
            left = b.energy_left,
            en = b.energy,
            lit = if me_first && !over { " lit" } else { "" },
            mb = badge(me_first),
            go = if no_moves { " go" } else { "" },
            endd = if plan && !over { "" } else { " disabled" },
            endl = if b.phase == Phase::Reveal { "\u{2026}" } else { "End Turn" },
            undod = if plan && b.any_staged() { "" } else { " disabled" },
        );
        // ---- over the table: hints, the taunt, banners, the log, the big card's buttons, the result ----
        let mut ov = String::new();
        let dragging = matches!(self.press, Some(Press::Card { moved: true, .. }));
        let hint_y = l.bottom_px + 10.0;
        if let Some((t, _)) = self.taunt {
            ov.push_str(&format!("<div class='bt-taunt'>{t}</div>"));
        }
        if over || self.inspect.is_some() {
        } else if no_moves {
            ov.push_str(&format!("<button class='bt-suggest' data-action='bt_suggest' style='bottom:{:.0}px'>\u{2713} No more moves \u{2014} <b>End Turn</b></button>", l.bottom_px + 10.0));
        } else if dragging && matches!(self.press, Some(Press::Card { from_hand: false, .. })) {
            ov.push_str(&format!("<div class='bt-hint' style='bottom:{hint_y:.0}px'>Drop on your hand to take it back &middot; or on another location to move it</div>"));
        } else if let (Some(_), false) = (self.selected, dragging) {
            ov.push_str(&format!("<div class='bt-hint' style='bottom:{hint_y:.0}px'>Tap a glowing location to play it</div>"));
        } else if plan && b.turn == 1 && !b.any_staged() && !dragging {
            let h = if self.touch_ui || l.phone { "Tap a card, then a location to play it." } else { "Tap a card to see it, then pick a location to play it \u{b7} or drag it there." };
            ov.push_str(&format!("<div class='bt-hint' style='bottom:{hint_y:.0}px'>{h}</div>"));
        }
        if let Some((t, s, _)) = &self.banner {
            let sub = if s.is_empty() { String::new() } else { format!("<small>{s}</small>") };
            ov.push_str(&format!("<div class='bt-banner' data-seq='{}'>{t}{sub}</div>", self.banner_seq));
        }
        if self.log_open {
            let lines: String = b.log.iter().map(|l| format!("<div>&bull; {l}</div>")).collect();
            ov.push_str(&format!("<div class='bt-log'><b>Battle log</b>{lines}</div>"));
        }
        if let (Some(insp), false) = (self.inspect, over) {
            let (_, cy, _, h) = self.inspect_rect();
            let mut inner = String::new();
            match insp {
                Inspect::Zone(_) => inner.push_str("<span>Tap anywhere to close</span>"),
                Inspect::Hand(ci) | Inspect::Board(ci) => {
                    let c = &b.c[ci];
                    let in_hand = matches!(insp, Inspect::Hand(_)) && c.zone < 0;
                    if in_hand && plan {
                        let cost = cards::card(c.id).cost;
                        if cost > b.energy_left {
                            inner.push_str(&format!("<span>Not enough energy ({cost} needed)</span>"));
                        } else {
                            let mut zb = String::new();
                            for z in 0..3 {
                                let (name, col) = if b.zone_rev[z] { (cards::ZONES[b.zone[z]].name.to_string(), crate::cardface::hex(cards::ZONES[b.zone[z]].color)) } else { (format!("Location {}", z + 1), "#555555".into()) };
                                let dis = if b.can_play(ci, z) { "" } else { " disabled" };
                                zb.push_str(&format!("<button data-action='bt_play:{z}' style='background:{col}'{dis}>{name}</button>"));
                            }
                            inner.push_str(&format!("<span>Play to:</span><div class='bt-zones'>{zb}</div>"));
                        }
                    } else if c.owner == SIDE_P && c.staged && plan {
                        inner.push_str("<button class='bt-btn' data-action='bt_take_back'>\u{21ba} Take back</button>");
                    } else if c.owner == SIDE_P && b.can_activate(ci) && plan {
                        inner.push_str("<button class='bt-btn green' data-action='bt_activate'>\u{25b6} Activate</button>");
                    } else if c.owner == SIDE_P && c.act_order >= 0 && plan {
                        inner.push_str("<span>Activates when the turn is revealed, in the order you played</span><button class='bt-btn' data-action='bt_cancel_act'>\u{21ba} Cancel activate</button>");
                    } else if cards::card(c.id).kind == cards::Kind::Activate && c.activated {
                        inner.push_str("<span>Already activated</span>");
                    }
                    inner.push_str("<button class='bt-btn' data-action='bt_close'>\u{d7} Close</button>");
                }
            }
            ov.push_str(&format!("<div class='bt-insp' style='top:{:.0}px'>{inner}</div>", cy + h * 0.5 + 10.0));
        }
        if over && b.result != Outcome::Retreat {
            let title = match b.result {
                Outcome::Win => "VICTORY!",
                Outcome::Lose => "DEFEAT",
                _ => "DRAW",
            };
            let delta = match b.result {
                Outcome::Win => b.stakes,
                Outcome::Lose => -b.stakes,
                _ => 0,
            };
            let mut sc = String::new();
            for z in 0..3 {
                let w = b.zone_winner(z);
                let (mark, col) = match w {
                    Some(SIDE_P) => ("\u{2713}", "#4ade80"),
                    Some(_) => ("\u{2717}", "#f87171"),
                    None => ("\u{2013}", "rgba(255,255,255,.6)"),
                };
                sc.push_str(&format!("<div><small>{}</small><b>{} : {}</b><i style='color:{col}'>{mark}</i></div>", cards::ZONES[b.zone[z]].name, b.zone_power(z, SIDE_P), b.zone_power(z, SIDE_AI)));
            }
            ov.push_str(&format!(
                "<div class='bt-end'><div class='bt-endp'><h2 class='{}'>{title}</h2><div class='bt-sc'>{sc}</div><div class='bt-studs'><i></i>{}{delta} Studs</div><button class='bt-btn blue' data-action='bt_continue'>Continue</button></div></div>",
                if b.result == Outcome::Win { "win" } else { "" },
                if delta >= 0 { "+" } else { "" }
            ));
        }
        for (i, (slot, html)) in [("bt_top", top), ("bt_bottom", bottom), ("bt_over", ov)].into_iter().enumerate() {
            if self.hud[i] != html {
                ctx.ui.set(slot, &html);
                self.hud[i] = html;
            }
        }
        let _ = data;
    }

    // ---------------------------------------------------------------- state for tests

    fn describe(&self, ctx: &Ctx, data: &Data) -> Value {
        let Some(b) = &self.b else { return Value::Null };
        let card = |ci: usize| -> Value {
            let c = &b.c[ci];
            let o = self.cards.get(ci);
            json!({
                "i": ci, "id": cards::card(c.id).id, "power": b.power(ci), "revealed": c.revealed, "staged": c.staged,
                "owner": c.owner, "zone": c.zone,
                "y": o.map(|o| ((o.pose.pos.y - BASE.y) * 1000.0).round() / 1000.0),
                "flip": o.map(|o| (o.pose.flip * 100.0).round() / 100.0),
                "anim": o.and_then(|o| o.anim.map(|a| json!({"style": a.style, "t": (a.t / a.dur * 100.0).round() / 100.0}))),
                "face": o.map(|o| o.face_shown),
            })
        };
        let locs: Vec<Value> = (0..3)
            .map(|z| {
                let zd = &cards::ZONES[b.zone[z]];
                json!({
                    "id": zd.id, "name": zd.name, "revealed": b.zone_rev[z],
                    "power": [b.zone_power(z, SIDE_P), b.zone_power(z, SIDE_AI)],
                    "winner": b.zone_winner(z).map(|s| if s == SIDE_P { "you" } else { "foe" }),
                    "mine": b.zone_order(z, SIDE_P).into_iter().map(card).collect::<Vec<_>>(),
                    "theirs": b.zone_order(z, SIDE_AI).into_iter().map(card).collect::<Vec<_>>(),
                    "open": b.zone_open(z, SIDE_P),
                    "tile_flip": self.tiles.get(z).map(|t| (t.flip * 100.0).round() / 100.0),
                    "screen": self.screen_of(ctx, self.lay.slot(z, SIDE_P, 0) - Vec3::Z * (self.lay.row0)),
                })
            })
            .collect();
        let hand: Vec<Value> = b.hand[SIDE_P].iter().map(|&ci| {
            let mut v = card(ci);
            v["playable"] = json!(b.playable(ci));
            v["cost"] = json!(cards::card(b.c[ci].id).cost);
            v["screen"] = json!(self.cards.get(ci).and_then(|o| self.screen_of(ctx, o.pose.pos)));
            v
        })
        .collect();
        json!({
            "kind": self.kind, "opponent": b.opponent, "phase": b.phase, "turn": b.turn, "max_turns": b.max_turns,
            "energy": b.energy, "energy_left": b.energy_left, "stakes": b.stakes, "first": if b.first == SIDE_P { "you" } else { "foe" },
            "result": b.result, "p_snapped": b.p_snapped, "ai_snapped": b.ai_snapped,
            "hand": hand, "hand_n": b.hand[SIDE_P].len(), "ai_hand_n": b.hand[SIDE_AI].len(),
            "deck_n": b.deck[SIDE_P].len(), "ai_deck_n": b.deck[SIDE_AI].len(),
            "locations": locs, "log": b.log, "no_moves": b.no_moves(), "step": b.step_i,
            "selected": self.selected.map(|c| cards::card(b.c[c].id).id),
            "inspect": self.inspect.map(|i| format!("{i:?}")),
            "dragging": matches!(self.press, Some(Press::Card { moved: true, .. })),
            "drag_over": self.drag_over,
            "taunt": self.taunt.map(|t| t.0),
            "banner": self.banner.as_ref().map(|b| b.0.clone()),
            "anims": self.cards.iter().filter(|o| o.anim.is_some()).count(),
            "lands": self.lands, "flips": self.flips, "shatters": self.shatters, "balls": self.balls_fired, "plays": self.plays,
            "confetti": self.bricks.len(), "confetti_resting": self.bricks.iter().filter(|b| b.rest).count(),
            "camera": {"dist": (self.cam_dist * 100.0).round() / 100.0, "pitch_deg": self.pitch.to_degrees().round(), "portrait": self.lay.portrait},
            "layout": {"w": self.lay.w, "h": self.lay.h, "hand_px": self.lay.hand_px, "bottom_px": self.lay.bottom_px},
            "pictures": data.art.pictures_loaded(),
            "speed": self.speed,
            "transition": data.trans.as_ref().map(|t| (t.t * 100.0).round() / 100.0),
            "track": data.sound.track().name(),
            "last_sfx": data.sound.last_sfx,
            "voice": data.sound.last_voice,
            "done": self.done,
        })
    }

    /// A world point in CSS px (null behind the camera).
    fn screen_of(&self, ctx: &Ctx, p: Vec3) -> Option<[f32; 2]> {
        let cam = self.cam_tf(ctx);
        let view = Mat4::from_rotation_translation(cam.rotation, cam.translation).inverse();
        let proj = Mat4::perspective_rh(FOV.to_radians(), self.lay.w / self.lay.h, 0.05, 60.0);
        let c = proj * view * p.extend(1.0);
        if c.w <= 0.0 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some([((n.x * 0.5 + 0.5) * self.lay.w).round(), ((0.5 - n.y * 0.5) * self.lay.h).round()])
    }

    fn card_arg(&self, a: &Value) -> Option<usize> {
        let b = self.b();
        if let Some(i) = a["hand"].as_u64() {
            return b.hand[SIDE_P].get(i as usize).copied();
        }
        let id = cards::by_id(a["card"].as_str()?)?;
        b.hand[SIDE_P].iter().copied().find(|&ci| b.c[ci].id == id).or_else(|| (0..b.c.len()).find(|&ci| b.c[ci].id == id && b.c[ci].owner == SIDE_P && b.c[ci].alive))
    }
}

impl Screen<Data> for BattleScreen {
    fn name(&self) -> &'static str {
        "battle"
    }

    fn enter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.ensure_loaded(ctx);
        // the village camera stays (with its state); the battle camera is made main once built, the stick is off
        self.prev_cam = engine::scene::main_camera(&ctx.world);
        ctx.input.touch.set_stick_enabled(false);
        // a table lit from above: no day cycle, no fog
        self.saved = Some(ctx.renderer.settings.clone());
        let s = &mut ctx.renderer.settings;
        s.day_cycle = false;
        s.sky = false;
        s.sky_mode = 0;
        s.clear_color = engine::renderer::srgb_hex(0x0d1712).truncate();
        s.fog_color = None;
        s.fog_density = 0.0;
        s.sun_dir = Vec3::new(-0.32, -1.0, -0.42).normalize();
        s.sun_color = engine::renderer::srgb_hex(0xfff1dc).truncate();
        s.sun_intensity = 3.2;
        s.sun_scale = 1.0;
        s.ambient_scale = 1.0;
        s.daylight_intensity = 1.0;
        s.atmosphere_intensity = 1.0;
        s.sky_color = engine::renderer::srgb_hex(0xdfeaff).truncate() * 0.6;
        s.ground_color = engine::renderer::srgb_hex(0x6a5038).truncate() * 0.45;
        s.shadow_distance = 16.0;
        s.shadow_cascades = s.shadow_cascades.min(2);
        s.exposure = 1.0;
        s.vignette = 0.35;
        s.bloom_threshold = 1.1;
        s.lens_flare = 0.0;
        self.layout(ctx);
        self.build(ctx, data);
        self.fit_camera(ctx);
        // everything starts on its pile
        for ci in 0..self.cards.len() {
            let side = self.b().c[ci].owner;
            self.cards[ci].pose.pos = self.lay.pile(side);
        }
        self.say(ctx, data, 0, 3.0);
        log_info!("battle vs {} ({:?}), seed {}", self.b().opponent, self.kind, self.seed);
    }

    fn exit(&mut self, ctx: &mut Ctx, data: &mut Data) {
        if let Some(e) = self.prev_cam.take().filter(|e| ctx.world.contains(*e)) {
            ctx.set_main_camera(e);
        }
        ctx.input.touch.set_stick_enabled(true);
        if let Some(s) = self.saved.take() {
            ctx.renderer.settings = s;
        }
        data.sound.stop_voice(ctx);
        for slot in ["bt_top", "bt_bottom", "bt_over"] {
            ctx.ui.set(slot, "");
        }
        if let Some(b) = &self.b {
            data.battle_out = Some(BattleOut { kind: self.kind, result: b.result, stakes: b.stakes });
        }
    }

    fn update(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) -> Trans<Data> {
        let dt = dt.min(0.1);
        let bdt = dt * self.speed;
        self.time += dt;
        data.sound.update(ctx, dt);
        data.track_music(ctx, true);
        transition_update(ctx, data, dt);
        data.art.poll(ctx);
        if self.backdrop.is_none() {
            let bg = match self.kind {
                BattleKind::Rival => 1,
                BattleKind::Practice => 2,
                _ => 0,
            };
            if let Some(t) = data.art.backdrop_tex[bg] {
                let m = ctx.renderer.add_material(Material { texture: Some(t), unlit: true, ..Material::color(0x8a96a0) });
                let quad = ctx.renderer.builtin.quad;
                self.backdrop = Some(spawn(ctx, "backdrop", quad, m, Transform::from_translation(BASE + Vec3::new(0.0, 3.0, -9.0)).with_scale(Vec3::new(28.0, 15.75, 1.0)), None, false));
            }
        }
        self.layout(ctx);
        self.fit_camera(ctx);
        let actions = ctx.ui.take_actions();
        self.input(ctx, data, &actions);
        self.run(ctx, data, bdt);
        if let Some((_, t)) = &mut self.taunt {
            *t -= dt;
            if *t <= 0.0 {
                self.taunt = None;
            }
        }
        if let Some((_, _, t)) = &mut self.banner {
            *t -= dt;
            if *t <= 0.0 {
                self.banner = None;
            }
        }
        self.sync_cards(ctx, data, bdt.max(dt));
        self.sync_tiles(ctx, data, dt);
        self.sync_inspect(ctx, data, dt);
        self.sync_effects(ctx, data, bdt.max(dt));
        self.draw_hud(ctx, data);
        data.save.update(ctx, dt);
        if self.done {
            return Trans::Pop;
        }
        Trans::None
    }

    fn debug_commands(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("battle_play", "Play a hand card of yours to a location (the UI's path). args: {card: 'id' | hand: index, zone: 0..2}"),
            ("battle_end_turn", "End Turn (as the button / Enter)."),
            ("battle_auto", "Play this turn as the AI would for you (stages through the UI's path), then optionally end the turn. args: {end?: bool}"),
            ("battle_ui", "Press a HUD button. args: {action: bt_end|bt_undo|bt_snap|bt_retreat|bt_log|bt_continue|bt_close|bt_play:z|bt_take_back|bt_activate}"),
            ("battle_inspect", "Open the big card or location. args: {card: id} | {zone: 0..2} | {close: true}"),
            ("battle_speed", "The battle's speed (reveal pauses, animations). args: {x: 1}"),
            ("battle_cheat", "Change power: every card of yours gets +n (or the opponent's with foe: true). args: {power: n, foe?: bool}"),
            ("battle_continue", "Leave the result screen (as Continue)."),
        ]
    }

    fn debug_command(&mut self, ctx: &mut Ctx, data: &mut Data, name: &str, a: &Value) -> Option<Result<Value, String>> {
        self.b.as_ref()?;
        match name {
            "battle_play" => {
                let ci = self.card_arg(a)?;
                let z = a["zone"].as_u64()? as usize;
                let ok = self.play(ctx, data, ci, z);
                Some(Ok(json!({"played": ok, "card": cards::card(self.b().c[ci].id).id, "zone": z})))
            }
            "battle_end_turn" => {
                self.end_turn(ctx, data);
                Some(Ok(json!({"phase": self.b().phase})))
            }
            "battle_auto" => {
                let plan = self.b.as_mut().unwrap().plan(SIDE_P);
                let mut played = vec![];
                for (ci, z) in plan {
                    if self.play(ctx, data, ci, z) {
                        played.push(json!([cards::card(self.b().c[ci].id).id, z]));
                    }
                }
                if a["end"].as_bool() == Some(true) {
                    self.end_turn(ctx, data);
                }
                Some(Ok(json!({"played": played})))
            }
            "battle_ui" => {
                let act = a["action"].as_str()?.to_string();
                ctx.ui.push_action(&act);
                Some(Ok(json!({"queued": act})))
            }
            "battle_inspect" => {
                if a["close"].as_bool() == Some(true) {
                    self.inspect = None;
                } else if let Some(z) = a["zone"].as_u64() {
                    self.inspect = Some(Inspect::Zone(z as usize % 3));
                } else {
                    let ci = self.card_arg(a)?;
                    let in_hand = self.b().hand[SIDE_P].contains(&ci);
                    self.inspect = Some(if in_hand { Inspect::Hand(ci) } else { Inspect::Board(ci) });
                    if in_hand {
                        self.selected = Some(ci);
                    }
                }
                Some(Ok(json!({"inspect": self.inspect.map(|i| format!("{i:?}"))})))
            }
            "battle_speed" => {
                self.speed = a["x"].as_f64().unwrap_or(1.0).clamp(0.05, 20.0) as f32;
                Some(Ok(json!({"speed": self.speed})))
            }
            "battle_cheat" => {
                let n = a["power"].as_i64().unwrap_or(10) as i32;
                let side = if a["foe"].as_bool() == Some(true) { SIDE_AI } else { SIDE_P };
                for c in &mut self.b.as_mut().unwrap().c {
                    if c.owner == side {
                        c.modp += n;
                    }
                }
                Some(Ok(json!({"power": n})))
            }
            "battle_continue" => {
                if self.b().phase == Phase::Over {
                    self.done = true;
                }
                Some(Ok(json!({"done": self.done})))
            }
            _ => None,
        }
    }

    fn debug_state(&self, ctx: &Ctx, data: &Data) -> Value {
        json!({"battle": self.describe(ctx, data)})
    }
}
