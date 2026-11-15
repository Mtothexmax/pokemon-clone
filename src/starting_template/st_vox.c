// Your Voxelmon and the real-time battles in the tall grass (three.js src/lib/three/voxelmon.ts): the card you raised
// to its final stage walks behind you as a 3D voxel beast; in the grass wild ones jump out, yours fights with its two
// skills (keys 1 and 2, they recharge) while the wild one lunges on its own. Positions are in studs.
// Included by starting_template.c above app_init, after st_cardgame.c.

#define VOX_CUBES 10
#define VOX_NUMBERS 8

typedef enum SkillKind { SK_DASH, SK_VOLLEY, SK_WAVE, SK_STRIKE } SkillKind;
typedef struct Skill {
  CString name;
  SkillKind kind;
  f32 power, cooldown;
  u32 color;
} Skill;

hz_internal Skill vox_skill(i32 id, i32 k) {
  if (id == CARD_TITANOAK)
    return k ? (Skill){"Leaf Storm", SK_VOLLEY, 30, 7.0f, 0x58ab41u} : (Skill){"Root Slam", SK_DASH, 14, 2.2f, 0x8a5a2bu};
  if (id == CARD_STORMAW)
    return k ? (Skill){"Storm Call", SK_STRIKE, 32, 7.5f, 0xffe14du} : (Skill){"Thunder Bite", SK_DASH, 13, 2.0f, 0xffd23du};
  if (id == CARD_ABYSSQUID)
    return k ? (Skill){"Tidal Whirl", SK_WAVE, 30, 7.0f, 0x3fa9f5u} : (Skill){"Ink Shot", SK_VOLLEY, 12, 2.2f, 0x6a4c93u};
  return k ? (Skill){"Brick Blast", SK_VOLLEY, 22, 6.0f, 0xf2cd37u} : (Skill){"Tackle", SK_DASH, 10, 2.0f, 0xffffffu};
}

typedef enum ActKind { ACT_IDLE, ACT_LUNGE, ACT_DASH } ActKind;

typedef struct Fighter {
  i32 id; // card, -1 none
  EcsEntity e;
  v3 p, home;
  f32 yaw, size; // size: studs high
  f32 hp, max;
  b32 shiny, tired;
  ActKind act;
  f32 act_t, dmg;
  b32 act_hit;
  f32 next_atk;
  v3 knock;
  f32 squash; // a hit squashes it for a moment
} Fighter;

typedef struct VoxProj {
  b32 live;
  i32 kind; // 0 cube, 1 ring, 2 mark, 3 bolt
  f32 t0, dur, dmg, arc;
  v3 from;
  b32 mine;
  EcsEntity e;
} VoxProj;

typedef struct VoxNumber {
  v3 p;
  f32 t;
  i32 n;
  b32 mine; // damage dealt by your beast
} VoxNumber;

typedef enum VoxPhase { VOX_NONE, VOX_INTRO, VOX_FIGHT, VOX_END } VoxPhase;

typedef struct Vox {
  Life *l;
  EcsEntity models[CARD_COUNT]; // spawned on first use, hidden when not needed
  Fighter me, wild;
  i32 me_id;   // the companion shown (card id, -1 none)
  i32 me_area; // where it was last (a door or stairs: it comes along at once)
  VoxPhase phase;
  f32 phase_t, now;
  f32 cool[2];
  VoxProj proj[VOX_CUBES + 3];
  HzHandleT(RendererSubMesh) cube_mesh, ring_mesh, bolt_mesh;
  HzHandleT(RendererMaterial) mat[4];
  VoxNumber num[VOX_NUMBERS];
  i32 num_i;
  f32 meter, rustle_cd;
  b32 hide_me; // the camera is at your eyes (a close-up while talking, a sign): your Voxelmon steps out of the view
  v3 last_p;
  v2 face; // where the player walks
  i32 result; // 0 win 1 lose 2 fled (VOX_END)
  v3 center;  // the arena
  v2 walk;    // the player steps back to see the fight: a world direction, 0..1
  f32 reward_at; i32 reward_card; b32 reward_shiny;
  char text[6][96];
  b32 click_skill[2], click_run;
  u32 seed;
  b32 force; // dev builds (F6): a wild one now
  // the upgrade to a Voxelmon: the beast shown rising in front of the camera (vox_showcase; -1 none)
  i32 show_id;
  EcsEntity show_ring[2];
  HzHandleT(RendererMaterial) show_mat;
  // the fight camera (vox_camera): off the player, onto the middle between the two beasts
  b32 cam_on;
  f32 cam_t;        // 0 the rig's camera .. 1 the fight's
  v3 cam_from;      // where the rig had it (metres) and how it looked
  quat cam_from_rot;
  v3 cam_pos, cam_look; // the fight view, eased towards its target every frame
  f32 cam_side;     // which side of the two it watches from (+1 / -1), kept for the fight
} Vox;

hz_internal f32 vox_rand(Vox *v) {
  v->seed = v->seed * 1664525u + 1013904223u;
  return (f32)(v->seed >> 8) / 16777216.0f;
}

hz_internal void shape_vcube(Town *t) { town_box(t, C_WHITE, 0.6f, 0.6f, 0.6f, 0.0f, -0.3f, 0.0f, false); }
hz_internal void shape_vring(Town *t) {
  MeshBuilderStyle st = town_style(C_WHITE);
  st.flat = false;
  mesh_builder_torus(t->b, &st, m4_trs(V3_ZERO, quat_from_euler((v3){M_PI * 0.5f, 0.0f, 0.0f}), V3_ONE),
                     &(PgTorusDesc){.radius = 1.0f * TOWN_STUD, .tube = 0.18f * TOWN_STUD, .radial_segments = 6, .tubular_segments = 24});
}
hz_internal void shape_vbolt(Town *t) { town_box(t, C_WHITE, 0.5f, 12.0f, 0.5f, 0.0f, 0.0f, 0.0f, false); }

hz_internal v4 vox_rgb(u32 hex) { return (v4){(f32)((hex >> 16) & 255) / 255.0f, (f32)((hex >> 8) & 255) / 255.0f, (f32)(hex & 255) / 255.0f, 1.0f}; }

