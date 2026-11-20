//! The play screen: the village, you, the villagers, talking, the HUD (goal, prompt, toast, markers,
//! the ☰ menu) and the touch buttons.

use crate::dialog::{dialog_for, DialogAction, DialogId, DialogState};
use crate::figure::Figure;
use crate::life::{self, Life};
use crate::settings::{self, MenuOut};
use crate::sound::{self, Sfx};
use crate::talk::{self, CamZoom, TARGETS};
use crate::town::{self, TownAssets, TownStats, STUD};
use crate::{day_colours, setup_look, Data, Title};
use engine::prelude::*;
use serde_json::{json, Value};

/// Where you start (outside, in front of your house) — the bedroom start comes with the interiors slice.
const START: [f32; 2] = [-15.0, -2.5];
/// Camera offset of the original (studs, before zoom) and its default zoom.
const CAM_OFFSET: Vec3 = Vec3::new(0.0, 32.0, 17.0);
const ZOOM: f32 = 1.15;

/// A villager in the town: id (as `town::NPCS`), figure, the way it faces (yaw, figures face +z at 0),
/// its "!" / "…" marker (a `WorldLabel` child) and that marker's current html.
struct Npc {
    id: String,
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
}

/// Help line for phones: the engine's joystick (left 40 % of the screen) walks, pushing it all the way
/// out runs, two fingers zoom.
const TOUCH_HELP: &str = "<div class='panel'><b>Brickhollow</b><br>left thumb: walk (push far = run) &middot; two fingers: zoom</div>";
const KEYS_HELP: &str = "<div class='panel'><b>Brickhollow</b><br><kbd>WASD</kbd> move &middot; <kbd>Shift</kbd> run &middot; <kbd>Space</kbd> jump &middot; <kbd>E</kbd> use / talk &middot; <kbd>Wheel</kbd> zoom &middot; <kbd>Esc</kbd> menu</div>";
/// Stick deflection from which the hero runs.
const TOUCH_RUN: f32 = 0.92;

