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
    // (the legs' hip-joint cylinders end 2 mm short of x = 0: the glTFs' reached 3 cm past it, the two coaxial
    // cylinders lay in each other, z-fighting; trimmed in assets/minifig/*/leg{L,R}.glb)
    Vec3::new(-0.152, 0.589, 0.0),
    Vec3::new(0.152, 0.589, 0.0),
];
const PARTS: [&str; 6] = ["head", "torso", "armL", "armR", "legL", "legR"];
const ARM_REST: f32 = -0.22;

/// What a figure does besides walking (Kaigen `FigPose`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Pose {
    /// walk / stand
    #[default]
    Normal,
    /// on a bench: legs forward, hands on the knees, feet swinging
    Sit,
    /// arms up, hopping (the caller turns `spin`)
    Dance,
    ArmsUp,
    /// hiding
    Crouch,
    /// right arm waving
    Wave,
}

pub struct Figure {
    /// The figure's root (feet at its origin, facing +z).
    pub root: Entity,
    parts: [Entity; 6],
    /// Swing angle per part (x rotation), as the three.js `rotation.x`.
    rot: [f32; 6],
    walk_t: f32,
    /// the pose (set by the game each step) and its clock
    pub pose: Pose,
    idle_t: f32,
    /// extra turn of the whole figure about its feet (dancing)
    pub spin: f32,
    /// the parts' offset (metres, figure space): hopping, sitting, crouching
    offset: Vec3,
    /// in the air (a jump): arms up, legs apart (set by the game)
    pub air: bool,
    /// the head turns towards this yaw (radians, figure space, clamped to +-1; Kaigen: a villager looks at you
    /// within 7 m while standing) and where it is now
    pub head_want: f32,
    head_yaw: f32,
    /// the folder in assets/minifig
    style: String,
    /// talking: the face material and its prints (closed = the head's own, half open, open; `want_mouths` loads
    /// them from head_mouth{1,2}.glb, made by tools/mouths.py), the copies still loading, the print shown
    face_mat: Option<MaterialId>,
    mouths: Vec<TextureId>,
    mouth_load: Vec<Entity>,
    mouth: usize,
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
        let mut f = Self { root, parts, rot: [0.0, 0.0, ARM_REST, ARM_REST, 0.0, 0.0], walk_t: 0.0, pose: Pose::Normal, idle_t: 0.0, spin: 0.0, offset: Vec3::ZERO, air: false, head_want: 0.0, head_yaw: 0.0, style: style.to_string(), face_mat: None, mouths: vec![], mouth_load: vec![], mouth: 0 };
        f.apply(ctx);
        f
    }

    /// Loads the open-mouth prints once (out of sight) and keeps their textures; the face material is the head's
    /// primitive whose material name ends in `_face`. Call every frame while someone may talk (cheap once done).
    pub fn want_mouths(&mut self, ctx: &mut Ctx) {
        if self.mouths.len() == 3 {
            return;
        }
        let opts = engine::gltf_loader::GltfOpts { auto_lod: false, ..Default::default() };
        if self.mouth_load.is_empty() {
            for n in 1..=2 {
                let e = ctx.load_gltf_with(&format!("assets/minifig/{}/head_mouth{n}.glb", self.style), Transform::from_xyz(0.0, -1000.0, 0.0), opts);
                self.mouth_load.push(e);
            }
            return;
        }
        let face_of = |ctx: &Ctx, root: Entity| -> Option<(MaterialId, TextureId)> {
            engine::scene::descendants(&ctx.world, root).into_iter().find_map(|e| {
                let mr = ctx.world.get::<&MeshRenderer>(e).ok()?;
                let m = ctx.renderer.material(mr.material)?;
                if m.name.ends_with("_face") { Some((mr.material, m.texture?)) } else { None }
            })
        };
        let Some((mat, closed)) = face_of(ctx, self.parts[0]) else { return };
        let opened: Vec<TextureId> = self.mouth_load.iter().filter_map(|e| face_of(ctx, *e).map(|f| f.1)).collect();
        if opened.len() < 2 {
            return; // still loading
        }
        self.face_mat = Some(mat);
        self.mouths = vec![closed, opened[0], opened[1]];
        for e in std::mem::take(&mut self.mouth_load) {
            ctx.despawn(e);
        }
    }

    /// Talking: how open the mouth is (0..1, the voice's loudness), shown as the closed, half or open print (a
    /// little hysteresis so it does not flicker between two).
    pub fn set_mouth(&mut self, ctx: &mut Ctx, open: f32) {
        let (Some(mat), true) = (self.face_mat, self.mouths.len() == 3) else { return };
        let up = [0.0, 0.18, 0.55];
        let down = [0.0, 0.1, 0.42];
        let mut f = self.mouth;
        while f < 2 && open > up[f + 1] {
            f += 1;
        }
        while f > 0 && open < down[f] {
            f -= 1;
        }
        if f != self.mouth {
            self.mouth = f;
            if let Some(m) = ctx.renderer.material_mut(mat) {
                m.texture = Some(self.mouths[f]);
            }
        }
    }

    /// The print shown (0 closed, 1 half, 2 open; debug).
    pub fn mouth(&self) -> usize {
        self.mouth
    }

    /// Each part: its pivot turned by the spin and moved by the offset, then its own swing (Kaigen `fig_set_part`).
    fn apply(&mut self, ctx: &mut Ctx) {
        let spin = Quat::from_rotation_y(self.spin);
        for i in 0..6 {
            if let Ok(mut t) = ctx.world.get::<&mut Transform>(self.parts[i]) {
                t.translation = spin * PIVOT[i] + self.offset;
                let head = if i == 0 { Quat::from_rotation_y(self.head_yaw) } else { Quat::IDENTITY };
                t.rotation = spin * head * Quat::from_rotation_x(self.rot[i]);
            }
        }
    }

    /// The walk cycle: `speed` 0 = standing (eases back), 1 = walking, 1.6 = running.
    pub fn animate(&mut self, ctx: &mut Ctx, dt: f32, moving: bool, speed: f32) {
        self.idle_t += dt;
        let moving = moving && self.pose != Pose::Sit;
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
        // the poses (Kaigen fig_animate) over the walk swing: breathing arms, the head turned, a jump
        let t = self.idle_t;
        self.offset = Vec3::ZERO;
        let mut rot = self.rot;
        let breathe = (t * 1.6).sin() * 0.04;
        rot[2] += breathe;
        rot[3] -= breathe;
        self.head_yaw += (self.head_want.clamp(-1.0, 1.0) - self.head_yaw) * (dt * 5.0).min(1.0);
        if self.air {
            rot[2] = ARM_REST - 1.6;
            rot[3] = ARM_REST - 1.6;
            rot[4] = -0.5;
            rot[5] = 0.3;
        }
        match self.pose {
            Pose::Normal => {}
            Pose::Sit => {
                let sw = (t * 2.0).sin() * 0.12;
                rot[4] = -1.5 + sw;
                rot[5] = -1.5 - sw;
                rot[2] = -0.75;
                rot[3] = -0.75;
                self.offset = Vec3::new(0.0, -0.08, -0.42); // onto the seat behind the feet
            }
            Pose::Dance | Pose::ArmsUp => {
                let wave = (t * 8.0).sin() * 0.3;
                rot[2] = -2.6 + wave;
                rot[3] = -2.6 - wave;
                self.offset.y = (t * 6.0).sin().abs() * if self.pose == Pose::Dance { 0.13 } else { 0.11 };
            }
            Pose::Crouch => {
                self.offset.y = -0.19;
                rot[4] = -0.6;
                rot[5] = -0.6;
            }
            Pose::Wave => rot[3] = -2.6 + (t * 8.0).sin() * 0.3,
        }
        let keep = self.rot;
        self.rot = rot;
        self.apply(ctx);
        self.rot = keep;
    }

    /// The head is (still) turned (the game keeps animating a figure until it looks ahead again).
    pub fn head_turned(&self) -> bool {
        self.head_yaw.abs() > 0.01
    }

    pub fn head_yaw(&self) -> f32 {
        self.head_yaw
    }

    /// Leg swing (for tests: a walking figure swings its legs).
    pub fn leg_swing(&self) -> f32 {
        self.rot[4]
    }
}
