// The rooms (three.js interiors.ts): built once far east of the village (x >= 200 studs) into the town's mesh; going
// through a door is a teleport (st_doors in starting_template.c), so there is no loading. The rooms are much bigger
// than the houses look from outside, open at the top and low at the front so the camera looks in from the south.
// Included by starting_template.c after st_town.c; built from town_build.

typedef enum RoomId { ROOM_HOME, ROOM_DEX, ROOM_LAB, ROOM_BEDROOM, ROOM_COUNT } RoomId;

typedef struct RoomDef {
  CString label;
  f32 ox, oz, w, d;      // centre and size, studs
  f32 door_x, door_z;    // the outside door face (studs); upstairs rooms have none
  b32 upstairs;
} RoomDef;

hz_internal RoomDef room_def(RoomId id) {
  switch (id) {
  case ROOM_HOME: return (RoomDef){"Home", 200.0f, 0.0f, 26.0f, 18.0f, -15.0f, -6.0f, false};
  case ROOM_DEX: return (RoomDef){"Dex & Lumi's House", 260.0f, 0.0f, 26.0f, 18.0f, 47.0f, -6.0f, false};
  case ROOM_LAB: return (RoomDef){"Quill Lab", 330.0f, 0.0f, 34.0f, 22.0f, 13.0f, 16.0f, false};
  default: return (RoomDef){"Your Room", 200.0f, 40.0f, 18.0f, 14.0f, 0.0f, 0.0f, true};
  }
}

// the stairs in the home (ascending towards -z, the top leads up) and the stairwell in the bedroom (leads down)
#define HOME_STAIRS ((TownRect){209.1f, 212.1f, -4.4f, 2.1f})
#define BED_WELL ((TownRect){205.5f, 208.5f, 42.0f, 46.0f})

// furniture: a brick that also blocks walking
hz_internal void room_furn(Town *t, u32 hex, f32 w, f32 h, f32 d, f32 x, f32 y, f32 z, b32 studs) {
  town_box(t, hex, w, h, d, x, y, z, studs);
  town_block(t, x - w * 0.5f, x + w * 0.5f, z - d * 0.5f, z + d * 0.5f);
}

// a brick the figure stands on (stairs): a box collider of its own size
hz_internal void room_solid(Town *t, u32 hex, f32 w, f32 h, f32 d, f32 x, f32 y, f32 z) {
  MeshBuilderStyle st = town_style(hex);
  st.collider = MESH_BUILDER_COLLIDER_BOX;
  mesh_builder_box(t->b, &st, m4_trs(town_m(x, y + h * 0.5f, z), quat_identity(), V3_ONE), town_m(w, h, d));
  town_stud_rect(t, hex, x - w * 0.5f, x + w * 0.5f, z - d * 0.5f, z + d * 0.5f, y + h);
}

