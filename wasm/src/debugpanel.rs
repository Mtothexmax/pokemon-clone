//! The debug panel (Kaigen `st_debug.c`): FPS and frame time (half-second readings), the picker (a click or tap on
//! the world names what is under it: the villagers, the ducks, the ball, golden studs, the chest, a model, a road
//! plate or the baseplate) and Copy buttons that put the picked point's or your coordinates (studs, "x, y, z") on the
//! clipboard. Kaigen toggled it with ` (backquote); the engine's debug mode owns that key here, so the panel shows
//! while the engine's debug mode is on (` / German ^), and phones turn it on in the menu ("Debug panel").

use crate::town::STUD;
use engine::prelude::*;
use serde_json::{json, Value};

#[derive(Default)]
pub struct DebugPanel {
    /// the menu's switch (phones)
    pub menu_on: bool,
    pub on: bool,
    pub name: Option<String>,
    /// the picked point (studs) and its distance (m)
    pub at: Vec3,
    pub dist: f32,
    copied_t: f32,
    copied: u8,
    fps_acc: f32,
    fps_frames: u32,
    pub fps: f32,
    pub fps_ms: f32,
    pub picks: u32,
    /// the last text put on the clipboard
    pub clip: Option<String>,
}

fn copy_text(text: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
}

pub fn fmt3(p: Vec3) -> String {
    format!("{:.1}, {:.1}, {:.1}", p.x, p.y, p.z)
}

impl DebugPanel {
    /// Every frame: on/off, the FPS reading, the Copy buttons (`dbg_copy:pick|me`).
    pub fn update(&mut self, ctx: &Ctx, actions: &[String], me: Vec3, dt: f32) {
        self.on = ctx.debug_mode.on() || self.menu_on;
        if !self.on {
            return;
        }
        self.copied_t = (self.copied_t - dt).max(0.0);
        self.fps_acc += dt;
        self.fps_frames += 1;
        if self.fps_acc >= 0.5 {
            self.fps = self.fps_frames as f32 / self.fps_acc;
            self.fps_ms = 1000.0 * self.fps_acc / self.fps_frames as f32;
            self.fps_acc = 0.0;
            self.fps_frames = 0;
        }
        for a in actions {
            let (which, p) = match a.as_str() {
                "dbg_copy:pick" if self.name.is_some() => (0, self.at),
                "dbg_copy:me" => (1, me),
                _ => continue,
            };
            let text = fmt3(p);
            copy_text(&text);
            log_info!("debug: copied {text}");
            self.clip = Some(text);
            self.copied = which;
            self.copied_t = 1.5;
        }
    }

    /// Names what a ray from the camera through `css` (CSS px) meets: the nearest moving thing along it
    /// (`spheres`: name, centre studs, radius studs), else what the physics ray hits (a model's name, a road
    /// plate, the baseplate). `hero` is skipped.
    pub fn pick(&mut self, ctx: &mut Ctx, css: Vec2, hero: Option<Entity>, spheres: &[(String, Vec3, f32)], on_road: impl Fn(f32, f32) -> bool) {
        let Some((o, d)) = ctx.screen_ray(css) else { return };
        let d = d.normalize_or_zero();
        let mut hit = None;
        let mut from = o;
        for _ in 0..3 {
            match ctx.physics.raycast(from, d, 200.0) {
                Some(h) if h.entity.is_some() && h.entity == hero => from = h.point + d * 0.05,
                Some(h) => {
                    hit = Some(h);
                    break;
                }
                None => break,
            }
        }
        let mut hit_p = hit.as_ref().map(|h| h.point);
        if hit_p.is_none() && d.y < -1e-3 {
            hit_p = Some(o + d * (-o.y / d.y)); // the baseplate
        }
        let hit_t = hit_p.map(|p| (p - o).length()).unwrap_or(1e9);
        let mut best: Option<(String, Vec3, f32)> = None;
        let mut best_t = hit_t + 0.3;
        for (name, c, r) in spheres {
            let c = *c * STUD;
            let oc = c - o;
            let t = oc.dot(d);
            if t > 0.0 && t < best_t && (oc - d * t).length() < r * STUD {
                best_t = t;
                best = Some((name.clone(), c, t));
            }
        }
        if best.is_none() {
            if let Some(p) = hit_p {
                let s = p / STUD;
                let info = ctx.inspect_at(css, false);
                // a model: its glTF file (the entity or a parent named *.glb)
                let mut model = None;
                if info["hit"].as_bool() == Some(true) {
                    let names = std::iter::once(info["name"].as_str().unwrap_or("")).chain(info["parents"].as_array().into_iter().flatten().filter_map(|v| v["name"].as_str()));
                    model = names.into_iter().find(|n| n.ends_with(".glb")).map(|n| n.trim_end_matches(".glb").replace('_', " "));
                }
                let name = model.unwrap_or_else(|| if on_road(s.x, s.z) { "Road plate".into() } else if s.y < 0.2 { "Baseplate".into() } else { "(unnamed brick)".into() });
                best = Some((name, p, hit_t));
            }
        }
        if let Some((name, p, t)) = best {
            self.at = p / STUD;
            self.dist = t;
            log_info!("debug pick: {name} at {}", fmt3(self.at));
            self.name = Some(name);
            self.picks += 1;
        }
    }

