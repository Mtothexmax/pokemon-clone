//! The rooms (Kaigen `st_rooms.c`, the three.js `interiors.ts`): built once far east of the village
//! (x >= 187 studs) by the town builder; going through a door is a teleport (`PlayScreen` doors), so
//! there is no loading. The rooms are bigger than the houses look from outside, open at the top and
//! low at the front so the fixed camera looks in from the south. Each room is drawn alone: the village
//! and every room have their own render layer (`layer_of`), the camera draws the one you are in.
//!
//! A child module of `town` (it builds with the town's private brick helpers).

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum RoomId {
    Home = 0,
    Dex = 1,
    Lab = 2,
    Bedroom = 3,
}

pub const ROOMS: [RoomId; 4] = [RoomId::Home, RoomId::Dex, RoomId::Lab, RoomId::Bedroom];

/// A room: label (the area title), centre and size (studs), the outside door face (studs; the bedroom
/// upstairs has none).
pub struct RoomDef {
    pub label: &'static str,
    pub ox: f32,
    pub oz: f32,
    pub w: f32,
    pub d: f32,
    pub door_x: f32,
    pub door_z: f32,
}

pub fn room_def(id: RoomId) -> RoomDef {
    match id {
        RoomId::Home => RoomDef { label: "Home", ox: 200.0, oz: 0.0, w: 26.0, d: 18.0, door_x: -15.0, door_z: -6.0 },
        RoomId::Dex => RoomDef { label: "Dex & Lumi's House", ox: 260.0, oz: 0.0, w: 26.0, d: 18.0, door_x: 47.0, door_z: -6.0 },
        RoomId::Lab => RoomDef { label: "Quill Lab", ox: 330.0, oz: 0.0, w: 34.0, d: 22.0, door_x: 13.0, door_z: 16.0 },
        RoomId::Bedroom => RoomDef { label: "Your Room", ox: 200.0, oz: 40.0, w: 18.0, d: 14.0, door_x: 0.0, door_z: 0.0 },
    }
}

/// The stairs in the home (ascending towards -z, the top leads up).
pub const HOME_STAIRS: Rect = Rect::new(209.1, 212.1, -4.4, 2.1);
/// The stairwell in the bedroom (leads down).
pub const BED_WELL: Rect = Rect::new(205.5, 208.5, 42.0, 46.0);

/// Where the Deck-o-Matic's screen hangs (studs): the middle lab machine.
pub const MACHINE_SCREEN: [f32; 3] = [341.6, 5.1, -8.19];

/// Render layers: 0 = you (always drawn), the village, one per room, and a layer no camera draws.
pub const VILLAGE_LAYER: u8 = 1;
pub const HIDDEN_LAYER: u8 = 31;
pub fn layer_of(area: Option<RoomId>) -> u8 {
    match area {
        None => VILLAGE_LAYER,
        Some(r) => 2 + r as u8,
    }
}

/// The room you are in at (x, z) studs (Kaigen `world_area_at`), None outside.
pub fn area_at(x: f32, z: f32) -> Option<RoomId> {
    if x < 150.0 {
        return None;
    }
    if z > 25.0 {
        return Some(RoomId::Bedroom);
    }
    [RoomId::Home, RoomId::Dex, RoomId::Lab].into_iter().min_by(|a, b| (room_def(*a).ox - x).abs().total_cmp(&(room_def(*b).ox - x).abs()))
}

