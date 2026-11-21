//! The play screen: the village, you, the villagers, talking, the HUD (goal, prompt, toast, markers,
//! the ☰ menu) and the touch buttons.

use crate::arena::{self, BattleOut, BattleScreen};
use crate::battle::{BattleKind, Outcome};
use crate::cards;
use crate::dive::{self, Dive};
use crate::dialog::{dialog_for, DialogAction, DialogId, DialogState};
use crate::figure::{Figure, Pose};
use crate::life::{self, Life, LifeIn, LifeView};
use crate::settings::{self, MenuOut};
use crate::sound::{self, Sfx, Track};
use crate::screens::{ScreenKind, Screens};
use crate::talk::{self, CamZoom, Use, TARGETS};
use crate::town::rooms::{self, RoomId};
use crate::town::{self, TownAssets, TownStats, STUD};
use crate::vox::Vox;
use crate::{day_colours, setup_look, Data, Title};
use engine::prelude::*;
use serde_json::{json, Value};

/// Where you start (outside, in front of your house); a new game wakes up in your room upstairs (`INTRO_AT`).
const START: [f32; 2] = [-15.0, -2.5];
/// Camera offset of the original (studs, before zoom) and its default zoom.
const CAM_OFFSET: Vec3 = Vec3::new(0.0, 32.0, 17.0);
const ZOOM: f32 = 1.15;
/// A new game: in your room, next to the bed (Kaigen app_update).
const INTRO_AT: [f32; 2] = [197.4, 41.2];
/// Going through a door: the screen goes dark, you are moved, it comes back (seconds).
const DOOR_OUT: f32 = 0.15;
const DOOR_IN: f32 = 0.25;
/// Indoors the sun is a soft remainder and the sky light dims: the warm ceiling lights carry the room.
const INDOOR_SUN: f32 = 0.04;
const INDOOR_SKY: f32 = 0.3;

/// A villager in the town: id (as `town::NPCS`), figure, the way it faces (yaw, figures face +z at 0),
/// its "!" / "…" marker (a `WorldLabel` child) and that marker's current html.
struct Npc {
    id: String,
    /// the room it stands in (None: the village)
    area: Option<RoomId>,
    fig: Figure,
    yaw: f32,
    marker: Entity,
    marker_html: String,
}

#[derive(Default)]
pub struct PlayScreen {
    hero: Option<Entity>,
    fig: Option<Figure>,
    npcs: Vec<Npc>,
    camera: Option<Entity>,
    stats: TownStats,
    /// the figure moved this frame (horizontal speed, m/s)
    speed: f32,
    /// the street lamps' lights (warm, switched on at night)
    lamps: Vec<Entity>,
    /// buildings and trees that fade while they hide you
    fader: crate::fade::Fader,
    /// the touch Jump button was tapped (consumed by the next fixed step)
    jump_tap: bool,
    /// the touch HUD (buttons, hints) is shown
    touch_ui: bool,
    /// Pip and the village around (life.rs)
    life: Life,
    dlg: DialogState,
    /// the villager you talk to (index into `npcs`)
    talk_npc: Option<usize>,
    /// you turn to whom you talk to (yaw of the character controller) while the dialog is open
    face_yaw: Option<f32>,
    /// the camera at your eyes looking at them
    zoom: CamZoom,
    /// the talk target in reach (index into `talk::TARGETS`)
    nearby: Option<usize>,
    /// Space closed a dialog: no jump until it is let go
    jump_block: bool,
    /// a dialog or the menu holds you in place
    busy: bool,
    mom_talked: bool,
    kid_talked: bool,
    /// seconds since the clock was last saved
    clock_t: f32,
    /// the last dialog action no part handles yet (battles, hide and seek, ...), for tests and later parts
    pending_action: Option<DialogAction>,
    // ---- rooms (rooms.rs) ----
    /// the room you are in (None: the village); the camera draws only this area
    area: Option<RoomId>,
    /// the area the camera and the lights were last set up for
    shown_area: Option<Option<RoomId>>,
    /// going through a door: seconds since it started, where to (studs), the yaw there
    door: Option<(f32, Vec2, f32)>,
    door_cooldown: f32,
    doors_used: u32,
    /// the area title ("Home", "Brickhollow") and its seconds left
    title: &'static str,
    title_t: f32,
    /// the rooms' ceiling lights (room, entity, relative intensity)
    room_lights: Vec<(RoomId, Entity, f32)>,
    /// 0 outside .. 1 inside (eased): dims the sun and the sky light
    indoor: f32,
    /// the Deck-o-Matic's screens: "MISSING PART" and "READY"
    machine: Option<(Entity, Entity)>,
    machine_ready: Option<bool>,
    /// the camera target while looking at something without a face (the Deck-o-Matic)
    talk_board: Option<talk::Board>,
    /// the intro was checked this session, and the seconds since the screen started
    intro_checked: bool,
    play_t: f32,
    /// your figure is hidden (the camera is where you stand: a close-up)
    hero_hidden: bool,
    // ---- things to look at ----
    /// the TV, the piano, a computer (screens.rs)
    scr: Screens,
    /// a sign's text is open (the readable; boards 0..3 bring the camera to them)
    reading: Option<usize>,
    /// naps taken (tests)
    sleeps: u32,
    /// 3D cards of the `card_preview` command
    previews: Vec<Entity>,
    /// `battle {instant: true}`: push the battle screen on the next update
    push_battle: Option<(BattleKind, u32)>,
    /// a seed for the next battle (`battle {seed}`; else the card game's generator)
    battle_seed: Option<u32>,
    /// battles played from here (tests)
    battles: u32,
    // ---- your Voxelmon (vox.rs) ----
    vox: Vox,
    /// a fight in the tall grass holds you (you only step back to watch it)
    fight: bool,
    help_hidden: bool,
    // ---- the lab quest's dive (dive.rs, cave.rs) ----
    dive: Dive,
    /// > 0: you jumped into the pond with the suit and sink in (seconds; then the dive)
    sink_t: f32,
    /// the pond does nothing for a moment after it threw you out / you came back up
    pond_cd: f32,
    /// back from the dive: the black fades out (seconds left)
    surface_fade: f32,
    /// times the pond sent you back to the bank (no suit; tests)
    climb_outs: u32,
    // ---- Route 1 north and Stonecrest (north.rs) ----
    /// the outdoor zone you are in (`town::zone_at`: 0 Brickhollow, 1 Route 1, 2 Stonecrest)
    zone: usize,
    /// the outdoor band layers the camera draws (`town::bands_near`)
    bands: u32,
    /// where Prof. Quill at the gap walks to (studs x, z, yaw): aside once the way is open
    gate_to: Option<[f32; 3]>,
    /// times the closed gap sent you back (tests)
    gate_stops: u32,
    /// the tall grass tufts bending and swaying (grass.rs)
    grass: crate::grass::Grass,
    /// a debug close-up (`look`): the camera eases to this board
    look: Option<talk::Board>,
    /// Stonecrest's fountain: its water, jet, streams, spray and ripples (fountain.rs)
    fountain: crate::fountain::Fountain,
    // ---- village life: ducks, ball, chest, weather, Pip's games (life.rs) ----
    life_view: LifeView,
    /// days passed (the clock's wraps) and the last hour seen
    day: u32,
    last_hours: Option<f32>,
    /// your move wish this step (world x/z, for kicking the ball) and the sprint key held while moving
    move_dir: Vec2,
    sprinting: bool,
    /// swipe walking on phones (swipe.rs) and the way it walks this step
    swipe: crate::swipe::Swipe,
    swipe_mode: bool,
    /// the debug panel (debugpanel.rs)
    dbg: crate::debugpanel::DebugPanel,
    /// seconds of the frame clock (swipe walking's stall timer)
    clock: f32,
    /// times a move wish into a villager was stopped (tests)
    npc_stops: u32,
    /// the save's golden studs (for the debug picker)
    golden_seen: u32,
    /// R / falls (tests)
    respawns: u32,
}

/// Where you come out of the pond after a dive (studs, the north bank; Kaigen -8, but Pip stands there at home).
const POND_EXIT: [f32; 2] = [-5.5, 13.6];
const SURFACE_FADE: f32 = 0.5;
/// The village HUD slots that go while you dive.
const DIVE_HIDES: [&str; 15] = ["help", "touch", "goal", "prompt", "menubtn", "settings", "title", "sign", "deck_btn", "vox_wild", "vox_mine", "vox_pill", "fade", "seekfade", "debug"];

/// Help line for phones: the engine's joystick (left 40 % of the screen) walks, pushing it all the way
/// out runs, two fingers zoom.
/// (the heading is the zone you are in: Kaigen `g_hud_place`)
const TOUCH_HELP: &str = "<div class='panel bh-thelp'><b>{place}</b><br>left thumb: walk (push far = run) &middot; two fingers: zoom</div>";
const SWIPE_HELP: &str = "<div class='panel bh-thelp'><b>{place}</b><br>swipe: walk that way &middot; tap: stop &middot; two fingers: zoom</div>";
const KEYS_HELP: &str = "<div class='panel'><b>{place}</b><br><kbd>WASD</kbd> move &middot; <kbd>Shift</kbd> run &middot; <kbd>Space</kbd> jump &middot; <kbd>E</kbd> use / talk &middot; <kbd>Q</kbd> dance &middot; <kbd>R</kbd> respawn &middot; <kbd>C</kbd> cards &middot; <kbd>Wheel</kbd> zoom &middot; <kbd>Esc</kbd> menu</div>";
/// Stick deflection from which the hero runs.
const TOUCH_RUN: f32 = 0.92;

