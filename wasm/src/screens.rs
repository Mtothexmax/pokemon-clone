//! The screens you can look at (Kaigen `st_screens.c`, the three.js `TvShow`, `PcDesktop`, `LabTerminal`,
//! `PianoKeys`): the camera flies up to the TV or the computer and the picture is drawn right over its
//! screen as HTML / SVG (the board projected to the page; a phone held upright gets a panel that fits).
//! The TV has four channels (the BBN news, the weather and the nature show are voiced), your PC runs
//! "BrickOS" (messages, stats, beasts, stickers), Prof. Quill's lab computer a read-only research terminal
//! that types by itself. The piano is a keyboard over the room. E or Esc closes; every button is a
//! `data-action` (phones).
//!
//! Slots: `scr` (the picture and its buttons; rebuilt only when the channel or the app changes, so SVG
//! animations and clicks are not disturbed), `scr_live` (subtitles, the terminal's lines; no pointer)
//! and `scr_pos` (a style slot that places both over the screen every frame).
//!
//! Hooks: the card names (Beasts app chips, the terminal's specimens) come with the cards part
//! (`SPECIMENS` stands in); butterflies (Stats) with the life part.

use crate::save::GameSave;
use crate::sound::{Sfx, Sound};
use crate::talk::Board;
use crate::town::{self, STUD};
use engine::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ScreenKind {
    Tv,
    Piano,
    Pc,
    Lab,
}

pub const TV_CHANNELS: [&str; 4] = ["BBN News", "Weather", "Cartoons", "Wild Bricks"];
const ANCHOR: &str = "Rex Studwell";
const REPORTER: &str = "Penny Plate";
const NARRATOR: &str = "Narrator";

/// A TV line: speaker, text, voice id. `weather`: 0 day, 1 night, 2 rain.
fn tv_lines(ch: usize, weather: u8) -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    match ch {
        0 => vec![
            (ANCHOR, "Hello, Brickhollow! This is BBN, the Brick Beast News. I'm Rex Studwell.", Some("0c57cbfe")),
            (REPORTER, "And I'm Penny Plate. Our top story: Glimmerlings were spotted folding light into tiny bricks!", Some("b92a25ec")),
            (ANCHOR, "Fascinating. Professor Quill says it is perfectly safe, as long as nobody sneezes.", Some("ecff39c8")),
            (REPORTER, "In sports: young Dex claims his deck is unbeatable. Local trainers would like a word.", Some("d7de0462")),
            (ANCHOR, "The ducks at the pond report record bread deliveries this week. Experts say: more, please.", Some("180b23f7")),
            (REPORTER, "And golden studs are still hidden all over the village. If you find one, keep it shiny!", Some("626a02a6")),
            (ANCHOR, "That's all for now. Stay snappy, Brickhollow!", Some("1206ef79")),
        ],
        1 => match weather {
            2 => vec![
                (REPORTER, "Rain, rain, and a little more rain! Grab your umbrella, Brickhollow.", Some("51d69dfd")),
                (REPORTER, "The good news: the puddles are perfect for splashing. Pip approves.", Some("f7f07197")),
            ],
            1 => vec![
                (REPORTER, "Good evening, Brickhollow. Tonight we expect a clear and starry sky.", Some("677efb60")),
                (REPORTER, "The fireflies are out by the pond. Catch them gently, and let them go again!", Some("7168691e")),
            ],
            _ => vec![
                (REPORTER, "Here's your Brickhollow weather! Sunny skies all day, perfect for a battle on Route 1.", Some("b511c369")),
                (REPORTER, "A little shower may still pop by later. Puddles ahead, so splash responsibly!", Some("691c329e")),
            ],
        },
        2 => vec![
            ("Captain Click", "Captain Click to the rescue! Who left this 1x1 plate on the floor?!", None),
            ("Sidekick Stud", "Not me, Captain! I only step on 2x4s.", None),
            ("Captain Click", "Then justice will be... snapped together!", None),
            ("Announcer", "Tune in next week for: The Case of the Missing Instructions!", None),
        ],
        _ => vec![
            (NARRATOR, "Here, in the tall grass of Route 1, a young Brick Beast takes its very first steps.", Some("c51f2db1")),
            (NARRATOR, "Its studs are still soft. Within a week, they will click firmly into place.", Some("58850af2")),
            (NARRATOR, "Look closely. It has found a lost two by four brick. A rare treasure.", Some("3c3b5ded")),
            (NARRATOR, "Brick Beasts talk with a gentle clicking sound. Scientists still argue about what it means.", Some("8cd330e6")),
            (NARRATOR, "Most of them agree: it means snack time.", Some("4c545f17")),
            (NARRATOR, "Nature is patient. And so, it seems, is this little Beast.", Some("89d05b53")),
        ],
    }
}