    pub fn draw(&self, ctx: &mut Ctx, me: Vec3, touch: bool) {
        if !self.on {
            ctx.ui.set("debug", "");
            return;
        }
        let btn = |which: u8, act: &str| {
            let done = self.copied_t > 0.0 && self.copied == which;
            format!("<button class='bh-dbgcopy{}' data-action='dbg_copy:{act}'>{}</button>", if done { " done" } else { "" }, if done { "Copied!" } else { "Copy" })
        };
        let col = if self.fps >= 50.0 { "#96ff96" } else if self.fps >= 28.0 { "#ffe678" } else { "#ff786e" };
        let pick = match &self.name {
            Some(n) => format!(
                "<div class='bh-dbgname'>{n}</div><div class='bh-dbgrow'><span>at x {:.1} / y {:.1} / z {:.1} studs &nbsp;({:.1} m away)</span>{}</div>",
                self.at.x, self.at.y, self.at.z, self.dist, btn(0, "pick")
            ),
            None => String::new(),
        };
        let html = format!(
            "<div class='bh-dbg'><div class='bh-dbghead'>DEBUG &nbsp;({}) &nbsp;{} on something</div><div class='bh-dbgfps' style='color:{col}'>{:.0} FPS &nbsp;({:.1} ms)</div>{pick}<div class='bh-dbgrow'><span>You: x {:.1} / y {:.1} / z {:.1} studs</span>{}</div></div>",
            if touch { "Menu to close" } else { "` to close" },
            if touch { "tap" } else { "click" },
            self.fps,
            self.fps_ms,
            me.x,
            me.y,
            me.z,
            btn(1, "me")
        );
        ctx.ui.set("debug", &html);
    }

    pub fn describe(&self) -> Value {
        json!({"on": self.on, "menu_on": self.menu_on, "name": self.name, "at": [self.at.x, self.at.y, self.at.z], "fps": (self.fps * 10.0).round() / 10.0, "ms": (self.fps_ms * 10.0).round() / 10.0, "picks": self.picks, "clip": self.clip})
    }
}

pub const STYLE: &str = "<style>
#ui [data-slot=debug]{position:fixed;right:16px;bottom:16px;z-index:25}
.bh-dbg{background:rgba(60,10,10,.86);color:#fff;border-radius:8px;padding:10px 14px;display:flex;flex-direction:column;gap:4px;max-width:calc(100vw - 32px);box-sizing:border-box}
.bh-dbghead{font-size:13px;color:#ffaa96}.bh-dbgfps{font-size:16px;font-weight:700}.bh-dbgname{font-size:18px}
.bh-dbgrow{display:flex;gap:10px;align-items:center;font-size:13px;color:#ffdcd2;flex-wrap:wrap}
#ui .bh-dbgcopy{margin:0;padding:4px 10px;border-radius:6px;background:#6e231e;color:#fff;font-size:13px;border:1px solid rgba(255,200,190,.6)}
#ui .bh-dbgcopy:hover{background:#963c32}#ui .bh-dbgcopy.done{background:#288c46}
</style>";
