// Brickhollow: the starter town, built in code with the mesh builder: houses, the lab, a pond, fences, trees,
// flowers, paths and a tree line around it. One vertex-coloured slot (the renderer's default material multiplies
// the vertex colour), static colliders from the same calls. Included by starting_template.c above app_init.

#define TOWN_SLOT 0
#define TOWN_HALF_X 19.0f // the walkable square: x in [-19, 19], z in [-21, 19]
#define TOWN_NORTH -21.0f
#define TOWN_SOUTH 19.0f

// a tiny deterministic random stream for the trees' and flowers' jitter
typedef struct TownRng {
  u32 s;
} TownRng;
hz_internal f32 town_rand(TownRng *r) {
  r->s = r->s * 1664525u + 1013904223u;
  return (f32)(r->s >> 8) / 16777216.0f;
}

hz_internal MeshBuilderStyle town_style(u32 hex, MeshBuilderColliderKind collider) {
  return (MeshBuilderStyle){.slot = TOWN_SLOT, .color = mesh_builder_hex(hex), .flat = true, .collider = collider};
}

// an axis-aligned box from its centre and size
hz_internal void town_box(MeshBuilder *b, u32 hex, v3 center, v3 size, b32 solid) {
  MeshBuilderStyle st = town_style(hex, solid ? MESH_BUILDER_COLLIDER_BOX : MESH_BUILDER_COLLIDER_NONE);
  mesh_builder_box(b, &st, m4_trs(center, quat_identity(), V3_ONE), size);
}

// a gable roof: a triangular prism along x, `width` across z, `height` from the eaves to the ridge, eaves at `base_y`
hz_internal void town_roof(MeshBuilder *b, u32 hex, v3 center_xz, f32 base_y, f32 length, f32 width, f32 height) {
  MeshBuilderStyle st = town_style(hex, MESH_BUILDER_COLLIDER_NONE);
  // a 3-sided unit cylinder turned onto x: one corner up, the triangle spans y -0.5..1 and z +-0.866
  f32 sx = height / 1.5f;
  m4 xf = m4_trs((v3){center_xz.x, base_y + 0.5f * sx, center_xz.z}, quat_from_euler((v3){0.0f, 0.0f, M_PI * 0.5f}),
                 (v3){sx, length, width / 1.732f});
  mesh_builder_cylinder(b, &st, xf,
                        &(PgCylinderDesc){.radius_top = 1.0f, .radius_bottom = 1.0f, .height = 1.0f, .radial_segments = 3,
                                          .theta_start = M_PI * 0.5f});
}

// a sign board on a post; `face` +1 faces +z, -1 faces -z
hz_internal void town_sign(MeshBuilder *b, v3 at, f32 face, u32 board_hex) {
  town_box(b, 0x6b4a2b, (v3){at.x, 0.55f, at.z}, (v3){0.14f, 1.1f, 0.14f}, true);
  town_box(b, 0x4a321c, (v3){at.x, 1.15f, at.z + face * 0.04f}, (v3){1.15f, 0.7f, 0.1f}, false);
  town_box(b, board_hex, (v3){at.x, 1.15f, at.z + face * 0.1f}, (v3){0.95f, 0.5f, 0.03f}, false);
}

// a window: frame, glass and a sill, flat on a wall facing `face` along z
hz_internal void town_window_z(MeshBuilder *b, v3 at, f32 face) {
  town_box(b, 0xffffff, (v3){at.x, at.y, at.z + face * 0.03f}, (v3){1.15f, 1.0f, 0.06f}, false);
  town_box(b, 0x8fd3ff, (v3){at.x, at.y, at.z + face * 0.07f}, (v3){0.95f, 0.8f, 0.04f}, false);
  town_box(b, 0xffffff, (v3){at.x, at.y, at.z + face * 0.09f}, (v3){0.06f, 0.8f, 0.03f}, false);
  town_box(b, 0xe8e0d0, (v3){at.x, at.y - 0.52f, at.z + face * 0.12f}, (v3){1.25f, 0.1f, 0.2f}, false);
}

