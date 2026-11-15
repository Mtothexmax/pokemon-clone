// The first deck's quest and the pond cave. In the lab: Prof. Quill's old diving suit (hidden in a corner) and the
// Deck-o-Matic, whose screen shows MISSING PART until you bring its Stud Core. With the suit the pond takes you down
// into the flooded cave (st_cave.c): you swim in first person - on a phone hold a finger to swim forward and move it to
// steer (as abyssal-garden), with keys and mouse W/S A/D, Space / Q up and down, Shift fast, drag to look - between
// swaying kelp, fish schools, mantas, rising bubbles and glowing plants, the water darker the deeper you go. Eight
// pearls to find, and the Stud Core at the bottom of the trench. Swim up into the hole (or the Surface button) to leave.
// Included by starting_template.c after st_vox.c.

#define DIVE_KELP 36
#define DIVE_FISH 42 // three schools of 14
#define DIVE_MANTAS 2
#define DIVE_BUBBLES 48
#define DIVE_TUFTS 40
#define DIVE_PEARLS 8
#define DIVE_LIGHTS 6
#define DIVE_SNOW 64 // marine snow: specks drifting around the diver (they wrap in a box around you)

typedef struct Dive {
  Life *l;
  // the lab
  EcsEntity suit, screen_missing, screen_ready;
  b32 shown_suit, shown_missing, shown_ready;
  // the dive
  b32 on, leaving;
  f32 t, fade;         // seconds under water; the black fade between the pond and the cave (1 black)
  v3 p;                // the diver, cave coordinates in studs
  f32 yaw, pitch;      // the view (yaw 0 looks towards -z)
  v3 vel;
  i32 touch;           // the finger that swims (-1 none)
  v2 touch_at, touch_last;
  f32 fov0;            // the camera's fov before the dive (given back when you surface); 0: not taken yet
  b32 pinch_on;        // two fingers down: pinching them together swims backwards, spreading them swims faster
  f32 pinch_d0;        // their distance when the second one came down (UI px)
  b32 click_surface;
  f32 core_t;          // since the Stud Core was taken (the message)
  // the living cave
  EcsEntity kelp[DIVE_KELP], fish[DIVE_FISH], manta[DIVE_MANTAS], bubble[DIVE_BUBBLES], tuft[DIVE_TUFTS], pearl[DIVE_PEARLS], core,
      surface, lamp, light[DIVE_LIGHTS], snow[DIVE_SNOW];
  v3 kelp_at[DIVE_KELP];
  f32 kelp_s[DIVE_KELP];
  v3 pearl_at[DIVE_PEARLS];
  b32 built;
  char txt[2][64];
} Dive;

