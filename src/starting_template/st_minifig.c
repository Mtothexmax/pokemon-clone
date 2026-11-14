// LEGO minifigures: six static glb parts (tools/make_minifig.py) hung under a root at their joint pivots, posed on the
// main lane before the frame's scope (the swing writes LocalTransform; ecs_transform_schedule carries it to the parts).
// The player's root is the character itself (its WorldTransform carries the facing); villagers get a root of their own.
// Included by starting_template.c above app_init.

#define FIG_PART_COUNT 6
#define FIG_MAX 8
#define FIG_ARM_REST -0.22f // arms held a little forward, like a real minifig

typedef enum FigStyle {
  FIG_STYLE_PLAYER,
  FIG_STYLE_DEX,
  FIG_STYLE_PIP,
  FIG_STYLE_QUILL,
  FIG_STYLE_MOM,
} FigStyle;

typedef struct Minifig {
  EcsEntity root;
  EcsEntity part[FIG_PART_COUNT]; // head, torso, armL, armR, legL, legR
  b32 is_player;
  f32 yaw;      // a villager's facing (yaw 0 faces +z)
  f32 walk_t;   // walk cycle phase
  f32 swing;    // current swing amplitude, eases in and out
  f32 head_yaw; // a villager looks at the player when close
  f32 idle_t;
} Minifig;

typedef struct MinifigSet {
  Minifig figs[FIG_MAX];
  u32 count;
} MinifigSet;

// joint pivots in figure space, metres (feet at 0, facing +z): make_minifig.py's PIVOT * STUD_M
hz_internal v3 fig_pivot(u32 part) {
  switch (part) {
  case 0: return (v3){0.0f, 1.501f, 0.0f};
  case 1: return (v3){0.0f, 0.0f, 0.0f};
  case 2: return (v3){-0.3268f, 1.1856f, -0.038f};
  case 3: return (v3){0.3268f, 1.1856f, -0.038f};
  case 4: return (v3){-0.152f, 0.589f, 0.0f};
  default: return (v3){0.152f, 0.589f, 0.0f};
  }
}

#define FIG_PATH(style, part) "brickhollow/minifig/" style "/" part ".glb"
#define FIG_PATHS(style)                                                                                                   \
  FIG_PATH(style, "head"), FIG_PATH(style, "torso"), FIG_PATH(style, "armL"), FIG_PATH(style, "armR"),                     \
      FIG_PATH(style, "legL"), FIG_PATH(style, "legR")

hz_internal CString fig_part_path(FigStyle style, u32 part) {
  switch (style) {
  case FIG_STYLE_DEX: {
    CString p[] = {FIG_PATHS("dex")};
    return p[part];
  }
  case FIG_STYLE_PIP: {
    CString p[] = {FIG_PATHS("pip")};
    return p[part];
  }
  case FIG_STYLE_QUILL: {
    CString p[] = {FIG_PATHS("quill")};
    return p[part];
  }
  case FIG_STYLE_MOM: {
    CString p[] = {FIG_PATHS("mom")};
    return p[part];
  }
  default: {
    CString p[] = {FIG_PATHS("player")};
    return p[part];
  }
  }
}

// main lane: the six parts under `root`
hz_internal Minifig *fig_attach(MinifigSet *set, EcsWorld *w, EcsEntity root, FigStyle style, b32 is_player, f32 yaw) {
  assert(set->count < FIG_MAX);
  AssetSystem *assets = ecs_resource_get_mut(w, AssetSystem);
  Minifig *f = &set->figs[set->count++];
  *f = (Minifig){.root = root, .is_player = is_player, .yaw = yaw, .idle_t = (f32)set->count * 1.7f};
  for (u32 i = 0; i < FIG_PART_COUNT; i++) {
    HzHandleT(ModelBlobAsset) model = asset_load(ModelBlobAsset, assets, .path = fig_part_path(style, i));
    EcsEntity e = ecs_entity_create(w);
    ecs_component_set(w, e, EcsParent, {.parent = root});
    ecs_component_set(w, e, LocalTransform, {.position = fig_pivot(i), .rotation = quat_identity(), .scale = V3_ONE});
    ecs_component_set(w, e, EcsModelRef, {.model = blob_asset_ref_from_handle(ModelBlobAsset, model)});
    ecs_component_set(w, e, EcsVisibility, {.visible = true, .casts_shadow = true});
    f->part[i] = e;
  }
  return f;
}