// a window on a wall facing `face` along x
hz_internal void town_window_x(MeshBuilder *b, v3 at, f32 face) {
  town_box(b, 0xffffff, (v3){at.x + face * 0.03f, at.y, at.z}, (v3){0.06f, 1.0f, 1.15f}, false);
  town_box(b, 0x8fd3ff, (v3){at.x + face * 0.07f, at.y, at.z}, (v3){0.04f, 0.8f, 0.95f}, false);
  town_box(b, 0xe8e0d0, (v3){at.x + face * 0.12f, at.y - 0.52f, at.z}, (v3){0.2f, 0.1f, 1.25f}, false);
}

// a flower: a stem and a coloured head
hz_internal void town_flower(MeshBuilder *b, v3 at, u32 hex) {
  town_box(b, 0x3f8a2e, (v3){at.x, 0.14f, at.z}, (v3){0.05f, 0.28f, 0.05f}, false);
  town_box(b, hex, (v3){at.x, 0.3f, at.z}, (v3){0.2f, 0.14f, 0.2f}, false);
  town_box(b, 0xffe066, (v3){at.x, 0.38f, at.z}, (v3){0.08f, 0.04f, 0.08f}, false);
}

// petal colours: red, yellow, white, violet, orange
hz_internal u32 town_petal(u32 i) {
  switch (i % 5u) {
  case 0: return 0xff5d73;
  case 1: return 0xffd23f;
  case 2: return 0xffffff;
  case 3: return 0xb57cff;
  default: return 0xff8c42;
  }
}

// a flower bed along x from x0 to x1 at z
hz_internal void town_flower_bed(MeshBuilder *b, f32 x0, f32 x1, f32 z, TownRng *rng) {
  town_box(b, 0x6b4a2b, (v3){(x0 + x1) * 0.5f, 0.06f, z}, (v3){x1 - x0 + 0.3f, 0.12f, 0.9f}, false);
  for (f32 x = x0 + 0.2f; x < x1; x += 0.45f)
    for (i32 row = 0; row < 2; row++)
      town_flower(b, (v3){x + (town_rand(rng) - 0.5f) * 0.12f, 0.0f, z - 0.2f + 0.4f * (f32)row},
                  town_petal((u32)(town_rand(rng) * 5.0f)));
}

// a house: walls (solid), base stripe, roof, chimney, door with step and a lamp, windows, sign; `face` is the door side
hz_internal void town_house(MeshBuilder *b, v3 c, v3 size, u32 wall, u32 roof, u32 door, f32 face, b32 chimney) {
  f32 w = size.x, h = size.y, d = size.z;
  f32 front = c.z + face * d * 0.5f;
  town_box(b, wall, (v3){c.x, h * 0.5f, c.z}, size, true);
  town_box(b, 0x8a7f72, (v3){c.x, 0.2f, c.z}, (v3){w + 0.12f, 0.4f, d + 0.12f}, false);
  // the eaves trim and the roof
  town_box(b, 0xffffff, (v3){c.x, h + 0.06f, c.z}, (v3){w + 0.3f, 0.12f, d + 0.3f}, false);
  town_roof(b, roof, (v3){c.x, 0.0f, c.z}, h + 0.1f, w + 0.9f, d + 1.0f, d * 0.42f);
  if (chimney)
    town_box(b, 0x9c5a3c, (v3){c.x + w * 0.28f, h + d * 0.36f, c.z - face * d * 0.18f}, (v3){0.7f, 1.6f, 0.7f}, false);
  // the door: frame, door, knob, step, a little awning and a lamp
  town_box(b, 0xffffff, (v3){c.x, 1.1f, front + face * 0.03f}, (v3){1.5f, 2.3f, 0.08f}, false);
  town_box(b, door, (v3){c.x, 1.05f, front + face * 0.08f}, (v3){1.2f, 2.1f, 0.06f}, false);
  town_box(b, 0xffd23f, (v3){c.x + 0.4f, 1.0f, front + face * 0.13f}, (v3){0.1f, 0.1f, 0.06f}, false);
  town_box(b, 0xbdb6aa, (v3){c.x, 0.08f, front + face * 0.45f}, (v3){1.8f, 0.16f, 0.8f}, false);
  town_box(b, roof, (v3){c.x, 2.55f, front + face * 0.4f}, (v3){1.9f, 0.12f, 0.8f}, false);
  town_box(b, 0xfff2b0, (v3){c.x - 1.0f, 2.0f, front + face * 0.12f}, (v3){0.18f, 0.3f, 0.18f}, false);
  // windows: two on the front, one on each side, two on the back
  f32 wy = h > 4.0f ? h * 0.55f : 1.75f;
  town_window_z(b, (v3){c.x - w * 0.3f, wy, front}, face);
  town_window_z(b, (v3){c.x + w * 0.3f, wy, front}, face);
  town_window_z(b, (v3){c.x - w * 0.25f, wy, c.z - face * d * 0.5f}, -face);
  town_window_z(b, (v3){c.x + w * 0.25f, wy, c.z - face * d * 0.5f}, -face);
  town_window_x(b, (v3){c.x - w * 0.5f, wy, c.z}, -1.0f);
  town_window_x(b, (v3){c.x + w * 0.5f, wy, c.z}, 1.0f);
}