/// The game's HUD styles (dialog box, goal pill, markers, prompt, toast, the menu), in a style slot.
pub const STYLE: &str = "<style>
#ui [data-slot=help]{position:fixed;left:16px;bottom:16px}
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
#ui [data-slot=toast]{position:fixed;left:0;right:0;top:70px;display:flex;justify-content:center}
.bh-toast{background:rgba(250,250,246,.95);color:#1b2a34;border:2px solid #1b2a34;border-radius:10px;padding:8px 16px;font-size:18px}
#ui .world-label.bh-mark{background:rgba(250,250,246,.88);color:#1b2a34;border:2px solid #1b2a34;padding:0;width:28px;height:18px;line-height:14px;text-align:center;font-weight:700;font-size:14px;border-radius:10px}
#ui .world-label.bh-mark.big{background:#f2cd37;width:24px;height:24px;line-height:24px;border-radius:14px;font-size:18px}
</style>";

impl PlayScreen {
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

    fn npc_index(&self, id: &str) -> Option<usize> {
        self.npcs.iter().position(|n| n.id == id)
    }

    /// Where a villager stands (feet, metres).
    fn npc_pos(&self, ctx: &Ctx, i: usize) -> Option<Vec3> {
        ctx.world.get::<&Transform>(self.npcs.get(i)?.fig.root).ok().map(|t| t.translation)
    }

    /// The talk spot in reach (Kaigen `world_pick`): the nearest within its radius.
    fn pick_target(&self, p: Vec3) -> Option<usize> {
        let (x, z) = (p.x / STUD, p.z / STUD);
        let mut best = None;
        let mut best_d = f32::MAX;
        for (i, t) in TARGETS.iter().enumerate() {
            let (tx, tz) = if i == talk::PIP_TARGET { (self.life.pip.x, self.life.pip.y) } else { (t.x, t.z) };
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
            DialogAction::DexScared => self.dlg.open(ctx, &mut data.sound, dialog_for(DialogId::DexScared, s)),
            DialogAction::BattlePip if s.seek_played == 0 && s.seek_wins == 0 => self.dlg.open(ctx, &mut data.sound, dialog_for(DialogId::PipNotYet, s)),
            // battles, hide and seek, tag, sleeping, the starter cards: their parts handle these
            _ => {
                log_info!("dialog action {act:?}: not implemented yet");
                self.pending_action = Some(act);
            }
        }
    }

    /// The touch buttons: Use only in reach of someone or in a dialog (Kaigen touch button 0).
    fn touch_buttons(&self, ctx: &mut Ctx) {
        if !self.touch_ui {
            return;
        }
        let use_btn = if self.dlg.is_open() || (self.nearby.is_some() && !self.busy) { "<button data-action='use'>Use</button> " } else { "" };
        ctx.ui.set("touch", &format!("<div class='touch-buttons'>{use_btn}<button data-action='jump'>Jump</button></div>"));
    }

    /// The goal pill, the talk prompt, the toast and the markers over the villagers.
    fn draw_hud(&mut self, ctx: &mut Ctx, data: &Data, night: f32) {
        let s = &data.save.data;
        ctx.ui.set("goal", &format!("<div class='bh-goal{}'><b>GOAL</b><span>{}</span></div>", if night > 0.5 { " night" } else { "" }, talk::goal_text(s)));
        let prompt = match self.nearby {
            Some(i) if !self.busy => {
                let t = &TARGETS[i];
                let (key, cls) = if self.touch_ui { ("", " touch") } else { ("<kbd>E</kbd>", "") };
                format!("<div class='bh-prompt{cls}'>{key}<span>{} {}</span></div>", t.verb, t.label)
            }
            _ => String::new(),
        };
        ctx.ui.set("prompt", &prompt);
        let toast = if self.life.toast_t > 0.0 { format!("<div class='bh-toast'>{}</div>", self.life.toast) } else { String::new() };
        ctx.ui.set("toast", &toast);
        // markers: "!" over whoever matters for the goal, "…" for small talk; none while the camera is at a face
        let me = self.hero_pos(ctx).map(|(p, _)| p).unwrap_or_default();
        for i in 0..self.npcs.len() {
            let d = self.npc_pos(ctx, i).map(|q| (q - me).length()).unwrap_or(f32::MAX);
            let show = !self.zoom.active && d < 40.0 * STUD && d > 1.2 * STUD;
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
        let (solids, stats, occluders) = town::build(ctx, data.town.as_mut().unwrap());
        self.life.blocks = solids.iter().map(|s| s.r).collect();
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

        for n in &town::NPCS {
            let f = Figure::spawn(ctx, n.style, Transform::from_translation(town::m(n.x, 0.0, n.z)).with_rotation(Quat::from_rotation_y(n.rot)), None);
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
            let marker = ctx.world.spawn((Transform::from_xyz(0.0, 2.45, 0.0), Parent(f.root), Name(format!("marker:{}", n.id))));
            self.npcs.push(Npc { id: n.id.to_string(), fig: f, yaw: n.rot, marker, marker_html: String::new() });
        }

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
        ctx.ui.set("help", KEYS_HELP);
    }

    fn exit(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.save.data.time_of_day = ctx.renderer.settings.time_of_day;
        data.save.flush(ctx);
        data.sound.stop_voice(ctx);
        data.settings.open = false;
        for slot in ["help", "touch", "goal", "prompt", "toast", "dialog", "menubtn", "settings"] {
            ctx.ui.set(slot, "");
        }
    }

    fn fixed_update(&mut self, ctx: &mut Ctx, _data: &mut Data) {
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
        if self.busy {
            axis = Vec2::ZERO;
            jump = false;
            run = false;
        }
        let face = if self.dlg.is_open() { self.face_yaw } else { None };
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
            f.animate(ctx, dt, self.speed > 0.3, if run { 1.6 } else { 1.0 });
            self.fig = Some(f);
        }

        // ---- village life: Pip walks his loop; whoever you talk to turns to you ----
        let me = self.hero_pos(ctx).map(|(p, _)| Vec2::new(p.x, p.z) / STUD).unwrap_or_default();
        let kid = self.npc_index("kid");
        let talking_pip = self.dlg.is_open() && self.talk_npc.is_some() && self.talk_npc == kid;
        self.life.step_pip(me, dt, true, talking_pip);
        if let Some(k) = kid {
            let (pos, yaw, sp) = (self.life.pip, self.life.pip_yaw, self.life.pip_speed);
            let n = &mut self.npcs[k];
            n.yaw = yaw;
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(n.fig.root) {
                t.translation = town::m(pos.x, 0.0, pos.y);
                t.rotation = Quat::from_rotation_y(yaw);
            }
            n.fig.animate(ctx, dt, sp > 0.1, 0.6);
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
        // phones: once a finger touched the screen, swap the keyboard hints for touch hints and buttons
        if ctx.input.touch.touch_device && !self.touch_ui {
            self.touch_ui = true;
            ctx.ui.set("help", TOUCH_HELP);
            data.settings.touch = true; // Auto: the phone profile
        }
        data.sound.update(ctx, dt);
        let actions = ctx.ui.take_actions();
        // Esc: the dialog's (it closes it), else the menu
        if ctx.input.key_pressed("Escape") && !self.dlg.is_open() {
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

        // ---- talking ----
        let use_key = (ctx.input.key_pressed("KeyE") || use_tap) && !menu;
        if self.dlg.is_open() {
            let advance = use_key || ((ctx.input.key_pressed("Space") || ctx.input.key_pressed("Enter")) && !menu);
            let clicks: Vec<String> = if menu { vec![] } else { actions.clone() };
            if let Some(act) = self.dlg.update(ctx, &mut data.sound, advance, &clicks) {
                if ctx.input.key("Space") {
                    self.jump_block = true;
                }
                self.dialog_action(ctx, data, act);
            }
        } else if let Some((p, _)) = pos {
            self.nearby = if menu { None } else { self.pick_target(p) };
            if let (true, Some(t)) = (use_key, self.nearby) {
                self.talk(ctx, data, t);
            }
        }
        if !self.dlg.is_open() {
            self.talk_npc = None;
            self.face_yaw = None;
        }
        self.busy = self.dlg.is_open() || menu;

        // the camera at your eyes, looking at whom you talk to (head and shoulders, above the dialog box)
        let board = match (self.dlg.is_open(), self.talk_npc, pos) {
            (true, Some(i), Some((p, _))) => self.npc_pos(ctx, i).map(|q| talk::face_board(q, p)),
            _ => None,
        };
        if let Some(cam) = self.camera {
            self.zoom.update(ctx, cam, board, dt);
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
        data.settings.draw(ctx, &data.save.data, &data.sound);
        self.touch_buttons(ctx);
        Trans::None
    }

    fn debug_commands(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("teleport", "Move the hero (studs, like the three.js town). args: {x, z}"),
            ("time", "Set the time of day. args: {hours}"),
            ("talk", "Talk to whoever is in reach (as E). args: {} | {dialog: 'Pip'|'Dex'|'Mom'|'Prof'|'Lumi'|'Gate'|'Machine'|'Knight'|'Baker'} opens that dialog directly"),
            ("life", "Village life. args: {freeze?: bool, pip_home?: true}"),
            ("save_reset", "A new game: the save (progress and menu settings) back to the defaults."),
            ("flags", "Set save fields (quest flags, menu). args: any GameSave field, e.g. {has_deck: true, rival_wins: 1, gfx: 0, music_level: 2}"),
            ("play_sfx", "Play a synthesized effect. args: {name: blip|coin|fanfare|alert|quack|kick|splash|piano|pop, pitch?: 1}"),
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
                match self.pick_target(p) {
                    Some(t) => {
                        self.talk(ctx, data, t);
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
                Some(Ok(json!({"frozen": self.life.frozen, "pip": [self.life.pip.x, self.life.pip.y]})))
            }
            "save_reset" => {
                data.save.reset(ctx);
                data.settings.applied = None;
                Some(Ok(json!({"reset": true})))
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
                json!({"id": n.id, "x": r(p.x), "z": r(p.z), "yaw": (n.yaw * 100.0).round() / 100.0, "facing_you": facing_you, "marker": n.marker_html})
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
            "dialog": dlg,
            "dialog_lines_shown": self.dlg.lines_shown,
            "last_dialog": self.dlg.last_closed.map(|(d, a)| json!({"id": format!("{d:?}"), "action": format!("{a:?}")})),
            "pending_action": self.pending_action.map(|a| format!("{a:?}")),
            "nearby": self.nearby.map(|i| TARGETS[i].label),
            "talk_npc": self.talk_npc.map(|i| self.npcs[i].id.clone()),
            "talk_cam": (self.zoom.t * 100.0).round() / 100.0,
            "busy": self.busy,
            "pip": {"x": (self.life.pip.x * 100.0).round() / 100.0, "z": (self.life.pip.y * 100.0).round() / 100.0, "mode": self.life.pip_mode, "wp": self.life.pip_wp, "speed": self.life.pip_speed},
            "toast": if self.life.toast_t > 0.0 { Some(self.life.toast.clone()) } else { None },
            "goal": talk::goal_text(g),
            "menu": {
                "open": data.settings.open, "gfx": settings::GFX_NAMES[g.gfx as usize % 4], "step": data.settings.step(g),
                "voices": !g.voices_off, "music_level": g.music_level, "fps": (data.settings.fps * 10.0).round() / 10.0,
                "shadow_cascades": ctx.renderer.settings.shadow_cascades,
            },
            "sound": {
                "track": data.sound.track().name(), "music_volume": ctx.audio.music_volume(),
                "voice": data.sound.voice_url_now(), "last_voice": data.sound.last_voice, "voices_started": data.sound.voices_started,
                "last_sfx": data.sound.last_sfx, "sfx_played": data.sound.sfx_played,
            },
            "save": {"plays": g.plays, "writes": data.save.writes, "time_of_day": (g.time_of_day * 100.0).round() / 100.0, "has_deck": g.has_deck, "gate_open": g.gate_open, "quest_part": g.quest_part},
        })
    }
}
