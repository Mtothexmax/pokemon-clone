//! The card battle's 3D pieces (made once, kept in `Data` like every GPU resource): the card as a thin slab with
//! rounded corners (printed face on top, the card back and a cream edge below), the location tiles (a thicker slab:
//! the location's picture with its name and rule on one side, "?" on the other, flipped over when it is revealed),
//! the number badges (one atlas texture, one small quad mesh per number and style, so every badge batches), the
//! card's shards, and the materials of the table, the glows and the confetti.
//!
//! Pictures (the location photos `assets/zones/<id>.jpg`, the battle backdrops `assets/backgrounds/*.png`) load as
//! `<img>` and are drawn into a canvas once they arrived (`TableArt::poll`).

use crate::cardface::{lin, no_shadow, rad, rr1, shadow, text, wrap, FONT};
use crate::cards;
use engine::mesh::{MeshData, Vertex};
use engine::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d as C2d, HtmlCanvasElement, HtmlImageElement};

/// The printed cards' tint (linear): the sun (3.2) plus the sky light took a white card face past white (washed-out
/// art, a glow from the bloom); a little darker, the prints keep their colours.
const CARD_TINT: Vec4 = Vec4::new(0.72, 0.72, 0.72, 1.0);
/// A card you cannot pay for (in your hand): much darker, a little blue-grey.
const CARD_DIM: Vec4 = Vec4::new(0.17, 0.18, 0.21, 1.0);
/// The same unlit (`Art::set_flat`, phones): the lit look's brightness without the lighting.
const FLAT_TINT: Vec4 = Vec4::new(1.45, 1.45, 1.45, 1.0);
const FLAT_DIM: Vec4 = Vec4::new(0.4, 0.42, 0.48, 1.0);
const FLAT_FELT: Vec4 = Vec4::new(1.2, 1.2, 1.2, 1.0);

/// Card slab: width 1 (scale the entity by the card's width), height 1.4, this thick (local units).
pub const CARD_H: f32 = 1.4;
pub const CARD_T: f32 = 0.024;
const CARD_R: f32 = 0.06;
/// Location tile slab: width 1, depth 9 / 16 (the photos are 16 : 9).
pub const TILE_H: f32 = 0.5625;
pub const TILE_T: f32 = 0.05;
const TILE_R: f32 = 0.045;
/// Tile texture size.
const TW: u32 = 512;
const TH: u32 = 288;
/// Number atlas: 20 x 15 cells of 64 px; values -9..=40 in 6 styles.
const NUM_MIN: i32 = -9;
const NUM_N: i32 = 50;
const CELL: u32 = 64;
const COLS: u32 = 20;
/// Number styles: the card's power as printed (orange), raised (green), lowered (red); a location's total:
/// even (dark), you lead (gold), they lead (red).
pub const NUM_BASE: usize = 0;
pub const NUM_UP: usize = 1;
pub const NUM_DOWN: usize = 2;
pub const NUM_TOTAL: usize = 3;
pub const NUM_LEAD: usize = 4;
pub const NUM_FOE: usize = 5;
/// Shards of a destroyed card: 2 x 3 pieces (centre in card units).
pub const SHARDS: [(f32, f32); 6] = [(-0.25, 0.4667), (0.25, 0.4667), (-0.25, 0.0), (0.25, 0.0), (-0.25, -0.4667), (0.25, -0.4667)];
pub const CONFETTI: [u32; 7] = [0xc91a09, 0xf2cd37, 0x0055bf, 0x237841, 0xfe8a18, 0xf4f4f4, 0xff8ac0];