/// A door or the stairs at (x, z) studs in `area`: where it takes you (studs) and the controller's yaw
/// there (0 = facing north / into a room, pi = south). Kaigen `world_doors`.
pub fn door_at(area: Option<RoomId>, x: f32, z: f32) -> Option<(Vec2, f32)> {
    use std::f32::consts::PI;
    match area {
        None => [RoomId::Home, RoomId::Dex, RoomId::Lab].into_iter().find_map(|id| {
            let r = room_def(id);
            ((x - r.door_x).abs() < 1.6 && z > r.door_z && z < r.door_z + 2.0).then(|| (Vec2::new(r.ox, r.oz + r.d * 0.5 - 2.6), 0.0))
        }),
        Some(RoomId::Bedroom) => (x > BED_WELL.x0 && x < BED_WELL.x1 && z > BED_WELL.z0 + 0.4).then(|| (Vec2::new(210.6, 3.6), PI)),
        Some(id) => {
            let r = room_def(id);
            let st = HOME_STAIRS;
            if (x - r.ox).abs() < 1.8 && z > r.oz + r.d * 0.5 - 0.5 {
                Some((Vec2::new(r.door_x, r.door_z + 3.0), PI))
            } else if id == RoomId::Home && x > st.x0 && x < st.x1 && z < st.z0 + 1.2 {
                Some((Vec2::new(207.0, 40.0), 0.0))
            } else {
                None
            }
        }
    }
}

/// The warm ceiling lights (studs, 8.5 up) of a room: two or three over the back, one over the front.
pub fn room_lights(id: RoomId) -> Vec<(Vec3, f32)> {
    let r = room_def(id);
    let n = if r.w > 30.0 { 3 } else { 2 };
    let mut v: Vec<(Vec3, f32)> = (0..n).map(|k| (Vec3::new(r.ox - r.w * 0.5 + r.w * (k as f32 + 0.5) / n as f32, 8.5, r.oz - r.d * 0.15), 1.0)).collect();
    v.push((Vec3::new(r.ox, 8.5, r.oz + r.d * 0.3), 0.6));
    v
}

/// The villagers in the rooms (Kaigen app_init): Mom in the kitchen, Lumi at home, the Professor in her
/// lab (the floors are 0.4 studs high). id, figure style, studs, yaw (figures face +z at 0).
pub const ROOM_NPCS: [(&str, &str, f32, f32, f32); 3] = [
    ("mom", "mom", 193.4, -4.4, 0.0),
    ("lumi", "lumi", 254.5, 2.0, std::f32::consts::FRAC_PI_2),
    ("lab-prof", "quill", 337.0, -1.2, -std::f32::consts::FRAC_PI_2),
];
/// Floor height of the rooms (studs).
pub const FLOOR: f32 = 0.4;

impl Town {
    /// Furniture: a brick that also blocks walking.
    fn furn(&mut self, hex: u32, w: f32, h: f32, d: f32, x: f32, y: f32, z: f32, studs: bool) {
        self.brick(hex, w, h, d, x, y, z, studs);
        self.block(x - w * 0.5, x + w * 0.5, z - d * 0.5, z + d * 0.5);
    }