hz_internal void vox_init(Vox *v, Life *l, u32 seed) {
  v->l = l;
  v->seed = seed | 1u;
  v->me_id = -1;
  v->me_area = -2;
  v->me.id = v->wild.id = -1;
  v->show_id = -1;
  v->cube_mesh = life_mesh(l, shape_vcube);
  v->ring_mesh = life_mesh(l, shape_vring);
  v->bolt_mesh = life_mesh(l, shape_vbolt);
  for (i32 i = 0; i < 4; i++)
    v->mat[i] = life_glow_material(l, (v4){1, 1, 1, 1}, 2.0f);
  v->show_mat = life_glow_material(l, (v4){0.68f, 0.45f, 1.0f, 1.0f}, 3.0f);
  for (i32 i = 0; i < 2; i++)
    v->show_ring[i] = life_spawn(l, v->ring_mesh, v->show_mat, (v3){0.0f, -60.0f, 0.0f}, false);
  for (i32 i = 0; i < VOX_CUBES + 3; i++) {
    HzHandleT(RendererSubMesh) m = i < VOX_CUBES ? v->cube_mesh : i == VOX_CUBES + 2 ? v->bolt_mesh : v->ring_mesh;
    v->proj[i].e = life_spawn(l, m, v->mat[i < VOX_CUBES ? 0 : i - VOX_CUBES + 1], (v3){0.0f, -60.0f, 0.0f}, false);
  }
}

hz_internal EcsEntity vox_model(Vox *v, i32 id) {
  if (!v->models[id]) {
    char path[96];
    cstr_format(path, sizeof(path), "brickhollow/beasts/%.glb", fmt_cstr(card_def(id).id));
    AssetSystem *assets = ecs_resource_get_mut(v->l->w, AssetSystem);
    HzHandleT(ModelBlobAsset) model = asset_load(ModelBlobAsset, assets, .path = path);
    v->models[id] = ecs_renderer_spawn(v->l->w, v->l->r,
                                       &(EcsRendererSpawnDesc){.model_asset = blob_asset_ref_from_handle(ModelBlobAsset, model),
                                                               .transform = {.position = {0.0f, -60.0f, 0.0f}, .rotation = quat_identity(), .scale = V3_ONE},
                                                               .casts_shadow = true});
  }
  return v->models[id];
}

hz_internal void vox_hide(Vox *v, EcsEntity e) {
  if (e)
    life_set(v->l, e, (v3){0.0f, -150.0f, 0.0f}, 0.0f, (v3){0.01f, 0.01f, 0.01f});
}

// the fighter's model: knocked over when tired, squashed when hit
hz_internal void vox_pose(Vox *v, const Fighter *f, f32 grow) {
  if (!f->e)
    return;
  f32 s = f->size * TOWN_STUD * grow;
  f32 sq = 1.0f - f->squash * 0.25f;
  quat q = quat_from_euler((v3){0.0f, f->yaw, f->tired ? M_PI / 2.4f : 0.0f});
  ecs_renderer_set_transform(v->l->w, f->e, (Transform){.position = town_m(f->p.x + f->knock.x, f->p.y, f->p.z + f->knock.z), .rotation = q,
                                                        .scale = {s * (2.0f - sq), s * sq, s * (2.0f - sq)}});
}

hz_internal f32 vox_face_to(v3 from, v3 to) { return m_atan2f(to.x - from.x, to.z - from.z); }

hz_internal f32 vox_turn(f32 a, f32 b, f32 k) {
  f32 d = m_fmodf(b - a + M_PI * 3.0f, M_PI * 2.0f) - M_PI;
  return a + d * m_minf(1.0f, k);
}

hz_internal void vox_move_to(Fighter *f, v3 to, f32 dt, f32 speed) {
  f32 dx = to.x - f->p.x, dz = to.z - f->p.z, d = m_sqrtf(dx * dx + dz * dz);
  if (d < 0.05f)
    return;
  f32 s = m_minf(d, speed * dt);
  f->p.x += dx / d * s;
  f->p.z += dz / d * s;
}

// any patch of tall grass (the village's and the three up Route 1: g_town_grass, st_north.c)
hz_internal b32 vox_in_grass(f32 x, f32 z) { return town_grass_at(x, z) >= 0; }

// ---- the fight ----
hz_internal void vox_end(Vox *v, i32 result);

hz_internal void vox_damage(Vox *v, Fighter *f, f32 dmg, b32 by_me) {
  if (v->phase != VOX_FIGHT)
    return;
  i32 n = MAX(1, (i32)m_roundf(dmg));
  f->hp -= (f32)n;
  f->squash = 1.0f;
  Fighter *src = by_me ? &v->me : &v->wild;
  v3 away = {f->p.x - src->p.x, 0.0f, f->p.z - src->p.z};
  f32 len = m_maxf(0.01f, v3_length(away));
  f->knock = v3_scale(away, 0.6f / len);
  VoxNumber *num = &v->num[v->num_i++ % VOX_NUMBERS];
  *num = (VoxNumber){.p = {f->p.x, f->p.y + f->size + 0.6f, f->p.z}, .t = 0.0f, .n = n, .mine = by_me};
  sound_play(v->l->snd, SND_KICK, by_me ? 1.2f : 0.85f, 0.7f);
  if (f->hp <= 0.0f) {
    f->hp = 0.0f;
    vox_end(v, by_me ? 0 : 1);
  }
}

hz_internal void vox_proj(Vox *v, i32 kind, i32 slot, f32 delay, f32 dur, f32 dmg, b32 mine, u32 color) {
  VoxProj *p = &v->proj[slot];
  Fighter *from = mine ? &v->me : &v->wild;
  *p = (VoxProj){.live = true, .kind = kind, .t0 = v->now + delay, .dur = dur, .dmg = dmg, .mine = mine, .e = p->e,
                 .from = {from->p.x + (vox_rand(v) - 0.5f) * 1.2f, from->p.y + from->size * 0.6f + vox_rand(v), from->p.z + (vox_rand(v) - 0.5f) * 1.2f},
                 .arc = 1.5f + vox_rand(v)};
  material_set_vec4(v->l->r, v->mat[kind == 0 ? 0 : kind == 3 ? 3 : kind == 1 ? 1 : 2], "emission_color", vox_rgb(color));
}

