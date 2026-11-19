//! LEGO minifigures: six glTF parts (head, torso, arms, legs; made by the Kaigen port's
//! `tools/make_minifig.py`) hung under a root at their joint pivots, posed with the three.js
//! `Minifig.animate` walk cycle (legs and arms swing around x).

use engine::prelude::*;

/// Joint pivots in figure space, metres (feet at 0, facing +z).
const PIVOT: [Vec3; 6] = [
    Vec3::new(0.0, 1.501, 0.0),
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(-0.3268, 1.1856, -0.038),
    Vec3::new(0.3268, 1.1856, -0.038),
    Vec3::new(-0.152, 0.589, 0.0),
    Vec3::new(0.152, 0.589, 0.0),
];
const PARTS: [&str; 6] = ["head", "torso", "armL", "armR", "legL", "legR"];
const ARM_REST: f32 = -0.22;

pub struct Figure {
    /// The figure's root (feet at its origin, facing +z).
    pub root: Entity,
    parts: [Entity; 6],
    /// Swing angle per part (x rotation), as the three.js `rotation.x`.
    rot: [f32; 6],
    walk_t: f32,
}

impl Figure {
    /// Spawns figure `style` (a folder in assets/minifig) with its root at `t`, or as a child of `parent`.
    pub fn spawn(ctx: &mut Ctx, style: &str, t: Transform, parent: Option<Entity>) -> Self {
        let root = ctx.world.spawn((t, Name(format!("fig:{style}"))));
        if let Some(p) = parent {
            let _ = ctx.world.insert_one(root, Parent(p));
        }
        let parts = std::array::from_fn(|i| {
            let e = ctx.load_gltf(&format!("assets/minifig/{style}/{}.glb", PARTS[i]), Transform::from_translation(PIVOT[i]));
            let _ = ctx.world.insert_one(e, Parent(root));
            e
        });
        let mut f = Self { root, parts, rot: [0.0, 0.0, ARM_REST, ARM_REST, 0.0, 0.0], walk_t: 0.0 };
        f.apply(ctx);
        f
    }

    fn apply(&mut self, ctx: &mut Ctx) {
        for i in 0..6 {
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(self.parts[i]) {
                t.rotation = Quat::from_rotation_x(self.rot[i]);
            }
        }
    }

    /// The walk cycle: `speed` 0 = standing (eases back), 1 = walking, 1.6 = running.
    pub fn animate(&mut self, ctx: &mut Ctx, dt: f32, moving: bool, speed: f32) {
        if moving {
            self.walk_t += dt * 9.0 * speed;
            let s = self.walk_t.sin() * 0.7;
            self.rot[4] = s;
            self.rot[5] = -s;
            self.rot[2] = ARM_REST - s * 0.8;
            self.rot[3] = ARM_REST + s * 0.8;
        } else {
            let k = (dt * 10.0).min(1.0);
            for i in [4, 5] {
                self.rot[i] *= 1.0 - k;
            }
            for i in [2, 3] {
                self.rot[i] += (ARM_REST - self.rot[i]) * k;
            }
        }
        self.apply(ctx);
    }

    /// Leg swing (for tests: a walking figure swings its legs).
    pub fn leg_swing(&self) -> f32 {
        self.rot[4]
    }
}
