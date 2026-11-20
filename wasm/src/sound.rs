//! Sound (Kaigen `st_sound.c` + `cg_music`): the synthesized effects (no files), the background music
//! (streamed) and the recorded voice lines (assets/voices/<id>.ogg, the ids of the three.js
//! `scripts/voice-lines.mjs`).
//!
//! Hooks for later parts:
//! * `sound.sfx(ctx, Sfx::Fanfare, pitch, volume)` — blip, coin, fanfare, alert, quack, kick, splash,
//!   piano (C4, pitch = 2^(semitones/12)), pop; `Sfx::Rain` is a 2 s loop (`sfx_loop`).
//! * `sound.music(ctx, Track::Wild)` — switches the track (crossfade); `Track::Home` is the village.
//!   The card battle plays Intro (then Fight after 10.8 s, `Data::track_music`), Victory, Defeat.
//! * `sound.voice(ctx, "10a293f7")` — a voice line (stops the one before); `prefetch_voice` loads early.
//!
//! Voice lines play on the engine's `Bus::Voice`; the menu's "voices" switch sets that bus's volume
//! (lines still play, silently, so their timing stays the same).

#![allow(dead_code)] // hooks for later parts (see the module doc)

use engine::prelude::*;

const RATE: f32 = 44100.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Blip,
    Coin,
    Fanfare,
    Alert,
    Quack,
    Kick,
    Splash,
    Piano,
    Pop,
    Rain,
}

pub const ALL_SFX: [Sfx; 10] = [Sfx::Blip, Sfx::Coin, Sfx::Fanfare, Sfx::Alert, Sfx::Quack, Sfx::Kick, Sfx::Splash, Sfx::Piano, Sfx::Pop, Sfx::Rain];

impl Sfx {
    pub fn name(self) -> &'static str {
        match self {
            Sfx::Blip => "blip",
            Sfx::Coin => "coin",
            Sfx::Fanfare => "fanfare",
            Sfx::Alert => "alert",
            Sfx::Quack => "quack",
            Sfx::Kick => "kick",
            Sfx::Splash => "splash",
            Sfx::Piano => "piano",
            Sfx::Pop => "pop",
            Sfx::Rain => "rain",
        }
    }
    fn seconds(self) -> f32 {
        match self {
            Sfx::Fanfare => 1.1,
            Sfx::Piano | Sfx::Rain => 2.0,
            Sfx::Coin | Sfx::Splash => 0.6,
            _ => 0.4,
        }
    }
}

fn env(t: f32, attack: f32, decay: f32) -> f32 {
    if t < attack {
        t / attack
    } else {
        (-(t - attack) / decay).exp()
    }
}

/// A decaying tone at `freq` from `start` seconds, added into `out` (Kaigen `snd_tone`).
fn tone(out: &mut [f32], start: f32, dur: f32, freq: f32, vol: f32, square: bool) {
    let a = (start * RATE) as usize;
    let b = out.len().min(((start + dur) * RATE) as usize);
    for i in a..b {
        let t = (i - a) as f32 / RATE;
        let ph = t * freq;
        let s = if square { (if ph.fract() < 0.5 { 1.0 } else { -1.0 }) * 0.5 } else { (ph * std::f32::consts::TAU).sin() };
        out[i] += s * vol * env(t, 0.008, dur * 0.35);
    }
}

fn noise(s: &mut u32) -> u32 {
    *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
    *s >> 8
}

