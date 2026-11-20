//! The menu behind the top-right ☰ button (Kaigen `st_settings.c`, the three.js +page.svelte menu):
//! progress, Voices on/off, the music level and Graphics Auto / High / Medium / Low, plus the way back
//! to the title screen. Esc or the ☰ button open and close it (desktop and touch), a click on the shade
//! closes it. The choices are kept in the save.
//!
//! Auto measures the frame rate in 2 s windows and steps down while the game runs below 38 fps (not
//! right after a change or a stall); the step it reached is saved. The engine fixes MSAA at start-up:
//! choosing High stores MSAA 4 for the next start (`engine::set_msaa_pref`, no reload in the middle of
//! the game), any other choice goes back to the game's default (no MSAA, FXAA); the steps themselves
//! are about shadows, effects, LOD distances and the adaptive resolution's budget.

use crate::save::{GameSave, Save};
use crate::sound::{Sound, MUSIC_LEVEL_NAMES};
use engine::prelude::*;

pub const GFX_NAMES: [&str; 4] = ["Auto", "High", "Medium", "Low"];
const STEP_NAMES: [&str; 3] = ["High", "Medium", "Low"];

/// What the menu asks the play screen to do.
#[derive(Debug, PartialEq, Eq)]
pub enum MenuOut {
    None,
    Title,
}

#[derive(Default)]
pub struct Settings {
    pub open: bool,
    /// The graphics step applied now (0 High, 1 Medium, 2 Low), None before the first.
    pub applied: Option<usize>,
    /// Phones (a touch was seen): Auto starts at Low.
    pub touch: bool,
    settle: f32,
    win_t: f32,
    win_frames: u32,
    /// The last window's frame rate (shown in the menu).
    pub fps: f32,
    /// Audio switches applied (voices, music level), None before the first.
    audio_applied: Option<(bool, u8)>,
}

/// One graphics step on the renderer. 0: the desktop look of `setup_look`; 2: the phone profile.
pub fn apply_step(ctx: &mut Ctx, step: usize) {
    let s = &mut ctx.renderer.settings;
    match step {
        0 => {
            s.shadow_cascades = 4;
            s.shadow_distance = 60.0;
            s.lod_bias = 1.0;
            s.depth_darkening = 1.0;
            s.bloom = true;
            s.water_ssr = true;
            s.lens_flare = 1.0;
            s.frame_budget_ms = 14.0;
            s.min_render_scale = 0.5;
        }
        1 => {
            s.shadow_cascades = 3;
            s.shadow_distance = 45.0;
            s.lod_bias = 2.0;
            s.depth_darkening = 0.6;
            s.bloom = true;
            s.water_ssr = false;
            s.lens_flare = 0.0;
            s.frame_budget_ms = 20.0;
            s.min_render_scale = 0.5;
        }
        _ => crate::phone_quality(ctx),
    }
}

