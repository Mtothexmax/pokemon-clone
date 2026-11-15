// Close-up reading of the sign boards (the camera flies to the painted board, as in the three.js version), and the
// debug picker: ` (backquote) toggles it, a click names what is under the mouse (from the town's tag registry, the
// moving things and the interaction targets). Included by starting_template.c above app_init.

typedef struct SignZoom {
  f32 t;          // 0 the rig's camera .. 1 at the board
  b32 active;     // the rig is off while > 0
  v3 from_pos;
  quat from_rot;
  SignBoard board;
} SignZoom;

// main lane, before the scope: eases the camera to the board (a sign's, a screen's; NULL: none) and back; returns true
// while it owns the camera
hz_internal b32 sign_zoom_update(SignZoom *z, EcsWorld *w, EcsQuery *camera_query, const SignBoard *want, f32 dt) {
  EcsEntity cam_e = ecs_query_get_singleton_entity(camera_query);
  if (cam_e == ECS_ENTITY_INVALID)
    return false;
  const Camera *cam = ecs_component_get(w, cam_e, Camera);
  const CameraFollow *follow = ecs_component_get(w, cam_e, CameraFollow);
  if (!cam || !follow)
    return false;
  b32 has = want != NULL;
  if (has && !z->active) { // start from where the rig has the camera now
    z->active = true;
    z->t = 0.0f;
    z->from_pos = cam->pos;
    z->from_rot = cam->rot;
    z->board = *want;
    CameraFollow f = *follow;
    f.disabled = true;
    ecs_component_set(w, cam_e, CameraFollow, f);
  }
  if (!z->active)
    return false;
  z->t = m_clampf(z->t + (has ? dt : -dt) / 0.6f, 0.0f, 1.0f);
  if (z->t <= 0.0f) { // back: the rig takes over again
    z->active = false;
    CameraFollow f = *follow;
    f.disabled = false;
    f.snap = true;
    ecs_component_set(w, cam_e, CameraFollow, f);
    return false;
  }
  // the board fills the view: distance from the vertical fov and the aspect
  SignBoard bd = z->board;
  f32 tan_half = m_tanf(cam->fov * (M_PI / 180.0f) * 0.5f);
  f32 aspect = m_maxf(cam->aspect, 0.5f);
  f32 dist = m_maxf(bd.h * 1.3f * 0.5f / tan_half, bd.w * 1.15f * 0.5f / (tan_half * aspect)) * TOWN_STUD;
  if (bd.max_dist > 0.0f)
    dist = m_minf(dist, bd.max_dist);
  v3 normal = {m_sinf(bd.yaw) * m_cosf(bd.tilt), m_sinf(bd.tilt), m_cosf(bd.yaw) * m_cosf(bd.tilt)};
  v3 center = town_m(bd.c.x, bd.c.y, bd.c.z);
  v3 to_pos = v3_add(center, v3_scale(normal, dist));
  quat to_rot = quat_look(v3_sub(to_pos, center), V3_UP);
  f32 k = z->t * z->t * (3.0f - 2.0f * z->t);
  Camera c = *cam;
  c.pos = v3_lerp(z->from_pos, to_pos, k);
  c.rot = quat_slerp(z->from_rot, to_rot, k);
  ecs_component_set(w, cam_e, Camera, c);
  return true;
}

// ---------------------------------------------------------------- the debug picker

typedef struct DebugPick {
  b32 on;
  char text[2][160];
  b32 has;
  v3 at;               // the picked point (studs), for the Copy button
  f32 copied_t;        // > 0: "Copied!" shows on the button that was pressed
  i32 copied;          // 0 the pick, 1 you
  char me_text[96];
} DebugPick;