// floor (planks or checker tiles), back and side walls with a wainscot, a low front wall with the door gap
hz_internal void room_shell(Town *t, RoomDef r, u32 floor_a, u32 floor_b, u32 wall, u32 trim, b32 tiles, b32 door) {
  f32 ox = r.ox, oz = r.oz, W = r.w, D = r.d;
  // a dark floor around the room (instead of the baseplate)
  town_box(t, 0x151f27u, W + 60.0f, 0.05f, D + 50.0f, ox, 0.0f, oz + 6.0f, false);
  if (tiles) {
    for (f32 x = -W * 0.5f; x < W * 0.5f; x += 2.0f)
      for (f32 z = -D * 0.5f; z < D * 0.5f; z += 2.0f)
        town_box(t, ((i32)((x + z) * 0.5f) & 1) ? floor_b : floor_a, 2.0f, 0.4f, 2.0f, ox + x + 1.0f, 0.0f, oz + z + 1.0f, false);
  } else {
    for (f32 x = -W * 0.5f; x < W * 0.5f; x += 1.0f)
      town_box(t, ((i32)m_absf(x) & 1) ? floor_b : floor_a, 1.0f, 0.4f, D, ox + x + 0.5f, 0.0f, oz, false);
  }
  // the floor is walked on: one collider under the whole room
  mesh_builder_collider_box(t->b, town_m(ox, 0.2f, oz), quat_identity(), town_m(W * 0.5f, 0.2f, D * 0.5f));
  const f32 H = 9.0f;
  room_furn(t, wall, W + 2.0f, H, 1.0f, ox, 0.0f, oz - D * 0.5f - 0.5f, false);
  town_box(t, trim, W, 2.4f, 0.3f, ox, 0.4f, oz - D * 0.5f + 0.1f, false);
  town_box(t, trim, W + 2.0f, 0.4f, 1.4f, ox, H, oz - D * 0.5f - 0.5f, false);
  for (i32 s = -1; s <= 1; s += 2) {
    room_furn(t, wall, 1.0f, H, D + 1.0f, ox + (f32)s * (W * 0.5f + 0.5f), 0.0f, oz - 0.5f, false);
    town_box(t, trim, 0.3f, 2.4f, D, ox + (f32)s * (W * 0.5f - 0.1f), 0.4f, oz, false);
  }
  if (door) {
    f32 seg = W * 0.5f - 1.6f;
    for (i32 s = -1; s <= 1; s += 2)
      room_furn(t, wall, seg + 1.0f, 1.2f, 1.0f, ox + (f32)s * (1.6f + seg * 0.5f + 0.5f), 0.0f, oz + D * 0.5f + 0.5f, true);
    town_box(t, C_RED, 3.0f, 0.1f, 1.6f, ox, 0.4f, oz + D * 0.5f - 0.6f, false); // door mat
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, trim, 0.5f, 1.6f, 1.2f, ox + (f32)s * 1.8f, 0.0f, oz + D * 0.5f + 0.5f, true);
    // beyond the gap: you leave through the door before you get here
    town_block(t, ox - 2.0f, ox + 2.0f, oz + D * 0.5f + 1.6f, oz + D * 0.5f + 2.6f);
  } else {
    room_furn(t, wall, W + 2.0f, 1.2f, 1.0f, ox, 0.0f, oz + D * 0.5f + 0.5f, true);
  }
}

hz_internal void room_window(Town *t, RoomDef r, f32 x, f32 w, f32 h) {
  f32 z = r.oz - r.d * 0.5f;
  town_box(t, C_WHITE, w + 0.6f, h + 0.6f, 0.3f, r.ox + x, 3.2f, z + 0.1f, false);
  town_box(t, 0xbfe9ffu, w, h, 0.3f, r.ox + x, 3.5f, z + 0.2f, false);
  town_box(t, C_WHITE, 0.2f, h, 0.35f, r.ox + x, 3.5f, z + 0.3f, false);
}

hz_internal void room_frame(Town *t, RoomDef r, f32 x, f32 y, u32 hex) {
  f32 z = r.oz - r.d * 0.5f;
  town_box(t, C_BROWN, 2.2f, 1.8f, 0.2f, r.ox + x, y, z + 0.1f, false);
  town_box(t, hex, 1.8f, 1.4f, 0.25f, r.ox + x, y + 0.2f, z + 0.15f, false);
}

// a sign board on the back wall (the painted strip from tools/make_signs.py)
hz_internal void room_sign(Town *t, RoomDef r, CString path, f32 x, f32 y) {
  town_prop(t, path, r.ox + x, r.oz - r.d * 0.5f + 0.07f, y, 1.0f, 0.0f, false);
}

// a potted plant: a brick pot with a LEGO bush or bamboo leaves on top
hz_internal void room_plant(Town *t, f32 x, f32 z, f32 s, b32 bamboo) {
  town_box(t, C_RED_BROWN, 1.4f, 1.2f, 1.4f, x, 0.4f, z, false);
  town_prop(t, bamboo ? PROP("bamboo") : PROP("bush"), x, z, 1.6f, bamboo ? s * 0.75f : s, town_rot(t), false);
  town_block(t, x - 0.8f, x + 0.8f, z - 0.8f, z + 0.8f);
}

hz_internal void room_books(Town *t, f32 x, f32 z, f32 w, f32 h, f32 y0) {
  u32 cols[] = {C_RED, C_BLUE, C_YELLOW, C_DARK_GREEN, C_WHITE, C_ORANGE};
  i32 n = (i32)(w / 0.5f);
  for (i32 i = 0; i < n; i++)
    town_box(t, cols[(i * 7) % 6], 0.4f, h * (0.7f + town_rand(&t->rng) * 0.3f), 0.9f, x - w * 0.5f + 0.25f + (f32)i * 0.5f, y0, z, false);
}

