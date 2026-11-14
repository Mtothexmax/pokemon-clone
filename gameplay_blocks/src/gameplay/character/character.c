#include "character.h"
#include "ecs/ecs_assets/ecs_assets.h"
#include "ecs/ecs_rigidbody/ecs_rigidbody.h"
#include "lib/assert.h"
#include "lib/profiler/profiler.h"
#include "lib/radix_sort.h"

#define CHAR_SKIN 0.02f // CharacterVirtual::mCharacterPadding: the gap kept to every surface, so sweeps never start in contact
#define CHAR_PROBE 0.06f // contacts within this gap are touching: the resting floor is support without a sweep
#define CHAR_ONE_WAY_TOL 0.05f
#define CHAR_INTO_EPS 1e-3f
#define CHAR_LAND_MIN_SPEED 1.0f
#define CHAR_SEPARATION_SPEED 0.5f // m/s along a floor's normal that leaves it (a jump); slower is moving along it
#define CHAR_LANDED_HOLD_FRAMES 3
#define CHAR_DEFAULT_MODEL "gameplay_blocks/character/default_character.glb"
#define CHAR_DEFAULT_IDLE "gameplay_blocks/character/default_character/Idle_Loop.hza"
#define CHAR_DEFAULT_RUN "gameplay_blocks/character/default_character/Sprint_Loop.hza"
#define CHAR_DEFAULT_FALL "gameplay_blocks/character/default_character/Jump_Loop.hza"

// runtime animation driver state, attached by character_avatar_add
typedef struct CharacterAvatar {
  EcsEntity avatar;
  HzHandleT(AnimationBlobAsset) idle;
  HzHandleT(AnimationBlobAsset) run;
  HzHandleT(AnimationBlobAsset) fall;
  f32 run_lo;
  f32 run_hi;
  b32 run_dominant;
  b32 was_airborne;
  u32 landed_hold;
} CharacterAvatar;
ECS_COMPONENT_TYPEDEF(CharacterAvatar);

#define CHAR_DEFAULT(field, value) \
  if ((field) <= 0.0f)             \
  (field) = (value)

CharacterTuning character_tuning_resolve(CharacterTuning t) {
  CHAR_DEFAULT(t.radius, 0.35f);
  CHAR_DEFAULT(t.height, 1.8f);
  CHAR_DEFAULT(t.eye_height, 1.65f);
  CHAR_DEFAULT(t.walk_speed, 6.0f);
  CHAR_DEFAULT(t.run_speed, t.walk_speed);
  CHAR_DEFAULT(t.accel_time, 0.1f);
  CHAR_DEFAULT(t.decel_time, 0.1f);
  CHAR_DEFAULT(t.air_accel_time, 0.4f);
  CHAR_DEFAULT(t.air_decel_time, 0.4f);
  CHAR_DEFAULT(t.ground_accel, 11.0f);
  CHAR_DEFAULT(t.air_accel, 2.2f);
  CHAR_DEFAULT(t.friction, 7.5f);
  CHAR_DEFAULT(t.stop_speed, 1.5f);
  CHAR_DEFAULT(t.move_damping, 14.0f);
  CHAR_DEFAULT(t.air_control, 0.35f);
  CHAR_DEFAULT(t.gravity, 36.0f);
  CHAR_DEFAULT(t.max_fall_speed, 55.0f);
  CHAR_DEFAULT(t.jump_height, 1.8f);
  CHAR_DEFAULT(t.coyote_time, 0.1f);
  CHAR_DEFAULT(t.jump_buffer, 0.15f);
  CHAR_DEFAULT(t.slope_limit_deg, 50.0f);
  CHAR_DEFAULT(t.step_height, 0.35f);
  CHAR_DEFAULT(t.snap_distance, 0.35f);
  CHAR_DEFAULT(t.turn_rate, 12.0f);
  CHAR_DEFAULT(t.footstep_walk, 2.1f);
  CHAR_DEFAULT(t.footstep_run, 2.6f);
  CHAR_DEFAULT(t.swim_depth, 1.25f);
  CHAR_DEFAULT(t.swim_speed_scale, 0.6f);
  CHAR_DEFAULT(t.buoyancy, 6.0f);
  if (t.crouch_height > 0.0f) {
    t.crouch_height = m_clampf(t.crouch_height, 2.0f * t.radius, t.height);
    CHAR_DEFAULT(t.crouch_eye_height, m_maxf(t.eye_height - (t.height - t.crouch_height), t.radius));
    CHAR_DEFAULT(t.crouch_speed, t.walk_speed * 0.5f);
    CHAR_DEFAULT(t.crouch_ease_s, 0.18f);
  }
  return t;
}

f32 character_stand_height(const CharacterTuning *tuning) { return character_tuning_resolve(*tuning).height; }

f32 character_eye_height(const CharacterTuning *t, const Character *c) {
  if (!c || t->crouch_height <= 0.0f)
    return t->eye_height;
  f32 k = c->crouch_k * c->crouch_k * (3.0f - 2.0f * c->crouch_k);
  return m_lerpf(t->eye_height, t->crouch_eye_height, k);
}

f32 character_planar_speed(const Character *c) { return m_sqrtf(c->velocity.x * c->velocity.x + c->velocity.z * c->velocity.z); }

// the static and kinematic layers, the bodies a character stands on and is blocked by
hz_internal EcsPhysicsQuery char_query(EcsWorld *w) {
  const EcsCollisionLayer *s = ecs_physics_get_static_layer(w);
  const EcsCollisionLayer *k = ecs_physics_get_kinematic_layer(w);
  u32 layers = (s ? s->mask : 0u) | (k ? k->mask : 0u);
  // no layer at all: a mask no body carries, so nothing is hit
  return (EcsPhysicsQuery){.layers = layers ? layers : 1u << 31, .filter = PHYSICS_QUERY_FILTER_ALL};
}

f32 character_spawn_height(EcsWorld *w, f32 x, f32 z, f32 from_y, f32 max_drop, f32 fallback) {
  const EcsPhysicsWorld *pw = ecs_resource_get(w, EcsPhysicsWorld);
  if (!pw)
    return fallback;
  EcsPhysicsQuery q = char_query(w);
  EcsPhysicsHit hit;
  if (ecs_physics_raycast(pw, (v3){x, from_y, z}, (v3){x, from_y - max_drop, z}, &q, &hit))
    return hit.hit.point.y;
  return fallback;
}

// ---- queries: the capsule against the static and kinematic layers, one body at a time so one-way platforms can opt out ----

typedef struct CharEnv {
  EcsWorld *w;
  const EcsPhysicsWorld *pw;
  EcsPhysicsQuery q;
  Collider capsule; // feet space
  f32 cos_slope;
  f32 ref_feet_y; // feet height at the start of the step: decides one-way platforms
} CharEnv;

typedef struct CharProbe {
  b32 hit;
  b32 walkable;
  f32 dist;
  v3 normal;
  EcsEntity entity;
} CharProbe;

hz_internal CharEnv char_env_make(EcsWorld *w, const CharacterTuning *t, f32 ref_feet_y) {
  CharEnv e = {.w = w, .pw = ecs_resource_get(w, EcsPhysicsWorld), .q = char_query(w), .ref_feet_y = ref_feet_y};
  e.capsule = collider_capsule((v3){0.0f, t->radius, 0.0f}, (v3){0.0f, t->height - t->radius, 0.0f}, t->radius);
  e.cos_slope = m_cosf(m_radians(t->slope_limit_deg));
  return e;
}

// walls and floors always block, pass-through bodies never; a one-way platform only once the feet started above its top
hz_internal b32 char_blocks(const CharEnv *e, const PhysicsWorld *world, PhysicsShapeRef shape, EcsEntity ent) {
  if (ecs_component_get(e->w, ent, CharacterPassThrough))
    return false;
  if (!ecs_component_get(e->w, ent, CharacterOneWay))
    return true;
  aabb box;
  return physics_shape_get_aabb(world, shape, &box) && e->ref_feet_y >= box.max.y - CHAR_ONE_WAY_TOL;
}

hz_internal aabb char_aabb(const CharEnv *e, v3 a, v3 b) {
  aabb ba = physics_aabb_from(e->capsule, a, quat_identity(), 1.0f);
  aabb bb = physics_aabb_from(e->capsule, b, quat_identity(), 1.0f);
  const f32 pad = CHAR_PROBE + 0.02f;
  return (aabb){{m_minf(ba.min.x, bb.min.x) - pad, m_minf(ba.min.y, bb.min.y) - pad, m_minf(ba.min.z, bb.min.z) - pad},
                {m_maxf(ba.max.x, bb.max.x) + pad, m_maxf(ba.max.y, bb.max.y) + pad, m_maxf(ba.max.z, bb.max.z) + pad}};
}

// the signed rounded-core distance to one shape within max; false past it or with no separating direction
hz_internal b32 char_distance(const CharEnv *e, const PhysicsWorld *world, PhysicsShapeRef shape, v3 pos, f32 max,
                              PhysicsShapeDistance *r) {
  if (!physics_shape_distance(world, shape, &e->capsule, pos, quat_identity(), max, r))
    return false;
  // cores overlapping: Box3D's distance has no plane to push along, and its mover skips them too
  return v3_lengthsq(r->normal) > 0.0f;
}

// a cast's first obstacle: where it was hit, its surface normal and how far the cast travelled
typedef struct {
  v3 position;
  v3 normal_on_target;
  f32 distance;
} CharCastHit;

typedef struct CharCastCtx {
  const CharEnv *env;
  v3 from;
  v3 delta;
  v3 dir;
  f32 len;
  b32 hit;
  CharCastHit best;
  EcsEntity entity;
} CharCastCtx;

hz_internal b32 char_cast_cb(EcsEntity ent, const PhysicsWorld *world, PhysicsShapeRef shape, void *user) {
  CharCastCtx *c = user;
  PhysicsCastHit h;
  if (!physics_shape_cast(world, shape, &c->env->capsule, c->from, quat_identity(), c->delta, &h))
    return true;
  f32 distance = h.fraction * c->len;
  // a surface the character is not moving into is no obstacle: tangent contacts must not stall the slide
  if (v3_dot(h.normal, c->dir) > -CHAR_INTO_EPS || (c->hit && distance >= c->best.distance))
    return true;
  if (!char_blocks(c->env, world, shape, ent))
    return true;
  c->best = (CharCastHit){.position = h.point, .normal_on_target = h.normal, .distance = distance};
  c->entity = ent;
  c->hit = true;
  return true;
}

// closest obstacle along from -> to
hz_internal b32 char_cast(const CharEnv *e, v3 from, v3 to, CharCastHit *out, EcsEntity *ent) {
  v3 d = v3_sub(to, from);
  f32 len = v3_length(d);
  if (len < 1e-6f || !e->pw)
    return false;
  CharCastCtx c = {.env = e, .from = from, .delta = d, .dir = v3_scale(d, 1.0f / len), .len = len};
  ecs_physics_find(e->pw, char_aabb(e, from, to), &e->q, char_cast_cb, &c);
  if (c.hit) {
    *out = c.best;
    *ent = c.entity;
  }
  return c.hit;
}

hz_internal CharProbe char_probe(const CharEnv *e, v3 pos, f32 reach) {
  CharProbe p = {0};
  if (reach <= 0.0f)
    return p;
  CharCastHit r = {0};
  EcsEntity ent = 0;
  if (!char_cast(e, pos, v3_add(pos, (v3){0.0f, -reach, 0.0f}), &r, &ent))
    return p;
  p.hit = true;
  p.dist = r.distance;
  p.normal = r.normal_on_target;
  p.entity = ent;
  p.walkable = r.normal_on_target.y >= e->cos_slope;
  return p;
}

typedef struct CharGroundCtx {
  const CharEnv *env;
  v3 pos;
  CharProbe walk;  // the flattest walkable contact
  CharProbe steep; // the flattest too-steep upward contact
} CharGroundCtx;

hz_internal b32 char_ground_cb(EcsEntity ent, const PhysicsWorld *world, PhysicsShapeRef shape, void *user) {
  CharGroundCtx *c = user;
  PhysicsShapeDistance r;
  if (!char_distance(c->env, world, shape, c->pos, CHAR_PROBE, &r) || r.distance > CHAR_PROBE)
    return true;
  v3 n = r.normal;
  if (n.y <= 1e-3f)
    return true;
  if (!char_blocks(c->env, world, shape, ent))
    return true;
  b32 walkable = n.y >= c->env->cos_slope;
  CharProbe *slot = walkable ? &c->walk : &c->steep;
  if (slot->hit && n.y <= slot->normal.y)
    return true;
  *slot = (CharProbe){.hit = true, .walkable = walkable, .dist = r.distance, .normal = n, .entity = ent};
  return true;
}

// CharacterVirtual::GetContactsAtPosition: the surfaces touching the capsule from below; walkable wins over steep
hz_internal CharProbe char_ground_contact(const CharEnv *e, v3 pos) {
  CharGroundCtx c = {.env = e, .pos = pos};
  if (e->pw)
    ecs_physics_find(e->pw, char_aabb(e, pos, pos), &e->q, char_ground_cb, &c);
  return c.walk.hit ? c.walk : c.steep;
}

