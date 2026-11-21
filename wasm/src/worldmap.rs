//! The world map v1 (OPEN_WORLD.md section 5): painted once from the island's lattice on a 2D canvas (biome tint,
//! hill-shade, an inked coast, the roads; the strip of Brickhollow, Route 1 and Stonecrest drawn in) and shown as a
//! parchment over the game. Place names, the "?" on undiscovered pads and your arrow are HTML on top of the picture
//! (they change; the picture never does). M, the ☰ menu's Map row: open; M / Esc / a tap: close.

use crate::island::{self, Island, PADS, SOUTH_Z};
use crate::town::{self, Rect};
use engine::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d as C2d, HtmlCanvasElement};

/// The map's extent (studs): the whole lattice plus the strip south of it.
pub const MX0: f32 = island::X0;
pub const MX1: f32 = island::X1;
pub const MZ0: f32 = island::Z0;
pub const MZ1: f32 = 48.0;
/// Studs per pixel of the picture.
const SCALE: f32 = 2.0;

pub const STYLE: &str = "<style>
#ui [data-slot=map],#ui [data-slot=map_me]{position:fixed;inset:0;display:flex;align-items:center;justify-content:center;z-index:22}
#ui [data-slot=map]:empty,#ui [data-slot=map_me]:empty{display:none}
#ui [data-slot=map_me]{pointer-events:none;z-index:23}
.bh-mapshade{position:fixed;inset:0;background:rgba(10,14,22,.55);pointer-events:auto}
.bh-mapbox{position:relative;height:min(86vh,calc(94vw * 793 / 704));aspect-ratio:704/793;border-radius:14px;box-shadow:0 10px 30px rgba(0,0,0,.45);border:4px solid #6b4a2b;box-sizing:border-box;overflow:hidden}
.bh-mapbox img{display:block;width:100%;height:100%;image-rendering:auto}
.bh-maptitle{position:absolute;left:12px;top:10px;font:800 18px/1.2 system-ui;color:#4a321c;background:rgba(239,227,194,.85);border-radius:8px;padding:4px 10px}
.bh-mapclose{position:absolute;right:12px;top:10px;font:600 12px/1.2 system-ui;color:#4a321c;background:rgba(239,227,194,.85);border-radius:8px;padding:5px 9px}
.bh-mapname{position:absolute;transform:translate(-50%,-50%);font:700 13px/1.1 system-ui;color:#3a2814;text-shadow:0 0 3px #efe3c2,0 0 3px #efe3c2;white-space:nowrap}
.bh-mapname.small{font-size:11px;font-weight:600}
.bh-mapq{position:absolute;transform:translate(-50%,-50%);width:24px;height:24px;border-radius:12px;background:#f2cd37;border:2px solid #3a2814;font:900 15px/20px system-ui;text-align:center;color:#3a2814}
.bh-mapme{position:absolute;width:0;height:0;border-left:8px solid transparent;border-right:8px solid transparent;border-bottom:18px solid #c91a09;filter:drop-shadow(0 0 2px #fff) drop-shadow(0 1px 2px rgba(0,0,0,.6));transform-origin:50% 60%}
</style>";

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn rgb(hex: u32) -> [f32; 3] {
    [((hex >> 16) & 255) as f32, ((hex >> 8) & 255) as f32, (hex & 255) as f32]
}

/// Percent position of a point (studs) on the map.
pub fn pct(x: f32, z: f32) -> (f32, f32) {
    ((x - MX0) / (MX1 - MX0) * 100.0, (z - MZ0) / (MZ1 - MZ0) * 100.0)
}

/// The strip's land south of the island (studs): the village plate, the north plate, its grey roads, the trail.
fn strip_at(x: f32, z: f32) -> Option<[f32; 3]> {
    let village = Rect::new(-32.0, 72.0, -47.0, 29.0);
    if !village.contains(x, z) && !town::NORTH_PLATE.contains(x, z) {
        return None;
    }
    if town::ROADS.iter().any(|r| r.contains(x, z)) {
        return Some(rgb(0xa0a5a9));
    }
    if town::TRAIL.contains(x, z) {
        return Some(rgb(0xc8a46a));
    }
    if town::POND.contains(x, z) || town::CREEK.contains(x, z) {
        return Some(rgb(0x5b9fd4));
    }
    Some(rgb(0x6fae5a))
}

/// Paints the map (once): returns a PNG data URL.
pub fn paint(law: &Island) -> Option<String> {
    let t0 = engine::log::now_ms();
    let w = ((MX1 - MX0) / SCALE) as u32;
    let h = ((MZ1 - MZ0) / SCALE) as u32;
    let paper = rgb(0xefe3c2);
    let mut land = vec![false; (w * h) as usize];
    let mut px = vec![0u8; (w * h * 4) as usize];
    let sun = Vec3::new(-0.6, 0.7, -0.4).normalize();
    for j in 0..h {
        let z = MZ0 + (j as f32 + 0.5) * SCALE;
        for i in 0..w {
            let x = MX0 + (i as f32 + 0.5) * SCALE;
            let idx = (j * w + i) as usize;
            let col = if z > SOUTH_Z {
                match strip_at(x, z) {
                    Some(c) => {
                        land[idx] = true;
                        c
                    }
                    None => rgb(0x7fb6d6),
                }
            } else {
                let n = law.lerp(x, z);
                if n.coast > 0.0 {
                    // the sea: the shelf lighter, the deep darker
                    if n.h > -3.0 {
                        rgb(0x9fd0e0)
                    } else {
                        rgb(0x6fa6c8)
                    }
                } else {
                    land[idx] = true;
                    let hh = n.h;
                    let mut c = if hh > 18.0 + 14.0 * n.warm && n.warm < 0.5 {
                        rgb(0xf4f6f8)
                    } else if hh > 11.0 + 14.0 * n.warm {
                        rgb(0xa8acae)
                    } else if n.coast > -12.0 {
                        if n.warm > 0.45 { rgb(0xe8d39a) } else { rgb(0xc9c3a8) }
                    } else if n.forest > 0.5 {
                        rgb(0x3f8a4a)
                    } else if n.warm < 0.45 {
                        rgb(0x7fa883)
                    } else {
                        rgb(0x8cc06a)
                    };
                    for p in PADS.iter() {
                        if (x - p.x).abs() < p.hx && (z - p.z).abs() < p.hz {
                            c = mix(c, rgb(0xd8cfb4), 0.6);
                        }
                    }
                    // hill-shade from the lattice's slope
                    let dx = law.lerp(x + 4.0, z).h - law.lerp(x - 4.0, z).h;
                    let dz = law.lerp(x, z + 4.0).h - law.lerp(x, z - 4.0).h;
                    let nrm = Vec3::new(-dx / 8.0 * 2.5, 1.0, -dz / 8.0 * 2.5).normalize();
                    let k = (0.72 + 0.45 * nrm.dot(sun)).clamp(0.55, 1.15);
                    [c[0] * k, c[1] * k, c[2] * k]
                }
            };
            let c = mix(col, paper, 0.22);
            px[idx * 4..idx * 4 + 4].copy_from_slice(&[c[0].clamp(0.0, 255.0) as u8, c[1].clamp(0.0, 255.0) as u8, c[2].clamp(0.0, 255.0) as u8, 255]);
        }
    }
    // the coast in ink
    for j in 1..h - 1 {
        for i in 1..w - 1 {
            let idx = (j * w + i) as usize;
            if land[idx] && (!land[idx - 1] || !land[idx + 1] || !land[idx - w as usize] || !land[idx + w as usize]) {
                px[idx * 4..idx * 4 + 3].copy_from_slice(&[74, 50, 28]);
            }
        }
    }
    let doc = web_sys::window()?.document()?;
    let c: HtmlCanvasElement = doc.create_element("canvas").ok()?.dyn_into().ok()?;
    c.set_width(w);
    c.set_height(h);
    let g: C2d = c.get_context("2d").ok()??.dyn_into().ok()?;
    let img = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&px), w, h).ok()?;
    g.put_image_data(&img, 0, 0).ok()?;
    let to = |x: f32, z: f32| (((x - MX0) / SCALE) as f64, ((z - MZ0) / SCALE) as f64);
    // the roads: a dark outline, the trail colour inside; the strip's trail and the way to the gate
    let mut lines: Vec<Vec<Vec2>> = law.road_lines();
    lines.push(vec![Vec2::new(0.0, -27.0), Vec2::new(0.0, -146.0), Vec2::new(island::GATE_X, -146.0), Vec2::new(island::GATE_X, SOUTH_Z)]);
    for (wdt, col) in [(5.0, "#5a3d22"), (2.6, "#e9c98a")] {
        g.set_line_width(wdt);
        g.set_stroke_style_str(col);
        g.set_line_cap("round");
        g.set_line_join("round");
        for l in &lines {
            g.begin_path();
            for (k, p) in l.iter().enumerate() {
                let (a, b) = to(p.x, p.y);
                if k == 0 {
                    g.move_to(a, b);
                } else {
                    g.line_to(a, b);
                }
            }
            g.stroke();
        }
    }
    // the north gate: a small dark bar across the way
    let (gx, gz) = to(island::GATE_X, island::GATE_Z);
    g.set_fill_style_str("#3a2814");
    g.fill_rect(gx - 6.0, gz - 2.0, 12.0, 4.0);
    let url = c.to_data_url().ok();
    log_info!("world map painted ({w}x{h}) in {:.0} ms", engine::log::now_ms() - t0);
    url
}

/// The map's HTML: the picture with its names (a pad's name once found, else "?").
pub fn html(url: &str, found: u32) -> String {
    let mut s = format!("<div class='bh-mapshade' data-action='map_close'></div><div class='bh-mapbox' data-action='map_close'><img src='{url}' alt='map'>");
    let name = |s: &mut String, x: f32, z: f32, t: &str, small: bool| {
        let (l, tp) = pct(x, z);
        s.push_str(&format!("<div class='bh-mapname{}' style='left:{l:.2}%;top:{tp:.2}%'>{}</div>", if small { " small" } else { "" }, crate::esc(t)));
    };
    name(&mut s, 20.0, -6.0, "Brickhollow", false);
    name(&mut s, -26.0, -80.0, "Route 1", true);
    name(&mut s, 2.0, -168.0, "Stonecrest", false);
    name(&mut s, island::GATE_X + 46.0, island::GATE_Z, "North Gate", true);
    for (i, p) in PADS.iter().enumerate() {
        if found & (1 << i) != 0 {
            name(&mut s, p.x, p.z, p.name, false);
        } else {
            let (l, tp) = pct(p.x, p.z);
            s.push_str(&format!("<div class='bh-mapq' style='left:{l:.2}%;top:{tp:.2}%'>?</div>"));
        }
    }
    s.push_str("<div class='bh-maptitle'>Brick Isle</div><div class='bh-mapclose'>M / Esc / tap: close</div></div>");
    s
}

/// Your arrow over the map (studs, the controller's yaw: 0 north, -pi/2 east).
pub fn me_html(x: f32, z: f32, yaw: f32) -> String {
    let (l, t) = pct(x, z);
    format!("<div class='bh-mapbox' style='border-color:transparent;box-shadow:none;background:none'><div class='bh-mapme' style='left:calc({l:.2}% - 8px);top:calc({t:.2}% - 11px);transform:rotate({:.0}deg)'></div></div>", -yaw.to_degrees())
}