hz_internal void room_bookshelf(Town *t, f32 x, f32 z, f32 w) {
  room_furn(t, C_BROWN, w, 7.0f, 1.4f, x, 0.4f, z, false);
  for (i32 i = 0; i < 4; i++) {
    town_box(t, C_RED_BROWN, w - 0.4f, 0.2f, 1.2f, x, 1.2f + (f32)i * 1.6f, z + 0.15f, false);
    room_books(t, x, z + 0.3f, w - 0.6f, 1.2f, 1.4f + (f32)i * 1.6f);
  }
}

// a screen: a dark frame with a bright face
hz_internal void room_screen(Town *t, f32 x, f32 y, f32 z, f32 w, f32 h, u32 glow) {
  town_box(t, C_BLACK, w + 0.3f, h + 0.3f, 0.3f, x, y - 0.15f, z, false);
  town_box(t, glow, w, h, 0.32f, x, y, z + 0.03f, false);
}

// warm ceiling lights in every room (point lights; the rooms lie far from the village and from each other)
hz_internal void room_light(EcsWorld *w, f32 x, f32 z, f32 intensity) {
  EcsEntity e = ecs_entity_create(w);
  v3 at = town_m(x, 8.5f, z);
  ecs_component_set(w, e, LocalTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, WorldTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, EcsPointLight, {.color = {1.0f, 0.84f, 0.62f, 1.0f}, .radius = 9.0f, .intensity = intensity});
}

hz_internal void rooms_lights(EcsWorld *w) {
  for (i32 i = 0; i < ROOM_COUNT; i++) {
    RoomDef r = room_def((RoomId)i);
    f32 n = r.w > 30.0f ? 3.0f : 2.0f;
    for (f32 k = 0; k < n; k++) {
      f32 x = r.ox - r.w * 0.5f + r.w * (k + 0.5f) / n;
      room_light(w, x, r.oz - r.d * 0.15f, 600000.0f);
    }
    room_light(w, r.ox, r.oz + r.d * 0.3f, 350000.0f);
  }
}