    /// Floor (planks or checker tiles), back and side walls with a wainscot, a low front wall with the door gap.
    fn room_shell(&mut self, r: &RoomDef, floor_a: u32, floor_b: u32, wall: u32, trim: u32, tiles: bool, door: bool) {
        let (ox, oz, w, d) = (r.ox, r.oz, r.w, r.d);
        // a dark floor around the room (instead of the baseplate)
        // (just below 0: its bottom face would lie in one plane with every wall's and tile's bottom; each room's a
        // little lower than the last, as they overlap)
        let k = ROOMS.iter().position(|id| room_def(*id).label == r.label).unwrap_or(0) as f32;
        self.brick(0x151f27, w + 60.0, 0.05, d + 50.0, ox, -0.05 - 0.04 * k, oz + 6.0, false);
        if tiles {
            // one slab in the first colour and thin inlays of the second: touching tiles of
            // two colours lay their tops within a hair of each other (z-fighting the whole floor, bridge check)
            self.brick(floor_a, w, 0.37, d, ox, 0.03, oz, false);
            let mut x = -w * 0.5;
            while x < w * 0.5 {
                let mut z = -d * 0.5;
                while z < d * 0.5 {
                    if ((x + z) * 0.5) as i32 & 1 != 0 {
                        // (sunk a little: their bottoms clear the furniture's, which stand at 0.4)
                        self.brick(floor_b, 1.92, 0.09, 1.92, ox + x + 1.0, 0.36, oz + z + 1.0, false);
                    }
                    z += 2.0;
                }
                x += 2.0;
            }
        } else {
            let mut x = -w * 0.5;
            while x < w * 0.5 {
                let c = if x.abs() as i32 & 1 != 0 { floor_b } else { floor_a };
                self.brick(c, 1.0, 0.4, d, ox + x + 0.5, 0.0, oz, false);
                x += 1.0;
            }
        }
        // the floor is walked on: one collider under the whole room
        self.solids.push(Solid { r: Rect::new(ox - w * 0.5, ox + w * 0.5, oz - d * 0.5, oz + d * 0.5 + 2.6), y0: 0.0, h: FLOOR * STUD });
        const H: f32 = 9.0;
        self.furn(wall, w + 2.0, H, 1.0, ox, 0.0, oz - d * 0.5 - 0.5, false);
        self.brick(trim, w, 2.4, 0.3, ox, 0.4, oz - d * 0.5 + 0.1, false);
        self.brick(trim, w + 2.0, 0.4, 1.4, ox, H, oz - d * 0.5 - 0.5, false);
        for s in [-1.0f32, 1.0] {
            self.furn(wall, 1.0, H, d + 1.0, ox + s * (w * 0.5 + 0.5), 0.0, oz - 0.5, false);
            self.brick(trim, 0.3, 2.4, d, ox + s * (w * 0.5 - 0.1), 0.4, oz, false);
        }
        if door {
            let seg = w * 0.5 - 1.6;
            for s in [-1.0f32, 1.0] {
                self.furn(wall, seg + 1.0, 1.2, 1.0, ox + s * (1.6 + seg * 0.5 + 0.5), 0.0, oz + d * 0.5 + 0.5, true);
            }
            self.brick(C_RED, 3.0, 0.1, 1.6, ox, 0.4, oz + d * 0.5 - 0.6, false); // door mat
            for s in [-1.0f32, 1.0] {
                self.brick(trim, 0.5, 1.6, 1.2, ox + s * 1.8, 0.0, oz + d * 0.5 + 0.5, true);
            }
            // beyond the gap: you leave through the door before you get here
            self.block(ox - 2.0, ox + 2.0, oz + d * 0.5 + 1.6, oz + d * 0.5 + 2.6);
        } else {
            self.furn(wall, w + 2.0, 1.2, 1.0, ox, 0.0, oz + d * 0.5 + 0.5, true);
        }
    }

    fn room_window(&mut self, r: &RoomDef, x: f32, w: f32, h: f32) {
        let z = r.oz - r.d * 0.5;
        self.brick(C_WHITE, w + 0.6, h + 0.6, 0.3, r.ox + x, 3.2, z + 0.1, false);
        self.brick(0xbfe9ff, w, h, 0.3, r.ox + x, 3.5, z + 0.2, false);
        self.brick(C_WHITE, 0.2, h, 0.35, r.ox + x, 3.5, z + 0.3, false);
    }

    fn room_frame(&mut self, r: &RoomDef, x: f32, y: f32, hex: u32) {
        let z = r.oz - r.d * 0.5;
        self.brick(C_BROWN, 2.2, 1.8, 0.2, r.ox + x, y, z + 0.1, false);
        self.brick(hex, 1.8, 1.4, 0.25, r.ox + x, y + 0.2, z + 0.15, false);
    }

    /// A sign board on the back wall (the painted strip of the Kaigen `tools/make_signs.py`).
    fn room_sign(&mut self, r: &RoomDef, path: &str, x: f32, y: f32) {
        self.prop(path, r.ox + x, r.oz - r.d * 0.5 + 0.07, y, 1.0, 0.0);
    }

    /// A potted plant: a brick pot with a LEGO bush or bamboo leaves on top.
    fn room_plant(&mut self, x: f32, z: f32, s: f32, bamboo: bool) {
        self.brick(C_RED_BROWN, 1.4, 1.2, 1.4, x, 0.4, z, false);
        let rot = self.rot();
        self.prop(if bamboo { "assets/props/bamboo.glb" } else { "assets/props/bush.glb" }, x, z, 1.6, if bamboo { s * 0.75 } else { s }, rot);
        self.block(x - 0.8, x + 0.8, z - 0.8, z + 0.8);
    }