impl Settings {
    /// The step in use: a fixed choice, or where Auto is.
    pub fn step(&self, g: &GameSave) -> usize {
        match g.gfx {
            1 => 0,
            2 => 1,
            3 => 2,
            _ => {
                let start = if self.touch { 2 } else { 0 };
                (g.auto_step.saturating_sub(1) as usize).max(start).min(2)
            }
        }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Every frame: the menu's actions (`menu`, `set_voices`, `set_music:N`, `set_gfx:N`, `menu_title`,
    /// `menu_close`), the audio switches, Auto's frame-rate watch and the graphics level.
    pub fn update(&mut self, ctx: &mut Ctx, save: &mut Save, sound: &mut Sound, actions: &[String], dt: f32) -> MenuOut {
        let mut out = MenuOut::None;
        let g = &mut save.data;
        let mut changed = false;
        for a in actions {
            match a.as_str() {
                "menu" => self.toggle(),
                "menu_close" => self.open = false,
                "menu_title" => out = MenuOut::Title,
                "set_voices" => {
                    g.voices_off = !g.voices_off;
                    changed = true;
                }
                _ => {
                    if let Some(n) = a.strip_prefix("set_music:").and_then(|n| n.parse::<u8>().ok()) {
                        g.music_level = n.min(3);
                        changed = true;
                    } else if let Some(n) = a.strip_prefix("set_gfx:").and_then(|n| n.parse::<u8>().ok()) {
                        g.gfx = n.min(3);
                        // High: sharper edges (MSAA 4) from the next start; the rest: the default (FXAA)
                        engine::set_msaa_pref(if g.gfx == 1 { Some(4) } else { None }, false);
                        if g.gfx == 0 {
                            g.auto_step = 1; // Auto starts from the top again and steps down if needed
                        }
                        self.settle = 0.0;
                        changed = true;
                    }
                }
            }
        }
        if g.auto_step == 0 {
            g.auto_step = 1;
            changed = true;
        }
        let audio = (!g.voices_off, g.music_level);
        if self.audio_applied != Some(audio) {
            self.audio_applied = Some(audio);
            sound.set_voices(ctx, audio.0);
            sound.set_music_level(ctx, audio.1);
        }

        // ---- Auto: 2 s windows; below 38 fps one step down (not right after a change, a load or a stall) ----
        self.settle += dt;
        let paused = ctx.time.paused || !ctx.audio.focused();
        if paused || dt > 0.25 || dt <= 0.0 || self.settle < 4.0 || ctx.assets_pending() > 0 {
            self.win_t = 0.0;
            self.win_frames = 0;
        } else {
            self.win_t += dt;
            self.win_frames += 1;
            if self.win_t >= 2.0 {
                self.fps = self.win_frames as f32 / self.win_t;
                let cur = self.step(g);
                if g.gfx == 0 && self.fps < 38.0 && cur < 2 {
                    g.auto_step = (cur + 2) as u8;
                    changed = true;
                    self.settle = 0.0;
                    log_info!("graphics: {:.1} fps, Auto steps down to {}", self.fps, STEP_NAMES[cur + 1]);
                }
                self.win_t = 0.0;
                self.win_frames = 0;
            }
        }
        if changed {
            save.mark();
        }
        let step = self.step(&save.data);
        if self.applied != Some(step) {
            self.applied = Some(step);
            apply_step(ctx, step);
        }
        out
    }

    /// The ☰ button (always) and the menu (when open).
    pub fn draw(&self, ctx: &mut Ctx, g: &GameSave, sound: &Sound) {
        ctx.ui.set(
            "menubtn",
            &format!("<button class='bh-menubtn{}' data-action='menu' aria-label='Menu'>{}</button>", if self.open { " on" } else { "" }, if self.open { "&times;" } else { "<i></i><i></i><i></i>" }),
        );
        if !self.open {
            ctx.ui.set("settings", "");
            return;
        }
        let seg = |names: &[&str], on: usize, action: &str| -> String {
            names
                .iter()
                .enumerate()
                .map(|(i, n)| format!("<button class='{}' data-action='{action}:{i}'>{n}</button>", if i == on { "on" } else { "" }))
                .collect::<String>()
        };
        let hint = if g.gfx == 0 {
            format!("Auto: now {}. It turns down if the game gets slow.", STEP_NAMES[self.step(g)])
        } else if g.gfx == 1 {
            "Smoother edges (MSAA) from the next start.".to_string()
        } else {
            "Lower settings run smoother on slow devices.".to_string()
        };
        let fps = if self.fps > 0.0 { format!("<div class='bh-fps'>{:.0} fps</div>", self.fps) } else { String::new() };
        let html = format!(
            "<div class='bh-shade' data-action='menu_close'></div><div class='bh-menu'>\
             <div class='bh-prog'><span class='gold'>&#9733; {}/15</span><span>&#9679; {} studs</span></div>\
             <button class='bh-row' data-action='set_voices'><b>Voices</b><span>{}</span></button>\
             <div class='bh-cap'>MUSIC</div><div class='bh-seg'>{}</div>\
             <div class='bh-cap'>GRAPHICS</div><div class='bh-seg'>{}</div><div class='bh-hint'>{}</div>{}\
             <button class='bh-row' data-action='menu_title'><b>Title screen</b><span>&rsaquo;</span></button></div>",
            g.golden_count(),
            g.studs,
            if g.voices_off { "off" } else { "on" },
            seg(&MUSIC_LEVEL_NAMES, sound.music_level(), "set_music"),
            seg(&GFX_NAMES, g.gfx as usize, "set_gfx"),
            hint,
            fps,
        );
        ctx.ui.set("settings", &html);
    }
}