/// The samples of one effect, as the Kaigen `snd_render` (which follows the three.js sfx.ts).
pub fn render(id: Sfx) -> Vec<f32> {
    let n = (id.seconds() * RATE) as usize;
    let mut out = vec![0.0f32; n];
    let mut seed = 77u32;
    let tau = std::f32::consts::TAU;
    match id {
        Sfx::Blip => tone(&mut out, 0.0, 0.09, 880.0, 0.25, true),
        Sfx::Coin => {
            tone(&mut out, 0.0, 0.12, 987.8, 0.3, true);
            tone(&mut out, 0.08, 0.45, 1318.5, 0.3, true);
        }
        Sfx::Fanfare => {
            let notes = [523.3, 659.3, 784.0, 1046.5];
            for (i, f) in notes.iter().enumerate() {
                tone(&mut out, i as f32 * 0.11, if i == 3 { 0.7 } else { 0.2 }, *f, 0.22, true);
            }
        }
        Sfx::Alert => {
            tone(&mut out, 0.0, 0.12, 1200.0, 0.25, true);
            tone(&mut out, 0.13, 0.2, 1600.0, 0.25, true);
        }
        Sfx::Quack => {
            for i in 0..n.min((0.22 * RATE) as usize) {
                let t = i as f32 / RATE;
                let f = 620.0 - t * 1300.0;
                let ph = t * f;
                let mut s = (if ph.fract() < 0.3 { 1.0 } else { -0.45 }) * 0.35;
                s += ((noise(&mut seed) & 0xffff) as f32 / 65535.0 - 0.5) * 0.08;
                out[i] += s * env(t, 0.01, 0.08);
            }
        }
        Sfx::Kick => {
            for i in 0..n.min((0.2 * RATE) as usize) {
                let t = i as f32 / RATE;
                out[i] += (t * (140.0 - t * 300.0) * tau).sin() * 0.6 * env(t, 0.002, 0.05);
            }
        }
        Sfx::Splash => {
            for i in 0..n.min((0.5 * RATE) as usize) {
                let t = i as f32 / RATE;
                let w = (noise(&mut seed) & 0xffff) as f32 / 65535.0 - 0.5;
                out[i] += w * 0.5 * env(t, 0.01, 0.12);
            }
        }
        Sfx::Piano => {
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                let ph = t * 261.63 * tau;
                let s = ph.sin() + 0.45 * (ph * 2.0).sin() + 0.2 * (ph * 3.0).sin() + 0.08 * (ph * 4.0).sin();
                *o += s * 0.22 * env(t, 0.004, 0.55);
            }
        }
        Sfx::Pop => tone(&mut out, 0.0, 0.07, 520.0, 0.3, false),
        Sfx::Rain => {
            let mut lp = 0.0f32;
            for i in 0..n {
                let wn = (noise(&mut seed) & 0xffff) as f32 / 65535.0 - 0.5;
                lp += (wn - lp) * 0.18;
                let crackle = if noise(&mut seed) & 0x3ff == 0 { 0.6 } else { 0.0 };
                let edge = ((i.min(n - i)) as f32 / 400.0).min(1.0);
                out[i] += (lp * 0.9 + crackle * wn) * edge;
            }
        }
    }
    for s in &mut out {
        *s = s.clamp(-1.0, 1.0);
    }
    out
}

/// The music tracks (Kaigen `MusicTrack`), assets/music/*.ogg (opus 96k).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    None,
    Home,
    Intro,
    Fight,
    Victory,
    Defeat,
    Wild,
}

impl Track {
    pub fn url(self) -> Option<&'static str> {
        match self {
            Track::None => None,
            Track::Home => Some("assets/music/home.ogg"),
            Track::Intro => Some("assets/music/battle-intro.ogg"),
            Track::Fight => Some("assets/music/fight.ogg"),
            Track::Victory => Some("assets/music/victory.ogg"),
            Track::Defeat => Some("assets/music/defeat.ogg"),
            Track::Wild => Some("assets/music/wild.ogg"),
        }
    }
    fn looped(self) -> bool {
        matches!(self, Track::Home | Track::Fight | Track::Wild)
    }
    pub fn name(self) -> &'static str {
        match self {
            Track::None => "none",
            Track::Home => "home",
            Track::Intro => "intro",
            Track::Fight => "fight",
            Track::Victory => "victory",
            Track::Defeat => "defeat",
            Track::Wild => "wild",
        }
    }
}

/// The music's level per menu step (off, quiet, normal, loud). Normal is the Kaigen / three.js level
/// (0.15): the voices (0.95) stay clear above it.
pub const MUSIC_LEVELS: [f32; 4] = [0.0, 0.08, 0.15, 0.3];
pub const MUSIC_LEVEL_NAMES: [&str; 4] = ["Off", "Quiet", "Normal", "Loud"];

#[derive(Default)]
pub struct Sound {
    ready: bool,
    track: Option<Track>,
    music_level: usize,
    pub voices_on: bool,
    /// The voice line playing or waiting for its file (sound, voice, its url, seconds waited for the file).
    voice: Option<(SoundId, Option<VoiceId>, String, f32)>,
    /// Voice lines started (for tests).
    pub voices_started: u32,
    pub last_voice: Option<String>,
    /// Effects played (for tests): name of the last one.
    pub last_sfx: Option<&'static str>,
    pub sfx_played: u32,
    /// seconds the current track has played (the battle intro hands over to the fight music)
    pub track_t: f32,
}

impl Sound {
    /// Once: renders the effects into the engine's sound bank.
    pub fn init(&mut self, ctx: &mut Ctx) {
        if self.ready {
            return;
        }
        self.ready = true;
        for s in ALL_SFX {
            ctx.audio.add_samples(s.name(), &render(s), RATE);
        }
        self.music_level = 2;
        self.voices_on = true;
    }