// ---- the meshes (studs, built at the origin) ----
hz_internal void shape_suit(Town *t) {
  const u32 brass = 0xc9a227u, yel = 0xe8b923u, dark = 0x2b2f36u, glass = 0x2a5d8au;
  town_box(t, 0x6b4a2au, 2.6f, 0.4f, 2.0f, 0.0f, 0.0f, 0.0f, true);           // the wooden stand
  town_box(t, dark, 0.9f, 0.8f, 1.0f, -0.55f, 0.4f, 0.0f, false);             // boots
  town_box(t, dark, 0.9f, 0.8f, 1.0f, 0.55f, 0.4f, 0.0f, false);
  town_box(t, yel, 0.85f, 1.5f, 0.85f, -0.5f, 1.2f, 0.0f, false);             // legs
  town_box(t, yel, 0.85f, 1.5f, 0.85f, 0.5f, 1.2f, 0.0f, false);
  town_box(t, yel, 2.1f, 1.8f, 1.1f, 0.0f, 2.7f, 0.0f, false);                // the body
  town_box(t, brass, 2.2f, 0.3f, 1.2f, 0.0f, 3.0f, 0.0f, false);              // the belt
  town_box(t, yel, 0.6f, 1.6f, 0.7f, -1.4f, 2.9f, 0.0f, false);               // arms
  town_box(t, yel, 0.6f, 1.6f, 0.7f, 1.4f, 2.9f, 0.0f, false);
  town_box(t, brass, 1.8f, 0.4f, 1.6f, 0.0f, 4.5f, 0.0f, false);              // the collar
  town_cyl(t, brass, 0.0f, 4.9f, 0.0f, 1.0f, 1.0f, 1.5f, 16, true);           // the helmet
  town_cyl(t, brass, 0.0f, 6.4f, 0.0f, 1.0f, 0.55f, 0.35f, 16, true);
  town_box(t, glass, 1.0f, 0.9f, 0.3f, 0.0f, 5.2f, 0.95f, false);             // the window
  town_box(t, brass, 1.2f, 0.15f, 0.35f, 0.0f, 6.1f, 0.98f, false);
  town_box(t, brass, 1.2f, 0.15f, 0.35f, 0.0f, 5.05f, 0.98f, false);
  town_box(t, dark, 0.35f, 0.35f, 1.4f, 0.0f, 3.6f, -0.9f, false);            // the air hose
}
hz_internal void shape_kelp_a(Town *t) { town_box(t, 0x7a9a2eu, 0.2f, 16.0f, 0.7f, 0.0f, 0.0f, 0.0f, false); }
hz_internal void shape_kelp_b(Town *t) { town_box(t, 0x2f8a4au, 0.2f, 16.0f, 0.6f, 0.0f, 0.0f, 0.0f, false); }
hz_internal void shape_kelp_c(Town *t) { town_box(t, 0x2a9a9au, 0.2f, 16.0f, 0.5f, 0.0f, 0.0f, 0.0f, false); }
hz_internal void shape_cfish(Town *t) {
  town_box(t, 0x7fb0ccu, 0.4f, 0.4f, 1.0f, 0.0f, -0.2f, 0.0f, false);
  town_box(t, 0x355c80u, 0.08f, 0.45f, 0.35f, 0.0f, -0.22f, 0.65f, false);
}
hz_internal void shape_cfish_y(Town *t) {
  town_box(t, 0xf2cc4du, 0.4f, 0.4f, 1.0f, 0.0f, -0.2f, 0.0f, false);
  town_box(t, 0xb08a20u, 0.08f, 0.45f, 0.35f, 0.0f, -0.22f, 0.65f, false);
}
hz_internal void shape_manta(Town *t) {
  town_box(t, 0x0f1a2au, 2.0f, 0.35f, 3.0f, 0.0f, -0.17f, 0.0f, false);
  town_box_rot(t, 0x12203au, (v3){3.4f, 0.2f, 2.0f}, (v3){-2.4f, 0.0f, 0.2f}, (v3){0.0f, 0.0f, 0.12f});
  town_box_rot(t, 0x12203au, (v3){3.4f, 0.2f, 2.0f}, (v3){2.4f, 0.0f, 0.2f}, (v3){0.0f, 0.0f, -0.12f});
  town_box(t, 0x8a96a8u, 1.4f, 0.1f, 2.0f, 0.0f, 0.18f, 0.0f, false); // the pale stripe
  town_box(t, 0x0f1a2au, 0.15f, 0.15f, 3.0f, 0.0f, -0.07f, 2.8f, false); // the tail
}
hz_internal void shape_bubble(Town *t) { town_box(t, C_WHITE, 0.5f, 0.5f, 0.5f, 0.0f, -0.25f, 0.0f, false); }
hz_internal void shape_tuft(Town *t) {
  for (i32 i = 0; i < 5; i++) {
    f32 a = (f32)i * 1.256f;
    town_box_rot(t, C_WHITE, (v3){0.25f, 1.6f + 0.3f * (f32)(i % 2), 0.25f}, (v3){m_sinf(a) * 0.35f, 0.8f, m_cosf(a) * 0.35f},
                 (v3){m_cosf(a) * 0.35f, 0.0f, -m_sinf(a) * 0.35f});
  }
}
hz_internal void shape_pearl(Town *t) {
  town_cyl(t, C_WHITE, 0.0f, -0.4f, 0.0f, 0.7f, 0.7f, 0.8f, 12, true);
  town_cyl(t, C_WHITE, 0.0f, 0.4f, 0.0f, 0.4f, 0.4f, 0.3f, 12, true);
}
hz_internal void shape_core(Town *t) { town_box(t, C_WHITE, 2.0f, 1.2f, 2.0f, 0.0f, -0.6f, 0.0f, true); }
hz_internal void shape_surface(Town *t) { town_box(t, C_WHITE, 12.0f, 0.4f, 12.0f, 0.0f, 0.0f, 0.0f, false); }

hz_internal EcsEntity dive_glb(Life *l, CString path, v3 studs, f32 yaw) {
  HzHandleT(ModelBlobAsset) model = asset_load(ModelBlobAsset, ecs_resource_get_mut(l->w, AssetSystem), .path = path);
  return ecs_renderer_spawn(l->w, l->r,
                            &(EcsRendererSpawnDesc){.model_asset = blob_asset_ref_from_handle(ModelBlobAsset, model),
                                                    .transform = {.position = town_m(studs.x, studs.y, studs.z),
                                                                  .rotation = quat_from_euler((v3){0.0f, yaw, 0.0f}), .scale = V3_ONE},
                                                    .casts_shadow = false});
}

hz_internal EcsEntity dive_light(EcsWorld *w, v3 studs, v4 color, f32 radius, f32 intensity) {
  EcsEntity e = ecs_entity_create(w);
  v3 at = town_m(studs.x, studs.y, studs.z);
  ecs_component_set(w, e, LocalTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, WorldTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, EcsPointLight, {.color = {color.x, color.y, color.z, color.w}, .radius = radius, .intensity = intensity});
  return e;
}