#[derive(Default)]
pub struct TableArt {
    ready: bool,
    pub card_front: Option<MeshId>,
    pub card_back: Option<MeshId>,
    pub tile_front: Option<MeshId>,
    pub tile_back: Option<MeshId>,
    pub round: Option<MeshId>,
    pub tile_round: Option<MeshId>,
    pub shards: Vec<MeshId>,
    pub back_mat: Option<MaterialId>,
    /// materials of textured faces (cards, tiles), per texture
    face_mats: HashMap<TextureId, MaterialId>,
    face_mats_dim: HashMap<TextureId, MaterialId>,
    /// the cards and the felt unlit (phones: the battle's pixels cost a texture fetch, no lighting or shadow lookups)
    pub flat: bool,
    num_tex: Option<TextureId>,
    pub num_mat: Option<MaterialId>,
    num_meshes: HashMap<(usize, i32), MeshId>,
    /// the location tiles: revealed per zone, unrevealed per turn it reveals on (1..3)
    pub zone_tex: Vec<Option<TextureId>>,
    pub hidden_tex: Vec<Option<TextureId>>,
    pub backdrop_tex: Vec<Option<TextureId>>,
    images: HashMap<String, HtmlImageElement>,
    canvas: Option<(HtmlCanvasElement, C2d)>,
    // materials
    pub wood: Option<MaterialId>,
    pub felt: Option<MaterialId>,
    pub rim: Option<MaterialId>,
    pub ghost: Option<MaterialId>,
    pub glow_gold: Option<MaterialId>,
    pub glow_white: Option<MaterialId>,
    pub glow_red: Option<MaterialId>,
    pub glow_green: Option<MaterialId>,
    pub dim: Option<MaterialId>,
    pub ball_hit: Option<MaterialId>,
    pub ball_buff: Option<MaterialId>,
    pub confetti: Vec<MaterialId>,
    pub pile_shadow: Option<MaterialId>,
}