// what a ray hit: the nearest moving thing along it, else the tags at the hit point, else the ground
hz_internal void debug_pick(DebugPick *dp, EcsWorld *w, Camera *cam, f32 mx, f32 my, u32 sw, u32 sh, const TownTag *tags,
                            u32 tag_count, const Life *life, const MinifigSet *figs, EcsEntity player) {
  v3 o, d;
  camera_mouse_to_ray(cam, mx, my, sw, sh, &o, &d);
  v3 hit = v3_add(o, v3_scale(d, 200.0f));
  b32 got = false;
  const EcsPhysicsWorld *pw = ecs_resource_get(w, EcsPhysicsWorld);
  EcsPhysicsHit h;
  EcsPhysicsQuery q = {.ignore = player};
  if (pw && ecs_physics_raycast(pw, o, hit, &q, &h)) {
    hit = h.hit.point;
    got = true;
  } else if (d.y < -1e-3f) { // the baseplate
    hit = v3_add(o, v3_scale(d, -o.y / d.y));
    got = true;
  }
  f32 hit_t = v3_length(v3_sub(hit, o));
  // moving things: a sphere each (studs, radius), nearest along the ray in front of the hit
  CString best = NULL;
  f32 best_t = got ? hit_t + 0.3f : 1e9f;
  v3 best_p = hit;
#define DBG_SPHERE(name, studs_pos, rad)                                                                                   \
  do {                                                                                                                     \
    v3 c_ = town_m((studs_pos).x, (studs_pos).y, (studs_pos).z);                                                           \
    v3 oc_ = v3_sub(c_, o);                                                                                                \
    f32 t_ = v3_dot(oc_, d);                                                                                               \
    if (t_ > 0.0f && t_ < best_t && v3_length(v3_sub(oc_, v3_scale(d, t_))) < (rad)*TOWN_STUD) {                       \
      best_t = t_;                                                                                                         \
      best = (name);                                                                                                       \
      best_p = c_;                                                                                                         \
    }                                                                                                                      \
  } while (0)
  CString fig_names[8] = {"You", "Dex", "Pip", "Prof. Quill (at the gap)", "Mom", "Lumi", "Prof. Quill (lab)", "villager"};
  for (u32 i = 1; i < figs->count && i < 8; i++) {
    const WorldTransform *wt = ecs_component_get(w, figs->figs[i].root, WorldTransform);
    if (wt)
      DBG_SPHERE(fig_names[i], ((v3){wt->position.x / TOWN_STUD, wt->position.y / TOWN_STUD + 2.4f, wt->position.z / TOWN_STUD}), 1.8f);
  }
  for (u32 i = 0; i < DUCKS; i++)
    DBG_SPHERE(i == 0 ? "Duck (mother)" : "Duckling", life->duck_p[i], 1.0f);
  DBG_SPHERE("Football", life->ball_p, 0.9f);
  for (u32 i = 0; i < GOLDEN_COUNT; i++)
    if (!((life->save->data.golden_mask >> i) & 1u)) {
      GoldenDef g = golden_def(i);
      DBG_SPHERE("Golden stud", ((v3){g.x, 1.4f, g.z}), 0.9f);
    }
  if (life->chest_live)
    DBG_SPHERE("Treasure chest", life->chest_p, 1.5f);
#undef DBG_SPHERE
  v3 hs = v3_scale(best ? best_p : hit, 1.0f / TOWN_STUD);
  if (!best && got) {
    // the smallest tag around the hit point
    f32 best_r = 1e9f;
    for (u32 i = 0; i < tag_count; i++) {
      const TownTag *tg = &tags[i];
      f32 dx = hs.x - tg->x, dz = hs.z - tg->z;
      if (dx * dx + dz * dz < tg->r * tg->r && tg->r < best_r) {
        best_r = tg->r;
        best = tg->name;
      }
    }
    if (!best)
      best = life_on_road(hs.x, hs.z) ? "Road plate" : hs.y < 0.2f ? "Baseplate" : "(unnamed brick)";
  }
  dp->has = best != NULL;
  if (!dp->has)
    return;
  dp->at = hs;
  cstr_format(dp->text[0], sizeof(dp->text[0]), "%", fmt_cstr(best));
  cstr_format(dp->text[1], sizeof(dp->text[1]), "at x % / y % / z % studs   (% m away)", fmt_f32_p(hs.x, 1), fmt_f32_p(hs.y, 1),
              fmt_f32_p(hs.z, 1), fmt_f32_p(best_t < 1e8f ? best_t : hit_t, 1));
  log_info("debug pick: % at % % %", fmt_cstr(best), fmt_f32_p(hs.x, 1), fmt_f32_p(hs.y, 1), fmt_f32_p(hs.z, 1));
}

