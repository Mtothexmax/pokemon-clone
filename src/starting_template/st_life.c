// Life in the village (the three.js town's update loop, ported from its behaviour spec): golden studs, the duck family
// (and feeding it), the jumping fish, butterflies, fireflies at night, the ball and the goal (Pip plays along), the
// treasure chest, rain and puddles, the day/night clock with glowing lamps, sleeping, the park bench, dancing, and
// Pip's hide and seek and tag. Main lane, before the scope. Coordinates in studs unless named _m.
// Included by starting_template.c above app_init.

#define GOLDEN_COUNT 15
#define DUCKS 4
#define CRUMBS 5
#define FLIES 8
#define FIREFLIES 30
#define RAIN_DROPS 160
#define PUDDLES 14
#define LAMPS 6
#define LIFE_MAX_BLOCKS 1024
#define DAY_SECONDS 360.0f // a whole day, as in the three.js version
#define STUD_PER_THREE 1.24f // our walk speed (10.5 studs/s) over the three.js one (8.5): speeds scale by this

typedef struct GoldenDef {
  CString id;
  f32 x, z;
} GoldenDef;

hz_internal GoldenDef golden_def(u32 i) {
  switch (i) {
  case 0: return (GoldenDef){"g-nw", -22.0f, -24.0f};
  case 1: return (GoldenDef){"g-pond", -14.0f, 22.5f};
  case 2: return (GoldenDef){"g-garden", -23.0f, 12.0f};
  case 3: return (GoldenDef){"g-park", 15.0f, -12.0f};
  case 4: return (GoldenDef){"g-route1", 0.0f, -40.0f};
  case 5: return (GoldenDef){"g-route2", -7.5f, -30.0f};
  case 6: return (GoldenDef){"g-route3", 6.5f, -36.0f};
  case 7: return (GoldenDef){"g-gap", 24.5f, -14.0f};
  case 8: return (GoldenDef){"g-behind", 64.5f, -20.0f};
  case 9: return (GoldenDef){"g-yard", 60.0f, 22.8f};
  case 10: return (GoldenDef){"g-lamp", -25.0f, 0.5f};
  case 11: return (GoldenDef){"g-home", 191.0f, 6.0f};
  case 12: return (GoldenDef){"g-dex", 256.0f, -5.0f};
  case 13: return (GoldenDef){"g-lab", 342.0f, 8.5f};
  default: return (GoldenDef){"g-bedroom", 194.5f, 44.6f};
  }
}

hz_internal v2 lamp_pos(u32 i) {
  switch (i) {
  case 0: return (v2){-4, -4};
  case 1: return (v2){4, 4};
  case 2: return (v2){-4, 10};
  case 3: return (v2){4, -14};
  case 4: return (v2){-22, 4};
  default: return (v2){22, -3.5f};
  }
}

typedef enum PipMode {
  PIP_WALK,      // his loop around the pond
  PIP_FOOTBALL,  // chasing the ball
  PIP_HIDING,    // hide and seek: hidden somewhere
  PIP_WAVE,      // found, or the game is over: waves until you walk off
  PIP_FLEE,      // tag: runs from you
} PipMode;

typedef struct Crumb {
  b32 active, landed;
  f32 t;
  v3 from, to;
  EcsEntity e;
} Crumb;

typedef struct Life {
  EcsWorld *w;
  RendererState *r;
  SoundBank *snd;
  SaveState *save;
  AudioSystem *audio;
  Allocator alloc;
  u32 seed;
  f32 time;
  TownRect *blocks; // the town's colliders (studs)
  u32 block_count;
  HzHandleT(RendererMaterial) mat, gold, glow, bulb;
  // golden studs
  EcsEntity golden[GOLDEN_COUNT];
  // ducks
  EcsEntity duck[DUCKS];
  v3 duck_p[DUCKS];
  f32 duck_yaw[DUCKS];
  v2 duck_goal;
  f32 duck_timer, quack_t;
  Crumb crumb[CRUMBS];
  f32 feed_cd;
  // fish
  EcsEntity fish;
  f32 fish_next, fish_jump; // seconds until the next jump; the jump's progress (-1: swimming)
  // butterflies and fireflies
  EcsEntity fly[FLIES];
  f32 fly_gone[FLIES], fly_catch[FLIES];
  EcsEntity firefly[FIREFLIES];
  v3 ff_home[FIREFLIES];
  f32 ff_gone[FIREFLIES], ff_catch[FIREFLIES];
  // the ball
  EcsEntity ball;
  v3 ball_p, ball_v;
  f32 kick_cd, goal_t;
  b32 goal_armed;
  // the chest
  EcsEntity chest, chest_lid;
  v3 chest_p;
  b32 chest_live, chest_open;
  f32 chest_t, chest_next;
  u32 chest_day;
  // weather
  EcsEntity drop[RAIN_DROPS];
  v3 drop_p[RAIN_DROPS];
  f32 rain, rain_target, rain_timer;
  EcsEntity puddle[PUDDLES];
  v3 puddle_p[PUDDLES]; // x, radius, z
  u32 puddle_count;
  f32 wet, splash_cd;
  HzHandleT(AudioVoice) rain_voice;
  // lamps, day and night
  EcsEntity bulb_e[LAMPS];
  EcsQuery *sun_query;
  b32 clock_set;
  f32 night, save_clock_t;
  u32 day;
  // the player's doings
  b32 sitting, dancing;
  v3 sit_from;
  // Pip
  Minifig *pip;
  v3 pip_p;
  f32 pip_yaw;
  PipMode pip_mode;
  i32 pip_wp;
  f32 pip_pause, pip_t, pip_cd, hint_t;
  f32 seek_count; // > 0 while the countdown runs (eyes closed)
  f32 seek_left;  // seconds of hide and seek left
  // messages
  char toast[160];
  f32 toast_t;
  char hud[3][64]; // the HUD's texts, alive until the UI is drawn
  b32 fade; // the screen goes dark (hide and seek countdown)
} Life;

hz_internal f32 life_rand(Life *l) {
  l->seed = l->seed * 1664525u + 1013904223u;
  return (f32)(l->seed >> 8) / 16777216.0f;
}

hz_internal f32 life_range(Life *l, f32 a, f32 b) { return a + (b - a) * life_rand(l); }

hz_internal void life_toast(Life *l, CString text) {
  cstr_format(l->toast, sizeof(l->toast), "%", fmt_cstr(text));
  l->toast_t = 2.6f;
}

#define LIFE_TOAST(l, ...)                                                                                                 \
  do {                                                                                                                     \
    cstr_format((l)->toast, sizeof((l)->toast), __VA_ARGS__);                                                              \
    (l)->toast_t = 2.6f;                                                                                                   \
  } while (0)

hz_internal void life_set(Life *l, EcsEntity e, v3 studs, f32 yaw, v3 scale) {
  ecs_renderer_set_transform(l->w, e, (Transform){.position = town_m(studs.x, studs.y, studs.z),
                                                  .rotation = quat_from_euler((v3){0.0f, yaw, 0.0f}),
                                                  .scale = scale});
}

hz_internal void life_set_rot(Life *l, EcsEntity e, v3 studs, quat rot, v3 scale) {
  ecs_renderer_set_transform(l->w, e, (Transform){.position = town_m(studs.x, studs.y, studs.z), .rotation = rot, .scale = scale});
}

// ---------------------------------------------------------------- meshes

typedef void (*LifeShape)(Town *t);

hz_internal HzHandleT(RendererSubMesh) life_mesh(Life *l, LifeShape fn) {
  ArenaAllocator *arena = arena_create(MB(8), MB(1));
  MeshBuilder mb;
  mesh_builder_begin(&mb, arena, &(MeshBuilderDesc){.slot_count = 1});
  Town t = {.b = &mb, .rng = {.s = 5u}};
  fn(&t);
  MeshBuilderResult res = mesh_builder_finish(&mb, &(MeshBuilderFinishDesc){.renderer = l->r, .world = NULL, .alloc = &l->alloc});
  HzHandleT(RendererSubMesh) m = res.slots[0].parts[0];
  arena_destroy(arena);
  return m;
}

