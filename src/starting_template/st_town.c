// Brickhollow: the LEGO village of the three.js version, rebuilt in code with the mesh builder. All coordinates below
// are in STUDS like the original (1 stud = TOWN_STUD metres, a brick 1.2 studs high, a plate 0.4); x east, z south,
// the house fronts face +z (south, towards the camera). One vertex-coloured slot drawn with the renderer's default
// material, static colliders from the same calls. The baseplate's studs are drawn by the floor shader (st_grid.glsl).
// Included by starting_template.c above app_init.

#define TOWN_SLOT 0
#define TOWN_STUD 0.38f // metres per stud; the minifig (4.9 studs) is then 1.86 m, the character controller's size
#define TOWN_STUD_H 0.17f  // stud height in studs
#define TOWN_STUD_R 0.3f   // stud radius in studs

// the LEGO palette (sRGB)
#define C_GREEN 0x4b9f4au
#define C_DARK_GREEN 0x237841u
#define C_PINE 0x1d6630u
#define C_GRAY 0xa0a5a9u
#define C_DARK_GRAY 0x6c6e68u
#define C_RED 0xc91a09u
#define C_BLUE 0x0055bfu
#define C_ORANGE 0xfe8a18u
#define C_TAN 0xe4cd9eu
#define C_WHITE 0xf4f4f4u
#define C_BROWN 0x583927u
#define C_RED_BROWN 0x82422au
#define C_YELLOW 0xf2cd37u
#define C_GLASS 0x9fd8efu
#define C_WATER 0x2a7fd4u
#define C_BLACK 0x1b2a34u
#define C_PINK 0xff8ac0u

// a tiny deterministic random stream (the three.js version used Math.random; the layout only needs to look random)
typedef struct TownRng {
  u32 s;
} TownRng;
hz_internal f32 town_rand(TownRng *r) {
  r->s = r->s * 1664525u + 1013904223u;
  return (f32)(r->s >> 8) / 16777216.0f;
}

typedef struct TownRect {
  f32 x0, x1, z0, z1;
} TownRect;

#define TOWN_MAX_RECTS 1024
// draw groups: 0 the village, 1 + RoomId each room. Only the group you are in is drawn (from inside a room you would see
// the other rooms, they lie side by side east of the village)
#define TOWN_GROUPS 6 // the village, the four rooms, the pond cave (st_cave.c)
#define TOWN_GROUP_MAX 512

// a named thing for the debug picker: a circle (studs) around x, z
typedef struct TownTag {
  CString name;
  f32 x, z, r;
} TownTag;
#define TOWN_MAX_TAGS 2048

typedef struct TownGroups {
  EcsEntity ents[TOWN_GROUPS][TOWN_GROUP_MAX];
  u32 count[TOWN_GROUPS];
} TownGroups;
typedef struct Town {
  MeshBuilder *b;
  TownRng rng;
  EcsWorld *world;
  RendererState *renderer;
  AssetSystem *assets;
  TownRect *used; // blockers and kept-free areas, for the plants' free-spot search
  u32 used_count;
  u32 props;      // placed LEGO plant models
  u32 box_n;      // bricks so far (their size jitter)
  TownRect *solid; // the colliders only (town_block), handed to the game
  u32 solid_count;
  MeshBuilder *mbs; // one builder per draw group
  TownTag *tags;    // the debug picker's names
  u32 tag_count;
  u32 group;
  TownGroups *groups;
} Town;

hz_internal void town_tag(Town *t, CString name, f32 x, f32 z, f32 r) {
  if (t->tags && t->tag_count < TOWN_MAX_TAGS)
    t->tags[t->tag_count++] = (TownTag){name, x, z, r};
}

hz_internal void town_group_add(TownGroups *g, u32 group, EcsEntity e) {
  if (g && g->count[group] < TOWN_GROUP_MAX)
    g->ents[group][g->count[group]++] = e;
}

// what follows is built into draw group `group`
hz_internal void town_group_switch(Town *t, u32 group) {
  t->group = group;
  t->b = &t->mbs[group];
}

hz_internal v3 town_m(f32 x, f32 y, f32 z) { return (v3){x * TOWN_STUD, y * TOWN_STUD, z * TOWN_STUD}; }

hz_internal MeshBuilderStyle town_style(u32 hex) {
  return (MeshBuilderStyle){.slot = TOWN_SLOT, .color = mesh_builder_hex(hex), .flat = true};
}

// a darker or lighter shade of a colour (the stepped roof alternates)
hz_internal u32 town_shade(u32 hex, f32 k) {
  u32 r = (u32)m_minf(255.0f, (f32)((hex >> 16) & 255u) * k);
  u32 g = (u32)m_minf(255.0f, (f32)((hex >> 8) & 255u) * k);
  u32 b = (u32)m_minf(255.0f, (f32)(hex & 255u) * k);
  return (r << 16) | (g << 8) | b;
}

// one stud on top of a brick at (x, y, z), y = the brick's top
hz_internal void town_stud(Town *t, u32 hex, f32 x, f32 y, f32 z) {
  MeshBuilderStyle st = town_style(hex);
  st.flat = false;
  mesh_builder_cylinder(t->b, &st, m4_trs(town_m(x, y + TOWN_STUD_H * 0.5f, z), quat_identity(), V3_ONE),
                        &(PgCylinderDesc){.radius_top = TOWN_STUD_R * TOWN_STUD, .radius_bottom = TOWN_STUD_R * TOWN_STUD,
                                          .height = TOWN_STUD_H * TOWN_STUD, .radial_segments = 8});
}

// studs on a brick's top face, centred like real LEGO (one per full stud of its size)
hz_internal void town_stud_rect(Town *t, u32 hex, f32 x0, f32 x1, f32 z0, f32 z1, f32 y) {
  if (x1 - x0 < 0.7f || z1 - z0 < 0.7f)
    return;
  i32 nx = (i32)m_maxf(1.0f, m_floorf(x1 - x0 + 0.05f));
  i32 nz = (i32)m_maxf(1.0f, m_floorf(z1 - z0 + 0.05f));
  f32 ox = x0 + (x1 - x0 - (f32)nx) * 0.5f + 0.5f, oz = z0 + (z1 - z0 - (f32)nz) * 0.5f + 0.5f;
  for (i32 i = 0; i < nx; i++)
    for (i32 k = 0; k < nz; k++)
      town_stud(t, hex, ox + (f32)i, y, oz + (f32)k);
}

// a brick: size w x h x d studs, centred on x/z, its BOTTOM at y (as box() in the three.js town)
hz_internal void town_box(Town *t, u32 hex, f32 w, f32 h, f32 d, f32 x, f32 y, f32 z, b32 studs) {
  MeshBuilderStyle st = town_style(hex);
  // bricks that touch would share faces exactly (a sofa's arm and seat, a trim on a wall) and flicker: every brick is
  // a few hundredths of a stud smaller or larger than the last, so no two faces ever lie in one plane
  t->box_n++;
  f32 e = 0.006f * (f32)((i32)(t->box_n % 7u) - 3);
  f32 e2 = 0.006f * (f32)((i32)(t->box_n * 3u % 7u) - 3);
  mesh_builder_box(t->b, &st, m4_trs(town_m(x, y + h * 0.5f + e2 * 0.5f, z), quat_identity(), V3_ONE),
                   town_m(w + e, h + e2, d - e));
  if (studs)
    town_stud_rect(t, hex, x - w * 0.5f, x + w * 0.5f, z - d * 0.5f, z + d * 0.5f, y + h);
}

// a rotated brick (centre, euler radians)
hz_internal void town_box_rot(Town *t, u32 hex, v3 size, v3 center, v3 euler) {
  MeshBuilderStyle st = town_style(hex);
  mesh_builder_box(t->b, &st, m4_trs(town_m(center.x, center.y, center.z), quat_from_euler(euler), V3_ONE),
                   town_m(size.x, size.y, size.z));
}

// a cylinder standing on y (bottom), radius r, height h, in studs
hz_internal void town_cyl(Town *t, u32 hex, f32 x, f32 y, f32 z, f32 r_bottom, f32 r_top, f32 h, u32 seg, b32 smooth) {
  MeshBuilderStyle st = town_style(hex);
  st.flat = !smooth;
  mesh_builder_cylinder(t->b, &st, m4_trs(town_m(x, y + h * 0.5f, z), quat_identity(), V3_ONE),
                        &(PgCylinderDesc){.radius_top = r_top * TOWN_STUD, .radius_bottom = r_bottom * TOWN_STUD,
                                          .height = h * TOWN_STUD, .radial_segments = seg});
}

// a solid block nobody walks through (the three.js "blockers"), 2 m high
hz_internal void town_occupy(Town *t, f32 x0, f32 x1, f32 z0, f32 z1) {
  if (t->used_count < TOWN_MAX_RECTS)
    t->used[t->used_count++] = (TownRect){x0, x1, z0, z1};
}