// CharacterVirtual::WalkStairs: up, forward, down; true when it lands on walkable ground at a new height
hz_internal b32 char_step_up(const CharEnv *e, const CharacterTuning *t, v3 start, v3 horiz, v3 *out_pos) {
  CharCastHit r = {0};
  EcsEntity ent = 0;
  f32 up = t->step_height;
  if (char_cast(e, start, v3_add(start, (v3){0.0f, up, 0.0f}), &r, &ent))
    up = m_maxf(r.distance - CHAR_SKIN, 0.0f);
  if (up < 0.02f)
    return false;
  v3 p1 = v3_add(start, (v3){0.0f, up, 0.0f});
  v3 p2 = v3_add(p1, horiz);
  if (char_cast(e, p1, p2, &r, &ent))
    p2 = v3_add(p1, v3_scale(v3_normalize(horiz), m_maxf(r.distance - CHAR_SKIN, 0.0f)));
  if (v3_distance(p1, p2) < 0.01f)
    return false;
  CharProbe g = char_probe(e, p2, up + CHAR_SKIN * 2.0f);
  if (!g.hit || !g.walkable)
    return false;
  v3 land = v3_sub(p2, (v3){0.0f, g.dist, 0.0f});
  if (land.y <= start.y + 0.01f || land.y > start.y + t->step_height + CHAR_SKIN)
    return false;
  *out_pos = land;
  return true;
}

// ---- steering: input to velocity, shared by both movers ----

hz_internal f32 char_yaw_of(quat q) {
  v3 f = quat_rotate(q, V3_FORWARD);
  return m_atan2f(f.x, f.z);
}

hz_internal void char_place(Character *c, v3 pos) {
  c->position = pos;
  c->prev_position = pos;
  c->prev_yaw = c->yaw;
  c->velocity = V3_ZERO;
  c->ground = CHARACTER_GROUND_IN_AIR;
  c->ground_entity = ECS_ENTITY_INVALID;
  c->ground_body = (PhysicsBody){0};
  c->ground_velocity = V3_ZERO;
  c->ground_body_valid = false;
  c->coyote = 0.0f;
  c->jump_buffer = 0.0f;
  c->swimming = false;
  c->footstep_dist = 0.0f;
}

// jump pads and knockback: velocity is added; returns true for an upward launch, which also clears the grace timers
hz_internal b32 char_launch(Character *c, CharacterInput *in, v3 *vel) {
  if (v3_lengthsq(in->add_velocity) <= 0.0f)
    return false;
  b32 up = in->add_velocity.y > 0.0f;
  *vel = v3_add(*vel, in->add_velocity);
  if (up) {
    c->coyote = 0.0f;
    c->jump_buffer = 0.0f;
  }
  in->add_velocity = V3_ZERO;
  c->launches++;
  return up;
}

// the horizontal velocity, jump request and facing; vel is relative to the ground. Returns true when a jump fired
hz_internal b32 char_steer(const CharacterTuning *t, Character *c, CharacterInput *in, f32 dt, b32 supported, b32 swimming,
                           f32 gravity, v3 *vel) {
  if (in->jump) {
    c->jump_buffer = t->jump_buffer;
    in->jump = false;
  } else {
    c->jump_buffer -= dt;
  }
  c->coyote = supported ? t->coyote_time : c->coyote - dt;

  v2 mv = in->move;
  f32 mag = m_sqrtf(mv.x * mv.x + mv.y * mv.y);
  if (mag > 1.0f) {
    mv = (v2){mv.x / mag, mv.y / mag};
    mag = 1.0f;
  }
  f32 sn = m_sinf(in->heading), cs = m_cosf(in->heading);
  v3 wish = {cs * mv.x - sn * mv.y, 0.0f, -sn * mv.x - cs * mv.y};
  f32 wl = v3_length(wish);
  v3 wdir = wl > 1e-4f ? v3_scale(wish, 1.0f / wl) : V3_ZERO;
  b32 moving = mag > 0.01f;
  c->running = in->run && moving;
  f32 speed = c->running ? t->run_speed : t->walk_speed;
  if (swimming)
    speed *= t->swim_speed_scale;
  f32 wish_speed = speed * mag;
  b32 on_floor = supported || swimming;

  v3 hv = {vel->x, 0.0f, vel->z};
  if (t->move_model == CHARACTER_MOVE_DAMPED) {
    f32 k = 1.0f - m_expf(-t->move_damping * (on_floor ? 1.0f : t->air_control) * dt);
    hv = v3_add(hv, v3_scale(v3_sub(v3_scale(wdir, wish_speed), hv), k));
  } else if (t->move_model == CHARACTER_MOVE_QUAKE) {
    if (on_floor) {
      f32 s = v3_length(hv);
      if (s < 1e-4f) {
        hv = V3_ZERO;
      } else {
        f32 drop = m_maxf(s, t->stop_speed) * t->friction * dt;
        hv = v3_scale(hv, m_maxf(s - drop, 0.0f) / s);
      }
    }
    if (moving) {
      f32 add = wish_speed - v3_dot(hv, wdir);
      if (add > 0.0f)
        hv = v3_add(hv, v3_scale(wdir, m_minf((on_floor ? t->ground_accel : t->air_accel) * wish_speed * dt, add)));
    }
  } else {
    f32 secs = on_floor ? (moving ? t->accel_time : t->decel_time) : (moving ? t->air_accel_time : t->air_decel_time);
    f32 max_change = m_maxf(t->walk_speed, t->run_speed) / m_maxf(secs, 1e-3f) * dt;
    v3 dv = v3_sub(v3_scale(wdir, wish_speed), hv);
    f32 dl = v3_length(dv);
    hv = dl <= max_change ? v3_scale(wdir, wish_speed) : v3_add(hv, v3_scale(dv, max_change / dl));
  }
  vel->x = hv.x;
  vel->z = hv.z;

  b32 jumped = false;
  if (c->jump_buffer > 0.0f && c->coyote > 0.0f && !swimming && c->ground != CHARACTER_GROUND_STEEP) {
    vel->y = m_sqrtf(2.0f * gravity * t->jump_height);
    c->jump_buffer = 0.0f;
    c->coyote = 0.0f;
    c->jumps++;
    jumped = true;
  }

  f32 target_yaw = c->yaw;
  b32 turn = false;
  switch (t->facing) {
  case CHARACTER_FACING_HEADING:
    target_yaw = in->heading + M_PI;
    turn = true;
    break;
  case CHARACTER_FACING_INPUT:
    target_yaw = in->face_yaw;
    turn = true;
    break;
  case CHARACTER_FACING_MOVEMENT:
  default:
    if (mag > 0.1f) {
      target_yaw = m_atan2f(wdir.x, wdir.z);
      turn = true;
    }
    break;
  }
  if (turn)
    c->yaw = m_wrap_pi(m_damp_angle(c->yaw, target_yaw, t->turn_rate, dt));
  return jumped;
}

hz_internal void char_footsteps(const CharacterTuning *t, Character *c, b32 supported, f32 dt) {
  if (!supported) {
    return;
  }
  c->footstep_dist += character_planar_speed(c) * dt;
  f32 stride = c->running ? t->footstep_run : t->footstep_walk;
  if (c->footstep_dist > stride) {
    c->footstep_dist = 0.0f;
    c->footsteps++;
    c->footstep_running = c->running;
  }
}

// carries the character with the platform it stood on: translation, rotation about the platform, conveyor velocity
hz_internal v3 char_platform_carry(EcsWorld *w, const CharacterTuning *t, Character *c, f32 dt) {
  v3 carry = V3_ZERO, surface = V3_ZERO;
  if (c->ground == CHARACTER_GROUND_ON_GROUND && c->ground_entity && !t->disable_riding) {
    const ColliderBody *gb = ecs_component_get(w, c->ground_entity, ColliderBody);
    if (gb) {
      b32 moved = !v3_eqv(gb->pos, c->ground_body_pos) || gb->rot.x != c->ground_body_rot.x ||
                  gb->rot.y != c->ground_body_rot.y || gb->rot.z != c->ground_body_rot.z || gb->rot.w != c->ground_body_rot.w;
      if (c->ground_body_valid && moved) {
        quat dq = quat_mul(gb->rot, quat_conjugate(c->ground_body_rot));
        v3 rel = v3_sub(c->position, c->ground_body_pos);
        carry = v3_sub(v3_add(gb->pos, quat_rotate(dq, rel)), c->position);
        v3 f = quat_rotate(dq, V3_FORWARD);
        c->yaw = m_wrap_pi(c->yaw + m_atan2f(f.x, f.z));
      }
      c->ground_body_pos = gb->pos;
      c->ground_body_rot = gb->rot;
      c->ground_body_valid = true;
      const CharacterSurface *sf = ecs_component_get(w, c->ground_entity, CharacterSurface);
      if (sf)
        surface = quat_rotate(gb->rot, sf->local_velocity);
    }
  }
  v3 move = v3_add(carry, v3_scale(surface, dt));
  c->ground_velocity = v3_scale(move, 1.0f / dt);
  return move;
}

hz_internal void char_record_ground_body(EcsWorld *w, Character *c, EcsEntity prev_entity) {
  if (c->ground != CHARACTER_GROUND_ON_GROUND || !c->ground_entity) {
    c->ground_body_valid = false;
    return;
  }
  if (c->ground_entity == prev_entity && c->ground_body_valid)
    return;
  const ColliderBody *gb = ecs_component_get(w, c->ground_entity, ColliderBody);
  c->ground_body_valid = gb != NULL;
  if (gb) {
    c->ground_body_pos = gb->pos;
    c->ground_body_rot = gb->rot;
  }
}

// ---- kinematic mover ----

// an outside write to the root WorldTransform is a teleport, taken by whichever of fixed step or frame pose sees it first
hz_internal void char_kinematic_take_outside_write(Character *c, const WorldTransform *wt) {
  if (v3_eqv(wt->position, c->render_position))
    return;
  char_place(c, wt->position);
  c->render_position = wt->position;
  c->teleports++;
}

// crouch: the capsule drops to crouch_height while the input holds it or the standing capsule has no room; no running
// crouched. Writes the step's height and speeds into the resolved copy `t`.
hz_internal void char_crouch(EcsWorld *w, CharacterTuning *t, Character *c, CharacterInput *in, f32 dt) {
  if (t->crouch_height <= 0.0f || t->crouch_height >= t->height) {
    c->crouched = false;
    c->crouch_k = 0.0f;
    return;
  }
  b32 want = in->crouch;
  if (!want && c->crouched) {
    CharacterTuning low = *t;
    low.height = t->crouch_height;
    CharEnv env = char_env_make(w, &low, c->position.y);
    CharCastHit r = {0};
    EcsEntity ent = 0;
    v3 up = {0.0f, t->height - t->crouch_height + CHAR_SKIN, 0.0f};
    if (char_cast(&env, c->position, v3_add(c->position, up), &r, &ent))
      want = true;
  }
  c->crouched = want;
  c->crouch_k = m_saturatef(c->crouch_k + (want ? dt : -dt) / t->crouch_ease_s);
  if (want) {
    t->height = t->crouch_height;
    t->walk_speed = t->crouch_speed;
    t->run_speed = t->crouch_speed;
    in->run = false;
  }
}

// ---- kinematic mover on physics: Box3D's mover primitives (mover.c, physics_world.c:2777, :3216) ----

#define CHAR_V2_PLANES 32      // mover planes gathered per collide, the probe band's included
#define CHAR_V2_ITERS 5        // Box3D CharacterMover::SolveMove passes
#define CHAR_V2_MIN_MOVE 1e-4f // squared metres: a pass moving less ends the solve (Box3D's 0.01 m)

typedef struct CharV2Env {
  const CharacterWorld *cw;
  PhysicsCapsule capsule; // feet space
  f32 cos_slope;
  f32 ref_feet_y; // feet height at the start of the step: decides one-way platforms
} CharV2Env;

typedef struct CharV2Contact {
  b32 touched;
  b32 wall;
  b32 ground;
  f32 ground_ny;
  v3 ground_normal;
} CharV2Contact;

typedef struct CharV2Probe {
  b32 hit;
  b32 walkable;
  f32 dist; // gap from the capsule to the surface; negative inside it
  v3 normal;
  PhysicsBody body;
} CharV2Probe;

typedef struct CharV2Scratch {
  PhysicsMoverPlane *found;
  PhysicsCollisionPlane *planes; // two per found plane: a steep one adds its horizontal
  CharacterV2Pushes *pushes;       // NULL: the caller does not push
  u32 found_count;                 // the move's last pass: planes within radius + CHAR_PROBE ...
  v3 found_shift;                  // ... gathered this far behind where the move ended
  b32 net;                         // character_net_step: the ground body's pose is read on the wire's grids
} CharV2Scratch;

hz_internal CharV2Env char_v2_env_make(const CharacterWorld *cw, const CharacterTuning *t, f32 ref_feet_y) {
  return (CharV2Env){.cw = cw,
                     .capsule = {{0.0f, t->radius, 0.0f}, {0.0f, t->height - t->radius, 0.0f}, t->radius},
                     .cos_slope = m_cosf(m_radians(t->slope_limit_deg)),
                     .ref_feet_y = ref_feet_y};
}

// swim_enabled and deep enough: the world's water at the feet, else water_height everywhere
hz_internal b32 char_swimming(const CharacterTuning *t, CharacterWaterFn *water, void *ctx, v3 feet, f32 *surface) {
  if (!t->swim_enabled)
    return false;
  *surface = t->water_height;
  if (water && !water(ctx, feet, surface))
    return false;
  return feet.y <= *surface - t->swim_depth + 0.1f;
}

hz_internal f32 char_net_snap(f32 x, f32 grid) { return net_dq_f32(net_q_f32(x, grid), grid); }

hz_internal v3 char_net_snap3(v3 v, f32 grid) {
  return (v3){char_net_snap(v.x, grid), char_net_snap(v.y, grid), char_net_snap(v.z, grid)};
}

hz_internal quat char_net_snapq(quat q) {
  return (quat){char_net_snap(q.x, CHARACTER_NET_ANGLE_GRID), char_net_snap(q.y, CHARACTER_NET_ANGLE_GRID),
                char_net_snap(q.z, CHARACTER_NET_ANGLE_GRID), char_net_snap(q.w, CHARACTER_NET_ANGLE_GRID)};
}