hz_internal void shape_golden(Town *t) {
  MeshBuilderStyle st = town_style(0xffcc33u);
  st.flat = false;
  quat up = quat_from_euler((v3){M_PI * 0.5f, 0.0f, 0.0f}); // standing on its edge, facing z
  mesh_builder_cylinder(t->b, &st, m4_trs(V3_ZERO, up, V3_ONE),
                        &(PgCylinderDesc){.radius_top = 0.55f * TOWN_STUD, .radius_bottom = 0.55f * TOWN_STUD,
                                          .height = 0.32f * TOWN_STUD, .radial_segments = 20});
  mesh_builder_cylinder(t->b, &st, m4_trs(town_m(0.0f, 0.0f, 0.2f), up, V3_ONE),
                        &(PgCylinderDesc){.radius_top = 0.42f * TOWN_STUD, .radius_bottom = 0.42f * TOWN_STUD,
                                          .height = 0.12f * TOWN_STUD, .radial_segments = 20});
}
hz_internal void shape_mother(Town *t) { town_duck(t, 0.0f, 0.0f, 0.95f * 1.3f, 0.0f, 0.0f, C_WHITE); }
hz_internal void shape_duckling(Town *t) { town_duck(t, 0.0f, 0.0f, 0.52f * 1.3f, 0.0f, 0.0f, C_YELLOW); }
hz_internal void shape_crumb(Town *t) { town_box(t, 0xd9a35cu, 0.3f, 0.2f, 0.3f, 0.0f, 0.0f, 0.0f, false); }
hz_internal void shape_fish(Town *t) {
  town_box(t, C_ORANGE, 0.8f, 0.3f, 0.35f, 0.0f, -0.15f, 0.0f, false);
  town_box(t, 0xf06d00u, 0.25f, 0.35f, 0.08f, -0.5f, -0.17f, 0.0f, false);
}
hz_internal void shape_wing(Town *t) {
  town_box(t, C_WHITE, 0.55f, 0.04f, 0.4f, 0.3f, 0.0f, 0.0f, false);
  town_box(t, C_WHITE, 0.55f, 0.04f, 0.4f, -0.3f, 0.0f, 0.0f, false);
  town_box(t, C_BLACK, 0.1f, 0.08f, 0.5f, 0.0f, 0.0f, 0.0f, false);
}
hz_internal void shape_firefly(Town *t) {
  MeshBuilderStyle st = town_style(0xfff07au);
  mesh_builder_polyhedron(t->b, &st, m4_trs(V3_ZERO, quat_identity(), town_m(0.12f, 0.12f, 0.12f)), PG_POLYHEDRON_ICOSA, 0);
}
hz_internal void shape_ball(Town *t) {
  MeshBuilderStyle st = town_style(C_WHITE);
  st.flat = false;
  mesh_builder_sphere(t->b, &st, m4_trs(V3_ZERO, quat_identity(), town_m(0.6f, 0.6f, 0.6f)), 14, 10);
  MeshBuilderStyle band = town_style(C_RED);
  band.flat = false;
  mesh_builder_torus(t->b, &band, m4_trs(V3_ZERO, quat_identity(), V3_ONE),
                     &(PgTorusDesc){.radius = 0.6f * TOWN_STUD, .tube = 0.06f * TOWN_STUD, .radial_segments = 6, .tubular_segments = 18});
}
hz_internal void shape_chest(Town *t) {
  town_box(t, C_RED_BROWN, 1.6f, 0.9f, 1.1f, 0.0f, 0.0f, 0.0f, false);
  town_box(t, C_YELLOW, 1.65f, 0.12f, 1.15f, 0.0f, 0.75f, 0.0f, false);
  town_box(t, C_YELLOW, 0.25f, 0.3f, 0.06f, 0.0f, 0.45f, 0.57f, false);
}
hz_internal void shape_lid(Town *t) { // hinged at its back edge (z -0.55)
  town_box(t, C_BROWN, 1.6f, 0.45f, 1.1f, 0.0f, 0.0f, 0.55f, true);
}
hz_internal void shape_drop(Town *t) { town_box(t, 0xb8d4ffu, 0.06f, 1.1f, 0.06f, 0.0f, 0.0f, 0.0f, false); }
hz_internal void shape_puddle(Town *t) { town_cyl(t, 0x5c7f9cu, 0.0f, 0.0f, 0.0f, 1.0f, 1.0f, 0.03f, 16, false); }
hz_internal void shape_bulb(Town *t) { town_box(t, 0xfff5c0u, 0.7f, 0.8f, 0.7f, 0.0f, 0.0f, 0.0f, false); }

hz_internal EcsEntity life_spawn(Life *l, HzHandleT(RendererSubMesh) m, HzHandleT(RendererMaterial) mat, v3 studs, b32 shadow) {
  return ecs_renderer_spawn(l->w, l->r,
                            &(EcsRendererSpawnDesc){.submesh = m,
                                                    .material = mat,
                                                    .transform = {.position = town_m(studs.x, studs.y, studs.z),
                                                                  .rotation = quat_identity(),
                                                                  .scale = V3_ONE},
                                                    .casts_shadow = shadow});
}

hz_internal HzHandleT(RendererMaterial) life_glow_material(Life *l, v4 color, f32 intensity) {
  HzHandleT(RendererMaterial) m = renderer_create_material_instance(l->r, renderer_default_material(l->r));
  material_set_vec4(l->r, m, "emission_color", color);
  material_set_float(l->r, m, "emission_intensity", intensity);
  return m;
}

// is (x, z) in a town collider (grown by r)
hz_internal b32 life_blocked(const Life *l, f32 x, f32 z, f32 r) {
  for (u32 i = 0; i < l->block_count; i++) {
    TownRect b = l->blocks[i];
    if (x > b.x0 - r && x < b.x1 + r && z > b.z0 - r && z < b.z1 + r)
      return true;
  }
  return false;
}

hz_internal b32 life_on_road(f32 x, f32 z) {
  for (i32 i = 0; i < TOWN_ROAD_COUNT; i++) {
    TownRect rd = town_road(i);
    if (x >= rd.x0 && x <= rd.x1 && z >= rd.z0 && z <= rd.z1)
      return true;
  }
  return false;
}

hz_internal b32 life_in_pond(f32 x, f32 z, f32 m) { return x > -12.0f + m && x < -4.0f - m && z > 15.0f + m && z < 22.0f - m; }

// ---------------------------------------------------------------- init