// a block `h` metres high (buildings are as high as their roofs: the camera's view ray must not pass over them)
hz_internal void town_block_h(Town *t, f32 x0, f32 x1, f32 z0, f32 z1, f32 h) {
  town_occupy(t, x0, x1, z0, z1);
  if (t->solid && t->solid_count < TOWN_MAX_RECTS)
    t->solid[t->solid_count++] = (TownRect){x0, x1, z0, z1};
  v3 c = town_m((x0 + x1) * 0.5f, 0.0f, (z0 + z1) * 0.5f);
  c.y = h * 0.5f;
  mesh_builder_collider_box(t->b, c, quat_identity(), (v3){(x1 - x0) * 0.5f * TOWN_STUD, h * 0.5f, (z1 - z0) * 0.5f * TOWN_STUD});
}

hz_internal void town_block(Town *t, f32 x0, f32 x1, f32 z0, f32 z1) {
  town_occupy(t, x0, x1, z0, z1);
  if (t->solid && t->solid_count < TOWN_MAX_RECTS)
    t->solid[t->solid_count++] = (TownRect){x0, x1, z0, z1};
  v3 c = town_m((x0 + x1) * 0.5f, 0.0f, (z0 + z1) * 0.5f);
  c.y = 1.0f;
  mesh_builder_collider_box(t->b, c, quat_identity(), (v3){(x1 - x0) * 0.5f * TOWN_STUD, 1.0f, (z1 - z0) * 0.5f * TOWN_STUD});
}

// ---------------------------------------------------------------- plants

// grey road plates (0.4 high, studs on top), as the three.js roads list

#define TOWN_ROAD_COUNT 7
hz_internal TownRect town_road(i32 i) {
  switch (i) {
  case 0: return (TownRect){-3, 3, -27, 24};
  case 1: return (TownRect){-26, 29, -2, 3};
  case 2: return (TownRect){26, 29, 3, 10};
  case 3: return (TownRect){29, 66, 7, 10};
  case 4: return (TownRect){-16, -14, -6, -2};
  case 5: return (TownRect){3, 15, 16.5f, 19.5f};
  default: return (TownRect){-20, -18, 3, 7};
  }
}

// ---- the LEGO plant models of the three.js version (tools/make_props.py: true LEGO parts, metres) ----
#define PROP(name) "brickhollow/props/" name ".glb"

// the strip in front of each readable sign board, where the camera stands for the close-up (sign_board, st_world.c):
// nothing grows there, or a plant fills the view of the sign
hz_internal b32 town_in_sign_view(f32 x, f32 z) {
  static const f32 boards[4][4] = {{5.9f, 20.41f, 0.0f, 3.8f}, {5.0f, -24.09f, 0.0f, 3.4f}, {-22.91f, 8.9f, M_PI * 0.5f, 2.8f}, {13.0f, 16.17f, 0.0f, 5.2f}};
  for (i32 i = 0; i < 4; i++) {
    f32 dx = x - boards[i][0], dz = z - boards[i][1], yaw = boards[i][2];
    f32 along = dx * m_sinf(yaw) + dz * m_cosf(yaw), side = dx * m_cosf(yaw) - dz * m_sinf(yaw);
    if (along > -0.5f && along < 9.0f && m_absf(side) < boards[i][3] * 0.5f + 1.2f)
      return true;
  }
  return false;
}

// one instance of a prop model at (x, z) studs, standing on y studs, scaled by s, turned by rot (plants not in front of a sign)
hz_internal void town_prop(Town *t, CString path, f32 x, f32 z, f32 y, f32 s, f32 rot, b32 shadow) {
  if (!cstr_contains(path, "/signs/") && y < 1.0f && t->group == 0 && town_in_sign_view(x, z))
    return;
  HzHandleT(ModelBlobAsset) model = asset_load(ModelBlobAsset, t->assets, .path = path);
  EcsEntity e = ecs_renderer_spawn(t->world, t->renderer,
                     &(EcsRendererSpawnDesc){.model_asset = blob_asset_ref_from_handle(ModelBlobAsset, model),
                                             .transform = {.position = town_m(x, y, z),
                                                           .rotation = quat_from_euler((v3){0.0f, rot, 0.0f}),
                                                           .scale = (v3){s, s, s}},
                                             .casts_shadow = shadow,
                                             .static_shadow = shadow});
  town_group_add(t->groups, t->group, e);
  town_tag(t, path, x, z, 0.8f + s * 0.25f);
  t->props++;
}

hz_internal f32 town_rot(Town *t) { return town_rand(&t->rng) * 6.2832f; }

// the flower colours the models exist in
typedef enum TownPetal { PETAL_RED, PETAL_YELLOW, PETAL_PINK, PETAL_WHITE, PETAL_ORANGE, PETAL_PURPLE } TownPetal;

hz_internal TownPetal town_petal_of(u32 hex) {
  switch (hex) {
  case C_RED: return PETAL_RED;
  case C_YELLOW: return PETAL_YELLOW;
  case C_PINK: return PETAL_PINK;
  case C_WHITE: return PETAL_WHITE;
  case C_ORANGE: return PETAL_ORANGE;
  default: return PETAL_PURPLE;
  }
}

#define PETAL_PATHS(prefix)                                                                                                \
  PROP(prefix "_red"), PROP(prefix "_yellow"), PROP(prefix "_pink"), PROP(prefix "_white"), PROP(prefix "_orange"),        \
      PROP(prefix "_purple")

// the LEGO flower plant (2x2 leaves + 1x1 flower)
hz_internal void town_flower(Town *t, f32 x, f32 z, u32 hex, f32 y) {
  CString paths[] = {PETAL_PATHS("flower")};
  town_prop(t, paths[town_petal_of(hex)], x, z, y, 1.0f, town_rot(t), false);
  town_occupy(t, x - 0.6f, x + 0.6f, z - 0.6f, z + 0.6f);
}

// a "Lego Frog" plant: 0 leafy a, 1 leafy b, 2 sprout, 3 bouquet, 4 / 5 the flat stems
hz_internal void town_frog_plant(Town *t, u32 variant, f32 x, f32 z, TownPetal petal) {
  f32 s = 0.9f + town_rand(&t->rng) * 0.25f;
  switch (variant) {
  case 0: town_prop(t, PROP("frog_leafy_a"), x, z, 0.0f, s, town_rot(t), false); break;
  case 1: town_prop(t, PROP("frog_leafy_b"), x, z, 0.0f, s, town_rot(t), false); break;
  case 2: {
    CString paths[] = {PETAL_PATHS("frog_sprout")};
    town_prop(t, paths[petal], x, z, 0.0f, s, town_rot(t), false);
  } break;
  case 3: {
    CString paths[] = {PETAL_PATHS("frog_bouquet")};
    town_prop(t, paths[petal], x, z, 0.0f, s, town_rot(t), false);
  } break;
  case 4: {
    CString paths[] = {PETAL_PATHS("frog_stem_a")};
    town_prop(t, paths[petal], x, z, 0.0f, s, town_rot(t), false);
  } break;
  default: {
    CString paths[] = {PETAL_PATHS("frog_stem_b")};
    town_prop(t, paths[petal], x, z, 0.0f, s, town_rot(t), false);
  } break;
  }
}

// LEGO bamboo leaves (part 30176)
hz_internal void town_leaves(Town *t, f32 x, f32 z, f32 s) {
  town_prop(t, PROP("bamboo"), x, z, 0.0f, s, town_rot(t), false);
  town_occupy(t, x - 0.8f, x + 0.8f, z - 0.8f, z + 0.8f);
}

// the LEGO flower pot (4x4), or only its flowers (the park)
hz_internal void town_flower_pot(Town *t, f32 x, f32 z, b32 pot) {
  town_prop(t, pot ? PROP("flowerpot") : PROP("flower_bunch"), x, z, 0.0f, 1.0f, town_rot(t), true);
  town_block(t, x - 1.8f, x + 1.8f, z - 1.8f, z + 1.8f);
}

// the flower pot standing on something (a table), no collider of its own
hz_internal void town_flower_pot_on(Town *t, f32 x, f32 z, f32 y) {
  town_prop(t, PROP("flowerpot"), x, z, y, 1.0f, town_rot(t), true);
}

hz_internal void town_spruce(Town *t, f32 x, f32 z, f32 s) { town_prop(t, PROP("spruce"), x, z, 0.0f, s, town_rot(t), true); }

hz_internal void town_round_tree(Town *t, f32 x, f32 z, f32 s) {
  town_prop(t, PROP("round_tree"), x, z, 0.0f, s, town_rot(t), true);
}

hz_internal void town_bush(Town *t, f32 x, f32 z, f32 s) { town_prop(t, PROP("bush"), x, z, 0.0f, s, town_rot(t), false); }

// is (x, z) free for a plant of radius r: inside the village, off the roads, every used rect and placed plant
hz_internal b32 town_free(Town *t, f32 x, f32 z, f32 r, const v3 *taken, u32 taken_count) {
  if (x < -26.2f + r || x > 66.2f - r || z < -25.6f + r || z > 23.4f - r)
    return false;
  for (i32 i = 0; i < TOWN_ROAD_COUNT; i++) {
    TownRect rd = town_road(i);
    if (x > rd.x0 - r && x < rd.x1 + r && z > rd.z0 - r && z < rd.z1 + r)
      return false;
  }
  for (u32 i = 0; i < t->used_count; i++) {
    TownRect u = t->used[i];
    if (x > u.x0 - r && x < u.x1 + r && z > u.z0 - r && z < u.z1 + r)
      return false;
  }
  for (u32 i = 0; i < taken_count; i++) {
    f32 dx = taken[i].x - x, dz = taken[i].z - z;
    if (dx * dx + dz * dz < (taken[i].y + r) * (taken[i].y + r))
      return false;
  }
  return true;
}