/// A rounded rectangle (w x h, corner radius r), counter-clockwise from the top right corner.
fn outline(w: f32, h: f32, r: f32) -> Vec<Vec2> {
    let mut v = vec![];
    let seg = 6;
    let corners = [(w * 0.5 - r, h * 0.5 - r, 0.0f32), (-w * 0.5 + r, h * 0.5 - r, 90.0), (-w * 0.5 + r, -h * 0.5 + r, 180.0), (w * 0.5 - r, -h * 0.5 + r, 270.0)];
    for (cx, cy, a0) in corners {
        for k in 0..=seg {
            let a = (a0 + 90.0 * k as f32 / seg as f32).to_radians();
            v.push(Vec2::new(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    v
}

fn vert(p: Vec3, n: Vec3, uv: Vec2) -> Vertex {
    Vertex { pos: p.into(), normal: n.into(), uv: uv.into(), color: Vertex::WHITE }
}

/// A slab: (the front face at +z with the texture's uv, the back face at -z (u mirrored, or v for `flip_v`) and
/// the edge with the uv of `edge_uv`).
pub fn slab(w: f32, h: f32, r: f32, t: f32, flip_v: bool, edge_uv: Vec2) -> (MeshData, MeshData) {
    let o = outline(w, h, r);
    let n = o.len() as u32;
    let uv = |p: Vec2, back: bool| -> Vec2 {
        let (u, v) = (p.x / w + 0.5, 0.5 - p.y / h);
        match (back, flip_v) {
            (false, _) => Vec2::new(u, v),
            (true, false) => Vec2::new(1.0 - u, v),
            (true, true) => Vec2::new(u, 1.0 - v),
        }
    };
    let mut front = MeshData::default();
    front.vertices.push(vert(Vec3::new(0.0, 0.0, t * 0.5), Vec3::Z, uv(Vec2::ZERO, false)));
    for p in &o {
        front.vertices.push(vert(p.extend(t * 0.5), Vec3::Z, uv(*p, false)));
    }
    for i in 0..n {
        front.indices.extend([0, 1 + i, 1 + (i + 1) % n]);
    }
    let mut back = MeshData::default();
    back.vertices.push(vert(Vec3::new(0.0, 0.0, -t * 0.5), -Vec3::Z, uv(Vec2::ZERO, true)));
    for p in &o {
        back.vertices.push(vert(p.extend(-t * 0.5), -Vec3::Z, uv(*p, true)));
    }
    for i in 0..n {
        back.indices.extend([0, 1 + (i + 1) % n, 1 + i]);
    }
    // the edge (outward normals, flat per segment)
    for i in 0..o.len() {
        let (a, b) = (o[i], o[(i + 1) % o.len()]);
        let d = b - a;
        let nrm = Vec3::new(d.y, -d.x, 0.0).normalize_or_zero();
        let base = back.vertices.len() as u32;
        back.vertices.push(vert(a.extend(t * 0.5), nrm, edge_uv));
        back.vertices.push(vert(a.extend(-t * 0.5), nrm, edge_uv));
        back.vertices.push(vert(b.extend(-t * 0.5), nrm, edge_uv));
        back.vertices.push(vert(b.extend(t * 0.5), nrm, edge_uv));
        back.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (front, back)
}

/// A flat quad (1 x 1, facing +z) with the uv rectangle (u0, v0)-(u1, v1).
fn quad_uv(w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32) -> MeshData {
    let n = Vec3::Z;
    MeshData {
        vertices: vec![
            vert(Vec3::new(-w * 0.5, h * 0.5, 0.0), n, Vec2::new(u0, v0)),
            vert(Vec3::new(w * 0.5, h * 0.5, 0.0), n, Vec2::new(u1, v0)),
            vert(Vec3::new(w * 0.5, -h * 0.5, 0.0), n, Vec2::new(u1, v1)),
            vert(Vec3::new(-w * 0.5, -h * 0.5, 0.0), n, Vec2::new(u0, v1)),
        ],
        indices: vec![0, 3, 2, 0, 2, 1],
    }
}

fn canvas(w: u32, h: u32) -> Option<(HtmlCanvasElement, C2d)> {
    let doc = web_sys::window()?.document()?;
    let c: HtmlCanvasElement = doc.create_element("canvas").ok()?.dyn_into().ok()?;
    c.set_width(w);
    c.set_height(h);
    let g: C2d = c.get_context("2d").ok()??.dyn_into().ok()?;
    Some((c, g))
}

fn pixels(g: &C2d, w: u32, h: u32) -> Option<Vec<u8>> {
    Some(g.get_image_data(0, 0, w as i32, h as i32).ok()?.data().0)
}

/// css object-fit: cover of an image into the box.
fn cover(g: &C2d, img: &HtmlImageElement, x: f64, y: f64, w: f64, h: f64) {
    let (iw, ih) = (img.natural_width() as f64, img.natural_height() as f64);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let s = (w / iw).max(h / ih);
    let (dw, dh) = (iw * s, ih * s);
    let _ = g.draw_image_with_html_image_element_and_dw_and_dh(img, x + (w - dw) * 0.5, y + (h - dh) * 0.5, dw, dh);
}

impl TableArt {
    /// Once: the meshes, the number atlas and the materials; starts loading the pictures.
    pub fn ensure(&mut self, ctx: &mut Ctx) {
        if self.ready {
            return;
        }
        self.ready = true;
        let r = &mut ctx.renderer;
        let (f, b) = slab(1.0, CARD_H, CARD_R, CARD_T, false, Vec2::new(0.5, 0.004));
        self.card_front = Some(r.add_mesh("card front", &f));
        self.card_back = Some(r.add_mesh("card back", &b));
        let (f, b) = slab(1.0, TILE_H, TILE_R, TILE_T, true, Vec2::new(0.5, 0.004));
        self.tile_front = Some(r.add_mesh("tile front", &f));
        self.tile_back = Some(r.add_mesh("tile back", &b));
        let (f, _) = slab(1.0, CARD_H, CARD_R * 1.6, 0.0, false, Vec2::ZERO);
        self.round = Some(r.add_mesh("card round", &f));
        let (f, _) = slab(1.0, TILE_H, TILE_R * 1.4, 0.0, false, Vec2::ZERO);
        self.tile_round = Some(r.add_mesh("tile round", &f));
        for (cx, cy) in SHARDS {
            let (w, h) = (0.5, CARD_H / 3.0);
            let (u0, v0) = (cx - w * 0.5 + 0.5, 0.5 - (cy + h * 0.5) / CARD_H);
            let md = quad_uv(w, h, u0, v0, u0 + w, v0 + h / CARD_H);
            self.shards.push(r.add_mesh("card shard", &md));
        }
        // materials
        let wood = ctx.generate_texture(&TexGen::new(TexPreset::Wood).size(512).seed(4).scale(3.0).colors(0x4a2a14, 0x7a4e2c));
        let felt = ctx.generate_texture(&TexGen::new(TexPreset::Noise).size(256).seed(9).scale(6.0).colors(0x1d4a30, 0x2c6a45));
        let r = &mut ctx.renderer;
        self.wood = Some(r.add_material(Material { name: "table wood".into(), texture: wood, ..Material::color(0xffffff).rough(0.55) }));
        self.felt = Some(r.add_material(Material { texture: felt, ..Material::color(0xffffff).rough(0.95) }));
        self.rim = Some(r.add_material(Material::color(0xf2cd37).rough(0.35).metal(0.6)));
        self.ghost = Some(r.add_material(Material { unlit: true, ..Material::color(0xd8ffe0).transparent(0.07) }));
        let glow = |r: &mut engine::renderer::Renderer, c: u32, e: Vec3| r.add_material(Material { unlit: true, emissive: e, ..Material::color(c).transparent(0.85) });
        self.glow_gold = Some(glow(r, 0xffd23d, Vec3::new(2.2, 1.6, 0.3)));
        self.glow_white = Some(glow(r, 0xffffff, Vec3::splat(1.8)));
        self.glow_red = Some(glow(r, 0xff5a3d, Vec3::new(2.2, 0.5, 0.25)));
        self.glow_green = Some(glow(r, 0x7dff6a, Vec3::new(0.6, 2.2, 0.5)));
        self.dim = Some(r.add_material(Material { unlit: true, ..Material::color(0x05080c).transparent(0.86) }));
        self.ball_hit = Some(r.add_material(Material { unlit: true, emissive: Vec3::new(4.0, 1.6, 0.4), ..Material::color(0xffb020) }));
        self.ball_buff = Some(r.add_material(Material { unlit: true, emissive: Vec3::new(1.0, 4.0, 0.8), ..Material::color(0x6dff4a) }));
        self.confetti = CONFETTI.iter().map(|&c| r.add_material(Material::color(c).rough(0.3))).collect();
        self.pile_shadow = Some(r.add_material(Material { unlit: true, ..Material::color(0x000000).transparent(0.35) }));
        self.zone_tex = vec![None; cards::ZONE_COUNT];
        self.hidden_tex = vec![None; 3];
        self.backdrop_tex = vec![None; 3];
        self.canvas = canvas(TW, TH);
        self.build_numbers(ctx);
        for t in 0..3 {
            self.hidden_tex[t] = self.draw_tile(ctx, None, t + 1);
        }
        for z in 0..cards::ZONE_COUNT {
            self.request(&cards::zone_picture(z));
        }
        for b in BACKDROPS {
            self.request(b);
        }
    }

    fn request(&mut self, url: &str) {
        if self.images.contains_key(url) {
            return;
        }
        if let Ok(img) = HtmlImageElement::new() {
            img.set_src(url);
            self.images.insert(url.to_string(), img);
        }
    }

    fn image(&self, url: &str) -> Option<&HtmlImageElement> {
        self.images.get(url).filter(|i| i.complete() && i.natural_width() > 0)
    }

    /// Every frame while a battle shows: turns arrived pictures into textures. Returns true if one changed.
    pub fn poll(&mut self, ctx: &mut Ctx) -> bool {
        let mut changed = false;
        for z in 0..cards::ZONE_COUNT {
            if self.zone_tex[z].is_none() && self.image(&cards::zone_picture(z)).is_some() {
                self.zone_tex[z] = self.draw_tile(ctx, Some(z), 0);
                changed |= self.zone_tex[z].is_some();
            }
        }
        for (i, b) in BACKDROPS.iter().enumerate() {
            if self.backdrop_tex[i].is_none() {
                if let Some(img) = self.image(b).cloned() {
                    let (w, h) = (img.natural_width(), img.natural_height());
                    if let Some((_, g)) = canvas(w, h) {
                        let _ = g.draw_image_with_html_image_element(&img, 0.0, 0.0);
                        if let Some(px) = pixels(&g, w, h) {
                            self.backdrop_tex[i] = ctx.renderer.add_texture_rgba(w, h, &px, &format!("backdrop {b}")).ok();
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    pub fn pictures_loaded(&self) -> usize {
        self.zone_tex.iter().filter(|t| t.is_some()).count() + self.backdrop_tex.iter().filter(|t| t.is_some()).count()
    }

    /// The material of a textured face (cards, tiles; opaque, double-sided for the shards).
    /// A card face dimmed (in your hand, too expensive for the energy left).
    pub fn face_mat_dim(&mut self, ctx: &mut Ctx, tex: TextureId) -> MaterialId {
        let (unlit, base_color) = if self.flat { (true, FLAT_DIM) } else { (false, CARD_DIM) };
        *self.face_mats_dim.entry(tex).or_insert_with(|| ctx.renderer.add_material(Material { name: "card face dim".into(), texture: Some(tex), double_sided: true, base_color, unlit, ..Material::color(0xffffff).rough(0.6) }))
    }

    pub fn face_mat(&mut self, ctx: &mut Ctx, tex: TextureId) -> MaterialId {
        let (unlit, base_color) = if self.flat { (true, FLAT_TINT) } else { (false, CARD_TINT) };
        *self.face_mats.entry(tex).or_insert_with(|| ctx.renderer.add_material(Material { name: "card face".into(), texture: Some(tex), double_sided: true, base_color, unlit, ..Material::color(0xffffff).rough(0.6) }))
    }

    /// The cards (faces, backs, tiles) and the felt unlit or lit again: every material made so far, and the ones
    /// made from now on.
    pub fn set_flat(&mut self, ctx: &mut Ctx, on: bool) {
        self.flat = on;
        let mut set = |id: MaterialId, tint: Vec4, flat: Vec4| {
            if let Some(m) = ctx.renderer.material_mut(id) {
                m.unlit = on;
                m.base_color = if on { flat } else { tint };
            }
        };
        for &id in self.face_mats.values().chain(self.back_mat.iter()) {
            set(id, CARD_TINT, FLAT_TINT);
        }
        for &id in self.face_mats_dim.values() {
            set(id, CARD_DIM, FLAT_DIM);
        }
        if let Some(id) = self.felt {
            set(id, Vec4::ONE, FLAT_FELT);
        }
    }

    pub fn back_mat(&mut self, ctx: &mut Ctx, faces: &mut crate::cardface::CardFaces) -> Option<MaterialId> {
        if self.back_mat.is_none() {
            let t = faces.texture(ctx, crate::cardface::Face::back())?;
            let (unlit, base_color) = if self.flat { (true, FLAT_TINT) } else { (false, CARD_TINT) };
            self.back_mat = Some(ctx.renderer.add_material(Material { name: "card back".into(), texture: Some(t), base_color, unlit, ..Material::color(0xffffff).rough(0.6) }));
        }
        self.back_mat
    }

    /// A location tile's side: the picture with its name and rule (`zone` Some), or the grey "?" of a location still
    /// hidden (reveals on turn `turn`).
    fn draw_tile(&mut self, ctx: &mut Ctx, zone: Option<usize>, turn: usize) -> Option<TextureId> {
        let img = zone.and_then(|z| self.image(&cards::zone_picture(z)).cloned());
        let (_, g) = self.canvas.clone()?;
        let (w, h) = (TW as f64, TH as f64);
        let em = w / 20.0;
        g.clear_rect(0.0, 0.0, w, h);
        g.save();
        rr1(&g, 0.0, 0.0, w, h, TILE_R as f64 * w);
        g.clip();
        match (zone, &img) {
            (Some(z), Some(img)) => {
                let zd = &cards::ZONES[z];
                g.set_fill_style_str(&crate::cardface::hex(zd.color));
                g.fill_rect(0.0, 0.0, w, h);
                cover(&g, img, 0.0, 0.0, w, h);
                g.set_fill_style_canvas_gradient(&lin(&g, 0.0, 0.0, w, h, 0.0, &[(0.0, "rgba(0,0,0,0.78)".into()), (0.62, "rgba(0,0,0,0.12)".into()), (1.0, "rgba(0,0,0,0.0)".into())]));
                g.fill_rect(0.0, 0.0, w, h);
                shadow(&g, "rgba(0,0,0,0.9)", 6.0, 2.0);
                g.set_font(&format!("600 {:.1}px {FONT}", 1.25 * em));
                let n = wrap(&g, zd.text, w - 2.4 * em).len().min(2);
                text(&g, zd.name, w * 0.5, h - 1.6 * em - n as f64 * 1.45 * em, 2.1 * em, 800, "#ffffff", w - 2.0 * em);
                g.set_font(&format!("600 {:.1}px {FONT}", 1.25 * em));
                let lines = wrap(&g, zd.text, w - 2.4 * em);
                let n = lines.len().min(2);
                for (i, l) in lines.iter().take(2).enumerate() {
                    text(&g, l, w * 0.5, h - 1.25 * em - (n - 1 - i) as f64 * 1.45 * em, 1.25 * em, 600, "rgba(255,255,255,0.95)", w - 1.6 * em);
                }
                no_shadow(&g);
                // the location's colour as a frame
                rr1(&g, 0.0, 0.0, w, h, TILE_R as f64 * w);
                g.set_stroke_style_str(&crate::cardface::hex(zd.color));
                g.set_line_width(0.45 * em);
                g.stroke();
            }
            (Some(_), None) => {
                g.restore();
                return None;
            }
            (None, _) => {
                if let Some(gr) = rad(&g, 0.0, 0.0, w, h, 0.5, 0.35, "#6d7178", "#3d4046", 1.0) {
                    g.set_fill_style_canvas_gradient(&gr);
                }
                g.fill_rect(0.0, 0.0, w, h);
                g.set_fill_style_str("rgba(255,255,255,0.16)");
                let step = 1.2 * em;
                let mut y = step * 0.5;
                while y < h {
                    let mut x = step * 0.5;
                    while x < w {
                        g.begin_path();
                        let _ = g.arc(x, y, 0.18 * em, 0.0, std::f64::consts::TAU);
                        g.fill();
                        x += step;
                    }
                    y += step;
                }
                shadow(&g, "rgba(0,0,0,0.7)", 6.0, 2.0);
                text(&g, "?", w * 0.5, h * 0.34, 5.5 * em, 900, "#ffffff", w);
                text(&g, "Unrevealed", w * 0.5, h * 0.63, 1.9 * em, 800, "#ffffff", w - 2.0 * em);
                text(&g, &format!("Reveals on turn {turn}"), w * 0.5, h * 0.79, 1.3 * em, 600, "rgba(255,255,255,0.9)", w - 2.0 * em);
                no_shadow(&g);
                rr1(&g, 0.0, 0.0, w, h, TILE_R as f64 * w);
                g.set_stroke_style_str("rgba(0,0,0,0.45)");
                g.set_line_width(0.45 * em);
                g.stroke();
            }
        }
        g.restore();
        let px = pixels(&g, TW, TH)?;
        let label = match zone {
            Some(z) => format!("tile {}", cards::ZONES[z].id),
            None => format!("tile hidden {turn}"),
        };
        ctx.renderer.add_texture_rgba(TW, TH, &px, &label).ok()
    }

    /// The number atlas: every value in every style.
    fn build_numbers(&mut self, ctx: &mut Ctx) {
        let rows = (6 * NUM_N as u32).div_ceil(COLS);
        let (w, h) = (COLS * CELL, rows * CELL);
        let Some((_, g)) = canvas(w, h) else { return };
        let c = CELL as f64;
        for style in 0..6usize {
            for k in 0..NUM_N {
                let v = NUM_MIN + k;
                let idx = style as u32 * NUM_N as u32 + k as u32;
                let (x, y) = ((idx % COLS) as f64 * c, (idx / COLS) as f64 * c);
                let pad = c * 0.1;
                let (bx, by, bs) = (x + pad, y + pad, c - 2.0 * pad);
                let s = v.to_string();
                match style {
                    NUM_BASE | NUM_UP | NUM_DOWN => {
                        let (a, b) = match style {
                            NUM_UP => ("#b6ff8f", "#1f9a12"),
                            NUM_DOWN => ("#ff9a9a", "#c01010"),
                            _ => ("#ffc08a", "#d9531b"),
                        };
                        rr1(&g, bx, by, bs, bs, bs * 0.22);
                        if let Some(gr) = rad(&g, bx, by, bs, bs, 0.35, 0.3, a, b, 1.0) {
                            g.set_fill_style_canvas_gradient(&gr);
                        }
                        g.fill();
                        g.set_stroke_style_str("#ffffff");
                        g.set_line_width(c * 0.06);
                        g.stroke();
                        shadow(&g, "rgba(0,0,0,0.5)", 2.0, 1.0);
                        text(&g, &s, bx + bs * 0.5, by + bs * 0.54, bs * 0.58, 800, "#ffffff", bs * 0.9);
                        no_shadow(&g);
                    }
                    _ => {
                        let (bg, ink, glow) = match style {
                            NUM_LEAD => ("#f0a020", "#2a1a00", Some("#ffd23d")),
                            NUM_FOE => ("#d9361b", "#ffffff", Some("#ff5a3d")),
                            _ => ("#1b2a34", "#ffffff", None),
                        };
                        if let Some(gl) = glow {
                            shadow(&g, gl, c * 0.12, 0.0);
                        }
                        g.begin_path();
                        let _ = g.arc(x + c * 0.5, y + c * 0.5, bs * 0.45, 0.0, std::f64::consts::TAU);
                        g.set_fill_style_str(bg);
                        g.fill();
                        no_shadow(&g);
                        g.set_stroke_style_str("rgba(255,255,255,0.7)");
                        g.set_line_width(c * 0.05);
                        g.stroke();
                        text(&g, &s, x + c * 0.5, y + c * 0.53, bs * 0.5, 800, ink, bs * 0.8);
                    }
                }
            }
        }
        let Some(px) = pixels(&g, w, h) else { return };
        self.num_tex = ctx.renderer.add_texture_rgba(w, h, &px, "battle numbers").ok();
        self.num_mat = self.num_tex.map(|t| ctx.renderer.add_material(Material { texture: Some(t), unlit: true, ..Material::color(0xffffff).cutout(0.5) }));
    }

    /// The quad (1 x 1, facing +z) showing number `v` in `style`.
    pub fn number(&mut self, ctx: &mut Ctx, style: usize, v: i32) -> Option<MeshId> {
        let v = v.clamp(NUM_MIN, NUM_MIN + NUM_N - 1);
        if let Some(m) = self.num_meshes.get(&(style, v)) {
            return Some(*m);
        }
        self.num_tex?;
        let rows = (6 * NUM_N as u32).div_ceil(COLS);
        let (w, h) = ((COLS * CELL) as f32, (rows * CELL) as f32);
        let idx = style as u32 * NUM_N as u32 + (v - NUM_MIN) as u32;
        let (x, y) = ((idx % COLS * CELL) as f32, (idx / COLS * CELL) as f32);
        let e = 1.0;
        let md = quad_uv(1.0, 1.0, (x + e) / w, (y + e) / h, (x + CELL as f32 - e) / w, (y + CELL as f32 - e) / h);
        let m = ctx.renderer.add_mesh("battle number", &md);
        self.num_meshes.insert((style, v), m);
        Some(m)
    }
}

/// The battle backdrops: the woods (wild, Pip), Dex's cave, Lumi's lab.
pub const BACKDROPS: [&str; 3] = ["assets/backgrounds/forest.png", "assets/backgrounds/cave.png", "assets/backgrounds/lab.png"];