// a body's pose; networked, on the wire's grids so the stored pose and the live one compare exactly
hz_internal b32 char_v2_body_pose(const CharacterWorld *cw, PhysicsBody body, b32 net, v3 *p, quat *q) {
  if (!physics_body_get_transform(cw->world, body, p, q))
    return false;
  if (net) {
    *p = char_net_snap3(*p, CHARACTER_NET_GRID);
    *q = char_net_snapq(*q);
  }
  return true;
}

// every body; a one-way platform only once the feet started above its top
hz_internal b32 char_v2_blocks(PhysicsShapeRef shape, void *ctx) {
  const CharV2Env *e = (const CharV2Env *)ctx;
  const CharacterWorld *cw = e->cw;
  if (cw->pass_through && cw->pass_through(shape.body, cw->ctx)) return false;
  if (!cw->one_way || !cw->one_way(shape.body, cw->ctx)) return true;
  aabb box;
  return physics_shape_get_aabb(cw->world, shape, &box) && e->ref_feet_y >= box.max.y - CHAR_ONE_WAY_TOL;
}

// the planes of the surfaces within the capsule's radius grown by `grow`
hz_internal u32 char_v2_collide(const CharV2Env *e, v3 pos, f32 grow, PhysicsMoverPlane *out) {
  PhysicsCapsule m = e->capsule;
  m.radius += grow;
  return physics_world_collide_mover(e->cw->world, pos, &m, e->cw->filter, char_v2_blocks, (void *)e, out,
                                        CHAR_V2_PLANES);
}

// the fraction of delta free of obstacles
hz_internal f32 char_v2_cast(const CharV2Env *e, v3 pos, v3 delta) {
  if (v3_lengthsq(delta) < 1e-12f) return 1.0f;
  return physics_world_cast_mover(e->cw->world, pos, &e->capsule, delta, e->cw->filter, char_v2_blocks, (void *)e);
}

hz_internal void char_v2_note(const CharV2Env *e, CharV2Contact *ct, v3 n) {
  ct->touched = true;
  if (n.y >= e->cos_slope) {
    if (n.y > ct->ground_ny) ct->ground = true, ct->ground_ny = n.y, ct->ground_normal = n;
  } else {
    ct->wall = true;
  }
}

// the surfaces under the capsule within CHAR_PROBE, walkable over steep; planes gathered `shift` behind the capsule
hz_internal CharV2Probe char_v2_ground_from(const CharV2Env *e, const PhysicsMoverPlane *found, u32 n, v3 shift) {
  CharV2Probe walk = {0}, steep = {0};
  for (u32 i = 0; i < n; i++) {
    v3 nn = found[i].normal;
    if (nn.y <= 1e-3f) continue;
    f32 dist = CHAR_PROBE - found[i].offset + v3_dot(nn, shift);
    if (dist > CHAR_PROBE) continue;
    b32 walkable = nn.y >= e->cos_slope;
    CharV2Probe *slot = walkable ? &walk : &steep;
    if (slot->hit && nn.y <= slot->normal.y) continue;
    *slot = (CharV2Probe){.hit = true, .walkable = walkable, .dist = dist, .normal = nn, .body = found[i].shape.body};
  }
  return walk.hit ? walk : steep;
}

// CharacterVirtual::GetContactsAtPosition within CHAR_PROBE
hz_internal CharV2Probe char_v2_ground(const CharV2Env *e, v3 pos, PhysicsMoverPlane *found) {
  return char_v2_ground_from(e, found, char_v2_collide(e, pos, CHAR_PROBE, found), V3_ZERO);
}

// Box3D SolveMove with DetermineConstraints' horizontal plane on steep slopes; walking, walkable planes keep the velocity
// the planes reach CHAR_PROBE past the radius for the ground (Jolt's predictive contacts); the solve takes those within it
hz_internal b32 char_v2_move(const CharV2Env *e, v3 *pos, v3 *vel, v3 disp, b32 walk, CharV2Contact *ct,
                             CharV2Scratch *sc) {
  v3 target = v3_add(*pos, disp);
  u32 np = 0, n = 0;
  b32 settled = false;
  v3 d = V3_ZERO;
  for (u32 iter = 0; iter < CHAR_V2_ITERS && !settled; iter++) {
    n = char_v2_collide(e, *pos, CHAR_PROBE, sc->found);
    np = 0;
    for (u32 i = 0; i < n; i++) {
      v3 nn = sc->found[i].normal;
      f32 offset = sc->found[i].offset - CHAR_PROBE;
      if (offset < 0.0f) continue;
      b32 walkable = nn.y >= e->cos_slope;
      char_v2_note(e, ct, nn);
      sc->planes[np++] = (PhysicsCollisionPlane){nn, offset, FLT_MAX, 0.0f, !(walk && walkable)};
      f32 hl = m_sqrtf(nn.x * nn.x + nn.z * nn.z);
      if (nn.y > 1e-3f && !walkable && hl > 1e-4f)
        sc->planes[np++] = (PhysicsCollisionPlane){{nn.x / hl, 0.0f, nn.z / hl}, offset / hl, FLT_MAX, 0.0f, true};
    }
    PhysicsPlaneSolverResult r = physics_solve_planes(v3_sub(target, *pos), sc->planes, np);
    f32 f = char_v2_cast(e, *pos, r.delta);
    d = v3_scale(r.delta, f);
    *pos = v3_add(*pos, d);
    // no plane pushed and the sweep ran free: the whole move is done (Jolt MoveShape's time_remaining, :1278)
    b32 pushed = false;
    for (u32 i = 0; i < np && !pushed; i++) pushed = sc->planes[i].push > 0.0f;
    settled = v3_lengthsq(d) < CHAR_V2_MIN_MOVE || (f == 1.0f && !pushed);
  }
  sc->found_count = n;
  sc->found_shift = d;
  // the last pass's dynamic planes within the radius at the unclipped velocity (Box3D mover.cpp:215-253)
  for (u32 i = 0; sc->pushes && i < n && sc->pushes->len < fixed_arr_cap(*sc->pushes); i++) {
    if (sc->found[i].offset < CHAR_PROBE) continue;
    if (physics_body_get_type(e->cw->world, sc->found[i].shape.body) != PHYSICS_BODY_DYNAMIC) continue;
    v3 dir = v3_negate(sc->found[i].normal);
    fixed_arr_append(*sc->pushes,
                     ((CharacterV2Push){sc->found[i].shape.body, sc->found[i].point, dir, v3_dot(*vel, dir)}));
  }
  *vel = physics_clip_vector(*vel, sc->planes, np);
  return settled;
}

// CharacterVirtual::WalkStairs: up, forward, down; true when it lands on walkable ground at a new height
hz_internal b32 char_v2_step_up(const CharV2Env *e, const CharacterTuning *t, v3 start, v3 horiz, v3 *out_pos,
                                const CharV2Scratch *sc) {
  f32 up = t->step_height * char_v2_cast(e, start, (v3){0.0f, t->step_height, 0.0f});
  if (up < 0.02f) return false;
  v3 p1 = v3_add(start, (v3){0.0f, up, 0.0f});
  v3 p2 = v3_add(p1, v3_scale(horiz, char_v2_cast(e, p1, horiz)));
  if (v3_distance(p1, p2) < 0.01f) return false;
  f32 reach = up + CHAR_PROBE;
  f32 down = char_v2_cast(e, p2, (v3){0.0f, -reach, 0.0f});
  if (down >= 1.0f) return false;
  v3 land = v3_sub(p2, (v3){0.0f, reach * down, 0.0f});
  CharV2Probe g = char_v2_ground(e, land, sc->found);
  if (!g.hit || !g.walkable) return false;
  if (land.y <= start.y + 0.01f || land.y > start.y + t->step_height + CHAR_SKIN) return false;
  *out_pos = land;
  return true;
}

// the ground body's motion since the last step carries the character; a conveyor adds its velocity
hz_internal v3 char_v2_platform_carry(const CharacterWorld *cw, const CharacterTuning *t, Character *c, f32 dt, b32 net) {
  v3 carry = V3_ZERO, surface = V3_ZERO;
  v3 gp;
  quat gq;
  if (c->ground == CHARACTER_GROUND_ON_GROUND && !t->disable_riding &&
      char_v2_body_pose(cw, c->ground_body, net, &gp, &gq)) {
    b32 moved = !v3_eqv(gp, c->ground_body_pos) || gq.x != c->ground_body_rot.x || gq.y != c->ground_body_rot.y ||
                gq.z != c->ground_body_rot.z || gq.w != c->ground_body_rot.w;
    if (c->ground_body_valid && moved) {
      quat dq = quat_mul(gq, quat_conjugate(c->ground_body_rot));
      v3 rel = v3_sub(c->position, c->ground_body_pos);
      carry = v3_sub(v3_add(gp, quat_rotate(dq, rel)), c->position);
      v3 f = quat_rotate(dq, V3_FORWARD);
      c->yaw = m_wrap_pi(c->yaw + m_atan2f(f.x, f.z));
    }
    c->ground_body_pos = gp;
    c->ground_body_rot = gq;
    c->ground_body_valid = true;
    if (cw->surface_velocity) surface = cw->surface_velocity(c->ground_body, cw->ctx);
  }
  v3 move = v3_add(carry, v3_scale(surface, dt));
  c->ground_velocity = v3_scale(move, 1.0f / dt);
  return move;
}

hz_internal void char_v2_record_ground_body(const CharacterWorld *cw, Character *c, PhysicsBody prev, b32 net) {
  if (c->ground != CHARACTER_GROUND_ON_GROUND || !handle_is_valid(c->ground_body)) {
    c->ground_body_valid = false;
    return;
  }
  if (handle_equals(c->ground_body, prev) && c->ground_body_valid) return;
  c->ground_body_valid = char_v2_body_pose(cw, c->ground_body, net, &c->ground_body_pos, &c->ground_body_rot);
}

// crouch as char_crouch: stays down while the standing capsule has no room above the crouched one
hz_internal void char_v2_crouch(const CharacterWorld *cw, CharacterTuning *t, Character *c, CharacterInput *in,
                                f32 dt) {
  if (t->crouch_height <= 0.0f || t->crouch_height >= t->height) {
    c->crouched = false;
    c->crouch_k = 0.0f;
    return;
  }
  b32 want = in->crouch;
  if (!want && c->crouched) {
    CharacterTuning low = *t;
    low.height = t->crouch_height;
    CharV2Env env = char_v2_env_make(cw, &low, c->position.y);
    if (char_v2_cast(&env, c->position, (v3){0.0f, t->height - t->crouch_height + CHAR_SKIN, 0.0f}) < 1.0f) want = true;
  }
  c->crouched = want;
  c->crouch_k = m_saturatef(c->crouch_k + (want ? dt : -dt) / t->crouch_ease_s);
  if (want) {
    t->height = t->crouch_height;
    t->walk_speed = t->crouch_speed;
    t->run_speed = t->crouch_speed;
    in->run = false;
  }
}