hz_internal void life_init(Life *l, EcsWorld *w, RendererState *r, SoundBank *snd, SaveState *save, AudioSystem *audio,
                           Allocator *alloc, TownRect *blocks, u32 block_count, Minifig *pip) {
  *l = (Life){.w = w, .r = r, .snd = snd, .save = save, .audio = audio, .alloc = *alloc, .seed = 4242u,
              .blocks = blocks, .block_count = block_count, .pip = pip};
  l->mat = renderer_default_material(r);
  l->gold = life_glow_material(l, (v4){1.0f, 0.62f, 0.1f, 1.0f}, 1.2f);
  l->glow = life_glow_material(l, (v4){1.0f, 0.95f, 0.45f, 1.0f}, 6.0f);
  l->bulb = life_glow_material(l, (v4){1.0f, 0.86f, 0.5f, 1.0f}, 0.4f);
  HzHandleT(RendererSubMesh) m_gold = life_mesh(l, shape_golden);
  for (u32 i = 0; i < GOLDEN_COUNT; i++) {
    GoldenDef g = golden_def(i);
    l->golden[i] = life_spawn(l, m_gold, l->gold, (v3){g.x, 1.4f + (g.x > 150.0f ? 0.4f : 0.0f), g.z}, true);
  }
  HzHandleT(RendererSubMesh) m_mom = life_mesh(l, shape_mother), m_kid = life_mesh(l, shape_duckling);
  for (u32 i = 0; i < DUCKS; i++) {
    l->duck_p[i] = (v3){-8.0f, 0.3f, 18.5f + (f32)i * 0.9f};
    l->duck[i] = life_spawn(l, i == 0 ? m_mom : m_kid, l->mat, l->duck_p[i], true);
  }
  l->duck_goal = (v2){-8.0f, 18.5f};
  l->duck_timer = 3.0f;
  HzHandleT(RendererSubMesh) m_crumb = life_mesh(l, shape_crumb);
  for (u32 i = 0; i < CRUMBS; i++)
    l->crumb[i].e = life_spawn(l, m_crumb, l->mat, (v3){0.0f, -50.0f, 0.0f}, false);
  l->fish = life_spawn(l, life_mesh(l, shape_fish), l->mat, (v3){-7.0f, 0.1f, 16.5f}, false);
  l->fish_next = 8.0f;
  l->fish_jump = -1.0f;
  HzHandleT(RendererSubMesh) m_wing = life_mesh(l, shape_wing);
  u32 wing_cols[5] = {C_PINK, C_YELLOW, 0x5aa9ffu, C_WHITE, C_ORANGE};
  for (u32 i = 0; i < FLIES; i++) {
    HzHandleT(RendererMaterial) m = renderer_create_material_instance(r, l->mat);
    v4 c = mesh_builder_hex(wing_cols[i % 5]);
    material_set_vec4(r, m, "color", c);
    l->fly[i] = life_spawn(l, m_wing, m, V3_ZERO, false);
  }
  HzHandleT(RendererSubMesh) m_ff = life_mesh(l, shape_firefly);
  for (u32 i = 0; i < FIREFLIES; i++) {
    v3 h = i < 14 ? (v3){life_range(l, -8, 8), 0, life_range(l, -40, -29)}
                  : i < 24 ? (v3){life_range(l, -13, -3), 0, life_range(l, 14, 23)} : (v3){life_range(l, 10, 20), 0, life_range(l, -15, -8)};
    l->ff_home[i] = h;
    l->firefly[i] = life_spawn(l, m_ff, l->glow, (v3){h.x, -50.0f, h.z}, false);
  }
  l->ball_p = (v3){15.0f, 0.6f, -6.2f};
  l->ball = life_spawn(l, life_mesh(l, shape_ball), l->mat, l->ball_p, true);
  l->goal_armed = true;
  l->chest = life_spawn(l, life_mesh(l, shape_chest), l->mat, (v3){0.0f, -50.0f, 0.0f}, true);
  l->chest_lid = life_spawn(l, life_mesh(l, shape_lid), l->mat, (v3){0.0f, -50.0f, 0.0f}, true);
  l->chest_next = 50.0f;
  l->chest_day = 0xffffffffu;
  HzHandleT(RendererSubMesh) m_drop = life_mesh(l, shape_drop);
  for (u32 i = 0; i < RAIN_DROPS; i++)
    l->drop[i] = life_spawn(l, m_drop, l->mat, (v3){0.0f, -50.0f, 0.0f}, false);
  l->rain_timer = life_range(l, 150.0f, 240.0f);
  // puddles on the roads, at least 5 apart, the whole puddle on the road
  HzHandleT(RendererSubMesh) m_pud = life_mesh(l, shape_puddle);
  for (u32 tries = 0; tries < 400 && l->puddle_count < PUDDLES; tries++) {
    TownRect rd = town_road((i32)(life_rand(l) * TOWN_ROAD_COUNT) % TOWN_ROAD_COUNT);
    f32 rr = life_range(l, 0.9f, 1.8f);
    f32 x = life_range(l, rd.x0 + rr, rd.x1 - rr), z = life_range(l, rd.z0 + rr, rd.z1 - rr);
    if (rd.x1 - rd.x0 < 2.0f * rr || rd.z1 - rd.z0 < 2.0f * rr)
      continue;
    b32 ok = true;
    for (u32 k = 0; k < l->puddle_count && ok; k++)
      ok = v2_length((v2){l->puddle_p[k].x - x, l->puddle_p[k].z - z}) > 5.0f;
    if (!ok)
      continue;
    l->puddle_p[l->puddle_count] = (v3){x, rr, z};
    l->puddle[l->puddle_count] = life_spawn(l, m_pud, l->mat, (v3){x, -50.0f, z}, false);
    l->puddle_count++;
  }
  HzHandleT(RendererSubMesh) m_bulb = life_mesh(l, shape_bulb);
  for (u32 i = 0; i < LAMPS; i++) {
    v2 p = lamp_pos(i);
    l->bulb_e[i] = life_spawn(l, m_bulb, l->bulb, (v3){p.x, 5.2f, p.y}, false);
  }
  l->sun_query = ecs_query_cached(w, EcsInOut(DayNightCycle));
  l->pip_p = (v3){-8.0f, 0.0f, 13.2f};
  l->rain_voice = audio_play(audio, snd->clip[SND_RAIN], &(AudioPlayDesc){.volume = 0.0001f, .loop = true, .bus = AUDIO_BUS_SFX});
}

// ---------------------------------------------------------------- the clock

// three.js tod (0..1, noon at 0.25) to hours and back
hz_internal f32 tod_to_hours(f32 tod) { return m_fmodf(tod * 24.0f + 6.0f, 24.0f); }
hz_internal f32 hours_to_tod(f32 h) { return m_fmodf((h - 6.0f) / 24.0f + 1.0f, 1.0f); }

hz_internal DayNightCycle *life_sun(Life *l, EcsEntity *out) {
  EcsEntity e = ecs_query_get_singleton_entity(l->sun_query);
  if (e == ECS_ENTITY_INVALID)
    return NULL;
  *out = e;
  return (DayNightCycle *)ecs_component_get(l->w, e, DayNightCycle);
}

hz_internal void life_set_clock(Life *l, f32 hours, f32 day_length) {
  EcsEntity e;
  DayNightCycle *c = life_sun(l, &e);
  if (!c)
    return;
  DayNightCycle n = *c;
  if (hours >= 0.0f)
    n.time_of_day = hours;
  n.day_length = day_length;
  ecs_component_set(l->w, e, DayNightCycle, n);
}

hz_internal f32 life_hours(Life *l) {
  const DayNightState *st = ecs_resource_get(l->w, DayNightState);
  return st ? st->time_of_day : 10.0f;
}

// your bed: past the evening it is morning, otherwise a nap until the evening
hz_internal void life_sleep(Life *l) {
  f32 tod = hours_to_tod(life_hours(l));
  b32 night = tod > 0.55f || tod < 0.05f;
  life_set_clock(l, tod_to_hours(night ? 0.1f : 0.56f), DAY_SECONDS);
  sound_play(l->snd, SND_BLIP, 1.0f, 0.8f);
  life_toast(l, night ? "Zzz\xe2\x80\xa6 Good morning, Brickhollow!" : "Zzz\xe2\x80\xa6 A cosy nap until the evening.");
  l->save->data.time_of_day = night ? 0.1f : 0.56f;
  save_mark(l->save);
}

// ---------------------------------------------------------------- the parts