    fn room_books(&mut self, x: f32, z: f32, w: f32, h: f32, y0: f32) {
        let cols = [C_RED, C_BLUE, C_YELLOW, C_DARK_GREEN, C_WHITE, C_ORANGE];
        let n = (w / 0.5) as i32;
        for i in 0..n {
            let hh = h * (0.7 + self.rand() * 0.3);
            self.brick(cols[(i as usize * 7) % 6], 0.4, hh, 0.9, x - w * 0.5 + 0.25 + i as f32 * 0.5, y0, z, false);
        }
    }

    fn room_bookshelf(&mut self, x: f32, z: f32, w: f32) {
        self.furn(C_BROWN, w, 7.0, 1.4, x, 0.4, z, false);
        for i in 0..4 {
            self.brick(C_RED_BROWN, w - 0.4, 0.2, 1.2, x, 1.2 + i as f32 * 1.6, z + 0.15, false);
            self.room_books(x, z + 0.3, w - 0.6, 1.2, 1.4 + i as f32 * 1.6);
        }
    }

    /// A screen: a dark frame with a bright face.
    fn room_screen(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, glow: u32) {
        self.brick(C_BLACK, w + 0.3, h + 0.3, 0.3, x, y - 0.15, z, false);
        self.brick(glow, w, h, 0.32, x, y, z + 0.03, false);
    }

    fn flower_pot_on(&mut self, x: f32, z: f32, y: f32) {
        let r = self.rot();
        self.prop("assets/props/flowerpot.glb", x, z, y, 1.0, r);
    }

    /// Builds the four rooms, each into its own group (`group_area` = the room).
    pub(crate) fn rooms_build(&mut self) {
        for id in ROOMS {
            self.area = id as i8;
            self.group(|t| match id {
                RoomId::Home => t.room_home(),
                RoomId::Bedroom => t.room_bedroom(),
                RoomId::Dex => t.room_dex(),
                RoomId::Lab => t.room_lab(),
            });
        }
        self.area = -1;
    }