hz_internal void vox_use_skill(Vox *v, i32 k) {
  if (v->phase != VOX_FIGHT || v->cool[k] > 0.0f || v->me.act != ACT_IDLE)
    return;
  Skill sk = vox_skill(v->me.id, k);
  v->cool[k] = sk.cooldown;
  sound_play(v->l->snd, SND_POP, 0.8f + 0.1f * (f32)card_def(v->me.id).cost * 0.1f, 0.7f);
  switch (sk.kind) {
  case SK_DASH:
    v->me.act = ACT_DASH;
    v->me.act_t = 0.0f;
    v->me.act_hit = false;
    v->me.dmg = sk.power;
    break;
  case SK_VOLLEY:
    for (i32 i = 0; i < 5; i++) {
      i32 slot = -1;
      for (i32 s = 0; s < VOX_CUBES && slot < 0; s++)
        if (!v->proj[s].live)
          slot = s;
      if (slot >= 0)
        vox_proj(v, 0, slot, (f32)i * 0.09f, 0.45f, sk.power / 5.0f, true, sk.color);
    }
    break;
  case SK_WAVE: vox_proj(v, 1, VOX_CUBES, 0.0f, 0.55f, sk.power, true, sk.color); break;
  case SK_STRIKE:
    vox_proj(v, 2, VOX_CUBES + 1, 0.0f, 0.5f, 0.0f, true, sk.color);
    vox_proj(v, 3, VOX_CUBES + 2, 0.5f, 0.25f, sk.power, true, sk.color);
    break;
  }
}

// a lunge (the wild one) or a dash (your Tackle): wind up, leap at the foe, hit, hop back
hz_internal void vox_run_act(Vox *v, Fighter *f, Fighter *foe, f32 dt, b32 mine) {
  if (f->act == ACT_IDLE)
    return;
  f->act_t += dt;
  f32 wind = f->act == ACT_LUNGE ? 0.55f : 0.18f, leap = 0.16f, back = 0.34f;
  v3 to = {foe->p.x - f->p.x, 0.0f, foe->p.z - f->p.z};
  f32 d = m_maxf(0.01f, v3_length(to));
  v3 dir = v3_scale(to, 1.0f / d);
  f32 reach = (foe->size + f->size) * 0.3f;
  f->yaw = vox_turn(f->yaw, vox_face_to(f->p, foe->p), dt * 14.0f);
  f32 t = f->act_t;
  if (t < wind) { // lean back
    v3 base = f->home;
    f32 k = t / wind;
    f->p.x = base.x - dir.x * k * 0.8f;
    f->p.z = base.z - dir.z * k * 0.8f;
    f->p.y = base.y;
  } else if (t < wind + leap) {
    f32 k = (t - wind) / leap;
    v3 hit = {foe->p.x - dir.x * reach, f->home.y, foe->p.z - dir.z * reach};
    f->p = v3_lerp(f->home, hit, k);
    f->p.y = f->home.y + m_sinf(k * M_PI) * 1.2f;
  } else {
    if (!f->act_hit) {
      f->act_hit = true;
      f32 dmg = f->act == ACT_LUNGE ? m_roundf((6.0f + (f32)card_def(f->id).cost * 1.6f) * (f->shiny ? 1.25f : 1.0f) * (0.85f + vox_rand(v) * 0.3f))
                                    : f->dmg;
      vox_damage(v, foe, dmg, mine);
    }
    f32 k = m_minf(1.0f, (t - wind - leap) / back);
    v3 hit = {foe->p.x - dir.x * reach, f->home.y, foe->p.z - dir.z * reach};
    f->p = v3_lerp(hit, f->home, k);
    f->p.y = f->home.y + m_sinf(k * M_PI) * 0.6f;
    if (k >= 1.0f) {
      f->act = ACT_IDLE;
      if (!mine)
        f->next_atk = v->now + 2.2f + vox_rand(v) * 1.4f;
    }
  }
}

hz_internal void vox_start(Vox *v, v3 player, f32 heading, CardGame *cg, EcsWorld *w, AudioSystem *audio) {
  // the pool: cost 5 and below, cheaper ones more often
  f32 total = 0.0f;
  for (i32 c = 0; c < CARD_POOL; c++)
    if (card_def(c).cost <= 5)
      total += (f32)(6 - card_def(c).cost);
  f32 r = vox_rand(v) * total;
  i32 id = 0;
  for (i32 c = 0; c < CARD_POOL; c++) {
    if (card_def(c).cost > 5)
      continue;
    r -= (f32)(6 - card_def(c).cost);
    if (r <= 0.0f) {
      id = c;
      break;
    }
  }
  b32 shiny = vox_rand(v) < 1.0f / 12.0f;
  i32 cost = card_def(id).cost;
  v2 fwd = {-m_sinf(heading), -m_cosf(heading)}; // where the camera looks: the fight is in front of you
  v2 side = {-fwd.y, fwd.x};
  v3 c = {player.x + fwd.x * 7.0f, player.y, player.z + fwd.y * 7.0f};
  { // the fight stays in the patch you stand in
    i32 gi = town_grass_at(player.x, player.z);
    TownRect g = g_town_grass[gi < 0 ? 0 : gi];
    c.x = m_clampf(c.x, g.x0 + 3.6f, g.x1 - 3.6f);
    c.z = m_clampf(c.z, g.z0 + 3.6f, g.z1 - 2.0f);
  }
  Fighter *wf = &v->wild;
  vox_hide(v, wf->e);
  *wf = (Fighter){.id = id, .e = vox_model(v, id), .size = 2.4f + (f32)cost * 0.25f, .shiny = shiny,
                  .max = m_roundf((40.0f + (f32)cost * 13.0f) * (shiny ? 1.4f : 1.0f))};
  wf->hp = wf->max;
  wf->home = wf->p = (v3){c.x + side.x * 3.4f, c.y, c.z + side.y * 3.4f};
  v->me.home = (v3){c.x - side.x * 3.4f, c.y, c.z - side.y * 3.4f};
  v->center = c;
  v->me.act = ACT_IDLE;
  wf->yaw = vox_face_to(wf->p, v->me.home);
  v->phase = VOX_INTRO;
  v->phase_t = 0.0f;
  v->cool[0] = v->cool[1] = 0.0f;
  if (shiny) {
    sound_play(v->l->snd, SND_FANFARE, 1.0f, 0.8f);
    LIFE_TOAST(v->l, "A SHINY wild % appeared!", fmt_cstr(card_def(id).name));
  } else {
    sound_play(v->l->snd, SND_ALERT, 1.0f, 0.7f);
    LIFE_TOAST(v->l, "A wild % jumped out of the grass!", fmt_cstr(card_def(id).name));
  }
  cg_music(cg, w, audio, MUSIC_FIGHT);
}