hz_internal void dive_move_light(EcsWorld *w, EcsEntity e, v3 studs) {
  v3 at = town_m(studs.x, studs.y, studs.z);
  ecs_component_set(w, e, LocalTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, WorldTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
}

hz_internal v3 cave_world(v3 c) { return (v3){CAVE_X + c.x, c.y, CAVE_Z + c.z}; }

hz_internal void dive_init(Dive *d, Life *l) {
  *d = (Dive){.l = l, .touch = -1};
  // the lab: the suit in the corner behind the bookshelves, the Deck-o-Matic's two screens (st_rooms.c's middle machine)
  d->suit = life_spawn(l, life_mesh(l, shape_suit), renderer_default_material(l->r), (v3){314.2f, 0.4f, -9.4f}, true);
  life_set(l, d->suit, (v3){314.2f, 0.4f, -9.4f}, 0.6f, V3_ONE);
  d->screen_missing = dive_glb(l, "brickhollow/signs/machine_missing.glb", (v3){341.6f, 5.1f, -8.19f}, 0.0f);
  d->screen_ready = dive_glb(l, "brickhollow/signs/machine_ready.glb", (v3){341.6f, 5.1f, -8.19f}, 0.0f);
  d->shown_suit = d->shown_missing = d->shown_ready = true;
  // the cave's living things
  HzHandleT(RendererMaterial) plain = renderer_default_material(l->r);
  HzHandleT(RendererSubMesh) kelp_m[3] = {life_mesh(l, shape_kelp_a), life_mesh(l, shape_kelp_b), life_mesh(l, shape_kelp_c)};
  for (i32 i = 0; i < DIVE_KELP; i++) { // the kelp forest in the south-west, a few strands elsewhere
    f32 x = i < 30 ? -36.0f + cave_hash(i, 11) * 28.0f : -30.0f + cave_hash(i, 12) * 60.0f;
    f32 z = i < 30 ? -36.0f + cave_hash(i, 13) * 30.0f : -30.0f + cave_hash(i, 14) * 40.0f;
    d->kelp_at[i] = (v3){x, cave_floor(x, z), z};
    d->kelp_s[i] = 1.4f + cave_hash(i, 15) * 1.6f;
    d->kelp[i] = life_spawn(l, kelp_m[i % 3], plain, cave_world(d->kelp_at[i]), false);
  }
  HzHandleT(RendererSubMesh) fish_m = life_mesh(l, shape_cfish), fish_y = life_mesh(l, shape_cfish_y);
  for (i32 i = 0; i < DIVE_FISH; i++)
    d->fish[i] = life_spawn(l, i % 7 == 3 ? fish_y : fish_m, plain, (v3){CAVE_X, -60.0f, 0.0f}, false);
  HzHandleT(RendererSubMesh) manta_m = life_mesh(l, shape_manta);
  for (i32 i = 0; i < DIVE_MANTAS; i++)
    d->manta[i] = life_spawn(l, manta_m, plain, (v3){CAVE_X, -60.0f, 0.0f}, false);
  HzHandleT(RendererSubMesh) bub_m = life_mesh(l, shape_bubble);
  HzHandleT(RendererMaterial) bub_mat = life_glow_material(l, (v4){0.75f, 0.9f, 1.0f, 1.0f}, 0.6f);
  for (i32 i = 0; i < DIVE_BUBBLES; i++)
    d->bubble[i] = life_spawn(l, bub_m, bub_mat, (v3){CAVE_X, -60.0f, 0.0f}, false);
  // glowing tufts: magenta and violet in the grotto, cyan and gold here and there on the ledges and the floor
  HzHandleT(RendererSubMesh) tuft_m = life_mesh(l, shape_tuft);
  HzHandleT(RendererMaterial) tuft_mat[4] = {life_glow_material(l, (v4){0.85f, 0.05f, 0.4f, 1.0f}, 5.0f),
                                             life_glow_material(l, (v4){0.6f, 0.1f, 0.8f, 1.0f}, 5.0f),
                                             life_glow_material(l, (v4){0.1f, 0.85f, 0.9f, 1.0f}, 4.0f),
                                             life_glow_material(l, (v4){1.0f, 0.75f, 0.2f, 1.0f}, 4.0f)};
  for (i32 i = 0; i < DIVE_TUFTS; i++) {
    b32 grotto = i < 22;
    f32 x = grotto ? 14.0f + cave_hash(i, 21) * 24.0f : -36.0f + cave_hash(i, 22) * 72.0f;
    f32 z = grotto ? -38.0f + cave_hash(i, 23) * 20.0f : -36.0f + cave_hash(i, 24) * 72.0f;
    f32 s = 0.8f + cave_hash(i, 25) * 0.9f;
    v3 at = cave_world((v3){x, cave_floor(x, z), z});
    d->tuft[i] = life_spawn(l, tuft_m, tuft_mat[grotto ? i % 2 : 2 + i % 2], at, false);
    life_set(l, d->tuft[i], at, cave_hash(i, 26) * 6.28f, (v3){s, s, s});
  }
  // the pearls: on ledges, in the grotto, by the boat, under the arch, in the kelp, by the whale tail
  v3 pearls[DIVE_PEARLS] = {{-34.0f, 0, -30.0f}, {28.0f, 0, -32.0f}, {-26.0f, 0, 22.0f}, {-20.0f, 36.0f, 0.0f},
                            {2.0f, 0, -14.0f}, {34.0f, 0, 30.0f}, {-8.0f, 0, 34.0f}, {18.0f, 0, 2.0f}};
  HzHandleT(RendererSubMesh) pearl_m = life_mesh(l, shape_pearl);
  HzHandleT(RendererMaterial) pearl_mat = life_glow_material(l, (v4){0.85f, 0.95f, 1.0f, 1.0f}, 3.5f);
  for (i32 i = 0; i < DIVE_PEARLS; i++) {
    v3 p = pearls[i];
    if (p.y == 0.0f)
      p.y = cave_floor(p.x, p.z) + 1.4f;
    d->pearl_at[i] = p;
    d->pearl[i] = life_spawn(l, pearl_m, pearl_mat, cave_world(p), false);
  }
  HzHandleT(RendererMaterial) snow_mat = life_glow_material(l, (v4){0.8f, 0.9f, 1.0f, 1.0f}, 0.8f);
  for (i32 i = 0; i < DIVE_SNOW; i++)
    d->snow[i] = life_spawn(l, bub_m, snow_mat, (v3){CAVE_X, -60.0f, 0.0f}, false);
  d->core = life_spawn(l, life_mesh(l, shape_core), life_glow_material(l, (v4){1.0f, 0.78f, 0.15f, 1.0f}, 1.6f),
                       cave_world((v3){0.0f, -19.6f, 34.0f}), false);
  d->surface = life_spawn(l, life_mesh(l, shape_surface), life_glow_material(l, (v4){0.55f, 0.85f, 1.0f, 1.0f}, 2.5f),
                          cave_world((v3){0.0f, CAVE_TOP + 19.0f, 0.0f}), false);
  // the lights: daylight down the hole, the grotto, the kelp, the trench, the core, and your helmet lamp
  EcsWorld *w = l->w;
  d->light[0] = dive_light(w, cave_world((v3){0.0f, CAVE_TOP - 4.0f, 0.0f}), (v4){0.55f, 0.85f, 1.0f, 1.0f}, 34.0f, 2600000.0f);
  d->light[1] = dive_light(w, cave_world((v3){24.0f, 14.0f, -28.0f}), (v4){0.85f, 0.25f, 0.85f, 1.0f}, 18.0f, 900000.0f);
  d->light[2] = dive_light(w, cave_world((v3){-22.0f, 22.0f, -22.0f}), (v4){0.3f, 0.95f, 0.7f, 1.0f}, 18.0f, 700000.0f);
  d->light[3] = dive_light(w, cave_world((v3){0.0f, -6.0f, 26.0f}), (v4){0.2f, 0.6f, 1.0f, 1.0f}, 14.0f, 500000.0f);
  d->light[4] = dive_light(w, cave_world((v3){0.0f, -17.0f, 34.0f}), (v4){1.0f, 0.75f, 0.25f, 1.0f}, 12.0f, 220000.0f);
  d->light[5] = dive_light(w, cave_world((v3){-24.0f, 10.0f, 18.0f}), (v4){0.4f, 0.8f, 1.0f, 1.0f}, 14.0f, 500000.0f);
  d->lamp = dive_light(w, (v3){CAVE_X, -60.0f, 0.0f}, (v4){1.0f, 0.96f, 0.85f, 1.0f}, 9.0f, 90000.0f);
  d->built = true;
}

hz_internal void dive_hide(Dive *d, EcsEntity e) { life_set(d->l, e, (v3){0.0f, -150.0f, 0.0f}, 0.0f, (v3){0.01f, 0.01f, 0.01f}); }

hz_internal void dive_show(EcsWorld *w, EcsEntity e, b32 *shown, b32 want) {
  if (*shown == want)
    return;
  *shown = want;
  const EcsVisibility *v = ecs_component_get(w, e, EcsVisibility);
  if (v) {
    EcsVisibility nv = *v;
    nv.visible = want;
    ecs_component_set(w, e, EcsVisibility, nv);
  }
}

// main lane: the lab follows the quest (the suit until taken, the screen), the flags for the targets (st_world.c)
hz_internal void dive_lab(Dive *d, const GameSave *s) {
  dive_show(d->l->w, d->suit, &d->shown_suit, !s->has_suit && !s->has_deck);
  dive_show(d->l->w, d->screen_missing, &d->shown_missing, !s->has_deck);
  dive_show(d->l->w, d->screen_ready, &d->shown_ready, s->has_deck);
  g_suit_here = !s->has_suit && !s->has_deck;
  g_can_dive = s->has_suit || s->has_deck;
}

hz_internal void dive_start(Dive *d) {
  d->on = true;
  d->leaving = false;
  d->t = 0.0f;
  d->fade = 1.0f;
  d->p = (v3){0.0f, CAVE_TOP + 9.0f, 0.0f};
  d->yaw = 0.0f;
  d->pitch = -1.25f;
  d->vel = V3_ZERO;
  d->touch = -1;
  d->core_t = -1.0f;
}

hz_internal v3 dive_forward(const Dive *d) {
  f32 cp = m_cosf(d->pitch);
  return (v3){-m_sinf(d->yaw) * cp, m_sinf(d->pitch), -m_cosf(d->yaw) * cp};
}

// keep the diver in the water: in the basin, above the floor, below the ceiling (but up the hole), out of the rocks
hz_internal void dive_collide(Dive *d) {
  const f32 r = 1.2f;
  v3 p = d->p;
  b32 in_hole = m_absf(p.x) < CAVE_HOLE - 1.0f && m_absf(p.z) < CAVE_HOLE - 1.0f;
  p.x = m_clampf(p.x, -CAVE_R + r + 0.5f, CAVE_R - r - 0.5f);
  p.z = m_clampf(p.z, -CAVE_R + r + 0.5f, CAVE_R - r - 0.5f);
  p.y = m_minf(p.y, in_hole ? CAVE_TOP + 16.0f : CAVE_TOP - r);
  p.y = m_maxf(p.y, cave_floor(p.x, p.z) + r);
  for (u32 i = 0; i < g_cave_solid_n; i++) {
    CaveBox b = g_cave_solid[i];
    if (p.x < b.x0 - r || p.x > b.x1 + r || p.y < b.y0 - r || p.y > b.y1 + r || p.z < b.z0 - r || p.z > b.z1 + r)
      continue;
    f32 pen[6] = {p.x - (b.x0 - r), (b.x1 + r) - p.x, p.y - (b.y0 - r), (b.y1 + r) - p.y, p.z - (b.z0 - r), (b.z1 + r) - p.z};
    i32 best = 0;
    for (i32 k = 1; k < 6; k++)
      if (pen[k] < pen[best])
        best = k;
    switch (best) {
    case 0: p.x = b.x0 - r; break;
    case 1: p.x = b.x1 + r; break;
    case 2: p.y = b.y0 - r; break;
    case 3: p.y = b.y1 + r; break;
    case 4: p.z = b.z0 - r; break;
    default: p.z = b.z1 + r; break;
    }
  }
  d->p = p;
}

// main lane, every frame while diving: steer, swim, collect, animate; returns true when it is over (back at the pond)
hz_internal b32 dive_update(Dive *d, InputSystem *in, UISystem *ui, SaveState *save, Camera *cam, f32 sw, f32 sh, f32 dt) {
  if (!d->on)
    return false;
  Life *l = d->l;
  GameSave *s = &save->data;
  d->t += dt;
  d->fade = d->leaving ? m_minf(1.0f, d->fade + dt / 0.45f) : m_maxf(0.0f, d->fade - dt / 0.6f);
  if (d->leaving && d->fade >= 1.0f) {
    d->on = false;
    if (cam && d->fov0 > 0.0f)
      cam->fov = d->fov0;
    d->fov0 = 0.0f;
    for (i32 i = 0; i < DIVE_FISH; i++)
      dive_hide(d, d->fish[i]);
    for (i32 i = 0; i < DIVE_SNOW; i++)
      dive_hide(d, d->snow[i]);
    dive_move_light(l->w, d->lamp, (v3){CAVE_X, -60.0f, 0.0f});
    return true;
  }
  // ---- steering: a held finger swims forward and steers like a stick from where it went down; the mouse looks ----
  f32 short_side = m_maxf(m_minf(sw, sh), 1.0f);
  b32 fwd_touch = false, fast_touch = false;
  i32 fingers = 0;
  v2 tp[2] = {0};
  i32 down = 0; // fingers on the glass (two: a pinch, the swim finger does not steer meanwhile)
  for (i32 i = 0; i < MAX_TOUCHES; i++)
    down += in->touches.items[i].is_active ? 1 : 0;
  for (i32 i = 0; i < MAX_TOUCHES; i++) {
    MobileTouch *t = &in->touches.items[i];
    if (!t->is_active && !t->started_this_frame)
      continue;
    if (fingers < 2)
      tp[fingers] = (v2){t->current_x, t->current_y};
    fingers++;
    if (d->touch < 0 && t->started_this_frame && !t->claimed && !ui_point_over_ui(ui, (v2){t->start_x, t->start_y})) {
      d->touch = (i32)t->id;
      d->touch_at = (v2){t->start_x, t->start_y};
      d->touch_last = d->touch_at;
    }
    if ((i32)t->id == d->touch) {
      if (t->stopped_this_frame || !t->is_active) {
        d->touch = -1;
        continue;
      }
      fwd_touch = true;
      v2 now = {t->current_x, t->current_y};
      if (down >= 2) {
        d->touch_at = d->touch_last = now; // pinching: no steering; after it the stick starts from here
        continue;
      }
      f32 jx = m_clampf((now.x - d->touch_at.x) / (0.25f * short_side), -1.0f, 1.0f);
      f32 jy = m_clampf((now.y - d->touch_at.y) / (0.25f * short_side), -1.0f, 1.0f);
      d->yaw -= 1.5f * jx * dt;    // a stick: hold it out to keep turning
      d->pitch -= 1.0f * jy * dt;
      d->yaw -= (now.x - d->touch_last.x) * 2.2f / short_side; // and the drag turns the view at once
      d->pitch -= (now.y - d->touch_last.y) * 2.2f / short_side;
      d->touch_last = now;
    }
  }
  if (d->touch >= 0) {
    b32 alive = false;
    for (i32 i = 0; i < MAX_TOUCHES; i++)
      alive |= (i32)in->touches.items[i].id == d->touch && in->touches.items[i].is_active;
    if (!alive)
      d->touch = -1;
  }
  // the pinch: together (closer than when the second finger came down) swims backwards, apart swims faster
  f32 squeeze = 0.0f;
  if (fingers >= 2) {
    f32 dist = v2_length(v2_sub(tp[0], tp[1]));
    if (!d->pinch_on) {
      d->pinch_on = true;
      d->pinch_d0 = dist;
    }
    squeeze = m_clampf((d->pinch_d0 - dist) / (0.12f * short_side), -1.0f, 1.0f);
  } else
    d->pinch_on = false;
  fast_touch = squeeze < -0.2f;
  b32 back_touch = squeeze > 0.2f;
  b32 look = in->buttons[MOUSE_RIGHT].is_pressed || (in->buttons[MOUSE_LEFT].is_pressed && !fwd_touch && !ui_point_over_ui(ui, in->mouse_pos));
  if (look && fingers == 0) {
    d->yaw -= in->mouse_delta.x * 0.0035f;
    d->pitch -= in->mouse_delta.y * 0.0035f;
  }
  d->yaw += ((in->buttons[KEY_LEFT].is_pressed ? 1.0f : 0.0f) - (in->buttons[KEY_RIGHT].is_pressed ? 1.0f : 0.0f)) * 1.8f * dt;
  d->pitch += ((in->buttons[KEY_UP].is_pressed ? 1.0f : 0.0f) - (in->buttons[KEY_DOWN].is_pressed ? 1.0f : 0.0f)) * 1.2f * dt;
  d->pitch = m_clampf(d->pitch, -1.5f, 1.5f);
  // ---- swimming: along the view, sideways, straight up and down ----
  v3 f = dive_forward(d), right = {m_cosf(d->yaw), 0.0f, -m_sinf(d->yaw)};
  f32 ax = (in->buttons[KEY_D].is_pressed ? 1.0f : 0.0f) - (in->buttons[KEY_A].is_pressed ? 1.0f : 0.0f);
  f32 az = (in->buttons[KEY_W].is_pressed ? 1.0f : 0.0f) - (in->buttons[KEY_S].is_pressed ? 1.0f : 0.0f);
  f32 ay = (in->buttons[KEY_SPACE].is_pressed ? 1.0f : 0.0f) - (in->buttons[KEY_Q].is_pressed || in->buttons[KEY_LEFT_CONTROL].is_pressed ? 1.0f : 0.0f);
  if (back_touch)
    az = -m_clampf(squeeze, 0.4f, 1.0f); // pinched: backwards (stronger the more you pinch)
  else if (fwd_touch && fingers < 2)
    az = m_maxf(az, 0.7f);
  else if (fast_touch)
    az = 1.0f;
  f32 speed = (in->buttons[KEY_LEFT_SHIFT].is_pressed || fast_touch) ? 22.0f : 9.0f;
  v3 want = v3_scale(v3_add(v3_add(v3_scale(f, az), v3_scale(right, ax)), (v3){0.0f, ay, 0.0f}), speed);
  if (v3_length(want) > speed)
    want = v3_scale(v3_normalize(want), speed);
  if (d->fade > 0.6f || d->leaving)
    want = V3_ZERO;
  f32 k = 1.0f - m_expf(-4.5f * dt);
  d->vel = v3_lerp(d->vel, want, k);
  d->p = v3_add(d->p, v3_scale(d->vel, dt));
  dive_collide(d);
  // up the hole: back to the pond
  if (!d->leaving && (d->p.y > CAVE_TOP + 12.0f || d->click_surface || in->buttons[KEY_ESCAPE].pressed_this_frame) && d->t > 1.0f) {
    d->leaving = true;
    sound_play(l->snd, SND_SPLASH, 1.0f, 0.7f);
  }
  d->click_surface = false;
  // ---- the camera is your eyes, the helmet lamp a little ahead ----
  if (cam) {
    cam->pos = town_m(CAVE_X + d->p.x, d->p.y, CAVE_Z + d->p.z);
    cam->rot = quat_look(v3_scale(f, -1.0f), V3_UP);
    // a wide lens under water: at least 105 degrees across (a phone held upright saw a slit before), 75 up and down,
    // never more than 120 up and down (a real perspective, no warping afterwards)
    if (d->fov0 <= 0.0f)
      d->fov0 = cam->fov;
    f32 aspect = m_maxf(sw / m_maxf(sh, 1.0f), 0.2f);
    f32 across = 105.0f * (M_PI / 180.0f);
    f32 v = 2.0f * m_atanf(m_tanf(across * 0.5f) / aspect) * (180.0f / M_PI);
    cam->fov = m_clampf(v, 75.0f, 120.0f);
  }
  dive_move_light(l->w, d->lamp, cave_world(v3_add(d->p, v3_scale(f, 3.0f))));
  // ---- the living cave ----
  f32 t = d->t;
  for (i32 i = 0; i < DIVE_KELP; i++) { // sways about its root, taller strands more
    f32 sw_ = 0.05f + 0.02f * d->kelp_s[i];
    quat q = quat_from_euler((v3){m_sinf(t * 0.7f + (f32)i) * sw_, (f32)i * 1.7f, m_cosf(t * 0.55f + (f32)i * 0.6f) * sw_});
    life_set_rot(l, d->kelp[i], cave_world(d->kelp_at[i]), q, (v3){d->kelp_s[i], d->kelp_s[i], d->kelp_s[i]});
  }
  static const f32 school[3][5] = {{-14.0f, -18.0f, 14.0f, 22.0f, 0.35f}, {18.0f, 14.0f, 18.0f, 34.0f, -0.28f}, {-6.0f, 10.0f, 12.0f, 46.0f, 0.22f}};
  for (i32 i = 0; i < DIVE_FISH; i++) { // three schools circling, each fish a little apart and bobbing
    i32 sc = i / 14, j = i % 14;
    f32 a = t * school[sc][4] + (f32)j * 0.09f;
    f32 rr = school[sc][2] + (cave_hash(i, 31) - 0.5f) * 4.0f;
    v3 p = {school[sc][0] + m_sinf(a) * rr, school[sc][3] + (cave_hash(i, 32) - 0.5f) * 5.0f + m_sinf(t * 1.3f + (f32)j) * 0.6f,
            school[sc][1] + m_cosf(a) * rr};
    f32 yaw = a + (school[sc][4] > 0.0f ? M_PI * 0.5f : -M_PI * 0.5f) + m_sinf(t * 6.0f + (f32)j) * 0.15f;
    life_set(l, d->fish[i], cave_world(p), yaw, (v3){1.4f, 1.4f, 1.4f});
  }
  for (i32 i = 0; i < DIVE_MANTAS; i++) { // two mantas gliding wide circles, wings flapping slowly
    f32 a = t * (i ? -0.12f : 0.15f) + (f32)i * 2.0f;
    f32 rr = i ? 26.0f : 20.0f;
    v3 p = {m_sinf(a) * rr, (i ? 56.0f : 34.0f) + m_sinf(t * 0.4f) * 3.0f, m_cosf(a) * rr};
    f32 yaw = a + (i ? -M_PI * 0.5f : M_PI * 0.5f);
    quat q = quat_from_euler((v3){0.0f, yaw, (i ? 0.28f : -0.28f) + m_sinf(t * 1.2f) * 0.18f});
    f32 flap = 1.0f + 0.12f * m_sinf(t * 1.6f + (f32)i);
    life_set_rot(l, d->manta[i], cave_world(p), q, (v3){1.6f * flap, 1.6f, 1.6f});
  }
  static const f32 vents[6][2] = {{-10.0f, 6.0f}, {12.0f, -6.0f}, {26.0f, -26.0f}, {-28.0f, -14.0f}, {4.0f, 30.0f}, {30.0f, 22.0f}};
  for (i32 i = 0; i < DIVE_BUBBLES; i++) { // rising from six vents, wobbling, shrinking as they go
    i32 v = i % 6;
    f32 base = cave_floor(vents[v][0], vents[v][1]);
    f32 rise = m_fmodf(t * (5.0f + cave_hash(i, 41) * 4.0f) + cave_hash(i, 42) * 60.0f, CAVE_TOP - base);
    f32 wob = m_sinf(t * 3.0f + (f32)i) * 0.5f;
    f32 sz = 0.5f + cave_hash(i, 43) * 0.7f;
    life_set(l, d->bubble[i], cave_world((v3){vents[v][0] + wob, base + rise, vents[v][1] + m_cosf(t * 2.5f + (f32)i) * 0.5f}), 0.0f,
             (v3){sz, sz, sz});
  }
  for (i32 i = 0; i < DIVE_SNOW; i++) { // marine snow: sinking slowly, wrapped into a 28-stud box around the diver
    const f32 B = 28.0f;
    v3 o = {cave_hash(i, 51) * B, cave_hash(i, 52) * B - t * (0.4f + cave_hash(i, 53) * 0.5f), cave_hash(i, 54) * B};
    v3 rel = {m_fmodf(o.x - d->p.x, B), m_fmodf(o.y - d->p.y, B), m_fmodf(o.z - d->p.z, B)};
    rel.x += rel.x < 0.0f ? B : 0.0f;
    rel.y += rel.y < 0.0f ? B : 0.0f;
    rel.z += rel.z < 0.0f ? B : 0.0f;
    v3 at = v3_add(d->p, (v3){rel.x - B * 0.5f, rel.y - B * 0.5f, rel.z - B * 0.5f});
    f32 sz = 0.16f + cave_hash(i, 55) * 0.14f;
    life_set(l, d->snow[i], cave_world(at), 0.0f, (v3){sz, sz, sz});
  }
  // ---- the pearls and the Stud Core ----
  for (i32 i = 0; i < DIVE_PEARLS; i++) {
    if ((s->dive_found >> i) & 1u) {
      dive_hide(d, d->pearl[i]);
      continue;
    }
    v3 p = d->pearl_at[i];
    life_set(l, d->pearl[i], cave_world((v3){p.x, p.y + m_sinf(t * 2.0f + (f32)i) * 0.3f, p.z}), t * 1.2f, V3_ONE);
    if (v3_length(v3_sub(d->p, p)) < 3.0f) {
      s->dive_found |= 1u << i;
      s->studs += 5;
      save_mark(save);
      u32 n = 0;
      for (i32 k2 = 0; k2 < DIVE_PEARLS; k2++)
        n += (s->dive_found >> k2) & 1u;
      sound_play(l->snd, SND_COIN, 1.3f, 0.8f);
      LIFE_TOAST(l, "A glowing pearl! (%/%) \xc2\xb7 +5 studs", fmt_u32(n), fmt_i32(DIVE_PEARLS));
    }
  }
  if (!s->has_part && !s->has_deck) {
    v3 cp = {0.0f, -19.6f + m_sinf(t * 1.5f) * 0.4f, 34.0f};
    f32 pulse = 1.0f + 0.08f * m_sinf(t * 4.0f);
    life_set(l, d->core, cave_world(cp), t * 0.8f, (v3){pulse, pulse, pulse});
    if (v3_length(v3_sub(d->p, cp)) < 5.0f) {
      s->has_part = true;
      save_mark(save);
      d->core_t = 0.0f;
      sound_play(l->snd, SND_FANFARE, 1.0f, 0.9f);
      life_toast(l, "You found the Stud Core! Swim back up and bring it to Prof. Quill.");
    }
  } else {
    dive_hide(d, d->core);
  }
  if (d->core_t >= 0.0f)
    d->core_t += dt;
  return false;
}

// how deep you are, 0 at the hole .. 1 at the bottom of the trench (the water's colour, the light)
hz_internal f32 dive_depth(const Dive *d) { return m_clampf((CAVE_TOP - d->p.y) / (CAVE_TOP + 24.0f), 0.0f, 1.0f); }

// the UI: the water's tint and edge, depth and pearls, the hint, the Surface button, the fade
hz_internal void dive_draw(Dive *d, const GameSave *s, const InputSystem *in, b32 touch, f32 sw, f32 sh) {
  if (!d->on)
    return;
  OsSafeAreaInsets safe = os_get_safe_area();
  f32 dep = dive_depth(d);
  // the water: a blue-green veil, darker and bluer below; the edges darkest (a helmet's window)
  ui_element({.id = ui_id("DiveTint"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 5, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
              .gradient = cu_lin2(180.0f, cu_rgba(0x2994e6, 0.08f * (1.0f - dep)), cu_shade(0x02101e, 0.08f + 0.2f * dep))}) {}
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 6, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
              .gradient = cu_rad(0.5f, 0.5f, cu_rgba(0x000000, 0.0f), cu_shade(0x010810, 0.35f + 0.2f * dep), 1.0f)}) {}
  // little bubbles from the helmet, now and then
  for (i32 i = 0; i < 6; i++) {
    f32 ph = m_fmodf(d->t * 0.35f + (f32)i * 0.17f, 1.0f);
    if (ph > 0.6f)
      continue;
    f32 q = ph / 0.6f, r = 4.0f + (f32)(i % 3) * 3.0f;
    f32 x = sw * (i % 2 ? 0.86f : 0.12f) + m_sinf(d->t * 3.0f + (f32)i) * 8.0f, y = sh * (0.95f - q * 0.9f);
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 7, .offset = {x, y}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(r * 2.0f), .height = ui_sizing_fixed(r * 2.0f)}},
                .corner_radius = ui_corner_radius(r),
                .border = {.color = cu_rgba(0xcfeaff, 0.6f * (1.0f - q)), .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
                .background_color = cu_rgba(0xffffff, 0.08f * (1.0f - q))}) {}
  }
  // depth, pearls, what is still to find
  u32 n = 0;
  for (i32 k = 0; k < DIVE_PEARLS; k++)
    n += (s->dive_found >> k) & 1u;
  b32 narrow = sw < 600.0f; // a phone upright: a short pill top left, the Surface button keeps the right
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 8,
                           .attach_points = {.element = narrow ? UI_ATTACH_POINT_LEFT_TOP : UI_ATTACH_POINT_CENTER_TOP,
                                             .parent = narrow ? UI_ATTACH_POINT_LEFT_TOP : UI_ATTACH_POINT_CENTER_TOP},
                           .offset = {narrow ? safe.left + 12.0f : 0.0f, safe.top + 14.0f}},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(narrow ? 10 : 16),
                         .padding = {.left = 12, .right = 12, .top = 7, .bottom = 7}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_shade(0x041424, 0.6f),
              .corner_radius = ui_corner_radius(18)}) {
    f32 metres = m_maxf(0.0f, (CAVE_TOP + 4.0f - d->p.y) * TOWN_STUD);
    if (narrow) {
      cstr_format(d->txt[0], sizeof(d->txt[0]), "% m", fmt_f32_p(metres, 0));
      cstr_format(d->txt[1], sizeof(d->txt[1]), "\xe2\x97\x86 %/%", fmt_u32(n), fmt_i32(DIVE_PEARLS));
      cu_textb(d->txt[0], 14, cu_rgb(0x9fe0ff), 700, false, (HzUIColor){0});
      cu_textb(d->txt[1], 14, cu_rgb(0xe8f6ff), 700, false, (HzUIColor){0});
      b32 found = s->has_part || s->has_deck;
      cu_textb(found ? "Core \xe2\x9c\x93" : "Core: deep", 13, found ? cu_rgb(0x8dff9a) : cu_rgb(0xffd23d), 700, false, (HzUIColor){0});
    } else {
      cstr_format(d->txt[0], sizeof(d->txt[0]), "Depth % m", fmt_f32_p(metres, 1));
      cstr_format(d->txt[1], sizeof(d->txt[1]), "\xe2\x97\x86 Pearls %/%", fmt_u32(n), fmt_i32(DIVE_PEARLS));
      cu_textb(d->txt[0], 15, cu_rgb(0x9fe0ff), 700, false, (HzUIColor){0});
      cu_textb(d->txt[1], 15, cu_rgb(0xe8f6ff), 700, false, (HzUIColor){0});
      if (!s->has_part && !s->has_deck)
        cu_textb("Find the Stud Core (deep down)", 13, cu_rgb(0xffd23d), 700, false, (HzUIColor){0});
      else
        cu_textb("Stud Core \xe2\x9c\x93", 13, cu_rgb(0x8dff9a), 700, false, (HzUIColor){0});
    }
  }
  // the hint, for a while
  if (d->t < 9.0f) {
    f32 a = m_clampf(9.0f - d->t, 0.0f, 1.0f);
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 8,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0.0f, -safe.bottom - 26.0f}},
                .layout = {.padding = {.left = 16, .right = 16, .top = 8, .bottom = 8}, .sizing = {.width = ui_sizing_fit(0, sw - 32.0f)}},
                .background_color = cu_shade(0x041424, 0.6f * a),
                .corner_radius = ui_corner_radius(14),
                .opacity = opt_some(f32, a)}) {
      cu_textb(touch ? "Hold a finger to swim \xc2\xb7 move it to steer \xc2\xb7 pinch: back \xc2\xb7 spread: faster"
                     : "W / S swim \xc2\xb7 hold a mouse button and drag to look \xc2\xb7 Space up \xc2\xb7 Q down \xc2\xb7 Shift faster",
               14, cu_rgb(0xe8f6ff), 600, true, (HzUIColor){0});
    }
  }
  // the way out
  ui_element({.id = ui_id("DiveSurface"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 8,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                           .offset = {-safe.right - 14.0f, safe.top + 14.0f}},
              .layout = {.padding = {.left = 14, .right = 14, .top = 8, .bottom = 8}},
              .background_color = cu_rgba(0x1e6fe0, 0.9f),
              .corner_radius = ui_corner_radius(18),
              .brightness = ui_hovered() ? opt_some(f32, 1.15f) : opt_none(f32)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      d->click_surface = true;
    cu_textb("\xe2\x96\xb2 Surface", 14, cu_rgb(0xffffff), 700, false, (HzUIColor){0});
  }
  if (d->fade > 0.0f)
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 90},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                .background_color = cu_rgba(0x020a14, d->fade)}) {}
}
