//! Brickhollow on the wasmOnly engine: the LEGO village of the three.js version (`src/lib/three/town.ts`
//! in the repo root), ported in vertical slices. Slice 1: the walkable village with your minifig, the
//! fixed top-down camera of the original and the villagers standing in it. Start: CLAUDE.md.

mod fade;
mod figure;
mod town;

use engine::prelude::*;
use figure::Figure;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use town::{TownAssets, TownStats, STUD};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    engine::run(
        ScreenStack::new(Data::default(), Box::new(Title))
            .register("title", || Box::new(Title))
            .register("play", || Box::new(PlayScreen::default())),
    );
}

/// Saved progress (localStorage slot "progress").
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Progress {
    plays: u32,
}

/// Data shared by all screens (GPU resources are created once and kept).
#[derive(Default)]
struct Data {
    progress: Progress,
    loaded: bool,
    town: Option<TownAssets>,
}

impl Data {
    fn ensure(&mut self, ctx: &mut Ctx) {
        if !self.loaded {
            self.loaded = true;
            ctx.save.set_namespace("brickhollow", 1);
            self.progress = ctx.save.read("progress").unwrap_or_default();
        }
    }
}

/// The sky and light of the three.js town: a pale blue background with fog, a warm sun, a soft hemisphere.
fn setup_look(ctx: &mut Ctx) {
    let sky = engine::renderer::srgb_hex(0xbfe6ff).truncate();
    let s = &mut ctx.renderer.settings;
    s.clear_color = sky;
    s.sky = false;
    s.fog_color = Some(sky);
    s.fog_start = 90.0 * STUD;
    s.fog_density = 0.055;
    s.fog_max = 1.0;
    s.fog_height_falloff = 0.0;
    s.sun_dir = Vec3::new(-0.45, -1.0, -0.55);
    s.sun_color = engine::renderer::srgb_hex(0xfff6e0).truncate();
    s.sun_intensity = 3.0;
    s.sky_color = engine::renderer::srgb_hex(0xe8f4ff).truncate() * 0.55;
    s.ground_color = engine::renderer::srgb_hex(0x5a7a4a).truncate() * 0.45;
    s.shadow_distance = 60.0;
    // desktop: the default 11 ms budget held the resolution at 0.5-0.65 (GPU ~9 ms); 14 ms still fits a
    // 60 Hz frame and renders sharper. Phones get their own budget (phone_quality)
    s.frame_budget_ms = 14.0;
    // the day/night cycle of the original: 6 minutes per day, starting in the morning. The engine
    // places the sun and moon and lights the scene; the sky and fog keep the original's flat colours
    // (day_colours, every frame)
    s.sky_mode = 0;
    s.day_cycle = true;
    s.day_cycle_exposure = true;
    s.time_of_day = START_HOUR;
    s.day_length_minutes = DAY_MINUTES;
}

/// The original's day: `dayLength` 360 s, `timeOfDay` 0.15 of a day after sunrise (6:00).
const DAY_MINUTES: f32 = 6.0;
const START_HOUR: f32 = 6.0 + 0.15 * 24.0;

/// The original `updateDayNight` sky: night -> dusk -> day colours for the background and the fog.
fn day_colours(ctx: &mut Ctx) {
    let hours = ctx.renderer.settings.time_of_day;
    let sun_h = ((hours - 6.0) / 24.0 * std::f32::consts::TAU).sin(); // 1 at noon, -1 at midnight
    let day = (sun_h * 1.6 + 0.35).clamp(0.0, 1.0);
    let dusk = 1.0 - (sun_h.abs() * 3.5).min(1.0);
    let c = |hex| engine::renderer::srgb_hex(hex).truncate();
    let sky = c(0x1c2d52).lerp(c(0xbfe6ff), day).lerp(c(0xffb98a), dusk * 0.55);
    let s = &mut ctx.renderer.settings;
    s.clear_color = sky;
    s.fog_color = Some(sky);
    s.sky_zenith = sky;
    s.sky_horizon = sky;
}

struct Title;