/// Piano: 9 white keys (C4..D5) then 7 black; semitones above C4 and the keyboard keys.
pub const PIANO_KEYS: usize = 16;
const PIANO_WHITE: usize = 9;
const PIANO_SEMI: [i32; PIANO_KEYS] = [0, 2, 4, 5, 7, 9, 11, 12, 14, 1, 3, 6, 8, 10, 13, 15];
const PIANO_CODES: [&str; PIANO_KEYS] = ["KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "KeyW", "", "KeyT", "KeyY", "KeyU", "KeyO", "KeyP"];
const PIANO_LABELS: [&str; PIANO_WHITE] = ["A", "S", "D", "F", "G", "H", "J", "K", "L"];

/// BrickOS apps: label, icon, colour.
const APPS: [(&str, &str, &str); 5] = [("Home", "brick", "#1b2a34"), ("Messages", "mail", "#3d9bff"), ("Stats", "chart", "#27b05a"), ("Beasts", "beast", "#f2a705"), ("Stickers", "sticker", "#e0457b")];

/// Stand-ins for the card data of the cards part: specimens the lab terminal analyses.
const SPECIMENS: [&str; 12] = ["brickling", "glimmerling", "sparkmole", "tidalotl", "mosscub", "cinderpup", "pebbit", "zapling", "drizzlet", "sproutle", "echowl", "quickhop"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cls {
    Plain,
    Cmd,
    Dim,
    Warn,
    Ok,
    Gene,
    Title,
}

impl Cls {
    fn css(self) -> &'static str {
        match self {
            Cls::Cmd => "#c8ffe0",
            Cls::Dim => "#2f9e64",
            Cls::Warn => "#ffd23d",
            Cls::Gene => "#7fd8ff",
            Cls::Title => "#ffffff",
            Cls::Ok | Cls::Plain => "#5dff9d",
        }
    }
}

const TERM_LINES: usize = 17;

/// The board (studs: centre, yaw, size, tilt) the camera flies to; `x, z` = where the target stands.
pub fn screen_board(kind: ScreenKind, x: f32, z: f32) -> Option<Board> {
    let b = |c: [f32; 3], w: f32, h: f32, tilt: f32| Some(Board { c: town::m(c[0], c[1], c[2]), yaw: 0.0, w, h, max_dist: 0.0, tilt });
    match kind {
        ScreenKind::Tv if x < 240.0 => b([205.0, 4.35, -7.11], 3.8, 2.1, 0.0),
        ScreenKind::Tv => b([269.0, 5.2, -7.61], 3.0, 1.8, 0.0),
        ScreenKind::Pc => b([201.4, 4.8, 33.69], 2.6, 1.6, 0.0),
        ScreenKind::Lab => b([345.2, 4.33, if z < 0.5 { -2.0 } else { 3.5 } - 0.17], 2.2, 1.64, std::f32::consts::FRAC_PI_4),
        ScreenKind::Piano => None,
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[derive(Default)]
pub struct Screens {
    pub kind: Option<ScreenKind>,
    pub time: f32,
    pub board: Option<Board>,
    pub channel: usize,
    weather: u8,
    pub line: usize,
    line_t: f32,
    voiced: bool,
    noise: u32,
    key_lit: [f32; PIANO_KEYS],
    /// piano notes played (tests)
    pub notes: u32,
    pub app: usize,
    fact: u32,
    rng: u32,
    term: VecDeque<(String, Cls)>,
    /// lines the terminal wrote (tests)
    pub term_n: u32,
    queue: Vec<(String, Cls, bool)>,
    queue_i: usize,
    typing: String,
    typed: usize,
    term_wait: f32,
    /// what `scr` holds (rebuilt when this changes)
    built: String,
    /// the picture is up (the camera is there)
    pub shown: bool,
}

impl Screens {
    pub fn is_open(&self) -> bool {
        self.kind.is_some()
    }

    fn rand(&mut self) -> u32 {
        let mut x = if self.rng != 0 { self.rng } else { 0x9e37_79b9 };
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }
    fn randf(&mut self) -> f32 {
        (self.rand() & 0xff_ffff) as f32 / 16_777_216.0
    }

    /// Opens `kind`; `x, z` = where its target stands (picks the TV or computer), `weather` 0 day 1 night 2 rain.
    pub fn open(&mut self, ctx: &mut Ctx, sound: &mut Sound, kind: ScreenKind, x: f32, z: f32, weather: u8, seed: u32) {
        *self = Screens { kind: Some(kind), weather, rng: seed | 1, board: screen_board(kind, x, z), ..Default::default() };
        self.fact = self.rand();
        if kind == ScreenKind::Tv {
            self.line_t = -0.5; // the first line after the picture is up
            self.tv_line(ctx, sound);
        }
        if kind == ScreenKind::Lab {
            self.push("QuillOS 3.7 \u{b7} Brick Beast Research Terminal", Cls::Title);
            self.push("read-only session \u{b7} logged by Prof. Quill", Cls::Dim);
            self.term_wait = 1.0;
        }
        log_info!("screen: {kind:?} open");
    }

    pub fn close(&mut self, ctx: &mut Ctx, sound: &mut Sound) {
        if self.kind.take().is_some() {
            sound.stop_voice(ctx);
        }
        for s in ["scr", "scr_live", "scr_pos"] {
            ctx.ui.set(s, "");
        }
        self.built.clear();
        self.shown = false;
    }

    fn tv_line(&mut self, ctx: &mut Ctx, sound: &mut Sound) {
        let lines = tv_lines(self.channel, self.weather);
        let v = lines[self.line % lines.len()].2;
        self.voiced = v.is_some() && sound.voices_on;
        sound.voice(ctx, v);
        self.line_t = 0.0;
    }

    /// Every frame while open: keys and clicks (`actions`). Returns true when it closed.
    pub fn update(&mut self, ctx: &mut Ctx, sound: &mut Sound, g: &GameSave, actions: &[String], keys: bool, dt: f32) -> bool {
        let Some(kind) = self.kind else { return false };
        const CODES: [&str; 10] = ["Escape", "KeyE", "ArrowRight", "KeyD", "ArrowLeft", "KeyA", "Backspace", "Digit1", "Digit2", "Digit3"];
        let down: Vec<&str> = if keys { CODES.iter().chain(PIANO_CODES.iter()).chain(["Digit4"].iter()).copied().filter(|k| !k.is_empty() && ctx.input.key_pressed(k)).collect() } else { vec![] };
        let pressed = |k: &str| down.contains(&k);
        if pressed("Escape") || pressed("KeyE") || actions.iter().any(|a| a == "scr_close") {
            self.close(ctx, sound);
            sound.sfx(ctx, Sfx::Blip, 0.9, 0.3);
            return true;
        }
        self.time += dt;
        for k in &mut self.key_lit {
            *k = (*k - dt).max(0.0);
        }
        match kind {
            ScreenKind::Tv => {
                let mut zap = 0i32;
                if pressed("ArrowRight") || pressed("KeyD") || actions.iter().any(|a| a == "tv_next") {
                    zap = 1;
                }
                if pressed("ArrowLeft") || pressed("KeyA") || actions.iter().any(|a| a == "tv_prev") {
                    zap = -1;
                }
                if zap != 0 {
                    self.channel = (self.channel as i32 + zap).rem_euclid(TV_CHANNELS.len() as i32) as usize;
                    self.line = 0;
                    self.noise += 1;
                    sound.sfx(ctx, Sfx::Blip, 0.7, 0.5);
                    self.tv_line(ctx, sound);
                    self.line_t = -0.35;
                    return false;
                }
                self.line_t += dt;
                let lines = tv_lines(self.channel, self.weather);
                let text = lines[self.line % lines.len()].1;
                // next line: the voice is over (plus a breath), or a while for a line without one
                let over = if self.voiced { self.line_t > 1.2 && !sound.voice_active(ctx) } else { self.line_t > 1.2 + text.len() as f32 * 0.055 };
                if over || self.line_t > 14.0 {
                    self.line = (self.line + 1) % lines.len();
                    self.tv_line(ctx, sound);
                    if self.line == 0 {
                        self.line_t = -1.9; // a longer pause before the show starts over
                    }
                }
            }
            ScreenKind::Piano => {
                for k in 0..PIANO_KEYS {
                    let code = PIANO_CODES[k];
                    let hit = (!code.is_empty() && pressed(code)) || actions.iter().any(|a| a.strip_prefix("piano:").and_then(|n| n.parse::<usize>().ok()) == Some(k));
                    if hit {
                        sound.sfx(ctx, Sfx::Piano, 2f32.powf(PIANO_SEMI[k] as f32 / 12.0), 0.9);
                        self.key_lit[k] = 0.18;
                        self.notes += 1;
                    }
                }
            }
            ScreenKind::Pc => {
                let mut app = actions.iter().find_map(|a| a.strip_prefix("pc_app:").and_then(|n| n.parse::<usize>().ok()));
                if self.app != 0 && (pressed("Backspace") || pressed("ArrowLeft")) {
                    app = Some(0);
                }
                if self.app == 0 {
                    for (i, k) in ["Digit1", "Digit2", "Digit3", "Digit4"].into_iter().enumerate() {
                        if pressed(k) {
                            app = Some(i + 1);
                        }
                    }
                }
                if let Some(a) = app.filter(|a| *a < APPS.len() && *a != self.app) {
                    self.app = a;
                    sound.sfx(ctx, Sfx::Blip, if a == 0 { 1.1 } else { 1.4 }, 0.35);
                }
            }
            ScreenKind::Lab => {
                if self.time > 0.5 {
                    self.term_update(ctx, sound, g, dt);
                }
            }
        }
        false
    }

    // ------------------------------------------------------------------ the lab terminal

    fn push(&mut self, s: &str, c: Cls) {
        self.term.push_back((s.to_string(), c));
        while self.term.len() > TERM_LINES {
            self.term.pop_front();
        }
        self.term_n += 1;
    }

    fn bar(p: f32) -> String {
        let p = p.clamp(0.0, 1.0);
        let n = (p * 20.0 + 0.5) as usize;
        format!("[{}{}] {}%", "#".repeat(n), ".".repeat(20 - n), (p * 100.0 + 0.5) as i32)
    }

    /// One session: commands typed letter by letter and what they print (LabTerminal.svelte script()).
    fn term_script(&mut self, g: &GameSave) {
        self.queue.clear();
        self.queue_i = 0;
        let sp = SPECIMENS[self.rand() as usize % SPECIMENS.len()];
        let gene: String = (0..24).map(|_| ['B', 'R', 'I', 'C', 'K'][self.rand() as usize % 5]).collect();
        let hex: Vec<String> = (0..2).map(|_| (0..8).map(|_| format!("{:02x}", self.rand() & 255)).collect::<Vec<_>>().join(" ")).collect();
        let (cost, power) = (1 + self.rand() % 6, 1 + self.rand() % 9);
        let kinds = ["on reveal", "ongoing", "activate", "end of turn", "game start", "vanilla"];
        let kind = kinds[self.rand() as usize % kinds.len()];
        let mut q = |s: String, c: Cls, cmd: bool| self.queue.push((s, c, cmd));
        q(format!("./analyze --specimen {sp}"), Cls::Cmd, true);
        let name = format!("{}{}", sp[..1].to_uppercase(), &sp[1..]);
        q(format!("loading sample {name}.brk ..."), Cls::Dim, false);
        q(format!("  cost {cost} | power {power} | {kind}"), Cls::Plain, false);
        q(format!("  genome  {gene}"), Cls::Gene, false);
        let r = (self.rand() & 0xff_ffff) as f32 / 16_777_216.0;
        let r2 = (self.rand() & 0xff_ffff) as f32 / 16_777_216.0;
        let r3 = (self.rand() & 0xff_ffff) as f32 / 16_777_216.0;
        let r4 = (self.rand() & 0xff_ffff) as f32 / 16_777_216.0;
        let mut q = |s: String, c: Cls, cmd: bool| self.queue.push((s, c, cmd));
        q(format!("  {} stud resonance", Self::bar(0.3 + r * 0.7)), Cls::Ok, false);
        q("cat /var/log/route1.log | tail -3".into(), Cls::Cmd, true);
        q(format!("  [{:.2}] wild pack detected in tall grass", r2 * 99.0), Cls::Warn, false);
        q(format!("  [{:.2}] glimmer flux {:.3} kBr", r3 * 99.0, r4 * 3.0), Cls::Plain, false);
        q("hexdump -n 16 /dev/brickcore".into(), Cls::Cmd, true);
        q(format!("  0000  {}", hex[0]), Cls::Dim, false);
        q(format!("  0008  {}", hex[1]), Cls::Dim, false);
        q("./collection --status".into(), Cls::Cmd, true);
        let own = owned(g);
        q(format!("  {} {own}/{} beasts", Self::bar(own as f32 / crate::dialog::CARD_POOL as f32), crate::dialog::CARD_POOL), Cls::Ok, false);
        q(format!("  battles won: {}  \u{b7}  rival defeated: {}x", g.wins, g.rival_wins), Cls::Plain, false);
        let jokes = [
            ("ping dex-house.local", "  64 bytes from dex-house: \"SNAP!\" time=1ms"),
            ("sudo make coffee", "  [sudo] password for quill: ******  ACCESS DENIED (no coffee)"),
            ("ssh quill@moon-base", "  connection refused: the moon is made of bricks"),
            ("git log --oneline -1", "  a1b2c3d fix: beasts escaping through the vents again"),
        ];
        let j = jokes[self.rand() as usize % 4];
        self.queue.push((j.0.into(), Cls::Cmd, true));
        self.queue.push((j.1.into(), Cls::Warn, false));
    }

    fn term_update(&mut self, ctx: &mut Ctx, sound: &mut Sound, g: &GameSave, dt: f32) {
        self.term_wait -= dt;
        while self.term_wait <= 0.0 {
            if self.queue_i >= self.queue.len() {
                self.term_script(g);
            }
            let (text, cls, cmd) = self.queue[self.queue_i].clone();
            if !cmd {
                self.push(&text, cls);
                self.queue_i += 1;
                self.term_wait += 0.12 + self.randf() * 0.26;
                continue;
            }
            let len = text.chars().count();
            if self.typed < len {
                self.typed += 1;
                self.typing = text.chars().take(self.typed).collect();
                if self.typed % 3 == 0 {
                    let p = 1.6 + self.randf() * 0.3;
                    sound.sfx(ctx, Sfx::Blip, p, 0.12);
                }
                self.term_wait += if self.typed < len { 0.035 + self.randf() * 0.045 } else { 0.28 };
            } else {
                self.push(&format!("quill@lab:~$ {text}"), Cls::Cmd);
                self.typing.clear();
                self.typed = 0;
                self.queue_i += 1;
                self.term_wait += 0.55; // a breath before the next command starts typing
            }
        }
    }

    // ------------------------------------------------------------------ drawing

    /// Where the board is on the page now (CSS px: x, y, w, h), as the camera sees it.
    fn board_rect(ctx: &Ctx, b: &Board) -> Option<[f32; 4]> {
        let (view, proj, _) = ctx.renderer.camera_matrices(&ctx.world)?;
        let (sw, sh) = (ctx.canvas.client_width().max(1) as f32, ctx.canvas.client_height().max(1) as f32);
        let n = Vec3::new(b.yaw.sin() * b.tilt.cos(), b.tilt.sin(), b.yaw.cos() * b.tilt.cos());
        let right = Vec3::new(b.yaw.cos(), 0.0, -b.yaw.sin());
        let up = n.cross(right);
        let vp = proj * view;
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for i in 0..4 {
            let sx = if i & 1 != 0 { 0.5 } else { -0.5 };
            let sy = if i & 2 != 0 { 0.5 } else { -0.5 };
            let p = b.c + right * (sx * b.w * STUD) + up * (sy * b.h * STUD);
            let c = vp * p.extend(1.0);
            if c.w <= 0.01 {
                return None;
            }
            let (x, y) = ((c.x / c.w * 0.5 + 0.5) * sw, (0.5 - c.y / c.w * 0.5) * sh);
            x0 = x0.min(x);
            x1 = x1.max(x);
            y0 = y0.min(y);
            y1 = y1.max(y);
        }
        (x1 - x0 > 40.0 && y1 - y0 > 30.0 && x1 - x0 < sw * 3.0).then_some([x0, y0, x1 - x0, y1 - y0])
    }

    /// The HUD slots. `zoom_t`: how far the camera has flown to the screen (0..1).
    pub fn draw(&mut self, ctx: &mut Ctx, g: &GameSave, zoom_t: f32, touch: bool) {
        let Some(kind) = self.kind else { return };
        let (sw, sh) = (ctx.canvas.client_width().max(1) as f32, ctx.canvas.client_height().max(1) as f32);
        if kind == ScreenKind::Piano {
            self.shown = true;
            let key = format!("piano{touch}");
            if self.built != key {
                ctx.ui.set("scr", &piano_html(touch || sw <= 640.0));
                self.built = key;
            }
            let lit: String = (0..PIANO_KEYS).filter(|k| self.key_lit[*k] > 0.0).map(|k| format!("#ui .bh-pk{k}{{filter:brightness(.8) sepia(1) hue-rotate(170deg) saturate(4)}}")).collect();
            ctx.ui.set("scr_pos", &format!("<style>{lit}</style>"));
            ctx.ui.set("scr_live", "");
            return;
        }
        // it switches on once the camera is nearly there
        let shown = if self.board.is_some() { ((zoom_t - 0.75) / 0.25).clamp(0.0, 1.0) } else { 1.0 };
        let rect = self.board.as_ref().and_then(|b| Self::board_rect(ctx, b));
        if shown <= 0.0 || (self.board.is_some() && rect.is_none() && zoom_t < 1.0) {
            ctx.ui.set("scr_pos", "");
            return;
        }
        self.shown = true;
        let mut r = rect.unwrap_or_else(|| {
            let w = (sw * 0.9).min(760.0);
            let h = w * if kind == ScreenKind::Tv { 0.55 } else { 0.62 };
            [(sw - w) * 0.5, (sh - h) * 0.45, w, h]
        });
        // a phone held upright: the real screen is too small to read, a panel that fits the phone
        if sh > sw * 1.15 {
            let pw = sw - 16.0;
            let ph = if kind == ScreenKind::Tv { pw * 0.62 } else { (pw * 1.25).min(sh * 0.72) };
            r = [(sw - pw) * 0.5, (sh - ph) * 0.42, pw, ph];
        }
        let fs = match kind {
            ScreenKind::Tv => r[3] / 14.0,
            ScreenKind::Pc => (r[3] / 17.0).min(r[2] / 22.0),
            _ => (r[3] / 30.0).min(r[2] / 28.0),
        };
        ctx.ui.set(
            "scr_pos",
            &format!(
                "<style>#ui [data-slot=scr],#ui [data-slot=scr_live]{{position:fixed;left:{:.1}px;top:{:.1}px;width:{:.1}px;height:{:.1}px;font-size:{:.2}px;opacity:{shown:.2};z-index:35}}</style>",
                r[0], r[1], r[2], r[3], fs
            ),
        );
        let narrow = r[2] < 520.0;
        let key = format!("{kind:?}{}{}{}{narrow}", self.channel, self.noise, self.app);
        if self.built != key {
            let html = match kind {
                ScreenKind::Tv => tv_html(self.channel, self.noise, self.weather, g),
                ScreenKind::Pc => pc_html(self.app, g, self.fact, narrow),
                _ => lab_html(narrow),
            };
            ctx.ui.set("scr", &html);
            self.built = key;
        }
        let live = match kind {
            ScreenKind::Tv => {
                let lines = tv_lines(self.channel, self.weather);
                let (who, text, _) = lines[self.line % lines.len()];
                if self.line_t < 0.0 {
                    String::new()
                } else {
                    format!("<div class='tv-sub{}'><b>{}:</b> {}</div>", if self.channel == 0 { " top" } else { "" }, esc(who), esc(text))
                }
            }
            ScreenKind::Lab => {
                let lines: String = self.term.iter().map(|(t, c)| format!("<div style='color:{}{}'>{}</div>", c.css(), if *c == Cls::Title { ";font-weight:700" } else { "" }, esc(t))).collect();
                format!(
                    "<div class='lab-con{}'>{lines}<div style='color:#c8ffe0'>quill@lab:~$ {}<i class='lab-cur'></i></div></div>",
                    if narrow { " narrow" } else { "" },
                    esc(&self.typing)
                )
            }
            _ => String::new(),
        };
        ctx.ui.set("scr_live", &live);
    }

    /// For `debug_state`.
    pub fn describe(&self) -> serde_json::Value {
        let tv = self.kind.map(|_| {
            let lines = tv_lines(self.channel, self.weather);
            lines[self.line % lines.len()].1
        });
        serde_json::json!({
            "kind": self.kind, "shown": self.shown, "channel": TV_CHANNELS[self.channel], "line": self.line,
            "tv_text": if self.kind == Some(ScreenKind::Tv) { tv } else { None },
            "notes": self.notes, "app": APPS[self.app].0, "term_lines": self.term_n, "typing": self.typing,
        })
    }
}

/// Cards owned (the collection's bits).
fn owned(g: &GameSave) -> u32 {
    (g.collection & ((1u64 << crate::dialog::CARD_POOL) - 1)).count_ones()
}

/// An icon of the art set (`assets/screens/ic_*.svg`) in a colour.
fn icon(name: &str, size_em: f32, color: &str) -> String {
    format!("<i class='scr-ic' style='width:{size_em}em;height:{size_em}em;background:{color};-webkit-mask-image:url(assets/screens/ic_{name}.svg);mask-image:url(assets/screens/ic_{name}.svg)'></i>")
}

// ---------------------------------------------------------------------- the TV

fn tv_html(ch: usize, noise: u32, weather: u8, g: &GameSave) -> String {
    let art = match ch {
        0 => tv_bbn(),
        1 => tv_weather(weather),
        2 => tv_cartoon(),
        _ => tv_wild(),
    };
    let news = [
        "Glimmerlings spotted folding light into tiny bricks!".to_string(),
        format!("Local trainer owns {} of {} Brick Beast cards", owned(g), crate::dialog::CARD_POOL),
        if g.rival_wins > 0 { "BREAKING: Dex loses again, demands a rematch".into() } else { "Dex claims his deck is \"unbeatable\"".to_string() },
        format!("{} golden studs found so far. Where are the rest?", g.golden_count()),
        "Mom reminds everyone: jackets on Route 1!".into(),
    ];
    let ticker = esc(&news.join("   \u{2605}   "));
    let overlay = match ch {
        0 => format!(
            "<div class='tv-lower'><div class='tv-live'><span class='tv-dot'></span>LIVE</div><div class='tv-head'>{}</div></div><div class='tv-ticker'><span>{ticker}   \u{2605}   {ticker}</span></div>",
            esc(&news[0])
        ),
        1 => {
            let night = weather == 1;
            let temp = if night { 13 } else if weather == 2 { 15 } else { 21 };
            let sub = if night { "Clear night \u{b7} fireflies likely" } else if weather == 2 { "Rain \u{b7} puddles everywhere" } else { "Sunny \u{b7} perfect for battles" };
            format!("<div class='tv-weather'><div class='tv-temp'>{temp}\u{b0}</div><div><div style='font-weight:900'>Brickhollow</div><div style='font-size:.75em;opacity:.9'>{sub}</div></div></div>")
        }
        _ => String::new(),
    };
    let stat = if noise > 0 {
        "<svg class='tv-static' viewBox='0 0 100 60' preserveAspectRatio='none'><filter id='tvnoise'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' seed='1'><animate attributeName='seed' values='1;9;3;7;2' dur='0.3s' repeatCount='indefinite'/></feTurbulence><feColorMatrix type='saturate' values='0'/></filter><rect width='100' height='60' filter='url(#tvnoise)'/></svg>"
    } else {
        ""
    };
    format!(
        "<div class='tv'>{art}{overlay}{stat}<div class='tv-scan'></div><div class='tv-osd'><button data-action='tv_prev' title='Previous channel'>{}</button><span style='font-weight:700'>CH {} \u{b7} {}</span><button data-action='tv_next' title='Next channel'>{}</button><span style='flex:1'></span><button class='off' data-action='scr_close' title='Turn off (E / Esc)'>{}</button></div></div>",
        icon("chevron_left", 1.2, "#fff"),
        ch + 1,
        TV_CHANNELS[ch],
        icon("chevron_right", 1.2, "#fff"),
        icon("power", 1.2, "#fff")
    )
}

const SVG_OPEN: &str = "<svg viewBox='0 0 320 176' preserveAspectRatio='xMidYMid slice' class='tv-art'>";

fn tv_bbn() -> String {
    let mut s = String::from(SVG_OPEN);
    s += "<defs><linearGradient id='studio' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#1e5aa8'/><stop offset='1' stop-color='#0b2550'/></linearGradient></defs><rect width='320' height='176' fill='url(#studio)'/>";
    for i in 0..6 {
        s += &format!("<circle cx='{}' cy='30' r='14' fill='#fff' opacity='0.06'/>", 30 + i * 52);
    }
    s += "<rect x='176' y='22' width='120' height='70' rx='6' fill='#0d1b2a' stroke='#ffd23d' stroke-width='2'/><g transform='translate(236 57)'><polygon points='0,-22 6,-6 22,-6 9,4 14,20 0,10 -14,20 -9,4 -22,-6 -6,-6' fill='#ffd23d'><animateTransform attributeName='transform' type='rotate' from='0' to='360' dur='6s' repeatCount='indefinite'/></polygon></g>";
    s += "<g transform='translate(92 66)'><g><animateTransform attributeName='transform' type='translate' values='0 0; 0 -2; 0 0' dur='1.6s' repeatCount='indefinite'/><rect x='-30' y='0' width='60' height='58' rx='10' fill='#4fc06a' stroke='#1e7a35' stroke-width='3'/>";
    for x in [-18, 0, 18] {
        s += &format!("<rect x='{}' y='-6' width='12' height='8' rx='3' fill='#4fc06a' stroke='#1e7a35' stroke-width='2'/>", x - 6);
    }
    s += "<circle cx='-12' cy='22' r='7' fill='#fff'/><circle cx='12' cy='22' r='7' fill='#fff'/>";
    for cx in [-11, 13] {
        s += &format!("<circle cx='{cx}' cy='23' r='3.5' fill='#1b2a34'><animate attributeName='r' values='3.5;3.5;0.4;3.5' keyTimes='0;0.9;0.95;1' dur='3.2s' repeatCount='indefinite'/></circle>");
    }
    s += "<ellipse cx='0' cy='40' rx='8' ry='3' fill='#1b2a34'><animate attributeName='ry' values='2;6;2;4;2' dur='0.7s' repeatCount='indefinite'/></ellipse><polygon points='-4,50 4,50 6,58 -6,58' fill='#c91a09'/></g></g>";
    s += "<rect x='20' y='118' width='150' height='40' rx='4' fill='#c91a09'/><rect x='20' y='118' width='150' height='8' fill='#ff5a4a'/><text x='95' y='146' text-anchor='middle' font-size='16' font-weight='900' fill='#fff' font-family='sans-serif'>BBN</text></svg>";
    s
}

fn tv_weather(weather: u8) -> String {
    let night = weather == 1;
    let mut s = String::from(SVG_OPEN);
    s += &format!("<rect width='320' height='176' fill='{}'/>", if night { "#16213e" } else if weather == 2 { "#6f8aa6" } else { "#7cc6ff" });
    if night {
        for i in 0..18 {
            s += &format!("<circle cx='{}' cy='{}' r='1.4' fill='#fff'><animate attributeName='opacity' values='0.2;1;0.2' dur='{}s' repeatCount='indefinite'/></circle>", (i * 53) % 320, (i * 37) % 110, 2 + i % 3);
        }
        s += "<circle cx='230' cy='58' r='26' fill='#fff6c8'/><circle cx='242' cy='50' r='24' fill='#16213e'/>";
    } else if weather != 2 {
        s += "<g transform='translate(230 58)'><g><animateTransform attributeName='transform' type='rotate' from='0' to='360' dur='14s' repeatCount='indefinite'/>";
        for i in 0..12 {
            s += &format!("<rect x='-3' y='-46' width='6' height='14' rx='3' fill='#ffd23d' transform='rotate({})'/>", i * 30);
        }
        s += "</g><circle r='26' fill='#ffd23d' stroke='#f2a705' stroke-width='3'/></g>";
    }
    for (x, y, d) in [(40, 50, 18), (120, 80, 24), (280, 110, 16)] {
        s += &format!("<g opacity='0.95'><animateTransform attributeName='transform' type='translate' values='-20 0; 20 0; -20 0' dur='{d}s' repeatCount='indefinite'/><g transform='translate({x} {y})'><ellipse cx='0' cy='0' rx='26' ry='12' fill='#fff'/><ellipse cx='-12' cy='-6' rx='14' ry='11' fill='#fff'/><ellipse cx='10' cy='-9' rx='16' ry='13' fill='#fff'/></g></g>");
    }
    if weather == 2 {
        for i in 0..24 {
            let x = (i * 41) % 320;
            s += &format!("<line x1='{x}' y1='0' x2='{}' y2='12' stroke='#cfe6ff' stroke-width='1.5' opacity='0.7'><animateTransform attributeName='transform' type='translate' values='0 -20; 0 180' dur='{}s' begin='-{}s' repeatCount='indefinite'/></line>", x - 3, 0.6 + (i % 4) as f32 * 0.1, (i % 7) as f32 * 0.1);
        }
    }
    s += "<path d='M0 150 Q 60 118 120 146 T 240 140 T 320 136 V176 H0 Z' fill='#3e9a32'/><path d='M0 162 Q 80 140 160 160 T 320 154 V176 H0 Z' fill='#4fc06a'/></svg>";
    s
}

fn tv_cartoon() -> String {
    let mut s = String::from(SVG_OPEN);
    s += "<rect width='320' height='176' fill='#ffe7a8'/><g><animateTransform attributeName='transform' type='translate' from='0 0' to='-160 0' dur='4s' repeatCount='indefinite'/>";
    for i in 0..6 {
        s += &format!("<path d='M{} 150 q 40 -50 80 0' fill='#f2a705' opacity='0.5'/>", i * 80 - 10);
    }
    s += "</g><rect y='150' width='320' height='26' fill='#c96a2a'/><g><animateTransform attributeName='transform' type='translate' from='0 0' to='-40 0' dur='0.8s' repeatCount='indefinite'/>";
    for i in 0..10 {
        s += &format!("<rect x='{}' y='150' width='20' height='5' fill='#a85520'/>", i * 40);
    }
    s += "</g><g transform='translate(130 0)'><g><animateTransform attributeName='transform' type='translate' values='0 120; 0 64; 0 120' keyTimes='0;0.5;1' keySplines='0.2 0.8 0.4 1; 0.6 0 0.8 0.2' calcMode='spline' dur='0.8s' repeatCount='indefinite'/><rect x='-24' y='-30' width='48' height='30' rx='5' fill='#c91a09' stroke='#7a0f05' stroke-width='3'/>";
    for x in [-12, 12] {
        s += &format!("<rect x='{}' y='-37' width='14' height='8' rx='3' fill='#c91a09' stroke='#7a0f05' stroke-width='2'/>", x - 7);
    }
    s += "<circle cx='-8' cy='-17' r='4' fill='#fff'/><circle cx='10' cy='-17' r='4' fill='#fff'/><circle cx='-7' cy='-17' r='2' fill='#1b2a34'/><circle cx='11' cy='-17' r='2' fill='#1b2a34'/><path d='M-6 -8 q 7 6 14 0' stroke='#1b2a34' stroke-width='2.5' fill='none' stroke-linecap='round'/></g></g>";
    s += "<g transform='translate(60 138)'><g><animateTransform attributeName='transform' type='rotate' from='0' to='360' dur='1.2s' repeatCount='indefinite'/><circle r='12' fill='#ffd23d' stroke='#a87a00' stroke-width='3'/><rect x='-6' y='-2' width='12' height='4' rx='2' fill='#a87a00'/></g></g>";
    s += "<text x='300' y='26' text-anchor='end' font-size='14' font-weight='900' fill='#c91a09' font-family='sans-serif'>BRICK &amp; STUD</text></svg>";
    s
}

fn tv_wild() -> String {
    let mut s = String::from(SVG_OPEN);
    s += "<defs><linearGradient id='dusk' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#ffcf8a'/><stop offset='1' stop-color='#ffefc9'/></linearGradient><radialGradient id='lens' cx='0.5' cy='0.5' r='0.6'><stop offset='0.7' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#000' stop-opacity='0.75'/></radialGradient></defs>";
    s += "<rect width='320' height='176' fill='url(#dusk)'/><circle cx='250' cy='60' r='22' fill='#fff3c4' opacity='0.9'/><g><animateTransform attributeName='transform' type='translate' values='0 0; -30 0; 0 0' dur='24s' repeatCount='indefinite'/><path d='M-20 130 Q 80 100 180 126 T 380 118 V176 H-20 Z' fill='#6bb04a'/>";
    s += "<g transform='translate(150 118)'><g><animateTransform attributeName='transform' type='translate' values='-30 0; -26 -3; -22 0; -18 -3; -14 0; -18 -3; -22 0; -26 -3; -30 0' dur='9s' repeatCount='indefinite'/><rect x='-14' y='-18' width='28' height='18' rx='4' fill='#3d9bff' stroke='#1e5aa8' stroke-width='2'/>";
    for x in [-7, 7] {
        s += &format!("<rect x='{}' y='-23' width='8' height='6' rx='2' fill='#3d9bff' stroke='#1e5aa8' stroke-width='1.5'/>", x - 4);
    }
    s += "<circle cx='-5' cy='-10' r='3' fill='#fff'/><circle cx='6' cy='-10' r='3' fill='#fff'/><circle cx='-4' cy='-10' r='1.5' fill='#1b2a34'><animate attributeName='cx' values='-4;-3;-4;-5;-4' dur='5s' repeatCount='indefinite'/></circle><circle cx='7' cy='-10' r='1.5' fill='#1b2a34'><animate attributeName='cx' values='7;8;7;6;7' dur='5s' repeatCount='indefinite'/></circle></g></g>";
    s += "<g transform='translate(196 116)'><rect x='-12' y='-6' width='24' height='7' rx='1.5' fill='#c91a09'/>";
    for x in [-9, -3, 3, 9] {
        s += &format!("<rect x='{}' y='-9' width='4' height='3' rx='1' fill='#c91a09'/>", x - 2);
    }
    s += "</g>";
    for i in 0..26 {
        let (x, a, b, c, d) = (i * 14 - 10, 3 + i % 3, 30 + (i * 7) % 22, if i % 2 == 1 { 6 } else { -4 }, 44 + (i * 11) % 26);
        s += &format!(
            "<path d='M{x} 176 q {a} -{b} {c} -{d}' stroke='{}' stroke-width='4' fill='none' stroke-linecap='round'><animateTransform attributeName='transform' type='rotate' values='0 {p} 176; {r} {p} 176; 0 {p} 176' dur='{}s' repeatCount='indefinite'/></path>",
            if i % 2 == 1 { "#3e8a2a" } else { "#4f9e33" },
            3 + i % 4,
            p = i * 14,
            r = if i % 2 == 1 { 2 } else { -2 }
        );
    }
    s += "</g><rect width='320' height='176' fill='url(#lens)'/><text x='16' y='24' font-size='12' font-weight='700' fill='#fff' opacity='0.85' font-family='sans-serif'>WILD BRICKS</text></svg>";
    s
}

// ---------------------------------------------------------------------- the piano

fn piano_html(narrow: bool) -> String {
    let mut keys = String::new();
    let ww = 100.0 / PIANO_WHITE as f32;
    for k in 0..PIANO_WHITE {
        keys += &format!("<button class='pk-w bh-pk{k}' data-action='piano:{k}' style='left:{:.2}%;width:{ww:.2}%'><small>{}</small></button>", k as f32 * ww, PIANO_LABELS[k]);
    }
    // the black keys sit between the white ones: after C, D, F, G, A, C, D
    for (b, after) in [0, 1, 3, 4, 5, 7, 8].into_iter().enumerate() {
        let k = PIANO_WHITE + b;
        // (the last one, D#5, has no E5 after it: it keeps inside the keyboard)
        let left = ((after + 1) as f32 * ww - ww * 0.32).min(100.0 - ww * 0.64);
        keys += &format!("<button class='pk-b bh-pk{k}' data-action='piano:{k}' style='left:{left:.2}%;width:{:.2}%'></button>", ww * 0.64);
    }
    let hint = if narrow { "Tap the keys to play. \u{d7} to stop." } else { "Keys A..L (white) and W, T, Y, U, O, P (black), or click the keys. E / Esc / \u{d7} to stop." };
    format!(
        "<div class='pi-dim'><div class='pi'><button class='pi-x' data-action='scr_close' title='Stop playing'>\u{d7}</button><div class='pi-title'>Lumi's piano</div><div class='pi-keys'>{keys}</div><div class='pi-hint'>{hint}</div></div></div>"
    )
}

// ---------------------------------------------------------------------- BrickOS

fn stickers(g: &GameSave) -> [(&'static str, &'static str, &'static str, bool); 10] {
    [
        ("Card Trainer", "Get a deck from Prof. Quill", "#3d9bff", g.has_deck),
        ("Rival Crusher", "Beat Dex in a card battle", "#c91a09", g.rival_wins > 0),
        ("Gold Digger", "Find 5 golden studs", "#f2a705", g.golden_count() >= 5),
        ("Duck Friend", "Feed the ducks 3 times", "#27b05a", g.duck_feeds >= 3),
        ("Treasure Hunter", "Open a treasure chest", "#8a5a2b", g.chests >= 1),
        ("Firefly Catcher", "Catch 5 fireflies", "#9bd23d", g.fireflies >= 5),
        ("Seeker", "Find Pip at hide and seek", "#ff8ac0", g.seek_wins >= 1),
        ("Striker", "Score a goal", "#1b2a34", g.goals >= 1),
        ("Collector", "Own 20 different cards", "#7a4fd1", owned(g) >= 20),
        ("Voxelmon!", "Raise a card into a Voxelmon", "#16b5c9", g.voxelmon > 0),
    ]
}

/// The inbox, newest first, from what has happened so far.
fn messages(g: &GameSave) -> Vec<(&'static str, &'static str)> {
    let mut m = vec![];
    if g.voxelmon > 0 {
        m.push(("Prof. Quill", "A real Voxelmon! Show it to me at the village exit, then Route 1 is yours."));
    }
    if g.rival_wins > 0 {
        m.push(("Dex", "That was LUCK. Rematch. Tomorrow. Bring snacks."));
    }
    if g.has_deck {
        m.push(("Prof. Quill", "Your deck suits you. Dex is waiting at his big house - go show him!"));
    }
    m.push(("Mom", "Breakfast is in the kitchen. Don't forget your jacket!"));
    m.push(("Dex", "SNAP! snap. SNAP!!!"));
    m.push(("Prof. Quill", "Come to the lab when you wake up. I have something for you!"));
    m
}

fn pc_html(app: usize, g: &GameSave, fact: u32, narrow: bool) -> String {
    let mut bar = format!("{}<b>BrickOS</b>", icon("brick", 1.1, "#ffd23d"));
    if app != 0 {
        bar += &format!("<button class='pc-back' data-action='pc_app:0'>{}{}</button>", icon("back", 0.9, "#fff"), APPS[app].0);
    }
    bar += &format!("<span style='flex:1'></span>{}<span>{}</span><button class='pc-off' data-action='scr_close' title='Log off (E / Esc)'>{}</button>", icon("coin", 0.95, "#ffd23d"), g.studs, icon("power", 0.95, "#fff"));
    let st = stickers(g);
    let body = match app {
        0 => {
            let icons: String = APPS
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, (label, ic, col))| {
                    let badge = match i {
                        1 => format!("<i class='pc-badge'>{}</i>", messages(g).len()),
                        4 => format!("<i class='pc-badge'>{}/{}</i>", st.iter().filter(|s| s.3).count(), st.len()),
                        _ => String::new(),
                    };
                    format!(
                        "<button class='pc-icon' data-action='pc_app:{i}' style='animation-delay:{}ms'><span class='pc-tile' style='background:linear-gradient(160deg,color-mix(in srgb,{col},#fff 25%),{col})'>{}{badge}</span><span>{label}</span></button>",
                        i * 50,
                        icon(ic, 1.9, "#fff")
                    )
                })
                .collect();
            format!("<div class='pc-home'><div class='pc-welcome'>Welcome back, trainer!</div><div class='pc-icons'>{icons}</div></div>")
        }
        1 => {
            let rows: String = messages(g)
                .iter()
                .enumerate()
                .map(|(i, (f, t))| format!("<div class='pc-msg{}' style='animation-delay:{}ms'><b>{f}</b><span>{}</span></div>", if i == 0 { " new" } else { "" }, i * 70, esc(t)))
                .collect();
            format!("<div class='pc-sheet'>{rows}</div>")
        }
        2 => {
            let rows = [
                ("Battles won", g.wins.to_string()),
                ("Battles lost", g.losses.to_string()),
                ("Beat Dex", format!("{}\u{d7}", g.rival_wins)),
                ("Golden studs", format!("{}/15", g.golden_count())),
                ("Butterflies caught", "0".to_string()), // (the butterflies come with the life part)
                ("Ducks fed", g.duck_feeds.to_string()),
                ("Treasure chests", g.chests.to_string()),
                ("Pip found", g.seek_wins.to_string()),
            ];
            let cells: String = rows.iter().map(|(l, v)| format!("<div class='pc-stat'><span>{l}</span><b>{v}</b></div>")).collect();
            format!("<div class='pc-sheet'><div class='pc-stats{}'>{cells}</div></div>", if narrow { " one" } else { "" })
        }
        3 => {
            let own = owned(g);
            let facts = [
                "Brick Beasts love studs, sunshine and being collected on cards.",
                "They hide in tall grass and only come out for trainers with a deck.",
                "Six-energy cards that reach their final stage become real Voxelmon.",
                "A Beast's studs are soft at first. Within a week they click into place.",
                "Shiny Voxelmon are rare. Prof. Quill has seen exactly one. Maybe.",
            ];
            let p = (own as f32 / crate::dialog::CARD_POOL as f32).max(0.02) * 100.0;
            format!(
                "<div class='pc-sheet'><div class='pc-coll'>Collection <b>{own}/{}</b></div><div class='pc-prog'><div style='width:{p:.0}%'></div></div><div class='pc-fact'>{}<span>{}</span></div></div>",
                crate::dialog::CARD_POOL,
                icon("sticker", 1.2, "#f2a705"),
                facts[fact as usize % 5]
            )
        }
        _ => {
            let cells: String = st
                .iter()
                .enumerate()
                .map(|(i, (n, h, c, got))| {
                    format!(
                        "<div class='pc-stk{}' style='--c:{c};--r:{}deg;animation-delay:{}ms'>{}<b>{}</b><small>{h}</small></div>",
                        if *got { " got" } else { "" },
                        if *got { (i * 37 % 9) as i32 - 4 } else { 0 },
                        i * 40,
                        if *got { icon("sticker", 1.4, c) } else { icon("lock", 1.4, "#a0a8b0") },
                        if *got { n } else { "???" }
                    )
                })
                .collect();
            format!("<div class='pc-sheet'><div class='pc-stks' style='grid-template-columns:repeat({},1fr)'>{cells}</div></div>", if narrow { 3 } else { 5 })
        }
    };
    format!("<div class='pc'><div class='pc-bar'>{bar}</div><div class='pc-body'>{body}</div></div>")
}

// ---------------------------------------------------------------------- the lab terminal

fn lab_html(narrow: bool) -> String {
    // a double helix of studs, turning: each pair swings across, half a turn apart
    let mut helix = String::new();
    for i in 0..14 {
        let y = 8.0 + i as f32 * 6.4;
        let delay = -(i as f32) * 0.55 / 1.6;
        helix += &format!(
            "<line x1='20' y1='{y}' x2='80' y2='{y}' stroke='#1f6f4a' stroke-width='1.2'/><circle cx='50' cy='{y}' r='2.6' fill='#5dff9d' class='hx' style='animation-delay:{delay:.2}s'/><circle cx='50' cy='{y}' r='2.6' fill='#2a9d60' class='hx b' style='animation-delay:{delay:.2}s'/>"
        );
    }
    // the stud resonance wave: a few shapes the path morphs through
    let wave = |t: f32| -> String {
        (0..60)
            .map(|i| {
                let x = i as f32 / 59.0 * 100.0;
                let y = 20.0 + (i as f32 * 0.35 + t * 4.0).sin() * 9.0 * (t * 0.7 + i as f32 * 0.05).sin() + (i as f32 * 1.7 + t * 9.0).sin() * 2.0;
                format!("{}{x:.1} {y:.1}", if i == 0 { "M" } else { " L" })
            })
            .collect()
    };
    let shapes: Vec<String> = (0..6).map(|k| wave(k as f32 * 0.9 + 0.4)).collect();
    let values = format!("{};{}", shapes.join(";"), shapes[0]);
    format!(
        "<div class='lab'><div class='lab-bar'><i style='background:#ff5f56'></i><i style='background:#ffbd2e'></i><i style='background:#27c93f'></i><span>quill@lab \u{2014} research</span><span style='flex:1'></span><button class='lab-off' data-action='scr_close' title='Log off (E / Esc)'>{}</button></div>\
<div class='lab-main{}'><div class='lab-space'></div><div class='lab-side'>\
<div class='lab-panel'><div class='lab-cap'>GENOME HELIX</div><svg viewBox='0 0 100 100' preserveAspectRatio='xMidYMid meet'>{helix}</svg></div>\
<div class='lab-panel'><div class='lab-cap'>STUD RESONANCE <span class='lab-livedot'>\u{25cf} LIVE</span></div><svg viewBox='0 0 100 40' preserveAspectRatio='none'><line x1='0' y1='10' x2='100' y2='10' stroke='#14402c' stroke-width='0.5'/><line x1='0' y1='20' x2='100' y2='20' stroke='#14402c' stroke-width='0.5'/><line x1='0' y1='30' x2='100' y2='30' stroke='#14402c' stroke-width='0.5'/><path d='{}' fill='none' stroke='#5dff9d' stroke-width='1.2' style='filter:drop-shadow(0 0 1.5px #5dff9d)'><animate attributeName='d' values='{values}' dur='3.6s' repeatCount='indefinite'/></path></svg></div>\
</div></div><div class='lab-scan'></div></div>",
        icon("power", 1.3, "#fff"),
        if narrow { " narrow" } else { "" },
        shapes[0]
    )
}

/// The screens' styles (a style slot; set once).
pub const STYLE: &str = "<style>
#ui [data-slot=scr_live]{pointer-events:none}
#ui [data-slot=scr]:empty,#ui [data-slot=scr_live]:empty{display:none}
.scr-ic{display:inline-block;-webkit-mask-size:contain;mask-size:contain;-webkit-mask-repeat:no-repeat;mask-repeat:no-repeat;-webkit-mask-position:center;mask-position:center;vertical-align:middle;flex:none}
#ui [data-slot=scr] button{margin:0;padding:0;font:inherit;color:inherit}
.tv{position:absolute;inset:0;overflow:hidden;border-radius:.25em;background:#000;color:#fff;font-family:system-ui,sans-serif;animation:tv-on .4s ease-out}
@keyframes tv-on{0%{clip-path:inset(49% 0 49% 0);filter:brightness(4)}60%{clip-path:inset(0 0 0 0)}}
.tv-art{position:absolute;inset:0;width:100%;height:100%}
.tv-scan{position:absolute;inset:0;pointer-events:none;background:repeating-linear-gradient(0deg,rgb(0 0 0/.12) 0 1px,transparent 1px 3px);box-shadow:inset 0 0 2em rgb(0 0 0/.6)}
.tv-static{position:absolute;inset:0;width:100%;height:100%;pointer-events:none;animation:tv-fade .45s ease-in forwards}
@keyframes tv-fade{to{opacity:0}}
.tv-lower{position:absolute;left:4%;right:4%;bottom:2.3em;display:flex;font-size:.8em;box-shadow:0 .2em .5em rgb(0 0 0/.4)}
.tv-live{display:flex;align-items:center;gap:.3em;background:#c91a09;padding:.2em .6em;font-weight:900}
.tv-dot{width:.5em;height:.5em;border-radius:50%;background:#fff;animation:tv-blink 1s infinite}
@keyframes tv-blink{50%{opacity:.2}}
.tv-head{flex:1;background:#fff;color:#1b2a34;padding:.2em .6em;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.tv-ticker{position:absolute;left:0;right:0;bottom:1.15em;background:#ffd23d;color:#1b2a34;font-size:.7em;font-weight:700;white-space:nowrap;overflow:hidden;padding:.1em 0}
.tv-ticker span{display:inline-block;padding-left:100%;animation:tv-scroll 28s linear infinite}
@keyframes tv-scroll{to{transform:translateX(-100%)}}
.tv-weather{position:absolute;left:6%;top:10%;display:flex;align-items:center;gap:.5em;background:rgb(0 0 0/.35);padding:.3em .8em;border-radius:.5em;text-shadow:0 .08em .2em rgb(0 0 0/.5)}
.tv-temp{font-size:2.4em;font-weight:900;line-height:1}
.tv-sub{position:absolute;left:6%;right:6%;bottom:1.9em;padding:.25em .6em;border-radius:.3em;background:rgb(0 0 0/.62);color:#fff;font-size:.72em;line-height:1.25;text-align:center;font-family:system-ui,sans-serif;animation:tv-subin .25s ease-out}
.tv-sub.top{top:4%;bottom:auto}.tv-sub b{color:#ffd23d}
@keyframes tv-subin{from{opacity:0;translate:0 -.3em}}
.tv-osd{position:absolute;left:0;right:0;bottom:0;display:flex;align-items:center;gap:.3em;padding:.1em .4em;background:rgb(0 0 0/.55);font-size:.7em}
#ui .tv-osd button{display:grid;place-items:center;border-radius:.3em;background:rgb(255 255 255/.15);padding:.1em}
#ui .tv-osd button.off{background:#c91a09}
.pi-dim{position:fixed;inset:0;background:rgb(0 0 0/.55);display:flex;align-items:center;justify-content:center;z-index:35;pointer-events:auto}
.pi{position:relative;display:flex;flex-direction:column;align-items:center;gap:12px;padding:20px;border-radius:16px;background:linear-gradient(#2b2622,#15120f);box-shadow:0 10px 30px rgb(0 0 0/.5);color:#fff;width:min(560px,calc(100vw - 32px));box-sizing:border-box;user-select:none;touch-action:none}
.pi-title{font-size:26px;font-weight:700}
#ui .pi-x{position:absolute;right:10px;top:10px;width:40px;height:40px;border-radius:20px;background:#4a423b;border:2px solid rgb(255 255 255/.35);font-size:26px;font-weight:700;line-height:1}
.pi-keys{position:relative;width:100%;height:min(190px,32vw)}
#ui .pk-w{position:absolute;top:0;height:100%;box-sizing:border-box;border:2px solid #15120f;border-radius:0 0 6px 6px;background:linear-gradient(#fff,#e6e4dc);color:#788088;display:flex;align-items:flex-end;justify-content:center;padding-bottom:8px}
#ui .pk-b{position:absolute;top:0;height:60%;z-index:1;border-radius:0 0 5px 5px;background:linear-gradient(#3a3a40,#0c0c0e);box-shadow:0 3px 4px rgb(0 0 0/.5)}
#ui .pk-w:active,#ui .pk-b:active{filter:brightness(.8) sepia(1) hue-rotate(170deg) saturate(4)}
.pi-hint{font-size:14px;color:#bec4cc;text-align:center}
.pc{position:absolute;inset:0;display:flex;flex-direction:column;overflow:hidden;color:#fff;font-family:system-ui,sans-serif;border-radius:.2em;background:radial-gradient(circle at 75% 30%,rgb(255 255 255/.18),transparent 40%),linear-gradient(160deg,#2f7fe0,#5b9cf0 55%,#9b7be8);animation:pc-boot .35s ease-out}
@keyframes pc-boot{from{opacity:0;filter:brightness(3)}}
.pc-bar{display:flex;align-items:center;gap:.4em;padding:.25em .35em .25em .5em;background:rgb(11 20 36/.55);font-size:.85em}
#ui .pc-back{display:flex;align-items:center;gap:.2em;padding:.1em .5em .1em .3em;border-radius:1em;background:rgb(255 255 255/.16);font-weight:700}
#ui .pc-off{padding:.15em .25em;border-radius:.35em;background:#c91a09;display:grid;place-items:center}
.pc-body{flex:1;min-height:0;display:flex;padding:.6em}
.pc-home{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:.8em}
.pc-welcome{font-size:1.25em;font-weight:800}
.pc-icons{display:flex;gap:1.1em;flex-wrap:wrap;justify-content:center}
#ui .pc-icon{display:flex;flex-direction:column;align-items:center;gap:.3em;font-weight:700;font-size:.85em;text-shadow:0 1px 2px rgb(0 0 0/.5);animation:pc-pop .3s ease-out both;transition:transform .12s}
#ui .pc-icon:hover{transform:scale(1.06)}
.pc-tile{position:relative;width:3.4em;height:3.4em;border-radius:.9em;display:grid;place-items:center;box-shadow:0 .2em .6em rgb(0 0 0/.3)}
.pc-badge{position:absolute;right:-.5em;top:-.5em;background:#c91a09;border:2px solid #fff;border-radius:1em;padding:0 .35em;font-style:normal;font-size:.8em;font-weight:800}
@keyframes pc-pop{from{opacity:0;transform:translateY(1em)}}
.pc-sheet{flex:1;min-height:0;overflow:hidden;display:flex;flex-direction:column;gap:.45em;padding:.8em;background:rgb(250 250 246/.97);color:#1b2a34;border-radius:.6em;animation:pc-pop .25s ease-out both;font-size:.85em}
.pc-msg{display:flex;gap:.5em;padding:.35em .6em;border-radius:.5em;background:#eef1f4;animation:pc-slide .25s ease-out both}
.pc-msg.new{background:#e3efff}.pc-msg b{color:#1655b0;white-space:nowrap}
@keyframes pc-slide{from{opacity:0;transform:translateX(2em)}}
.pc-stats{display:grid;grid-template-columns:1fr 1fr;gap:.35em .8em}.pc-stats.one{grid-template-columns:1fr}
.pc-stat{display:flex;justify-content:space-between;align-items:center;padding:.3em .6em .3em .5em;border-radius:.4em;background:#eef1f4}
.pc-stat span{color:#5c6670;font-weight:600}.pc-stat b{font-size:1.2em}
.pc-coll{font-size:1.3em;font-weight:700}.pc-coll b{color:#1655b0}
.pc-prog{height:.8em;border-radius:1em;background:#e3e7eb;overflow:hidden}.pc-prog div{height:100%;border-radius:1em;background:linear-gradient(90deg,#3d9bff,#27b05a);animation:pc-grow .6s ease-out}
@keyframes pc-grow{from{width:0}}
.pc-fact{display:flex;gap:.5em;align-items:center;padding:.5em;border-radius:.5em;background:#fff8e1;border:1px solid #f2d27a}
.pc-stks{display:grid;gap:.4em}
.pc-stk{display:flex;flex-direction:column;align-items:center;gap:2px;padding:.35em;border-radius:.5em;background:#eef1f4;border:2px solid #d5dae0;text-align:center;animation:pc-pop .3s ease-out both;transform:rotate(var(--r))}
.pc-stk.got{background:color-mix(in srgb,var(--c),#fff 85%);border-color:var(--c)}
.pc-stk b{font-size:.75em}.pc-stk small{font-size:.62em;color:#5c6670;line-height:1.1}
.lab{position:absolute;inset:0;display:flex;flex-direction:column;overflow:hidden;background:#03130c;color:#5dff9d;font-family:ui-monospace,Consolas,monospace;border-radius:.3em;animation:pc-boot .35s ease-out}
.lab-bar{display:flex;align-items:center;gap:.45em;padding:.3em .4em .3em .7em;background:#0b2a1b;color:#9dffc6;font-weight:700;font-size:.95em}
.lab-bar i{width:.7em;height:.7em;border-radius:1em;display:inline-block}
#ui .lab-off{padding:.2em .35em;border-radius:.35em;background:#c91a09;display:grid;place-items:center}
.lab-main{flex:1;min-height:0;display:flex;gap:.5em;padding:.4em .6em .5em .7em}
.lab-main.narrow{flex-direction:column}
.lab-space{width:64%;flex:none}.lab-main.narrow .lab-space{width:auto;flex:1}
.lab-side{flex:1;min-height:0;display:flex;flex-direction:column;gap:.5em}.lab-main.narrow .lab-side{flex-direction:row;flex:none;height:30%}
.lab-panel{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;padding:.25em .4em .3em;background:rgb(10 50 30/.45);border:1px solid #1f6f4a;border-radius:.3em}
.lab-panel svg{flex:1;min-height:0;width:100%}
.lab-cap{font-size:.7em;font-weight:700;color:#9dffc6}
.lab-livedot{color:#ff5f56;animation:tv-blink 1.2s steps(2) infinite}
.hx{animation:hx 3.93s ease-in-out infinite alternate}.hx.b{animation-name:hxb}
@keyframes hx{from{transform:translateX(-30px)}to{transform:translateX(30px)}}
@keyframes hxb{from{transform:translateX(30px)}to{transform:translateX(-30px)}}
.lab-scan{position:absolute;inset:0;pointer-events:none;background:repeating-linear-gradient(0deg,rgb(0 0 0/.25) 0 1px,transparent 1px 3px),radial-gradient(circle,transparent 55%,rgb(0 0 0/.65))}
.lab-con{position:absolute;left:.7em;top:2.2em;bottom:.6em;width:calc(64% - .9em);display:flex;flex-direction:column;justify-content:flex-end;overflow:hidden;font-family:ui-monospace,Consolas,monospace;font-size:1em;line-height:1.3;white-space:pre;text-shadow:0 0 .5em rgb(93 255 157/.45)}
.lab-con.narrow{width:auto;right:.6em;bottom:calc(30% + 1em)}
.lab-cur{display:inline-block;width:.55em;height:1.05em;vertical-align:text-bottom;background:#5dff9d;animation:tv-blink 1s steps(2) infinite}
</style>";