hz_internal void char_v2_step(const CharacterWorld *cw, const CharacterTuning *t, Character *c, CharacterInput *in,
                              f32 dt, CharV2Scratch *sc) {
  if (!c->placed) {
    char_place(c, c->position);
    c->placed = true;
  }
  if (in->teleport) {
    char_place(c, in->teleport_pos);
    c->yaw = in->teleport_yaw;
    c->prev_yaw = c->yaw;
    c->teleports++;
    in->teleport = false;
  }
  c->prev_position = c->position;
  c->prev_yaw = c->yaw;

  b32 supported = c->ground == CHARACTER_GROUND_ON_GROUND;
  PhysicsBody prev_ground = c->ground_body;
  v3 ground_move = char_v2_platform_carry(cw, t, c, dt, sc->net);
  CharV2Env env = char_v2_env_make(cw, t, c->position.y);

  v3 vel = c->velocity;
  f32 vy_before = vel.y;
  if (char_launch(c, in, &vel)) supported = false;
  f32 surface = 0.0f;
  b32 swimming = char_swimming(t, cw->water, cw->water_ctx, c->position, &surface);
  b32 jumped = char_steer(t, c, in, dt, supported, swimming, t->gravity, &vel);
  if (jumped) {
    vel = v3_add(vel, c->ground_velocity);
    supported = false;
  }
  if (swimming) {
    f32 float_y = surface - t->swim_depth;
    vel.y = m_clampf((float_y - c->position.y) * t->buoyancy, -4.0f, 4.0f);
  } else if (supported && !jumped && vel.y <= 0.0f) {
    vel.y = 0.0f;
  } else {
    vel.y = m_maxf(vel.y - t->gravity * dt, -t->max_fall_speed);
  }
  c->swimming = swimming;

  b32 walk = supported && !jumped && !swimming;
  v3 pos = c->position;
  CharV2Contact ct = {0};
  v3 disp = v3_scale(vel, dt);
  if (walk) {
    // along the ground the character stands on, the horizontal length kept: slopes neither slow nor speed the walk
    v3 gn = c->ground_normal;
    disp = (v3){vel.x * dt, -(gn.x * vel.x + gn.z * vel.z) * dt / gn.y, vel.z * dt};
    disp = v3_add(disp, ground_move);
  }
  v3 begin = pos, vel_in = vel;
  b32 planes_current = char_v2_move(&env, &pos, &vel, disp, walk, &ct, sc);

  if (walk && ct.wall) {
    v3 horiz = {disp.x, 0.0f, disp.z};
    f32 want = v3_length(horiz);
    v3 got = {pos.x - begin.x, 0.0f, pos.z - begin.z};
    v3 stepped;
    // from where the move stopped (WalkStairs), at least until the lip's contact is walkable
    f32 reach = t->radius * (1.0f - m_sqrtf(1.0f - env.cos_slope * env.cos_slope)) + 2.0f * CHAR_SKIN;
    f32 rest = m_maxf(want - v3_length(got), reach);
    // a move that ends against the riser keeps its length but loses its speed: that is blocked too
    f32 kept = vel.x * vel_in.x + vel.z * vel_in.z, had = vel_in.x * vel_in.x + vel_in.z * vel_in.z;
    b32 blocked = v3_length(got) < want * 0.9f || kept < had * 0.81f;
    // the step up probes into the move's planes
    if (want > 1e-4f && blocked) planes_current = false;
    if (want > 1e-4f && blocked && char_v2_step_up(&env, t, pos, v3_scale(horiz, rest / want), &stepped, sc)) {
      v3 sh = {stepped.x - begin.x, 0.0f, stepped.z - begin.z};
      if (v3_length(sh) > v3_length(got) + 0.01f) {
        pos = stepped;
        vel.x = vel_in.x;
        vel.z = vel_in.z;
      }
    }
  }

  // the resting contact is the support; walking, StickToFloor sweeps further down for slopes and ledges
  b32 leaving = !walk && vel.y > 0.05f && !(ct.ground && v3_dot(vel, ct.ground_normal) < CHAR_SEPARATION_SPEED);
  CharV2Probe pr = {0};
  // settled, the move's last pass is the contact set it ended in (CharacterVirtual::Update, :1468)
  if (!leaving)
    pr = planes_current ? char_v2_ground_from(&env, sc->found, sc->found_count, sc->found_shift)
                        : char_v2_ground(&env, pos, sc->found);
  // to contact both ways: resting a slop in, as a cast leaves it, the next sweep stops on every new triangle's edge
  if (pr.hit && pr.walkable && walk) pos.y -= pr.dist / pr.normal.y;
  if (!(pr.hit && pr.walkable) && walk) {
    f32 f = char_v2_cast(&env, pos, (v3){0.0f, -t->snap_distance, 0.0f});
    if (f < 1.0f) {
      v3 snapped = v3_sub(pos, (v3){0.0f, t->snap_distance * f, 0.0f});
      CharV2Probe snap = char_v2_ground(&env, snapped, sc->found);
      if (snap.hit && snap.walkable) {
        pos = snapped;
        pr = snap;
      }
    }
  }
  CharacterGround next = CHARACTER_GROUND_IN_AIR;
  PhysicsBody ground_body = {0};
  v3 ground_normal = V3_UP;
  if (pr.hit && pr.walkable) {
    next = CHARACTER_GROUND_ON_GROUND;
    ground_body = pr.body;
    ground_normal = pr.normal;
  } else if (pr.hit) {
    next = CHARACTER_GROUND_STEEP;
    ground_normal = pr.normal;
  } else if (ct.touched) {
    next = CHARACTER_GROUND_NOT_SUPPORTED;
  }
  if (next == CHARACTER_GROUND_ON_GROUND) vel.y = 0.0f;
  if (c->ground != CHARACTER_GROUND_ON_GROUND && next == CHARACTER_GROUND_ON_GROUND && vy_before < -CHAR_LAND_MIN_SPEED &&
      !swimming) {
    c->landings++;
    c->land_speed = -vy_before;
  }

  c->ground = next;
  c->ground_body = ground_body;
  c->ground_normal = ground_normal;
  c->position = pos;
  c->velocity = vel;
  char_v2_record_ground_body(cw, c, prev_ground, sc->net);
  char_footsteps(t, c, next == CHARACTER_GROUND_ON_GROUND, dt);
}

hz_internal void char_v2_step_full(const CharacterWorld *world, const CharacterTuning *resolved, Character *c,
                                   CharacterInput *in, f32 dt, CharacterV2Pushes *pushes, b32 net) {
  CharacterTuning t = *resolved;
  if (pushes) pushes->len = 0;
  char_v2_crouch(world, &t, c, in, dt);
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  CharV2Scratch sc = {.found = ALLOC_ARRAY_NO_ZERO(&ta.allocator, PhysicsMoverPlane, CHAR_V2_PLANES),
                      .planes = ALLOC_ARRAY_NO_ZERO(&ta.allocator, PhysicsCollisionPlane, 2 * CHAR_V2_PLANES),
                      .pushes = pushes,
                      .net = net};
  char_v2_step(world, &t, c, in, dt, &sc);
  tctx_temp_allocator_end(ta);
}

void character_kinematic_step_v2(const CharacterWorld *world, const CharacterTuning *resolved, Character *c,
                                 CharacterInput *in, f32 dt, CharacterV2Pushes *pushes) {
  char_v2_step_full(world, resolved, c, in, dt, pushes, false);
}

void character_v2_apply_pushes(PhysicsWorld *world, const CharacterV2Pushes *pushes, u32 count) {
  assert_single_thread_begin("character_v2_apply_pushes");
  for (u32 c = 0; c < count; c++)
    for (u32 i = 0; i < pushes[c].len; i++) {
      const CharacterV2Push *p = &pushes[c].items[i];
      physics_body_push(world, p->body, p->point, p->direction, p->speed);
    }
  assert_single_thread_end();
}

// ---- the ECS kinematic mover: the v2 mover in ecs_rigidbody's world (dynamic bodies block and are pushed), else the query world ----

// a kinematic character's pushes from its last step, applied by character_push_system in table order
typedef struct CharacterPushes {
  CharacterV2Pushes pushes;
  CharacterTuning resolved; // character_step's: refreshed when the table's tuning column or rows changed
} CharacterPushes;
ECS_COMPONENT_TYPEDEF(CharacterPushes);

typedef struct CharEcsWorld {
  EcsWorld *w;
  const EcsPhysicsWorld *pw;
  const EcsRigidBodyWorld *sim; // ecs_rigidbody's world: bodies map through its entity table
} CharEcsWorld;

hz_internal EcsEntity char_ecs_entity(const CharEcsWorld *e, PhysicsBody body) {
  return e->sim ? ecs_rigidbody_entity_of(e->sim, body) : ecs_physics_entity(e->pw, e->pw->world, body);
}

hz_internal b32 char_ecs_one_way(PhysicsBody body, void *ctx) {
  const CharEcsWorld *e = (const CharEcsWorld *)ctx;
  EcsEntity ent = char_ecs_entity(e, body);
  return ent && ecs_component_get(e->w, ent, CharacterOneWay) != NULL;
}

hz_internal b32 char_ecs_pass_through(PhysicsBody body, void *ctx) {
  const CharEcsWorld *e = (const CharEcsWorld *)ctx;
  EcsEntity ent = char_ecs_entity(e, body);
  return ent && ecs_component_get(e->w, ent, CharacterPassThrough) != NULL;
}

// a conveyor's local velocity in the world, by the body's rotation
hz_internal v3 char_ecs_surface(PhysicsBody body, void *ctx) {
  const CharEcsWorld *e = (const CharEcsWorld *)ctx;
  EcsEntity ent = char_ecs_entity(e, body);
  const CharacterSurface *sf = ent ? ecs_component_get(e->w, ent, CharacterSurface) : NULL;
  if (!sf)
    return V3_ZERO;
  const PhysicsWorld *world = e->sim ? e->sim->world : e->pw->world;
  v3 p;
  quat q;
  return physics_body_get_transform(world, body, &p, &q) ? quat_rotate(q, sf->local_velocity) : V3_ZERO;
}

hz_internal void char_water_get(EcsWorld *w, CharacterWaterFn **fn, void **ctx);

// a callback reads its component per contact: none is installed while no entity carries it
hz_internal CharacterWorld char_ecs_world_ex(EcsWorld *w, CharEcsWorld *ctx, b32 one_way, b32 pass_through, b32 surface) {
  const EcsRigidBodyWorld *rb = ecs_resource_get(w, EcsRigidBodyWorld);
  PhysicsWorld *sim = rb ? rb->world : NULL;
  *ctx = (CharEcsWorld){.w = w, .pw = ecs_resource_get(w, EcsPhysicsWorld), .sim = sim ? rb : NULL};
  assert_msg(sim || ctx->pw, "character: a kinematic character needs ecs_rigidbody_init or ecs_physics_init");
  CharacterWorld cw = {.world = sim ? sim : ctx->pw->world,
                       .filter = PHYSICS_QUERY_FILTER_ALL,
                       .one_way = one_way ? char_ecs_one_way : NULL,
                       .pass_through = pass_through ? char_ecs_pass_through : NULL,
                       .surface_velocity = surface ? char_ecs_surface : NULL,
                       .ctx = ctx};
  char_water_get(w, &cw.water, &cw.water_ctx);
  return cw;
}

hz_internal b32 char_any_one_way(EcsWorld *w);
hz_internal b32 char_any_pass_through(EcsWorld *w);
hz_internal b32 char_any_surface(EcsWorld *w);

// any lane: the callbacks' presence from character_controller_init's cached queries, never a call-site query
hz_internal CharacterWorld char_ecs_world(EcsWorld *w, CharEcsWorld *ctx) {
  return char_ecs_world_ex(w, ctx, char_any_one_way(w), char_any_pass_through(w), char_any_surface(w));
}

hz_internal void char_ecs_kinematic_step(const CharacterWorld *cw, const CharEcsWorld *ctx, const CharacterTuning *t,
                                         Character *c, CharacterInput *in, const WorldTransform *wt, f32 dt,
                                         CharacterV2Pushes *pushes) {
  if (!c->placed) {
    c->yaw = char_yaw_of(wt->rotation);
    c->position = wt->position;
    c->render_position = wt->position;
  } else {
    char_kinematic_take_outside_write(c, wt);
  }
  // only ecs_rigidbody's bodies move when pushed
  b32 push = ctx->sim && !t->disable_push;
  character_kinematic_step_v2(cw, t, c, in, dt, push ? pushes : NULL);
  if (pushes && !push)
    pushes->len = 0;
  c->ground_entity = handle_is_valid(c->ground_body) ? char_ecs_entity(ctx, c->ground_body) : 0;
}

void character_kinematic_step(EcsWorld *w, const CharacterTuning *resolved, Character *c, CharacterInput *in,
                              const WorldTransform *wt, f32 dt, CharacterV2Pushes *pushes) {
  CharEcsWorld ctx;
  CharacterWorld cw = char_ecs_world(w, &ctx);
  char_ecs_kinematic_step(&cw, &ctx, resolved, c, in, wt, dt, pushes);
}

ecs_system(character_attach_system, EcsIn(CharacterTuning), EcsIn(WorldTransform), EcsNot(Character)) {
  for (i32 i = 0; i < it->count; i++) {
    EcsEntity e = it->entities[i];
    v3 p = ecs_sys_get_component(WorldTransform, i)->position;
    ecs_component_set(it->world, e, Character, {.position = p, .prev_position = p, .render_position = p, .ground = CHARACTER_GROUND_IN_AIR});
    ecs_component_set(it->world, e, CharacterInput, {0});
    ecs_component_set(it->world, e, CharacterPushes, {0});
  }
}

hz_internal void char_kinematic_rows(EcsIter *it, Character *c, CharacterInput *in, CharacterPushes *pushes,
                                     const CharacterTuning *tuning, const WorldTransform *wt) {
  CharEcsWorld ctx;
  CharacterWorld cw = char_ecs_world(it->world, &ctx);
  for (i32 i = 0; i < it->count; i++) {
    pushes[i].pushes.len = 0;
    if (tuning[i].mover != CHARACTER_MOVER_KINEMATIC)
      continue;
    CharacterTuning t = character_tuning_resolve(tuning[i]);
    char_ecs_kinematic_step(&cw, &ctx, &t, &c[i], &in[i], &wt[i], it->delta_time, &pushes[i].pushes);
  }
}

// ecs_rigidbody's world: read between its steps; the pushes go in by character_push_system
ecs_system(character_kinematic_body_system, EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(CharacterInput),
           EcsInOut(CharacterPushes), EcsIn(WorldTransform), EcsResIn(EcsRigidBodyWorld),
           EcsExternIn(CharacterOneWay), EcsExternIn(CharacterSurface)) {
  PROFILER_HW_SCOPE_BEGIN("character_kinematic");
  char_kinematic_rows(it, ecs_sys_get_component_mut(Character, 0), ecs_sys_get_component_mut(CharacterInput, 0),
                      ecs_sys_get_component_mut(CharacterPushes, 0), ecs_sys_get_component(CharacterTuning, 0),
                      ecs_sys_get_component(WorldTransform, 0));
  PROFILER_HW_SCOPE_END();
}

// no ecs_rigidbody: the query world's static and kinematic bodies
ecs_system(character_kinematic_system, EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(CharacterInput),
           EcsInOut(CharacterPushes), EcsIn(WorldTransform), EcsResIn(EcsPhysicsWorld), EcsExternIn(CharacterOneWay),
           EcsExternIn(CharacterSurface)) {
  PROFILER_HW_SCOPE_BEGIN("character_kinematic");
  char_kinematic_rows(it, ecs_sys_get_component_mut(Character, 0), ecs_sys_get_component_mut(CharacterInput, 0),
                      ecs_sys_get_component_mut(CharacterPushes, 0), ecs_sys_get_component(CharacterTuning, 0),
                      ecs_sys_get_component(WorldTransform, 0));
  PROFILER_HW_SCOPE_END();
}

// the direct API's rows query fields, which character_push_system reads too
enum { CHAR_ROW_TUNING, CHAR_ROW_CHARACTER, CHAR_ROW_INPUT, CHAR_ROW_PUSHES, CHAR_ROW_TRANSFORM, CHAR_ROW_BODY, CHAR_ROW_SLEEP, CHAR_ROW_COLLIDER,
       CHAR_ROW_LOCAL };

// one caller, in entity order: table rows move with the lanes, and pushes on one body do not commute (Box3D's serial loop)
typedef struct CharPushRows {
  u64 *keys;
  u32 *vals;
  const CharacterV2Pushes **rows;
  u32 capacity, count;
  b32 sorted;
} CharPushRows;