    fn room_home(&mut self) {
        let r = room_def(RoomId::Home);
        let (ox, oz) = (r.ox, r.oz);
        self.room_shell(&r, 0xc8955c, 0xb88550, 0xf4e6c8, C_RED_BROWN, false, true);
        self.room_window(&r, -4.0, 4.0, 3.0);
        self.room_window(&r, 5.0, 4.0, 3.0);
        self.room_frame(&r, 0.6, 5.0, 0x7fc0ff);
        self.room_frame(&r, 9.5, 5.0, 0xffb070);
        // kitchen (back left): fridge, counter, stove
        self.furn(C_WHITE, 2.6, 6.4, 2.2, ox - 11.4, 0.4, oz - 7.4, false);
        self.brick(C_GRAY, 0.2, 2.0, 0.2, ox - 10.4, 3.4, oz - 6.2, false);
        self.furn(C_WHITE, 7.0, 2.6, 2.2, ox - 6.6, 0.4, oz - 7.4, false);
        self.brick(C_DARK_GRAY, 7.2, 0.3, 2.4, ox - 6.6, 3.0, oz - 7.4, true);
        self.brick(C_BLACK, 2.4, 0.15, 1.6, ox - 4.6, 3.3 + STUD_H, oz - 7.4, false);
        for s in [-1.0f32, 1.0] {
            self.brick(C_RED, 0.7, 0.1, 0.7, ox - 4.6 + s * 0.6, 3.45 + STUD_H, oz - 7.4, false);
        }
        // dining table and chairs
        self.furn(C_RED_BROWN, 5.0, 0.4, 3.0, ox - 6.0, 2.4, oz - 1.0, true);
        for l in [[-2.1, -1.1], [2.1, -1.1], [-2.1, 1.1], [2.1, 1.1]] {
            self.brick(C_BROWN, 0.4, 2.0, 0.4, ox - 6.0 + l[0], 0.4, oz - 1.0 + l[1], false);
        }
        for c in [-1.0f32, 1.0] {
            for s in [-1.0f32, 1.0] {
                self.brick(C_RED, 1.6, 0.3, 1.4, ox - 6.0 + c * 1.3, 1.8, oz - 1.0 + s * 2.4, true);
                self.brick(C_RED, 1.6, 1.8, 0.3, ox - 6.0 + c * 1.3, 2.1, oz - 1.0 + s * 3.0, false);
            }
        }
        self.flower_pot_on(ox - 6.0, oz - 1.0, 2.8 + STUD_H);
        // living room: rug, sofa, TV
        self.brick(0x3d6fd1, 9.0, 0.12, 7.0, ox + 5.0, 0.4, oz + 0.5, false);
        self.brick(0x6fa0ff, 7.6, 0.13, 5.6, ox + 5.0, 0.46, oz + 0.5, false);
        self.furn(C_RED, 6.0, 1.4, 2.2, ox + 5.0, 0.4, oz + 4.8, true);
        self.brick(C_RED, 6.0, 2.2, 0.7, ox + 5.0, 1.8, oz + 5.6, false);
        for s in [-1.0f32, 1.0] {
            self.brick(0xa01408, 0.7, 2.2, 2.2, ox + 5.0 + s * 3.2, 0.4, oz + 4.8, true);
        }
        self.furn(C_BROWN, 5.0, 1.6, 1.6, ox + 5.0, 0.4, oz - 7.4, false);
        self.brick(C_BLACK, 4.2, 2.6, 0.4, ox + 5.0, 2.0, oz - 7.6, false);
        self.room_screen(ox + 5.0, 3.3, oz - 7.3, 3.8, 2.1, 0x3da0ff);
        self.brick(C_GRAY, 1.2, 0.4, 0.8, ox + 5.0, 2.0, oz - 6.9, true);
        self.furn(C_RED_BROWN, 3.0, 1.2, 1.6, ox + 5.0, 0.4, oz + 1.2, true);
        // bookshelf and the stairs up to your room (walkable steps, a banister on the open side)
        self.room_bookshelf(ox + 10.6, oz - 8.0, 3.0);
        // the real steps are walked on (one box collider each): the engine's controller climbs steps shallower than its
        // capsule since 3edabf4 (before, a sloped collider through the steps' tops stood in for them)
        for i in 0..6 {
            let (z, top) = (oz + 1.6 - i as f32, 0.4 + 0.6 * (i + 1) as f32);
            self.brick(0xc8955c, 3.0, top - 0.4, 1.0, ox + 10.6, 0.4, z, true);
            self.solids.push(Solid { r: Rect::new(ox + 9.1, ox + 12.1, z - 0.5, z + 0.5), y0: 0.0, h: top * STUD });
        }
        for i in 0..6 {
            self.brick(C_WHITE, 0.25, 1.6, 0.25, ox + 9.0, 0.4 + 0.6 * (i + 1) as f32, oz + 1.6 - i as f32, false);
        }
        self.brick_rot(C_RED_BROWN, Vec3::new(0.35, 0.3, 6.6), Vec3::new(ox + 9.0, 0.4 + 0.6 * 3.5 + 1.6, oz - 1.0), Vec3::new(0.6f32.atan2(1.0), 0.0, 0.0));
        self.block(ox + 8.7, ox + 9.1, oz - 4.4, oz + 1.1);
        self.block(ox + 9.0, ox + 12.2, oz - 7.2, oz - 4.4);
        self.prop("assets/signs/room_stairs.glb", ox + 12.9, oz - 1.0, 5.2, 1.0, -std::f32::consts::FRAC_PI_2);
        self.room_plant(ox - 11.6, oz + 6.8, 2.2, false);
        self.room_plant(ox + 11.6, oz + 6.8, 2.2, true);
        self.room_plant(ox - 1.2, oz - 7.6, 1.8, false);
    }

