// Swipe walking for phones (the three.js "Swipe" controls, the default on touch): swipe once anywhere and the figure
// keeps walking that way, around obstacles (it picks one side and keeps it, towards a door when one is near); a tap
// stops it, and so does a wall it cannot get past for 1.5 s. Two fingers still pinch to zoom; the talk button works
// as before. The direction is kept in the world (the camera may turn while walking).
// Included by starting_template.c above app_init, after st_world.c.

typedef struct Swipe {
  // walking
  b32 on;
  v2 dir;           // world x/z, unit
  f32 best, gained; // how far it got along dir, and when it last got further
  i32 side;         // 0 not decided, +1 / -1: the side it walks around an obstacle
  i32 area;         // the area it started in: through a door it stops
  // the finger
  b32 tracking;
  u32 touch_id;
  v2 start;
  // the arrow where the swipe started
  f32 mark_t;
  v2 mark_pos;
  f32 mark_angle;
} Swipe;

#define SWIPE_MIN_PX 28.0f

// a screen swipe (x right, y down) to a world direction for the camera's yaw
hz_internal v2 swipe_world_dir(v2 d, f32 yaw) {
  v2 fw = {-m_sinf(yaw), -m_cosf(yaw)}, rt = {m_cosf(yaw), -m_sinf(yaw)};
  v2 w = v2_add(v2_scale(rt, d.x), v2_scale(fw, -d.y));
  f32 l = v2_length(w);
  return l > 1e-4f ? v2_scale(w, 1.0f / l) : (v2){0, 0};
}

hz_internal void swipe_stop(Swipe *sw) { sw->on = false; }

hz_internal void swipe_start(Swipe *sw, v2 dir, v3 pos, f32 now, i32 area) {
  sw->on = true;
  sw->dir = dir;
  sw->side = 0;
  sw->best = pos.x * dir.x + pos.z * dir.y;
  sw->gained = now;
  sw->area = area;
}

// main lane, after touch_controls_update (its buttons, the jump button and UI touches are claimed or skipped first)
hz_internal void swipe_touch(Swipe *sw, InputSystem *in, UISystem *ui, f32 yaw, v3 pos, f32 now, i32 area) {
  u32 fingers = 0;
  for (i32 i = 0; i < MAX_TOUCHES; i++)
    fingers += in->touches.items[i].is_active ? 1u : 0u;
  for (i32 i = 0; i < MAX_TOUCHES; i++) {
    MobileTouch *t = &in->touches.items[i];
    if (!sw->tracking && t->started_this_frame && !t->claimed && fingers == 1 && !ui_point_over_ui(ui, (v2){t->start_x, t->start_y})) {
      sw->tracking = true;
      sw->touch_id = t->id;
      sw->start = (v2){t->start_x, t->start_y};
    }
  }
  if (!sw->tracking)
    return;
  if (fingers > 1) { // a second finger: it is a pinch, not a swipe
    sw->tracking = false;
    return;
  }
  for (i32 i = 0; i < MAX_TOUCHES; i++) {
    MobileTouch *t = &in->touches.items[i];
    if (t->id != sw->touch_id || (!t->is_active && !t->stopped_this_frame))
      continue;
    v2 d = {t->current_x - sw->start.x, t->current_y - sw->start.y};
    b32 swiped = v2_length(d) > SWIPE_MIN_PX;
    // already walking while the finger is still down, as soon as it is clearly a swipe
    if (swiped) {
      v2 wd = swipe_world_dir(d, yaw);
      if (!sw->on || v2_dot(wd, sw->dir) < 0.999f)
        swipe_start(sw, wd, pos, now, area);
      sw->mark_t = now;
      sw->mark_pos = sw->start;
      sw->mark_angle = m_atan2f(d.y, d.x) * 180.0f / M_PI;
    }
    if (t->stopped_this_frame || !t->is_active) {
      if (!swiped)
        swipe_stop(sw); // a tap stops
      sw->tracking = false;
    }
    return;
  }
  sw->tracking = false; // the touch is gone
}

// is a step of the figure's width this way free? Rays at chest height (low steps and curbs do not count)
hz_internal b32 swipe_free(const EcsPhysicsWorld *pw, EcsEntity player, v3 pos, v2 d) {
  if (!pw)
    return true;
  EcsPhysicsQuery q = {.ignore = player};
  v2 side = {-d.y, d.x};
  for (i32 k = -1; k <= 1; k++) {
    v3 a = {pos.x + side.x * 0.2f * (f32)k, pos.y + 0.6f, pos.z + side.y * 0.2f * (f32)k};
    v3 b = {a.x + d.x * 0.75f, a.y, a.z + d.y * 0.75f};
    EcsPhysicsHit hit;
    if (ecs_physics_raycast(pw, a, b, &q, &hit))
      return false;
  }
  return true;
}