hz_internal CharPushRows char_push_rows_begin(Allocator *a, u32 capacity) {
  return (CharPushRows){.keys = ALLOC_ARRAY_NO_ZERO(a, u64, 2u * capacity),
                        .vals = ALLOC_ARRAY_NO_ZERO(a, u32, 2u * capacity),
                        .rows = ALLOC_ARRAY_NO_ZERO(a, const CharacterV2Pushes *, capacity),
                        .capacity = capacity,
                        .sorted = true};
}

hz_internal void char_push_rows_gather_range(CharPushRows *pr, EcsIter *it, i32 field, i32 lo, i32 hi) {
  const CharacterPushes *p = ecs_iter_get_component(it, CharacterPushes, field);
  if (!p)
    return;
  for (i32 i = lo; i < hi && pr->count < pr->capacity; i++) {
    if (!p[i].pushes.len)
      continue;
    u32 k = pr->count++;
    pr->sorted &= !k || pr->keys[k - 1u] < it->entities[i];
    pr->keys[k] = it->entities[i];
    pr->vals[k] = k;
    pr->rows[k] = &p[i].pushes;
  }
}

hz_internal void char_push_rows_gather(CharPushRows *pr, EcsIter *it, i32 field) {
  char_push_rows_gather_range(pr, it, field, 0, it->count);
}

// rows in spawn order are already in entity order
hz_internal void char_push_rows_apply(CharPushRows *pr, PhysicsWorld *world) {
  u32 n = pr->capacity, count = pr->count;
  const u32 *order =
      pr->sorted ? pr->vals : radix_sort_u64_auto(pr->keys, pr->vals, pr->keys + n, pr->vals + n, count, (RadixSortDesc){0}).vals;
  for (u32 i = 0; i < count; i++)
    character_v2_apply_pushes(world, pr->rows[order[i]], 1);
}

hz_internal void char_push_rows(EcsQuery *q, i32 field, PhysicsWorld *world) {
  u32 n = (u32)ecs_query_entity_count(q);
  if (!n)
    return;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  CharPushRows pr = char_push_rows_begin(&ta.allocator, n);
  ecs_query_foreach(q, qit) char_push_rows_gather(&pr, &qit, field);
  char_push_rows_apply(&pr, world);
  tctx_temp_allocator_end(ta);
}

hz_internal EcsQuery *char_rows_query(EcsWorld *w);

ecs_system(character_push_system, EcsResInOut(EcsRigidBodyWorld)) {
  char_push_rows(char_rows_query(it->world), CHAR_ROW_PUSHES, ecs_sys_get_resource_mut(EcsRigidBodyWorld)->world);
}

// ---- rigidbody mover (Character.cpp): the solver owns collision, the system steers the velocity before ecs_rigidbody_step ----

hz_internal void char_wake(RigidBodySleep *slp) {
  slp->state.asleep = false;
  slp->state.timer = 0.0f;
  slp->state.wake_requested = false;
}

hz_internal void char_rigidbody_step(EcsWorld *w, const CharacterTuning *t, Character *c, CharacterInput *in, RigidBody *rb,
                                     RigidBodySleep *slp, ColliderBody *cb, WorldTransform *wt, f32 gravity, f32 dt) {
  if (!c->placed) {
    c->yaw = char_yaw_of(wt->rotation);
    char_place(c, wt->position);
    c->placed = true;
  }
  if (in->teleport) {
    char_place(c, in->teleport_pos);
    c->yaw = in->teleport_yaw;
    c->prev_yaw = c->yaw;
    c->teleports++;
    in->teleport = false;
    wt->position = in->teleport_pos;
    cb->pos = in->teleport_pos;
    rb->velocity.linear = V3_ZERO;
    rb->velocity.angular = V3_ZERO;
  }
  c->position = wt->position;
  c->prev_position = c->position;
  c->prev_yaw = c->yaw;
  char_wake(slp);

  v3 carry = char_platform_carry(w, t, c, dt);
  if (v3_lengthsq(carry) > 0.0f) {
    c->position = v3_add(c->position, carry);
    wt->position = c->position;
    cb->pos = c->position;
  }
  CharEnv env = char_env_make(w, t, c->position.y);
  v3 vel = rb->velocity.linear;
  f32 vy_before = vel.y;
  CharProbe pr = char_ground_contact(&env, c->position);
  // leaving the floor along its normal is a jump; rising along it (the solver pushing up a slope) is support
  if (pr.hit && v3_dot(vel, pr.normal) > CHAR_SEPARATION_SPEED)
    pr = (CharProbe){0};
  if (!(pr.hit && pr.walkable) && vel.y <= 0.5f && c->ground == CHARACTER_GROUND_ON_GROUND) {
    CharProbe snap = char_probe(&env, c->position, t->snap_distance);
    if (snap.hit && snap.walkable)
      pr = snap;
  }
  b32 supported = pr.hit && pr.walkable;
  if (char_launch(c, in, &vel))
    supported = false;
  CharacterWaterFn *water = NULL;
  void *water_ctx = NULL;
  char_water_get(w, &water, &water_ctx);
  f32 surface = 0.0f;
  b32 swimming = char_swimming(t, water, water_ctx, c->position, &surface);
  b32 jumped = char_steer(t, c, in, dt, supported, swimming, gravity, &vel);
  // the integrator subtracts gravity after this system: cancel it wherever the body must hold its height
  f32 hold = gravity * dt;
  if (swimming) {
    vel.y = m_clampf((surface - t->swim_depth - c->position.y) * t->buoyancy, -4.0f, 4.0f) + hold;
  } else if (jumped) {
    vel.y += hold * 0.5f;
  } else if (supported) {
    vel.y = -m_minf(pr.dist, t->snap_distance) / dt * (pr.dist > 0.002f ? 1.0f : 0.0f) + hold;
    v3 horiz = {vel.x, 0.0f, vel.z};
    f32 hl = v3_length(horiz);
    CharCastHit r = {0};
    EcsEntity ent = 0;
    v3 look = v3_scale(horiz, hl > 1e-4f ? (hl * dt * 2.0f + t->radius * 0.5f) / hl : 0.0f);
    v3 stepped;
    if (hl > 1e-4f && char_cast(&env, c->position, v3_add(c->position, look), &r, &ent) && r.normal_on_target.y < env.cos_slope &&
        char_step_up(&env, t, c->position, look, &stepped)) {
      wt->position = stepped;
      cb->pos = stepped;
      c->position = stepped;
    }
  }
  rb->velocity.linear = vel;

  CharacterGround next = supported ? CHARACTER_GROUND_ON_GROUND : (pr.hit && !pr.walkable ? CHARACTER_GROUND_STEEP : CHARACTER_GROUND_IN_AIR);
  if (c->ground != CHARACTER_GROUND_ON_GROUND && next == CHARACTER_GROUND_ON_GROUND && vy_before < -CHAR_LAND_MIN_SPEED && !swimming) {
    c->landings++;
    c->land_speed = -vy_before;
  }
  EcsEntity prev_ground = c->ground_entity;
  c->ground = next;
  c->ground_entity = supported ? pr.entity : ECS_ENTITY_INVALID;
  c->ground_normal = supported ? pr.normal : V3_UP;
  c->swimming = swimming;
  c->velocity = vel;
  char_record_ground_body(w, c, prev_ground);
  char_footsteps(t, c, supported, dt);
}

// gravity is fixed at create, so reading it needs no resource term (systems carry at most 8)
hz_internal f32 char_rigidbody_gravity(EcsWorld *w) {
  PhysicsWorldSettings st;
  physics_world_settings(ecs_rigidbody_world(w), &st);
  return v3_length(st.gravity);
}

hz_internal void char_rigidbody_row(EcsWorld *w, const CharacterTuning *raw, Character *c, CharacterInput *in, RigidBody *rb,
                                    RigidBodySleep *slp, ColliderBody *cb, WorldTransform *wt, f32 gravity, f32 dt) {
  CharacterTuning t = character_tuning_resolve(*raw);
  b32 was = c->crouched;
  char_crouch(w, &t, c, in, dt);
  if (was != c->crouched)
    cb->collider = collider_capsule((v3){0.0f, t.radius, 0.0f}, (v3){0.0f, t.height - t.radius, 0.0f}, t.radius);
  char_rigidbody_step(w, &t, c, in, rb, slp, cb, wt, gravity, dt);
}

ecs_system(character_rigidbody_system, EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(CharacterInput),
           EcsInOut(RigidBody), EcsInOut(RigidBodySleep), EcsInOut(ColliderBody), EcsInOut(WorldTransform),
           EcsResIn(EcsPhysicsWorld)) {
  f32 gravity = char_rigidbody_gravity(it->world);
  PROFILER_HW_SCOPE_BEGIN("character_rigidbody");
  for (i32 i = 0; i < it->count; i++) {
    const CharacterTuning *raw = ecs_sys_get_component(CharacterTuning, i);
    if (raw->mover != CHARACTER_MOVER_RIGIDBODY)
      continue;
    char_rigidbody_row(it->world, raw, ecs_sys_get_component_mut(Character, i), ecs_sys_get_component_mut(CharacterInput, i),
                       ecs_sys_get_component_mut(RigidBody, i), ecs_sys_get_component_mut(RigidBodySleep, i),
                       ecs_sys_get_component_mut(ColliderBody, i), ecs_sys_get_component_mut(WorldTransform, i), gravity,
                       it->delta_time);
  }
  PROFILER_HW_SCOPE_END();
}

void character_schedule_fixed(EcsWorld *w) {
  if (ecs_resource_get(w, EcsRigidBodyWorld)) {
    ecs_schedule(w, character_kinematic_body_system);
    ecs_schedule_single(w, character_push_system);
  } else {
    ecs_schedule(w, character_kinematic_system);
  }
  // the rigidbody mover probes the query world: no EcsPhysicsWorld, no rigidbody character (character_spawn says so)
  if (ecs_resource_get(w, EcsRigidBodyWorld) && ecs_resource_get(w, EcsPhysicsWorld))
    ecs_schedule(w, character_rigidbody_system);
}

// ---- frame: interpolated pose and the avatar animation ----

// a placed character's interpolated pose; a kinematic one's transforms take it, a rigidbody one's visual is the caller's
hz_internal b32 char_pose_row(f32 alpha, const CharacterTuning *tuning, Character *c, LocalTransform *lt, WorldTransform *wt,
                              v3 *out_p, quat *out_rot) {
  if (!c->placed)
    return false;
  b32 kinematic = tuning->mover == CHARACTER_MOVER_KINEMATIC;
  if (kinematic)
    char_kinematic_take_outside_write(c, wt);
  v3 curr = kinematic ? c->position : wt->position;
  if (!kinematic)
    c->position = curr;
  v3 p = v3_lerp(c->prev_position, curr, alpha);
  f32 yaw = m_wrap_pi(c->prev_yaw + m_wrap_pi(c->yaw - c->prev_yaw) * alpha);
  quat rot = quat_rotate_y(yaw);
  *out_p = p;
  *out_rot = rot;
  if (!kinematic)
    return true;
  lt->position = p;
  lt->rotation = rot;
  wt->position = p;
  wt->rotation = rot;
  c->render_position = p;
  return false;
}

// a kinematic character's pose right after its step, while the row is in this lane's cache
hz_internal void char_pose_kinematic(f32 alpha, Character *c, LocalTransform *lt, WorldTransform *wt) {
  v3 p = v3_lerp(c->prev_position, c->position, alpha);
  quat rot = quat_rotate_y(m_wrap_pi(c->prev_yaw + m_wrap_pi(c->yaw - c->prev_yaw) * alpha));
  lt->position = p;
  lt->rotation = rot;
  wt->position = p;
  wt->rotation = rot;
  c->render_position = p;
}

hz_internal void char_visual_set(EcsWorld *w, const CharacterVisual *vis, v3 p, quat rot) {
  if (!vis || !vis->entity)
    return;
  ecs_component_set(w, vis->entity, LocalTransform, {.position = p, .rotation = rot, .scale = V3_ONE});
  ecs_component_set(w, vis->entity, WorldTransform, {.position = p, .rotation = rot, .scale = V3_ONE});
}

ecs_system(character_pose_system, EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(LocalTransform),
           EcsInOut(WorldTransform), EcsOptional(CharacterVisual), EcsResIn(CharacterFrame)) {
  f32 alpha = ecs_sys_get_resource(CharacterFrame)->alpha;
  for (i32 i = 0; i < it->count; i++) {
    v3 p;
    quat rot;
    if (char_pose_row(alpha, ecs_sys_get_component(CharacterTuning, i), ecs_sys_get_component_mut(Character, i),
                      ecs_sys_get_component_mut(LocalTransform, i), ecs_sys_get_component_mut(WorldTransform, i), &p, &rot))
      char_visual_set(it->world, ecs_sys_get_component_optional(CharacterVisual, i), p, rot);
  }
}

hz_internal b32 char_airborne(CharacterGround g) { return g != CHARACTER_GROUND_ON_GROUND; }