hz_internal void vox_end(Vox *v, i32 result) {
  v->phase = VOX_END;
  v->phase_t = 0.0f;
  v->result = result;
  v->me.act = v->wild.act = ACT_IDLE;
  for (i32 i = 0; i < VOX_CUBES + 3; i++)
    if (v->proj[i].live) {
      v->proj[i].live = false;
      vox_hide(v, v->proj[i].e);
    }
  if (result == 1)
    v->me.tired = true;
}

// the upgrade to a Voxelmon (up_draw, st_cardui.c): once the card has burst, the beast rises out of two glowing rings in
// front of the player (between it and the camera), spinning and growing, and keeps turning there until the moment is
// closed (id -1: put away)
hz_internal void vox_showcase(Vox *v, i32 id, f32 t, const Camera *cam, v3 player) {
  if (id < 0 || t < UP_BURST || !cam) {
    if (v->show_id >= 0) {
      if (v->show_id != v->me.id && v->show_id != v->wild.id)
        vox_hide(v, v->models[v->show_id]);
      for (i32 i = 0; i < 2; i++)
        vox_hide(v, v->show_ring[i]);
      v->show_id = -1;
    }
    return;
  }
  v->show_id = id;
  EcsEntity e = vox_model(v, id);
  v3 cs = v3_scale(cam->pos, 1.0f / TOWN_STUD); // studs
  v2 back = v2_normalize((v2){cs.x - player.x, cs.z - player.z}); // from the player towards the camera
  v3 at = {player.x + back.x * 4.0f, player.y, player.z + back.y * 4.0f};
  f32 k = m_clampf((t - UP_BURST) / 1.2f, 0.0f, 1.0f), grow = 1.0f - (1.0f - k) * (1.0f - k) * (1.0f - k);
  f32 face = m_atan2f(cs.x - at.x, cs.z - at.z);
  f32 yaw = face + (t - UP_BURST) * 0.9f + (1.0f - grow) * 6.0f;
  f32 size = 6.0f * TOWN_STUD * m_maxf(grow, 0.01f);
  f32 y = at.y + (1.0f - grow) * -1.5f + m_absf(m_sinf(t * 2.0f)) * 0.25f * k;
  ecs_renderer_set_transform(v->l->w, e, (Transform){.position = town_m(at.x, y, at.z), .rotation = quat_from_euler((v3){0.0f, yaw, 0.0f}),
                                                     .scale = {size, size, size}});
  for (i32 i = 0; i < 2; i++) {
    f32 r = (i == 0 ? 3.4f : 4.6f + 0.3f * m_sinf(t * 3.0f)) * m_minf(1.0f, k * 1.6f);
    life_set(v->l, v->show_ring[i], (v3){at.x, at.y + 0.1f + (f32)i * 0.05f, at.z}, t * (i ? -1.2f : 1.6f), (v3){r, 1.0f, r});
  }
}

// main lane, before the scope: during a fight in the tall grass the camera leaves the player and watches the middle
// between the two beasts from the side away from the player (so the figure is not in the way: it is seen behind the
// fight, watching), far enough for both; it eases there and back to the rig afterwards. Returns true while it owns it
hz_internal b32 vox_camera(Vox *v, EcsWorld *w, EcsQuery *camera_query, v3 player_m, f32 dt) {
  EcsEntity cam_e = ecs_query_get_singleton_entity(camera_query);
  if (cam_e == ECS_ENTITY_INVALID)
    return false;
  const Camera *cam = ecs_component_get(w, cam_e, Camera);
  const CameraFollow *follow = ecs_component_get(w, cam_e, CameraFollow);
  if (!cam || !follow)
    return false;
  b32 want = v->phase != VOX_NONE && v->me.id >= 0 && v->wild.id >= 0;
  // the view's direction from the arena (the spots both beasts fight from: steady while they dash about), its centre
  // from where they are now
  v3 ha = town_m(v->me.home.x, v->me.home.y, v->me.home.z), hb = town_m(v->wild.home.x, v->wild.home.y, v->wild.home.z);
  v3 a = town_m(v->me.p.x, v->me.p.y, v->me.p.z), b = town_m(v->wild.p.x, v->wild.p.y, v->wild.p.z);
  v3 mid = v3_scale(v3_add(a, b), 0.5f), hmid = v3_scale(v3_add(ha, hb), 0.5f);
  v2 axis = {hb.x - ha.x, hb.z - ha.z};
  f32 hs = m_maxf(v2_length(axis), 1.0f);
  v2 n = {-axis.y / hs, axis.x / hs}; // across the line between them
  f32 sep = m_maxf(v2_length((v2){b.x - a.x, b.z - a.z}), hs);
  if (want && !v->cam_on) { // start from where the rig has the camera; watch from its side
    v->cam_on = true;
    v->cam_t = 0.0f;
    v->cam_from = cam->pos;
    v->cam_from_rot = cam->rot;
    v->cam_side = (player_m.x - hmid.x) * n.x + (player_m.z - hmid.z) * n.y >= 0.0f ? -1.0f : 1.0f;
    v->cam_pos = cam->pos;
    v->cam_look = mid;
    CameraFollow f = *follow;
    f.disabled = true;
    ecs_component_set(w, cam_e, CameraFollow, f);
  }
  if (!v->cam_on)
    return false;
  v->cam_t = m_clampf(v->cam_t + (want ? dt : -dt) / 0.9f, 0.0f, 1.0f);
  if (v->cam_t <= 0.0f) { // back: the rig takes over again
    v->cam_on = false;
    CameraFollow f = *follow;
    f.disabled = false;
    f.snap = true;
    ecs_component_set(w, cam_e, CameraFollow, f);
    return false;
  }
  // the view: from the side, a little above, far enough that both beasts (and the space between) fit
  f32 dist = m_maxf(sep * 0.8f + 1.6f, 5.0f);
  v3 look = v3_add(mid, (v3){0.0f, 0.6f, 0.0f});
  v3 pos = v3_add(look, (v3){n.x * v->cam_side * dist, dist * 0.72f, n.y * v->cam_side * dist}); // above the tall grass
  f32 kf = 1.0f - m_expf(-dt * 3.0f); // eases after the beasts as they move
  if (want) {
    v->cam_pos = v3_lerp(v->cam_pos, pos, kf);
    v->cam_look = v3_lerp(v->cam_look, look, kf);
  }
  f32 k = v->cam_t * v->cam_t * (3.0f - 2.0f * v->cam_t);
  Camera c = *cam;
  c.pos = v3_lerp(v->cam_from, want ? pos : v->cam_pos, k);
  c.rot = quat_slerp(v->cam_from_rot, quat_look(v3_sub(c.pos, v->cam_look), V3_UP), k);
  ecs_component_set(w, cam_e, Camera, c);
  return true;
}