// a picket fence from a to b (straight along x or z): posts, two rails and pickets; one box collider for the run
hz_internal void town_fence(MeshBuilder *b, v3 a, v3 e) {
  v3 d = v3_sub(e, a);
  f32 len = v3_length(d);
  if (len < 0.01f)
    return;
  v3 dir = v3_scale(d, 1.0f / len);
  b32 along_x = m_absf(dir.x) > m_absf(dir.z);
  i32 posts = (i32)(len / 1.2f) + 1;
  for (i32 i = 0; i <= posts; i++) {
    v3 p = v3_add(a, v3_scale(d, (f32)i / (f32)posts));
    town_box(b, 0xfffaf0, (v3){p.x, 0.5f, p.z}, (v3){0.16f, 1.0f, 0.16f}, false);
  }
  v3 mid = v3_scale(v3_add(a, e), 0.5f);
  v3 rail = along_x ? (v3){len, 0.08f, 0.06f} : (v3){0.06f, 0.08f, len};
  town_box(b, 0xf2ead8, (v3){mid.x, 0.35f, mid.z}, rail, false);
  town_box(b, 0xf2ead8, (v3){mid.x, 0.75f, mid.z}, rail, false);
  mesh_builder_collider_box(b, (v3){mid.x, 0.6f, mid.z}, quat_identity(),
                            along_x ? (v3){len * 0.5f, 0.6f, 0.15f} : (v3){0.15f, 0.6f, len * 0.5f});
}

// a tree: trunk and two or three foliage cones, or a round crown
hz_internal void town_tree(MeshBuilder *b, v3 at, f32 scale, b32 round, TownRng *rng, b32 solid) {
  MeshBuilderStyle trunk = town_style(0x7a5232, MESH_BUILDER_COLLIDER_NONE);
  f32 th = 1.2f * scale;
  mesh_builder_cylinder(b, &trunk, m4_trs((v3){at.x, th * 0.5f, at.z}, quat_identity(), V3_ONE),
                        &(PgCylinderDesc){.radius_top = 0.18f * scale, .radius_bottom = 0.26f * scale, .height = th,
                                          .radial_segments = 6});
  if (solid)
    mesh_builder_collider_box(b, (v3){at.x, 1.0f, at.z}, quat_identity(), (v3){0.35f * scale, 1.0f, 0.35f * scale});
  u32 greens[] = {0x2f8f3a, 0x3aa845, 0x267a33};
  u32 g = greens[(u32)(town_rand(rng) * 3.0f) % 3];
  if (round) {
    MeshBuilderStyle leaf = town_style(g, MESH_BUILDER_COLLIDER_NONE);
    mesh_builder_polyhedron(b, &leaf, m4_trs((v3){at.x, th + 0.9f * scale, at.z}, quat_identity(), v3_scale(V3_ONE, 1.15f * scale)),
                            PG_POLYHEDRON_ICOSA, 0);
    MeshBuilderStyle leaf2 = town_style(0x45b84f, MESH_BUILDER_COLLIDER_NONE);
    mesh_builder_polyhedron(b, &leaf2,
                            m4_trs((v3){at.x + 0.35f * scale, th + 1.4f * scale, at.z - 0.2f * scale}, quat_identity(),
                                   v3_scale(V3_ONE, 0.7f * scale)),
                            PG_POLYHEDRON_ICOSA, 0);
    return;
  }
  for (i32 k = 0; k < 3; k++) {
    MeshBuilderStyle leaf = town_style(k == 2 ? 0x45b84f : g, MESH_BUILDER_COLLIDER_NONE);
    f32 base = th * 0.7f + (f32)k * 0.8f * scale;
    f32 r = (1.5f - 0.38f * (f32)k) * scale;
    mesh_builder_cone_between(b, &leaf, (v3){at.x, base, at.z}, (v3){at.x, base + 1.6f * scale, at.z}, r, 7);
  }
}

