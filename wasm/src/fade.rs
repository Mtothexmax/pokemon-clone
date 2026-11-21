//! See-through occluders: a building or tree between the camera and the hero fades out smoothly
//! (the camera stays where it is) and comes back once the hero is out from behind it.
//! Uses the engine's per-entity `MeshRenderer.opacity`, which keeps the full shadow.

use engine::prelude::*;
use std::collections::HashMap;

use crate::town::OccluderSpec;

/// Opacity of a fully faded occluder.
const MIN_ALPHA: f32 = 0.25;
/// Fading speed, opacity per second.
const SPEED: f32 = 3.0;

struct Occluder {
    root: Entity,
    extra: Vec<Entity>,
    /// World bounds (box) — for models filled in from their meshes once they have loaded.
    bounds: Option<(Vec3, Vec3)>,
    meshes: Vec<Entity>,
    alpha: f32,
}

#[derive(Default)]
pub struct Fader {
    items: Vec<Occluder>,
    /// Seconds until the next look for occluders whose meshes have not appeared yet.
    retry: f32,
    /// Occluders currently hiding the hero (for tests).
    pub hiding: usize,
}

impl Fader {
    pub fn new(specs: Vec<OccluderSpec>) -> Self {
        let items = specs
            .into_iter()
            .map(|s| match s {
                OccluderSpec::Bricks { root, extra, min, max } => Occluder { root, extra, bounds: Some((min, max)), meshes: vec![], alpha: 1.0 },
                OccluderSpec::Model { root } => Occluder { root, extra: vec![], bounds: None, meshes: vec![], alpha: 1.0 },
            })
            .collect();
        Self { items, ..Default::default() }
    }

    /// More occluders (a town on the island, built later).
    pub fn add(&mut self, specs: Vec<OccluderSpec>) {
        for s in specs {
            if let OccluderSpec::Bricks { root, extra, min, max } = s {
                self.items.push(Occluder { root, extra, bounds: Some((min, max)), meshes: vec![], alpha: 1.0 });
            }
        }
        self.retry = 0.0;
    }

    /// Collects the mesh entities (and a model's bounds) of every occluder that has none yet.
    /// A glTF can still be instantiating when the asset counter first reaches 0, so this runs
    /// again (once a second) until every occluder has its meshes.
    fn resolve(&mut self, ctx: &Ctx) {
        let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
        for (e, p) in ctx.world.query::<(Entity, &Parent)>().iter() {
            children.entry(p.0).or_default().push(e);
        }
        for it in self.items.iter_mut().filter(|it| it.meshes.is_empty()) {
            let mut stack = vec![it.root];
            stack.extend(it.extra.iter().copied());
            let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            while let Some(e) = stack.pop() {
                if let Ok(mr) = ctx.world.get::<&MeshRenderer>(e) {
                    it.meshes.push(e);
                    if let (Ok(g), Some(mesh)) = (ctx.world.get::<&GlobalTransform>(e), ctx.renderer.mesh(mr.mesh)) {
                        let m = g.0;
                        let c = m.transform_point3(mesh.bounds_center);
                        let s = m.x_axis.truncate().length().max(m.y_axis.truncate().length()).max(m.z_axis.truncate().length());
                        let r = Vec3::splat(mesh.bounds_radius * s);
                        min = min.min(c - r);
                        max = max.max(c + r);
                    }
                }
                if let Some(c) = children.get(&e) {
                    stack.extend(c.iter().copied());
                }
            }
            if it.meshes.is_empty() {
                continue;
            }
            if it.alpha < 1.0 {
                for e in &it.meshes {
                    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(*e) {
                        mr.opacity = it.alpha;
                    }
                }
            }
            if it.bounds.is_none() && min.x < max.x {
                // a tree's bounding spheres are generous: shrink to the trunk and crown a little
                let c = (min + max) * 0.5;
                let h = (max - min) * 0.5 * Vec3::new(0.7, 1.0, 0.7);
                it.bounds = Some((c - h, c + h));
            }
        }
    }

    /// Occluders whose meshes have not been found yet (for tests).
    pub fn unresolved(&self) -> usize {
        self.items.iter().filter(|it| it.meshes.is_empty()).count()
    }

    /// Fades whatever hides any of `targets` (points on the hero) from `cam`.
    pub fn update(&mut self, ctx: &mut Ctx, cam: Vec3, targets: &[Vec3], dt: f32, loaded: bool) {
        self.retry -= dt;
        if loaded && self.retry <= 0.0 && self.unresolved() > 0 {
            self.resolve(ctx);
            self.retry = 1.0;
        }
        self.hiding = 0;
        let step = SPEED * dt;
        for it in &mut self.items {
            let hides = it.bounds.is_some_and(|(min, max)| targets.iter().any(|t| segment_hits_box(cam, *t, min, max)));
            self.hiding += hides as usize;
            let goal = if hides { MIN_ALPHA } else { 1.0 };
            let a = if it.alpha < goal { (it.alpha + step).min(goal) } else { (it.alpha - step).max(goal) };
            if a != it.alpha {
                it.alpha = a;
                for e in &it.meshes {
                    if let Ok(mut mr) = ctx.world.get::<&mut MeshRenderer>(*e) {
                        mr.opacity = a;
                    }
                }
            }
        }
    }

    /// Occluders that are not fully opaque (for tests).
    pub fn faded(&self) -> usize {
        self.items.iter().filter(|it| it.alpha < 0.999).count()
    }
}

/// Does the segment a -> b pass through the box (slab test)?
fn segment_hits_box(a: Vec3, b: Vec3, min: Vec3, max: Vec3) -> bool {
    let d = b - a;
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        if d[k].abs() < 1e-6 {
            if a[k] < min[k] || a[k] > max[k] {
                return false;
            }
        } else {
            let (mut n, mut f) = ((min[k] - a[k]) / d[k], (max[k] - a[k]) / d[k]);
            if n > f {
                std::mem::swap(&mut n, &mut f);
            }
            t0 = t0.max(n);
            t1 = t1.min(f);
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}