    pub fn sfx(&mut self, ctx: &mut Ctx, id: Sfx, pitch: f32, volume: f32) {
        ctx.audio.play_named(id.name(), Play::new().pitch(pitch).volume(volume));
        self.last_sfx = Some(id.name());
        self.sfx_played += 1;
    }

    /// A looping effect (rain); stop it with `ctx.audio.stop`.
    pub fn sfx_loop(&mut self, ctx: &mut Ctx, id: Sfx, volume: f32) -> Option<VoiceId> {
        ctx.audio.play_named(id.name(), Play::new().looped().volume(volume))
    }

    /// Switches the music (no-op if it already plays).
    pub fn music(&mut self, ctx: &mut Ctx, t: Track) {
        if self.track == Some(t) {
            return;
        }
        self.track = Some(t);
        self.track_t = 0.0;
        match t.url() {
            Some(url) => {
                ctx.audio.play_music(url, MusicOpts { fade: 1.5, volume: 1.0, looped: t.looped() });
            }
            None => ctx.audio.stop_music(0.4),
        }
    }

    pub fn track(&self) -> Track {
        self.track.unwrap_or(Track::None)
    }

    /// The menu's music level (0 off .. 3 loud).
    pub fn set_music_level(&mut self, ctx: &mut Ctx, level: u8) {
        self.music_level = (level as usize).min(3);
        ctx.audio.set_music_volume(MUSIC_LEVELS[self.music_level]);
    }

    /// The menu's Voices switch: the voice bus's volume (a line playing goes quiet at once).
    pub fn set_voices(&mut self, ctx: &mut Ctx, on: bool) {
        self.voices_on = on;
        ctx.audio.set_bus_volume(Bus::Voice, if on { 1.0 } else { 0.0 });
    }

    fn voice_url(id: &str) -> String {
        format!("assets/voices/{id}.ogg")
    }

    /// Starts loading a voice line so it plays at once later.
    pub fn prefetch_voice(&mut self, ctx: &mut Ctx, id: &str) {
        ctx.load_sound(&Self::voice_url(id));
    }

    /// Plays voice line `id` (stops the one before). A line that is still loading starts when it arrives.
    pub fn voice(&mut self, ctx: &mut Ctx, id: Option<&str>) {
        self.stop_voice(ctx);
        let Some(id) = id else { return };
        let url = Self::voice_url(id);
        let snd = ctx.load_sound(&url);
        self.voice = Some((snd, None, url, 0.0));
        self.start_pending(ctx);
    }

    pub fn stop_voice(&mut self, ctx: &mut Ctx) {
        if let Some((_, Some(v), _, _)) = self.voice.take() {
            ctx.audio.stop(v);
        }
    }

    fn start_pending(&mut self, ctx: &mut Ctx) {
        let Some((snd, None, url, _)) = self.voice.clone() else { return };
        match ctx.audio.sound_state(snd) {
            Some(SoundState::Loading) => {}
            Some(SoundState::Ready) => {
                let v = ctx.audio.play(snd, Play::new().volume(0.95).bus(Bus::Voice));
                self.voice = Some((snd, v, url.clone(), 0.0));
                self.voices_started += 1;
                self.last_voice = Some(url);
            }
            _ => {
                log_warn!("voice {url} failed to load");
                self.voice = None;
            }
        }
    }

    /// Every frame: starts a voice line whose file has just arrived.
    pub fn update(&mut self, ctx: &mut Ctx, dt: f32) {
        self.track_t += dt;
        if let Some(v) = &mut self.voice {
            if v.1.is_none() {
                v.3 += dt;
            }
            if v.1.is_none() && v.3 > 10.0 {
                self.voice = None; // never arrived: forget it (a late start would not match the line any more)
                return;
            }
        }
        if matches!(self.voice, Some((_, None, _, _))) {
            self.start_pending(ctx);
        }
    }

    /// A voice line is loading or playing (the TV waits for it before the next line).
    pub fn voice_active(&self, ctx: &Ctx) -> bool {
        match &self.voice {
            Some((_, None, _, _)) => true,
            Some((_, Some(v), _, _)) => ctx.audio.is_playing(*v),
            None => false,
        }
    }

    /// The voice line being played or waiting (url).
    pub fn voice_url_now(&self) -> Option<&str> {
        self.voice.as_ref().map(|v| v.2.as_str())
    }

    pub fn music_level(&self) -> usize {
        self.music_level
    }
}