// the pond: a stone rim (solid), water and a couple of lily pads
hz_internal void town_pond(MeshBuilder *b, v3 c, f32 radius) {
  MeshBuilderStyle rim = town_style(0x9a9a92, MESH_BUILDER_COLLIDER_CYLINDER);
  mesh_builder_cylinder(b, &rim, m4_trs((v3){c.x, 0.25f, c.z}, quat_identity(), V3_ONE),
                        &(PgCylinderDesc){.radius_top = radius + 0.4f, .radius_bottom = radius + 0.5f, .height = 0.5f,
                                          .radial_segments = 14});
  MeshBuilderStyle water = town_style(0x3d9be0, MESH_BUILDER_COLLIDER_NONE);
  water.flat = false;
  mesh_builder_cylinder(b, &water, m4_trs((v3){c.x, 0.52f, c.z}, quat_identity(), V3_ONE),
                        &(PgCylinderDesc){.radius_top = radius, .radius_bottom = radius, .height = 0.06f, .radial_segments = 14});
  MeshBuilderStyle pad = town_style(0x4fbf4a, MESH_BUILDER_COLLIDER_NONE);
  v3 pads[] = {{c.x - 1.2f, 0.56f, c.z + 0.8f}, {c.x + 1.0f, 0.56f, c.z - 1.1f}, {c.x + 0.3f, 0.56f, c.z + 1.6f}};
  for (u32 i = 0; i < 3; i++)
    mesh_builder_cylinder(b, &pad, m4_trs(pads[i], quat_identity(), V3_ONE),
                          &(PgCylinderDesc){.radius_top = 0.42f, .radius_bottom = 0.42f, .height = 0.03f, .radial_segments = 7});
  town_flower(b, (v3){c.x - 1.2f, 0.3f, c.z + 0.8f}, 0xff9ad5);
}

// a patch of tall grass: clumps of blades as thin cones
hz_internal void town_tall_grass(MeshBuilder *b, f32 x0, f32 z0, f32 x1, f32 z1, TownRng *rng) {
  MeshBuilderStyle blade = town_style(0x2c7d2c, MESH_BUILDER_COLLIDER_NONE);
  MeshBuilderStyle tip = town_style(0x5cc24a, MESH_BUILDER_COLLIDER_NONE);
  for (f32 x = x0; x <= x1; x += 0.8f)
    for (f32 z = z0; z <= z1; z += 0.8f) {
      v3 p = {x + (town_rand(rng) - 0.5f) * 0.4f, 0.0f, z + (town_rand(rng) - 0.5f) * 0.4f};
      for (i32 k = 0; k < 3; k++) {
        v3 base = {p.x + (f32)(k - 1) * 0.18f, 0.0f, p.z + (town_rand(rng) - 0.5f) * 0.2f};
        v3 top = {base.x + (town_rand(rng) - 0.5f) * 0.25f, 0.7f + town_rand(rng) * 0.35f, base.z};
        mesh_builder_cone_between(b, k == 1 ? &tip : &blade, base, top, 0.14f, 4);
      }
    }
}

// a flat strip of path, slightly above the grass
hz_internal void town_path(MeshBuilder *b, f32 x0, f32 z0, f32 x1, f32 z1) {
  town_box(b, 0xe2c98f, (v3){(x0 + x1) * 0.5f, 0.015f, (z0 + z1) * 0.5f}, (v3){x1 - x0, 0.03f, z1 - z0}, false);
  // a darker edge on both long sides
  if (x1 - x0 < z1 - z0) {
    town_box(b, 0xc9ad72, (v3){x0, 0.02f, (z0 + z1) * 0.5f}, (v3){0.18f, 0.04f, z1 - z0}, false);
    town_box(b, 0xc9ad72, (v3){x1, 0.02f, (z0 + z1) * 0.5f}, (v3){0.18f, 0.04f, z1 - z0}, false);
  } else {
    town_box(b, 0xc9ad72, (v3){(x0 + x1) * 0.5f, 0.02f, z0}, (v3){x1 - x0, 0.04f, 0.18f}, false);
    town_box(b, 0xc9ad72, (v3){(x0 + x1) * 0.5f, 0.02f, z1}, (v3){x1 - x0, 0.04f, 0.18f}, false);
  }
}