hz_internal void char_avatar_row(EcsWorld *w, const AssetSystem *assets, const Character *c, CharacterAvatar *av) {
  EcsAnimationLayer *al = ecs_component_get_mut(w, av->avatar, EcsAnimationLayer);
  if (!al || !asset_is_ready(assets, av->idle) || !asset_is_ready(assets, av->run) || !asset_is_ready(assets, av->fall))
    return;

  // the fall pose is held a few frames past landing before the crossfade
  b32 air = char_airborne(c->ground);
  if (air)
    av->landed_hold = 0;
  else if (av->was_airborne)
    av->landed_hold = CHAR_LANDED_HOLD_FRAMES;
  av->was_airborne = air;
  if (av->landed_hold > 0) {
    air = true;
    av->landed_hold--;
  }

  f32 speed = character_planar_speed(c);
  f32 blend = m_saturatef((speed - av->run_lo) / m_maxf(av->run_hi - av->run_lo, 1e-3f));
  if (blend > 0.6f)
    av->run_dominant = true;
  else if (blend < 0.4f)
    av->run_dominant = false;

  HzHandleT(AnimationBlobAsset) target = air ? av->fall : (av->run_dominant ? av->run : av->idle);
  AnimationBlobAsset *blob = asset_get(assets, target, AnimationBlobAsset);
  if (animation_layer_current_animation(&al->layer) != blob && !animation_layer_has_animation(&al->layer, blob)) {
    animation_layer_play_ex(&al->layer, blob, ANIM_TRANSITION_CROSSFADE, 0.20f, 1.0f, true, 0.0f);
    ecs_component_set(w, av->avatar, EcsAnimationRef, {.animation = blob_asset_ref_from_handle(AnimationBlobAsset, target)});
  }
}

ecs_system(character_avatar_system, EcsIn(Character), EcsInOut(CharacterAvatar), EcsExternInOut(EcsAnimationLayer),
           EcsResIn(AssetSystem)) {
  const AssetSystem *assets = ecs_sys_get_resource(AssetSystem);
  for (i32 i = 0; i < it->count; i++)
    char_avatar_row(it->world, assets, ecs_sys_get_component(Character, i), ecs_sys_get_component_mut(CharacterAvatar, i));
}

void character_schedule_frame(EcsWorld *w) {
  ecs_schedule(w, character_attach_system); // once a frame: its deferred sets apply at the scope's end, after every fixed step
  ecs_schedule(w, character_pose_system);
  ecs_schedule_single(w, character_avatar_system); // writes other entities' animation layers
}

// ---- the direct lock-step API: every lane, no flush ----

#define CHAR_DIRECT_CHUNK 4 // rows per lane claim

enum { CHAR_POSE_TUNING, CHAR_POSE_CHARACTER, CHAR_POSE_LOCAL, CHAR_POSE_WORLD, CHAR_POSE_VISUAL, CHAR_POSE_PUSHES, CHAR_POSE_INPUT };
enum { CHAR_Q_ROWS, CHAR_Q_POSE, CHAR_Q_ATTACH, CHAR_Q_VISUAL, CHAR_Q_AVATAR, CHAR_Q_ONE_WAY, CHAR_Q_PASS_THROUGH, CHAR_Q_SURFACE, CHAR_Q_COUNT };

// one lane's query counts, each valid while the world's structure version equals its key
typedef struct CharLaneCounts {
  _Alignas(64) u64 step_version;
  u32 table_count, block_count;
  b32 have_bodies;
  u64 head_version;
  u32 unattached;
  b32 visuals, avatars;
  u64 pose_version;
  u32 pose_total;
  b32 rigid;
  u64 world_version;
  CharEcsWorld ctx;
  CharacterWorld cw; // its ctx is this slot's
  u64 frame_version;
  const CharacterFrame *frame;
} CharLaneCounts;

// character_step's and character_frame's cached queries, claim words and completion count
typedef struct CharacterDirect {
  Allocator alloc;
  EcsQuery *q; // CHAR_Q_COUNT, at a stable address
  EcsEntity comp_tuning, comp_character, comp_input, comp_pushes, comp_transform, comp_body, comp_sleep, comp_collider;
  EcsEntity comp_local, comp_visual, comp_avatar;
  volatile u32 *blocks;
  u32 *push_blocks; // the step's blocks whose rows pushed, in claim order; lane 0's tail empties it
  u32 block_capacity;
  u32 lane_pass[HZ_MAX_LANES];
  u64 frame_gen[HZ_MAX_LANES]; // per lane: the posed_gen its last character_frame saw
  f32 posed_alpha;             // lane 0's, in character_step's tail: the alpha the step posed its kinematic rows at
  u64 posed_gen;               // character_steps that posed
  _Alignas(64) volatile u64 done;
  u64 target;
  volatile u32 push_count;
  CharacterWaterFn *water; // character_water_set's
  void *water_ctx;
  CharLaneCounts counts[HZ_MAX_LANES];
} CharacterDirect;
ECS_COMPONENT_TYPEDEF(CharacterDirect);

#define CHAR_COMP(w, T) ecs_component_ensure((w), sizeof(T), _Alignof(T), #T)

hz_internal void char_direct_init(EcsWorld *w) {
  ecs_resource_set_zero(w, CharacterDirect);
  CharacterDirect *d = ecs_resource_get_mut(w, CharacterDirect);
  d->alloc = ecs_owned_tlsf(w, CharacterDirect, KB(256));
  d->q = ALLOC_ARRAY(&d->alloc, EcsQuery, CHAR_Q_COUNT);
  ecs_query_cached_init(w, &d->q[CHAR_Q_ROWS], EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(CharacterInput),
                        EcsOptional(CharacterPushes), EcsInOut(WorldTransform), EcsOptional(RigidBody),
                        EcsOptional(RigidBodySleep), EcsOptional(ColliderBody), EcsOptional(LocalTransform));
  ecs_query_cached_init(w, &d->q[CHAR_Q_POSE], EcsIn(CharacterTuning), EcsInOut(Character), EcsInOut(LocalTransform),
                        EcsInOut(WorldTransform), EcsOptional(CharacterVisual), EcsOptional(CharacterPushes),
                        EcsOptional(CharacterInput));
  ecs_query_cached_init(w, &d->q[CHAR_Q_ATTACH], EcsIn(CharacterTuning), EcsIn(WorldTransform), EcsNot(Character));
  ecs_query_cached_init(w, &d->q[CHAR_Q_VISUAL], EcsIn(CharacterVisual), EcsWith(Character), EcsWith(LocalTransform));
  ecs_query_cached_init(w, &d->q[CHAR_Q_AVATAR], EcsIn(Character), EcsInOut(CharacterAvatar));
  ecs_query_cached_init(w, &d->q[CHAR_Q_ONE_WAY], EcsIn(CharacterOneWay));
  ecs_query_cached_init(w, &d->q[CHAR_Q_PASS_THROUGH], EcsIn(CharacterPassThrough));
  ecs_query_cached_init(w, &d->q[CHAR_Q_SURFACE], EcsIn(CharacterSurface));
  d->comp_tuning = CHAR_COMP(w, CharacterTuning);
  d->comp_character = CHAR_COMP(w, Character);
  d->comp_input = CHAR_COMP(w, CharacterInput);
  d->comp_pushes = CHAR_COMP(w, CharacterPushes);
  d->comp_transform = CHAR_COMP(w, WorldTransform);
  d->comp_body = CHAR_COMP(w, RigidBody);
  d->comp_sleep = CHAR_COMP(w, RigidBodySleep);
  d->comp_collider = CHAR_COMP(w, ColliderBody);
  d->comp_local = CHAR_COMP(w, LocalTransform);
  d->comp_visual = CHAR_COMP(w, CharacterVisual);
  d->comp_avatar = CHAR_COMP(w, CharacterAvatar);
}

hz_internal EcsQuery *char_rows_query(EcsWorld *w) {
  const CharacterDirect *d = ecs_resource_get(w, CharacterDirect);
  assert_msg(d, "character: call character_controller_init first");
  return &d->q[CHAR_Q_ROWS];
}

hz_internal b32 char_any_one_way(EcsWorld *w) {
  const CharacterDirect *d = ecs_resource_get(w, CharacterDirect);
  assert_msg(d, "character: call character_controller_init first");
  return ecs_query_entity_count(&d->q[CHAR_Q_ONE_WAY]) > 0;
}

hz_internal b32 char_any_pass_through(EcsWorld *w) {
  const CharacterDirect *d = ecs_resource_get(w, CharacterDirect);
  assert_msg(d, "character: call character_controller_init first");
  return ecs_query_entity_count(&d->q[CHAR_Q_PASS_THROUGH]) > 0;
}

hz_internal b32 char_any_surface(EcsWorld *w) {
  const CharacterDirect *d = ecs_resource_get(w, CharacterDirect);
  assert_msg(d, "character: call character_controller_init first");
  return ecs_query_entity_count(&d->q[CHAR_Q_SURFACE]) > 0;
}

hz_internal void char_water_get(EcsWorld *w, CharacterWaterFn **fn, void **ctx) {
  const CharacterDirect *d = ecs_resource_get(w, CharacterDirect);
  assert_msg(d, "character: call character_controller_init first");
  *fn = d->water;
  *ctx = d->water_ctx;
}

hz_internal CharacterDirect *char_direct(EcsWorld *w) {
  CharacterDirect *d = ecs_resource_get_mut(w, CharacterDirect);
  assert_msg(d, "character_step / character_frame: call character_controller_init first");
  return d;
}

// the claim words cover every block; every lane reads the same counts, so the barriers are taken by all or none
hz_internal void char_direct_reserve(CharacterDirect *d, u32 blocks) {
  if (blocks <= d->block_capacity)
    return;
  lane_sync();
  if (is_main_thread()) {
    u32 cap = MAX(blocks, d->block_capacity * 2u);
    if (d->blocks)
      ALLOC_FREE(&d->alloc, (u32 *)d->blocks);
    if (d->push_blocks)
      ALLOC_FREE(&d->alloc, d->push_blocks);
    d->blocks = ALLOC_ARRAY(&d->alloc, u32, cap);
    d->push_blocks = ALLOC_ARRAY_NO_ZERO(&d->alloc, u32, cap);
    d->block_capacity = cap;
    for (u32 l = 0; l < HZ_MAX_LANES; l++)
      d->lane_pass[l] = 0;
  }
  lane_sync();
}

// one claimed block: its table's columns and row span
typedef struct CharRowsBlock {
  EcsIter it;
  u32 first_block;
} CharRowsBlock;

// true if a row's step pushed a body
hz_internal b32 char_direct_rows(CharacterDirect *d, CharEcsWorld *ctx, const CharacterWorld *cw, b32 rigid, f32 gravity,
                                 f32 alpha, EcsIter *it, i32 lo, i32 hi, f32 dt) {
  // the claim's check, before lane 0's sync after every block is done
  b32 refresh = ecs_iter_field_changed(it, CHAR_ROW_TUNING);
  const CharacterTuning *tuning = ecs_iter_get_component_id(it, CHAR_ROW_TUNING, d->comp_tuning);
  Character *c = ecs_iter_get_component_id(it, CHAR_ROW_CHARACTER, d->comp_character);
  CharacterInput *in = ecs_iter_get_component_id(it, CHAR_ROW_INPUT, d->comp_input);
  CharacterPushes *pushes = ecs_iter_get_component_id(it, CHAR_ROW_PUSHES, d->comp_pushes);
  WorldTransform *wt = ecs_iter_get_component_id(it, CHAR_ROW_TRANSFORM, d->comp_transform);
  RigidBody *rb = ecs_iter_get_component_id(it, CHAR_ROW_BODY, d->comp_body);
  RigidBodySleep *slp = ecs_iter_get_component_id(it, CHAR_ROW_SLEEP, d->comp_sleep);
  ColliderBody *cb = ecs_iter_get_component_id(it, CHAR_ROW_COLLIDER, d->comp_collider);
  LocalTransform *lt = ecs_iter_get_component_id(it, CHAR_ROW_LOCAL, d->comp_local);
  b32 bodies = rigid && rb && slp && cb;
  b32 pushed = false;
  for (i32 i = lo; i < hi; i++) {
    // as character_schedule_fixed: the kinematic mover in tables carrying pushes, the rigidbody mover in body tables
    if (pushes) {
      pushes[i].pushes.len = 0;
      if (tuning[i].mover == CHARACTER_MOVER_KINEMATIC) {
        if (refresh)
          pushes[i].resolved = character_tuning_resolve(tuning[i]);
        char_ecs_kinematic_step(cw, ctx, &pushes[i].resolved, &c[i], &in[i], &wt[i], dt, &pushes[i].pushes);
        pushed |= pushes[i].pushes.len != 0;
        if (lt)
          char_pose_kinematic(alpha, &c[i], &lt[i], &wt[i]);
        continue;
      }
    }
    if (bodies && tuning[i].mover == CHARACTER_MOVER_RIGIDBODY)
      char_rigidbody_row(ctx->w, &tuning[i], &c[i], &in[i], &rb[i], &slp[i], &cb[i], &wt[i], gravity, dt);
  }
  return pushed;
}

hz_internal b32 char_rows_have_bodies(EcsIter *it) {
  return ecs_iter_component_is_set(it, CHAR_ROW_BODY) && ecs_iter_component_is_set(it, CHAR_ROW_SLEEP) &&
         ecs_iter_component_is_set(it, CHAR_ROW_COLLIDER);
}

hz_internal void char_direct_step_tail(CharacterDirect *d, EcsWorld *w, EcsQuery *q, const CharRowsBlock *tables, u32 table_count,
                                       b32 rigid, f32 alpha, u32 block_count);

hz_internal f32 char_lane_alpha(EcsWorld *w, CharLaneCounts *lc, u64 version) {
  if (lc->frame_version != version) {
    lc->frame = ecs_resource_get(w, CharacterFrame);
    lc->frame_version = version;
  }
  return lc->frame ? lc->frame->alpha : 1.0f;
}