hz_internal void life_golden(Life *l, v3 p, b32 inside) {
  GameSave *s = &l->save->data;
  for (u32 i = 0; i < GOLDEN_COUNT; i++) {
    GoldenDef g = golden_def(i);
    b32 taken = (s->golden_mask >> i) & 1u;
    f32 base = 1.4f + (g.x > 150.0f ? 0.4f : 0.0f);
    if (taken) {
      life_set(l, l->golden[i], (v3){g.x, -50.0f, g.z}, 0.0f, V3_ONE);
      continue;
    }
    life_set(l, l->golden[i], (v3){g.x, base + m_sinf(l->time * 3.0f + g.x) * 0.2f, g.z}, l->time * 2.2f, V3_ONE);
    f32 dx = g.x - p.x, dz = g.z - p.z;
    if (dx * dx + dz * dz < 1.4f * 1.4f * 1.3f) {
      s->golden_mask |= 1u << i;
      s->studs += 5;
      u32 n = 0;
      for (u32 k = 0; k < GOLDEN_COUNT; k++)
        n += (s->golden_mask >> k) & 1u;
      sound_play(l->snd, SND_COIN, 1.0f, 0.9f);
      LIFE_TOAST(l, "Golden stud! %/15 found \xc2\xb7 +5 studs", fmt_u32(n));
      if (n == GOLDEN_COUNT && !s->legendary) {
        s->legendary = true;
        sound_play(l->snd, SND_FANFARE, 1.0f, 0.9f);
        life_toast(l, "All golden studs found! A legendary card appears\xe2\x80\xa6");
      }
      save_mark(l->save);
    }
  }
  (void)inside;
}

hz_internal f32 life_yaw_to(v2 d) { return m_atan2f(-d.y, d.x); } // meshes facing +x: the yaw that turns +x to d

hz_internal void life_ducks(Life *l, v3 p, f32 dt, b32 outside) {
  // crumbs fly and land
  i32 crumbs_left = 0;
  for (u32 i = 0; i < CRUMBS; i++) {
    Crumb *c = &l->crumb[i];
    if (!c->active)
      continue;
    crumbs_left++;
    c->t += dt;
    if (!c->landed) {
      f32 k = m_clampf(c->t / 0.6f, 0.0f, 1.0f);
      v3 q = v3_lerp(c->from, c->to, k);
      q.y += m_sinf(k * M_PI) * 1.6f;
      life_set(l, c->e, q, c->t * 5.0f, V3_ONE);
      if (k >= 1.0f) {
        c->landed = true;
        sound_play(l->snd, SND_SPLASH, 1.6f, 0.25f);
      }
    } else
      life_set(l, c->e, (v3){c->to.x, 0.3f + m_sinf(l->time * 3.0f + (f32)i) * 0.03f, c->to.z}, 0.0f, V3_ONE);
  }
  // the mother: a crumb, the player, or a wandering goal
  l->duck_timer -= dt;
  b32 close_by = outside && p.x > -16.5f && p.x < 0.5f && p.z > 10.5f && p.z < 26.5f;
  f32 speed = 1.1f;
  if (l->duck_timer <= 0.0f) {
    l->duck_goal = (v2){life_range(l, -11.0f, -5.0f), life_range(l, 16.0f, 21.0f)};
    l->duck_timer = life_range(l, 5.0f, 11.0f);
  }
  if (close_by) {
    l->duck_goal = (v2){m_clampf(p.x, -11.0f, -5.0f), m_clampf(p.z, 16.0f, 21.0f)};
    speed = 2.2f;
  }
  for (u32 i = 0; i < DUCKS; i++) {
    v2 goal = l->duck_goal;
    f32 sp = speed;
    if (i > 0) { // a line behind the one in front
      v3 lead = l->duck_p[i - 1];
      v2 back = {m_cosf(l->duck_yaw[i - 1]), -m_sinf(l->duck_yaw[i - 1])};
      f32 gap = i == 1 ? 1.15f : 0.75f;
      goal = (v2){lead.x - back.x * gap, lead.z - back.y * gap};
      sp = 2.6f;
    }
    // a crumb in the water wins
    f32 best = 1e9f;
    i32 bi = -1;
    for (u32 c = 0; c < CRUMBS; c++)
      if (l->crumb[c].active && l->crumb[c].landed) {
        f32 d = v2_length((v2){l->crumb[c].to.x - l->duck_p[i].x, l->crumb[c].to.z - l->duck_p[i].z});
        if (d < best) {
          best = d;
          bi = (i32)c;
        }
      }
    if (bi >= 0) {
      goal = (v2){l->crumb[bi].to.x, l->crumb[bi].to.z};
      sp = i == 0 ? 2.4f : 2.9f;
      f32 scale = i == 0 ? 0.95f : 0.52f;
      if (best < 0.45f * (scale + 0.4f) * 1.6f) {
        l->crumb[bi].active = false;
        life_set(l, l->crumb[bi].e, (v3){0.0f, -50.0f, 0.0f}, 0.0f, V3_ONE);
        sound_play(l->snd, SND_QUACK, i == 0 ? 1.0f : 1.7f, 0.5f);
      }
    }
    v2 d = {goal.x - l->duck_p[i].x, goal.y - l->duck_p[i].z};
    f32 len = v2_length(d);
    if (len > 0.15f) {
      f32 step = m_minf(len, sp * m_minf(1.0f, len) * dt * 1.4f);
      l->duck_p[i].x += d.x / len * step;
      l->duck_p[i].z += d.y / len * step;
      f32 want = life_yaw_to(d);
      l->duck_yaw[i] += fig_wrap(want - l->duck_yaw[i]) * m_minf(1.0f, dt * 3.0f);
    } else if (i == 0 && close_by) {
      l->duck_yaw[i] += fig_wrap(life_yaw_to((v2){p.x - l->duck_p[i].x, p.z - l->duck_p[i].z}) - l->duck_yaw[i]) * m_minf(1.0f, dt * 3.0f);
    }
    l->duck_p[i].x = m_clampf(l->duck_p[i].x, -11.4f, -4.6f);
    l->duck_p[i].z = m_clampf(l->duck_p[i].z, 15.6f, 21.4f);
    f32 y = 0.16f + m_sinf(l->time * 2.4f + (f32)i * 1.3f) * 0.03f;
    quat q = quat_mul(quat_from_euler((v3){0.0f, l->duck_yaw[i], 0.0f}),
                      quat_from_euler((v3){m_sinf(l->time * 1.8f + (f32)i) * 0.05f, 0.0f, 0.0f}));
    life_set_rot(l, l->duck[i], (v3){l->duck_p[i].x, y, l->duck_p[i].z}, q, V3_ONE);
  }
  // quacks while you are close_by
  l->quack_t -= dt;
  if (close_by && l->quack_t <= 0.0f) {
    sound_play(l->snd, SND_QUACK, 1.0f, 0.45f);
    if (life_rand(l) < 0.6f)
      sound_play(l->snd, SND_QUACK, life_range(l, 1.6f, 1.9f), 0.3f);
    l->quack_t = life_range(l, 2.5f, 5.5f);
  }
  l->feed_cd = m_maxf(0.0f, l->feed_cd - dt);
  (void)crumbs_left;
}

// the pond's feeding spot: five crumbs thrown into the water close_by you
hz_internal void life_feed(Life *l, v3 p) {
  for (u32 i = 0; i < CRUMBS; i++)
    if (l->crumb[i].active)
      return;
  if (l->feed_cd > 0.0f)
    return;
  l->feed_cd = 3.0f;
  sound_play(l->snd, SND_QUACK, 1.0f, 0.5f);
  for (u32 i = 0; i < CRUMBS; i++) {
    Crumb *c = &l->crumb[i];
    c->active = true;
    c->landed = false;
    c->t = -(f32)i * 0.08f;
    c->from = (v3){p.x, 2.6f, p.z};
    c->to = (v3){m_clampf(p.x - 2.4f + life_range(l, -1.75f, 1.75f), -11.2f, -4.8f), 0.3f,
                 m_clampf(p.z + life_range(l, -1.75f, 1.75f), 15.8f, 21.2f)};
  }
  GameSave *s = &l->save->data;
  s->duck_feeds++;
  if (s->duck_feeds == 1)
    life_toast(l, "Quack! The ducks love bread.");
  else if (s->duck_feeds % 10 == 0)
    LIFE_TOAST(l, "You've fed the ducks % times!", fmt_u32(s->duck_feeds));
  save_mark(l->save);
}