// a mailbox on a post
hz_internal void town_mailbox(MeshBuilder *b, v3 at, u32 hex) {
  town_box(b, 0x6b4a2b, (v3){at.x, 0.5f, at.z}, (v3){0.12f, 1.0f, 0.12f}, false);
  town_box(b, hex, (v3){at.x, 1.08f, at.z}, (v3){0.4f, 0.32f, 0.6f}, false);
  town_box(b, 0xff3b3b, (v3){at.x + 0.22f, 1.2f, at.z + 0.15f}, (v3){0.04f, 0.3f, 0.08f}, false);
}

// main lane, once: the whole town into one slot; the parts become static renderer entities
hz_internal void town_build(EcsWorld *w, RendererState *r, Allocator *alloc) {
  ArenaAllocator *arena = arena_create(MB(64), MB(1));
  MeshBuilder mb;
  mesh_builder_begin(&mb, arena, &(MeshBuilderDesc){.slot_count = 1, .vtx_reserve = 65536, .idx_reserve = 131072});
  MeshBuilder *b = &mb;
  TownRng rng = {.s = 20261114u};

  // ---- paths: the main street north-south, a lane east-west past the houses, walks to the doors ----
  town_path(b, -1.4f, TOWN_NORTH - 4.0f, 1.4f, 15.0f);
  town_path(b, -13.0f, -3.0f, 13.0f, -0.2f);
  town_path(b, -7.8f, -5.0f, -6.2f, -3.0f);
  town_path(b, 6.2f, -5.0f, 7.8f, -3.0f);

  // ---- your home (west) and the neighbour's (east), doors facing south to the lane ----
  town_house(b, (v3){-7.0f, 0.0f, -8.5f}, (v3){7.0f, 3.6f, 6.0f}, 0xfff1dc, 0xd9463b, 0x8a5a33, 1.0f, true);
  town_house(b, (v3){7.0f, 0.0f, -8.5f}, (v3){7.0f, 3.6f, 6.0f}, 0xf4ecff, 0x3b6fd9, 0x5a3b8a, 1.0f, false);
  town_sign(b, (v3){-10.0f, 0.0f, -4.6f}, 1.0f, 0xffe7b0);
  town_sign(b, (v3){10.0f, 0.0f, -4.6f}, 1.0f, 0xffe7b0);
  town_mailbox(b, (v3){-4.6f, 0.0f, -4.4f}, 0xd9463b);
  town_mailbox(b, (v3){4.6f, 0.0f, -4.4f}, 0x3b6fd9);
  town_flower_bed(b, -9.8f, -8.2f, -4.9f, &rng);
  town_flower_bed(b, 8.2f, 9.8f, -4.9f, &rng);
  // a little garden fence behind each house
  town_fence(b, (v3){-12.0f, 0.0f, -13.0f}, (v3){-2.0f, 0.0f, -13.0f});
  town_fence(b, (v3){2.0f, 0.0f, -13.0f}, (v3){12.0f, 0.0f, -13.0f});

  // ---- the lab (south-east): bigger, white with a grey roof, door facing west to the main street ----
  {
    v3 c = {9.0f, 0.0f, 9.0f};
    v3 size = {10.0f, 4.6f, 7.0f};
    town_box(b, 0xf6f8fa, (v3){c.x, size.y * 0.5f, c.z}, size, true);
    town_box(b, 0x7d8a96, (v3){c.x, 0.2f, c.z}, (v3){size.x + 0.12f, 0.4f, size.z + 0.12f}, false);
    town_box(b, 0xffffff, (v3){c.x, size.y + 0.06f, c.z}, (v3){size.x + 0.3f, 0.12f, size.z + 0.3f}, false);
    town_roof(b, 0x66727e, (v3){c.x, 0.0f, c.z}, size.y + 0.1f, size.x + 0.9f, size.z + 1.0f, 1.8f);
    // a dish and a vent on the roof
    town_box(b, 0xc7d0d8, (v3){c.x + 3.0f, size.y + 1.0f, c.z + 1.2f}, (v3){0.25f, 1.4f, 0.25f}, false);
    MeshBuilderStyle dish = town_style(0xe6ebef, MESH_BUILDER_COLLIDER_NONE);
    mesh_builder_cone_between(b, &dish, (v3){c.x + 3.0f, size.y + 1.9f, c.z + 1.4f}, (v3){c.x + 3.0f, size.y + 1.5f, c.z + 1.0f},
                              0.8f, 10);
    // the entrance faces west (-x): door, frame, step, a blue band of windows on the long sides
    f32 fx = c.x - size.x * 0.5f;
    town_box(b, 0x2f5d8a, (v3){fx - 0.04f, 1.15f, c.z}, (v3){0.08f, 2.4f, 1.8f}, false);
    town_box(b, 0x9fd7ff, (v3){fx - 0.08f, 1.1f, c.z}, (v3){0.06f, 2.1f, 1.5f}, false);
    town_box(b, 0xbdb6aa, (v3){fx - 0.5f, 0.08f, c.z}, (v3){0.9f, 0.16f, 2.2f}, false);
    town_box(b, 0x2f5d8a, (v3){fx - 0.45f, 2.75f, c.z}, (v3){0.9f, 0.14f, 2.4f}, false);
    town_box(b, 0x2f5d8a, (v3){c.x, size.y - 0.7f, c.z - size.z * 0.5f - 0.03f}, (v3){size.x, 0.25f, 0.06f}, false);
    town_box(b, 0x2f5d8a, (v3){c.x, size.y - 0.7f, c.z + size.z * 0.5f + 0.03f}, (v3){size.x, 0.25f, 0.06f}, false);
    for (i32 i = 0; i < 4; i++) {
      f32 x = c.x - 3.3f + 2.2f * (f32)i;
      town_window_z(b, (v3){x, 2.0f, c.z - size.z * 0.5f}, -1.0f);
      town_window_z(b, (v3){x, 2.0f, c.z + size.z * 0.5f}, 1.0f);
    }
    town_window_x(b, (v3){fx, 2.0f, c.z - 2.2f}, -1.0f);
    town_window_x(b, (v3){fx, 2.0f, c.z + 2.2f}, -1.0f);
    town_sign(b, (v3){fx - 1.2f, 0.0f, c.z + 2.4f}, 1.0f, 0xd6ecff);
    // a fenced herb garden behind the lab
    town_fence(b, (v3){5.0f, 0.0f, 14.0f}, (v3){14.0f, 0.0f, 14.0f});
  }
  // the walk to the lab door
  town_path(b, 1.4f, 8.2f, 3.9f, 9.8f);

  // ---- the pond (south-west) with a bench-like rock and flowers around ----
  town_pond(b, (v3){-8.0f, 0.0f, 9.0f}, 3.4f);
  for (i32 i = 0; i < 10; i++) {
    f32 a = (f32)i * 0.63f + 0.3f;
    town_flower(b, (v3){-8.0f + m_cosf(a) * 4.6f, 0.0f, 9.0f + m_sinf(a) * 4.6f}, town_petal((u32)i % 3u));
  }

  // ---- a few trees inside the town, round ones by the houses ----
  town_tree(b, (v3){-13.5f, 0.0f, -3.5f}, 1.0f, true, &rng, true);
  town_tree(b, (v3){13.5f, 0.0f, -3.5f}, 1.0f, true, &rng, true);
  town_tree(b, (v3){-3.5f, 0.0f, 4.0f}, 0.9f, true, &rng, true);
  town_tree(b, (v3){-14.0f, 0.0f, 15.0f}, 1.1f, false, &rng, true);
  town_tree(b, (v3){3.5f, 0.0f, 15.5f}, 0.9f, true, &rng, true);

  // ---- the north exit: the street leads out between two fences, tall grass beyond, a sign ----
  town_fence(b, (v3){-2.4f, 0.0f, -18.0f}, (v3){-2.4f, 0.0f, TOWN_NORTH});
  town_fence(b, (v3){2.4f, 0.0f, -18.0f}, (v3){2.4f, 0.0f, TOWN_NORTH});
  town_sign(b, (v3){3.4f, 0.0f, -16.5f}, 1.0f, 0xc8f0c0);
  town_tall_grass(b, 4.5f, -19.5f, 12.0f, -15.5f, &rng);
  town_tall_grass(b, -12.0f, -19.5f, -5.0f, -16.0f, &rng);
  town_tall_grass(b, 12.0f, 1.5f, 17.0f, 4.0f, &rng);

  // ---- the tree line around the town: two staggered rows outside the walkable square, invisible walls inside ----
  for (i32 row = 0; row < 2; row++) {
    f32 inset = 1.2f + 2.4f * (f32)row;
    f32 step = 2.6f;
    for (f32 x = -TOWN_HALF_X - inset; x <= TOWN_HALF_X + inset; x += step) {
      f32 jx = (town_rand(&rng) - 0.5f) * 0.8f;
      if (m_absf(x) > 3.2f) // the gap where the street leaves north
        town_tree(b, (v3){x + jx, 0.0f, TOWN_NORTH - inset}, 1.0f + town_rand(&rng) * 0.4f, false, &rng, false);
      town_tree(b, (v3){x + jx, 0.0f, TOWN_SOUTH + inset}, 1.0f + town_rand(&rng) * 0.4f, row == 1, &rng, false);
    }
    for (f32 z = TOWN_NORTH; z <= TOWN_SOUTH; z += step) {
      f32 jz = (town_rand(&rng) - 0.5f) * 0.8f;
      town_tree(b, (v3){-TOWN_HALF_X - inset, 0.0f, z + jz}, 1.0f + town_rand(&rng) * 0.4f, row == 0, &rng, false);
      town_tree(b, (v3){TOWN_HALF_X + inset, 0.0f, z + jz}, 1.0f + town_rand(&rng) * 0.4f, row == 1, &rng, false);
    }
  }
  // beyond the north gap: more forest, the route is for later
  for (f32 x = -2.0f; x <= 2.0f; x += 2.0f)
    town_tree(b, (v3){x, 0.0f, TOWN_NORTH - 4.5f}, 1.2f, false, &rng, false);
  f32 cz = (TOWN_NORTH + TOWN_SOUTH) * 0.5f, half_z = (TOWN_SOUTH - TOWN_NORTH) * 0.5f;
  mesh_builder_collider_box(b, (v3){-TOWN_HALF_X - 0.5f, 2.0f, cz}, quat_identity(), (v3){0.5f, 2.0f, half_z + 1.0f});
  mesh_builder_collider_box(b, (v3){TOWN_HALF_X + 0.5f, 2.0f, cz}, quat_identity(), (v3){0.5f, 2.0f, half_z + 1.0f});
  mesh_builder_collider_box(b, (v3){0.0f, 2.0f, TOWN_SOUTH + 0.5f}, quat_identity(), (v3){TOWN_HALF_X + 1.0f, 2.0f, 0.5f});
  mesh_builder_collider_box(b, (v3){0.0f, 2.0f, TOWN_NORTH - 0.5f}, quat_identity(), (v3){TOWN_HALF_X + 1.0f, 2.0f, 0.5f});

  MeshBuilderResult built = mesh_builder_finish(b, &(MeshBuilderFinishDesc){.renderer = r, .world = w, .alloc = alloc});
  HzHandleT(RendererMaterial) mat = renderer_default_material(r);
  const MeshBuilderSlotOut *slot = &built.slots[TOWN_SLOT];
  for (u32 i = 0; i < slot->part_count; i++)
    ecs_renderer_spawn(w, r,
                       &(EcsRendererSpawnDesc){.submesh = slot->parts[i],
                                               .material = mat,
                                               .transform = {.position = V3_ZERO, .rotation = quat_identity(), .scale = V3_ONE},
                                               .casts_shadow = true,
                                               .static_shadow = true,
                                               .name = "Town"});
  log_info("town: % vertices, % tris, % parts, % colliders", fmt_u32(built.vertex_count), fmt_u32(slot->tri_count),
           fmt_u32(slot->part_count), fmt_u32(built.collider_count));
  arena_destroy(arena);
}