// the door nearest to the figure (village: the house doors; inside: the way out), in metres; false = none in reach
hz_internal b32 swipe_near_door(v3 pos, i32 area, v2 *out) {
  v2 p = {pos.x / TOWN_STUD, pos.z / TOWN_STUD};
  f32 best = 14.0f;
  b32 found = false;
  if (area < 0) {
    for (i32 i = 0; i < ROOM_BEDROOM; i++) {
      RoomDef r = room_def((RoomId)i);
      f32 dd = v2_length((v2){r.door_x - p.x, r.door_z + 0.8f - p.y});
      if (dd < best) {
        best = dd;
        *out = (v2){r.door_x * TOWN_STUD, (r.door_z + 0.8f) * TOWN_STUD};
        found = true;
      }
    }
  } else if (area != ROOM_BEDROOM) {
    RoomDef r = room_def((RoomId)area);
    *out = (v2){r.ox * TOWN_STUD, (r.oz + r.d * 0.5f) * TOWN_STUD};
    found = true;
  }
  return found;
}

// the way to go now (world x/z), or false: stuck, stop
hz_internal b32 swipe_step(Swipe *sw, const EcsPhysicsWorld *pw, EcsEntity player, v3 pos, f32 now, i32 area, v2 *out) {
  if (area != sw->area) // through a door: stand still on the other side
    return false;
  v2 a = sw->dir;
  // no progress in the swipe direction for 1.5 s: it is a wall, stop instead of searching back and forth
  f32 along = pos.x * a.x + pos.z * a.y;
  if (along > sw->best + 0.12f) {
    sw->best = along;
    sw->gained = now;
  } else if (now - sw->gained > 1.5f) {
    return false;
  }
  if (swipe_free(pw, player, pos, a)) {
    *out = a;
    return true;
  }
  // around the obstacle: decide on one side the first time (towards a door nearby, if there is one), then keep it
  i32 order[2] = {1, -1};
  v2 door;
  if (swipe_near_door(pos, area, &door)) {
    v2 to = {door.x - pos.x, door.y - pos.z};
    f32 tp = (a.x * m_cosf(1.0f) - a.y * m_sinf(1.0f)) * to.x + (a.x * m_sinf(1.0f) + a.y * m_cosf(1.0f)) * to.y;
    f32 tm = (a.x * m_cosf(-1.0f) - a.y * m_sinf(-1.0f)) * to.x + (a.x * m_sinf(-1.0f) + a.y * m_cosf(-1.0f)) * to.y;
    if (tm > tp) {
      order[0] = -1;
      order[1] = 1;
    }
  }
  static const f32 angles[4] = {0.45f, 0.8f, 1.15f, 1.5708f};
  for (i32 k = 0; k < 4; k++) {
    for (i32 o = 0; o < (sw->side ? 1 : 2); o++) {
      i32 sd = sw->side ? sw->side : order[o];
      f32 c = m_cosf(angles[k] * (f32)sd), s = m_sinf(angles[k] * (f32)sd);
      v2 d = {a.x * c - a.y * s, a.x * s + a.y * c};
      if (swipe_free(pw, player, pos, d)) {
        sw->side = sd;
        *out = d;
        return true;
      }
    }
  }
  return false;
}

// the fading arrow where a swipe started
hz_internal void swipe_draw(const Swipe *sw, f32 now) {
  f32 q = (now - sw->mark_t) / 0.7f;
  if (q < 0.0f || q >= 1.0f)
    return;
  f32 sz = 56.0f;
  ui_element({.id = ui_id("SwipeMark"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 4, .offset = {sw->mark_pos.x - sz * 0.5f, sw->mark_pos.y - sz * 0.5f},
                           .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
              .layout = {.sizing = {.width = ui_sizing_fixed(sz), .height = ui_sizing_fixed(sz)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = {255, 255, 255, 140},
              .corner_radius = ui_corner_radius(sz * 0.5f),
              .transform = cu_tf(0, 0, 1.0f + 0.3f * q, sw->mark_angle),
              .opacity = opt_some(f32, 1.0f - q)}) {
    cu_textb("\xe2\x86\x92", 34, cu_rgb(0x1b2a34), 800, false, (HzUIColor){0}); // arrow
  }
}