impl Screen<Data> for Title {
    fn name(&self) -> &'static str {
        "title"
    }
    fn enter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.ensure(ctx);
        setup_look(ctx);
        ctx.world.spawn((Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y), Camera::default()));
        ctx.ui.set(
            "menu",
            "<div class='panel menu'><h1>Brickhollow</h1><div>Snap Beasts &middot; a LEGO village</div><button data-action='play'>Play (Enter)</button></div>",
        );
    }
    fn exit(&mut self, ctx: &mut Ctx, _data: &mut Data) {
        ctx.ui.set("menu", "");
    }
    fn update(&mut self, ctx: &mut Ctx, _data: &mut Data, _dt: f32) -> Trans<Data> {
        let clicked = ctx.ui.take_actions().iter().any(|a| a == "play");
        if clicked || ctx.input.key_pressed("Enter") {
            return Trans::Switch(Box::new(PlayScreen::default()));
        }
        Trans::None
    }
}

/// Where you start (outside, in front of your house) — the bedroom start comes with the interiors slice.
const START: [f32; 2] = [-15.0, -2.5];
/// Camera offset of the original (studs, before zoom) and its default zoom.
const CAM_OFFSET: Vec3 = Vec3::new(0.0, 32.0, 17.0);
const ZOOM: f32 = 1.15;

#[derive(Default)]
struct PlayScreen {
    hero: Option<Entity>,
    fig: Option<Figure>,
    npcs: Vec<(String, Figure)>,
    camera: Option<Entity>,
    stats: TownStats,
    /// the figure moved this frame (horizontal speed, m/s)
    speed: f32,
    /// the street lamps' lights (warm, switched on at night)
    lamps: Vec<Entity>,
    /// buildings and trees that fade while they hide you
    fader: fade::Fader,
    /// the touch Jump button was tapped (consumed by the next fixed step)
    jump_tap: bool,
    /// the touch HUD (buttons, hints) is shown
    touch_ui: bool,
}

/// Help line and buttons for phones: the engine's joystick (left 40 % of the screen) walks,
/// pushing it all the way out runs, two fingers zoom.
const TOUCH_HELP: &str = "<div class='panel'><b>Brickhollow</b><br>left thumb: walk (push far = run) &middot; two fingers: zoom</div>";
const TOUCH_BUTTONS: &str = "<div class='touch-buttons'><button data-action='jump'>Jump</button> <button data-action='menu'>Menu</button></div>";
/// Stick deflection from which the hero runs.
const TOUCH_RUN: f32 = 0.92;

/// Phones (a touch was seen): the village has ~300 trees and bushes and four shadow cascades, which a phone GPU
/// draws at ~19 fps. Two cascades over a shorter range, simpler tree LODs sooner and no screen-space extras.
fn phone_quality(ctx: &mut Ctx) {
    let s = &mut ctx.renderer.settings;
    s.shadow_cascades = 2;
    s.shadow_distance = 30.0;
    s.lod_bias = 3.0;
    s.depth_darkening = 0.0;
    s.bloom = false;
    s.water_ssr = false;
    s.lens_flare = 0.0;
    // aim for ~30 fps instead of the desktop budget, resolution may drop to half
    s.frame_budget_ms = 30.0;
    s.min_render_scale = 0.5;
}

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
}