// the three.js placeFrogPlants: plants along the village's edge ring, then cypresses, ferns, small pines and reeds
// wherever there is room, outer areas first. taken: x, z and the radius (in y) of everything placed here.
hz_internal void town_wild_plants(Town *t, Allocator *a) {
  const u32 cap = 4096;
  v3 *taken = ALLOC_ARRAY(a, v3, cap);
  u32 nt = 0;
  // the pond banks and garden corners first
  f32 first[4][3] = {{-3.0f, 21.5f, 0}, {-13.4f, 18.4f, 3}, {-24.2f, 16.2f, 1}, {-24.0f, 10.5f, 2}};
  for (i32 i = 0; i < 4; i++)
    if (town_free(t, first[i][0], first[i][1], 1.4f, taken, nt)) {
      town_frog_plant(t, (u32)first[i][2], first[i][0], first[i][1], (TownPetal)((u32)(town_rand(&t->rng) * 6.0f) % 6u));
      taken[nt++] = (v3){first[i][0], 1.4f, first[i][1]};
    }
  // the edge ring, shuffled
  v3 *cand = ALLOC_ARRAY(a, v3, cap);
  u32 nc = 0;
  for (f32 x = -26.2f + 1.0f; x < 66.2f - 1.0f; x += 1.3f)
    for (f32 z = -25.6f + 1.0f; z < 23.4f - 1.0f; z += 1.3f) {
      f32 edge = m_minf(m_minf(x + 26.2f, 66.2f - x), m_minf(z + 25.6f, 23.4f - z));
      if (edge > 0.8f && edge < 7.0f && nc < cap)
        cand[nc++] = (v3){x + (town_rand(&t->rng) - 0.5f) * 0.6f, 0.0f, z + (town_rand(&t->rng) - 0.5f) * 0.6f};
    }
  for (u32 i = nc - 1; i > 0; i--) {
    u32 j = (u32)(town_rand(&t->rng) * (f32)(i + 1)) % (i + 1);
    v3 tmp = cand[i];
    cand[i] = cand[j];
    cand[j] = tmp;
  }
  u32 plants = 0;
  for (u32 i = 0; i < nc && plants < 30; i++) {
    u32 v = (u32)(town_rand(&t->rng) * 4.0f) % 4u;
    f32 r = v < 2 ? 1.9f : 1.3f;
    if (!town_free(t, cand[i].x, cand[i].z, r, taken, nt))
      continue;
    town_frog_plant(t, v, cand[i].x, cand[i].z, (TownPetal)((u32)(town_rand(&t->rng) * 6.0f) % 6u));
    taken[nt++] = (v3){cand[i].x, r, cand[i].z};
    plants++;
  }
  // reeds standing in the pond's corners
  f32 reeds[4][2] = {{-11.2f, 15.8f}, {-4.6f, 15.9f}, {-11.3f, 21.2f}, {-4.7f, 21.3f}};
  for (i32 i = 0; i < 4; i++)
    town_prop(t, PROP("reed"), reeds[i][0], reeds[i][1], 0.0f, 0.45f + town_rand(&t->rng) * 0.1f, town_rot(t), false);
  // the whole village, outer areas first: a key of edge distance plus noise (in y), taken in bands
  nc = 0;
  for (f32 x = -26.2f + 1.0f; x < 66.2f - 1.0f; x += 1.1f)
    for (f32 z = -25.6f + 1.0f; z < 23.4f - 1.0f; z += 1.1f) {
      f32 edge = m_minf(m_minf(x + 26.2f, 66.2f - x), m_minf(z + 25.6f, 23.4f - z));
      if (nc < cap)
        cand[nc++] = (v3){x + (town_rand(&t->rng) - 0.5f) * 0.5f, edge + town_rand(&t->rng) * 4.0f,
                          z + (town_rand(&t->rng) - 0.5f) * 0.5f};
    }
  // per kind: how many, radius, scale, collider half-size (0: walk through)
  for (i32 kind = 0; kind < 4; kind++) {
    i32 want = kind == 0 ? 6 : kind == 1 ? 4 : kind == 2 ? 8 : 10;
    f32 r = kind == 0 ? 1.7f : kind == 1 ? 2.2f : kind == 2 ? 1.6f : 1.1f;
    f32 sc = kind == 0 ? 0.8f : kind == 1 ? 0.3f : kind == 2 ? 0.75f : 0.5f;
    f32 bl = kind == 0 ? 0.8f : kind == 2 ? 0.6f : 0.0f;
    CString path = kind == 0 ? PROP("cypress") : kind == 1 ? PROP("fern") : kind == 2 ? PROP("pine_small") : PROP("reed");
    i32 k = 0;
    for (f32 band = 0.0f; band < 40.0f && k < want; band += 0.5f)
      for (u32 i = 0; i < nc && k < want; i++) {
        if (cand[i].y < band || cand[i].y >= band + 0.5f || !town_free(t, cand[i].x, cand[i].z, r, taken, nt))
          continue;
        town_prop(t, path, cand[i].x, cand[i].z, 0.0f, sc * (0.9f + town_rand(&t->rng) * 0.2f), town_rot(t), kind == 0 || kind == 2);
        taken[nt++] = (v3){cand[i].x, r, cand[i].z};
        if (bl > 0.0f)
          town_block(t, cand[i].x - bl, cand[i].x + bl, cand[i].z - bl, cand[i].z + bl);
        k++;
      }
  }
}

// a LEGO grass stem: a round 1x1 plate with six flat blades and a centre blade
hz_internal void town_grass_tuft(Town *t, f32 x, f32 z, f32 s, u32 hex) {
  if (t->group == 0 && town_in_sign_view(x, z))
    return;
  town_cyl(t, hex, x, 0.0f, z, 0.4f, 0.4f, 0.3f, 6, false);
  MeshBuilderStyle blade = town_style(hex);
  f32 a0 = town_rand(&t->rng) * 6.283f;
  for (i32 i = 0; i < 7; i++) {
    f32 a = a0 + (f32)i * 0.9f;
    f32 lean = i == 6 ? 0.0f : 0.55f + 0.15f * (f32)(i % 2);
    f32 h = (i == 6 ? 2.4f : 2.0f) * s;
    v3 base = town_m(x, 0.25f, z);
    v3 tip = town_m(x + m_cosf(a) * lean * s, h, z + m_sinf(a) * lean * s);
    mesh_builder_cone_between(t->b, &blade, base, tip, 0.17f * TOWN_STUD, 3);
  }
}

// ---------------------------------------------------------------- buildings

// a window with a white frame, glass and a mullion; axis 'z' = in a wall facing +-z
hz_internal void town_window(Town *t, f32 x, f32 y, f32 z, f32 w, f32 h, b32 along_z, f32 out) {
  if (along_z) {
    town_box(t, C_WHITE, w + 0.4f, h + 0.4f, 0.3f, x, y - 0.2f, z + out * 0.05f, false);
    town_box(t, C_GLASS, w - 0.2f, h - 0.2f, 0.3f, x, y + 0.1f, z + out * 0.2f, false);
    town_box(t, C_WHITE, 0.15f, h, 0.2f, x, y, z + out * 0.3f, false);
  } else {
    town_box(t, C_WHITE, 0.3f, h + 0.4f, w + 0.4f, x + out * 0.05f, y - 0.2f, z, false);
    town_box(t, C_GLASS, 0.3f, h - 0.2f, w - 0.2f, x + out * 0.2f, y + 0.1f, z, false);
    town_box(t, C_WHITE, 0.2f, h, 0.15f, x + out * 0.3f, y, z, false);
  }
}

// a brick wall (bottom on the 0.4 foundation) with the courses drawn as thin grooves every brick (1.2 studs)
hz_internal void town_wall(Town *t, u32 hex, f32 w, f32 h, f32 d, f32 x, f32 z) {
  town_box(t, hex, w, h, d, x, 0.4f, z, false);
  u32 groove = town_shade(hex, 0.86f);
  for (f32 y = 0.4f + 1.2f; y < 0.4f + h - 0.3f; y += 1.2f)
    town_box(t, groove, w + 0.06f, 0.07f, d + 0.06f, x, y - 0.035f, z, false);
}