void character_step(EcsWorld *w, f32 dt) {
  CharacterDirect *d = char_direct(w);
  EcsQuery *q = &d->q[CHAR_Q_ROWS];
  u32 lane = tctx_current()->thread_idx;
  CharLaneCounts *lc = &d->counts[lane];
  u64 version = ecs_world_structure_version(w);
  if (lc->step_version != version) {
    lc->rigid = ecs_resource_get(w, EcsRigidBodyWorld) && ecs_resource_get(w, EcsPhysicsWorld);
    lc->table_count = lc->block_count = 0;
    lc->have_bodies = false;
    ecs_query_foreach(q, qit) {
      lc->table_count++;
      lc->block_count += ((u32)qit.count + CHAR_DIRECT_CHUNK - 1u) / CHAR_DIRECT_CHUNK;
      lc->have_bodies |= char_rows_have_bodies(&qit);
    }
    lc->step_version = version;
  }
  b32 rigid = lc->rigid;
  u32 table_count = lc->table_count, block_count = lc->block_count;
  b32 bodies = rigid && lc->have_bodies;
  if (!block_count)
    return;
  char_direct_reserve(d, block_count);
  if (lc->world_version != version) {
    lc->cw = char_ecs_world(w, &lc->ctx);
    lc->world_version = version;
  }
  // only the rigidbody movers read gravity
  f32 gravity = bodies ? char_rigidbody_gravity(w) : 0.0f;
  f32 alpha = char_lane_alpha(w, lc, version);
  u32 pass = d->lane_pass[lane] + 1u ? d->lane_pass[lane] + 1u : 1u;
  d->lane_pass[lane] = pass;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  CharRowsBlock *tables = ALLOC_ARRAY_NO_ZERO(&ta.allocator, CharRowsBlock, table_count);
  u32 t = 0, first = 0;
  ecs_query_foreach(q, qit) {
    debug_assert_msg(t < table_count, "character_step: the world changed without a structure version bump");
    tables[t++] = (CharRowsBlock){.it = qit, .first_block = first};
    first += ((u32)qit.count + CHAR_DIRECT_CHUNK - 1u) / CHAR_DIRECT_CHUNK;
  }
  debug_assert(t == table_count && first == block_count);
  // a character stays on its lane step to step, so its state stays in that core's cache
  LaneBlocks claims = lane_blocks_begin(d->blocks, block_count, pass);
  u32 block, done = 0, k = 0;
  PROFILER_HW_SCOPE_BEGIN("character_step");
  while (lane_blocks_next(&claims, &block)) {
    while (k + 1u < table_count && tables[k + 1u].first_block <= block)
      k++;
    while (tables[k].first_block > block)
      k--;
    i32 lo = (i32)((block - tables[k].first_block) * CHAR_DIRECT_CHUNK);
    i32 hi = MIN(lo + CHAR_DIRECT_CHUNK, tables[k].it.count);
    if (char_direct_rows(d, &lc->ctx, &lc->cw, rigid, gravity, alpha, &tables[k].it, lo, hi, dt))
      d->push_blocks[atomic_u32_inc_eval(&d->push_count) - 1u] = block;
    done++;
  }
  PROFILER_HW_SCOPE_END();
  if (done)
    atomic_u64_add_eval(&d->done, done);
  // a lane holding no claim never holds the step; the rigidbody step reads body rows, so they are published here
  if (is_main_thread())
    char_direct_step_tail(d, w, q, tables, table_count, rigid, alpha, block_count);
  tctx_temp_allocator_end(ta);
  if (bodies)
    lane_sync();
}

hz_internal void char_direct_step_tail(CharacterDirect *d, EcsWorld *w, EcsQuery *q, const CharRowsBlock *tables, u32 table_count,
                                       b32 rigid, f32 alpha, u32 block_count) {
  d->target += block_count;
  while (atomic_u64_read_acquire(&d->done) < d->target)
    cpu_pause();
  // only the blocks that pushed: the apply's radix sort restores entity order
  PhysicsWorld *sim = ecs_rigidbody_world(w);
  u32 push_blocks = d->push_count;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  CharPushRows pr = char_push_rows_begin(&ta.allocator, sim ? push_blocks * CHAR_DIRECT_CHUNK : 0u);
  for (u32 b = 0; sim && b < push_blocks; b++) {
    u32 block = d->push_blocks[b], lo_t = 0, hi_t = table_count;
    while (hi_t - lo_t > 1u) {
      u32 mid = (lo_t + hi_t) / 2u;
      if (tables[mid].first_block <= block) lo_t = mid;
      else hi_t = mid;
    }
    EcsIter it = tables[lo_t].it;
    i32 lo = (i32)((block - tables[lo_t].first_block) * CHAR_DIRECT_CHUNK);
    char_push_rows_gather_range(&pr, &it, CHAR_ROW_PUSHES, lo, MIN(lo + CHAR_DIRECT_CHUNK, it.count));
  }
  d->push_count = 0;
  ecs_query_foreach(q, qit) {
    if (ecs_iter_field_changed(&qit, CHAR_ROW_TUNING))
      ecs_iter_sync(&qit);
    ecs_iter_mark_modified(&qit, CHAR_ROW_CHARACTER);
    ecs_iter_mark_modified(&qit, CHAR_ROW_INPUT);
    ecs_iter_mark_modified(&qit, CHAR_ROW_PUSHES);
    b32 posed = ecs_iter_component_is_set(&qit, CHAR_ROW_PUSHES) && ecs_iter_component_is_set(&qit, CHAR_ROW_LOCAL);
    if (posed) {
      ecs_iter_mark_modified(&qit, CHAR_ROW_LOCAL);
      ecs_iter_mark_modified(&qit, CHAR_ROW_TRANSFORM);
    }
    if (!rigid || !char_rows_have_bodies(&qit))
      continue;
    ecs_iter_mark_modified(&qit, CHAR_ROW_TRANSFORM);
    ecs_iter_mark_modified(&qit, CHAR_ROW_BODY);
    ecs_iter_mark_modified(&qit, CHAR_ROW_SLEEP);
    ecs_iter_mark_modified(&qit, CHAR_ROW_COLLIDER);
  }
  d->posed_alpha = alpha;
  d->posed_gen++;
  if (sim)
    char_push_rows_apply(&pr, sim);
  tctx_temp_allocator_end(ta);
}

// the attach system's sets, made now: the rows move tables, so the entities are gathered first
hz_internal void char_direct_attach(CharacterDirect *d, EcsWorld *w, u32 count) {
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  EcsEntity *ents = ALLOC_ARRAY_NO_ZERO(&ta.allocator, EcsEntity, count);
  v3 *feet = ALLOC_ARRAY_NO_ZERO(&ta.allocator, v3, count);
  u32 n = 0;
  ecs_query_foreach(&d->q[CHAR_Q_ATTACH], qit) {
    const WorldTransform *wt = ecs_iter_get_component_id(&qit, 1, d->comp_transform);
    for (i32 i = 0; i < qit.count && n < count; i++, n++) {
      ents[n] = qit.entities[i];
      feet[n] = wt[i].position;
    }
  }
  for (u32 i = 0; i < n; i++) {
    v3 p = feet[i];
    ecs_component_set(w, ents[i], Character, {.position = p, .prev_position = p, .render_position = p, .ground = CHARACTER_GROUND_IN_AIR});
    ecs_component_set(w, ents[i], CharacterInput, {0});
    ecs_component_set(w, ents[i], CharacterPushes, {0});
  }
  tctx_temp_allocator_end(ta);
}

// the rigidbody movers' visual pivots and the avatars: other entities' components, one caller
hz_internal void char_direct_frame_serial(CharacterDirect *d, EcsWorld *w, f32 alpha, b32 visuals, b32 avatars) {
  if (visuals) {
    ecs_query_foreach(&d->q[CHAR_Q_POSE], qit) {
      if (!ecs_iter_component_is_set(&qit, CHAR_POSE_VISUAL))
        continue;
      const CharacterTuning *tuning = ecs_iter_get_component_id(&qit, CHAR_POSE_TUNING, d->comp_tuning);
      Character *c = ecs_iter_get_component_id(&qit, CHAR_POSE_CHARACTER, d->comp_character);
      const CharacterVisual *vis = ecs_iter_get_component_id(&qit, CHAR_POSE_VISUAL, d->comp_visual);
      for (i32 i = 0; i < qit.count; i++) {
        if (!c[i].placed || tuning[i].mover == CHARACTER_MOVER_KINEMATIC)
          continue;
        f32 yaw = m_wrap_pi(c[i].prev_yaw + m_wrap_pi(c[i].yaw - c[i].prev_yaw) * alpha);
        char_visual_set(w, &vis[i], v3_lerp(c[i].prev_position, c[i].position, alpha), quat_rotate_y(yaw));
      }
    }
  }
  if (avatars) {
    const AssetSystem *assets = ecs_resource_get(w, AssetSystem);
    ecs_query_foreach(&d->q[CHAR_Q_AVATAR], qit) {
      const Character *c = ecs_iter_get_component_id(&qit, 0, d->comp_character);
      CharacterAvatar *av = ecs_iter_get_component_id(&qit, 1, d->comp_avatar);
      ecs_iter_mark_modified(&qit, 1);
      for (i32 i = 0; i < qit.count; i++)
        char_avatar_row(w, assets, &c[i], &av[i]);
    }
  }
}

void character_frame(EcsWorld *w) {
  CharacterDirect *d = char_direct(w);
  u32 lane = tctx_current()->thread_idx;
  CharLaneCounts *lc = &d->counts[lane];
  // every lane reads the counts before any lane changes the world, so the barriers are taken by all or none
  u64 version = ecs_world_structure_version(w);
  if (lc->head_version != version) {
    lc->unattached = (u32)ecs_query_entity_count(&d->q[CHAR_Q_ATTACH]);
    lc->visuals = ecs_query_entity_count(&d->q[CHAR_Q_VISUAL]) > 0;
    lc->avatars = ecs_query_entity_count(&d->q[CHAR_Q_AVATAR]) > 0;
    lc->head_version = version;
  }
  u32 unattached = lc->unattached;
  b32 visuals = lc->visuals, avatars = lc->avatars;
  if (unattached) {
    lane_sync();
    if (is_main_thread())
      char_direct_attach(d, w, unattached);
    lane_sync();
  }
  f32 alpha = char_lane_alpha(w, lc, ecs_world_structure_version(w));
  // a step since this lane's last frame posed the kinematic rows of tables carrying pushes, at this alpha
  b32 posed = d->posed_gen != d->frame_gen[lane] && d->posed_alpha == alpha;
  d->frame_gen[lane] = d->posed_gen;
  EcsQuery *q = &d->q[CHAR_Q_POSE];
  version = ecs_world_structure_version(w);
  if (lc->pose_version != version) {
    lc->pose_total = (u32)ecs_query_entity_count(q);
    lc->pose_version = version;
  }
  u32 total = lc->pose_total;
  Range(u32) r = lane_range(total);
  u32 base = 0;
  ecs_query_foreach(q, qit) {
    u32 lo = MAX(r.min, base), hi = MIN(r.max, base + (u32)qit.count);
    // character_step's rows: the tables carrying pushes and input
    b32 stepped = posed && ecs_iter_component_is_set(&qit, CHAR_POSE_PUSHES) && ecs_iter_component_is_set(&qit, CHAR_POSE_INPUT);
    if (lo < hi) {
      const CharacterTuning *tuning = ecs_iter_get_component_id(&qit, CHAR_POSE_TUNING, d->comp_tuning);
      Character *c = ecs_iter_get_component_id(&qit, CHAR_POSE_CHARACTER, d->comp_character);
      LocalTransform *lt = ecs_iter_get_component_id(&qit, CHAR_POSE_LOCAL, d->comp_local);
      WorldTransform *wt = ecs_iter_get_component_id(&qit, CHAR_POSE_WORLD, d->comp_transform);
      b32 wrote = false;
      for (u32 i = lo - base; i < hi - base; i++) {
        if (stepped && tuning[i].mover == CHARACTER_MOVER_KINEMATIC)
          continue;
        v3 p;
        quat rot;
        char_pose_row(alpha, &tuning[i], &c[i], &lt[i], &wt[i], &p, &rot);
        wrote = true;
      }
      if (wrote) {
        ecs_iter_mark_modified(&qit, CHAR_POSE_CHARACTER);
        ecs_iter_mark_modified(&qit, CHAR_POSE_LOCAL);
        ecs_iter_mark_modified(&qit, CHAR_POSE_WORLD);
      }
    }
    base += (u32)qit.count;
  }
  if (!visuals && !avatars)
    return;
  lane_sync();
  if (is_main_thread())
    char_direct_frame_serial(d, w, alpha, visuals, avatars);
  lane_sync();
}

// ---- setup ----

void character_controller_init(EcsWorld *w) {
  assert_msg(is_main_thread(), "character_controller_init: main lane only"); // ECS resources are set outside a scope
  ecs_resource_set(w, CharacterFrame, ((CharacterFrame){.alpha = 1.0f}));
  char_direct_init(w);
}

void character_water_set(EcsWorld *w, CharacterWaterFn *fn, void *ctx) {
  assert_msg(is_main_thread(), "character_water_set: main lane only"); // writes a resource outside a scope
  CharacterDirect *d = char_direct(w);
  d->water = fn;
  d->water_ctx = ctx;
}

// ---- networking: CharacterNet is the whole step state, on the wire's grids before and after every step ----

// compared on the grid, never by bits: the fast-math build may keep a -0 the wire sends as 0
hz_internal b32 char_net_same(f32 a, f32 b, f32 grid) { return net_q_f32(a, grid) == net_q_f32(b, grid); }

hz_internal b32 char_net_same3(v3 a, v3 b, f32 grid) {
  return char_net_same(a.x, b.x, grid) && char_net_same(a.y, b.y, grid) && char_net_same(a.z, b.z, grid);
}