    fn room_bedroom(&mut self) {
        let r = room_def(RoomId::Bedroom);
        let (ox, oz) = (r.ox, r.oz);
        self.room_shell(&r, 0xc8955c, 0xb88550, 0xd8ecff, C_BLUE, false, false);
        self.room_window(&r, -1.5, 4.0, 3.0);
        self.room_frame(&r, -6.5, 5.2, C_YELLOW);
        self.room_frame(&r, 3.4, 5.2, C_RED);
        // bed with a blanket in your jacket's teal
        self.furn(C_RED_BROWN, 4.0, 1.2, 6.6, ox - 6.4, 0.4, oz - 3.4, false);
        self.brick(C_BROWN, 4.0, 3.0, 0.4, ox - 6.4, 0.4, oz - 6.7, false);
        self.brick(C_WHITE, 3.6, 0.6, 5.9, ox - 6.4, 1.6, oz - 3.45, false);
        self.brick(0x1b9e8f, 3.8, 0.7, 4.0, ox - 6.4, 1.6, oz - 2.2, false);
        self.brick(C_WHITE, 2.6, 0.5, 1.3, ox - 6.4, 2.2, oz - 5.6, false);
        // desk, PC and chair
        self.furn(C_WHITE, 4.6, 2.4, 2.0, ox + 1.4, 0.4, oz - 5.9, true);
        self.room_screen(ox + 1.4, 4.0, oz - 6.5, 2.6, 1.6, 0x3da0ff);
        self.brick(C_BLACK, 1.8, 0.15, 0.6, ox + 1.4, 2.8 + STUD_H, oz - 5.4, false);
        self.brick(C_RED, 1.5, 1.4, 1.5, ox + 1.4, 0.4, oz - 3.9, true);
        self.brick(C_RED, 1.5, 1.8, 0.3, ox + 1.4, 1.8, oz - 3.3, false);
        // wardrobe
        self.furn(C_RED_BROWN, 3.2, 6.4, 1.8, ox + 6.6, 0.4, oz - 5.8, false);
        for s in [-1.0f32, 1.0] {
            self.brick(C_YELLOW, 0.2, 0.9, 0.2, ox + 6.6 + s * 0.35, 3.4, oz - 4.85, false);
        }
        // rug and a toy shelf with little brick beasts
        self.brick(C_YELLOW, 6.0, 0.12, 4.4, ox - 1.2, 0.4, oz + 1.6, false);
        self.brick(C_ORANGE, 5.0, 0.13, 3.4, ox - 1.2, 0.46, oz + 1.6, false);
        self.furn(C_BROWN, 1.4, 3.2, 4.0, ox - 8.2, 0.4, oz + 3.4, true);
        for (i, c) in [C_RED, C_BLUE, C_DARK_GREEN, C_YELLOW, 0x7b2fbe].into_iter().enumerate() {
            self.brick(c, 0.8, 0.8, 0.6, ox - 8.2, 3.6 + STUD_H, oz + 1.8 + i as f32 * 0.8, true);
        }
        self.room_plant(ox - 3.4, oz - 5.8, 1.8, false);
        // the stairwell down: a dark opening, steps fading into the dark, railings
        let wl = BED_WELL;
        let cx = (wl.x0 + wl.x1) * 0.5;
        self.brick(0x151f27, wl.x1 - wl.x0, 0.06, wl.z1 - wl.z0, cx, 0.4, (wl.z0 + wl.z1) * 0.5, false);
        for (i, c) in [0xc8955c, 0xa87a48, 0x7a5834, 0x4d3822].into_iter().enumerate() {
            self.brick(c, wl.x1 - wl.x0 - 0.2, 0.05, 0.8, cx, 0.43, wl.z0 + 0.5 + i as f32 * 0.95, false);
        }
        self.brick(C_WHITE, 0.3, 1.6, wl.z1 - wl.z0 + 0.3, wl.x0 - 0.15, 0.4, (wl.z0 + wl.z1) * 0.5, false);
        self.brick(C_WHITE, wl.x1 - wl.x0 + 0.3, 1.6, 0.3, cx, 0.4, wl.z1 + 0.15, false);
        self.block(wl.x0 - 0.3, wl.x0, wl.z0, wl.z1);
        self.block(wl.x0 - 0.3, wl.x1, wl.z1, wl.z1 + 0.3);
    }

