// LEGO minifigures: six static glb parts (tools/make_minifig.py) hung under a root at their joint pivots, posed on the
// main lane before the frame's scope (the swing writes LocalTransform; ecs_transform_schedule carries it to the parts).
// The player's root is the character itself (its WorldTransform carries the facing); villagers get a root of their own.
// Included by starting_template.c above app_init.

#define FIG_PART_COUNT 6
#define FIG_MAX 10
#define FIG_ARM_REST -0.22f // arms held a little forward, like a real minifig

typedef enum FigStyle {
  FIG_STYLE_PLAYER,
  FIG_STYLE_DEX,
  FIG_STYLE_PIP,
  FIG_STYLE_QUILL,
  FIG_STYLE_MOM,
  FIG_STYLE_LUMI,
  FIG_STYLE_KNIGHT, // Stonecrest: Sir Bricksworth
  FIG_STYLE_BAKER,  // Stonecrest: Marla
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
  f32 speed;    // a villager walking somewhere (m/s); the player's comes from its character
  u32 pose;     // FigPose
  f32 spin;     // extra turn of the whole figure (dancing)
  v3 offset;    // the whole figure moved (sitting, hopping, crouching), figure space
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
  case FIG_STYLE_LUMI: {
    CString p[] = {FIG_PATHS("lumi")};
    return p[part];
  }
  case FIG_STYLE_MOM: {
    CString p[] = {FIG_PATHS("mom")};
    return p[part];
  }
  case FIG_STYLE_KNIGHT: {
    CString p[] = {FIG_PATHS("knight")};
    return p[part];
  }
  case FIG_STYLE_BAKER: {
    CString p[] = {FIG_PATHS("baker")};
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
hz_internal Minifig *fig_spawn_villager(MinifigSet *set, EcsWorld *w, FigStyle style, v3 at, f32 yaw) {
  EcsEntity root = ecs_entity_create(w);
  quat q = quat_from_euler((v3){0.0f, yaw, 0.0f});
  ecs_component_set(w, root, LocalTransform, {.position = at, .rotation = q, .scale = V3_ONE});
  ecs_component_set(w, root, WorldTransform, {.position = at, .rotation = q, .scale = V3_ONE});
  return fig_attach(set, w, root, style, false, yaw);
}

typedef enum FigPose {
  FIG_POSE_NORMAL, // walk / stand
  FIG_POSE_SIT,    // on a bench: legs forward, hands on the knees, feet swinging
  FIG_POSE_DANCE,  // arms up, hopping (spin is set by the caller)
  FIG_POSE_ARMS_UP,
  FIG_POSE_CROUCH, // hiding
  FIG_POSE_WAVE,   // right arm waving
} FigPose;

hz_internal f32 fig_wrap(f32 a) {
  while (a > M_PI)
    a -= 2.0f * M_PI;
  while (a < -M_PI)
    a += 2.0f * M_PI;
  return a;
}

// one part: its pivot turned by the figure's spin and moved by its offset, then its own rotation
hz_internal void fig_set_part(EcsWorld *w, const Minifig *f, u32 part, v3 euler) {
  quat spin = quat_from_euler((v3){0.0f, f->spin, 0.0f});
  v3 pos = v3_add(quat_rotate(spin, fig_pivot(part)), f->offset);
  quat rot = quat_mul(spin, quat_from_euler(euler));
  ecs_component_set(w, f->part[part], LocalTransform, {.position = pos, .rotation = rot, .scale = V3_ONE});
}

// main lane, before the scope: walk swing from the speed (the player's character, or a villager's `speed`), the poses
hz_internal void fig_animate(MinifigSet *set, EcsWorld *w, EcsEntity player, f32 dt) {
  const WorldTransform *pwt = ecs_component_get(w, player, WorldTransform);
  v3 ppos = pwt ? pwt->position : V3_ZERO;
  for (u32 i = 0; i < set->count; i++) {
    Minifig *f = &set->figs[i];
    f32 speed = f->speed;
    b32 air = false;
    if (f->is_player) {
      const Character *c = ecs_component_get(w, f->root, Character);
      if (c) {
        speed = character_planar_speed(c);
        air = !character_grounded(c);
      }
    }
    f32 target = air || f->pose == FIG_POSE_SIT ? 0.0f : m_minf(1.0f, speed / 3.0f) * 0.6f;
    f->swing += (target - f->swing) * m_minf(1.0f, dt * 10.0f);
    f->walk_t += dt * (speed * 2.4f);
    f->idle_t += dt;
    f32 s = m_sinf(f->walk_t) * f->swing;
    f32 breathe = m_sinf(f->idle_t * 1.6f) * 0.04f;
    // a villager turns the head to the player within 7 m (not while walking somewhere)
    if (!f->is_player) {
      const WorldTransform *wt = ecs_component_get(w, f->root, WorldTransform);
      f32 want = 0.0f;
      if (wt && speed < 0.3f) {
        v3 d = v3_sub(ppos, wt->position);
        if (d.x * d.x + d.z * d.z < 49.0f)
          want = m_clampf(fig_wrap(m_atan2f(d.x, d.z) - f->yaw), -1.0f, 1.0f);
      }
      f->head_yaw += (want - f->head_yaw) * m_minf(1.0f, dt * 5.0f);
    }
    v3 arm_l = {FIG_ARM_REST - s * 0.8f + breathe, 0.0f, 0.0f}, arm_r = {FIG_ARM_REST + s * 0.8f - breathe, 0.0f, 0.0f};
    v3 leg_l = {s, 0.0f, 0.0f}, leg_r = {-s, 0.0f, 0.0f};
    f->offset = V3_ZERO;
    if (air) {
      arm_l.x = arm_r.x = FIG_ARM_REST - 1.6f;
      leg_l.x = -0.5f;
      leg_r.x = 0.3f;
    }
    switch (f->pose) {
    case FIG_POSE_SIT: {
      f32 swing = m_sinf(f->idle_t * 2.0f) * 0.12f;
      leg_l.x = -1.5f + swing;
      leg_r.x = -1.5f - swing;
      arm_l.x = arm_r.x = -0.75f;
      f->offset = (v3){0.0f, -0.08f, -0.42f}; // onto the seat behind the feet
    } break;
    case FIG_POSE_DANCE:
    case FIG_POSE_ARMS_UP: {
      f32 wave = m_sinf(f->idle_t * 8.0f) * 0.3f;
      arm_l.x = -2.6f + wave;
      arm_r.x = -2.6f - wave;
      f->offset.y = m_absf(m_sinf(f->idle_t * 6.0f)) * (f->pose == FIG_POSE_DANCE ? 0.13f : 0.11f);
    } break;
    case FIG_POSE_CROUCH:
      f->offset.y = -0.19f;
      leg_l.x = leg_r.x = -0.6f;
      break;
    case FIG_POSE_WAVE: arm_r.x = -2.6f + m_sinf(f->idle_t * 8.0f) * 0.3f; break;
    default: break;
    }
    fig_set_part(w, f, 0, (v3){0.0f, f->head_yaw, 0.0f});
    fig_set_part(w, f, 1, V3_ZERO);
    fig_set_part(w, f, 2, arm_l);
    fig_set_part(w, f, 3, arm_r);
    fig_set_part(w, f, 4, leg_l);
    fig_set_part(w, f, 5, leg_r);
  }
}

// a villager's root to (x, z) metres, facing yaw (the walk swing follows `speed`)
hz_internal void fig_place(EcsWorld *w, Minifig *f, v3 at, f32 yaw) {
  f->yaw = yaw;
  quat q = quat_from_euler((v3){0.0f, yaw, 0.0f});
  ecs_component_set(w, f->root, LocalTransform, {.position = at, .rotation = q, .scale = V3_ONE});
  ecs_component_set(w, f->root, WorldTransform, {.position = at, .rotation = q, .scale = V3_ONE}); // a root: no parent carries it
}