// main lane, before the scope. Returns true while a fight holds the player in place
hz_internal b32 vox_update(Vox *v, SaveState *save, CardGame *cg, EcsWorld *w, AudioSystem *audio, const InputSystem *in, v3 player_m,
                           f32 heading, i32 area, b32 busy, f32 dt) {
  GameSave *s = &save->data;
  v->now += dt;
  v3 p = v3_scale(player_m, 1.0f / TOWN_STUD);
  v3 step = v3_sub(p, v->last_p);
  step.y = 0.0f;
  f32 moved = v3_length(step);
  if (moved > 0.01f && moved < 3.0f)
    v->face = v2_lerp(v->face, (v2){step.x / moved, step.z / moved}, m_minf(1.0f, dt * 8.0f));
  v->last_p = p;
  if (v->rustle_cd > 0.0f)
    v->rustle_cd -= dt;
  for (i32 i = 0; i < VOX_NUMBERS; i++)
    v->num[i].t += dt;
  // the companion: the card you picked in the deck menu
  i32 want = s->voxelmon > 0 && s->card_level[s->voxelmon - 1] >= VOXEL_LEVEL ? s->voxelmon - 1 : -1;
  if (want != v->me_id && v->phase == VOX_NONE) {
    vox_hide(v, v->me.e);
    v->me_id = want;
    v->me = (Fighter){.id = want};
    if (want >= 0) {
      CardDef d = card_def(want);
      v->me.e = vox_model(v, want);
      v->me.size = 4.2f;
      v->me.max = v->me.hp = 90.0f + (f32)d.power * 3.0f;
      v->me.p = (v3){p.x + 2.0f, p.y, p.z + 3.0f};
      v->me_area = area;
    }
  }
  Fighter *me = &v->me;
  if (me->id < 0)
    return false;
  // hits fade
  me->squash = m_maxf(0.0f, me->squash - dt * 7.0f);
  me->knock = v3_scale(me->knock, m_maxf(0.0f, 1.0f - dt * 8.0f));
  v->wild.squash = m_maxf(0.0f, v->wild.squash - dt * 7.0f);
  v->wild.knock = v3_scale(v->wild.knock, m_maxf(0.0f, 1.0f - dt * 8.0f));
  if (v->phase == VOX_NONE) {
    // rest: HP comes back, standing up again above half
    me->hp = m_minf(me->max, me->hp + me->max / 90.0f * dt);
    if (me->tired && me->hp > me->max * 0.5f)
      me->tired = false;
    // follow: 3 studs behind you, 1.6 to the side; through a door it comes along at once
    v2 f = v2_length(v->face) > 0.1f ? v2_normalize(v->face) : (v2){0.0f, -1.0f};
    v3 target = {p.x - f.x * 3.0f + f.y * 1.6f, p.y, p.z - f.y * 3.0f - f.x * 1.6f};
    f32 dist = v3_length(v3_sub(target, me->p));
    if (dist > 20.0f || area != v->me_area) {
      me->p = target;
      v->me_area = area;
    }
    f32 speed = m_minf(dist * 3.0f, me->tired ? 6.0f : 11.0f) * STUD_PER_THREE;
    v3 before = me->p;
    vox_move_to(me, target, dt, speed);
    f32 sp = v3_length(v3_sub(me->p, before)) / m_maxf(dt, 1e-4f);
    me->p.y = m_lerpf(me->p.y, p.y, m_minf(1.0f, dt * 10.0f)) ;
    f32 ground = me->p.y;
    if (sp > 0.5f) {
      me->yaw = vox_turn(me->yaw, m_atan2f(me->p.x - before.x, me->p.z - before.z), dt * 10.0f);
      ground += m_absf(m_sinf(v->now * 10.0f)) * 0.45f;
    } else
      me->yaw = vox_turn(me->yaw, vox_face_to(me->p, p), dt * 3.0f);
    Fighter shown = *me;
    shown.p.y = ground;
    if (v->hide_me)
      vox_hide(v, me->e);
    else if (v->show_id != me->id) // (rising as the new Voxelmon: vox_showcase poses it)
      vox_pose(v, &shown, 1.0f);
    if (v->force) {
      v->force = false;
      if (!me->tired && !busy)
        vox_start(v, p, heading, cg, w, audio);
    }
    // the grass: every 3 studs walked in it a 22% chance
    if (!busy && area < 0 && vox_in_grass(p.x, p.z) && moved < 3.0f) {
      v->meter += moved;
      if (v->meter >= 3.0f) {
        v->meter -= 3.0f;
        if (vox_rand(v) < 0.22f) {
          if (!me->tired)
            vox_start(v, p, heading, cg, w, audio);
          else if (v->rustle_cd <= 0.0f) {
            v->rustle_cd = 6.0f;
            LIFE_TOAST(v->l, "The grass rustles\xe2\x80\xa6 but % is too tired to fight. Let it rest!", fmt_cstr(card_def(me->id).name));
          }
        }
      }
    }
    return false;
  }
  Fighter *wf = &v->wild;
  v->phase_t += dt;
  // you back off to 8.5 studs from the arena's centre, so both beasts stand in front of the camera
  v->walk = (v2){0};
  if (v->phase != VOX_END) {
    v2 off = {p.x - v->center.x, p.z - v->center.z};
    f32 d = v2_length(off);
    if (d < 8.0f) {
      v2 away = d > 0.1f ? v2_scale(off, 1.0f / d) : (v2){m_sinf(heading), m_cosf(heading)};
      v->walk = v2_scale(away, 0.5f);
    }
  }
  f32 t = v->phase_t;
  if (v->phase == VOX_INTRO) {
    f32 k = m_minf(1.0f, t / 0.5f);
    f32 grow = m_maxf(0.01f, k < 1.0f ? k * 1.15f : 1.0f);
    wf->p.y = wf->home.y + m_sinf(m_minf(1.0f, t / 0.7f) * M_PI) * 1.6f;
    vox_move_to(me, me->home, dt, 14.0f);
    me->yaw = vox_turn(me->yaw, vox_face_to(me->p, wf->p), dt * 10.0f);
    wf->yaw = vox_face_to(wf->p, me->p);
    vox_pose(v, wf, grow);
    vox_pose(v, me, 1.0f);
    if (t > 1.1f) {
      v->phase = VOX_FIGHT;
      v->phase_t = 0.0f;
      wf->next_atk = v->now + 1.6f;
    }
    return true;
  }
  if (v->phase == VOX_END) {
    if (t > 1.6f) {
      v->phase = VOX_NONE;
      vox_hide(v, wf->e);
      wf->e = 0;
      wf->id = -1;
      cg_music(cg, w, audio, MUSIC_HOME);
      me->home = me->p;
      me->p.y = p.y;
    } else if (v->result != 0) { // it hops away and shrinks
      f32 sc = m_maxf(0.01f, 1.0f - t / 0.9f);
      Fighter shown = *wf;
      shown.p.y = wf->home.y + m_absf(m_sinf(t * 9.0f)) * 0.8f;
      vox_pose(v, &shown, sc);
    } else { // won: it bursts, yours hops happily
      vox_hide(v, wf->e);
      Fighter shown = *me;
      shown.p.y = me->home.y + m_absf(m_sinf(t * 10.0f)) * 0.7f;
      if (v->show_id != me->id) // (rising as the new Voxelmon: vox_showcase poses it)
      vox_pose(v, &shown, 1.0f);
    }
    if (v->result == 1)
      vox_pose(v, me, 1.0f);
    return t <= 1.6f;
  }
  // ---- fight ----
  for (i32 k = 0; k < 2; k++)
    v->cool[k] = m_maxf(0.0f, v->cool[k] - dt);
  if (!busy) {
    if (in->buttons[KEY_1].pressed_this_frame || v->click_skill[0])
      vox_use_skill(v, 0);
    if (in->buttons[KEY_2].pressed_this_frame || v->click_skill[1])
      vox_use_skill(v, 1);
    if (v->click_run || in->buttons[KEY_ESCAPE].pressed_this_frame) {
      life_toast(v->l, "Got away safely!");
      vox_end(v, 2);
    }
  }
  v->click_skill[0] = v->click_skill[1] = v->click_run = false;
  if (v->phase != VOX_FIGHT)
    return true;
  vox_run_act(v, me, wf, dt, true);
  if (v->phase == VOX_FIGHT)
    vox_run_act(v, wf, me, dt, false);
  if (me->act == ACT_IDLE) {
    vox_move_to(me, me->home, dt, 8.0f);
    me->yaw = vox_turn(me->yaw, vox_face_to(me->p, wf->p), dt * 10.0f);
    me->p.y = me->home.y + m_absf(m_sinf(v->now * 5.0f)) * 0.15f;
  }
  if (wf->act == ACT_IDLE) {
    vox_move_to(wf, wf->home, dt, 8.0f);
    wf->yaw = vox_turn(wf->yaw, vox_face_to(wf->p, me->p), dt * 10.0f);
    wf->p.y = wf->home.y + m_absf(m_sinf(v->now * 4.0f + 1.0f)) * 0.12f;
    if (v->now > wf->next_atk && v->phase == VOX_FIGHT) {
      wf->act = ACT_LUNGE;
      wf->act_t = 0.0f;
      wf->act_hit = false;
    }
  }
  // projectiles
  for (i32 i = 0; i < VOX_CUBES + 3 && v->phase == VOX_FIGHT; i++) {
    VoxProj *pr = &v->proj[i];
    if (!pr->live)
      continue;
    f32 k = (v->now - pr->t0) / pr->dur;
    if (k < 0.0f)
      continue;
    Fighter *foe = pr->mine ? wf : me;
    v3 to = {foe->p.x, foe->p.y + foe->size * 0.45f, foe->p.z};
    if (pr->kind == 0) {
      v3 q = v3_lerp(pr->from, to, m_minf(k, 1.0f));
      q.y += m_sinf(m_minf(k, 1.0f) * M_PI) * pr->arc;
      ecs_renderer_set_transform(w, pr->e, (Transform){.position = town_m(q.x, q.y, q.z),
                                                       .rotation = quat_from_euler((v3){k * 9.0f, k * 7.0f, 0.0f}), .scale = V3_ONE});
    } else if (pr->kind == 1) {
      v3 a = {me->p.x, me->p.y + 0.6f, me->p.z};
      v3 q = v3_lerp(a, to, m_minf(k, 1.0f));
      f32 sc = 1.0f + k * 1.6f;
      life_set(v->l, pr->e, (v3){q.x, a.y, q.z}, vox_face_to(me->p, wf->p), (v3){sc, sc, sc});
    } else if (pr->kind == 2) {
      f32 sc = 2.6f - k * 1.6f;
      life_set(v->l, pr->e, (v3){foe->p.x, foe->p.y + 0.1f, foe->p.z}, v->now * 3.0f, (v3){sc, 0.4f, sc});
    } else {
      life_set(v->l, pr->e, (v3){foe->p.x, foe->p.y + 6.0f, foe->p.z}, 0.0f, (v3){1.0f + k, 1.0f, 1.0f + k});
    }
    if (k >= 1.0f) {
      pr->live = false;
      vox_hide(v, pr->e);
      if (pr->dmg > 0.0f)
        vox_damage(v, foe, pr->dmg, pr->mine);
    }
  }
  vox_pose(v, me, 1.0f);
  if (v->phase == VOX_FIGHT)
    vox_pose(v, wf, 1.0f);
  return true;
}