impl Screen<Data> for PlayScreen {
    fn name(&self) -> &'static str {
        "play"
    }

    fn enter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.ensure(ctx);
        setup_look(ctx);
        data.progress.plays += 1;
        let _ = ctx.save.write("progress", &data.progress);
        if data.town.is_none() {
            data.town = Some(TownAssets::new(ctx));
        }
        // the LEGO models are small and seen from 16 m: full detail (the engine's vertex-clustered
        // LOD levels misshaped the figures and popped while walking)
        // (tried auto_lod with lod_distance 3 on engine 890610b: no level switches at this camera
        // distance, still 860k triangles, so nothing gained)
        ctx.gltf_defaults.auto_lod = false;
        let (_solids, stats, occluders) = town::build(ctx, data.town.as_mut().unwrap());
        self.fader = fade::Fader::new(occluders);
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
            self.npcs.push((n.id.to_string(), f));
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
        ctx.ui.set("help", "<div class='panel'><b>Brickhollow</b><br><kbd>WASD</kbd> move &middot; <kbd>Shift</kbd> run &middot; <kbd>Space</kbd> jump &middot; <kbd>Wheel</kbd> zoom &middot; <kbd>Esc</kbd> menu</div>");
    }

    fn exit(&mut self, ctx: &mut Ctx, _data: &mut Data) {
        ctx.ui.set("help", "");
        ctx.ui.set("touch", "");
    }

    fn fixed_update(&mut self, ctx: &mut Ctx, _data: &mut Data) {
        // the camera never turns (the original's fixed view): W is always north (-z)
        if let Some(mut c) = self.camera.and_then(|c| ctx.world.get::<&mut ThirdPersonCamera>(c).ok()) {
            c.yaw = 0.0;
        }
        let axis = ctx.input.move_axis();
        let jump = ctx.input.key_down_or_pressed("Space") || std::mem::take(&mut self.jump_tap);
        let run = ctx.input.key("ShiftLeft") || ctx.input.key("ShiftRight") || ctx.input.touch.axis.length() > TOUCH_RUN;
        if let Some(mut cc) = self.hero.and_then(|h| ctx.world.get::<&mut CharacterController>(h).ok()) {
            cc.drive(axis, 0.0);
            cc.jump = jump;
            cc.run = run;
        }
        // the walk cycle follows the simulation (deterministic, also while stepping a paused game)
        let (v, run) = self
            .hero
            .and_then(|h| ctx.world.get::<&CharacterController>(h).ok().map(|c| (c.velocity, c.run)))
            .unwrap_or((Vec3::ZERO, false));
        self.speed = Vec2::new(v.x, v.z).length();
        let dt = ctx.time.fixed_dt;
        if let Some(mut f) = self.fig.take() {
            f.animate(ctx, dt, self.speed > 0.3, if run { 1.6 } else { 1.0 });
            self.fig = Some(f);
        }
    }

    fn update(&mut self, ctx: &mut Ctx, _data: &mut Data, dt: f32) -> Trans<Data> {
        // phones: once a finger touched the screen, swap the keyboard hints for touch hints and buttons
        if ctx.input.touch.touch_device && !self.touch_ui {
            self.touch_ui = true;
            ctx.ui.set("help", TOUCH_HELP);
            ctx.ui.set("touch", TOUCH_BUTTONS);
            phone_quality(ctx);
        }
        let mut menu = ctx.input.key_pressed("Escape");
        for a in ctx.ui.take_actions() {
            match a.as_str() {
                "jump" => self.jump_tap = true,
                "menu" => menu = true,
                _ => {}
            }
        }
        if menu {
            return Trans::Switch(Box::new(Title));
        }
        // whatever stands between the camera and you fades (the camera stays)
        if let (Some((p, _)), Some(cam)) = (self.hero_pos(ctx), self.camera.and_then(|c| ctx.world.get::<&Transform>(c).ok().map(|t| t.translation))) {
            let loaded = ctx.assets_pending() == 0;
            self.fader.update(ctx, cam, &[p + Vec3::Y * 0.5, p + Vec3::Y * 1.5], dt, loaded);
        }
        day_colours(ctx);
        let night = ctx.renderer.sky.night;
        for l in &self.lamps {
            if let Ok(mut p) = ctx.world.get::<&mut PointLight>(*l) {
                p.intensity = night * 6.0;
            }
        }
        Trans::None
    }

    fn debug_commands(&self) -> Vec<(&'static str, &'static str)> {
        vec![("teleport", "Move the hero (studs, like the three.js town). args: {x, z}"), ("time", "Set the time of day. args: {hours}")]
    }

    fn debug_command(&mut self, ctx: &mut Ctx, _data: &mut Data, name: &str, a: &Value) -> Option<Result<Value, String>> {
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
            _ => None,
        }
    }

    /// Everything an agent needs to assert progress (`call game_state` -> {screen, stack, state}).
    /// Positions are in studs (the three.js town's units).
    fn debug_state(&self, ctx: &Ctx, data: &Data) -> Value {
        let hero = self.hero.and_then(|h| {
            let (p, yaw) = self.hero_pos(ctx)?;
            let cc = ctx.world.get::<&CharacterController>(h).ok()?;
            let r = |v: f32| (v / STUD * 100.0).round() / 100.0;
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
        json!({
            "hero": hero,
            "camera": cam,
            "town": self.stats,
            "hiding": self.fader.hiding,
            "faded": self.fader.faded(),
            "fade_unresolved": self.fader.unresolved(),
            "npcs": self.npcs.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
            "assets_pending": ctx.assets_pending(),
            "time": (ctx.renderer.settings.time_of_day * 100.0).round() / 100.0,
            "night": (ctx.renderer.sky.night * 100.0).round() / 100.0,
            "lamps": self.lamps.iter().filter_map(|l| ctx.world.get::<&PointLight>(*l).ok().map(|p| (p.intensity * 100.0).round() / 100.0)).collect::<Vec<_>>(),
            "lods_left": ctx.world.query::<&Lod>().iter().count(),
            "progress": data.progress,
        })
    }
}
