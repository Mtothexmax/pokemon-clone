//! Brickhollow on the wasmOnly engine: the LEGO village of the three.js version (`src/lib/three/town.ts`
//! in the repo root, and the Kaigen port). `Data` is shared by every screen (save, sound, settings, the town's
//! assets, the card UI); the screens are `Title`, `PlayScreen` (play.rs) and `arena::BattleScreen`.
//! Module map and hooks: PORT_STATUS.md.

#![recursion_limit = "512"]

mod arena;
mod battle;
mod cardface;
mod cards;
mod deck;
mod cave;
mod debugpanel;
mod dialog;
mod dive;
mod fade;
mod figure;
mod fountain;
mod grass;
mod island;
mod islandview;
mod life;
mod places;
mod play;
mod save;
mod savefile;
mod screens;
mod settings;
mod sound;
mod swipe;
mod table;
mod talk;
mod town;
mod vox;
mod worldmap;

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
    // ~30 % faster on desktop (much more on phones). `?msaa=4` in the URL brings MSAA back.
    // Direct output: the engine also builds pipelines that tonemap in the fragment shader and draw straight into the
    // canvas, used in frames whose composite would only tonemap (the phone's card battle: no bloom, FXAA, depth
    // darkening, ...): no HDR target, no composite pass. `?direct=0` turns it off
    engine::run_with(
        ScreenStack::new(Data::default(), Box::new(Title))
            .register("title", || Box::new(Title))
            .register("play", || Box::new(PlayScreen::default()))
            .register("battle", || Box::new(arena::BattleScreen::default())),
        engine::renderer::RenderConfig { msaa: 1, direct_output: true },
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
    /// the island's height law, baked once (island.rs)
    pub island: Option<std::rc::Rc<island::Island>>,
    /// the card faces (canvas -> textures / PNG urls), kept for every screen
    pub faces: cardface::CardFaces,
    /// the deck screen, the foil packs, the upgrade moment
    pub cards: deck::CardUi,
    /// the card battle's 3D pieces (table.rs)
    pub art: table::TableArt,
    /// the way into a battle (the bricks over the screen), drawn by whichever screen is on top
    pub trans: Option<arena::Transition>,
    /// a battle just ended: the play screen pays out (studs, EXP, cards) and opens the after-dialogue
    pub battle_out: Option<arena::BattleOut>,
    /// the card game's generator (battle seeds, reward cards; Kaigen `cg->seed`)
    pub cg_seed: u32,
    /// the asset bar (`loadbar`): the most models pending at once since it showed, the percent last shown
    load_max: usize,
    load_pct: Option<u32>,
    /// materials the plastic pass (`plastic`) has seen
    plastic_seen: usize,
}

impl Data {
    pub fn ensure(&mut self, ctx: &mut Ctx) {
        self.ensure_loaded(ctx);
        // the village theme (the browser starts it on the first key or tap)
        self.sound.music(ctx, Track::Home);
    }

    /// The intro fanfare hands over to the fight music after 10.8 s (as the three.js one), or back home.
    pub fn track_music(&mut self, ctx: &mut Ctx, in_battle: bool) {
        if self.sound.track() == Track::Intro && self.sound.track_t > 10.8 {
            self.sound.music(ctx, if in_battle { Track::Fight } else { Track::Home });
        }
    }

    /// A random number of the card game (Kaigen `cg_rand`).
    pub fn cg_rand(&mut self, ctx: &mut Ctx) -> u32 {
        if self.cg_seed == 0 {
            // a fresh seed per session: `ctx.rng` starts from the same fixed seed on every page load, so the first
            // battle always dealt the same deck order and locations (Sky Ledge, Brick Gym, ...). Mixed with the
            // engine's generator; tests that need one battle pass `battle {seed}`
            let entropy = (js_sys::Math::random() * 4294967296.0) as u32 ^ (js_sys::Date::now() as u64 as u32).rotate_left(16);
            self.cg_seed = (ctx.rng.next_u64() as u32 ^ entropy) | 1;
        }
        lcg(&mut self.cg_seed)
    }

    /// Every frame (title, play, battle): while models load, a brick bar at the top shows how far it is (the page's
    /// own bar, web/index.html, covers the game download before that).
    pub fn loadbar(&mut self, ctx: &mut Ctx) {
        self.plastic(ctx);
        let pending = ctx.assets_pending();
        let pct = if pending > 0 {
            self.load_max = self.load_max.max(pending);
            Some((100 * (self.load_max - pending) / self.load_max.max(1)) as u32)
        } else {
            self.load_max = 0;
            None
        };
        if pct != self.load_pct {
            self.load_pct = pct;
            ctx.ui.set("loadbar", &pct.map(|p| format!("<div class='lbar'><div class='track'><div class='fill' style='width:{p}%'></div></div><div class='label'>loading bricks … {p} %</div></div>")).unwrap_or_default());
        }
    }