// the wins: EXP, studs, the wild card; returns the card shown as a reward (-1 none)
hz_internal void vox_reward(Vox *v, SaveState *save, CardGame *cg) {
  if (v->phase != VOX_END || v->phase_t > 0.0f || v->result != 0)
    return;
  GameSave *s = &save->data;
  s->vox_wins++;
  s->exp += WILD_EXP;
  s->studs += v->wild.shiny ? 10u : 3u;
  i32 id = -1;
  if (v->wild.shiny)
    id = reward_card(s, &v->seed, true);
  else if (!owns(s, v->wild.id)) {
    give_card(s, v->wild.id);
    id = v->wild.id;
  }
  save_mark(save);
  sound_play(v->l->snd, SND_FANFARE, v->wild.shiny ? 1.2f : 1.0f, 0.7f);
  LIFE_TOAST(v->l, "You won! +% EXP \xc2\xb7 +% studs", fmt_i32(WILD_EXP), fmt_u32(v->wild.shiny ? 10u : 3u));
  if (id >= 0) {
    v->reward_card = id;
    v->reward_at = 1.4f;
  }
}

hz_internal void vox_after(Vox *v, CardGame *cg, f32 dt) {
  if (v->reward_at > 0.0f) {
    v->reward_at -= dt;
    if (v->reward_at <= 0.0f) {
      char title[96];
      cstr_format(title, sizeof(title), "%'s card joined your collection!", fmt_cstr(card_def(v->reward_card).name));
      cg_show_rewards(cg, title, &v->reward_card, 1);
    }
  }
}