// a two-storey village house (buildHouse): foundation, brick walls, floor band, door with step, windows, a stepped
// brick roof with studs on the ridge, a chimney, the sign board above the door and a mailbox
hz_internal void town_house(Town *t, f32 cx, f32 cz, f32 w, f32 d, u32 roof, u32 wall, u32 door, u32 mailbox,
                             CString sign) {
  f32 x0 = cx - w * 0.5f, x1 = cx + w * 0.5f, z0 = cz - d * 0.5f, z1 = cz + d * 0.5f;
  town_block_h(t, x0 - 0.6f, x1 + 0.6f, z0 - 0.6f, z1 + 0.5f, 11.0f * TOWN_STUD);
  town_box(t, C_GRAY, w + 1.0f, 0.4f, d + 1.0f, cx, 0.0f, cz, false);
  const f32 H = 7.2f;
  town_wall(t, wall, w, H, 1.0f, cx, z1 - 0.5f);
  town_wall(t, wall, w, H, 1.0f, cx, z0 + 0.5f);
  town_wall(t, wall, 1.0f, H, d - 2.0f, x0 + 0.5f, cz);
  town_wall(t, wall, 1.0f, H, d - 2.0f, x1 - 0.5f, cz);
  town_box(t, wall, w - 2.0f, 0.4f, d - 2.0f, cx, H, cz, false); // the ceiling (no one looks in, but no gaps)
  town_box(t, C_WHITE, w + 0.2f, 0.4f, d + 0.2f, cx, 3.8f, cz, false);
  // door (front faces +z)
  town_box(t, C_WHITE, 2.2f, 4.0f, 0.4f, cx, 0.4f, z1 + 0.1f, false);
  town_box(t, door, 1.8f, 3.6f, 0.4f, cx, 0.4f, z1 + 0.25f, false);
  town_box(t, C_YELLOW, 0.3f, 0.3f, 0.3f, cx + 0.55f, 2.05f, z1 + 0.5f, false);
  town_box(t, C_DARK_GRAY, 3.0f, 0.4f, 1.2f, cx, 0.4f, z1 + 0.7f, true);
  // windows: two below, two above at the sides (the middle above the door is the sign), one on each side wall
  for (i32 s = -1; s <= 1; s += 2) {
    f32 wx = cx + (f32)s * (w * 0.5f - 2.3f);
    town_window(t, wx, 1.6f, z1, 2.4f, 1.8f, true, 1.0f);
    town_window(t, wx, 5.0f, z1, 1.9f, 1.6f, true, 1.0f);
    town_window(t, cx + (f32)s * w * 0.5f, 5.0f, cz, 2.4f, 1.6f, false, (f32)s);
    town_window(t, wx, 5.0f, z0, 1.9f, 1.6f, true, -1.0f);
  }
  // stepped brick roof
  const f32 ov = 0.8f;
  for (i32 i = 0; i < 6; i++) {
    f32 dd = d + ov * 2.0f - (f32)i * 1.9f;
    if (dd <= 0.6f)
      break;
    town_box(t, i % 2 == 0 ? roof : town_shade(roof, 0.92f), w + ov * 2.0f, 0.6f, dd, cx, 0.4f + H + (f32)i * 0.6f, cz, false);
  }
  town_box(t, roof, w + ov * 2.0f, 0.4f, 1.0f, cx, 0.4f + H + 6.0f * 0.6f, cz, true);
  town_box(t, C_RED_BROWN, 1.6f, 3.4f, 1.6f, cx + w * 0.5f - 2.2f, 0.4f + H + 0.6f, cz - 1.5f, true);
  // the sign board above the door (white frame, a coloured board)
  f32 sw = m_minf(3.4f, w - 7.4f), sh = sw / 4.0f;
  town_box(t, C_WHITE, sw + 0.3f, sh + 0.3f, 0.2f, cx, 5.8f - sh * 0.5f - 0.15f, z1, false);
  town_box(t, mailbox, sw, sh, 0.1f, cx, 5.8f - sh * 0.5f, z1 + 0.12f, false);
  town_prop(t, sign, cx, z1 + 0.19f, 5.8f, 1.0f, 0.0f, false);
  // mailbox
  f32 mx = cx - 3.2f, mz = z1 + 2.2f;
  town_box(t, C_DARK_GRAY, 0.4f, 1.6f, 0.4f, mx, 0.0f, mz, false);
  town_box(t, mailbox, 1.0f, 0.9f, 1.4f, mx, 1.6f, mz, true);
  town_block(t, mx - 0.6f, mx + 0.6f, mz - 0.8f, mz + 0.8f);
}

// Prof. Quill's lab (buildLab): tan walls, a green door, wide windows with striped awnings, round windows, a flat
// roof with tiles, a rim with studs, solar panels, a chimney and a satellite dish
hz_internal void town_lab(Town *t, f32 cx, f32 cz, f32 w, f32 d) {
  f32 x0 = cx - w * 0.5f, x1 = cx + w * 0.5f, z0 = cz - d * 0.5f, z1 = cz + d * 0.5f;
  town_block_h(t, x0 - 0.6f, x1 + 0.6f, z0 - 0.6f, z1 + 0.5f, 8.6f * TOWN_STUD);
  town_box(t, C_GRAY, w + 1.0f, 0.4f, d + 1.0f, cx, 0.0f, cz, false);
  const f32 H = 7.2f;
  town_wall(t, C_TAN, w, H, 1.0f, cx, z1 - 0.5f);
  town_wall(t, C_TAN, w, H, 1.0f, cx, z0 + 0.5f);
  town_wall(t, C_TAN, 1.0f, H, d - 2.0f, x0 + 0.5f, cz);
  town_wall(t, C_TAN, 1.0f, H, d - 2.0f, x1 - 0.5f, cz);
  f32 fz = z1;
  town_box(t, C_WHITE, 2.4f, 4.2f, 0.4f, cx, 0.4f, fz + 0.1f, false);
  town_box(t, C_DARK_GREEN, 2.0f, 3.8f, 0.4f, cx, 0.4f, fz + 0.25f, false);
  town_box(t, C_GLASS, 0.6f, 1.2f, 0.2f, cx, 2.6f, fz + 0.45f, false);
  town_box(t, C_DARK_GRAY, 3.4f, 0.4f, 1.4f, cx, 0.4f, fz + 0.8f, true);
  for (i32 s = -1; s <= 1; s += 2) {
    f32 wx = cx + (f32)s * 4.6f;
    town_window(t, wx, 1.6f, fz, 5.0f, 1.6f, true, 1.0f);
    for (i32 i = 0; i < 3; i++)
      town_box(t, i % 2 ? C_WHITE : C_RED, 5.4f, 0.3f, 0.6f, wx, 3.6f + (f32)i * 0.25f, fz + 0.4f + (f32)i * 0.35f, false);
  }
  // round windows
  f32 rw[] = {-6.0f, -3.5f, 3.5f, 6.0f};
  for (i32 i = 0; i < 4; i++) {
    MeshBuilderStyle ring = town_style(C_DARK_GRAY);
    quat up = quat_from_euler((v3){M_PI * 0.5f, 0.0f, 0.0f});
    mesh_builder_cylinder(t->b, &ring, m4_trs(town_m(cx + rw[i], 5.8f, fz + 0.1f), up, V3_ONE),
                          &(PgCylinderDesc){.radius_top = 0.85f * TOWN_STUD, .radius_bottom = 0.85f * TOWN_STUD,
                                            .height = 0.3f * TOWN_STUD, .radial_segments = 14});
    MeshBuilderStyle glass = town_style(0x40b8d8u);
    mesh_builder_cylinder(t->b, &glass, m4_trs(town_m(cx + rw[i], 5.8f, fz + 0.18f), up, V3_ONE),
                          &(PgCylinderDesc){.radius_top = 0.65f * TOWN_STUD, .radius_bottom = 0.65f * TOWN_STUD,
                                            .height = 0.3f * TOWN_STUD, .radial_segments = 14});
  }
  // flat roof with white tiles and a rim with studs
  town_box(t, C_DARK_GRAY, w + 1.0f, 0.6f, d + 1.0f, cx, 0.4f + H, cz, false);
  for (f32 x = x0 + 1.0f; x < x1 - 0.5f; x += 2.5f)
    for (f32 z = z0 + 1.0f; z < z1 - 0.5f; z += 2.5f)
      town_box(t, C_WHITE, 2.3f, 0.25f, 2.3f, x + 1.15f, 0.4f + H + 0.6f, z + 1.15f, false);
  town_box(t, C_GRAY, w + 1.0f, 0.8f, 0.6f, cx, 0.4f + H + 0.6f, z0 - 0.2f, true);
  town_box(t, C_GRAY, w + 1.0f, 0.8f, 0.6f, cx, 0.4f + H + 0.6f, z1 + 0.2f, true);
  town_box(t, C_GRAY, 0.6f, 0.8f, d, x0 - 0.2f, 0.4f + H + 0.6f, cz, false);
  town_box(t, C_GRAY, 0.6f, 0.8f, d, x1 + 0.2f, 0.4f + H + 0.6f, cz, false);
  // solar panels at the back of the roof
  for (i32 i = 0; i < 3; i++) {
    f32 px = x0 + 3.0f + (f32)i * 3.4f;
    town_box_rot(t, 0x1d3f8au, (v3){3.0f, 0.2f, 2.0f}, (v3){px, 0.4f + H + 1.6f, z0 + 2.0f}, (v3){0.45f, 0.0f, 0.0f});
    town_box(t, C_DARK_GRAY, 0.3f, 0.9f, 0.3f, px, 0.4f + H + 0.8f, z0 + 1.6f, false);
  }
  // chimney with a hole
  town_box(t, C_RED_BROWN, 2.6f, 2.4f, 2.6f, x1 - 2.8f, 0.4f + H + 0.6f, z0 + 2.8f, false);
  town_box(t, C_BLACK, 1.4f, 0.1f, 1.4f, x1 - 2.8f, 0.4f + H + 3.0f, z0 + 2.8f, false);
  // satellite dish on a pole
  town_box(t, C_DARK_GRAY, 0.4f, 1.6f, 0.4f, cx + 1.0f, 0.4f + H + 0.6f, cz, false);
  MeshBuilderStyle dish = town_style(C_WHITE);
  v3 rim_c = town_m(cx + 1.0f, 0.4f + H + 3.0f, cz + 0.6f);
  mesh_builder_cone_between(t->b, &dish, rim_c, town_m(cx + 1.0f, 0.4f + H + 2.3f, cz - 0.2f), 1.4f * TOWN_STUD, 14);
  // the lab sign: a blue board with a white frame over the door
  town_box(t, C_WHITE, 5.4f, 2.1f, 0.15f, cx, 4.7f, fz + 0.02f, false);
  town_box(t, 0x1e5aa8u, 5.0f, 1.7f, 0.1f, cx, 4.9f, fz + 0.1f, false);
  town_prop(t, "brickhollow/signs/lab.glb", cx, fz + 0.17f, 5.75f, 1.0f, 0.0f, false);
}

