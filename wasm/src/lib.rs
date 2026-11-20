//! Brickhollow on the wasmOnly engine: the LEGO village of the three.js version (`src/lib/three/town.ts`
//! in the repo root), ported in vertical slices. Slice 1: the walkable village with your minifig, the
//! fixed top-down camera of the original and the villagers standing in it. Start: CLAUDE.md.

#![recursion_limit = "256"]

mod cardface;
mod cards;
mod deck;
mod dialog;
mod fade;
mod figure;
mod life;
mod play;
mod save;
mod screens;
mod settings;
mod sound;
mod talk;
mod town;

use engine::prelude::*;
use play::PlayScreen;
use save::Save;
use settings::Settings;
use sound::{Sound, Track};
use town::{TownAssets, STUD};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // no MSAA, FXAA in the composite: at dpr 2 the village looks the same as with 4x MSAA and renders
    // ~30 % faster on desktop (much more on phones). `?msaa=4` in the URL brings MSAA back
    engine::run_with(
        ScreenStack::new(Data::default(), Box::new(Title))
            .register("title", || Box::new(Title))
            .register("play", || Box::new(PlayScreen::default())),
        engine::renderer::RenderConfig { msaa: 1 },
    );
}

/// Data shared by all screens (GPU resources are created once and kept).
#[derive(Default)]
pub(crate) struct Data {
    pub save: Save,
    pub sound: Sound,
    pub settings: Settings,
    loaded: bool,
    pub town: Option<TownAssets>,
    /// the card faces (canvas -> textures / PNG urls), kept for every screen
    pub faces: cardface::CardFaces,
    /// the deck screen, the foil packs, the upgrade moment
    pub cards: deck::CardUi,
}

impl Data {
    pub fn ensure(&mut self, ctx: &mut Ctx) {
        if !self.loaded {
            self.loaded = true;
            self.save.load(ctx);
            self.sound.init(ctx);
            ctx.ui.set("style", play::STYLE);
            ctx.ui.set("style_screens", screens::STYLE);
            ctx.ui.set("style_cards", deck::STYLE);
            self.cards = deck::CardUi::new();
        }
        // the village theme (the browser starts it on the first key or tap)
        self.sound.music(ctx, Track::Home);
    }
}

/// The sky and light of the three.js town: a pale blue background with fog, a warm sun, a soft hemisphere.
pub(crate) fn setup_look(ctx: &mut Ctx) {
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
    // the sun moves 1 degree before the shadows follow (default 0.5): every step re-renders all
    // cascades, at a 6-minute day that was once a second
    s.shadow_step_deg = 1.0;
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
pub(crate) const START_HOUR: f32 = 6.0 + 0.15 * 24.0;

/// The original `updateDayNight` sky: night -> dusk -> day colours for the background and the fog.
pub(crate) fn day_colours(ctx: &mut Ctx) {
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

pub(crate) struct Title;

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

/// Phones (a touch was seen): the village has ~300 trees and bushes and four shadow cascades, which a phone GPU
/// draws at ~19 fps. Two cascades over a shorter range, simpler tree LODs sooner and no screen-space extras.
pub(crate) fn phone_quality(ctx: &mut Ctx) {
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