    fn room_dex(&mut self) {
        let r = room_def(RoomId::Dex);
        let (ox, oz) = (r.ox, r.oz);
        self.room_shell(&r, C_TAN, 0xd8c090, 0xd8e8f4, C_BLUE, false, true);
        self.room_window(&r, -7.0, 4.0, 3.0);
        self.room_window(&r, 7.0, 4.0, 3.0);
        self.room_sign(&r, "assets/signs/room_dex.glb", 0.0, 7.0);
        // trophy shelf with golden cups
        self.brick(C_BROWN, 8.0, 0.4, 1.4, ox, 3.6, oz - 7.8, false);
        for i in 0..4 {
            let x = ox - 3.0 + i as f32 * 2.0;
            self.cyl(C_YELLOW, x, 4.0, oz - 7.8, 0.35, 0.3, 0.3, 10);
            self.cyl(C_YELLOW, x, 4.3, oz - 7.8, 0.2, 0.45, 0.9, 12);
        }
        // the piano
        self.furn(C_BLACK, 5.0, 2.6, 2.0, ox - 9.0, 0.4, oz - 7.2, false);
        self.brick(C_BLACK, 5.0, 2.2, 0.6, ox - 9.0, 3.0, oz - 7.9, false);
        for i in 0..9 {
            self.brick(C_WHITE, 0.45, 0.2, 0.8, ox - 11.0 + i as f32 * 0.5, 3.0, oz - 6.5, false);
        }
        self.furn(C_BLACK, 2.4, 1.2, 1.2, ox - 9.0, 0.4, oz - 5.0, true);
        // gaming corner
        self.brick(C_WHITE, 5.0, 0.4, 2.2, ox + 9.0, 2.6, oz - 7.2, true);
        self.block(ox + 6.5, ox + 11.5, oz - 8.3, oz - 6.1);
        for s in [-1.0f32, 1.0] {
            self.brick(C_WHITE, 0.4, 2.2, 2.0, ox + 9.0 + s * 2.2, 0.4, oz - 7.2, false);
        }
        self.room_screen(ox + 9.0, 4.3, oz - 7.8, 3.0, 1.8, 0xff6ad5);
        self.brick(0x7b2fbe, 1.8, 1.5, 1.8, ox + 9.0, 0.4, oz - 4.8, true);
        self.brick(0x7b2fbe, 1.8, 2.0, 0.4, ox + 9.0, 1.9, oz - 4.0, false);
        // couch, rug, table, two small beds
        self.brick(0x7b2fbe, 10.0, 0.12, 6.0, ox, 0.4, oz - 0.5, false);
        self.furn(C_DARK_GREEN, 6.0, 1.4, 2.2, ox, 0.4, oz + 3.2, true);
        self.brick(C_DARK_GREEN, 6.0, 2.2, 0.7, ox, 1.8, oz + 4.0, false);
        self.furn(C_WHITE, 3.4, 1.2, 2.0, ox, 0.4, oz - 0.9, true);
        self.flower_pot_on(ox, oz - 0.9, 1.6 + STUD_H);
        for s in [-1.0f32, 1.0] {
            self.furn(if s < 0.0 { 0xff9ecd } else { 0x3d6fd1 }, 3.0, 1.2, 5.0, ox + s * 11.0, 0.4, oz + 3.5, true);
            self.brick(C_WHITE, 2.4, 0.5, 1.2, ox + s * 11.0, 1.6, oz + 1.6, false);
        }
        self.room_plant(ox - 4.0, oz - 7.4, 1.8, false);
        self.room_plant(ox + 4.5, oz - 7.4, 1.8, true);
    }