// main lane: a villager standing at (x, y, z) metres, facing yaw
hz_internal void fig_spawn_villager(MinifigSet *set, EcsWorld *w, FigStyle style, v3 at, f32 yaw) {
  EcsEntity root = ecs_entity_create(w);
  quat q = quat_from_euler((v3){0.0f, yaw, 0.0f});
  ecs_component_set(w, root, LocalTransform, {.position = at, .rotation = q, .scale = V3_ONE});
  ecs_component_set(w, root, WorldTransform, {.position = at, .rotation = q, .scale = V3_ONE});
  fig_attach(set, w, root, style, false, yaw);
}

hz_internal f32 fig_wrap(f32 a) {
  while (a > M_PI)
    a -= 2.0f * M_PI;
  while (a < -M_PI)
    a += 2.0f * M_PI;
  return a;
}

hz_internal void fig_set_rot(EcsWorld *w, EcsEntity e, u32 part, v3 euler) {
  ecs_component_set(w, e, LocalTransform, {.position = fig_pivot(part), .rotation = quat_from_euler(euler), .scale = V3_ONE});
}

// main lane, before the scope: walk swing from the character's speed, arms up in a jump, villagers breathe and look
hz_internal void fig_animate(MinifigSet *set, EcsWorld *w, EcsEntity player, f32 dt) {
  const WorldTransform *pwt = ecs_component_get(w, player, WorldTransform);
  v3 ppos = pwt ? pwt->position : V3_ZERO;
  for (u32 i = 0; i < set->count; i++) {
    Minifig *f = &set->figs[i];
    f32 speed = 0.0f;
    b32 air = false;
    if (f->is_player) {
      const Character *c = ecs_component_get(w, f->root, Character);
      if (c) {
        speed = character_planar_speed(c);
        air = !character_grounded(c);
      }
    }
    f32 target = air ? 0.0f : m_minf(1.0f, speed / 3.0f) * 0.7f;
    f->swing += (target - f->swing) * m_minf(1.0f, dt * 10.0f);
    f->walk_t += dt * (speed * 2.4f);
    f->idle_t += dt;
    f32 s = m_sinf(f->walk_t) * f->swing;
    f32 breathe = m_sinf(f->idle_t * 1.6f) * 0.04f;
    // a villager turns the head to the player within 7 m
    if (!f->is_player) {
      const WorldTransform *wt = ecs_component_get(w, f->root, WorldTransform);
      f32 want = 0.0f;
      if (wt) {
        v3 d = v3_sub(ppos, wt->position);
        if (d.x * d.x + d.z * d.z < 49.0f)
          want = m_clampf(fig_wrap(m_atan2f(d.x, d.z) - f->yaw), -1.0f, 1.0f);
      }
      f->head_yaw += (want - f->head_yaw) * m_minf(1.0f, dt * 5.0f);
    }
    f32 arm_air = air ? -1.6f : 0.0f;
    fig_set_rot(w, f->part[0], 0, (v3){0.0f, f->head_yaw, 0.0f});
    fig_set_rot(w, f->part[2], 2, (v3){FIG_ARM_REST - s * 0.8f + arm_air + breathe, 0.0f, 0.0f});
    fig_set_rot(w, f->part[3], 3, (v3){FIG_ARM_REST + s * 0.8f + arm_air - breathe, 0.0f, 0.0f});
    fig_set_rot(w, f->part[4], 4, (v3){air ? -0.5f : s, 0.0f, 0.0f});
    fig_set_rot(w, f->part[5], 5, (v3){air ? 0.3f : -s, 0.0f, 0.0f});
  }
}