// ---- the HUD ----
hz_internal void vox_bar(HzUIElementId id, f32 frac, f32 w, HzUIColor fill) {
  ui_element({.id = id, .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(10)}},
              .background_color = {40, 44, 52, 255}, .corner_radius = ui_corner_radius(5)}) {
    ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(m_maxf(0.0f, w * m_clampf(frac, 0.0f, 1.0f))), .height = ui_sizing_grow(0)}},
                .background_color = fill, .corner_radius = ui_corner_radius(5)}) {}
  }
}

hz_internal HzUIColor vox_hp_color(f32 f) { return f > 0.5f ? (HzUIColor){60, 190, 80, 255} : f > 0.2f ? (HzUIColor){240, 190, 40, 255} : (HzUIColor){220, 60, 40, 255}; }

// each skill's icon (tools/make_screen_art.py sk_*)
hz_internal CString vox_skill_icon(Skill sk) {
  static const struct { CString name, icon; } map[] = {
      {"Root Slam", ART_SK_ROOT}, {"Leaf Storm", ART_SK_LEAF}, {"Storm Call", ART_SK_STORM}, {"Thunder Bite", ART_SK_BOLT},
      {"Tidal Whirl", ART_SK_WAVE}, {"Ink Shot", ART_SK_DROP}, {"Brick Blast", ART_SK_BLAST}, {"Tackle", ART_SK_DASH}};
  for (u32 i = 0; i < sizeof(map) / sizeof(map[0]); i++)
    if (cstr_equal(map[i].name, sk.name))
      return map[i].icon;
  return sk.kind == SK_DASH ? ART_SK_DASH : ART_SK_BLAST;
}