// a small button in the debug panel: copies "x, y, z" (studs, one decimal) to the clipboard
hz_internal void debug_copy_btn(DebugPick *dp, const InputSystem *in, HzUIElementId id, i32 which, v3 p, f32 dt) {
  b32 done = dp->copied_t > 0.0f && dp->copied == which;
  ui_element({.id = id,
              .layout = {.padding = {.left = 10, .right = 10, .top = 4, .bottom = 4}},
              .background_color = done ? (HzUIColor){40, 140, 70, 255} : ui_hovered() ? (HzUIColor){150, 60, 50, 255} : (HzUIColor){110, 35, 30, 255},
              .border = {.color = {255, 200, 190, 160}, .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
              .corner_radius = ui_corner_radius(6)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame) {
      char buf[96];
      cstr_format(buf, sizeof(buf), "%, %, %", fmt_f32_p(p.x, 1), fmt_f32_p(p.y, 1), fmt_f32_p(p.z, 1));
      u32 n = 0;
      while (n < sizeof(buf) && buf[n])
        n++;
      os_clipboard_set_text(buf, n);
      dp->copied = which;
      dp->copied_t = 1.5f;
      log_info("debug: copied %", fmt_cstr(buf));
    }
    ui_text(ui_str_c(done ? "Copied!" : "Copy"), ui_text_config({.fontSize = 13, .textColor = {255, 255, 255, 255}}));
  }
  (void)dt;
}

// me: the player's position (studs)
hz_internal void debug_draw(DebugPick *dp, const InputSystem *in, v3 me, f32 dt) {
  if (!dp->on)
    return;
  dp->copied_t = m_maxf(0.0f, dp->copied_t - dt);
  OsSafeAreaInsets safe = os_get_safe_area();
  ui_element({.id = ui_id("DebugPick"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_BOTTOM, .parent = UI_ATTACH_POINT_RIGHT_BOTTOM},
                           .offset = {-safe.right - 16.0f, -safe.bottom - 16.0f}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .padding = {.left = 14, .right = 14, .top = 10, .bottom = 10}},
              .background_color = {60, 10, 10, 220},
              .corner_radius = ui_corner_radius(8)}) {
    ui_text(ui_string("DEBUG  ( ` to close)  click on something"),
            ui_text_config({.fontSize = 13, .textColor = {255, 170, 150, 255}}));
    if (dp->has) {
      ui_text(ui_str_c((CString)dp->text[0]), ui_text_config({.fontSize = 18, .textColor = {255, 255, 255, 255}}));
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
        ui_text(ui_str_c((CString)dp->text[1]), ui_text_config({.fontSize = 13, .textColor = {255, 220, 210, 255}}));
        debug_copy_btn(dp, in, ui_id("DbgCopyPick"), 0, dp->at, dt);
      }
    }
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
      cstr_format(dp->me_text, sizeof(dp->me_text), "You: x % / y % / z % studs", fmt_f32_p(me.x, 1), fmt_f32_p(me.y, 1), fmt_f32_p(me.z, 1));
      ui_text(ui_str_c((CString)dp->me_text),
              ui_text_config({.fontSize = 13, .textColor = {255, 220, 210, 255}}));
      debug_copy_btn(dp, in, ui_id("DbgCopyMe"), 1, me, dt);
    }
  }
}