hz_internal void rooms_build(Town *t) {

  // ======================= HOME =======================
  town_group_switch(t, 1 + ROOM_HOME);
  {
    RoomDef r = room_def(ROOM_HOME);
    f32 ox = r.ox, oz = r.oz;
    room_shell(t, r, 0xc8955cu, 0xb88550u, 0xf4e6c8u, C_RED_BROWN, false, true);
    room_window(t, r, -4.0f, 4.0f, 3.0f);
    room_window(t, r, 5.0f, 4.0f, 3.0f);
    room_frame(t, r, 0.6f, 5.0f, 0x7fc0ffu);
    room_frame(t, r, 9.5f, 5.0f, 0xffb070u);
    // kitchen (back left): fridge, counter, stove
    room_furn(t, C_WHITE, 2.6f, 6.4f, 2.2f, ox - 11.4f, 0.4f, oz - 7.4f, false);
    town_box(t, C_GRAY, 0.2f, 2.0f, 0.2f, ox - 10.4f, 3.4f, oz - 6.2f, false);
    room_furn(t, C_WHITE, 7.0f, 2.6f, 2.2f, ox - 6.6f, 0.4f, oz - 7.4f, false);
    town_box(t, C_DARK_GRAY, 7.2f, 0.3f, 2.4f, ox - 6.6f, 3.0f, oz - 7.4f, true);
    town_box(t, C_BLACK, 2.4f, 0.15f, 1.6f, ox - 4.6f, 3.3f, oz - 7.4f, false);
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, C_RED, 0.7f, 0.1f, 0.7f, ox - 4.6f + (f32)s * 0.6f, 3.45f, oz - 7.4f, false);
    // dining table and chairs
    room_furn(t, C_RED_BROWN, 5.0f, 0.4f, 3.0f, ox - 6.0f, 2.4f, oz - 1.0f, true);
    f32 legs[4][2] = {{-2.1f, -1.1f}, {2.1f, -1.1f}, {-2.1f, 1.1f}, {2.1f, 1.1f}};
    for (i32 i = 0; i < 4; i++)
      town_box(t, C_BROWN, 0.4f, 2.0f, 0.4f, ox - 6.0f + legs[i][0], 0.4f, oz - 1.0f + legs[i][1], false);
    for (i32 c = -1; c <= 1; c += 2)
      for (i32 s = -1; s <= 1; s += 2) {
        town_box(t, C_RED, 1.6f, 0.3f, 1.4f, ox - 6.0f + (f32)c * 1.3f, 1.8f, oz - 1.0f + (f32)s * 2.4f, true);
        town_box(t, C_RED, 1.6f, 1.8f, 0.3f, ox - 6.0f + (f32)c * 1.3f, 2.1f, oz - 1.0f + (f32)s * 3.0f, false);
      }
    town_flower_pot_on(t, ox - 6.0f, oz - 1.0f, 2.8f + TOWN_STUD_H);
    // living room: rug, sofa, TV
    town_box(t, 0x3d6fd1u, 9.0f, 0.12f, 7.0f, ox + 5.0f, 0.4f, oz + 0.5f, false);
    town_box(t, 0x6fa0ffu, 7.6f, 0.13f, 5.6f, ox + 5.0f, 0.41f, oz + 0.5f, false);
    room_furn(t, C_RED, 6.0f, 1.4f, 2.2f, ox + 5.0f, 0.4f, oz + 4.8f, true);
    town_box(t, C_RED, 6.0f, 2.2f, 0.7f, ox + 5.0f, 1.8f, oz + 5.6f, false);
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, 0xa01408u, 0.7f, 2.2f, 2.2f, ox + 5.0f + (f32)s * 3.2f, 0.4f, oz + 4.8f, true);
    room_furn(t, C_BROWN, 5.0f, 1.6f, 1.6f, ox + 5.0f, 0.4f, oz - 7.4f, false);
    town_box(t, C_BLACK, 4.2f, 2.6f, 0.4f, ox + 5.0f, 2.0f, oz - 7.6f, false);
    room_screen(t, ox + 5.0f, 3.3f, oz - 7.3f, 3.8f, 2.1f, 0x3da0ffu);
    town_box(t, C_GRAY, 1.2f, 0.4f, 0.8f, ox + 5.0f, 2.0f, oz - 6.9f, true);
    room_furn(t, C_RED_BROWN, 3.0f, 1.2f, 1.6f, ox + 5.0f, 0.4f, oz + 1.2f, true);
    // bookshelf and the stairs up to your room (walkable steps, a banister on the open side)
    room_bookshelf(t, ox + 10.6f, oz - 8.0f, 3.0f);
    for (i32 i = 0; i < 6; i++)
      room_solid(t, 0xc8955cu, 3.0f, 0.6f * (f32)(i + 1), 1.0f, ox + 10.6f, 0.4f, oz + 1.6f - (f32)i);
    for (i32 i = 0; i < 6; i++)
      town_box(t, C_WHITE, 0.25f, 1.6f, 0.25f, ox + 9.0f, 0.4f + 0.6f * (f32)(i + 1), oz + 1.6f - (f32)i, false);
    town_box_rot(t, C_RED_BROWN, (v3){0.35f, 0.3f, 6.6f}, (v3){ox + 9.0f, 0.4f + 0.6f * 3.5f + 1.6f, oz - 1.0f},
                 (v3){m_atan2f(0.6f, 1.0f), 0.0f, 0.0f});
    town_block(t, ox + 8.7f, ox + 9.1f, oz - 4.4f, oz + 1.1f);
    town_block(t, ox + 9.0f, ox + 12.2f, oz - 7.2f, oz - 4.4f);
    town_prop(t, "brickhollow/signs/room_stairs.glb", ox + 12.9f, oz - 1.0f, 5.2f, 1.0f, -M_PI * 0.5f, false);
    room_plant(t, ox - 11.6f, oz + 6.8f, 2.2f, false);
    room_plant(t, ox + 11.6f, oz + 6.8f, 2.2f, true);
    room_plant(t, ox - 1.2f, oz - 7.6f, 1.8f, false);
  }

  // ======================= BEDROOM (upstairs at home) =======================
  town_group_switch(t, 1 + ROOM_BEDROOM);
  {
    RoomDef r = room_def(ROOM_BEDROOM);
    f32 ox = r.ox, oz = r.oz;
    room_shell(t, r, 0xc8955cu, 0xb88550u, 0xd8ecffu, C_BLUE, false, false);
    room_window(t, r, -1.5f, 4.0f, 3.0f);
    room_frame(t, r, -6.5f, 5.2f, C_YELLOW);
    room_frame(t, r, 3.4f, 5.2f, C_RED);
    // bed with a blanket in your jacket's teal
    room_furn(t, C_RED_BROWN, 4.0f, 1.2f, 6.6f, ox - 6.4f, 0.4f, oz - 3.4f, false);
    town_box(t, C_BROWN, 4.0f, 3.0f, 0.4f, ox - 6.4f, 0.4f, oz - 6.7f, false);
    town_box(t, C_WHITE, 3.6f, 0.6f, 5.9f, ox - 6.4f, 1.6f, oz - 3.45f, false);
    town_box(t, 0x1b9e8fu, 3.8f, 0.7f, 4.0f, ox - 6.4f, 1.6f, oz - 2.2f, false);
    town_box(t, C_WHITE, 2.6f, 0.5f, 1.3f, ox - 6.4f, 2.2f, oz - 5.6f, false);
    // desk, PC and chair
    room_furn(t, C_WHITE, 4.6f, 2.4f, 2.0f, ox + 1.4f, 0.4f, oz - 5.9f, true);
    room_screen(t, ox + 1.4f, 4.0f, oz - 6.5f, 2.6f, 1.6f, 0x3da0ffu);
    town_box(t, C_BLACK, 1.8f, 0.15f, 0.6f, ox + 1.4f, 2.8f, oz - 5.4f, false);
    town_box(t, C_RED, 1.5f, 1.4f, 1.5f, ox + 1.4f, 0.4f, oz - 3.9f, true);
    town_box(t, C_RED, 1.5f, 1.8f, 0.3f, ox + 1.4f, 1.8f, oz - 3.3f, false);
    // wardrobe
    room_furn(t, C_RED_BROWN, 3.2f, 6.4f, 1.8f, ox + 6.6f, 0.4f, oz - 5.8f, false);
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, C_YELLOW, 0.2f, 0.9f, 0.2f, ox + 6.6f + (f32)s * 0.35f, 3.4f, oz - 4.85f, false);
    // rug and a toy shelf with little brick beasts
    town_box(t, C_YELLOW, 6.0f, 0.12f, 4.4f, ox - 1.2f, 0.4f, oz + 1.6f, false);
    town_box(t, C_ORANGE, 5.0f, 0.13f, 3.4f, ox - 1.2f, 0.41f, oz + 1.6f, false);
    room_furn(t, C_BROWN, 1.4f, 3.2f, 4.0f, ox - 8.2f, 0.4f, oz + 3.4f, true);
    u32 toys[] = {C_RED, C_BLUE, C_DARK_GREEN, C_YELLOW, 0x7b2fbeu};
    for (i32 i = 0; i < 5; i++)
      town_box(t, toys[i], 0.8f, 0.8f, 0.6f, ox - 8.2f, 3.6f, oz + 1.8f + (f32)i * 0.8f, true);
    room_plant(t, ox - 3.4f, oz - 5.8f, 1.8f, false);
    // the stairwell down: a dark opening, steps fading into the dark, railings
    TownRect wl = BED_WELL;
    f32 cx = (wl.x0 + wl.x1) * 0.5f;
    town_box(t, 0x151f27u, wl.x1 - wl.x0, 0.06f, wl.z1 - wl.z0, cx, 0.4f, (wl.z0 + wl.z1) * 0.5f, false);
    u32 shades[] = {0xc8955cu, 0xa87a48u, 0x7a5834u, 0x4d3822u};
    for (i32 i = 0; i < 4; i++)
      town_box(t, shades[i], wl.x1 - wl.x0 - 0.2f, 0.05f, 0.8f, cx, 0.43f, wl.z0 + 0.5f + (f32)i * 0.95f, false);
    town_box(t, C_WHITE, 0.3f, 1.6f, wl.z1 - wl.z0 + 0.3f, wl.x0 - 0.15f, 0.4f, (wl.z0 + wl.z1) * 0.5f, false);
    town_box(t, C_WHITE, wl.x1 - wl.x0 + 0.3f, 1.6f, 0.3f, cx, 0.4f, wl.z1 + 0.15f, false);
    town_block(t, wl.x0 - 0.3f, wl.x0, wl.z0, wl.z1);
    town_block(t, wl.x0 - 0.3f, wl.x1, wl.z1, wl.z1 + 0.3f);
  }

  // ======================= DEX & LUMI =======================
  town_group_switch(t, 1 + ROOM_DEX);
  {
    RoomDef r = room_def(ROOM_DEX);
    f32 ox = r.ox, oz = r.oz;
    room_shell(t, r, C_TAN, 0xd8c090u, 0xd8e8f4u, C_BLUE, false, true);
    room_window(t, r, -7.0f, 4.0f, 3.0f);
    room_window(t, r, 7.0f, 4.0f, 3.0f);
    room_sign(t, r, "brickhollow/signs/room_dex.glb", 0.0f, 7.0f);
    // trophy shelf with golden cups
    town_box(t, C_BROWN, 8.0f, 0.4f, 1.4f, ox, 3.6f, oz - 7.8f, false);
    for (i32 i = 0; i < 4; i++) {
      town_cyl(t, C_YELLOW, ox - 3.0f + (f32)i * 2.0f, 4.0f, oz - 7.8f, 0.35f, 0.3f, 0.3f, 10, true);
      town_cyl(t, C_YELLOW, ox - 3.0f + (f32)i * 2.0f, 4.3f, oz - 7.8f, 0.2f, 0.45f, 0.9f, 12, true);
    }
    // the piano
    room_furn(t, C_BLACK, 5.0f, 2.6f, 2.0f, ox - 9.0f, 0.4f, oz - 7.2f, false);
    town_box(t, C_BLACK, 5.0f, 2.2f, 0.6f, ox - 9.0f, 3.0f, oz - 7.9f, false);
    for (i32 i = 0; i < 9; i++)
      town_box(t, C_WHITE, 0.45f, 0.2f, 0.8f, ox - 11.0f + (f32)i * 0.5f, 3.0f, oz - 6.5f, false);
    room_furn(t, C_BLACK, 2.4f, 1.2f, 1.2f, ox - 9.0f, 0.4f, oz - 5.0f, true);
    // gaming corner
    town_box(t, C_WHITE, 5.0f, 0.4f, 2.2f, ox + 9.0f, 2.6f, oz - 7.2f, true);
    town_block(t, ox + 6.5f, ox + 11.5f, oz - 8.3f, oz - 6.1f);
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, C_WHITE, 0.4f, 2.2f, 2.0f, ox + 9.0f + (f32)s * 2.2f, 0.4f, oz - 7.2f, false);
    room_screen(t, ox + 9.0f, 4.3f, oz - 7.8f, 3.0f, 1.8f, 0xff6ad5u);
    town_box(t, 0x7b2fbeu, 1.8f, 1.5f, 1.8f, ox + 9.0f, 0.4f, oz - 4.8f, true);
    town_box(t, 0x7b2fbeu, 1.8f, 2.0f, 0.4f, ox + 9.0f, 1.9f, oz - 4.0f, false);
    // couch, rug, table, two small beds
    town_box(t, 0x7b2fbeu, 10.0f, 0.12f, 6.0f, ox, 0.4f, oz - 0.5f, false);
    room_furn(t, C_DARK_GREEN, 6.0f, 1.4f, 2.2f, ox, 0.4f, oz + 3.2f, true);
    town_box(t, C_DARK_GREEN, 6.0f, 2.2f, 0.7f, ox, 1.8f, oz + 4.0f, false);
    room_furn(t, C_WHITE, 3.4f, 1.2f, 2.0f, ox, 0.4f, oz - 0.9f, true);
    town_flower_pot_on(t, ox, oz - 0.9f, 1.6f + TOWN_STUD_H);
    for (i32 s = -1; s <= 1; s += 2) {
      room_furn(t, s < 0 ? 0xff9ecdu : 0x3d6fd1u, 3.0f, 1.2f, 5.0f, ox + (f32)s * 11.0f, 0.4f, oz + 3.5f, true);
      town_box(t, C_WHITE, 2.4f, 0.5f, 1.2f, ox + (f32)s * 11.0f, 1.6f, oz + 1.6f, false);
    }
    room_plant(t, ox - 4.0f, oz - 7.4f, 1.8f, false);
    room_plant(t, ox + 4.5f, oz - 7.4f, 1.8f, true);
  }

  // ======================= LAB =======================
  town_group_switch(t, 1 + ROOM_LAB);
  {
    RoomDef r = room_def(ROOM_LAB);
    f32 ox = r.ox, oz = r.oz;
    room_shell(t, r, C_WHITE, 0xd8dde0u, C_WHITE, 0x3d6fd1u, true, true);
    room_sign(t, r, "brickhollow/signs/room_lab.glb", 0.0f, 7.4f);
    room_window(t, r, -4.0f, 3.0f, 2.4f);
    room_window(t, r, 4.0f, 3.0f, 2.4f);
    room_bookshelf(t, ox - 13.0f, oz - 10.0f, 5.0f);
    room_bookshelf(t, ox - 7.6f, oz - 10.0f, 5.0f);
    // machines with screens and lights
    u32 glow[3] = {0x40e0a0u, 0xffd23du, 0xff6a5au};
    u32 lights[3] = {0xff3a3au, 0x3aff6au, 0x3a9affu};
    for (i32 i = 0; i < 3; i++) {
      f32 x = ox + 8.0f + (f32)i * 3.6f;
      room_furn(t, C_GRAY, 3.2f, 6.0f, 2.2f, x, 0.4f, oz - 9.6f, false);
      room_screen(t, x, 4.4f, oz - 8.4f, 2.2f, 1.4f, glow[i]);
      for (i32 j = 0; j < 3; j++)
        town_box(t, lights[j], 0.4f, 0.4f, 0.2f, x - 0.8f + (f32)j * 0.8f, 2.2f, oz - 8.4f, false);
    }
    // aquariums along the left wall
    for (i32 i = 0; i < 2; i++) {
      f32 z = oz - 2.0f + (f32)i * 6.0f;
      room_furn(t, C_BLACK, 2.4f, 1.6f, 4.6f, ox - 15.4f, 0.4f, z, false);
      town_box(t, 0x4ab0ffu, 2.2f, 3.0f, 4.4f, ox - 15.4f, 2.0f, z, false);
      town_box(t, C_TAN, 2.25f, 0.6f, 4.45f, ox - 15.4f, 2.0f, z, false);
      town_box(t, C_ORANGE, 0.7f, 0.35f, 0.3f, ox - 15.4f + 1.11f, 3.4f, z + 0.6f, false);
      town_box(t, C_ORANGE, 0.7f, 0.35f, 0.3f, ox - 15.4f + 1.11f, 3.9f, z - 0.9f, false);
    }
    // computer desks along the right wall
    for (i32 i = 0; i < 2; i++) {
      f32 z = oz - 2.0f + (f32)i * 5.5f;
      room_furn(t, C_WHITE, 2.6f, 0.4f, 4.0f, ox + 15.2f, 2.6f, z, true);
      for (i32 s = -1; s <= 1; s += 2)
        town_box(t, C_WHITE, 2.4f, 2.2f, 0.3f, ox + 15.2f, 0.4f, z + (f32)s * 1.8f, false);
      town_box(t, C_GRAY, 1.6f, 1.6f, 1.4f, ox + 15.2f, 3.0f, z - 0.2f, false);
      town_box_rot(t, 0x40c0ffu, (v3){1.3f, 0.9f, 0.1f}, (v3){ox + 15.2f - 0.75f, 3.9f, z - 0.2f}, (v3){0.0f, -M_PI * 0.5f, 0.0f});
    }
    // the central table with three glowing pedestals
    room_furn(t, C_RED_BROWN, 10.0f, 0.5f, 3.6f, ox, 2.6f, oz - 1.2f, true);
    f32 tl[4][2] = {{-4.5f, -1.4f}, {4.5f, -1.4f}, {-4.5f, 1.4f}, {4.5f, 1.4f}};
    for (i32 i = 0; i < 4; i++)
      town_box(t, C_BROWN, 0.5f, 2.2f, 0.5f, ox + tl[i][0], 0.4f, oz - 1.2f + tl[i][1], false);
    u32 ped[3] = {0xff7a3du, 0x3d9bffu, 0x5ccb4cu};
    for (i32 i = 0; i < 3; i++) {
      town_box(t, ped[i], 1.6f, 0.6f, 1.6f, ox - 3.0f + (f32)i * 3.0f, 3.1f, oz - 1.2f, false);
      town_box(t, C_WHITE, 1.3f, 0.06f, 1.0f, ox - 3.0f + (f32)i * 3.0f, 3.7f, oz - 1.2f, false); // a starter card
    }
    town_flower_pot_on(t, ox - 15.4f, oz + 8.4f, 0.4f);
    town_block(t, ox - 17.2f, ox - 13.6f, oz + 6.6f, oz + 10.2f);
    room_plant(t, ox + 15.4f, oz + 8.6f, 2.2f, true);
    room_plant(t, ox + 1.2f, oz - 9.8f, 1.8f, false);
  }
}