// a white picket fence from (x0, z0) to (x1, z1): a post every stud, two rails, one collider
hz_internal void town_fence(Town *t, f32 x0, f32 z0, f32 x1, f32 z1) {
  f32 len = m_sqrtf((x1 - x0) * (x1 - x0) + (z1 - z0) * (z1 - z0));
  i32 n = (i32)m_maxf(1.0f, m_roundf(len));
  for (i32 i = 0; i <= n; i++) {
    f32 k = (f32)i / (f32)n;
    town_box(t, C_WHITE, 0.22f, 1.5f, 0.22f, x0 + (x1 - x0) * k, 0.0f, z0 + (z1 - z0) * k, false);
  }
  b32 along_x = m_absf(x1 - x0) > m_absf(z1 - z0);
  f32 mx = (x0 + x1) * 0.5f, mz = (z0 + z1) * 0.5f;
  for (i32 r = 0; r < 2; r++)
    town_box(t, C_WHITE, along_x ? len : 0.18f, 0.25f, along_x ? 0.18f : len, mx, 0.5f + 0.6f * (f32)r, mz, false);
  town_block(t, m_minf(x0, x1) - 0.35f, m_maxf(x0, x1) + 0.35f, m_minf(z0, z1) - 0.35f, m_maxf(z0, z1) + 0.35f);
}

// a sign: posts and a board (colour of the board); facing +z
hz_internal void town_sign_board(Town *t, f32 x, f32 z, f32 w, f32 h, f32 y, u32 board) {
  town_box(t, C_WHITE, w + 0.2f, h + 0.2f, 0.25f, x, y - 0.1f, z, false);
  town_box(t, board, w - 0.2f, h - 0.2f, 0.1f, x, y + 0.1f, z + 0.15f, false);
}

// a lamp post with a pale bulb and studs on the cap
hz_internal void town_lamp(Town *t, f32 x, f32 z) {
  town_box(t, C_BLACK, 0.35f, 5.0f, 0.35f, x, 0.0f, z, false);
  town_box(t, C_BLACK, 0.9f, 0.2f, 0.9f, x, 5.0f, z, false);
  town_box(t, C_BLACK, 1.0f, 0.25f, 1.0f, x, 6.0f, z, true);
  town_block(t, x - 0.4f, x + 0.4f, z - 0.4f, z + 0.4f);
}

// a pile of loose LEGO bricks (real brick sizes, studs on top), resting on each other
hz_internal void town_brick_pile(Town *t, u32 seed, f32 cx, f32 cz, u32 color) {
  TownRng rng = {.s = seed * 9973u + 17u};
  f32 sizes[8][2] = {{2, 4}, {2, 2}, {2, 4}, {1, 2}, {1, 4}, {2, 3}, {2, 2}, {1, 1}};
  u32 extra[] = {C_WHITE, C_YELLOW, C_BLUE, C_RED, C_GREEN, C_ORANGE};
  f32 px[16], pz[16], pr[16], ptop[16];
  i32 placed = 0;
  const f32 R = 3.6f;
  for (i32 i = 0; i < 16; i++) {
    u32 si = (u32)(town_rand(&rng) * 8.0f) % 8u;
    f32 w = sizes[si][0], d = sizes[si][1];
    u32 col = town_rand(&rng) < 0.7f ? color : extra[(u32)(town_rand(&rng) * 6.0f) % 6u];
    f32 my_r = m_sqrtf(w * w + d * d) * 0.5f;
    f32 x = 0.0f, z = 0.0f, y = 99.0f;
    for (i32 tries = 0; tries < 8 && y > 1.6f; tries++) {
      f32 ang = town_rand(&rng) * 6.283f, dist = m_sqrtf(town_rand(&rng)) * R;
      x = m_cosf(ang) * dist;
      z = m_sinf(ang) * dist;
      y = 0.0f;
      for (i32 p = 0; p < placed; p++) {
        f32 dx = px[p] - x, dz = pz[p] - z;
        if (m_sqrtf(dx * dx + dz * dz) < (pr[p] + my_r) * 0.85f)
          y = m_maxf(y, ptop[p]);
      }
    }
    if (y > 2.0f)
      continue;
    f32 yaw = town_rand(&rng) * M_PI;
    quat q = quat_from_euler((v3){0.0f, yaw, 0.0f});
    MeshBuilderStyle st = town_style(col);
    mesh_builder_box(t->b, &st, m4_trs(town_m(cx + x, y + 0.6f, cz + z), q, V3_ONE), town_m(w, 1.2f, d));
    for (i32 sx = 0; sx < (i32)w; sx++)
      for (i32 sz = 0; sz < (i32)d; sz++) {
        v3 local = {(f32)sx - (w - 1.0f) * 0.5f, 0.0f, (f32)sz - (d - 1.0f) * 0.5f};
        v3 rot = quat_rotate(q, local);
        town_stud(t, col, cx + x + rot.x, y + 1.2f, cz + z + rot.z);
      }
    px[placed] = x;
    pz[placed] = z;
    pr[placed] = my_r;
    ptop[placed] = y + 1.2f + TOWN_STUD_H;
    placed++;
  }
  town_block(t, cx - R + 0.3f, cx + R - 0.3f, cz - R + 0.3f, cz + R - 0.3f);
}

// a LEGO duck (body, head, orange beak, tail) in `hex`; yaw 0 looks +x
hz_internal void town_duck(Town *t, f32 x, f32 z, f32 s, f32 yaw, f32 y, u32 hex) {
  quat q = quat_from_euler((v3){0.0f, yaw, 0.0f});
  MeshBuilderStyle body = town_style(hex);
  body.flat = false;
  v3 fwd = quat_rotate(q, (v3){1, 0, 0});
  mesh_builder_sphere(t->b, &body, m4_trs(town_m(x, y + 0.45f * s, z), q, town_m(0.75f * s, 0.45f * s, 0.5f * s)), 10, 6);
  v3 head = {x + fwd.x * 0.5f * s, y + 1.0f * s, z + fwd.z * 0.5f * s};
  mesh_builder_sphere(t->b, &body, m4_trs(town_m(head.x, head.y, head.z), q, town_m(0.33f * s, 0.33f * s, 0.33f * s)), 8, 6);
  town_box_rot(t, C_ORANGE, (v3){0.4f * s, 0.12f * s, 0.25f * s},
               (v3){head.x + fwd.x * 0.35f * s, head.y - 0.05f * s, head.z + fwd.z * 0.35f * s}, (v3){0.0f, yaw, 0.0f});
  town_box_rot(t, town_shade(hex, 0.9f), (v3){0.3f * s, 0.12f * s, 0.3f * s},
               (v3){x - fwd.x * 0.7f * s, y + 0.7f * s, z - fwd.z * 0.7f * s}, (v3){0.0f, yaw, 0.0f});
}

// a LEGO cloud: stacked white plates
hz_internal void town_cloud(Town *t, f32 x, f32 y, f32 z, f32 s) {
  town_box(t, C_WHITE, 8.0f * s, 1.2f * s, 4.0f * s, x, y, z, false);
  town_box(t, C_WHITE, 5.0f * s, 1.2f * s, 3.0f * s, x - 0.8f * s, y + 1.2f * s, z, false);
  town_box(t, C_WHITE, 3.0f * s, 1.0f * s, 2.0f * s, x + 1.2f * s, y + 2.4f * s, z, false);
  town_box(t, C_WHITE, 4.0f * s, 0.8f * s, 2.6f * s, x + 4.6f * s, y + 0.2f * s, z + 0.4f * s, false);
}

// ---------------------------------------------------------------- the village

#include "st_rooms.c"
#include "st_cave.c"