hz_internal void life_fish(Life *l, v3 p, f32 dt, b32 outside) {
  f32 a = l->time * 0.6f;
  v3 at = {-8.0f + m_cosf(a) * 3.0f, 0.12f, 18.5f + m_sinf(a) * 2.0f};
  f32 yaw = life_yaw_to((v2){-m_sinf(a) * 3.0f, m_cosf(a) * 2.0f});
  f32 pitch = 0.0f;
  if (l->fish_jump >= 0.0f) {
    l->fish_jump += dt / 0.9f;
    f32 k = m_minf(1.0f, l->fish_jump);
    at.y = 0.08f + m_sinf(k * M_PI) * 1.9f;
    pitch = (0.5f - k) * 2.4f;
    if (l->fish_jump >= 1.0f) {
      l->fish_jump = -1.0f;
      sound_play(l->snd, SND_SPLASH, 1.0f, 0.35f);
    }
  } else {
    l->fish_next -= dt;
    f32 d = v2_length((v2){p.x + 8.0f, p.z - 18.5f});
    if (l->fish_next <= 0.0f && outside && d < 14.0f) {
      l->fish_jump = 0.0f;
      l->fish_next = life_range(l, 6.0f, 14.0f);
      ecs_audio_play(l->w, "brickhollow/music/fish-splash.hzt", &(EcsAudioPlayDesc){.volume = 0.6f, .bus = AUDIO_BUS_SFX});
    }
  }
  quat q = quat_mul(quat_from_euler((v3){0.0f, yaw, 0.0f}), quat_from_euler((v3){0.0f, 0.0f, pitch}));
  life_set_rot(l, l->fish, at, q, V3_ONE);
}

hz_internal void life_butterflies(Life *l, v3 p, f32 dt, b32 outside) {
  for (u32 i = 0; i < FLIES; i++) {
    v3 c;
    switch (i) {
    case 0: c = (v3){-19, 4, 12}; break;
    case 1: c = (v3){-17, 3, 11}; break;
    case 2: c = (v3){15, 4, -11}; break;
    case 3: c = (v3){0, 5, -36}; break;
    case 4: c = (v3){-4, 4, -32}; break;
    case 5: c = (v3){5, 4, -38}; break;
    case 6: c = (v3){45, 5, 16}; break;
    default: c = (v3){-8, 3, 19}; break;
    }
    f32 a = l->time * (0.4f + 0.05f * (f32)i) + (f32)i * 1.7f;
    v3 q = {c.x + m_sinf(a) * c.y, 2.2f + m_sinf(2.3f * a) * 0.8f, c.z + m_sinf(1.7f * a) * c.y * 0.6f};
    if (l->fly_catch[i] > 0.0f) { // spiralling away
      l->fly_catch[i] += dt / 1.6f;
      f32 k = l->fly_catch[i];
      q.x += m_cosf(k * 9.0f) * k * 2.0f;
      q.z += m_sinf(k * 9.0f) * k * 2.0f;
      q.y += k * k * 3.5f;
      if (k >= 1.0f) {
        l->fly_catch[i] = 0.0f;
        l->fly_gone[i] = life_range(l, 25.0f, 35.0f);
      }
    } else if (l->fly_gone[i] > 0.0f) {
      l->fly_gone[i] -= dt;
      q.y = -50.0f;
    } else if (outside && v2_length((v2){q.x - p.x, q.z - p.z}) < 1.3f * 1.3f) {
      l->fly_catch[i] = 0.001f;
      GameSave *s = &l->save->data;
      s->studs += 1;
      s->_reserved[0]++; // butterflies
      sound_play(l->snd, SND_POP, 1.6f, 0.4f);
      LIFE_TOAST(l, "A butterfly! (% caught) \xc2\xb7 +1 stud", fmt_u32(s->_reserved[0]));
      save_mark(l->save);
    }
    f32 flap = 0.25f + m_absf(m_sinf(l->time * 14.0f + (f32)i)) * 0.75f;
    life_set(l, l->fly[i], q, a * 2.0f, (v3){flap, 1.0f, 1.0f});
  }
}

hz_internal void life_fireflies(Life *l, v3 p, f32 dt, b32 outside, b32 walking) {
  b32 show = outside && l->night > 0.25f;
  for (u32 i = 0; i < FIREFLIES; i++) {
    v3 h = l->ff_home[i];
    f32 a = l->time * (0.5f + 0.03f * (f32)i) + (f32)i;
    v3 q = {h.x + m_cosf(a) * 1.4f, 1.2f + m_sinf(a * 1.3f) * 0.6f, h.z + m_sinf(a) * 1.4f};
    f32 s = 0.6f + 0.4f * m_absf(m_sinf(l->time * 2.0f + (f32)i * 1.7f));
    if (l->ff_catch[i] > 0.0f) {
      l->ff_catch[i] += dt / 0.6f;
      f32 k = m_minf(1.0f, l->ff_catch[i]);
      q = v3_lerp(q, (v3){p.x, 3.0f, p.z}, k);
      s *= 1.0f + k;
      if (l->ff_catch[i] >= 1.0f) {
        l->ff_catch[i] = 0.0f;
        l->ff_gone[i] = 30.0f;
      }
    } else if (l->ff_gone[i] > 0.0f) {
      l->ff_gone[i] -= dt;
      s = 0.0f;
    } else if (show && l->night > 0.5f && walking && v2_length((v2){q.x - p.x, q.z - p.z}) < 1.2f * 1.4f) {
      l->ff_catch[i] = 0.001f;
      GameSave *sv = &l->save->data;
      sv->fireflies++;
      sv->studs += 1;
      sound_play(l->snd, SND_COIN, 1.5f, 0.35f);
      LIFE_TOAST(l, "A firefly! (% caught) \xc2\xb7 +1 stud", fmt_u32(sv->fireflies));
      save_mark(l->save);
    }
    if (!show)
      q.y = -50.0f;
    life_set(l, l->firefly[i], q, 0.0f, (v3){s, s, s});
  }
}