// the timers only matter above 0: held there, they stay on a short range
hz_internal CharacterNet char_net_snapped(const CharacterNet *n) {
  b32 on = n->platform != 0;
  return (CharacterNet){.position = char_net_snap3(n->position, CHARACTER_NET_GRID),
                        .yaw = char_net_snap(m_wrap_pi(n->yaw), CHARACTER_NET_ANGLE_GRID),
                        .ground = n->ground,
                        .crouched = n->crouched,
                        .swimming = n->swimming,
                        .velocity = char_net_snap3(n->velocity, CHARACTER_NET_VEL_GRID),
                        .ground_normal = char_net_snap3(n->ground_normal, CHARACTER_NET_ANGLE_GRID),
                        .coyote = char_net_snap(m_maxf(n->coyote, 0.0f), CHARACTER_NET_TIME_GRID),
                        .jump_buffer = char_net_snap(m_maxf(n->jump_buffer, 0.0f), CHARACTER_NET_TIME_GRID),
                        .crouch_k = char_net_snap(m_saturatef(n->crouch_k), CHARACTER_NET_ANGLE_GRID),
                        .platform_pos = on ? char_net_snap3(n->platform_pos, CHARACTER_NET_GRID) : V3_ZERO,
                        .platform_rot = on ? (v4){char_net_snap(n->platform_rot.x, CHARACTER_NET_ANGLE_GRID),
                                                  char_net_snap(n->platform_rot.y, CHARACTER_NET_ANGLE_GRID),
                                                  char_net_snap(n->platform_rot.z, CHARACTER_NET_ANGLE_GRID),
                                                  char_net_snap(n->platform_rot.w, CHARACTER_NET_ANGLE_GRID)}
                                           : (v4){0},
                        .platform = n->platform,
                        .rest = n->rest};
}

hz_internal b32 char_net_same_state(const CharacterNet *a, const CharacterNet *b) {
  return char_net_same3(a->position, b->position, CHARACTER_NET_GRID) &&
         char_net_same(a->yaw, b->yaw, CHARACTER_NET_ANGLE_GRID) && a->ground == b->ground && a->crouched == b->crouched &&
         a->swimming == b->swimming && a->platform == b->platform &&
         char_net_same3(a->velocity, b->velocity, CHARACTER_NET_VEL_GRID) &&
         char_net_same3(a->ground_normal, b->ground_normal, CHARACTER_NET_ANGLE_GRID) &&
         char_net_same(a->coyote, b->coyote, CHARACTER_NET_TIME_GRID) &&
         char_net_same(a->jump_buffer, b->jump_buffer, CHARACTER_NET_TIME_GRID) &&
         char_net_same(a->crouch_k, b->crouch_k, CHARACTER_NET_ANGLE_GRID) &&
         char_net_same3(a->platform_pos, b->platform_pos, CHARACTER_NET_GRID);
}

// the walkable body under the feet: physics handles are each world's own, so the step finds its ground again from the pose
hz_internal PhysicsBody char_net_ground_body(const CharacterWorld *cw, const CharacterTuning *t, v3 feet) {
  CharV2Env env = char_v2_env_make(cw, t, feet.y);
  f32 grow = CHAR_PROBE + t->snap_distance;
  PhysicsBody best = {0};
  f32 best_d = FLT_MAX;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  PhysicsMoverPlane *found = ALLOC_ARRAY_NO_ZERO(&ta.allocator, PhysicsMoverPlane, CHAR_V2_PLANES);
  u32 n = char_v2_collide(&env, feet, grow, found);
  for (u32 i = 0; i < n; i++) {
    if (found[i].normal.y < env.cos_slope)
      continue;
    f32 d = m_absf(grow - found[i].offset);
    if (d < best_d) {
      best_d = d;
      best = found[i].shape.body;
    }
  }
  tctx_temp_allocator_end(ta);
  return best;
}

CharacterNet character_net_at(v3 feet, f32 yaw) {
  return char_net_snapped(&(CharacterNet){.position = feet, .yaw = yaw, .ground = CHARACTER_GROUND_IN_AIR, .ground_normal = V3_UP});
}

void character_net_init(EcsWorld *w, const CharacterTuning *tuning) {
  assert_msg(is_main_thread(), "character_net_init: main lane only"); // sets resources outside a scope
  if (!ecs_resource_get(w, CharacterDirect))
    character_controller_init(w);
  CharacterTuning t = character_tuning_resolve(*tuning);
  t.mover = CHARACTER_MOVER_KINEMATIC;
  ecs_resource_set(w, CharacterNetTuning, {.resolved = t});
}

void character_net_step(EcsWorld *w, const CharacterTuning *t, CharacterNet *n, CharacterInput *in, f32 dt) {
  f32 face = t->facing == CHARACTER_FACING_HEADING ? in->heading + M_PI : t->facing == CHARACTER_FACING_INPUT ? in->face_yaw : n->yaw;
  b32 idle = in->move.x == 0.0f && in->move.y == 0.0f && !in->jump && !in->teleport && (in->crouch != 0) == (n->crouched != 0) &&
             v3_lengthsq(in->add_velocity) == 0.0f && char_net_same(m_wrap_pi(face), n->yaw, CHARACTER_NET_ANGLE_GRID);
  if (idle && n->rest)
    return;
  CharacterNet from = char_net_snapped(n);
  CharEcsWorld ctx;
  CharacterWorld cw = char_ecs_world(w, &ctx);
  Character c = {.position = from.position,
                 .velocity = from.velocity,
                 .yaw = from.yaw,
                 .render_position = from.position,
                 .ground = (CharacterGround)from.ground,
                 .ground_normal = from.ground_normal,
                 .ground_body_pos = from.platform_pos,
                 .ground_body_rot = {from.platform_rot.x, from.platform_rot.y, from.platform_rot.z, from.platform_rot.w},
                 .ground_body_valid = from.platform != 0,
                 .crouched = from.crouched != 0,
                 .crouch_k = from.crouch_k,
                 .coyote = from.coyote,
                 .jump_buffer = from.jump_buffer,
                 .placed = true};
  // a static ground never carries (a conveyor's velocity needs only the body), so only a moving body's pose is kept
  if (c.ground == CHARACTER_GROUND_ON_GROUND && c.ground_body_valid) {
    c.ground_body = char_net_ground_body(&cw, t, c.position);
    c.ground_body_valid = from.platform == 2u && handle_is_valid(c.ground_body) &&
                          physics_body_get_type(cw.world, c.ground_body) != PHYSICS_BODY_STATIC;
  }
  char_v2_step_full(&cw, t, &c, in, dt, NULL, true);
  u32 platform = !c.ground_body_valid ? 0u : physics_body_get_type(cw.world, c.ground_body) == PHYSICS_BODY_STATIC ? 1u : 2u;
  if (platform != 2u)
    c.ground_body_pos = V3_ZERO, c.ground_body_rot = (quat){0};
  CharacterNet to = {.position = c.position,
                     .yaw = c.yaw,
                     .ground = (u32)c.ground,
                     .crouched = c.crouched ? 1u : 0u,
                     .swimming = c.swimming ? 1u : 0u,
                     .velocity = c.velocity,
                     .ground_normal = c.ground_normal,
                     .coyote = c.coyote,
                     .jump_buffer = c.jump_buffer,
                     .crouch_k = c.crouch_k,
                     .platform_pos = c.ground_body_pos,
                     .platform_rot = {c.ground_body_rot.x, c.ground_body_rot.y, c.ground_body_rot.z, c.ground_body_rot.w},
                     .platform = platform};
  *n = char_net_snapped(&to);
  // a body that moves may carry it next step without any input
  n->rest = idle && platform != 2u && char_net_same_state(n, &from);
}

// server, per character, parallel: the mover reads the query world, one-way marks and the ground's CharacterSurface
ecs_system(character_net_server_system, EcsInOut(CharacterNet), EcsInOut(CharacterInput), EcsResIn(CharacterNetTuning),
           EcsResIn(EcsPhysicsWorld), EcsExternIn(CharacterOneWay), EcsExternIn(CharacterSurface)) {
  const CharacterTuning *t = &ecs_sys_get_resource(CharacterNetTuning)->resolved;
  for (i32 i = 0; i < it->count; i++)
    character_net_step(it->world, t, ecs_sys_get_component_mut(CharacterNet, i), ecs_sys_get_component_mut(CharacterInput, i),
                       it->delta_time);
}

// client, per predicted character, parallel: from the tick after its prediction starts
ecs_system(character_net_predicted_system, EcsInOut(CharacterNet), EcsInOut(CharacterInput), EcsIn(NetPredicted),
           EcsResIn(NetSimTime), EcsResIn(CharacterNetTuning), EcsResIn(EcsPhysicsWorld), EcsExternIn(CharacterOneWay),
           EcsExternIn(CharacterSurface)) {
  NetTick tick = net_sim_tick(ecs_sys_get_resource(NetSimTime), it->step);
  const CharacterTuning *t = &ecs_sys_get_resource(CharacterNetTuning)->resolved;
  for (i32 i = 0; i < it->count; i++) {
    if (!net_predicted_simulate(ecs_sys_get_component(NetPredicted, i), tick))
      continue;
    character_net_step(it->world, t, ecs_sys_get_component_mut(CharacterNet, i), ecs_sys_get_component_mut(CharacterInput, i),
                       it->delta_time);
  }
}

void character_net_schedule(EcsWorld *w, b32 server) {
  if (server)
    ecs_schedule(w, character_net_server_system);
  else
    ecs_schedule(w, character_net_predicted_system);
}

EcsEntity character_spawn(EcsWorld *w, CString name, v3 feet, const CharacterTuning *tuning) {
  assert_msg(is_main_thread(), "character_spawn: main lane only"); // creates entities outside a scope
  CharacterTuning t = character_tuning_resolve(*tuning);
  EcsEntity e = ecs_entity_create(w);
  ecs_component_set(w, e, EntityName, {.name = fixedstr64_from_cstr((char *)name)});
  ecs_component_set(w, e, LocalTransform, {.position = feet, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, WorldTransform, {.position = feet, .rotation = quat_identity(), .scale = V3_ONE});
  ecs_component_set(w, e, CharacterTuning, *tuning);
  if (t.mover == CHARACTER_MOVER_RIGIDBODY) {
    if (!ecs_resource_get(w, EcsPhysicsWorld) || !ecs_resource_get(w, EcsRigidBodyWorld))
      log_error("character_spawn '%': a rigidbody character needs ecs_rigidbody_init and ecs_physics_init first; it will not move",
                fmt_cstr(name));
    ecs_component_set(w, e, ColliderBody,
                      {.collider = collider_capsule((v3){0.0f, t.radius, 0.0f}, (v3){0.0f, t.height - t.radius, 0.0f}, t.radius),
                       .pos = feet,
                       .rot = quat_identity(),
                       .scale = 1.0f});
    ecs_rigidbody_body_add(w, e, (RigidBody){.inverse_mass = 1.0f, .lock_rotation = true, .undamped = true});
  }
  return e;
}

void character_avatar_add(EcsWorld *w, EcsEntity character, const CharacterAvatarDesc *desc) {
  assert_msg(is_main_thread(), "character_avatar_add: main lane only"); // loads assets and creates entities outside a scope
  AssetSystem *assets = ecs_resource_get_mut(w, AssetSystem);
  b32 defaults = !desc->model_path;
  CString model = defaults ? CHAR_DEFAULT_MODEL : desc->model_path;
  quat rot = desc->local_rotation;
  if (quat_dot(rot, rot) < 1e-6f)
    rot = quat_identity(); // models face +Z, the character's forward
  v3 scale = v3_lengthsq(desc->local_scale) > 0.0f ? desc->local_scale : V3_ONE;

  CharacterAvatar av = {.run_lo = desc->run_speed_lo > 0.0f ? desc->run_speed_lo : 0.5f,
                        .run_hi = desc->run_speed_hi > 0.0f ? desc->run_speed_hi : 4.0f};
  HzHandleT(ModelBlobAsset) model_h = asset_load(ModelBlobAsset, assets, .path = model);
  av.idle = asset_load(AnimationBlobAsset, assets, .path = defaults ? CHAR_DEFAULT_IDLE : desc->idle_clip_path);
  av.run = asset_load(AnimationBlobAsset, assets, .path = defaults ? CHAR_DEFAULT_RUN : desc->run_clip_path);
  av.fall = asset_load(AnimationBlobAsset, assets, .path = defaults ? CHAR_DEFAULT_FALL : desc->fall_clip_path);

  // a rigidbody character's body never rotates: the avatar hangs under a pivot that character_pose_system turns
  EcsEntity parent = character;
  const CharacterTuning *tuning = ecs_component_get(w, character, CharacterTuning);
  const CharacterVisual *visual = ecs_component_get(w, character, CharacterVisual);
  if (visual && visual->entity) {
    parent = visual->entity;
  } else if (tuning && tuning->mover == CHARACTER_MOVER_RIGIDBODY) {
    const WorldTransform *wt = ecs_component_get(w, character, WorldTransform);
    v3 at = wt ? wt->position : V3_ZERO;
    parent = ecs_entity_create(w);
    ecs_component_set(w, parent, LocalTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
    ecs_component_set(w, parent, WorldTransform, {.position = at, .rotation = quat_identity(), .scale = V3_ONE});
    ecs_component_set(w, character, CharacterVisual, {.entity = parent});
  }

  av.avatar = ecs_entity_create(w);
  ecs_component_set(w, av.avatar, EcsParent, {.parent = parent});
  ecs_component_set(w, av.avatar, LocalTransform, {.position = desc->local_offset, .rotation = rot, .scale = scale});
  ecs_component_set(w, av.avatar, EcsModelRef, {.model = blob_asset_ref_from_handle(ModelBlobAsset, model_h)});
  ecs_component_set(w, av.avatar, EcsAnimationRef, {.animation = blob_asset_ref_from_handle(AnimationBlobAsset, av.idle)});
  ecs_component_set(w, av.avatar, EcsVisibility, {.visible = true, .casts_shadow = true});
  ecs_component_set(w, character, CharacterAvatar, av);
}
