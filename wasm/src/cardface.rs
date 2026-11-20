//! A card's printed face (Kaigen `cu_card` / the three.js `Card.svelte`): white edge (or the upgrade stage's
//! metal), the ability's colour, the art window with the beast's pixel art, the name banner, the cost and power
//! badges, the ability symbol (small) or the kind and the text (big), the foil sheen; and the card back (red
//! lattice, gold frame, BRICK BEASTS medallion). Drawn on one 2D canvas (`FACE_W` x `FACE_H`, 1 : 1.4), then
//!
//! * `CardFaces::texture(ctx, face)` -> an sRGB `TextureId` with mips (synchronous: `getImageData`) for a 3D
//!   card mesh (`Material { texture: Some(tex), ..Material::color(0xffffff).cutout(0.5) }` on a quad of aspect
//!   1 : 1.4; the corners are transparent), or
//! * `CardFaces::url(face)` -> a blob URL of a PNG for the DOM (`<img>`; asynchronous: `None` until the PNG is
//!   encoded, a frame or two later; `ready()` counts the finished ones so a screen knows when to redraw).
//!
//! Both are cached per `Face` (GPU textures are never freed: keep the variants few, e.g. draw a changed
//! battle power as a separate badge instead of a face per power value).

use crate::cards::{self, Kind};
use engine::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{CanvasGradient, CanvasRenderingContext2d as C2d, HtmlCanvasElement};

/// Face size in pixels (1 : 1.4, as the cards). `em` = a tenth of the width (the Kaigen card's unit).
pub const FACE_W: u32 = 300;
pub const FACE_H: u32 = 420;
const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif";

/// What a face shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Face {
    pub card: usize,
    /// upgrade stage: 0 basic, 1 bronze, 2 silver + glitter, 3 gold + holo, 4 Voxelmon
    pub level: u8,
    /// the power shown (green when above the card's, red below)
    pub power: i32,
    /// not collected yet: a dark silhouette, "???"
    pub unknown: bool,
    /// the detail card: art on top, the kind and the ability text below (else the symbol only)
    pub big: bool,
    /// cards lying on the table do not show their cost
    pub hide_cost: bool,
    /// the card back (all other fields ignored)
    pub back: bool,
}

impl Face {
    /// A card as printed (its own power), small.
    pub fn new(card: usize, level: u8) -> Self {
        Face { card, level, power: cards::card(card).power, unknown: false, big: false, hide_cost: false, back: false }
    }
    pub fn big(mut self) -> Self {
        self.big = true;
        self
    }
    pub fn unknown(mut self) -> Self {
        self.unknown = true;
        self.level = 0;
        self
    }
    pub fn power(mut self, p: i32) -> Self {
        self.power = p;
        self
    }
    pub fn back() -> Self {
        Face { card: 0, level: 0, power: 0, unknown: false, big: false, hide_cost: false, back: true }
    }
    fn key(&self) -> String {
        if self.back {
            return "back".into();
        }
        format!("{}-{}-{}{}{}{}", cards::card(self.card).id, self.level, self.power, if self.unknown { "-u" } else { "" }, if self.big { "-big" } else { "" }, if self.hide_cost { "-nc" } else { "" })
    }
}