hz_internal void life_ball(Life *l, v3 p, v2 move_dir, f32 player_speed, f32 dt, b32 outside) {
  l->kick_cd -= dt;
  v2 d = {l->ball_p.x - p.x, l->ball_p.z - p.z};
  f32 dist = v2_length(d);
  f32 ps = player_speed / TOWN_STUD; // studs/s
  if (outside && dist < 0.6f + 1.0f && ps > 1.0f && l->kick_cd <= 0.0f) {
    v2 away = dist > 0.01f ? v2_scale(d, 1.0f / dist) : move_dir;
    v2 dir = v2_add(v2_scale(away, 0.6f), v2_scale(move_dir, 0.4f));
    f32 dl = v2_length(dir);
    if (dl > 0.01f)
      dir = v2_scale(dir, 1.0f / dl);
    f32 power = m_minf(14.0f, 3.0f + ps * 0.9f / STUD_PER_THREE) * STUD_PER_THREE;
    l->ball_v = (v3){dir.x * power, 0.0f, dir.y * power};
    l->kick_cd = 0.25f;
    sound_play(l->snd, SND_KICK, 1.0f, m_minf(1.0f, power / 14.0f));
  }
  if (dist < 1.3f && dist > 0.01f) { // never inside the player
    l->ball_p.x = p.x + d.x / dist * 1.3f;
    l->ball_p.z = p.z + d.y / dist * 1.3f;
  }
  // roll, bounce off the colliders (70 % of the speed)
  f32 sp = v2_length((v2){l->ball_v.x, l->ball_v.z});
  if (sp > 0.02f) {
    f32 nx = l->ball_p.x + l->ball_v.x * dt, nz = l->ball_p.z + l->ball_v.z * dt;
    b32 bx = life_blocked(l, nx, l->ball_p.z, 0.5f) || life_in_pond(nx, l->ball_p.z, -0.6f);
    b32 bz = life_blocked(l, l->ball_p.x, nz, 0.5f) || life_in_pond(l->ball_p.x, nz, -0.6f);
    if (bx)
      l->ball_v.x *= -0.7f;
    else
      l->ball_p.x = nx;
    if (bz)
      l->ball_v.z *= -0.7f;
    else
      l->ball_p.z = nz;
    if ((bx || bz) && sp > 2.0f)
      sound_play(l->snd, SND_KICK, 1.5f, 0.3f);
    f32 k = m_expf(-1.4f * dt);
    l->ball_v.x *= k;
    l->ball_v.z *= k;
  } else
    l->ball_v = V3_ZERO;
  // the goal
  b32 in_goal = l->ball_p.x > 25.0f && l->ball_p.x < 27.0f && l->ball_p.z > -7.6f && l->ball_p.z < -4.4f;
  if (in_goal && l->goal_armed) {
    l->goal_armed = false;
    l->goal_t = 0.0f;
    GameSave *s = &l->save->data;
    s->goals++;
    s->studs += 1;
    sound_play(l->snd, SND_FANFARE, 1.0f, 0.7f);
    LIFE_TOAST(l, "GOAL! (% scored) \xc2\xb7 +1 stud", fmt_u32(s->goals));
    save_mark(l->save);
  }
  if (!l->goal_armed) {
    l->goal_t += dt;
    if (l->ball_p.x < 24.2f)
      l->goal_armed = true;
    else if (l->goal_t > 2.0f && sp < 1.5f)
      l->ball_v = (v3){-9.0f, 0.0f, 0.0f};
  }
  // it rolls: turn it about the axis across its motion
  f32 roll = l->time;
  life_set_rot(l, l->ball, l->ball_p, quat_from_euler((v3){l->ball_p.z * 1.6f, 0.0f, -l->ball_p.x * 1.6f}), V3_ONE);
  (void)roll;
}

hz_internal void life_chest(Life *l, v3 p, f32 dt, b32 outside) {
  if (l->chest_live) {
    l->chest_t += dt;
    f32 shrink = 1.0f;
    if (l->chest_open && l->chest_t > 3.0f) {
      shrink = 1.0f - m_minf(1.0f, (l->chest_t - 3.0f) / 0.6f);
      if (shrink <= 0.0f) {
        l->chest_live = false;
        l->chest_next = 30.0f;
      }
    }
    f32 jiggle = l->chest_open ? 0.0f : m_sinf(l->time * 9.0f) * 0.04f * (m_sinf(l->time * 1.3f) > 0.6f ? 1.0f : 0.0f);
    f32 s = 1.4f * shrink;
    life_set(l, l->chest, l->chest_p, jiggle, (v3){s, s, s});
    f32 lid = l->chest_open ? -m_minf(1.0f, l->chest_t / 0.4f) * 1.6f : 0.0f;
    v3 hinge = {l->chest_p.x, l->chest_p.y + 0.9f * s, l->chest_p.z - 0.55f * s};
    life_set_rot(l, l->chest_lid, hinge, quat_from_euler((v3){lid, jiggle, 0.0f}), (v3){s, s, s});
    return;
  }
  life_set(l, l->chest, (v3){0.0f, -50.0f, 0.0f}, 0.0f, V3_ONE);
  life_set(l, l->chest_lid, (v3){0.0f, -50.0f, 0.0f}, 0.0f, V3_ONE);
  l->chest_next -= dt;
  if (l->chest_next > 0.0f || !outside || l->chest_day == l->day)
    return;
  for (u32 tries = 0; tries < 80; tries++) {
    f32 x = life_range(l, -24.0f, 64.0f), z = life_range(l, -23.0f, 21.0f);
    f32 d = v2_length((v2){x - p.x, z - p.z});
    if (d < 12.0f || d > 50.0f || life_on_road(x, z) || life_blocked(l, x, z, 1.8f) || life_in_pond(x, z, -1.5f))
      continue;
    l->chest_p = (v3){x, 0.0f, z};
    l->chest_live = true;
    l->chest_open = false;
    l->chest_t = 0.0f;
    l->chest_day = l->day;
    sound_play(l->snd, SND_COIN, 0.7f, 0.5f);
    life_toast(l, "Something sparkles somewhere in the village\xe2\x80\xa6");
    return;
  }
  l->chest_next = 20.0f;
}

hz_internal void life_open_chest(Life *l) {
  if (!l->chest_live || l->chest_open)
    return;
  l->chest_open = true;
  l->chest_t = 0.0f;
  u32 n = 3u + (u32)(life_rand(l) * 6.0f);
  GameSave *s = &l->save->data;
  s->chests++;
  s->studs += n;
  sound_play(l->snd, SND_FANFARE, 1.2f, 0.7f);
  LIFE_TOAST(l, "Treasure! You found % studs.", fmt_u32(n));
  save_mark(l->save);
}

hz_internal void life_weather(Life *l, v3 p, f32 dt, b32 outside, b32 walking) {
  l->rain_timer -= dt;
  if (l->rain_timer <= 0.0f) {
    if (l->rain_target > 0.5f) {
      l->rain_target = 0.0f;
      l->rain_timer = life_range(l, 180.0f, 300.0f);
    } else {
      l->rain_target = 1.0f;
      l->rain_timer = life_range(l, 35.0f, 55.0f);
    }
  }
  l->rain += (l->rain_target - l->rain) * m_minf(1.0f, dt * 0.35f);
  f32 level = outside ? l->rain : 0.0f;
  audio_voice_set_volume(l->audio, l->rain_voice, level * 0.35f + 0.0001f);
  day_night_stylized_base(l->w)->haze = 0.6f + level * 0.9f;
  u32 n = (u32)(level * RAIN_DROPS);
  for (u32 i = 0; i < RAIN_DROPS; i++) {
    v3 *d = &l->drop_p[i];
    if (i >= n) {
      life_set(l, l->drop[i], (v3){0.0f, -50.0f, 0.0f}, 0.0f, V3_ONE);
      d->y = -1.0f;
      continue;
    }
    d->y -= (18.0f + (f32)(i % 7)) * dt;
    if (d->y < 0.0f || v2_length((v2){d->x - p.x, d->z - p.z + 4.0f}) > 26.0f)
      *d = (v3){p.x + life_range(l, -20.0f, 20.0f), life_range(l, 8.0f, 20.0f), p.z - 4.0f + life_range(l, -16.0f, 16.0f)};
    life_set(l, l->drop[i], *d, 0.0f, V3_ONE);
  }
  // puddles: wet while it rains hard, dry slowly
  if (l->rain > 0.3f)
    l->wet = m_minf(1.0f, l->wet + dt * 0.04f * l->rain);
  else
    l->wet = m_maxf(0.0f, l->wet - dt * 0.016f);
  l->splash_cd -= dt;
  for (u32 i = 0; i < l->puddle_count; i++) {
    v3 pd = l->puddle_p[i];
    f32 r = pd.y * (0.35f + 0.65f * l->wet);
    b32 show = l->wet > 0.02f;
    life_set(l, l->puddle[i], (v3){pd.x, show ? 0.42f : -50.0f, pd.z}, (f32)i, (v3){r, 1.0f, r * 0.7f});
    if (show && l->wet >= 0.25f && walking && l->splash_cd <= 0.0f && v2_length((v2){p.x - pd.x, p.z - pd.z}) < r * 0.9f) {
      l->splash_cd = 0.35f;
      sound_play(l->snd, SND_SPLASH, 1.1f, 0.45f);
    }
  }
}