    /// LEGO is shiny ABS plastic: every new opaque lit material (the code's bricks and the glTF minifigs, trees,
    /// props, beasts; the glTFs come with 0.72) gets the plastic look: a smooth base
    /// (`PLASTIC_ROUGH`) under a clear coat (the sharp white highlight, the sky in it with `ambient_specular`) and a
    /// little subsurface light (`PLASTIC_SSS`: a softer terminator, the shade side lighter, light through thin edges).
    /// Not: cloth / wood / paper / rock (`NOT_PLASTIC` in the name, or rougher than 0.85), water, terrain, unlit,
    /// glowing, transparent or metal materials. Runs whenever materials were added (models arrive over time).
    fn plastic(&mut self, ctx: &mut Ctx) {
        let n = ctx.renderer.material_count();
        for i in self.plastic_seen..n {
            if let Some(m) = ctx.renderer.material_mut(engine::renderer::MaterialId(i as u32)) {
                let name = m.name.to_lowercase();
                let skip = m.unlit || m.ui || m.water || m.terrain || m.metallic > 0.5 || m.roughness > 0.85 || m.emissive.max_element() > 0.0
                    || m.alpha_mode == engine::renderer::AlphaMode::Blend || NOT_PLASTIC.iter().any(|w| name.contains(w));
                if !skip {
                    m.roughness = PLASTIC_ROUGH;
                    m.clearcoat = 1.0;
                    m.clearcoat_roughness = PLASTIC_COAT_ROUGH;
                    m.subsurface = PLASTIC_SSS;
                }
            }
        }
        self.plastic_seen = n;
    }

    pub fn ensure_loaded(&mut self, ctx: &mut Ctx) {
        if !self.loaded {
            self.loaded = true;
            self.save.load(ctx);
            self.sound.init(ctx);
            ctx.ui.set("style", play::STYLE);
            ctx.ui.set("style_screens", screens::STYLE);
            ctx.ui.set("style_cards", deck::STYLE);
            ctx.ui.set("style_battle", arena::STYLE);
            ctx.ui.set("style_vox", vox::STYLE);
            ctx.ui.set("style_dive", dive::STYLE);
            ctx.ui.set("style_debug", debugpanel::STYLE);
            ctx.ui.set("style_map", worldmap::STYLE);
            ctx.ui.set("style_loadbar", LOADBAR_STYLE);
            self.cards = deck::CardUi::new();
            // a phone (a coarse primary pointer, or touch points and a mobile browser) gets the phone profile from the
            // first frame, not after the first touch
            self.settings.phone = web_sys::window().is_some_and(|w| {
                let coarse = w.match_media("(pointer: coarse)").ok().flatten().is_some_and(|m| m.matches());
                let nav = w.navigator();
                let ua = nav.user_agent().unwrap_or_default();
                let mobile = nav.max_touch_points() > 0 && ["Android", "iPhone", "iPad", "Mobile"].iter().any(|k| ua.contains(k));
                coarse || mobile
            });
            self.settings.touch = self.settings.phone;
        }
    }
}

/// Text into HTML (the HUD slots take markup).
pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The random generator of the Kaigen port (an LCG, the Numerical Recipes constants): steps `seed`, returns 24 bits.
/// Every deterministic stream in the game uses it (town layout, life, battles, Voxelmon, synthesized sounds).
pub(crate) fn lcg(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    *seed >> 8
}

/// [`lcg`] as 0..1.
pub(crate) fn lcg_f(seed: &mut u32) -> f32 {
    lcg(seed) as f32 / 16777216.0
}

/// LEGO plastic (`Data::plastic`): the base's roughness, the clear coat's, the subsurface amount; and the materials
/// that are not plastic.
pub(crate) const PLASTIC_ROUGH: f32 = 0.3;
const PLASTIC_COAT_ROUGH: f32 = 0.06;
const PLASTIC_SSS: f32 = 0.3;
pub(crate) const AMBIENT_SPECULAR: f32 = 0.8;
/// (`sign_`: the printed boards, a clear coat's sky reflection washed their text out at some angles)
const NOT_PLASTIC: [&str; 8] = ["wood", "felt", "card", "cave", "paper", "rock", "sand", "sign_"];