fn hex(c: u32) -> String {
    format!("#{:06x}", c & 0xffffff)
}
fn rgba(c: u32, a: f32) -> String {
    format!("rgba({},{},{},{:.3})", (c >> 16) & 255, (c >> 8) & 255, c & 255, a.clamp(0.0, 1.0))
}
/// css color-mix(in srgb, a (1-t), b t)
fn mix(a: u32, b: u32, t: f32) -> u32 {
    let ch = |s: u32| {
        let (x, y) = (((a >> s) & 255) as f32, ((b >> s) & 255) as f32);
        ((x + (y - x) * t).round() as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}
fn light(c: u32) -> bool {
    (0.299 * ((c >> 16) & 255) as f32 + 0.587 * ((c >> 8) & 255) as f32 + 0.114 * (c & 255) as f32) / 255.0 > 0.62
}
fn ink(bg: u32) -> u32 {
    if light(bg) {
        0x1b2a34
    } else {
        0xffffff
    }
}

/// The face renderer and its caches (create once, keep in `Data`).
#[derive(Default)]
pub struct CardFaces {
    canvas: Option<(HtmlCanvasElement, C2d, HtmlCanvasElement, C2d)>,
    textures: HashMap<Face, TextureId>,
    urls: Rc<RefCell<HashMap<Face, String>>>,
    requested: HashSet<Face>,
    /// faces drawn (tests)
    pub drawn: u32,
}

impl CardFaces {
    fn canvas(&mut self) -> Option<&(HtmlCanvasElement, C2d, HtmlCanvasElement, C2d)> {
        if self.canvas.is_none() {
            let doc = web_sys::window()?.document()?;
            let mk = |w: u32, h: u32| -> Option<(HtmlCanvasElement, C2d)> {
                let c: HtmlCanvasElement = doc.create_element("canvas").ok()?.dyn_into().ok()?;
                c.set_width(w);
                c.set_height(h);
                let g: C2d = c.get_context("2d").ok()??.dyn_into().ok()?;
                Some((c, g))
            };
            let (c, g) = mk(FACE_W, FACE_H)?;
            let (a, ag) = mk(64, 64)?;
            self.canvas = Some((c, g, a, ag));
        }
        self.canvas.as_ref()
    }

    /// The face as an sRGB texture with mips (cached).
    pub fn texture(&mut self, ctx: &mut Ctx, f: Face) -> Option<TextureId> {
        if let Some(t) = self.textures.get(&f) {
            return Some(*t);
        }
        let px = self.rgba(f)?;
        match ctx.renderer.add_texture_rgba(FACE_W, FACE_H, &px, &format!("card:{}", f.key())) {
            Ok(t) => {
                self.textures.insert(f, t);
                Some(t)
            }
            Err(e) => {
                log_warn!("card face texture: {e}");
                None
            }
        }
    }

    /// The face's RGBA pixels (`FACE_W` x `FACE_H`, top row first, straight alpha).
    pub fn rgba(&mut self, f: Face) -> Option<Vec<u8>> {
        self.draw(f)?;
        let (_, g, _, _) = self.canvas.as_ref()?;
        Some(g.get_image_data(0, 0, FACE_W as i32, FACE_H as i32).ok()?.data().0)
    }

    /// A blob URL of the face as PNG for `<img>`; `None` until it is encoded (asked once, then cached).
    pub fn url(&mut self, f: Face) -> Option<String> {
        if let Some(u) = self.urls.borrow().get(&f) {
            return Some(u.clone());
        }
        if self.requested.insert(f) && self.draw(f).is_some() {
            let urls = self.urls.clone();
            let cb = wasm_bindgen::closure::Closure::once_into_js(move |blob: JsValue| {
                if let Ok(b) = blob.dyn_into::<web_sys::Blob>() {
                    if let Ok(u) = web_sys::Url::create_object_url_with_blob(&b) {
                        urls.borrow_mut().insert(f, u);
                    }
                }
            });
            // (toBlob copies the bitmap now: the canvas can be drawn over at once)
            if let Some((c, ..)) = &self.canvas {
                let _ = c.to_blob(cb.unchecked_ref());
            }
        }
        None
    }

    /// Faces whose URL is ready (a screen showing `<img>`s redraws when this grows).
    pub fn ready(&self) -> usize {
        self.urls.borrow().len()
    }
    pub fn textures(&self) -> usize {
        self.textures.len()
    }

    /// Draws a face onto the canvas.
    fn draw(&mut self, f: Face) -> Option<()> {
        let (_, g, ac, ag) = self.canvas()?.clone();
        self.drawn += 1;
        let (w, h) = (FACE_W as f64, FACE_H as f64);
        g.set_global_alpha(1.0);
        g.set_global_composite_operation("source-over").ok()?;
        g.clear_rect(0.0, 0.0, w, h);
        g.save();
        if f.back {
            draw_back(&g, w, h);
        } else {
            draw_front(&g, ac, ag, f, w, h);
        }
        g.restore();
        Some(())
    }
}

// ---------------------------------------------------------------- drawing helpers

fn rr(g: &C2d, x: f64, y: f64, w: f64, h: f64, r: [f64; 4]) {
    // r: top-left, top-right, bottom-right, bottom-left
    g.begin_path();
    g.move_to(x + r[0], y);
    let _ = g.arc_to(x + w, y, x + w, y + h, r[1]);
    let _ = g.arc_to(x + w, y + h, x, y + h, r[2]);
    let _ = g.arc_to(x, y + h, x, y, r[3]);
    let _ = g.arc_to(x, y, x + w, y, r[0]);
    g.close_path();
}
fn rr1(g: &C2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    rr(g, x, y, w, h, [r; 4]);
}

/// A css linear-gradient(deg, stops) over the box.
fn lin(g: &C2d, x: f64, y: f64, w: f64, h: f64, deg: f64, stops: &[(f32, String)]) -> CanvasGradient {
    let a = deg.to_radians();
    let (dx, dy) = (a.sin(), -a.cos());
    let len = (w * dx).abs() + (h * dy).abs();
    let (cx, cy) = (x + w * 0.5, y + h * 0.5);
    let gr = g.create_linear_gradient(cx - dx * len * 0.5, cy - dy * len * 0.5, cx + dx * len * 0.5, cy + dy * len * 0.5);
    for (t, c) in stops {
        let _ = gr.add_color_stop(t.clamp(0.0, 1.0), c);
    }
    gr
}

/// radial-gradient(circle at fx fy, a, b t1) for a box (farthest corner).
fn rad(g: &C2d, x: f64, y: f64, w: f64, h: f64, fx: f64, fy: f64, a: &str, b: &str, t1: f32) -> Option<CanvasGradient> {
    let (cx, cy) = (x + w * fx, y + h * fy);
    let r = ((w * fx.max(1.0 - fx)).powi(2) + (h * fy.max(1.0 - fy)).powi(2)).sqrt();
    let gr = g.create_radial_gradient(cx, cy, 0.0, cx, cy, r).ok()?;
    let _ = gr.add_color_stop(0.0, a);
    let _ = gr.add_color_stop(t1, b);
    Some(gr)
}

/// Bold centred text, shrunk to fit `max_w`.
fn text(g: &C2d, s: &str, x: f64, y: f64, size: f64, weight: u32, color: &str, max_w: f64) {
    let mut sz = size;
    loop {
        g.set_font(&format!("{weight} {sz:.1}px {FONT}"));
        let tw = g.measure_text(s).map(|m| m.width()).unwrap_or(0.0);
        if tw <= max_w || sz < size * 0.5 {
            break;
        }
        sz *= 0.94;
    }
    g.set_text_align("center");
    g.set_text_baseline("middle");
    g.set_fill_style_str(color);
    let _ = g.fill_text(s, x, y);
}

/// Words wrapped into lines no wider than `max_w`.
fn wrap(g: &C2d, s: &str, max_w: f64) -> Vec<String> {
    let mut lines = vec![];
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let t = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if g.measure_text(&t).map(|m| m.width()).unwrap_or(0.0) > max_w && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = t;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn shadow(g: &C2d, color: &str, blur: f64, oy: f64) {
    g.set_shadow_color(color);
    g.set_shadow_blur(blur);
    g.set_shadow_offset_x(0.0);
    g.set_shadow_offset_y(oy);
}
fn no_shadow(g: &C2d) {
    g.set_shadow_color("rgba(0,0,0,0)");
    g.set_shadow_blur(0.0);
    g.set_shadow_offset_y(0.0);
}

fn draw_front(g: &C2d, ac: HtmlCanvasElement, ag: C2d, f: Face, w: f64, h: f64) {
    let em = w / 10.0;
    let d = cards::card(f.card);
    let ki = cards::kind_info(d.kind);
    let (mut t, mut td) = (ki.color, ki.dark);
    if f.unknown {
        t = 0x7a7f86;
        td = 0x3c4046;
    }
    let lv = f.level;
    // ---- the frame: white printed edge, or the stage's metal ----
    let bw = em * match lv {
        0 => 0.16,
        1 => 0.22,
        2 => 0.24,
        _ => 0.26,
    };
    rr1(g, 0.0, 0.0, w, h, 0.6 * em);
    let metal: Option<[(f32, u32); 4]> = match lv {
        1 => Some([(0.0, 0xf3c58f), (0.3, 0xa8642a), (0.52, 0xffdcae), (1.0, 0x8a4e1c)]),
        2 => Some([(0.0, 0xffffff), (0.28, 0x9aa6b2), (0.5, 0xf4f8fb), (1.0, 0x7d8995)]),
        3.. => Some([(0.0, 0xfff3a8), (0.26, 0xd9a300), (0.48, 0xfffbe0), (1.0, 0xb07d00)]),
        _ => None,
    };
    match metal {
        Some(m) => g.set_fill_style_canvas_gradient(&lin(g, 0.0, 0.0, w, h, 135.0, &m.map(|(a, c)| (a, hex(c))))),
        None => g.set_fill_style_str("#ffffff"),
    }
    g.fill();
    // a Voxelmon card: a violet line inside the gold
    if lv >= 4 {
        rr1(g, bw * 0.45, bw * 0.45, w - bw * 0.9, h - bw * 0.9, 0.55 * em);
        g.set_stroke_style_str("#a05aff");
        g.set_line_width(bw * 0.35);
        g.stroke();
    }
    // ---- the colour of the ability, light at the top left ----
    let (ix, iy, iw, ih) = (bw, bw, w - 2.0 * bw, h - 2.0 * bw);
    rr1(g, ix, iy, iw, ih, 0.45 * em);
    g.set_fill_style_canvas_gradient(&lin(g, ix, iy, iw, ih, 150.0, &[(0.0, hex(mix(t, 0xffffff, 0.3))), (0.38, hex(t)), (1.0, hex(td))]));
    g.fill();
    // ---- the cream panel ----
    let p = 0.3 * em;
    let (cx, cy, cw, ch) = (ix + p, iy + p, iw - 2.0 * p, ih - 2.0 * p);
    rr1(g, cx, cy, cw, ch, 0.38 * em);
    g.set_fill_style_canvas_gradient(&lin(g, cx, cy, cw, ch, 180.0, &[(0.0, hex(0xfffdf7)), (1.0, hex(mix(0xfffaf0, t, 0.18)))]));
    g.fill();
    g.save();
    rr1(g, cx, cy, cw, ch, 0.38 * em);
    g.clip();
    // ---- the art window: the beast on a little scene in the ability's colour ----
    let art_h = if f.big { cw * 5.0 / 6.0 } else { ch - 0.95 * em - 1.95 * em };
    let sky = if f.unknown { 0x3a3e44 } else { mix(t, 0xffffff, 0.55) };
    let ground = if f.unknown { 0x2a2d32 } else { mix(t, td, 0.35) };
    shadow(g, "rgba(0,0,0,0.18)", 0.0, 0.12 * em);
    rr(g, cx, cy, cw, art_h, [0.3 * em, 0.3 * em, 0.15 * em, 0.15 * em]);
    g.set_fill_style_canvas_gradient(&lin(g, cx, cy, cw, art_h, 180.0, &[(0.0, hex(sky)), (0.62, hex(mix(sky, t, 0.6))), (0.66, hex(ground)), (1.0, hex(mix(ground, td, 0.5)))]));
    g.fill();
    no_shadow(g);
    // the beast: crisp pixel blocks, as big as fits (Kaigen proportions)
    let (pw, ph, px) = cards::picture(f.card);
    if pw > 0 {
        ac.set_width(pw);
        ac.set_height(ph);
        let mut data = px.to_vec();
        if f.unknown {
            for q in data.chunks_mut(4) {
                q[0] = 0;
                q[1] = 0;
                q[2] = 0;
            }
        }
        if let Ok(img) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data), pw, ph) {
            let _ = ag.put_image_data(&img, 0, 0);
        }
        let bx = cw * 0.86;
        let by = if f.big { art_h } else { (h - 2.0 * bw - 0.6 * em) * 0.62 } * 0.86;
        let s = (bx / pw as f64).min(by / ph as f64);
        let (dw, dh) = (pw as f64 * s, ph as f64 * s);
        // centred in the window's content box (padding top 0.6 em, bottom 0.7 em)
        let (top, bot) = (cy + 0.6 * em, cy + art_h - 0.7 * em);
        g.set_image_smoothing_enabled(false);
        shadow(g, "rgba(0,0,0,0.25)", 0.15 * em, 0.08 * em);
        let _ = g.draw_image_with_html_canvas_element_and_dw_and_dh(&ac, cx + (cw - dw) * 0.5, (top + bot - dh) * 0.5, dw, dh);
        no_shadow(g);
        g.set_image_smoothing_enabled(true);
    }
    // silver and up: glitter specks; gold: a rainbow holo band over the picture
    if lv >= 2 && !f.unknown {
        let n = if lv >= 3 { 9 } else { 6 };
        for i in 0..n {
            let hsh = (i as u32).wrapping_mul(2654435761).wrapping_add(d.id.len() as u32 * 97);
            let (sx, sy) = (cx + cw * (((hsh % 90) + 5) as f64 / 100.0), cy + art_h * ((((hsh >> 8) % 85) + 5) as f64 / 100.0));
            let r = em * (0.12 + ((hsh >> 4) % 6) as f64 * 0.04);
            g.begin_path();
            g.move_to(sx, sy - r * 2.2);
            g.line_to(sx + r * 0.5, sy - r * 0.5);
            g.line_to(sx + r * 2.2, sy);
            g.line_to(sx + r * 0.5, sy + r * 0.5);
            g.line_to(sx, sy + r * 2.2);
            g.line_to(sx - r * 0.5, sy + r * 0.5);
            g.line_to(sx - r * 2.2, sy);
            g.line_to(sx - r * 0.5, sy - r * 0.5);
            g.close_path();
            g.set_fill_style_str("rgba(255,255,255,0.85)");
            g.fill();
        }
    }
    if lv >= 3 && !f.unknown {
        let q = 0.45f32;
        rr(g, cx, cy, cw, art_h, [0.3 * em, 0.3 * em, 0.15 * em, 0.15 * em]);
        g.set_fill_style_canvas_gradient(&lin(
            g,
            cx,
            cy,
            cw,
            art_h,
            115.0,
            &[(q - 0.14, rgba(0xff6ad5, 0.0)), (q - 0.04, rgba(0xffd23d, 0.4)), (q + 0.05, rgba(0x3dbbff, 0.4)), (q + 0.15, rgba(0xb06aff, 0.0))],
        ));
        g.fill();
    }
    // ---- the name on a banner overlapping the bottom of the art window ----
    let (bnw, bnh) = (cw - 0.6 * em, 1.5 * em);
    let (bnx, bny) = (cx + (cw - bnw) * 0.5, cy + art_h + 0.16 * em - bnh * 0.5);
    shadow(g, "rgba(0,0,0,0.30)", 0.2 * em, 0.1 * em);
    rr1(g, bnx, bny, bnw, bnh, 0.35 * em);
    g.set_fill_style_canvas_gradient(&lin(g, bnx, bny, bnw, bnh, 180.0, &[(0.0, "#ffffff".into()), (1.0, "#f1ebdc".into())]));
    g.fill();
    no_shadow(g);
    g.set_stroke_style_str(&hex(mix(td, 0, 0.3)));
    g.set_line_width((0.07 * em).max(1.0));
    g.stroke();
    text(g, if f.unknown { "???" } else { d.name }, bnx + bnw * 0.5, bny + bnh * 0.53, 1.1 * em, 800, "#1b2a34", bnw - 0.5 * em);
    let below = cy + art_h + 0.16 * em + bnh * 0.5;
    if f.big {
        // the kind as a pill, then the text
        let label = if ki.glyph.is_empty() { ki.label.to_string() } else { format!("{}  {}", ki.glyph, ki.label) };
        g.set_font(&format!("700 {:.1}px {FONT}", 0.75 * em));
        let lw = g.measure_text(&label).map(|m| m.width()).unwrap_or(0.0) + 1.2 * em;
        let (py, phh) = (cy + art_h + 1.3 * em - 0.1 * em, 1.25 * em);
        rr1(g, cx + (cw - lw) * 0.5, py, lw, phh, phh * 0.5);
        g.set_fill_style_str(&hex(t));
        g.fill();
        text(g, &label, cx + cw * 0.5, py + phh * 0.53, 0.75 * em, 700, &hex(ink(t)), cw);
        if !f.unknown {
            let len = d.text.len();
            let fs = em * if len > 90 { 0.6 } else if len > 60 { 0.68 } else { 0.78 };
            g.set_font(&format!("500 {fs:.1}px {FONT}"));
            let lines = wrap(g, d.text, cw - 1.0 * em);
            g.set_text_align("center");
            g.set_text_baseline("top");
            g.set_fill_style_str("#333333");
            for (i, l) in lines.iter().enumerate() {
                let _ = g.fill_text(l, cx + cw * 0.5, py + phh + 0.3 * em + i as f64 * fs * 1.22);
            }
        }
    } else if d.kind != Kind::None && !f.unknown {
        // the ability symbol in a round badge
        let r = 0.825 * em;
        let (bx, by) = (cx + cw * 0.5, below + (0.95 - 0.91) * em + r + 0.05 * em);
        let by = by.min(cy + ch - 0.3 * em - r);
        shadow(g, "rgba(0,0,0,0.35)", 0.3 * em, 0.1 * em);
        g.begin_path();
        let _ = g.arc(bx, by, r, 0.0, std::f64::consts::TAU);
        g.set_fill_style_str(&hex(t));
        g.fill();
        no_shadow(g);
        g.set_stroke_style_str("rgba(255,255,255,0.85)");
        g.set_line_width((0.06 * em).max(1.0));
        g.stroke();
        text(g, ki.glyph, bx, by + 0.04 * em, 1.0 * em, 800, &hex(ink(t)), 2.0 * r);
    }
    // ---- cost (round, blue) top left, power (rounded square, orange / green raised / red lowered) top right ----
    let bd = 2.15 * em;
    let border = (0.09 * em).max(2.0);
    if !f.hide_cost {
        let (x, y) = (cx + 0.15 * em, cy + 0.15 * em);
        shadow(g, "rgba(0,0,0,0.40)", 0.25 * em, 0.12 * em);
        g.begin_path();
        let _ = g.arc(x + bd * 0.5, y + bd * 0.5, bd * 0.5 - border * 0.5, 0.0, std::f64::consts::TAU);
        if let Some(gr) = rad(g, x, y, bd, bd, 0.35, 0.3, "#3f86e0", "#0a3478", 0.75) {
            g.set_fill_style_canvas_gradient(&gr);
        }
        g.fill();
        no_shadow(g);
        g.set_stroke_style_str("#ffffff");
        g.set_line_width(border);
        g.stroke();
        shadow(g, "rgba(0,0,0,0.5)", 2.0, 1.0);
        text(g, &d.cost.to_string(), x + bd * 0.5, y + bd * 0.53, 1.15 * em, 800, "#ffffff", bd);
        no_shadow(g);
    }
    let diff = if f.unknown { 0 } else { f.power - d.power };
    let (x, y) = (cx + cw - 0.15 * em - bd, cy + 0.15 * em);
    let (a, b, t1) = match diff {
        1.. => ("#b6ff8f", "#1f9a12", 1.0),
        ..=-1 => ("#ff9a9a", "#c01010", 1.0),
        _ => ("#ffc08a", "#d9531b", 0.75),
    };
    shadow(g, "rgba(0,0,0,0.40)", 0.25 * em, 0.12 * em);
    rr1(g, x + border * 0.5, y + border * 0.5, bd - border, bd - border, 0.46 * em);
    if let Some(gr) = rad(g, x, y, bd, bd, 0.35, 0.3, a, b, t1) {
        g.set_fill_style_canvas_gradient(&gr);
    }
    g.fill();
    no_shadow(g);
    g.set_stroke_style_str("#ffffff");
    g.set_line_width(border);
    g.stroke();
    shadow(g, "rgba(0,0,0,0.5)", 2.0, 1.0);
    text(g, &(if f.unknown { d.power } else { f.power }).to_string(), x + bd * 0.5, y + bd * 0.53, 1.15 * em, 800, "#ffffff", bd);
    no_shadow(g);
    g.restore();
    // ---- a soft foil sheen over the whole card ----
    rr1(g, 0.0, 0.0, w, h, 0.6 * em);
    g.set_fill_style_canvas_gradient(&lin(g, 0.0, 0.0, w, h, 125.0, &[(0.0, rgba(0xffffff, 0.35)), (0.28, rgba(0xffffff, 0.0)), (0.62, rgba(0xffffff, 0.0)), (0.78, rgba(0xffffff, 0.18))]));
    g.fill();
}

/// The card back: cream edge, red lattice, a gold frame line and the BRICK BEASTS medallion.
fn draw_back(g: &C2d, w: f64, h: f64) {
    let em = w / 10.0;
    rr1(g, 0.0, 0.0, w, h, 0.6 * em);
    g.set_fill_style_str("#fbf6ea");
    g.fill();
    let bw = 0.32 * em;
    let (x, y, iw, ih) = (bw, bw, w - 2.0 * bw, h - 2.0 * bw);
    g.save();
    rr1(g, x, y, iw, ih, 0.38 * em);
    g.clip();
    g.set_fill_style_str("#a3140a");
    g.fill_rect(x, y, iw, ih);
    // the lattice: lines both ways every 1.3 em
    g.set_stroke_style_str("#e8604c");
    g.set_line_width(0.1 * em);
    let step = 1.3 * em;
    let mut k = -h;
    while k < w + h {
        g.begin_path();
        g.move_to(k, 0.0);
        g.line_to(k + h, h);
        g.stroke();
        g.begin_path();
        g.move_to(k, h);
        g.line_to(k + h, 0.0);
        g.stroke();
        k += step;
    }
    if let Some(gr) = rad(g, x, y, iw, ih, 0.5, 0.5, "rgba(255,255,255,0.18)", "rgba(255,255,255,0)", 0.62) {
        g.set_fill_style_canvas_gradient(&gr);
        g.fill_rect(x, y, iw, ih);
    }
    g.restore();
    // the gold frame line
    rr1(g, x + 0.3 * em, y + 0.3 * em, iw - 0.6 * em, ih - 0.6 * em, 0.28 * em);
    g.set_stroke_style_str("#f2cd37");
    g.set_line_width(0.06 * em);
    g.stroke();
    // the medallion: a little brick and the name
    let (mw, mh) = (5.2 * em, 4.4 * em);
    let (mx, my) = ((w - mw) * 0.5, (h - mh) * 0.5);
    shadow(g, "rgba(0,0,0,0.35)", 0.3 * em, 0.1 * em);
    rr1(g, mx, my, mw, mh, 0.9 * em);
    g.set_fill_style_str("#8a1006");
    g.fill();
    no_shadow(g);
    g.set_stroke_style_str("#f2cd37");
    g.set_line_width(0.12 * em);
    g.stroke();
    let (bw2, bh2) = (1.8 * em, 0.9 * em);
    let (bx, by) = ((w - bw2) * 0.5, my + 0.45 * em);
    rr1(g, bx, by, bw2, bh2, 0.15 * em);
    g.set_fill_style_str("#f2cd37");
    g.fill();
    for i in 0..2 {
        rr1(g, bx + 0.2 * em + i as f64 * 0.85 * em, by + 0.25 * em, 0.55 * em, 0.4 * em, 0.2 * em);
        g.set_fill_style_str("#ffe27a");
        g.fill();
    }
    text(g, "BRICK", w * 0.5, by + bh2 + 0.85 * em, 0.95 * em, 900, "#f2cd37", mw);
    text(g, "BEASTS", w * 0.5, by + bh2 + 1.85 * em, 0.95 * em, 900, "#f2cd37", mw);
}