hz_internal void life_lamps(Life *l) {
  material_set_float(l->r, l->bulb, "emission_intensity", 0.4f + l->night * 8.0f);
}

// ---------------------------------------------------------------- Pip

hz_internal v2 pip_waypoint(i32 i) {
  switch (i & 3) {
  case 0: return (v2){-8.0f, 13.2f};
  case 1: return (v2){-3.0f, 13.8f};
  case 2: return (v2){-2.8f, 23.0f};
  default: return (v2){-11.0f, 23.2f};
  }
}

hz_internal void pip_say(Life *l, CString text) { LIFE_TOAST(l, "Pip: \"%\"", fmt_cstr(text)); }

// Pip moves toward `to` at `speed` studs/s; returns the remaining distance
hz_internal f32 pip_move(Life *l, v2 to, f32 speed, f32 dt) {
  v2 d = {to.x - l->pip_p.x, to.y - l->pip_p.z};
  f32 len = v2_length(d);
  if (len > 0.05f) {
    f32 step = m_minf(len, speed * dt);
    l->pip_p.x += d.x / len * step;
    l->pip_p.z += d.y / len * step;
    l->pip_yaw = m_atan2f(d.x, d.y); // figures face +z at yaw 0
    l->pip->speed = speed * TOWN_STUD;
  } else
    l->pip->speed = 0.0f;
  return len;
}

hz_internal void life_hide_seek_start(Life *l) {
  l->seek_count = 3.0f;
  l->fade = true;
  sound_play(l->snd, SND_BLIP, 1.0f, 0.7f);
}

hz_internal b32 pip_find_hiding_spot(Life *l, v3 p) {
  for (u32 tries = 0; tries < 400; tries++) {
    f32 x = life_range(l, -24.0f, 64.0f), z = life_range(l, -23.5f, 22.0f);
    f32 d = v2_length((v2){x - p.x, z - p.z});
    if (d < 16.0f || d > 45.0f || life_on_road(x, z) || life_blocked(l, x, z, 0.7f) || life_in_pond(x, z, -1.0f))
      continue;
    if (!life_blocked(l, x, z, 1.6f)) // close behind something
      continue;
    l->pip_p = (v3){x, 0.0f, z};
    return true;
  }
  return false;
}

hz_internal void life_pip(Life *l, v3 p, f32 dt, b32 outside, b32 sprinting) {
  Minifig *f = l->pip;
  f->pose = FIG_POSE_NORMAL;
  l->pip_cd -= dt;
  f32 dp = v2_length((v2){p.x - l->pip_p.x, p.z - l->pip_p.z});
  // hide and seek countdown (eyes closed)
  if (l->seek_count > 0.0f) {
    f32 before = l->seek_count;
    l->seek_count -= dt / 0.9f;
    if (before > 2.0f && l->seek_count <= 2.0f) {
      if (!pip_find_hiding_spot(l, p)) {
        l->seek_count = 0.0f;
        l->fade = false;
        life_toast(l, "Pip found no hiding spot. Try again outside!");
        return;
      }
      l->pip_mode = PIP_HIDING;
    }
    if ((i32)before != (i32)l->seek_count && l->seek_count > 0.0f)
      sound_play(l->snd, SND_BLIP, 1.0f, 0.7f);
    if (l->seek_count <= 0.0f) {
      l->fade = false;
      l->seek_left = 60.0f;
      l->hint_t = 9.0f;
      sound_play(l->snd, SND_ALERT, 1.0f, 0.7f);
      life_toast(l, "Ready or not! Find Pip within 60 seconds.");
    }
  }
  switch (l->pip_mode) {
  case PIP_HIDING:
    f->pose = FIG_POSE_CROUCH;
    f->speed = 0.0f;
    if (l->seek_count > 0.0f)
      break;
    l->seek_left -= dt;
    l->hint_t -= dt;
    if (dp < 3.2f) {
      l->pip_mode = PIP_WAVE;
      l->save->data.seek_wins++;
      l->save->data.studs += 5;
      save_mark(l->save);
      sound_play(l->snd, SND_FANFARE, 1.0f, 0.7f);
      life_toast(l, "You found Pip! +5 studs");
    } else if (l->seek_left <= -3.0f) {
      l->pip_mode = PIP_WAVE;
      life_toast(l, "Time is up! Pip was hiding really well.");
    } else if (l->hint_t <= 0.0f) {
      l->hint_t = 8.0f;
      u32 k = (u32)(life_rand(l) * 3.0f) % 3u;
      CString hot[3] = {"Hot! HOT!", "Uh oh, so close!", "Hot!!"};
      CString warm[3] = {"Warm...", "Getting warmer!", "Warm!"};
      CString cold[3] = {"Cold!", "Brrr, ice cold!", "Freezing!"};
      pip_say(l, dp < 8.0f ? hot[k] : dp < 16.0f ? warm[k] : cold[k]);
    }
    break;
  case PIP_WAVE:
    f->pose = FIG_POSE_WAVE;
    f->speed = 0.0f;
    l->pip_yaw = m_atan2f(p.x - l->pip_p.x, p.z - l->pip_p.z);
    if (dp > 24.0f || !outside)
      l->pip_mode = PIP_WALK;
    break;
  case PIP_FLEE: {
    l->pip_t += dt;
    v2 away = {l->pip_p.x - p.x, l->pip_p.z - p.z};
    f32 al = v2_length(away);
    away = al > 0.01f ? v2_scale(away, 1.0f / al) : (v2){1.0f, 0.0f};
    f32 wob = m_sinf(l->pip_t * 3.0f) * 0.5f;
    v2 best = away;
    f32 offs[7] = {0.0f, 0.5f, -0.5f, 1.0f, -1.0f, 1.5f, -1.5f};
    for (i32 k = 0; k < 7; k++) { // dodge obstacles
      f32 a = offs[k] + wob;
      v2 dir = {away.x * m_cosf(a) - away.y * m_sinf(a), away.x * m_sinf(a) + away.y * m_cosf(a)};
      f32 tx = l->pip_p.x + dir.x * 2.0f, tz = l->pip_p.z + dir.y * 2.0f;
      if (!life_blocked(l, tx, tz, 0.6f) && !life_in_pond(tx, tz, -0.8f) && tx > -25.5f && tx < 65.5f && tz > -24.8f && tz < 22.6f) {
        best = dir;
        break;
      }
    }
    pip_move(l, (v2){l->pip_p.x + best.x * 3.0f, l->pip_p.z + best.y * 3.0f}, 9.6f * STUD_PER_THREE, dt);
    if (dp < 2.3f && l->pip_t > 0.8f) {
      l->pip_mode = PIP_WAVE;
      l->pip_cd = 25.0f;
      l->save->data.studs += 3;
      l->save->data._reserved[1]++; // tag wins
      save_mark(l->save);
      sound_play(l->snd, SND_FANFARE, 1.0f, 0.6f);
      life_toast(l, "You tagged Pip! +3 studs");
    } else if (l->pip_t > 10.0f) {
      l->pip_mode = PIP_WALK;
      l->pip_cd = 15.0f;
      pip_say(l, "Too slow! Hehe!");
    }
  } break;
  case PIP_FOOTBALL: {
    l->pip_t += dt;
    v2 to_ball = {l->ball_p.x - l->pip_p.x, l->ball_p.z - l->pip_p.z};
    f32 db = v2_length(to_ball);
    f32 pb = v2_length((v2){p.x - l->ball_p.x, p.z - l->ball_p.z});
    if (l->pip_t > 8.0f || pb < 1.8f) {
      l->pip_mode = PIP_WALK;
      l->pip_cd = 4.0f;
      break;
    }
    pip_move(l, (v2){l->ball_p.x, l->ball_p.z}, 6.5f * STUD_PER_THREE, dt);
    if (db < 1.35f) { // shoot at the goal, or pass to you
      v2 aim;
      f32 power;
      if (life_rand(l) < 0.35f && l->ball_p.x > 12.0f) {
        aim = (v2){26.0f - l->ball_p.x, -6.0f - l->ball_p.z};
        power = 11.0f;
      } else {
        aim = (v2){p.x - l->ball_p.x, p.z - l->ball_p.z};
        power = m_minf(10.0f, 3.5f + v2_length(aim) * 0.55f);
      }
      f32 al = v2_length(aim);
      if (al > 0.01f)
        aim = v2_scale(aim, 1.0f / al);
      l->ball_v = (v3){aim.x * power * STUD_PER_THREE, 0.0f, aim.y * power * STUD_PER_THREE};
      sound_play(l->snd, SND_KICK, 1.0f, 0.6f);
      l->pip_mode = PIP_WALK;
      l->pip_cd = 5.0f;
    }
  } break;
  default: { // walking the loop around the pond
    f32 bs = v2_length((v2){l->ball_v.x, l->ball_v.z});
    f32 bp = v2_length((v2){l->ball_p.x - l->pip_p.x, l->ball_p.z - l->pip_p.z});
    f32 pbp = v2_length((v2){p.x - l->ball_p.x, p.z - l->ball_p.z});
    if (outside && sprinting && dp < 2.3f && l->pip_cd <= 0.0f) {
      l->pip_mode = PIP_FLEE;
      l->pip_t = 0.0f;
      sound_play(l->snd, SND_ALERT, 1.0f, 0.6f);
      life_toast(l, "Tag with Pip! Catch him again within 10 seconds (sprint with Shift)");
      break;
    }
    if (outside && l->pip_cd <= 0.0f && bs < 2.5f && l->goal_armed && bp < 16.0f && dp < 20.0f && pbp > 3.0f) {
      l->pip_mode = PIP_FOOTBALL;
      l->pip_t = 0.0f;
      pip_say(l, "My turn!");
      break;
    }
    if (dp < 6.0f) {
      f->speed = 0.0f;
      l->pip_yaw += fig_wrap(m_atan2f(p.x - l->pip_p.x, p.z - l->pip_p.z) - l->pip_yaw) * m_minf(1.0f, dt * 4.0f);
      break;
    }
    if (l->pip_pause > 0.0f) {
      l->pip_pause -= dt;
      f->speed = 0.0f;
      break;
    }
    if (pip_move(l, pip_waypoint(l->pip_wp), 3.2f * STUD_PER_THREE, dt) < 0.1f) {
      l->pip_wp = (l->pip_wp + 1) & 3;
      if (life_rand(l) < 0.5f)
        l->pip_pause = life_range(l, 1.5f, 3.5f);
    }
  } break;
  }
  if (l->pip_mode == PIP_HIDING || l->pip_mode == PIP_WAVE)
    f->speed = 0.0f;
  fig_place(l->w, f, town_m(l->pip_p.x, 0.0f, l->pip_p.z), l->pip_yaw);
}