    fn room_lab(&mut self) {
        let r = room_def(RoomId::Lab);
        let (ox, oz) = (r.ox, r.oz);
        self.room_shell(&r, C_WHITE, 0xd8dde0, C_WHITE, 0x3d6fd1, true, true);
        self.room_sign(&r, "assets/signs/room_lab.glb", 0.0, 7.4);
        self.room_window(&r, -4.0, 3.0, 2.4);
        self.room_window(&r, 4.0, 3.0, 2.4);
        self.room_bookshelf(ox - 13.0, oz - 10.0, 5.0);
        self.room_bookshelf(ox - 7.6, oz - 10.0, 5.0);
        // machines with screens and lights (the middle one is the Deck-o-Matic)
        let glow = [0x40e0a0, 0xffd23d, 0xff6a5a];
        let lights = [0xff3a3a, 0x3aff6a, 0x3a9aff];
        for i in 0..3 {
            let x = ox + 8.0 + i as f32 * 3.6;
            self.furn(C_GRAY, 3.2, 6.0, 2.2, x, 0.4, oz - 9.6, false);
            self.room_screen(x, 4.4, oz - 8.4, 2.2, 1.4, glow[i]);
            for (j, l) in lights.into_iter().enumerate() {
                self.brick(l, 0.4, 0.4, 0.2, x - 0.8 + j as f32 * 0.8, 2.2, oz - 8.4, false);
            }
        }
        // aquariums along the left wall
        for i in 0..2 {
            let z = oz - 2.0 + i as f32 * 6.0;
            self.furn(C_BLACK, 2.4, 1.6, 4.6, ox - 15.4, 0.4, z, false);
            self.brick(0x4ab0ff, 2.2, 3.0, 4.4, ox - 15.4, 2.0, z, false);
            self.brick(C_TAN, 2.25, 0.6, 4.45, ox - 15.4, 2.0, z, false);
            self.brick(C_ORANGE, 0.7, 0.35, 0.3, ox - 15.4 + 1.11, 3.4, z + 0.6, false);
            self.brick(C_ORANGE, 0.7, 0.35, 0.3, ox - 15.4 + 1.11, 3.9, z - 0.9, false);
        }
        // computer desks along the right wall
        for i in 0..2 {
            let z = oz - 2.0 + i as f32 * 5.5;
            self.furn(C_WHITE, 2.6, 0.4, 4.0, ox + 15.2, 2.6, z, true);
            for s in [-1.0f32, 1.0] {
                self.brick(C_WHITE, 2.4, 2.2, 0.3, ox + 15.2, 0.4, z + s * 1.8, false);
            }
            // the LEGO office computer of the three.js version, facing +z like there
            self.prop("assets/props/lab_computer.glb", ox + 15.2, z - 0.2, 3.0, 1.2, 0.0);
        }
        // the central table with three glowing pedestals
        self.furn(C_RED_BROWN, 10.0, 0.5, 3.6, ox, 2.6, oz - 1.2, true);
        for l in [[-4.5, -1.4], [4.5, -1.4], [-4.5, 1.4], [4.5, 1.4]] {
            self.brick(C_BROWN, 0.5, 2.2, 0.5, ox + l[0], 0.4, oz - 1.2 + l[1], false);
        }
        for (i, c) in [0xff7a3d, 0x3d9bff, 0x5ccb4c].into_iter().enumerate() {
            let x = ox - 3.0 + i as f32 * 3.0;
            self.brick(c, 1.6, 0.6, 1.6, x, 3.1 + STUD_H, oz - 1.2, false);
            self.brick(C_WHITE, 1.3, 0.06, 1.0, x, 3.7 + STUD_H, oz - 1.2, false); // a starter card
        }
        self.flower_pot_on(ox - 15.4, oz + 8.4, 0.4);
        self.block(ox - 17.2, ox - 13.6, oz + 6.6, oz + 10.2);
        self.room_plant(ox + 15.4, oz + 8.6, 2.2, true);
        self.room_plant(ox + 1.2, oz - 9.8, 1.8, false);
    }
}