// main lane, once: the whole village into one slot; the parts become static renderer entities
hz_internal void town_build(EcsWorld *w, RendererState *r, Allocator *alloc, TownRect *out_blocks, u32 *out_count, u32 cap,
                            TownGroups *groups, TownTag *tags, u32 *tag_count) {
  ArenaAllocator *arena = arena_create(MB(128), MB(1));
  Allocator garena = make_arena_allocator(arena);
  MeshBuilder *mbs = ALLOC_ARRAY(&garena, MeshBuilder, TOWN_GROUPS);
  for (u32 g = 0; g < TOWN_GROUPS; g++)
    mesh_builder_begin(&mbs[g], arena, &(MeshBuilderDesc){.slot_count = 1, .vtx_reserve = g ? 1u << 15 : 1u << 18,
                                                          .idx_reserve = g ? 1u << 16 : 1u << 19});
  Allocator scratch = make_arena_allocator(arena);
  Town town = {.b = &mbs[0],
               .mbs = mbs,
               .tags = tags,
               .groups = groups,
               .rng = {.s = 20261114u},
               .world = w,
               .renderer = r,
               .assets = ecs_resource_get_mut(w, AssetSystem),
               .used = ALLOC_ARRAY(&scratch, TownRect, TOWN_MAX_RECTS),
               .solid = ALLOC_ARRAY(&scratch, TownRect, TOWN_MAX_RECTS)};
  Town *t = &town;

  // ---- roads: grey plates with studs, solid so the figure steps up onto them ----
  for (i32 i = 0; i < TOWN_ROAD_COUNT; i++) {
    TownRect rd = town_road(i);
    MeshBuilderStyle st = town_style(C_GRAY);
    st.collider = MESH_BUILDER_COLLIDER_BOX;
    mesh_builder_box(t->b, &st, m4_trs(town_m((rd.x0 + rd.x1) * 0.5f, 0.2f, (rd.z0 + rd.z1) * 0.5f), quat_identity(), V3_ONE),
                     town_m(rd.x1 - rd.x0, 0.4f, rd.z1 - rd.z0));
    for (f32 x = rd.x0 + 0.5f; x < rd.x1; x += 1.0f)
      for (f32 z = rd.z0 + 0.5f; z < rd.z1; z += 1.0f) {
        // no studs where a road crosses another (each crossing has them once) or under the houses' steps
        b32 dup = false;
        for (i32 k = 0; k < i; k++) {
          TownRect o = town_road(k);
          if (x > o.x0 && x < o.x1 && z > o.z0 && z < o.z1)
            dup = true;
        }
        if (!dup)
          town_stud(t, C_GRAY, x, 0.4f, z);
      }
  }

  // ---- the houses: yours (west, blue roof) and Dex & Lumi's (east, red roof, yellow walls) ----
  town_house(t, -15.0f, -11.0f, 12.0f, 10.0f, C_BLUE, C_WHITE, C_RED_BROWN, C_BLUE, "brickhollow/signs/house_home.glb");
  town_house(t, 47.0f, -12.0f, 16.0f, 12.0f, C_RED, C_YELLOW, C_BLUE, C_RED, "brickhollow/signs/house_dex.glb");
  // Dex's garage next to it, with a grey door and a driveway plate
  town_box(t, C_WHITE, 8.0f, 5.6f, 8.0f, 35.0f, 0.4f, -10.0f, false);
  town_box(t, C_GRAY, 9.0f, 0.4f, 9.0f, 35.0f, 0.0f, -10.0f, false);
  town_box(t, C_DARK_GRAY, 6.0f, 4.0f, 0.3f, 35.0f, 0.4f, -5.9f, false);
  for (i32 i = 0; i < 4; i++)
    town_box(t, C_GRAY, 6.0f, 0.1f, 0.35f, 35.0f, 1.2f + (f32)i * 0.9f, -5.75f, false);
  town_box(t, C_RED, 9.0f, 0.6f, 9.0f, 35.0f, 6.0f, -10.0f, true);
  town_block_h(t, 30.4f, 39.6f, -14.6f, -5.5f, 6.6f * TOWN_STUD);
  town_box(t, C_GRAY, 6.0f, 0.4f, 1.0f, 35.0f, 0.0f, -5.0f, false);

  // ---- the little park where Dex's old house used to be: trees, flowers, a bench ----
  {
    f32 cx = 15.0f, cz = -11.0f;
    f32 tr[3][2] = {{-4.5f, -2.5f}, {4.5f, -3.0f}, {0.0f, -4.5f}};
    for (i32 i = 0; i < 3; i++) {
      town_round_tree(t, cx + tr[i][0], cz + tr[i][1], 5.5f + town_rand(&t->rng));
      town_block(t, cx + tr[i][0] - 0.9f, cx + tr[i][0] + 0.9f, cz + tr[i][1] - 0.9f, cz + tr[i][1] + 0.9f);
    }
    town_flower_pot(t, cx - 5.5f, cz + 3.0f, false);
    town_flower_pot(t, cx + 5.5f, cz + 3.0f, false);
    town_box(t, C_RED_BROWN, 3.2f, 0.4f, 1.2f, cx, 1.0f, cz + 1.5f, true);
    town_box(t, C_RED_BROWN, 3.2f, 1.2f, 0.3f, cx, 1.4f, cz + 0.95f, false);
    for (i32 s = -1; s <= 1; s += 2)
      town_box(t, C_DARK_GRAY, 0.4f, 1.0f, 1.0f, cx + (f32)s * 1.3f, 0.0f, cz + 1.5f, false);
    town_block(t, cx - 1.7f, cx + 1.7f, cz + 0.8f, cz + 2.2f);
  }

  // ---- loose LEGO brick piles ----
  town_brick_pile(t, 0, 45.0f, 16.5f, C_RED);
  town_brick_pile(t, 2, 57.5f, 16.5f, C_YELLOW);
  town_brick_pile(t, 3, 33.5f, 17.0f, C_BLUE);
  town_brick_pile(t, 1, -17.0f, -21.5f, C_ORANGE);

  // ---- the lab ----
  town_lab(t, 13.0f, 11.0f, 16.0f, 10.0f);

  // ---- the garden: a white fence with a gate, rows of flowers, a sign ----
  {
    f32 x0 = -25.0f, x1 = -13.0f, z0 = 7.0f, z1 = 17.0f;
    // the gate: 4 studs wide around x -19 (the path from the road), the posts outside it
    town_fence(t, x0, z0, -21.2f, z0);
    town_fence(t, -16.8f, z0, x1, z0);
    town_fence(t, x0, z1, x1, z1);
    town_fence(t, x0, z0, x0, z1);
    town_fence(t, x1, z0, x1, z1);
    town_box(t, C_WHITE, 0.8f, 2.0f, 0.8f, -21.4f, 0.0f, z0, true);
    town_box(t, C_WHITE, 0.8f, 2.0f, 0.8f, -16.6f, 0.0f, z0, true);
    u32 cols[] = {C_RED, C_RED, C_YELLOW, C_PINK, C_WHITE, C_ORANGE};
    for (f32 x = x0 + 1.6f; x < x1 - 1.0f; x += 2.2f)
      for (f32 z = z0 + 1.9f; z < z1 - 1.0f; z += 2.2f) {
        if (town_rand(&t->rng) < 0.15f)
          continue;
        if (z < 10.5f && x < -14.0f) // the sign's row: its readers stand here and look at it (and the way in)
          continue;
        town_flower(t, x + (town_rand(&t->rng) - 0.5f) * 0.3f, z + (town_rand(&t->rng) - 0.5f) * 0.3f,
                    cols[(u32)(town_rand(&t->rng) * 6.0f) % 6u], 0.0f);
      }
    town_box(t, C_BROWN, 0.3f, 2.6f, 0.3f, -23.2f, 0.0f, 8.9f, false);
    town_box(t, C_TAN, 0.15f, 1.5f, 2.8f, -23.0f, 1.35f, 8.9f, false);
    town_prop(t, "brickhollow/signs/garden.glb", -22.91f, 8.9f, 2.1f, 1.0f, M_PI * 0.5f, false); // faces the gate (+x)
    town_block(t, -23.4f, -22.9f, 7.6f, 10.2f);
  }

  // ---- the pond: a brown border with studs, a dark bed, water, lily pads, a fish and the duck family ----
  {
    f32 x0 = -12.0f, x1 = -4.0f, z0 = 15.0f, z1 = 22.0f;
    // a low rim (0.5 m: the walk steps 0.35): you jump in. With the diving suit you sink into the cave, without it
    // you climb back out (starting_template.c); the ducks, the ball and Pip keep out of it as before
    town_block_h(t, x0 - 0.4f, x1 + 0.4f, z0 - 0.4f, z1 + 0.4f, 0.5f);
    town_box(t, C_RED_BROWN, x1 - x0 + 1.0f, 0.4f, 0.6f, (x0 + x1) * 0.5f, 0.0f, z0 - 0.2f, true);
    town_box(t, C_RED_BROWN, x1 - x0 + 1.0f, 0.4f, 0.6f, (x0 + x1) * 0.5f, 0.0f, z1 + 0.2f, true);
    town_box(t, C_RED_BROWN, 0.6f, 0.4f, z1 - z0, x0 - 0.2f, 0.0f, (z0 + z1) * 0.5f, true);
    town_box(t, C_RED_BROWN, 0.6f, 0.4f, z1 - z0, x1 + 0.2f, 0.0f, (z0 + z1) * 0.5f, true);
    town_box(t, 0x1d4a5eu, x1 - x0, 0.05f, z1 - z0, (x0 + x1) * 0.5f, 0.0f, (z0 + z1) * 0.5f, false);
    town_box(t, C_WATER, x1 - x0, 0.1f, z1 - z0, (x0 + x1) * 0.5f, 0.2f, (z0 + z1) * 0.5f, false);
    f32 pads[3][2] = {{-10.0f, 17.0f}, {-6.5f, 20.0f}, {-8.0f, 18.8f}};
    for (i32 i = 0; i < 3; i++)
      town_cyl(t, C_GREEN, pads[i][0], 0.3f, pads[i][1], 0.6f, 0.6f, 0.08f, 12, false);
    town_flower(t, -6.5f, 20.0f, C_PINK, 0.38f);
    // (the fish and the duck family swim: st_life.c)
    town_box(t, C_RED_BROWN, 3.0f, 0.3f, 1.0f, -7.5f, 0.9f, 22.4f + 0.0f, false); // bench near the pond
    town_box(t, C_DARK_GRAY, 0.4f, 0.9f, 0.8f, -8.7f, 0.0f, 22.4f, false);
    town_box(t, C_DARK_GRAY, 0.4f, 0.9f, 0.8f, -6.3f, 0.0f, 22.4f, false);
  }

  // ---- signs ----
  town_box(t, C_BROWN, 0.35f, 3.2f, 0.35f, 4.4f, 0.0f, 20.0f, false);
  town_box(t, C_BROWN, 0.35f, 3.2f, 0.35f, 7.4f, 0.0f, 20.0f, false);
  town_sign_board(t, 5.9f, 20.2f, 3.8f, 2.0f, 1.3f, C_WHITE);
  town_prop(t, "brickhollow/signs/village.glb", 5.9f, 20.41f, 2.3f, 1.0f, 0.0f, false);
  town_block(t, 4.0f, 7.8f, 19.6f, 20.6f);
  town_box(t, C_BROWN, 0.35f, 3.2f, 0.35f, 5.0f, 0.0f, -24.5f, false);
  town_sign_board(t, 5.0f, -24.3f, 3.4f, 1.8f, 1.3f, 0xfff6c8u);
  town_prop(t, "brickhollow/signs/route.glb", 5.0f, -24.09f, 2.2f, 1.0f, 0.0f, false);
  town_block(t, 4.5f, 5.5f, -25.0f, -24.0f);

  // ---- lamp posts ----
  f32 lamps[6][2] = {{-4, -4}, {4, 4}, {-4, 10}, {4, -14}, {-22, 4}, {22, -3.5f}};
  for (i32 i = 0; i < 6; i++)
    town_lamp(t, lamps[i][0], lamps[i][1]);

  // ---- round trees inside the village ----
  f32 rt[6][2] = {{22, 20}, {-24, -20}, {8, -20}, {-9, -20}, {24, 4}, {21, -21}};
  for (i32 i = 0; i < 6; i++) {
    town_round_tree(t, rt[i][0], rt[i][1], 6.5f);
    town_block(t, rt[i][0] - 0.9f, rt[i][0] + 0.9f, rt[i][1] - 0.9f, rt[i][1] + 0.9f);
  }

  // ---- bamboo leaves around the pond, along the walls, by the garden and the exit ----
  f32 lv[][3] = {{-13, 15.5f, 1.6f}, {-13, 21, 1.7f}, {-3.4f, 17, 1.5f}, {-12.6f, 23.6f, 1.6f}, {-8, 23.8f, 1.6f},
                 {-20.6f, -5.2f, 1.2f}, {-18.6f, -5.2f, 1.2f}, {-11.6f, -5.2f, 1.2f},
                 {35, 10.6f, 1.5f}, {39, 10.6f, 1.5f}, {59, 10.6f, 1.5f}, {62, 10.6f, 1.5f},
                 {6, 17, 1.25f}, {18, 17, 1.25f}, {20, 17, 1.25f},
                 {-24, 8.5f, 1.3f}, {-24, 15.6f, 1.3f}, {-14, 15.6f, 1.3f},
                 {-25.5f, -4, 1.8f}, {25.5f, 8, 1.8f}, {-25.5f, 18, 1.8f}, {25.5f, -18, 1.8f}, {6.5f, -25.6f, 1.8f},
                 {-6.5f, -25.6f, 1.8f}};
  for (u32 i = 0; i < sizeof(lv) / sizeof(lv[0]); i++)
    town_leaves(t, lv[i][0], lv[i][1], lv[i][2]);

  // ---- flower beds, single flowers and flower pots ----
  for (f32 x = 4.2f; x <= 6.3f; x += 2.1f)
    town_flower(t, x, 18.6f, town_rand(&t->rng) < 0.5f ? C_RED : C_YELLOW, 0.0f);
  for (f32 x = 19.7f; x <= 21.8f; x += 2.1f)
    town_flower(t, x, 18.6f, town_rand(&t->rng) < 0.5f ? C_RED : C_YELLOW, 0.0f);
  for (f32 x = -11.5f; x <= -4.8f; x += 2.1f)
    town_flower(t, x, 23.7f, town_rand(&t->rng) < 0.5f ? C_PINK : C_WHITE, 0.0f);
  f32 fl[3][2] = {{8, -9.5f}, {8, -16}, {22, -16}};
  for (i32 i = 0; i < 3; i++)
    town_flower(t, fl[i][0], fl[i][1], C_ORANGE, 0.0f);
  town_flower_pot(t, -11.8f, -4.3f, true);
  town_flower_pot(t, -22.5f, -4.3f, true);
  town_flower_pot(t, 23.5f, 12.0f, true);
  town_flower_pot(t, 55.5f, -4.6f, true);
  // crates by the lab
  town_box(t, C_BROWN, 1.6f, 1.6f, 1.6f, 22.6f, 0.0f, 9.0f, true);
  town_box(t, C_TAN, 1.6f, 1.6f, 1.6f, 22.6f, 1.6f, 9.0f, true);
  town_block(t, 21.6f, 23.6f, 8.0f, 10.0f);

  // ---- tall grass north of the village gap (Route 1 starts here) ----
  {
    u32 greens[] = {0x58ab41u, C_GREEN, 0x7fc04au, 0x3c8c3au};
    i32 n = 0;
    for (f32 x = -8.6f + 0.5f; x < 8.6f; x += 1.05f)
      for (f32 z = -42.0f + 0.5f; z < -28.2f; z += 1.05f)
        town_grass_tuft(t, x + (town_rand(&t->rng) - 0.5f) * 0.4f, z + (town_rand(&t->rng) - 0.5f) * 0.4f,
                        0.8f + town_rand(&t->rng) * 0.5f, greens[(n++) % 4]);
  }

  // ---- trees: the borders around the village (two staggered rows), the line between village and route ----
  {
    const f32 step = 3.6f;
    for (f32 z = -45.0f; z <= 27.0f; z += step) {
      f32 xs[4] = {-30.3f, -27.4f, 70.3f, 67.4f};
      for (i32 k = 0; k < 4; k++) {
        f32 x = xs[k] + (town_rand(&t->rng) - 0.5f) * 0.8f, zz = z + (k % 2 ? step * 0.5f : 0.0f);
        if (town_rand(&t->rng) < 0.25f)
          town_round_tree(t, x, zz, 6.0f + town_rand(&t->rng) * 1.5f);
        else
          town_spruce(t, x, zz, 8.5f + town_rand(&t->rng) * 2.5f);
      }
    }
    for (f32 x = -24.0f; x <= 64.0f; x += step) {
      town_round_tree(t, x + (town_rand(&t->rng) - 0.5f) * 0.8f, 27.2f, 3.0f + town_rand(&t->rng) * 0.6f);
      town_bush(t, x + step * 0.5f, 25.0f, 1.6f + town_rand(&t->rng) * 0.5f);
    }
    for (f32 x = -25.0f; x <= 25.0f; x += step) {
      if (m_absf(x) > 6.0f)
        town_spruce(t, x, -27.6f, 8.5f + town_rand(&t->rng) * 2.5f);
      if (m_absf(x + step * 0.5f) > 11.0f)
        town_spruce(t, x + step * 0.5f, -30.2f, 8.5f + town_rand(&t->rng) * 2.5f);
    }
    for (f32 x = 28.0f; x <= 66.0f; x += step) {
      town_spruce(t, x, -29.6f, 9.0f + town_rand(&t->rng) * 2.0f);
      town_spruce(t, x + step * 0.5f, -32.4f, 9.0f + town_rand(&t->rng) * 2.0f);
    }
    for (f32 x = 11.5f; x <= 27.0f; x += step)
      for (f32 z = -44.5f; z <= -32.0f; z += step) {
        town_spruce(t, x, z, 8.5f + town_rand(&t->rng) * 2.5f);
        town_spruce(t, -x, z, 8.5f + town_rand(&t->rng) * 2.5f);
      }
    for (f32 x = -9.0f; x <= 9.0f; x += step) {
      if (m_absf(x) > 4.5f)
        town_spruce(t, x, -45.0f, 9.5f);
      if (m_absf(x + step * 0.5f) > 4.5f)
        town_spruce(t, x + step * 0.5f, -43.2f, 9.5f);
    }
    f32 bushes[][2] = {{-7.2f, -26.2f}, {7.2f, -26.2f}, {-24, 22}, {24, 22}, {-24, -24}, {-9, 22.5f},
                       {18, 22.5f},     {-26, 6},       {24, 14}, {64, 14}, {64, -24}};
    for (u32 i = 0; i < sizeof(bushes) / sizeof(bushes[0]); i++) {
      town_bush(t, bushes[i][0], bushes[i][1], 2.4f);
      town_block(t, bushes[i][0] - 1.1f, bushes[i][0] + 1.1f, bushes[i][1] - 1.1f, bushes[i][1] + 1.1f);
    }
  }

  // ---- the villagers who stand still (st_minifig.c spawns them): Dex, Prof. Quill at the gap, Mom and Lumi and the
  // Professor in the rooms; Pip walks around (st_life.c) ----
  f32 npcs[5][2] = {{52.5f, -3.4f}, {-5.4f, -24.2f}, {193.4f, -4.4f}, {254.5f, 2.0f}, {337.0f, -1.2f}};
  for (i32 i = 0; i < 5; i++)
    town_block(t, npcs[i][0] - 0.9f, npcs[i][0] + 0.9f, npcs[i][1] - 0.7f, npcs[i][1] + 0.7f);

  // ---- the little football goal east of the park: open to the west, the sides and the back block ----
  {
    f32 x0 = 24.5f, x1 = 27.0f, z0 = -7.6f, z1 = -4.4f;
    for (i32 k = 0; k < 2; k++) {
      f32 z = k ? z1 : z0;
      town_box(t, C_WHITE, 0.3f, 2.6f, 0.3f, x0, 0.0f, z, false);
      town_box(t, C_WHITE, 0.3f, 2.6f, 0.3f, x1, 0.0f, z, false);
      town_box(t, C_WHITE, x1 - x0, 0.25f, 0.25f, (x0 + x1) * 0.5f, 2.5f, z, false); // side top bar
      for (f32 x = x0 + 0.5f; x < x1; x += 0.5f)
        town_box(t, 0xdfe6eau, 0.05f, 2.5f, 0.05f, x, 0.0f, z, false);                // side net
      town_block(t, x0, x1, z - 0.2f, z + 0.2f);
    }
    town_box(t, C_WHITE, 0.25f, 0.25f, z1 - z0, x0, 2.5f, (z0 + z1) * 0.5f, false); // crossbar
    for (f32 z = z0 + 0.4f; z < z1; z += 0.4f)
      town_box(t, 0xdfe6eau, 0.05f, 2.5f, 0.05f, x1, 0.0f, z, false);                // back net
    for (f32 y = 0.5f; y < 2.5f; y += 0.5f)
      town_box(t, 0xdfe6eau, 0.05f, 0.05f, z1 - z0, x1, y, (z0 + z1) * 0.5f, false);
    town_block(t, x1 - 0.2f, x1 + 0.2f, z0, z1);
  }

  // ---- names for the debug picker ----
  town_tag(t, "Your house", -15.0f, -11.0f, 7.5f);
  town_tag(t, "Dex & Lumi's house", 47.0f, -12.0f, 9.5f);
  town_tag(t, "Dex's garage", 35.0f, -10.0f, 5.0f);
  town_tag(t, "Quill Lab", 13.0f, 11.0f, 9.0f);
  town_tag(t, "The park bench", 15.0f, -9.5f, 2.0f);
  town_tag(t, "The park", 15.0f, -11.0f, 7.0f);
  town_tag(t, "The garden (fence)", -19.0f, 12.0f, 7.5f);
  town_tag(t, "The pond", -8.0f, 18.5f, 5.0f);
  town_tag(t, "The football goal", 25.75f, -6.0f, 2.2f);
  town_tag(t, "Brick pile (red)", 45.0f, 16.5f, 3.8f);
  town_tag(t, "Brick pile (yellow)", 57.5f, 16.5f, 3.8f);
  town_tag(t, "Brick pile (blue)", 33.5f, 17.0f, 3.8f);
  town_tag(t, "Brick pile (orange)", -17.0f, -21.5f, 3.8f);
  town_tag(t, "Village sign", 5.9f, 20.2f, 2.2f);
  town_tag(t, "Route 1 sign", 5.0f, -24.3f, 2.0f);
  town_tag(t, "Garden sign", -23.0f, 8.9f, 1.8f);
  town_tag(t, "Your mailbox", -18.2f, -3.8f, 1.0f);
  town_tag(t, "Dex's mailbox", 43.8f, -3.8f, 1.0f);
  town_tag(t, "Crates", 22.6f, 9.0f, 1.3f);
  town_tag(t, "Tall grass (Route 1)", 0.0f, -35.0f, 9.0f);
  for (i32 i = 0; i < 6; i++)
    town_tag(t, "Lamp post", lamps[i][0], lamps[i][1], 0.8f);
  {
    CString room_names[ROOM_COUNT] = {"Home (inside)", "Dex & Lumi's house (inside)", "Quill Lab (inside)", "Your room (upstairs)"};
    for (i32 i = 0; i < ROOM_COUNT; i++) {
      RoomDef rd = room_def((RoomId)i);
      town_tag(t, room_names[i], rd.ox, rd.oz, rd.w * 0.75f);
    }
  }

  // ---- the rooms, far east of the village (st_rooms.c) ----
  rooms_build(t);
  cave_build(t); // the flooded cave under the pond, far east too (st_cave.c)
  town_group_switch(t, 0);

  // ---- the wild LEGO plants wherever there is room (the garden inside its fence stays as it is) ----
  town_occupy(t, -25.0f, -13.0f, 7.0f, 17.0f);
  town_wild_plants(t, &scratch);

  // ---- LEGO clouds ----
  town_cloud(t, -10.0f, 32.0f, -60.0f, 1.6f);
  town_cloud(t, 30.0f, 36.0f, -70.0f, 2.0f);
  town_cloud(t, 70.0f, 30.0f, -50.0f, 1.4f);
  town_cloud(t, -45.0f, 34.0f, -20.0f, 1.5f);

  // ---- where you can walk: the village, the gap north and the grass patch (walls along the edges) ----
  {
    TownRect walls[] = {
        {-26.2f, 66.2f, 23.4f, 24.4f},    // south
        {-27.2f, -26.2f, -25.6f, 23.4f},  // west
        {66.2f, 67.2f, -25.6f, 23.4f},    // east
        {-26.2f, -3.6f, -26.6f, -25.6f},  // north, west of the gap
        {3.6f, 66.2f, -26.6f, -25.6f},    // north, east of the gap
        {-4.6f, -3.6f, -28.6f, -25.6f},   // the gap
        {3.6f, 4.6f, -28.6f, -25.6f},
        {-8.4f, -3.6f, -29.6f, -28.6f},   // the grass patch
        {3.6f, 8.4f, -29.6f, -28.6f},
        {-9.4f, -8.4f, -41.6f, -28.6f},
        {8.4f, 9.4f, -41.6f, -28.6f},
        {-9.4f, 9.4f, -42.6f, -41.6f},
    };
    for (u32 i = 0; i < sizeof(walls) / sizeof(walls[0]); i++)
      town_block(t, walls[i].x0, walls[i].x1, walls[i].z0, walls[i].z1);
  }

  HzHandleT(RendererMaterial) mat = renderer_default_material(r);
  for (u32 g = 0; g < TOWN_GROUPS; g++) {
    MeshBuilderResult built = mesh_builder_finish(&mbs[g], &(MeshBuilderFinishDesc){.renderer = r, .world = w, .alloc = alloc});
    const MeshBuilderSlotOut *slot = &built.slots[TOWN_SLOT];
    for (u32 i = 0; i < slot->part_count; i++)
      town_group_add(groups, g,
                     ecs_renderer_spawn(w, r,
                                        &(EcsRendererSpawnDesc){.submesh = slot->parts[i],
                                                                .material = mat,
                                                                .transform = {.position = V3_ZERO, .rotation = quat_identity(), .scale = V3_ONE},
                                                                .casts_shadow = true,
                                                                .static_shadow = true,
                                                                .name = g ? "Room" : "Town"}));
    log_info("town group %: % vertices, % tris, % parts, % colliders", fmt_u32(g), fmt_u32(built.vertex_count),
             fmt_u32(slot->tri_count), fmt_u32(slot->part_count), fmt_u32(built.collider_count));
  }
  *tag_count = t->tag_count;
  *out_count = MIN(t->solid_count, cap);
  for (u32 i = 0; i < *out_count; i++)
    out_blocks[i] = t->solid[i];
  arena_destroy(arena);
}