/// The game's HUD styles (dialog box, goal pill, markers, prompt, toast, the menu), in a style slot.
pub const STYLE: &str = "<style>
#ui [data-slot=help]{position:fixed;left:16px;bottom:16px}
/* touch: the place and hints stay left of the Use and Jump buttons (2 x 84 px from 20 px on the right) */
#ui .bh-thelp{max-width:calc(100vw - 236px);box-sizing:border-box;font-size:12px;line-height:1.35}
#ui [data-slot=goal]{position:fixed;left:14px;top:14px;max-width:calc(100vw - 90px)}
.bh-goal{display:flex;gap:8px;align-items:center;background:rgba(250,250,246,.88);color:#1b2a34;border-radius:10px;padding:7px 14px 7px 12px;font-size:15px;font-weight:600;max-width:560px}
.bh-goal.night{background:rgba(24,32,42,.88);color:#e8eef4}
.bh-goal b{background:#f2cd37;color:#1b2a34;border-radius:5px;padding:1px 7px;font-size:12px;font-weight:800}
#ui [data-slot=menubtn]{position:fixed;right:14px;top:14px;z-index:21}
#ui .bh-menubtn{margin:0;width:40px;height:36px;border-radius:10px;background:rgba(250,250,246,.92);box-shadow:0 2px 6px rgba(0,0,0,.25);display:flex;flex-direction:column;gap:4px;align-items:center;justify-content:center;font-size:22px;color:#1b2a34;padding:0}
#ui .bh-menubtn.on{background:#fff}
#ui .bh-menubtn i{display:block;width:18px;height:2.5px;border-radius:2px;background:#1b2a34}
#ui [data-slot=settings]{position:fixed;inset:0;pointer-events:none;z-index:20}
#ui [data-slot=settings]:empty{display:none}
.bh-shade{position:fixed;inset:0;background:rgba(0,0,0,.15);pointer-events:auto}
.bh-menu{position:fixed;right:14px;top:58px;width:min(320px,calc(100vw - 28px));background:#fafaf6;color:#1b2a34;border-radius:16px;padding:10px;box-shadow:0 8px 24px rgba(0,0,0,.3);pointer-events:auto;box-sizing:border-box}
#ui .bh-menu button{margin:0;background:transparent;color:#1b2a34;font-weight:600}
#ui .bh-row{display:flex;width:100%;justify-content:space-between;padding:9px 12px;border-radius:10px;font-size:16px}
#ui .bh-row:hover{background:#eef2f5}
#ui .bh-row span{color:#5c6670;font-size:14px}
.bh-prog{display:flex;gap:14px;padding:4px 12px 8px;font-size:14px;font-weight:700;color:#5c6670}.bh-prog .gold{color:#b08600}
.bh-cap{padding:8px 12px 2px;font-size:12px;font-weight:800;color:#5c6670}
.bh-seg{display:flex;gap:3px;padding:3px;background:#e6e9ec;border-radius:10px}
#ui .bh-seg button{flex:1;padding:7px 0;border-radius:8px;color:#5c6670;font-size:14px}
#ui .bh-seg button.on{background:#fff;color:#1b2a34;font-weight:800;box-shadow:0 1px 3px rgba(0,0,0,.18)}
.bh-hint{padding:6px 12px;font-size:13px;color:#5c6670}.bh-fps{padding:0 12px 4px;font-size:12px;color:#8a929a}
#ui [data-slot=dialog]{position:fixed;left:0;right:0;bottom:22px;display:flex;justify-content:center;pointer-events:none}
.bh-dialog{width:min(760px,calc(100vw - 32px));box-sizing:border-box;background:rgba(250,250,246,.96);color:#1b2a34;border:3px solid #1b2a34;border-radius:12px;padding:14px 20px;display:flex;flex-direction:column;gap:8px;align-items:flex-start;pointer-events:auto;cursor:pointer}
.bh-dialog.touch{margin-bottom:118px}
.bh-dialog.night{background:rgba(24,32,42,.95);color:#ecf1f6;border-color:#465668}
.bh-speaker{background:#f2cd37;color:#1b2a34;border-radius:6px;padding:3px 10px;font-size:16px;font-weight:700}
.bh-line{font-size:20px;line-height:1.35}
.bh-cont{font-size:13px;color:#788088}
.bh-choices{display:flex;flex-wrap:wrap;gap:12px}
#ui .bh-choices button{margin:0;font-size:18px;padding:8px 16px;border-radius:8px;background:#e0e6ec;color:#1b2a34}
#ui .bh-dialog.night .bh-choices button{background:#2c3846;color:#ecf1f6}
#ui .bh-choices button.sel{background:#0055bf;color:#fff}
#ui [data-slot=prompt]{position:fixed;left:0;right:0;bottom:28px;display:flex;justify-content:center}
.bh-prompt{display:flex;gap:10px;align-items:center;background:rgba(16,18,24,.75);border-radius:8px;padding:8px 16px 8px 14px;font-size:16px}
.bh-prompt.touch{margin-bottom:122px}
#ui .bh-prompt kbd{background:#f2cd37;color:#1b2a34;font-weight:700;padding:2px 8px;font-size:16px}
#ui [data-slot=title]{position:fixed;left:0;right:0;top:112px;display:flex;justify-content:center;pointer-events:none}
.bh-title{background:rgba(16,18,24,.66);color:#fff;border-radius:8px;padding:8px 18px;font-size:22px}
.bh-fade{position:fixed;inset:0;background:#000;pointer-events:none;z-index:30}
#ui [data-slot=sign]{position:fixed;left:0;right:0;bottom:28px;top:0;display:flex;justify-content:center;align-items:flex-end;pointer-events:none}
#ui [data-slot=sign]:empty{display:none}
.bh-sign{align-self:center;width:min(480px,calc(100vw - 24px));box-sizing:border-box;border:4px solid;border-radius:10px;padding:20px 18px 18px;display:flex;flex-direction:column;gap:10px;text-align:center}
.bh-sign-t{font-size:30px;font-weight:800}.bh-sign hr{width:100%;height:3px;border:0;margin:0}
.bh-sign-l{font-size:18px}.bh-sign-n{font-size:16px;font-style:italic}.bh-sign-c{font-size:13px;opacity:.6}
#ui [data-slot=studs]{position:fixed;right:14px;top:58px;pointer-events:none}
.bh-studs{background:rgba(16,18,24,.66);color:#f2cd37;border-radius:8px;padding:6px 12px;font-size:16px;display:flex;gap:14px}
.bh-studs.narrow{flex-direction:column;gap:2px}.bh-seek{color:#ff8c78}
#ui [data-slot=goal]{display:flex;flex-direction:column;align-items:flex-start;gap:8px}
#ui [data-slot=seekfade]{position:fixed;inset:0;pointer-events:none;z-index:29}
#ui [data-slot=seekfade]:empty{display:none}
.bh-seekfade{position:fixed;inset:0;background:rgba(8,10,14,.94);color:#fff;font-size:34px;display:flex;align-items:center;justify-content:center}
#ui [data-slot=toast]{position:fixed;left:0;right:0;top:70px;display:flex;justify-content:center}
.bh-toast{background:rgba(250,250,246,.95);color:#1b2a34;border:2px solid #1b2a34;border-radius:10px;padding:8px 16px;font-size:18px}
#ui .world-label.bh-mark{background:rgba(250,250,246,.88);color:#1b2a34;border:2px solid #1b2a34;padding:0;width:28px;height:18px;line-height:14px;text-align:center;font-weight:700;font-size:14px;border-radius:10px}
#ui .world-label.bh-mark.big{background:#f2cd37;width:24px;height:24px;line-height:24px;border-radius:14px;font-size:18px}
</style>";

impl PlayScreen {
    /// The help line (keys or touch) under the zone's name (Brickhollow / Route 1 / Stonecrest).
    fn help(&self) -> String {
        let place = if self.area.is_some() { "Brickhollow" } else { town::ZONE_NAMES[self.zone] };
        (if self.swipe_mode { SWIPE_HELP } else if self.touch_ui { TOUCH_HELP } else { KEYS_HELP }).replace("{place}", place)
    }

    /// Outside: the zone you walk into gets its title (three.js `zoneLabel` toast, Kaigen `world_doors`), the help
    /// line its name; the camera draws the band layers near you (all of them while it looks far: a close-up or a fight).
    fn outdoor_zone(&mut self, ctx: &mut Ctx) {
        if self.area.is_some() {
            return;
        }
        let Some((p, _)) = self.hero_pos(ctx) else { return };
        let z = p.z / STUD;
        let zone = town::zone_at(z);
        if zone != self.zone {
            self.zone = zone;
            self.title = ["Back in Brickhollow", "Route 1", "Welcome to Stonecrest!"][zone];
            self.title_t = 3.5;
            if !self.help_hidden {
                ctx.ui.set("help", &self.help());
            }
        }
        let bands = if self.zoom.active || self.fight || self.vox.cam_on { town::all_bands() } else { town::bands_near(z, 30.0, 40.0) };
        if bands != self.bands {
            self.bands = bands;
            if let Some(mut c) = self.camera.and_then(|c| ctx.world.get::<&mut Camera>(c).ok()) {
                c.layers = 1 | bands;
            }
        }
    }

    /// The gap north (Kaigen app_update): Prof. Quill stands in it until you have shown her a Voxelmon
    /// (`gate_open`); walking into it then turns you back to her and she talks. Once open she walks aside.
    fn gate(&mut self, ctx: &mut Ctx, data: &mut Data) {
        if !data.save.loaded {
            return;
        }
        let open = data.save.data.gate_open;
        let want = if open { town::GATE_PROF_OPEN } else { town::GATE_PROF_CLOSED };
        if self.gate_to.is_none() {
            // (the first frame: she simply stands where the save says)
            if let Some(i) = self.npc_index("gate-prof") {
                self.npcs[i].yaw = want[2];
                if let Ok(mut t) = ctx.world.get::<&mut Transform>(self.npcs[i].fig.root) {
                    t.translation = town::m(want[0], 0.0, want[1]);
                    t.rotation = Quat::from_rotation_y(want[2]);
                }
            }
        }
        self.gate_to = Some(want);
        // (not in a fight: stepping back from one in the first patch may lead into the gap)
        if open || self.busy || self.fight || self.area.is_some() || self.door.is_some() {
            return;
        }
        let Some((p, _)) = self.hero_pos(ctx) else { return };
        let (x, z) = (p.x / STUD, p.z / STUD);
        if z < -25.3 && z > -27.5 && x.abs() < 5.0 {
            self.teleport(ctx, x, -22.6);
            if let Some(mut cc) = self.hero.and_then(|h| ctx.world.get::<&mut CharacterController>(h).ok()) {
                cc.yaw = 0.0;
            }
            self.gate_stops += 1;
            self.talk(ctx, data, talk::GATE_TARGET);
        }
    }

    /// Prof. Quill walks to her spot at the gap (fixed step, with the walk cycle).
    fn gate_walk(&mut self, ctx: &mut Ctx, dt: f32) {
        let (Some(to), Some(i)) = (self.gate_to, self.npc_index("gate-prof")) else { return };
        if self.dlg.is_open() && self.talk_npc == Some(i) {
            return; // (she talks first)
        }
        let Some(q) = self.npc_pos(ctx, i) else { return };
        let (cur, goal) = (Vec2::new(q.x, q.z) / STUD, Vec2::new(to[0], to[1]));
        let d = goal - cur;
        let n = &mut self.npcs[i];
        if d.length() < 0.05 {
            if (life::wrap(to[2] - n.yaw)).abs() > 0.01 {
                n.yaw += life::wrap(to[2] - n.yaw) * (dt * 6.0).min(1.0);
                if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.fig.root) {
                    t.rotation = Quat::from_rotation_y(n.yaw);
                }
            }
            n.fig.animate(ctx, dt, false, 1.0);
            return;
        }
        let step = (4.0 * dt).min(d.length());
        let at = cur + d.normalize() * step;
        let want = d.x.atan2(d.y);
        n.yaw += life::wrap(want - n.yaw) * (dt * 10.0).min(1.0);
        if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.fig.root) {
            t.translation = town::m(at.x, 0.0, at.y);
            t.rotation = Quat::from_rotation_y(n.yaw);
        }
        n.fig.animate(ctx, dt, true, 0.8);
    }

    /// A card face from command args {id, level, big, unknown, back, power}.
    fn face_arg(a: &Value) -> Option<crate::cardface::Face> {
        use crate::cardface::Face;
        if a["back"].as_bool() == Some(true) {
            return Some(Face::back());
        }
        let c = cards::by_id(a["id"].as_str()?)?;
        let mut f = Face::new(c, a["level"].as_u64().unwrap_or(0).min(4) as u8);
        if a["big"].as_bool() == Some(true) {
            f = f.big();
        }
        if a["unknown"].as_bool() == Some(true) {
            f = f.unknown();
        }
        if let Some(p) = a["power"].as_i64() {
            f = f.power(p as i32);
        }
        Some(f)
    }

    fn hero_pos(&self, ctx: &Ctx) -> Option<(Vec3, f32)> {
        let h = self.hero?;
        let t = ctx.world.get::<&Transform>(h).ok()?.translation;
        let cc = ctx.world.get::<&CharacterController>(h).ok()?;
        Some((t - Vec3::Y * cc.foot_offset(), cc.yaw))
    }

    fn teleport(&mut self, ctx: &mut Ctx, x: f32, z: f32) {
        if let Some(h) = self.hero {
            let off = ctx.world.get::<&CharacterController>(h).map(|c| c.foot_offset()).unwrap_or(0.85);
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(h) {
                t.translation = Vec3::new(x * STUD, 0.6 + off, z * STUD);
            }
            if let Ok(mut cc) = ctx.world.get::<&mut CharacterController>(h) {
                cc.velocity = Vec3::ZERO;
            }
        }
    }

    /// Moves you (a door, the stairs, the intro): turned to `yaw` (the controller's: 0 north, pi south), the
    /// camera snaps along (it would fly 70 m otherwise).
    fn move_to(&mut self, ctx: &mut Ctx, x: f32, z: f32, yaw: f32) {
        self.teleport(ctx, x, z);
        let Some(h) = self.hero else { return };
        if let Ok(mut cc) = ctx.world.get::<&mut CharacterController>(h) {
            cc.yaw = yaw;
        }
        if let Ok(mut t) = ctx.world.get::<&mut Transform>(h) {
            t.rotation = Quat::from_rotation_y(yaw);
        }
        self.area = rooms::area_at(x, z);
        if let Some(cam) = self.camera {
            if !self.zoom.active {
                let off = CAM_OFFSET * ZOOM * STUD;
                let foot = ctx.world.get::<&CharacterController>(h).map(|c| c.foot_offset()).unwrap_or(0.85);
                let target = town::m(x, 0.6, z) + Vec3::Y * (1.5 * STUD - foot + foot);
                if let Ok(mut c) = ctx.world.get::<&mut ThirdPersonCamera>(cam) {
                    c.pivot = None;
                }
                if let Ok(mut t) = ctx.world.get::<&mut Transform>(cam) {
                    *t = Transform::from_translation(target + off).looking_at(target, Vec3::Y);
                }
            }
        }
    }

    /// The area changed (or the screen started): the camera draws only this area, its ceiling lights
    /// are on, the title shows (Kaigen `world_doors` + the group visibility of app_update).
    fn show_area(&mut self, ctx: &mut Ctx) {
        if self.shown_area == Some(self.area) {
            return;
        }
        let first = self.shown_area.is_none();
        self.shown_area = Some(self.area);
        if self.area.is_none() {
            let z = self.hero_pos(ctx).map(|(p, _)| p.z / STUD).unwrap_or(0.0);
            self.zone = town::zone_at(z);
            self.bands = town::bands_near(z, 30.0, 40.0);
        }
        let layers = if self.area.is_some() { 1 | (1 << rooms::layer_of(self.area)) } else { 1 | self.bands };
        if let Some(mut c) = self.camera.and_then(|c| ctx.world.get::<&mut Camera>(c).ok()) {
            c.layers = layers;
        }
        for (r, e, k) in &self.room_lights {
            if let Ok(mut l) = ctx.world.get::<&mut PointLight>(*e) {
                l.intensity = if Some(*r) == self.area { 9.0 * k } else { 0.0 };
            }
        }
        self.title = self.area.map(|r| rooms::room_def(r).label).unwrap_or(town::ZONE_NAMES[self.zone]);
        self.title_t = if first && self.area.is_none() { 0.0 } else { 3.5 };
    }

    /// Doors and stairs: walking into one fades out, moves you, fades in (Kaigen `world_doors`).
    fn doors(&mut self, ctx: &mut Ctx, dt: f32) {
        self.door_cooldown = (self.door_cooldown - dt).max(0.0);
        if let Some((t, to, yaw)) = self.door {
            let t2 = t + dt;
            if t < DOOR_OUT && t2 >= DOOR_OUT {
                self.move_to(ctx, to.x, to.y, yaw);
                self.doors_used += 1;
            }
            self.door = if t2 >= DOOR_OUT + DOOR_IN { None } else { Some((t2, to, yaw)) };
            return;
        }
        if self.busy || self.door_cooldown > 0.0 {
            return;
        }
        let Some((p, _)) = self.hero_pos(ctx) else { return };
        let (x, z) = (p.x / STUD, p.z / STUD);
        if let Some((to, yaw)) = rooms::door_at(self.area, x, z) {
            self.door = Some((0.0, to, yaw));
            self.door_cooldown = DOOR_OUT + DOOR_IN + 0.6;
        }
    }

    /// The screen's black while going through a door.
    fn door_dark(&self) -> f32 {
        match self.door {
            Some((t, _, _)) if t < DOOR_OUT => t / DOOR_OUT,
            Some((t, _, _)) => 1.0 - (t - DOOR_OUT) / DOOR_IN,
            None => 0.0,
        }
    }

    /// Inside, the sun and the sky light dim (eased): the engine's `sun_scale` / `ambient_scale` apply after the day
    /// cycle, so the cycle's inputs (and the sky outside) stay as they are.
    fn indoor_light(&mut self, ctx: &mut Ctx, dt: f32) {
        let want = if self.area.is_some() { 1.0 } else { 0.0 };
        self.indoor += (want - self.indoor) * (dt * 6.0).min(1.0);
        if self.door.is_some() {
            self.indoor = want; // behind the black: no visible fade
        }
        let k = self.indoor;
        let s = &mut ctx.renderer.settings;
        s.sun_scale = 1.0 + (INDOOR_SUN - 1.0) * k;
        s.ambient_scale = 1.0 + (INDOOR_SKY - 1.0) * k;
    }

    fn npc_index(&self, id: &str) -> Option<usize> {
        self.npcs.iter().position(|n| n.id == id)
    }

    /// Where a villager stands (feet, metres).
    fn npc_pos(&self, ctx: &Ctx, i: usize) -> Option<Vec3> {
        ctx.world.get::<&Transform>(self.npcs.get(i)?.fig.root).ok().map(|t| t.translation)
    }

    /// The talk spot in reach (Kaigen `world_pick`): the nearest within its radius.
    fn pick_target(&self, p: Vec3, save: &crate::save::GameSave) -> Option<usize> {
        let (x, z) = (p.x / STUD, p.z / STUD);
        let mut best = None;
        let mut best_d = f32::MAX;
        for (i, t) in TARGETS.iter().enumerate() {
            if (i == talk::SUIT_TARGET && !dive::suit_here(save)) || (i == talk::CHEST_TARGET && !self.life.chest_ready()) {
                continue;
            }
            let (tx, tz) = if i == talk::PIP_TARGET {
                (self.life.pip.x, self.life.pip.y)
            } else if i == talk::CHEST_TARGET {
                (self.life.chest_p.x, self.life.chest_p.z)
            } else if i == talk::GATE_TARGET && save.gate_open {
                (town::GATE_PROF_OPEN[0], t.z)
            } else {
                (t.x, t.z)
            };
            let r = talk::reach(i);
            let d = (tx - x).powi(2) + (tz - z).powi(2);
            if d < r * r && d < best_d {
                best_d = d;
                best = Some(i);
            }
        }
        best
    }

    /// E at a talk spot: turn to the villager nearest to you (they may have walked), the camera goes to your
    /// eyes, their dialog opens (one code path for the key, the touch button and the `talk` command).
    fn talk(&mut self, ctx: &mut Ctx, data: &mut Data, target: usize) {
        let Some((p, _)) = self.hero_pos(ctx) else { return };
        let t = &TARGETS[target];
        let mut at = if target == talk::PIP_TARGET { self.life.pip * STUD } else { Vec2::new(t.x, t.z) * STUD };
        let mut best = 6.0 * STUD;
        self.talk_npc = None;
        self.talk_board = None;
        if t.npc.is_empty() {
            // a machine: the camera looks at its screen (no villager nearby turns to you)
            best = 0.0;
            if t.dialog == DialogId::Machine {
                let [mx, my, mz] = rooms::MACHINE_SCREEN;
                self.talk_board = Some(talk::Board { c: town::m(mx, my - 1.2, mz), yaw: 0.0, w: 4.4, h: 5.2, max_dist: 0.0, tilt: 0.0 });
            }
        }
        for i in 0..self.npcs.len() {
            if let Some(q) = self.npc_pos(ctx, i) {
                let d = Vec2::new(q.x - p.x, q.z - p.z).length();
                if d < best {
                    best = d;
                    at = Vec2::new(q.x, q.z);
                    self.talk_npc = Some(i);
                }
            }
        }
        let d = at - Vec2::new(p.x, p.z);
        self.face_yaw = Some((-d.x).atan2(-d.y)); // the controller turns its -z to the yaw
        self.mom_talked |= t.dialog == DialogId::Mom;
        self.kid_talked |= t.dialog == DialogId::Pip;
        data.sound.sfx(ctx, Sfx::Blip, 1.2, 0.4);
        self.dlg.open(ctx, &mut data.sound, dialog_for(t.dialog, &data.save.data));
    }

    /// E / Use at a target (Kaigen `world_pick` + the T_* cases of app_update).
    fn use_target(&mut self, ctx: &mut Ctx, data: &mut Data, i: usize) {
        let t = &TARGETS[i];
        let weather = self.life.weather_code();
        let seed = (ctx.time.frame as u32).wrapping_mul(2654435761) | 1;
        match t.kind {
            Use::Talk if i == talk::PIP_TARGET && self.life.pip_hidden() => {
                // hide and seek: found him, no chat
                self.life.pip_found(&mut data.save.data);
            }
            Use::Talk => return self.talk(ctx, data, i),
            Use::Feed => {
                let p = self.hero_pos(ctx).map(|h| h.0 / STUD).unwrap_or_default();
                self.life.feed(&mut data.save.data, p);
            }
            Use::Sit => {
                if self.life.sit(&mut data.save.data, true) {
                    ctx.renderer.settings.day_length_minutes = crate::DAY_MINUTES / 24.0;
                    self.move_to(ctx, life::BENCH_SEAT[0], life::BENCH_SEAT[1], std::f32::consts::PI);
                }
            }
            Use::Chest => self.life.open_chest(&mut data.save.data),
            Use::Read(r) => {
                // a sign opens and closes its text (the camera flies to a board)
                self.reading = if self.reading == Some(r) { None } else { Some(r) };
            }
            Use::Tv => self.scr.open(ctx, &mut data.sound, ScreenKind::Tv, t.x, t.z, weather, seed),
            Use::Piano => self.scr.open(ctx, &mut data.sound, ScreenKind::Piano, t.x, t.z, weather, seed),
            Use::Pc => self.scr.open(ctx, &mut data.sound, ScreenKind::Pc, t.x, t.z, weather, seed),
            Use::Lab => self.scr.open(ctx, &mut data.sound, ScreenKind::Lab, t.x, t.z, weather, seed),
            Use::Sleep => self.sleep(ctx, data),
            Use::Suit => {
                let s = &mut data.save.data;
                s.has_suit = true;
                data.save.mark();
                data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8);
                self.life.toast("Prof. Quill's old diving suit! Now the pond can take you down.");
            }
        }
        data.sound.sfx(ctx, Sfx::Blip, 1.2, 0.4);
    }

    /// Your bed (Kaigen `life_sleep`): at night you sleep until the morning, by day a nap until the evening.
    fn sleep(&mut self, ctx: &mut Ctx, data: &mut Data) {
        let h = ctx.renderer.settings.time_of_day;
        let tod = ((h - 6.0) / 24.0).rem_euclid(1.0);
        let night = tod > 0.55 || tod < 0.05;
        let to: f32 = if night { 0.1 } else { 0.56 };
        ctx.renderer.settings.time_of_day = (6.0 + to * 24.0).rem_euclid(24.0);
        data.save.data.time_of_day = ctx.renderer.settings.time_of_day;
        data.save.mark();
        data.sound.sfx(ctx, Sfx::Blip, 1.0, 0.8);
        self.life.toast(if night { "Zzz\u{2026} Good morning, Brickhollow!" } else { "Zzz\u{2026} A cosy nap until the evening." });
        self.sleeps += 1;
        self.vox.rest(); // a good night: your Voxelmon is fit again
    }

    /// What a closed dialog chose (Kaigen: starting_template.c + `cg_dialog_action`).
    fn dialog_action(&mut self, ctx: &mut Ctx, data: &mut Data, act: DialogAction) {
        let s = &mut data.save.data;
        match act {
            DialogAction::None => {}
            DialogAction::OpenGate => {
                s.gate_open = true;
                data.save.mark();
                data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8);
                self.life.toast("The way to Route 1 is open!");
            }
            DialogAction::QuestPart => {
                s.quest_part = true; // now find the suit (lab quest part)
                data.save.mark();
            }
            DialogAction::Sleep => self.sleep(ctx, data),
            DialogAction::GiveStarter => self.give_starter(ctx, data),
            DialogAction::DexScared => self.dlg.open(ctx, &mut data.sound, dialog_for(DialogId::DexScared, s)),
            DialogAction::BattlePip if s.seek_played == 0 && s.seek_wins == 0 => self.dlg.open(ctx, &mut data.sound, dialog_for(DialogId::PipNotYet, s)),
            DialogAction::BattleRival => self.start_battle(ctx, data, BattleKind::Rival),
            DialogAction::BattlePractice => self.start_battle(ctx, data, BattleKind::Practice),
            DialogAction::BattlePip => self.start_battle(ctx, data, BattleKind::Pip),
            DialogAction::HideSeek => self.life.hide_seek_start(),
            DialogAction::Tag => self.life.toast("Tag! Sprint into Pip (hold Shift) and run!"),
        }
        self.pending_action = Some(act); // (the last one, for tests)
    }

    /// A dialogue chose a battle (Kaigen `cg_start_battle`): the intro fanfare and the bricks; the battle screen is
    /// pushed under them (`arena::TRANS_BOARD`).
    fn start_battle(&mut self, ctx: &mut Ctx, data: &mut Data, kind: BattleKind) {
        if data.trans.is_some() {
            return;
        }
        arena::start_transition(ctx, data, kind);
    }

    /// The seed of the next battle.
    fn next_seed(&mut self, ctx: &mut Ctx, data: &mut Data) -> u32 {
        match self.battle_seed.take() {
            Some(s) => s,
            None => data.cg_rand(ctx),
        }
    }

    /// The battle is over and the result was seen (Kaigen `cg_battle_done`): studs, EXP, cards, then what the opponent
    /// says (the reward packs first, if any).
    fn battle_done(&mut self, ctx: &mut Ctx, data: &mut Data, out: BattleOut) {
        data.sound.music(ctx, Track::Home);
        let mut seed = if data.cg_seed == 0 { data.cg_rand(ctx) } else { data.cg_seed };
        let s = &mut data.save.data;
        let r = out.result;
        let delta = match r {
            Outcome::Win => {
                s.wins += 1;
                out.stakes
            }
            Outcome::Lose => {
                s.losses += 1;
                -out.stakes
            }
            Outcome::Retreat => -(out.stakes / 2).max(1),
            _ => 0,
        };
        s.studs = (s.studs as i32 + delta).max(0) as u32;
        let exp = cards::battle_exp(r.code(), matches!(out.kind, BattleKind::Rival | BattleKind::Pip));
        s.exp += exp;
        let mut ids = vec![];
        if r == Outcome::Win {
            let k = if (out.kind == BattleKind::Rival && s.rival_wins == 0) || (out.kind == BattleKind::Pip && s.pip_wins == 0) { 2 } else { 1 };
            for _ in 0..k {
                if let Some(c) = cards::reward_card(s, &mut seed, false) {
                    ids.push(c);
                }
            }
            match out.kind {
                BattleKind::Rival => s.rival_wins += 1,
                BattleKind::Pip => s.pip_wins += 1,
                _ => {}
            }
        }
        data.cg_seed = seed;
        data.save.mark();
        let win = r == Outcome::Win;
        let after = match out.kind {
            BattleKind::Pip => if win { DialogId::PipWon } else { DialogId::PipLost },
            BattleKind::Rival => if win { DialogId::DexWon } else { DialogId::DexLost },
            _ => if win { DialogId::LumiWon } else { DialogId::LumiLost },
        };
        self.life.toast(&format!("+{exp} EXP \u{b7} upgrade your cards in the Deck menu (C)"));
        self.battles += 1;
        log_info!("battle done: {r:?} vs {:?}, studs {delta:+}, +{exp} EXP, cards {ids:?}", out.kind);
        if ids.is_empty() {
            self.dlg.open(ctx, &mut data.sound, dialog_for(after, &data.save.data));
        } else {
            data.cards.show_rewards(&format!("New card added to your collection! \u{b7} +{exp} EXP"), &ids, Some(after));
        }
    }

    /// Prof. Quill's gift (Kaigen `cg_dialog_action` ACT_GIVE_STARTER): the 18 starter cards and a deck of 12,
    /// three of them shown as foil packs to tear open; her "after" dialogue follows.
    fn give_starter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        if data.save.data.has_deck {
            return;
        }
        cards::give_starter(&mut data.save.data);
        data.save.mark();
        data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8);
        data.cards.show_rewards(
            "You received your starter collection! 18 cards, and a deck of 12 (C)",
            &[cards::TITANOAK, cards::THUNDRAKE, cards::EMBERHORN],
            Some(DialogId::ProfAfter),
        );
    }

    /// The pond (Kaigen app_update): jump onto the water. With the suit you sink in and the dive begins; without it
    /// you climb back out on the bank you came from. Also the sinking itself.
    fn pond(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) {
        self.pond_cd = (self.pond_cd - dt).max(0.0);
        self.surface_fade = (self.surface_fade - dt).max(0.0);
        let Some(h) = self.hero else { return };
        let Some((p, _)) = self.hero_pos(ctx) else { return };
        let (x, y, z) = (p.x / STUD, p.y / STUD, p.z / STUD);
        if self.sink_t > 0.0 {
            // sinking: down through the water, then the cave
            self.sink_t += dt;
            let k = (self.sink_t / 0.9).min(1.0);
            let off = ctx.world.get::<&CharacterController>(h).map(|c| c.foot_offset()).unwrap_or(0.85);
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(h) {
                t.translation = town::m(x.clamp(-11.0, -5.0), 1.2 - k * 5.0, z.clamp(16.0, 21.0)) + Vec3::Y * off;
            }
            if let Ok(mut cc) = ctx.world.get::<&mut CharacterController>(h) {
                cc.velocity = Vec3::ZERO;
            }
            if k >= 1.0 {
                self.sink_t = 0.0;
                // (behind the black) you wait on the north bank; the cave takes the screen
                self.move_to(ctx, POND_EXIT[0], POND_EXIT[1], 0.0);
                for slot in DIVE_HIDES {
                    ctx.ui.set(slot, "");
                }
                data.settings.open = false;
                Vox::clear_hud(ctx);
                self.nearby = None;
                for n in &mut self.npcs {
                    let _ = ctx.world.remove_one::<WorldLabel>(n.marker);
                    n.marker_html.clear();
                }
                self.dive.start(ctx);
                self.help_hidden = true;
            }
            return;
        }
        let in_pond = x > town::POND.x0 - 0.4 && x < town::POND.x1 + 0.4 && z > town::POND.z0 - 0.4 && z < town::POND.z1 + 0.4;
        if self.busy || self.fight || self.dive.on || self.pond_cd > 0.0 || self.area.is_some() || !in_pond || y <= 0.4 {
            return;
        }
        data.sound.sfx(ctx, Sfx::Splash, 0.9, 0.9);
        if dive::can_dive(&data.save.data) {
            self.sink_t = 1e-4;
            self.life.toast("Splash! The suit takes you down...");
        } else {
            // too deep without the suit: back on the bank you came from
            let (cx, cz) = (x.clamp(-12.0, -4.0), z.clamp(15.0, 22.0));
            let d = [cx + 12.0, -4.0 - cx, cz - 15.0, 22.0 - cz];
            let m = d.iter().cloned().fold(f32::MAX, f32::min);
            let out = if m == d[0] { (-13.6, cz) } else if m == d[1] { (-2.4, cz) } else if m == d[2] { (cx, 13.6) } else { (cx, 23.4) };
            self.teleport(ctx, out.0, out.1);
            self.pond_cd = 1.0;
            self.climb_outs += 1;
            self.life.toast(if data.save.data.quest_part { "Brr! Too deep to swim. You need Prof. Quill's diving suit." } else { "Brr! The pond is deep and cold." });
        }
    }

    /// The dive is over (the screen is black): you stand on the north bank, the village comes back.
    fn surfaced(&mut self, ctx: &mut Ctx, data: &mut Data) {
        self.move_to(ctx, POND_EXIT[0], POND_EXIT[1], 0.0);
        self.pond_cd = 1.5;
        self.surface_fade = SURFACE_FADE;
        self.help_hidden = false;
        ctx.ui.set("help", &self.help());
        data.settings.applied = None;
    }

    /// The touch buttons: Use only in reach of someone or in a dialog (Kaigen touch button 0).
    fn touch_buttons(&self, ctx: &mut Ctx, cards_busy: bool) {
        if !self.touch_ui {
            return;
        }
        if cards_busy || self.fight {
            // (a fight: its own buttons only, no Jump and Use: Kaigen 8cf909e)
            ctx.ui.set("touch", "");
            return;
        }
        let use_btn = if self.dlg.is_open() || self.reading.is_some() || (self.nearby.is_some() && !self.busy) { "<button data-action='use'>Use</button> " } else { "" };
        ctx.ui.set("touch", &format!("<div class='touch-buttons'>{use_btn}<button data-action='jump'>Jump</button></div>"));
    }

    /// A sign's text (Kaigen `world_draw`): at a board only a hint (the camera shows the board), else a card.
    fn draw_sign(&self, ctx: &mut Ctx) {
        let html = match self.reading {
            None => String::new(),
            Some(r) if talk::sign_board(r).is_some() => {
                format!("<div class='bh-prompt{}'><span>{}</span></div>", if self.touch_ui { " touch" } else { "" }, if self.touch_ui { "tap Use or walk to close" } else { "E or walk to close" })
            }
            Some(r) => {
                let rd = talk::readable(r);
                let lines: String = rd.lines.iter().map(|l| format!("<div class='bh-sign-l'>{}</div>", l.replace('&', "&amp;").replace('<', "&lt;"))).collect();
                let note = rd.note.map(|n| format!("<div class='bh-sign-n'>{n}</div>")).unwrap_or_default();
                format!(
                    "<div class='bh-sign' style='background:{bg};color:{fg};border-color:{fg}'><div class='bh-sign-t'>{}</div><hr style='background:{fg}'>{lines}{note}<div class='bh-sign-c'>{}</div></div>",
                    rd.title,
                    if self.touch_ui { "(tap Use to close)" } else { "(E to close)" },
                    bg = rd.bg,
                    fg = rd.fg
                )
            }
        };
        ctx.ui.set("sign", &html);
    }

    /// The debug panel (Kaigen st_debug.c): on with the engine's debug mode (`) or the menu's switch; turning it on
    /// names what is in the middle of the screen, a click or tap on the world names what is under it.
    fn debug_panel(&mut self, ctx: &mut Ctx, actions: &[String], menu: bool, dt: f32) {
        if actions.iter().any(|a| a == "dbg_toggle") {
            self.dbg.menu_on = !self.dbg.menu_on;
        }
        let me = self.hero_pos(ctx).map(|h| h.0 / STUD).unwrap_or_default();
        let was = self.dbg.on;
        self.dbg.update(ctx, actions, me, dt);
        if self.dbg.on {
            let (w, h) = (ctx.canvas.client_width() as f32, ctx.canvas.client_height() as f32);
            let at = if !was {
                Some(Vec2::new(w * 0.5, h * 0.5))
            } else if menu {
                None
            } else if ctx.input.button_pressed(MouseButton::Left) && !ctx.input.pointer_over_panel {
                Some(ctx.input.pointer)
            } else {
                ctx.input.touch.taps.first().copied()
            };
            if let Some(c) = at {
                self.debug_pick(ctx, c);
            }
        }
        self.dbg.draw(ctx, me, self.touch_ui);
    }

    /// The debug picker at a canvas point (CSS px); also the `debug_pick` command.
    fn debug_pick(&mut self, ctx: &mut Ctx, css: Vec2) {
        let mut spheres: Vec<(String, Vec3, f32)> = vec![];
        for i in 0..self.npcs.len() {
            if self.npcs[i].area != self.area {
                continue;
            }
            if let Some(q) = self.npc_pos(ctx, i) {
                let name = match self.npcs[i].id.as_str() {
                    "rival" => "Dex",
                    "kid" => "Pip",
                    "gate-prof" => "Prof. Quill (at the gap)",
                    "mom" => "Mom",
                    "lumi" => "Lumi",
                    "lab-prof" => "Prof. Quill (lab)",
                    "knight" => "Sir Bricksworth",
                    "baker" => "Marla",
                    _ => "villager",
                };
                spheres.push((name.to_string(), q / STUD + Vec3::Y * 2.4, 1.8));
            }
        }
        let save = crate::save::GameSave { golden_mask: self.golden_seen, ..Default::default() };
        for (n, c, r) in LifeView::pick_spheres(&self.life, &save) {
            spheres.push((n.to_string(), c, r));
        }
        self.dbg.pick(ctx, css, self.hero, &spheres, |x, z| town::ROADS.iter().any(|r| r.contains(x, z)));
    }

    /// Every frame: the life's sounds, the day count, the rain's haze and loop, the lamps' glow.
    fn life_frame(&mut self, ctx: &mut Ctx, data: &mut Data) {
        for (sfx, pitch, vol) in std::mem::take(&mut self.life.sfx) {
            data.sound.sfx(ctx, sfx, pitch, vol);
        }
        if std::mem::take(&mut self.life.fish_splash) {
            if let Some(c) = self.life_view.fish_clip {
                ctx.audio.play(c, Play::new().volume(0.6));
            }
        }
        let h = ctx.renderer.settings.time_of_day;
        if let Some(last) = self.last_hours {
            if h + 12.0 < last {
                self.day += 1; // past midnight
            }
        }
        self.last_hours = Some(h);
        // rain: the air gets hazy (Kaigen: the stylized sky's haze 0.6 + 0.9 x rain)
        if !self.dive.on {
            let k = self.life.rain_level;
            let s = &mut ctx.renderer.settings;
            s.fog_density = 0.055 * (1.0 + 1.5 * k);
            s.fog_start = 90.0 * STUD * (1.0 - 0.5 * k);
        }
        self.life_view.frame(ctx, &self.life);
        self.golden_seen = data.save.data.golden_mask;
    }

    /// The goal pill, the talk prompt, the toast and the markers over the villagers.
    fn draw_hud(&mut self, ctx: &mut Ctx, data: &Data, night: f32) {
        let s = &data.save.data;
        let narrow = ctx.canvas.client_width() < 640;
        let (studs, seekfade) = self.life.hud(s, narrow);
        // the goal: outside only, not while talking, at a screen, in a fight or a close-up (Kaigen cg_quest_draw's
        // condition); phones keep the studs under where it is
        let show_goal = !self.fight && self.area.is_none() && !self.dlg.is_open() && !self.scr.is_open() && !self.zoom.active;
        let pill = if show_goal { format!("<div class='bh-goal{}'><b>GOAL</b><span>{}</span></div>", if night > 0.5 { " night" } else { "" }, talk::goal_text(s)) } else { String::new() };
        let goal = format!("{pill}{}", if narrow && !self.scr.is_open() { studs.as_str() } else { "" });
        ctx.ui.set("goal", &goal);
        ctx.ui.set("studs", if narrow || self.zoom.active { "" } else { &studs });
        ctx.ui.set("seekfade", &seekfade);
        if self.fight {
            self.title_t = 0.0; // a fight: its HP bar takes the title's place
        }
        let prompt = match self.nearby {
            Some(i) if !self.busy => {
                let t = &TARGETS[i];
                let (key, cls) = if self.touch_ui { ("", " touch") } else { ("<kbd>E</kbd>", "") };
                // hide and seek: E finds him
                let (verb, label) = if i == talk::PIP_TARGET && self.life.pip_hidden() { ("Found you,", "Pip!") } else { (t.verb, t.label) };
                format!("<div class='bh-prompt{cls}'>{key}<span>{verb} {label}</span></div>")
            }
            _ => String::new(),
        };
        ctx.ui.set("prompt", &prompt);
        let title = if self.title_t > 0.0 {
            format!("<div class='bh-title' style='opacity:{:.2}'>{}</div>", (self.title_t * 2.0).min(1.0), self.title)
        } else {
            String::new()
        };
        ctx.ui.set("title", &title);
        let dark = self.door_dark().max(self.surface_fade / SURFACE_FADE);
        ctx.ui.set("fade", &if dark > 0.0 { format!("<div class='bh-fade' style='opacity:{dark:.2}'></div>") } else { String::new() });
        let toast = if self.life.toast_t > 0.0 { format!("<div class='bh-toast'>{}</div>", self.life.toast) } else { String::new() };
        ctx.ui.set("toast", &toast);
        // markers: "!" over whoever matters for the goal, "…" for small talk; none while the camera is at a face
        let me = self.hero_pos(ctx).map(|(p, _)| p).unwrap_or_default();
        for i in 0..self.npcs.len() {
            let d = self.npc_pos(ctx, i).map(|q| (q - me).length()).unwrap_or(f32::MAX);
            // (Pip hiding has no marker: it would give him away)
            let hiding = self.npcs[i].id == "kid" && self.life.pip_mode == life::PipMode::Hiding;
            let show = !self.zoom.active && d < 40.0 * STUD && d > 1.2 * STUD && self.npcs[i].area == self.area && !hiding;
            let big = talk::marker_big(&self.npcs[i].id, s, self.mom_talked, self.kid_talked);
            let html = if !show {
                String::new()
            } else if big {
                "!".into()
            } else {
                "\u{2026}".into()
            };
            let n = &mut self.npcs[i];
            if html == n.marker_html {
                continue;
            }
            n.marker_html = html.clone();
            if html.is_empty() {
                let _ = ctx.world.remove_one::<WorldLabel>(n.marker);
            } else {
                let class = if big { "bh-mark big" } else { "bh-mark" };
                let _ = ctx.world.insert_one(n.marker, WorldLabel { html, offset: Vec3::ZERO, class: class.into() });
            }
        }
    }
}

impl Screen<Data> for PlayScreen {
    fn name(&self) -> &'static str {
        "play"
    }

    fn enter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.ensure(ctx);
        setup_look(ctx);
        data.settings.applied = None; // setup_look reset the graphics: apply the menu's level again
        data.save.data.plays += 1;
        data.save.mark();
        ctx.renderer.settings.time_of_day = data.save.data.time_of_day.rem_euclid(24.0);
        if data.town.is_none() {
            data.town = Some(TownAssets::new(ctx));
        }
        // the LEGO models are small and seen from 16 m: full detail (the engine's vertex-clustered
        // LOD levels misshaped the figures and popped while walking)
        // (tried auto_lod with lod_distance 3 on engine 890610b: no level switches at this camera
        // distance, still 860k triangles, so nothing gained)
        ctx.gltf_defaults.auto_lod = false;
        let (solids, stats, occluders, tufts) = town::build(ctx, data.town.as_mut().unwrap());
        self.grass = crate::grass::Grass::new(tufts);
        self.fountain = crate::fountain::Fountain::spawn(ctx);
        self.life.blocks = solids.iter().map(|s| s.r).collect();
        self.life_view = LifeView::spawn(ctx, data.town.as_mut().unwrap(), &self.life);
        self.fader = crate::fade::Fader::new(occluders);
        self.stats = stats;
        for l in town::LAMPS {
            self.lamps.push(ctx.world.spawn((Transform::from_translation(town::m(l[0], 5.6, l[1])), PointLight { color: engine::renderer::srgb_hex(0xffd27a).truncate(), intensity: 0.0, range: 16.0 * STUD })));
        }

        // you: a capsule the size of the minifig (1.86 m), walking 8.5 studs/s like the original
        let cc = CharacterController { radius: 0.32, half_height: 0.62, speed: 8.5 * STUD, run_multiplier: 1.6, turn_speed: 14.0, step_height: 0.3, ..Default::default() };
        let foot = cc.foot_offset();
        let hero = ctx.spawn_character("hero", town::m(START[0], 0.0, START[1]), cc);
        // the figure faces +z, the controller turns its entity's -z to the walking direction
        let fig = Figure::spawn(ctx, "player", Transform::from_xyz(0.0, -foot, 0.0).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)), Some(hero));
        if let Ok(mut c) = ctx.world.get::<&mut CharacterController>(hero) {
            c.yaw = std::f32::consts::PI; // facing south (towards the camera), like waking up
        }
        if let Ok(mut t) = ctx.world.get::<&mut Transform>(hero) {
            t.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        }
        self.hero = Some(hero);
        self.fig = Some(fig);

        // the villagers: outside, then in the rooms (Mom, Lumi, the Professor in her lab)
        let all: Vec<(&str, &str, f32, f32, f32, f32)> = town::NPCS
            .iter()
            .map(|n| (n.id, n.style, n.x, if town::on_road(n.x, n.z) { 0.4 } else { 0.0 }, n.z, n.rot))
            .chain(rooms::ROOM_NPCS.iter().map(|n| (n.0, n.1, n.2, rooms::FLOOR, n.3, n.4)))
            .collect();
        for (id, style, x, y, z, rot) in all {
            let f = Figure::spawn(ctx, style, Transform::from_translation(town::m(x, y, z)).with_rotation(Quat::from_rotation_y(rot)), None);
            let area = rooms::area_at(x, z);
            ctx.set_layer(f.root, if area.is_some() { rooms::layer_of(area) } else { town::layer_at(x, z) });
            // a kinematic capsule (as big as the hero's) so you bump into people instead of walking through them;
            // the figure's root is at its feet, so the capsule is lifted by its half height
            let (hh, r) = (0.62, 0.32);
            ctx.physics.add(
                &mut ctx.world,
                f.root,
                engine::physics::rapier::prelude::RigidBodyBuilder::kinematic_position_based(),
                engine::physics::rapier::prelude::ColliderBuilder::capsule_y(hh, r).translation(Vec3::Y * (hh + r)),
            );
            // the marker hangs a little above the head (Kaigen: 2.45 m above the feet)
            let marker = ctx.world.spawn((Transform::from_xyz(0.0, 2.45, 0.0), Parent(f.root), Name(format!("marker:{id}"))));
            self.npcs.push(Npc { id: id.to_string(), area, fig: f, yaw: rot, marker, marker_html: String::new() });
        }
        // the rooms' warm ceiling lights (on only in the room you are in)
        for r in rooms::ROOMS {
            for (p, k) in rooms::room_lights(r) {
                let e = ctx.world.spawn((Transform::from_translation(town::m(p.x, p.y, p.z)), PointLight { color: Vec3::new(1.0, 0.84, 0.62), intensity: 0.0, range: 9.0 }));
                self.room_lights.push((r, e, k));
            }
        }
        // the Deck-o-Matic's screen (the lab's middle machine): MISSING PART until the deck is printed
        let [mx, my, mz] = rooms::MACHINE_SCREEN;
        let missing = ctx.load_gltf("assets/signs/machine_missing.glb", Transform::from_translation(town::m(mx, my, mz)));
        let ready = ctx.load_gltf("assets/signs/machine_ready.glb", Transform::from_translation(town::m(mx, my, mz)));
        self.machine = Some((missing, ready));

        // the fixed top-down camera of the original: (0, 32, 17) studs * zoom above you, looking at your feet + 1.5 studs
        let off = CAM_OFFSET * ZOOM * STUD;
        let cam = ThirdPersonCamera {
            pivot_offset: Vec3::Y * (1.5 * STUD - foot),
            yaw: 0.0,
            pitch: off.y.atan2(off.z),
            min_pitch: off.y.atan2(off.z),
            max_pitch: off.y.atan2(off.z),
            distance: off.length(),
            current_distance: off.length(),
            want_distance: off.length(),
            min_distance: off.length() * 0.55,
            max_distance: off.length() * 1.4,
            collide: false,
            follow_rate: 4.0,
            ..ThirdPersonCamera::following(hero)
        };
        let camera = Camera { fov_y_deg: 38.0, near: 0.5 * STUD, far: 400.0 * STUD, ..Camera::default() };
        self.camera = Some(ctx.world.spawn((Transform::from_translation(town::m(START[0], 0.0, START[1]) + off), camera, cam)));
        self.area = None;
        self.show_area(ctx);
        ctx.ui.set("help", &self.help());
        self.vox.init(ctx);
        self.dive.init(ctx, data.town.as_mut().unwrap());
    }

    fn pause(&mut self, ctx: &mut Ctx, _data: &mut Data) {
        // a battle covers the village: its HUD and the markers go
        for slot in ["help", "touch", "goal", "prompt", "toast", "dialog", "menubtn", "settings", "title", "fade", "sign", "deck_btn", "vox_wild", "vox_mine", "vox_pill", "studs", "seekfade", "debug"] {
            ctx.ui.set(slot, "");
        }
        self.life_view.stop(ctx, 0.4); // (the rain is outside; it plays again once the village is back)
        self.swipe.stop();
        for n in &mut self.npcs {
            let _ = ctx.world.remove_one::<WorldLabel>(n.marker);
            n.marker_html.clear();
        }
    }

    fn resume(&mut self, ctx: &mut Ctx, data: &mut Data) {
        ctx.ui.set("help", &self.help());
        self.shown_area = None;
        self.show_area(ctx);
        if let Some(out) = data.battle_out.take() {
            self.battle_done(ctx, data, out);
        }
    }

    fn exit(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.trans = None;
        ctx.ui.set("bt_trans", "");
        data.save.data.time_of_day = ctx.renderer.settings.time_of_day;
        data.save.flush(ctx);
        data.sound.stop_voice(ctx);
        data.settings.open = false;
        let s = &mut ctx.renderer.settings;
        (s.sun_scale, s.ambient_scale) = (1.0, 1.0);
        self.scr.close(ctx, &mut data.sound);
        data.cards.deck_open = false;
        self.dive.abort(ctx);
        ctx.renderer.set_pushers(&[]);
        self.life_view.stop(ctx, 0.0);
        ctx.renderer.settings.day_length_minutes = crate::DAY_MINUTES;
        ctx.input.touch.set_stick_enabled(true);
        for slot in ["dive", "dive_fx", "help", "touch", "goal", "prompt", "toast", "dialog", "menubtn", "settings", "title", "fade", "sign", "deck_btn", "deck_head", "deck_coll", "deck_marks", "deck_side", "deck_strip", "packs", "upgrade", "vox_wild", "vox_mine", "vox_pill", "studs", "seekfade", "debug"] {
            ctx.ui.set(slot, "");
        }
    }

    fn fixed_update(&mut self, ctx: &mut Ctx, data: &mut Data) {
        // the camera never turns (the original's fixed view): W is always north (-z)
        if let Some(mut c) = self.camera.and_then(|c| ctx.world.get::<&mut ThirdPersonCamera>(c).ok()) {
            c.yaw = 0.0;
        }
        let dt = ctx.time.fixed_dt;
        let space = ctx.input.key_down_or_pressed("Space");
        if self.jump_block && !ctx.input.key("Space") {
            self.jump_block = false;
        }
        let mut axis = ctx.input.move_axis();
        let mut jump = (space && !self.jump_block) || std::mem::take(&mut self.jump_tap);
        let mut run = ctx.input.key("ShiftLeft") || ctx.input.key("ShiftRight") || ctx.input.touch.axis.length() > TOUCH_RUN;
        self.sprinting = run && axis.length() > 0.2;
        // swipe walking: keep going that way (keys and the stick take over while used); anything busy stops it
        if self.swipe.on {
            let feet = self.hero_pos(ctx).map(|h| h.0);
            if self.busy || self.fight || self.life.sitting || !self.swipe_mode || axis.length() > 0.05 {
                self.swipe.stop();
            } else if let Some(wd) = feet.and_then(|f| self.swipe.step(ctx, f, self.hero, self.clock, self.area)) {
                axis = Vec2::new(wd.x, -wd.y);
            } else {
                self.swipe.stop();
            }
        }
        if self.busy || self.life.sitting || self.life.fade {
            axis = Vec2::ZERO;
            jump = false;
            run = false;
        }
        // a villager in the way: the wish into them is dropped (you stop in front of them instead of sliding
        // round their capsule; Kaigen's villagers are boxes)
        if axis.length() > 0.01 {
            if let Some((p, _)) = self.hero_pos(ctx) {
                let wish = Vec2::new(axis.x, -axis.y); // world x/z
                for i in 0..self.npcs.len() {
                    if self.npcs[i].area != self.area {
                        continue;
                    }
                    let Some(q) = self.npc_pos(ctx, i) else { continue };
                    let to = Vec2::new(q.x - p.x, q.z - p.z);
                    let d = to.length();
                    if d < 0.32 * 2.0 + 0.12 && d > 1e-3 {
                        let n = to / d;
                        let into = wish.dot(n);
                        if into > 0.0 {
                            let w = wish - n * into;
                            axis = Vec2::new(w.x, -w.y);
                            self.npc_stops += 1;
                        }
                    }
                }
            }
        }
        self.move_dir = Vec2::new(axis.x, -axis.y).normalize_or_zero();
        let mut face = if self.dlg.is_open() { self.face_yaw } else { None };
        if self.fight {
            // step back from the fight (a world direction: x east, y south) and keep watching it
            axis = Vec2::new(self.vox.walk.x, -self.vox.walk.y);
            jump = false;
            run = false;
            face = self.hero_pos(ctx).and_then(|(p, _)| self.vox.face_yaw(p / STUD));
        }
        if let Some(h) = self.hero {
            let mut turned = None;
            if let Ok(mut cc) = ctx.world.get::<&mut CharacterController>(h) {
                cc.drive(axis, 0.0);
                cc.jump = jump;
                cc.run = run;
                // talking: you turn to them (as the controller turns, smoothly)
                if let Some(want) = face {
                    cc.yaw = life::wrap(cc.yaw + life::wrap(want - cc.yaw) * (1.0 - (-cc.turn_speed * dt).exp()));
                    turned = Some(cc.yaw);
                }
            }
            if let (Some(y), Ok(mut t)) = (turned, ctx.world.get::<&mut Transform>(h)) {
                t.rotation = Quat::from_rotation_y(y);
            }
        }
        // the walk cycle follows the simulation (deterministic, also while stepping a paused game)
        let (v, run) = self
            .hero
            .and_then(|h| ctx.world.get::<&CharacterController>(h).ok().map(|c| (c.velocity, c.run)))
            .unwrap_or((Vec3::ZERO, false));
        self.speed = Vec2::new(v.x, v.z).length();
        if let Some(mut f) = self.fig.take() {
            f.pose = if self.life.sitting { Pose::Sit } else if self.life.dancing { Pose::Dance } else { Pose::Normal };
            f.spin = if self.life.dancing { f.spin + dt * 2.6 } else { 0.0 };
            f.air = self.hero.and_then(|h| ctx.world.get::<&CharacterController>(h).ok().map(|c| !c.grounded)).unwrap_or(false) && !self.life.sitting;
            f.animate(ctx, dt, self.speed > 0.3, if run { 1.6 } else { 1.0 });
            self.fig = Some(f);
        }

        self.gate_walk(ctx, dt);
        // ---- the tall grass: whoever walks through pushes the tufts aside ----
        if self.area.is_none() {
            let mut push: Vec<(Vec3, f32)> = vec![];
            if let Some((p, _)) = self.hero_pos(ctx) {
                push.push((p, 0.3));
            }
            for (i, n) in self.npcs.iter().enumerate() {
                if n.area.is_none() {
                    if let Some(q) = self.npc_pos(ctx, i) {
                        push.push((q, 0.3));
                    }
                }
            }
            push.extend(self.vox.pushers());
            self.grass.set_pushers(push, dt);
        } else {
            self.grass.set_pushers(vec![], dt);
        }
        // ---- village life: golden studs, ducks, ball, chest, weather, Pip; whoever you talk to turns to you ----
        let feet = self.hero_pos(ctx).map(|(p, _)| p / STUD).unwrap_or_default();
        let me = Vec2::new(feet.x, feet.z);
        let kid = self.npc_index("kid");
        let talking_pip = self.dlg.is_open() && self.talk_npc.is_some() && self.talk_npc == kid;
        let input = LifeIn {
            p: feet,
            move_dir: self.move_dir,
            speed_m: self.speed,
            sprinting: self.sprinting,
            outside: self.area.is_none() && !self.dive.on && self.sink_t <= 0.0,
            night: ctx.renderer.sky.night,
            day: self.day,
            talking_pip,
        };
        self.life.step(&mut data.save.data, &input, dt);
        if std::mem::take(&mut self.life.dirty) {
            data.save.mark();
        }
        self.life_view.sync(ctx, &self.life, &data.save.data);
        let dancing = self.life.dancing;
        if let Some(k) = kid {
            let (pos, yaw, sp) = (self.life.pip, self.life.pip_yaw, self.life.pip_speed);
            let walk = self.life.pip_mode == life::PipMode::Walk;
            let n = &mut self.npcs[k];
            n.yaw = yaw;
            n.fig.pose = if dancing && walk { Pose::ArmsUp } else { self.life.pip_pose };
            n.fig.spin = if dancing && walk { n.fig.spin + dt * 4.0 } else { 0.0 };
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.fig.root) {
                t.translation = town::m(pos.x, 0.0, pos.y);
                t.rotation = Quat::from_rotation_y(yaw);
            }
            n.fig.animate(ctx, dt, sp > 0.1, (sp / life::PIP_WALK * 0.6).clamp(0.6, 1.6));
        }
        // the villagers: within 7 m they turn their head to you (Kaigen fig_animate); dancing, those within 10 studs
        // dance along (arms up)
        for i in 0..self.npcs.len() {
            let Some(q) = self.npc_pos(ctx, i) else { continue };
            let d = feet * STUD - q;
            let near = Vec2::new(d.x, d.z).length() < 7.0 && self.npcs[i].area == self.area;
            let n = &mut self.npcs[i];
            n.fig.head_want = if near && (Some(i) != kid || self.life.pip_speed < 0.1) { life::wrap(d.x.atan2(d.z) - n.yaw) } else { 0.0 };
            if Some(i) == kid || n.id == "gate-prof" {
                continue; // (animated above / by their walk)
            }
            if near || n.fig.head_turned() || n.fig.pose != Pose::Normal || (dancing && (q / STUD - feet).length() < 10.0) {
                n.fig.pose = if dancing && (q / STUD - feet).length() < 10.0 { Pose::ArmsUp } else { Pose::Normal };
                n.fig.animate(ctx, dt, false, 1.0);
            }
        }
        if let (true, Some(i)) = (self.dlg.is_open(), self.talk_npc) {
            if Some(i) != kid {
                if let Some(q) = self.npc_pos(ctx, i) {
                    let want = (me.x * STUD - q.x).atan2(me.y * STUD - q.z);
                    let n = &mut self.npcs[i];
                    n.yaw += life::wrap(want - n.yaw) * (dt * 8.0).min(1.0);
                    if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.fig.root) {
                        t.rotation = Quat::from_rotation_y(n.yaw);
                    }
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Ctx, data: &mut Data, dt: f32) -> Trans<Data> {
        data.loadbar(ctx);
        // phones: once a finger touched the screen, swap the keyboard hints for touch hints and buttons
        if ctx.input.touch.touch_device && !self.touch_ui {
            self.touch_ui = true;
            ctx.ui.set("help", &self.help());
            data.settings.touch = true; // Auto: the phone profile
        }
        data.sound.update(ctx, dt);
        data.track_music(ctx, false);
        // the way into a battle: the board is set up under the bricks
        let kind = data.trans.as_ref().map(|t| t.kind);
        if arena::transition_update(ctx, data, dt) {
            if let Some(k) = kind {
                let seed = self.next_seed(ctx, data);
                return Trans::Push(Box::new(BattleScreen::new(k, seed)));
            }
        }
        if let Some((k, seed)) = self.push_battle.take() {
            return Trans::Push(Box::new(BattleScreen::new(k, seed)));
        }
        let actions = ctx.ui.take_actions();
        // ---- diving: the cave owns the screen (only the toasts stay) ----
        if self.dive.on {
            self.busy = true;
            if self.dive.update(ctx, data, &mut self.life, &actions, dt) {
                self.surfaced(ctx, data);
            } else {
                self.dive.draw(ctx, &data.save.data, self.touch_ui);
            }
            self.life.update(dt);
            self.life_frame(ctx, data);
            data.save.update(ctx, dt);
            let toast = if self.life.toast_t > 0.0 { format!("<div class='bh-toast'>{}</div>", self.life.toast) } else { String::new() };
            ctx.ui.set("toast", &toast);
            return Trans::None;
        }
        // the card screens (deck, packs) own the keys while they are open
        let cards_before = data.cards.busy();
        // Esc: the dialog's (it closes it), else the menu
        // (a screen, a sign's text and the card screens close with Esc themselves)
        let esc_free = !self.dlg.is_open() && !self.scr.is_open() && self.reading.is_none() && !cards_before && !self.fight;
        if ctx.input.key_pressed("Escape") && esc_free {
            data.settings.toggle();
        }
        let mut use_tap = false;
        for a in &actions {
            match a.as_str() {
                "jump" => self.jump_tap = true,
                "use" => use_tap = true,
                _ => {}
            }
        }
        if data.settings.update(ctx, &mut data.save, &mut data.sound, &actions, dt) == MenuOut::Title {
            return Trans::Switch(Box::new(Title));
        }
        let menu = data.settings.open;
        let pos = self.hero_pos(ctx);

        // ---- the cards: the deck screen (C / the Deck button), the foil packs, the upgrade moment ----
        let world_free = !self.dlg.is_open() && !self.scr.is_open() && self.reading.is_none() && !menu && self.door.is_none() && !self.fight;
        let out = data.cards.update(ctx, &mut data.save, &mut data.sound, &actions, world_free, dt);
        if let Some(d) = out.after {
            self.dlg.open(ctx, &mut data.sound, dialog_for(d, &data.save.data));
        }
        let cards_busy = cards_before || data.cards.busy();
        let actions: Vec<String> = if cards_busy { vec![] } else { actions };

        // ---- talking ----
        let use_key = (ctx.input.key_pressed("KeyE") || use_tap) && !menu && !cards_busy;
        if self.dlg.is_open() {
            let advance = use_key || ((ctx.input.key_pressed("Space") || ctx.input.key_pressed("Enter")) && !menu && !cards_busy);
            let clicks: Vec<String> = if menu { vec![] } else { actions.clone() };
            if let Some(act) = self.dlg.update(ctx, &mut data.sound, advance, &clicks) {
                if ctx.input.key("Space") {
                    self.jump_block = true;
                }
                self.dialog_action(ctx, data, act);
            }
        } else if self.scr.is_open() {
            let clicks: Vec<String> = if menu { vec![] } else { actions.clone() };
            self.scr.update(ctx, &mut data.sound, &data.save.data, &clicks, !menu, dt);
        } else if let Some((p, _)) = pos {
            self.nearby = if menu || cards_busy || self.fight { None } else { self.pick_target(p, &data.save.data) };
            // a sign's text closes when you walk, press Esc or are no longer at it
            if let Some(r) = self.reading {
                let at = self.nearby.map(|i| TARGETS[i].kind) == Some(Use::Read(r));
                if !at || ctx.input.move_axis().length() > 0.3 || ctx.input.key_pressed("Escape") {
                    self.reading = None;
                }
            }
            if let (true, Some(t)) = (use_key, self.nearby) {
                self.use_target(ctx, data, t);
            }
        }
        if !self.dlg.is_open() {
            self.talk_npc = None;
            self.face_yaw = None;
        }
        self.busy = self.dlg.is_open() || menu || self.door.is_some() || self.scr.is_open() || self.reading.is_some() || data.cards.busy() || data.trans.is_some() || self.life.fade;
        // ---- R (or a fall off the world) puts you back at the start (Kaigen GAME_SPAWN, R respawn) ----
        let fell = self.hero_pos(ctx).is_some_and(|(p, _)| p.y < -20.0);
        if (ctx.input.key_pressed("KeyR") && !self.busy && !self.fight) || (fell && !self.dive.on && self.sink_t <= 0.0) {
            self.life.sit(&mut data.save.data, false);
            ctx.renderer.settings.day_length_minutes = crate::DAY_MINUTES;
            self.move_to(ctx, START[0], START[1], std::f32::consts::PI);
            self.respawns += 1;
        }
        // ---- the bench: any move gets you up; Q dances (moving stops it) ----
        let moving = ctx.input.move_axis().length() > 0.2 || self.swipe.on;
        if self.life.sitting && moving && !self.busy {
            self.life.sit(&mut data.save.data, false);
            ctx.renderer.settings.day_length_minutes = crate::DAY_MINUTES;
        }
        if ctx.input.key_pressed("KeyQ") && !self.busy && !self.life.sitting && !self.fight {
            self.life.dancing = !self.life.dancing;
            if self.life.dancing {
                data.sound.sfx(ctx, Sfx::Fanfare, 1.5, 0.5);
                self.life.toast("Dance party! Everyone joins in.");
            }
        } else if moving || self.busy || self.fight {
            self.life.dancing = false;
        }
        // ---- swipe walking on phones (the menu's CONTROLS: Swipe or Stick) ----
        self.clock += dt;
        let swipe_mode = self.touch_ui && data.save.data.touch_mode == 0;
        if swipe_mode != self.swipe_mode {
            self.swipe_mode = swipe_mode;
            if !self.help_hidden {
                ctx.ui.set("help", &self.help());
            }
        }
        if !self.fight && !self.dive.on {
            let want_stick = !self.swipe_mode;
            if ctx.input.touch.stick_enabled() != want_stick {
                ctx.input.touch.set_stick_enabled(want_stick);
            }
            if self.swipe_mode && !menu && !cards_busy {
                if let Some((p, _)) = self.hero_pos(ctx) {
                    self.swipe.touch(ctx, p, self.clock, self.area);
                }
            }
        }

        // ---- your Voxelmon: it follows you; in the tall grass a fight holds you in place ----
        if let Some((p, _)) = pos {
            self.vox.hide_me = self.hero_hidden;
            self.fight = self.vox.update(ctx, data, &mut self.life, p / STUD, self.area, self.busy, &actions, dt);
        }
        // ---- the pond: with the suit it takes you down into the cave ----
        self.pond(ctx, data, dt);
        if self.sink_t > 0.0 || self.dive.on {
            self.busy = true;
            if self.dive.on {
                return Trans::None;
            }
        }

        // ---- a new game wakes up in your room upstairs, with a thought about the day ----
        self.play_t += dt;
        if !self.intro_checked && self.play_t > 1.0 && data.save.loaded {
            self.intro_checked = true;
            if !data.save.data.intro_seen {
                data.save.data.intro_seen = true;
                data.save.mark();
                self.move_to(ctx, INTRO_AT[0], INTRO_AT[1], 0.0);
                let id = if !self.touch_ui { DialogId::IntroKeys } else if data.save.data.touch_mode == 0 { DialogId::IntroSwipe } else { DialogId::IntroTouch };
                self.dlg.open(ctx, &mut data.sound, dialog_for(id, &data.save.data));
                self.busy = true;
            }
        }

        // ---- doors and stairs, the room you are in ----
        self.doors(ctx, dt);
        if self.door.is_none() {
            if let Some((p, _)) = self.hero_pos(ctx) {
                self.area = rooms::area_at(p.x / STUD, p.z / STUD);
            }
        }
        self.show_area(ctx);
        self.outdoor_zone(ctx);
        self.gate(ctx, data);
        self.indoor_light(ctx, dt);
        self.title_t = (self.title_t - dt).max(0.0);
        let ready = data.save.data.has_deck;
        if let (Some((missing, rdy)), true) = (self.machine, self.machine_ready != Some(ready)) {
            self.machine_ready = Some(ready);
            let lab = rooms::layer_of(Some(RoomId::Lab));
            ctx.set_layer(missing, if ready { rooms::HIDDEN_LAYER } else { lab });
            ctx.set_layer(rdy, if ready { lab } else { rooms::HIDDEN_LAYER });
            // the hidden one also steps 2 m down: the engine's z-fighting check does not know layers
            let [mx, my, mz] = rooms::MACHINE_SCREEN;
            for (e, shown) in [(missing, !ready), (rdy, ready)] {
                if let Ok(mut t) = ctx.world.get::<&mut Transform>(e) {
                    t.translation = town::m(mx, my, mz) - Vec3::Y * if shown { 0.0 } else { 2.0 };
                }
            }
        }
        self.dive.lab(ctx, &data.save.data);

        // a villager within talking reach (or talking) gets the finer head for the close-up (loaded once, ahead)
        if let Some((p, _)) = pos {
            for i in 0..self.npcs.len() {
                let near = self.npc_pos(ctx, i).is_some_and(|q| (q - p).length() < 4.0) && self.npcs[i].area == self.area;
                if near || self.talk_npc == Some(i) {
                    self.npcs[i].fig.want_detail(ctx);
                }
            }
        }
        // the camera at your eyes, looking at whom you talk to (head and shoulders, above the dialog box)
        let board = match (self.dlg.is_open(), self.talk_npc, pos) {
            (true, Some(i), Some((p, _))) => self.npc_pos(ctx, i).map(|q| talk::face_board(q, p)),
            (true, None, _) => self.talk_board,
            _ if self.scr.is_open() => self.scr.board,
            _ => self.reading.and_then(talk::sign_board).or(self.look),
        };
        if let Some(cam) = self.camera {
            self.zoom.update(ctx, cam, board, dt);
            if let Some((p, _)) = pos {
                // a fight in the tall grass: the camera watches the two beasts
                self.vox.camera(ctx, cam, p, dt);
                // a new Voxelmon from the deck: it rises in 3D in front of the camera
                let up = data.cards.up.as_ref().filter(|u| u.to >= cards::VOXEL_LEVEL).map(|u| (u.card, u.t));
                let eye = ctx.world.get::<&Transform>(cam).ok().map(|t| t.translation);
                self.vox.showcase(ctx, up.map(|u| u.0), up.map(|u| u.1).unwrap_or(0.0), eye, p / STUD);
            }
        }
        // the camera is where you stand (a close-up): your figure would be in the way (Kaigen show_me)
        let hide = self.zoom.active && self.zoom.t > 0.25;
        if let (Some(f), true) = (&self.fig, hide != self.hero_hidden) {
            self.hero_hidden = hide;
            ctx.set_layer(f.root, if hide { rooms::HIDDEN_LAYER } else { 0 });
        }

        // whatever stands between the camera and you fades (the camera stays)
        if let (Some((p, _)), Some(cam)) = (pos, self.camera.and_then(|c| ctx.world.get::<&Transform>(c).ok().map(|t| t.translation))) {
            let loaded = ctx.assets_pending() == 0;
            if !self.zoom.active {
                self.fader.update(ctx, cam, &[p + Vec3::Y * 0.5, p + Vec3::Y * 1.5], dt, loaded);
            }
        }
        day_colours(ctx);
        let night = ctx.renderer.sky.night;
        for l in &self.lamps {
            if let Ok(mut p) = ctx.world.get::<&mut PointLight>(*l) {
                p.intensity = night * 6.0;
            }
        }
        self.life.update(dt);
        self.life_frame(ctx, data);
        // the tall grass leans away from whoever walks there (indoors: no pushers)
        let me = if self.area.is_none() { self.hero_pos(ctx).map(|h| h.0) } else { None };
        self.grass.update(ctx, me, dt);
        let layers = self.camera.and_then(|c| ctx.world.get::<&Camera>(c).ok().map(|c| c.layers)).unwrap_or(0);
        self.fountain.set_visible(layers, self.area.is_none());
        self.fountain.update(ctx, dt);

        // ---- the save: the clock every 20 s (the day goes on where you left it) ----
        self.clock_t += dt;
        if self.clock_t > 20.0 {
            self.clock_t = 0.0;
            data.save.data.time_of_day = ctx.renderer.settings.time_of_day;
            data.save.mark();
        }
        data.save.update(ctx, dt);

        // ---- HUD ----
        self.draw_hud(ctx, data, night);
        self.dlg.draw(ctx, self.touch_ui, night);
        self.scr.draw(ctx, &data.save.data, self.zoom.t, self.touch_ui);
        self.draw_sign(ctx);
        let cards_busy = data.cards.busy();
        if cards_busy {
            ctx.ui.set("menubtn", "");
            ctx.ui.set("settings", "");
        } else {
            data.settings.draw(ctx, &data.save.data, &data.sound, self.dbg.menu_on);
        }
        let hud = !self.dlg.is_open() && !self.scr.is_open() && !self.zoom.active && !self.fight;
        data.cards.draw(ctx, &data.save, &mut data.faces, hud, self.touch_ui);
        self.touch_buttons(ctx, cards_busy);
        let narrow = ctx.canvas.client_width() < 640;
        // a fight: the help line would sit under the skill tiles
        if self.fight != self.help_hidden {
            self.help_hidden = self.fight;
            ctx.ui.set("help", &if self.fight { String::new() } else { self.help() });
        }
        if cards_busy || self.scr.is_open() {
            Vox::clear_hud(ctx);
        } else {
            self.vox.draw(ctx, self.area.is_none() && !self.dlg.is_open() && !self.zoom.active, night > 0.5, narrow);
        }
        self.debug_panel(ctx, &actions, menu, dt);
        Trans::None
    }

    fn debug_commands(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("teleport", "Move the hero (studs, like the three.js town). args: {x, z}"),
            ("time", "Set the time of day. args: {hours}"),
            ("talk", "Talk to whoever is in reach (as E). args: {} | {dialog: 'Pip'|'Dex'|'Mom'|'Prof'|'Lumi'|'Gate'|'Machine'|'Knight'|'Baker'} opens that dialog directly"),
            ("life", "Village life (life.rs). args: {freeze?: bool, pip_home?: true, rain?: bool (a shower now / dry), chest?: true (one appears now, 12..50 studs away), chest_at?: [x, z], seek?: true (as the dialog's hide and seek), tag?: true (Pip flees now), ball?: [x, z, vx?, vz?], dance?: bool, sit?: bool, fish?: true (jumps when you are near), golden?: n (that many collected)}; state in game_state.state.life"),
            ("debug_pick", "The debug panel's picker at a canvas point (CSS px), as a click with the panel on. args: {x, y} | {copy: 'pick'|'me'} (the Copy buttons) | {menu: bool} (the menu's switch)"),
            ("save_reset", "A new game: the save (progress and menu settings) back to the defaults, intro already seen (see `intro`)."),
            ("intro", "Start a new game's intro: wake up in your room with the thought about the day (as after the first load)."),
            ("enter", "Go into a room as through its door (fade, teleport). args: {room: 'Home'|'Dex'|'Lab'|'Bedroom'}"),
            ("flags", "Set save fields (quest flags, menu). args: any GameSave field, e.g. {has_deck: true, rival_wins: 1, gfx: 0, music_level: 2}"),
            ("play_sfx", "Play a synthesized effect. args: {name: blip|coin|fanfare|alert|quack|kick|splash|piano|pop, pitch?: 1}"),
            ("cards", "Cards and the save. args: {starter?: true (Prof. Quill's gift with its packs), give?: ['pebbit'|..] | 'all', exp?: n (adds), level?: {id: stage}, deck?: ['id'..], rewards?: ['id'..] (foil packs, shown only), open?: bool (deck screen), pick?: 'id'}"),
            ("card_face", "Draw a card face (cardface.rs). args: {id, level?: 0, big?: false, unknown?: false, back?: false, power?: n, texture?: false} -> {url (null until encoded), texture id}"),
            ("battle", "Start a card battle (as the dialogue does: the bricks, then the 3D table). args: {kind: 'rival'|'practice'|'pip', seed?: n, instant?: false (no transition), start?: true (false: only set the seed of the next battle)}"),
            ("look", "Debug close-up: the camera eases to look at a spot like a sign close-up. args: {at: [x, y, z] studs, yaw?: 0 (the side it is seen from, 0 = south), w?: 8, h?: 6 (studs in view), tilt?: 0.5 (radians above)}; {} = back"),
            ("tall_grass", "The tall grass tufts (grass.rs, engine bend pushers). args: {wind?: 1.0 (sway strength, 0 = still)}; state in game_state.state.grass"),
            ("vox", "Your Voxelmon and the wild fights (vox.rs). args: {force?: true (a wild one now) | 'id', shiny?: false, skill?: 0|1 (as key 1 / 2), run?: true, hp?: {me, wild}, tired?: bool, wild_idle?: bool (it never lunges), rate?: 0.22 (chance per 3 studs in the grass), seed?: n}"),
            ("dive", "The lab quest's dive (dive.rs). args: {sink?: true (as jumping into the pond with the suit), start?: true (straight into the cave), at?: [x, y, z] (cave studs; the core lies at [0, -19.6, 34]), yaw?, pitch?, skip_fade?: true, surface?: true}"),
            ("card_preview", "A 3D card in front of you with a face texture (the battle part's path). args: {id, level?: 0, big?: true, back?: false} | {clear: true}"),
        ]
    }

    fn debug_command(&mut self, ctx: &mut Ctx, data: &mut Data, name: &str, a: &Value) -> Option<Result<Value, String>> {
        match name {
            "teleport" => {
                let x = a["x"].as_f64()? as f32;
                let z = a["z"].as_f64()? as f32;
                self.teleport(ctx, x, z);
                Some(Ok(json!({"x": x, "z": z})))
            }
            "time" => {
                let h = a["hours"].as_f64()? as f32;
                ctx.renderer.settings.time_of_day = h.rem_euclid(24.0);
                Some(Ok(json!({"hours": h})))
            }
            "talk" => {
                if let Some(d) = a["dialog"].as_str() {
                    let ids = [DialogId::Mom, DialogId::Prof, DialogId::Dex, DialogId::Lumi, DialogId::Pip, DialogId::Gate, DialogId::Machine, DialogId::Knight, DialogId::Baker];
                    let id = ids.into_iter().find(|i| format!("{i:?}").eq_ignore_ascii_case(d))?;
                    self.dlg.open(ctx, &mut data.sound, dialog_for(id, &data.save.data));
                    return Some(Ok(json!({"open": d})));
                }
                let p = self.hero_pos(ctx)?.0;
                match self.pick_target(p, &data.save.data) {
                    Some(t) => {
                        self.use_target(ctx, data, t);
                        Some(Ok(json!({"talk": TARGETS[t].label})))
                    }
                    None => Some(Err("nobody in reach".into())),
                }
            }
            "life" => {
                if let Some(f) = a["freeze"].as_bool() {
                    self.life.frozen = f;
                }
                if a["pip_home"].as_bool() == Some(true) {
                    self.life.pip_home();
                }
                if let Some(r) = a["rain"].as_bool() {
                    self.life.set_rain(r);
                }
                if a["chest"].as_bool() == Some(true) {
                    self.life.chest_live = false;
                    self.life.chest_next = 0.0;
                    self.life.chest_day = None;
                }
                if let Some(c) = a["chest_at"].as_array() {
                    let f = |i: usize| c.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    self.life.chest_p = Vec3::new(f(0), 0.0, f(1));
                    self.life.chest_live = true;
                    self.life.chest_open = false;
                }
                if a["seek"].as_bool() == Some(true) {
                    self.life.hide_seek_start();
                }
                if a["tag"].as_bool() == Some(true) {
                    self.life.pip_mode = life::PipMode::Flee;
                }
                if let Some(b) = a["ball"].as_array() {
                    let f = |i: usize| b.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    self.life.ball_p = Vec3::new(f(0), 0.6, f(1));
                    self.life.ball_v = Vec3::new(f(2), 0.0, f(3));
                }
                if let Some(d) = a["dance"].as_bool() {
                    self.life.dancing = d;
                }
                if let Some(on) = a["sit"].as_bool() {
                    if on {
                        self.use_target(ctx, data, talk::SIT_TARGET);
                    } else if self.life.sit(&mut data.save.data, false) {
                        ctx.renderer.settings.day_length_minutes = crate::DAY_MINUTES;
                    }
                }
                if let Some(i) = a["to_fly"].as_u64() {
                    let f = self.life.fly_at[i as usize % self.life.fly_at.len()].0;
                    self.teleport(ctx, f.x, f.z);
                }
                if let Some(i) = a["to_firefly"].as_u64() {
                    let f = self.life.ff_at[i as usize % self.life.ff_at.len()].0;
                    self.teleport(ctx, f.x - 0.8, f.z);
                }
                if let Some(d) = a["to_pip"].as_f64() {
                    let (px, pz) = (self.life.pip.x, self.life.pip.y);
                    self.teleport(ctx, px, pz + d as f32);
                }
                if let Some(w) = a["wet"].as_f64() {
                    self.life.wet = w as f32;
                }
                if let Some(t) = a["seek_left"].as_f64() {
                    self.life.seek_left = t as f32;
                }
                if a["fish"].as_bool() == Some(true) {
                    self.life.fish_next = 0.0;
                }
                if let Some(n) = a["golden"].as_u64() {
                    data.save.data.golden_mask = ((1u64 << n.min(15)) - 1) as u32;
                    data.save.mark();
                }
                Some(Ok(json!({"frozen": self.life.frozen, "pip": [self.life.pip.x, self.life.pip.y], "life": self.life.describe(&data.save.data)})))
            }
            "debug_pick" => {
                if let Some(m) = a["menu"].as_bool() {
                    self.dbg.menu_on = m;
                }
                if let Some(c) = a["copy"].as_str() {
                    let me = self.hero_pos(ctx).map(|h| h.0 / STUD).unwrap_or_default();
                    self.dbg.update(ctx, &[format!("dbg_copy:{c}")], me, 0.0);
                }
                if let (Some(x), Some(y)) = (a["x"].as_f64(), a["y"].as_f64()) {
                    self.debug_pick(ctx, Vec2::new(x as f32, y as f32));
                }
                Some(Ok(self.dbg.describe()))
            }
            "save_reset" => {
                data.save.reset(ctx);
                self.gate_to = None; // (Prof. Quill back where the new save says, at once)
                // (a test's new game: no intro on the next load either; `intro` starts it)
                data.save.data.intro_seen = true;
                data.save.flush(ctx);
                self.dlg.d = None;
                data.sound.stop_voice(ctx);
                self.intro_checked = true;
                data.settings.applied = None;
                Some(Ok(json!({"reset": true})))
            }
            "intro" => {
                data.save.data.intro_seen = false;
                self.intro_checked = false;
                self.play_t = 1.0;
                Some(Ok(json!({"intro": true})))
            }
            "enter" => {
                let name = a["room"].as_str()?;
                let id = rooms::ROOMS.into_iter().find(|r| format!("{r:?}").eq_ignore_ascii_case(name))?;
                let r = rooms::room_def(id);
                let to = if id == RoomId::Bedroom { Vec2::new(207.0, 40.0) } else { Vec2::new(r.ox, r.oz + r.d * 0.5 - 2.6) };
                self.door = Some((0.0, to, 0.0));
                Some(Ok(json!({"enter": name})))
            }
            "flags" => {
                let mut v = serde_json::to_value(&data.save.data).ok()?;
                for (k, val) in a.as_object()? {
                    v[k] = val.clone();
                }
                match serde_json::from_value(v) {
                    Ok(g) => {
                        data.save.data = g;
                        data.save.mark();
                        Some(Ok(json!({"ok": true})))
                    }
                    Err(e) => Some(Err(e.to_string())),
                }
            }
            "cards" => {
                if a["starter"].as_bool() == Some(true) {
                    data.save.data.has_deck = false;
                    self.give_starter(ctx, data);
                }
                let s = &mut data.save.data;
                let ids = |v: &Value| -> Vec<usize> { v.as_array().map(|l| l.iter().filter_map(|x| x.as_str().and_then(cards::by_id)).collect()).unwrap_or_default() };
                if a["give"].as_str() == Some("all") {
                    for c in 0..cards::CARD_POOL {
                        cards::give_card(s, c);
                    }
                } else {
                    for c in ids(&a["give"]) {
                        cards::give_card(s, c);
                    }
                }
                if let Some(e) = a["exp"].as_u64() {
                    s.exp += e as u32;
                }
                if let Some(m) = a["level"].as_object() {
                    s.card_level.resize(cards::CARD_COUNT, 0);
                    for (k, v) in m {
                        if let Some(c) = cards::by_id(k) {
                            s.card_level[c] = v.as_u64().unwrap_or(0).min(4) as u8;
                        }
                    }
                }
                if a["deck"].is_array() {
                    s.deck = ids(&a["deck"]).into_iter().take(cards::DECK_SIZE).map(|c| c as u8).collect();
                }
                data.save.mark();
                if a["rewards"].is_array() {
                    data.cards.show_rewards("Test packs", &ids(&a["rewards"]), None);
                }
                if let Some(o) = a["open"].as_bool() {
                    if o {
                        data.cards.open_deck(&mut data.sound, ctx);
                    } else {
                        data.cards.deck_open = false;
                    }
                }
                if let Some(p) = a["pick"].as_str() {
                    data.cards.pick = cards::by_id(p);
                }
                Some(Ok(data.cards.describe(&data.save, &data.faces)))
            }
            "battle" => {
                let kind = BattleKind::from_name(a["kind"].as_str().unwrap_or("practice"))?;
                if let Some(sd) = a["seed"].as_u64() {
                    self.battle_seed = Some(sd as u32);
                }
                if a["start"].as_bool() == Some(false) {
                    return Some(Ok(json!({"seed": self.battle_seed})));
                }
                if a["instant"].as_bool() == Some(true) {
                    let seed = self.next_seed(ctx, data);
                    self.push_battle = Some((kind, seed));
                } else {
                    self.start_battle(ctx, data, kind);
                }
                Some(Ok(json!({"battle": kind})))
            }
            "look" => {
                self.look = a.get("at").and_then(|v| v.as_array()).map(|v| {
                    let f = |i: usize| v.get(i).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    let g = |k: &str, d: f64| a.get(k).and_then(|x| x.as_f64()).unwrap_or(d) as f32;
                    talk::Board { c: town::m(f(0), f(1), f(2)), yaw: g("yaw", 0.0), w: g("w", 8.0), h: g("h", 6.0), max_dist: 0.0, tilt: g("tilt", 0.5) }
                });
                return Some(Ok(json!({"look": self.look.is_some()})));
            }
            "tall_grass" => {
                if let Some(w) = a.get("wind").and_then(|v| v.as_f64()) {
                    self.grass.wind = w as f32;
                }
                return Some(Ok(self.grass.describe(self.hero_pos(ctx).map(|h| h.0))));
            }
            "vox" => {
                self.vox.cheat(a);
                if let Some(f) = a.get("force").filter(|f| !f.is_null() && f.as_bool() != Some(false)) {
                    self.vox.force(f.as_str().and_then(cards::by_id), a["shiny"].as_bool().unwrap_or(false));
                }
                if let Some(k) = a["skill"].as_u64() {
                    let used = self.vox.use_skill(ctx, data, k.min(1) as usize);
                    return Some(Ok(json!({"used": used, "vox": self.vox.describe(ctx)})));
                }
                if a["run"].as_bool() == Some(true) {
                    self.vox.run();
                }
                Some(Ok(self.vox.describe(ctx)))
            }
            "dive" => {
                if a["sink"].as_bool() == Some(true) && !self.dive.on {
                    self.sink_t = 1e-4;
                }
                if a["start"].as_bool() == Some(true) && !self.dive.on {
                    self.sink_t = 1.0; // the next frame ends the sinking and starts the dive
                }
                self.dive.cheat(a);
                Some(Ok(self.dive.describe(ctx, &data.save.data)))
            }
            "card_face" => {
                let f = Self::face_arg(a)?;
                let url = data.faces.url(f);
                let tex = if a["texture"].as_bool() == Some(true) { data.faces.texture(ctx, f).map(|t| t.0) } else { None };
                Some(Ok(json!({"face": f, "url": url, "texture": tex, "w": crate::cardface::FACE_W, "h": crate::cardface::FACE_H})))
            }
            "card_preview" => {
                for e in self.previews.drain(..) {
                    ctx.despawn(e);
                }
                if a["clear"].as_bool() == Some(true) {
                    return Some(Ok(json!({"cleared": true})));
                }
                let mut f = Self::face_arg(a)?;
                if a["big"].is_null() && !f.back {
                    f.big = true;
                }
                let tex = data.faces.texture(ctx, f)?;
                let mat = ctx.renderer.add_material(Material { texture: Some(tex), double_sided: true, ..Material::color(0xffffff).cutout(0.5) });
                let (p, _) = self.hero_pos(ctx)?;
                let quad = ctx.renderer.builtin.quad;
                // standing a little in front of you (towards the camera), tilted back like a card on a stand
                let t = Transform::from_translation(p + Vec3::new(0.0, 1.2, 1.4)).with_rotation(Quat::from_rotation_x(-0.5)).with_scale(Vec3::new(1.0, 1.4, 1.0));
                let e = ctx.spawn_mesh("card_preview", quad, mat, t);
                ctx.set_layer(e, rooms::layer_of(self.area));
                self.previews.push(e);
                Some(Ok(json!({"texture": tex.0, "entity": e.id()})))
            }
            "play_sfx" => {
                let n = a["name"].as_str()?;
                let s = sound::ALL_SFX.into_iter().find(|s| s.name() == n)?;
                data.sound.sfx(ctx, s, a["pitch"].as_f64().unwrap_or(1.0) as f32, 0.8);
                Some(Ok(json!({"played": n})))
            }
            _ => None,
        }
    }

    /// Everything an agent needs to assert progress (`call game_state` -> {screen, stack, state}).
    /// Positions are in studs (the three.js town's units).
    fn debug_state(&self, ctx: &Ctx, data: &Data) -> Value {
        let r = |v: f32| (v / STUD * 100.0).round() / 100.0;
        let hero = self.hero.and_then(|h| {
            let (p, yaw) = self.hero_pos(ctx)?;
            let cc = ctx.world.get::<&CharacterController>(h).ok()?;
            Some(json!({
                "id": h.id(),
                "x": r(p.x), "y": r(p.y), "z": r(p.z),
                "yaw": (yaw * 100.0).round() / 100.0,
                "grounded": cc.grounded, "jumps": cc.jumps,
                "speed": (self.speed / STUD * 10.0).round() / 10.0,
                "legs": self.fig.as_ref().map(|f| (f.leg_swing() * 100.0).round() / 100.0),
            }))
        });
        let cam = self.camera.and_then(|c| ctx.world.get::<&Transform>(c).ok().map(|t| t.translation / STUD)).map(|p| [p.x.round(), p.y.round(), p.z.round()]);
        let npcs: Vec<Value> = self
            .npcs
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let p = self.npc_pos(ctx, i).unwrap_or_default();
                // looks at you: the figure's +z within ~18 degrees of the way to you
                let me = self.hero_pos(ctx).map(|h| h.0).unwrap_or_default();
                let to = Vec2::new(me.x - p.x, me.z - p.z).normalize_or_zero();
                let facing_you = Vec2::new(n.yaw.sin(), n.yaw.cos()).dot(to) > 0.95;
                json!({"id": n.id, "x": r(p.x), "z": r(p.z), "yaw": (n.yaw * 100.0).round() / 100.0, "facing_you": facing_you, "marker": n.marker_html, "head": (n.fig.head_yaw() * 100.0).round() / 100.0, "pose": n.fig.pose})
            })
            .collect();
        let dlg = self.dlg.d.as_ref().map(|d| {
            json!({
                "id": format!("{:?}", d.id), "speaker": d.speaker, "line": self.dlg.index, "lines": d.lines.len(),
                "text": d.lines.get(self.dlg.index).map(|l| l.text.clone()),
                "voice": d.lines.get(self.dlg.index).and_then(|l| l.voice),
                "choices": d.choices.iter().map(|c| c.0).collect::<Vec<_>>(), "hover": self.dlg.hover,
            })
        });
        let g = &data.save.data;
        json!({
            "hero": hero,
            "area": self.area.map(|r| format!("{r:?}")),
            "door": self.door.map(|d| (d.0 * 100.0).round() / 100.0),
            "doors_used": self.doors_used,
            "title": if self.title_t > 0.0 { Some(self.title) } else { None },
            "zone": town::ZONE_NAMES[self.zone],
            "bands": self.bands,
            "gate_stops": self.gate_stops,
            "indoor": (self.indoor * 100.0).round() / 100.0,
            "camera_layers": self.camera.and_then(|c| ctx.world.get::<&Camera>(c).ok().map(|c| c.layers)),
            "room_lights_on": self.room_lights.iter().filter(|(_, e, _)| ctx.world.get::<&PointLight>(*e).map(|l| l.intensity > 0.0).unwrap_or(false)).count(),
            "machine": self.machine_ready.map(|r| if r { "ready" } else { "missing" }),
            "hero_hidden": self.hero_hidden,
            "screen": self.scr.describe(),
            "reading": self.reading,
            "sleeps": self.sleeps,
            "sun": (ctx.renderer.settings.sun_intensity * 100.0).round() / 100.0,
            "camera": cam,
            "town": self.stats,
            "hiding": self.fader.hiding,
            "faded": self.fader.faded(),
            "fade_unresolved": self.fader.unresolved(),
            "npcs": npcs,
            "assets_pending": ctx.assets_pending(),
            "time": (ctx.renderer.settings.time_of_day * 100.0).round() / 100.0,
            "night": (ctx.renderer.sky.night * 100.0).round() / 100.0,
            "lamps": self.lamps.iter().filter_map(|l| ctx.world.get::<&PointLight>(*l).ok().map(|p| (p.intensity * 100.0).round() / 100.0)).collect::<Vec<_>>(),
            "lods_left": ctx.world.query::<&Lod>().iter().count(),
            // (LODs only on the forest beside Route 1 and around Stonecrest: none in the village)
            "lods_village": ctx.world.query::<(&Lod, &GlobalTransform)>().iter().filter(|(_, g)| g.0.w_axis.z > town::NORTH_EDGE * STUD).count(),
            "dialog": dlg,
            "dialog_lines_shown": self.dlg.lines_shown,
            "last_dialog": self.dlg.last_closed.map(|(d, a)| json!({"id": format!("{d:?}"), "action": format!("{a:?}")})),
            "pending_action": self.pending_action.map(|a| format!("{a:?}")),
            "battle_transition": data.trans.as_ref().map(|t| (t.t * 100.0).round() / 100.0),
            "battles": self.battles,
            "fight": self.fight,
            "dive": self.dive.describe(ctx, &data.save.data),
            "sink": (self.sink_t * 100.0).round() / 100.0,
            "pond_cd": (self.pond_cd * 100.0).round() / 100.0,
            "climb_outs": self.climb_outs,
            "vox": self.vox.describe(ctx),
            "grass": self.grass.describe(self.hero_pos(ctx).map(|h| h.0)),
            "fountain": self.fountain.describe(ctx),
            "stick": ctx.input.touch.stick_enabled(),
            "nearby": self.nearby.map(|i| TARGETS[i].label),
            "talk_npc": self.talk_npc.map(|i| self.npcs[i].id.clone()),
            "talk_cam": (self.zoom.t * 100.0).round() / 100.0,
            "fine_heads": self.npcs.iter().filter(|n| n.fig.detailed()).map(|n| n.id.clone()).collect::<Vec<_>>(),
            "busy": self.busy,
            "life": self.life.describe(g),
            "rain_volume": self.life_view.rain_volume(ctx),
            "debug_panel": self.dbg.describe(),
            "swipe": {"mode": self.swipe_mode, "on": self.swipe.on, "dir": [self.swipe.dir.x, self.swipe.dir.y], "started": self.swipe.started, "dodging": self.swipe.dodging},
            "npc_stops": self.npc_stops,
            "respawns": self.respawns,
            "day": self.day,
            "day_length": ctx.renderer.settings.day_length_minutes,
            "fog_density": (ctx.renderer.settings.fog_density * 1000.0).round() / 1000.0,
            "hero_pose": self.fig.as_ref().map(|f| f.pose),
            "pip": {"x": (self.life.pip.x * 100.0).round() / 100.0, "z": (self.life.pip.y * 100.0).round() / 100.0, "mode": self.life.pip_mode, "wp": self.life.pip_wp, "speed": self.life.pip_speed},
            "toast": if self.life.toast_t > 0.0 { Some(self.life.toast.clone()) } else { None },
            "goal": talk::goal_text(g),
            "menu": {
                "open": data.settings.open, "gfx": settings::GFX_NAMES[g.gfx as usize % 4], "step": data.settings.step(g),
                "voices": !g.voices_off, "music_level": g.music_level, "fps": (data.settings.fps * 10.0).round() / 10.0,
                "shadow_cascades": ctx.renderer.settings.shadow_cascades, "msaa_pref": engine::msaa_pref(),
            },
            "sound": {
                "track": data.sound.track().name(), "music_volume": ctx.audio.music_volume(),
                "voice": data.sound.voice_url_now(), "last_voice": data.sound.last_voice, "voices_started": data.sound.voices_started,
                "last_sfx": data.sound.last_sfx, "sfx_played": data.sound.sfx_played,
            },
            "cards": data.cards.describe(&data.save, &data.faces),
            "save": {"plays": g.plays, "studs": g.studs, "exp": g.exp, "wins": g.wins, "losses": g.losses, "rival_wins": g.rival_wins, "pip_wins": g.pip_wins, "vox_wins": g.vox_wins, "voxelmon": g.voxelmon, "writes": data.save.writes, "time_of_day": (g.time_of_day * 100.0).round() / 100.0, "has_deck": g.has_deck, "gate_open": g.gate_open, "quest_part": g.quest_part, "has_suit": g.has_suit, "has_part": g.has_part, "dive_found": g.dive_found, "intro_seen": g.intro_seen},
        })
    }
}