hz_internal void vox_draw(Vox *v, const InputSystem *in, const Camera *cam, f32 sw, f32 shgt, f32 dpr, b32 show_pill) {
  HzUIColor white = {255, 255, 255, 255}, ink = {27, 42, 52, 255};
  OsSafeAreaInsets safe = os_get_safe_area();
  Fighter *me = &v->me, *wf = &v->wild;
  if (me->id < 0)
    return;
  // damage numbers over the one who got hit
  if (cam) {
    for (i32 i = 0; i < VOX_NUMBERS; i++) {
      VoxNumber *n = &v->num[i];
      if (n->t > 0.9f || n->n <= 0)
        continue;
      f32 sp[2];
      v3 wp = town_m(n->p.x, n->p.y + n->t * 2.0f, n->p.z);
      camera_project_to_screen(cam->view_proj, wp, sw * dpr, shgt * dpr, sp);
      ui_element({.id = ui_idi("VoxNum", (u32)i),
                  .floating = {.attach_to = UI_ATTACH_TO_ROOT, .offset = {sp[0] / dpr - 20.0f, sp[1] / dpr - 14.0f}, .z_index = 20}}) {
        cu_text(cu_numstr(-n->n, false), (u16)(n->mine ? 30 : 24), n->mine ? (HzUIColor){255, 230, 80, 255} : (HzUIColor){255, 90, 70, 255});
      }
    }
  }
  if (v->phase == VOX_INTRO || v->phase == VOX_FIGHT || v->phase == VOX_END) {
    // the wild one: name and HP, top centre
    ui_element({.id = ui_id("VoxWild"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP},
                             .offset = {0.0f, g_hud.center}},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 5, .padding = {.left = 14, .right = 14, .top = 8, .bottom = 10}},
                .background_color = {16, 20, 26, 215}, .corner_radius = ui_corner_radius(10)}) {
      cstr_format(v->text[0], 96, "% %   %/%", fmt_cstr(wf->shiny ? "\xe2\x9c\xa8 Shiny" : "Wild"), fmt_cstr(card_def(wf->id < 0 ? 0 : wf->id).name),
                  fmt_i32((i32)wf->hp), fmt_i32((i32)wf->max));
      cu_text(v->text[0], 16, wf->shiny ? (HzUIColor){255, 210, 61, 255} : white);
      vox_bar(ui_id("VoxWildBar"), wf->hp / m_maxf(wf->max, 1.0f), 240.0f, wf->shiny ? (HzUIColor){255, 200, 40, 255} : vox_hp_color(wf->hp / m_maxf(wf->max, 1.0f)));
    }
    // yours: HP, the two skills with their recharge, Run
    ui_element({.id = ui_id("VoxMine"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0.0f, -safe.bottom - 18.0f}},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .padding = ui_padding_all(12), .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = {16, 20, 26, 215}, .corner_radius = ui_corner_radius(12)}) {
      cstr_format(v->text[1], 96, "%   %/%", fmt_cstr(card_def(me->id).name), fmt_i32((i32)me->hp), fmt_i32((i32)me->max));
      cu_text(v->text[1], 16, white);
      vox_bar(ui_id("VoxMeBar"), me->hp / m_maxf(me->max, 1.0f), 300.0f, vox_hp_color(me->hp / m_maxf(me->max, 1.0f)));
      // the skills (as Pokemon Quest): square icon tiles, the name small below; while one recharges it fills from the
      // bottom up in its colour (full = ready), the seconds left on it; the key number in the corner
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 14, .child_alignment = {.y = UI_ALIGN_Y_TOP}}}) {
        const f32 tile = 62.0f;
        for (i32 k = 0; k < 3; k++) {
          b32 run_tile = k == 2;
          Skill sk = run_tile ? (Skill){0} : vox_skill(me->id, k);
          b32 ready = run_tile || (v->cool[k] <= 0.0f && v->phase == VOX_FIGHT);
          v4 c = vox_rgb(run_tile ? 0xa0503cu : sk.color);
          HzUIColor skc = {c.x * 210.0f, c.y * 210.0f, c.z * 210.0f, 255};
          f32 fill = ready ? 1.0f : v->cool[k] > 0.0f ? 1.0f - v->cool[k] / m_maxf(sk.cooldown, 0.01f) : 0.0f;
          ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .child_alignment = {.x = UI_ALIGN_X_CENTER},
                                 .sizing = {.width = ui_sizing_fixed(tile + 14.0f)}}}) {
            ui_element({.id = run_tile ? ui_id("VoxRun") : ui_idi("VoxSkill", (u32)k),
                        .layout = {.sizing = {.width = ui_sizing_fixed(tile), .height = ui_sizing_fixed(tile)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                        .background_color = (HzUIColor){44, 48, 56, 255},
                        .corner_radius = ui_corner_radius(16),
                        .clip = {.horizontal = true, .vertical = true},
                        .border = {.color = ready ? (HzUIColor){255, 255, 255, 220} : (HzUIColor){90, 96, 106, 255},
                                   .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}, .overlay = true},
                        .shadow = {.color = ready && !run_tile ? (HzUIColor){skc.r, skc.g, skc.b, 160} : (HzUIColor){0, 0, 0, 90}, .offset_y = 2.0f, .blur = 8.0f},
                        .brightness = ready && ui_hovered() ? opt_some(f32, 1.15f) : opt_none(f32)}) {
              if (ready && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame) {
                if (run_tile)
                  v->click_run = true;
                else
                  v->click_skill[k] = true;
              }
              if (fill > 0.0f)
                ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 1, .clip_to = UI_CLIP_TO_ATTACHED_PARENT,
                                         .attach_points = {.element = UI_ATTACH_POINT_LEFT_BOTTOM, .parent = UI_ATTACH_POINT_LEFT_BOTTOM},
                                         .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                            .layout = {.sizing = {.width = ui_sizing_fixed(tile), .height = ui_sizing_fixed(tile * fill)}},
                            .corner_radius = {.top_left = fill > 0.9f ? 16.0f : 0.0f, .top_right = fill > 0.9f ? 16.0f : 0.0f, .bottom_left = 16.0f, .bottom_right = 16.0f},
                            .gradient = ready ? cu_lin2(180.0f, cu_mix(skc, white, 0.25f), skc)
                                              : cu_lin2(180.0f, (HzUIColor){skc.r * 0.75f, skc.g * 0.75f, skc.b * 0.75f, 255}, (HzUIColor){skc.r * 0.55f, skc.g * 0.55f, skc.b * 0.55f, 255}),
                            .border = {.color = ready ? (HzUIColor){0} : (HzUIColor){255, 255, 255, 170}, .width = {.top = 2}}}) {}
              // the icon (and while it recharges, the seconds over it)
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 2, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                          .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(tile), .height = ui_sizing_fixed(tile)},
                                     .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}}}) {
                f32 is = tile * 0.56f;
                HzUITextConfig ic = {.fontId = (u16)g_art.idx, .fontSize = is, .lineHeight = is, .textColor = ready ? white : (HzUIColor){255, 255, 255, 150}};
                ic.shadows[0] = (HzUITextShadow){.color = {0, 0, 0, 140}, .offset_y = 1.5f, .blur = 3.0f};
                ui_text(ui_str_c(run_tile ? ART_IC_BACK : vox_skill_icon(sk)), ic);
              }
              if (!ready && v->cool[k] > 0.0f)
                ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 3, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                                         .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                                         .offset = {0, -3.0f}}}) {
                  cstr_format(v->text[4 + k], 96, "%s", fmt_f32_p(v->cool[k], 1));
                  cu_textb(v->text[4 + k], 11, white, 800, false, (HzUIColor){0, 0, 0, 200});
                }
              if (!run_tile) // the key
                ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 3, .offset = {5.0f, 3.0f},
                                         .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}}) {
                  cu_textb(k == 0 ? "1" : "2", 11, (HzUIColor){255, 255, 255, 190}, 800, false, (HzUIColor){0, 0, 0, 160});
                }
            }
            HzUITextConfig lc = {.fontSize = 11, .textColor = (HzUIColor){230, 234, 240, 255}, .weight = 700, .textAlignment = CLAY_TEXT_ALIGN_CENTER,
                                 .wrapMode = UI_TEXT_WRAP_WORDS};
            lc.shadows[0] = (HzUITextShadow){.color = {0, 0, 0, 180}, .offset_y = 1.0f, .blur = 2.0f};
            ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { ui_text(ui_str_c(run_tile ? "Run" : sk.name), lc); }
          }
        }
      }
    }
  } else if (show_pill) {
    // between battles: a small pill with its HP (a moon while it rests)
    ui_element({.id = ui_id("VoxPill"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                             .offset = {-safe.right - 14.0f, g_hud.row3}},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .padding = {.left = 12, .right = 12, .top = 6, .bottom = 8}},
                .background_color = ui_night_mix((HzUIColor){250, 250, 246, 225}, (HzUIColor){24, 32, 42, 225}), // dark at night (as the goal)
                .corner_radius = ui_corner_radius(10)}) {
      cstr_format(v->text[1], 96, "%%  %/%", fmt_cstr(me->tired ? "\xe2\x98\xbe " : ""), fmt_cstr(card_def(me->id).name), fmt_i32((i32)me->hp),
                  fmt_i32((i32)me->max));
      cu_text(v->text[1], 14, ui_night_mix(ink, (HzUIColor){232, 238, 244, 255}));
      vox_bar(ui_id("VoxPillBar"), me->hp / m_maxf(me->max, 1.0f), 150.0f, me->tired ? (HzUIColor){120, 130, 200, 255} : vox_hp_color(me->hp / m_maxf(me->max, 1.0f)));
    }
  }
}
