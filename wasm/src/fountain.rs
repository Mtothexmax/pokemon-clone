//! The water of Stonecrest's fountain (the stone is built with the town: `north.rs` `fountain`).
//!
//! * The basin and the bowl hold the engine's water (`Material::water` on round meshes): small quick ripples,
//!   fresnel reflection of the sky (+ screen-space reflections where the graphics allow), refraction and depth
//!   tint over the tiled floor, caustics, sun glints and a little foam along the stone.
//! * A jet rises from the nozzle and falls back into the bowl (a clear tapering column + droplets), the bowl
//!   overflows in six arcs into the basin (clear streams + droplets running down them), and where they hit the
//!   water spray jumps up and rings spread out (flat ring meshes growing and fading).
//!
//! The emitters sit on the square's render layer (`ParticleEmitter::layer`): a camera without that band (the village,
//! the rooms, the dive, the battle table) does not draw them. The ring meshes only move while the band is drawn
//! (`set_visible`).

use crate::town::{self, BOWL_R, BOWL_WATER, FOUNTAIN, FOUNTAIN_IN, FOUNTAIN_WATER, NOZZLE_TOP, STUD};
use engine::geom::MeshBuilder;
use engine::mesh::MeshData;
use engine::particles::{ParticleEmitter, ParticlePreset, ParticleShape, ParticleSystem};
use engine::prelude::*;
use engine::renderer::Material;
use serde_json::{json, Value};

/// The overflowing streams: how many, the horizontal speed (m/s) they leave the rim with.
const STREAMS: usize = 6;
const STREAM_V: f32 = 0.62;
const G: f32 = 9.81;
/// Ripple rings: per spot at once, how long one lives (s), how wide it grows (m).
const RINGS_PER: usize = 2;
const RING_LIFE: f32 = 1.4;
const RING_SIZE: f32 = 0.34;

struct Ring {
    e: Entity,
    at: Vec3,
    t: f32,
    size: f32,
}

#[derive(Default)]
pub struct Fountain {
    built: bool,
    layer: u8,
    visible: bool,
    emitters: Vec<Entity>,
    rings: Vec<Ring>,
    t: f32,
    /// where the streams land (metres)
    pub impacts: Vec<Vec3>,
    toggles: u32,
}

fn centre() -> Vec3 {
    town::m(FOUNTAIN[0], 0.0, FOUNTAIN[1])
}

/// Where a stream leaving the rim at angle `a` is after `t` seconds (metres).
fn stream_at(a: f32, t: f32) -> Vec3 {
    let dir = Vec3::new(a.cos(), 0.0, a.sin());
    let start = centre() + dir * (BOWL_R + 0.02) * STUD + Vec3::Y * (BOWL_WATER + 0.06) * STUD;
    start + dir * STREAM_V * t - Vec3::Y * 0.5 * G * t * t
}

/// Seconds until a stream reaches the basin's water.
fn fall_time() -> f32 {
    let drop = (BOWL_WATER + 0.06 - FOUNTAIN_WATER) * STUD;
    (2.0 * drop / G).sqrt()
}