// ---------------------------------------------------------------- the frame

// main lane, before the scope. move_dir: the player's walking direction (studs, unit or zero)
hz_internal void life_update(Life *l, v3 pos_m, v2 move_dir, f32 speed_m, b32 sprinting, b32 outside, f32 dt) {
  l->time += dt;
  v3 p = v3_scale(pos_m, 1.0f / TOWN_STUD);
  b32 walking = speed_m > 0.5f;
  const DayNightState *dn = ecs_resource_get(l->w, DayNightState);
  l->night = dn ? dn->night : 0.0f;
  if (dn && dn->day != l->day)
    l->day = dn->day;
  if (!l->clock_set && l->save->loaded) {
    EcsEntity e;
    if (life_sun(l, &e)) {
      life_set_clock(l, tod_to_hours(l->save->data.time_of_day), DAY_SECONDS);
      l->clock_set = true;
    }
  }
  if (l->clock_set) {
    l->save_clock_t += dt;
    if (l->save_clock_t > 20.0f) {
      l->save_clock_t = 0.0f;
      l->save->data.time_of_day = hours_to_tod(life_hours(l));
      save_mark(l->save);
    }
  }
  l->toast_t = m_maxf(0.0f, l->toast_t - dt);
  life_golden(l, p, !outside);
  life_ducks(l, p, dt, outside);
  life_fish(l, p, dt, outside);
  life_butterflies(l, p, dt, outside);
  life_fireflies(l, p, dt, outside, walking);
  life_ball(l, p, move_dir, speed_m, dt, outside);
  life_chest(l, p, dt, outside);
  life_weather(l, p, dt, outside, walking);
  life_lamps(l);
  life_pip(l, p, dt, outside, sprinting);
}

// the park bench: sit (time runs 24x faster) until you move
hz_internal void life_sit(Life *l, b32 on) {
  if (on == l->sitting)
    return;
  l->sitting = on;
  life_set_clock(l, -1.0f, on ? DAY_SECONDS / 24.0f : DAY_SECONDS);
  if (on) {
    l->save->data._reserved[2]++; // bench sits
    save_mark(l->save);
    sound_play(l->snd, SND_BLIP, 1.0f, 0.7f);
    life_toast(l, "Time flies on the bench\xe2\x80\xa6 (move to get up)");
  }
}

// the HUD: studs and golden studs top left, the toast top centre, the dark screen of the hide and seek countdown
hz_internal void life_draw(Life *l) {
  OsSafeAreaInsets safe = os_get_safe_area();
  const GameSave *s = &l->save->data;
  u32 n = 0;
  for (u32 k = 0; k < GOLDEN_COUNT; k++)
    n += (s->golden_mask >> k) & 1u;
  if (l->fade) {
    ui_element({.id = ui_id("SeekFade"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = {8, 10, 14, 240}}) {
      cstr_format(l->hud[2], sizeof(l->hud[2]), "Pip is hiding\xe2\x80\xa6 %", fmt_u32((u32)m_ceilf(l->seek_count)));
      ui_text(ui_str_c(l->hud[2]), ui_text_config({.fontSize = 34, .textColor = {255, 255, 255, 255}}));
    }
  }
  ui_element({.id = ui_id("Studs"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                           .offset = {-safe.right - 14.0f, safe.top + 14.0f}},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 14, .padding = {.left = 12, .right = 12, .top = 6, .bottom = 6}},
              .background_color = {16, 18, 24, 170},
              .corner_radius = ui_corner_radius(8)}) {
    cstr_format(l->hud[0], sizeof(l->hud[0]), "\xe2\x97\x8f % studs   \xe2\x98\x85 %/15", fmt_u32(s->studs), fmt_u32(n));
    ui_text(ui_str_c(l->hud[0]), ui_text_config({.fontSize = 16, .textColor = {242, 205, 55, 255}}));
    if (l->seek_left > 0.0f && l->pip_mode == PIP_HIDING) {
      cstr_format(l->hud[1], sizeof(l->hud[1]), "Find Pip: % s", fmt_u32((u32)m_ceilf(m_maxf(0.0f, l->seek_left))));
      ui_text(ui_str_c(l->hud[1]), ui_text_config({.fontSize = 16, .textColor = {255, 140, 120, 255}}));
    }
  }
  if (l->toast_t > 0.0f) {
    u8 a = (u8)(255.0f * m_minf(1.0f, l->toast_t * 3.0f));
    ui_element({.id = ui_id("Toast"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP},
                             .offset = {0.0f, safe.top + 70.0f}},
                .layout = {.padding = {.left = 16, .right = 16, .top = 8, .bottom = 8}},
                .background_color = {250, 250, 246, (u8)(a * 0.95f)},
                .border = {.color = {27, 42, 52, a}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                .corner_radius = ui_corner_radius(10)}) {
      ui_text(ui_str_c((CString)l->toast), ui_text_config({.fontSize = 18, .textColor = {27, 42, 52, a}}));
    }
  }
}