/// The asset bar's look (as the page's loading bar in web/index.html: red bricks with studs filling a dark track).
const LOADBAR_STYLE: &str = "<style>#ui [data-slot=loadbar]{position:fixed;left:50%;top:calc(14px + env(safe-area-inset-top));transform:translateX(-50%);z-index:4}#ui .lbar{width:min(320px,calc(100vw - 48px));text-align:center;padding:8px 12px;border-radius:10px;background:rgb(10 14 22/.6)}#ui .lbar .track{position:relative;height:14px;border-radius:5px;background:#0e1420;box-shadow:inset 0 2px 4px rgb(0 0 0/.6);overflow:hidden}#ui .lbar .fill{position:absolute;inset:0 auto 0 0;border-radius:5px;transition:width .2s linear;background:radial-gradient(circle at 50% 38%,rgb(255 255 255/.45) 0 2px,transparent 3px) 0 0/14px 14px,linear-gradient(#e3343a,#b5161c)}#ui .lbar .label{margin-top:4px;font-size:12px;opacity:.8;font-variant-numeric:tabular-nums}</style>";

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
    settings::native_resolution(s);
    // the sun moves 1 degree before the shadows follow (default 0.5): every step re-renders all
    // cascades, at a 6-minute day that was once a second
    s.shadow_step_deg = 1.0;
    // LEGO plastic (with `PLASTIC_ROUGH` bricks): the sky's sheen and a rim on the sides away from the sun, not only
    // the sun's highlight
    s.ambient_specular = AMBIENT_SPECULAR;
    // a house or tree faded out between the camera and you (play.rs fader) shows only its front surface, like a
    // picture with lowered alpha (blending every face would show the bricks inside the walls and the back walls)
    s.solid_fade = true;
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
pub(crate) const DAY_MINUTES: f32 = 6.0;
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

/// The start screen's picture, filling the screen: the wide one on a landscape screen, the tall one on a portrait
/// phone (assets/title, from the generated key art).
const TITLE_BG: &str = "<style>.bh-titlebg{position:fixed;inset:0;z-index:-1;background:#7fc4f0 url(assets/title/title_wide.webp) center/cover no-repeat;animation:bh-tin .8s ease-out}@media (orientation:portrait){.bh-titlebg{background-image:url(assets/title/title_tall.webp)}}@keyframes bh-tin{from{opacity:0;transform:scale(1.04)}to{opacity:1;transform:none}}</style><div class='bh-titlebg'></div>";

/// The title and the Play button over the picture's sky.
const TITLE_MENU: &str = "<style>#ui [data-slot=\"menu\"]{place-items:start center}.bh-start{margin-top:max(4vh,18px);text-align:center;display:flex;flex-direction:column;align-items:center;gap:10px;animation:bh-sdrop .7s .2s both cubic-bezier(.2,1.4,.4,1)}.bh-start h1{margin:0;font:900 clamp(44px,9vw,104px)/1 system-ui,sans-serif;letter-spacing:1px;color:#ffd23f;text-shadow:0 4px 0 #b5161c,0 8px 0 #7a0e12,0 12px 18px rgba(0,0,0,.35);-webkit-text-stroke:2px #7a0e12}.bh-start .sub{font:700 clamp(15px,2.4vw,22px)/1.2 system-ui,sans-serif;color:#fff;text-shadow:0 2px 6px rgba(0,0,0,.55)}#ui .bh-start button{pointer-events:auto;margin-top:8px;font:800 clamp(18px,2.6vw,26px)/1 system-ui,sans-serif;padding:14px 38px;border:0;border-radius:14px;background:#ffd23f;color:#3a1d00;box-shadow:0 5px 0 #b07a00,0 9px 18px rgba(0,0,0,.3);cursor:pointer}#ui .bh-start button:hover{background:#ffe066}#ui .bh-start button:active{transform:translateY(3px);box-shadow:0 2px 0 #b07a00}@keyframes bh-sdrop{from{opacity:0;transform:translateY(-30px)}to{opacity:1;transform:none}}</style><div class='bh-start'><h1>Brickhollow</h1><div class='sub'>A brick island adventure</div><button data-action='play'>Play</button></div>";

impl Screen<Data> for Title {
    fn name(&self) -> &'static str {
        "title"
    }
    fn enter(&mut self, ctx: &mut Ctx, data: &mut Data) {
        data.ensure_loaded(ctx);
        // the start screen's own theme (the browser starts it on the first key or tap); the play screen hands over
        data.sound.music(ctx, Track::Title);
        setup_look(ctx);
        ctx.world.spawn((Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y), Camera::default()));
        ctx.ui.set("title_bg", TITLE_BG);
        ctx.ui.set("menu", TITLE_MENU);
    }
    fn exit(&mut self, ctx: &mut Ctx, _data: &mut Data) {
        ctx.ui.set("menu", "");
        ctx.ui.set("title_bg", "");
    }
    fn update(&mut self, ctx: &mut Ctx, data: &mut Data, _dt: f32) -> Trans<Data> {
        data.loadbar(ctx);
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
    // (the engine multiplies the LOD switch distances by this: under 1 the simpler levels come sooner)
    s.lod_distance_scale = 0.6;
    s.depth_darkening = 0.0;
    s.bloom = false;
    s.water_ssr = false;
    s.lens_flare = 0.0;
    // (the analytic studs: the 3x3 search, about as cheap as 5x5 but a little less work per pixel)
    s.stud_relief = 1;
    // a tight budget keeps the resolution at half, which holds up in the heavy parts of the village; it starts there
    // (at full resolution a phone measured 235-307 ms per frame, and every adaptation step down is a hitch).
    // No frame cap: the engine's frame pacing picks the fastest even cadence the GPU holds
    s.frame_budget_ms = 20.0;
    s.min_render_scale = 0.5;
    s.render_scale = 0.5;
    s.auto_resolution = true;
}