impl Fountain {
    pub fn spawn(ctx: &mut Ctx) -> Self {
        let c = centre();
        let layer = town::layer_at(FOUNTAIN[0], FOUNTAIN[1]);
        let mut f = Fountain { layer, built: true, visible: true, ..Default::default() };
        // ---- the water: clear, small quick ripples, foam along the stone ----
        let water = ctx.renderer.add_material(Material {
            name: "fountain water".into(),
            wave_height: 0.0035,
            wave_scale: 4.5,
            water_chop: 0.45,
            water_clarity: 0.45,
            water_foam: 0.45,
            water_vertex_spacing: 0.04,
            roughness: 0.04,
            ..Material::water(0x0b4f6a, 0x58c6d8)
        });
        for (r, y, name) in [(FOUNTAIN_IN + 0.04, FOUNTAIN_WATER, "fountain basin water"), (BOWL_R - 0.1, BOWL_WATER, "fountain bowl water")] {
            let rings = ((r * STUD) / 0.04).ceil() as u32;
            let mesh = ctx.renderer.add_mesh(name, &MeshData::disk(r * STUD, rings.clamp(6, 28), 48, 1.0));
            let e = ctx.spawn_mesh(name, mesh, water, Transform::from_translation(c + Vec3::Y * y * STUD));
            ctx.set_layer(e, layer);
        }
        // ---- the jet and the six falling streams: clear tubes ----
        let mut mb = MeshBuilder::new();
        mb.part("stream");
        let jet_top = NOZZLE_TOP + 1.25;
        mb.with(Mat4::from_translation(c + Vec3::Y * NOZZLE_TOP * STUD), |b| {
            b.lathe(&[Vec2::new(0.0, 0.0), Vec2::new(0.13 * STUD, 0.0), Vec2::new(0.05 * STUD, (jet_top - NOZZLE_TOP) * STUD), Vec2::new(0.0, (jet_top - NOZZLE_TOP + 0.08) * STUD)], 10);
        });
        let tf = fall_time();
        for k in 0..STREAMS {
            let a = k as f32 / STREAMS as f32 * std::f32::consts::TAU + 0.26;
            let path: Vec<Vec3> = (0..=14).map(|i| stream_at(a, tf * i as f32 / 14.0)).collect();
            // a flat ribbon of water: wide across, thin along the fall
            let profile: Vec<Vec2> = (0..8).map(|i| {
                let t = i as f32 / 8.0 * std::f32::consts::TAU;
                Vec2::new(t.cos() * 0.16 * STUD, t.sin() * 0.05 * STUD)
            }).collect();
            mb.sweep(&path, &profile, true);
            f.impacts.push(stream_at(a, tf));
        }
        let mut md = mb.finish().into_iter().next().unwrap().1;
        engine::geom::smooth_normals(&mut md, 70.0);
        let mesh = ctx.renderer.add_mesh("fountain streams", &md);
        let clear = ctx.renderer.add_material(Material {
            name: "fountain stream".into(),
            emissive: Vec3::new(0.05, 0.09, 0.12),
            double_sided: true,
            ..Material::color(0xd6f2ff).rough(0.05).transparent(0.5)
        });
        let e = ctx.spawn_mesh("fountain streams", mesh, clear, Transform::default());
        ctx.set_layer(e, layer);
        if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
            mr.cast_shadows = false;
        }
        // ---- droplets: up the jet and back into the bowl, down every stream; spray where they land ----
        let drop = |rate: f32, speed: (f32, f32), dir: Vec3, spread: f32, life: (f32, f32)| ParticleEmitter {
            rate,
            max: 120,
            lifetime: life,
            speed,
            direction: dir,
            spread,
            gravity: Vec3::new(0.0, -G, 0.0),
            drag: 0.0,
            size: (0.028, 0.02),
            color_start: Vec4::new(0.85, 0.95, 1.0, 0.8),
            color_end: Vec4::new(0.85, 0.95, 1.0, 0.25),
            fade_in: 0.05,
            shape: ParticleShape::Spark,
            stretch: 0.03,
            burst: 0,
            one_shot: false,
            ..ParticleEmitter::preset(ParticlePreset::Splash)
        };
        let jet = drop(38.0, (2.95, 3.15), Vec3::Y, 0.11, (0.64, 0.7));
        f.emitters.push(emitter(ctx, "fountain jet", c + Vec3::Y * NOZZLE_TOP * STUD, jet, layer));
        for k in 0..STREAMS {
            let a = k as f32 / STREAMS as f32 * std::f32::consts::TAU + 0.26;
            let dir = Vec3::new(a.cos(), 0.0, a.sin());
            let mut e = drop(14.0, (STREAM_V * 0.95, STREAM_V * 1.05), dir, 0.05, (tf * 0.95, tf));
            e.area = Vec3::new(0.02, 0.0, 0.02);
            f.emitters.push(emitter(ctx, "fountain stream drops", stream_at(a, 0.0), e, layer));
            let spray = ParticleEmitter {
                rate: 22.0,
                max: 50,
                lifetime: (0.22, 0.4),
                speed: (0.45, 0.85),
                direction: Vec3::Y,
                spread: 0.75,
                area: Vec3::new(0.03, 0.0, 0.03),
                gravity: Vec3::new(0.0, -G, 0.0),
                drag: 0.0,
                size: (0.04, 0.02),
                color_start: Vec4::new(0.95, 0.98, 1.0, 0.85),
                color_end: Vec4::new(0.95, 0.98, 1.0, 0.0),
                fade_in: 0.0,
                shape: ParticleShape::Dot,
                burst: 0,
            one_shot: false,
            ..ParticleEmitter::preset(ParticlePreset::Splash)
            };
            f.emitters.push(emitter(ctx, "fountain spray", f.impacts[k] + Vec3::Y * 0.01, spray, layer));
        }
        // ---- ripple rings: where the streams land and around the jet in the bowl ----
        let mut rb = MeshBuilder::new();
        rb.part("ring");
        rb.lathe(&[Vec2::new(1.0, 0.0), Vec2::new(0.78, 0.0)], 32);
        let ring_mesh = ctx.renderer.add_mesh("fountain ripple", &rb.finish().into_iter().next().unwrap().1);
        let foam = ctx.renderer.add_material(Material { name: "fountain ripple".into(), double_sided: true, ..Material::color(0xeaf8ff).rough(0.1).transparent(0.55) });
        let bowl_spot = c + Vec3::Y * (BOWL_WATER * STUD + 0.004);
        let spots: Vec<(Vec3, f32)> = f.impacts.iter().map(|p| (Vec3::new(p.x, FOUNTAIN_WATER * STUD + 0.004, p.z), RING_SIZE)).chain(std::iter::once((bowl_spot, (BOWL_R - 0.2) * STUD))).collect();
        for (i, (at, size)) in spots.into_iter().enumerate() {
            for k in 0..RINGS_PER {
                let e = ctx.spawn_mesh("fountain ripple", ring_mesh, foam, Transform::from_translation(at).with_scale(Vec3::splat(0.01)));
                ctx.set_layer(e, layer);
                if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(e) {
                    mr.cast_shadows = false;
                }
                // staggered: each ring its own start
                let t = (k as f32 / RINGS_PER as f32 + i as f32 * 0.37).fract() * RING_LIFE;
                f.rings.push(Ring { e, at, t, size });
            }
        }
        f
    }

    /// The rings only move while the fountain's band is drawn (outside, near Stonecrest); the particles need nothing
    /// (their layer).
    pub fn set_visible(&mut self, camera_layers: u32, outside: bool) {
        let v = self.built && outside && camera_layers & (1 << self.layer) != 0;
        if v != self.visible {
            self.visible = v;
            self.toggles += 1;
        }
    }

    pub fn update(&mut self, ctx: &mut Ctx, dt: f32) {
        if !self.visible {
            return;
        }
        self.t += dt;
        for r in &mut self.rings {
            r.t = (r.t + dt) % RING_LIFE;
            let k = r.t / RING_LIFE;
            // fast at first, slowing down; fading out as it grows
            let grow = 1.0 - (1.0 - k).powi(2);
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(r.e) {
                t.translation = r.at;
                t.scale = Vec3::new(0.02 + r.size * grow, 1.0, 0.02 + r.size * grow);
            }
            if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(r.e) {
                mr.opacity = (1.0 - k).powf(1.5) * (k * 12.0).min(1.0);
            }
        }
    }

    pub fn describe(&self, ctx: &Ctx) -> Value {
        let live: usize = self.emitters.iter().filter_map(|e| ctx.world.get::<&ParticleSystem>(*e).ok().map(|p| p.emitter.enabled as usize)).sum();
        let layers: Vec<u8> = self.emitters.iter().filter_map(|e| ctx.world.get::<&ParticleSystem>(*e).ok().map(|p| p.emitter.layer)).collect();
        let r = |v: f32| (v * 100.0).round() / 100.0;
        json!({"built": self.built, "visible": self.visible, "layer": self.layer, "emitters": self.emitters.len(), "emitting": live, "emitter_layers_ok": layers.iter().all(|l| *l == self.layer), "rings": self.rings.len(),
               "ring_opacity": self.rings.first().and_then(|g| ctx.world.get::<&MeshRenderer>(g.e).ok().map(|m| r(m.opacity))),
               "impacts": self.impacts.iter().map(|p| [r(p.x / STUD), r(p.z / STUD)]).collect::<Vec<_>>(), "fall_time": r(fall_time()), "toggles": self.toggles, "t": r(self.t)})
    }
}

/// An emitter on render layer `layer` with its own fixed seed (the same droplets every run).
fn emitter(ctx: &mut Ctx, name: &str, pos: Vec3, mut e: ParticleEmitter, layer: u8) -> Entity {
    let seed = 0x5eed_f0u64 + name.len() as u64 * 7919 + (pos.x.to_bits() as u64 ^ pos.z.to_bits() as u64);
    e.layer = layer;
    ctx.spawn_particles_seeded(name, pos, e, seed)
}
