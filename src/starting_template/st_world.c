// The village's interactions (app side, main lane): going through doors and up and down the stairs (a teleport into
// the rooms of st_rooms.c), reading signs and looking at things (E, or the Read button on touch), the area title.
// Included by starting_template.c above app_init.

typedef struct Readable {
  f32 x, z;        // where you stand to read it, studs
  CString label;   // the prompt: "Read <label>"
  CString title;
  CString lines[3];
  CString note;
  HzUIColor bg, fg;
} Readable;

#define READABLE_COUNT 13
#define READ_RADIUS 3.6f // studs

hz_internal Readable readable(i32 i) {
  HzUIColor paper = {244, 244, 244, 255}, ink = {27, 42, 52, 255};
  HzUIColor wood = {255, 246, 200, 255}, brown = {107, 63, 29, 255};
  switch (i) {
  case 0:
    return (Readable){5.9f, 21.8f, "the village sign", "BRICKHOLLOW",
                      {"\"A village where every brick has a story.\"", "Population: 6 minifigs, lots of Beasts, countless studs."},
                      NULL, paper, ink};
  case 1:
    return (Readable){5.0f, -22.8f, "the route sign", "ROUTE 1",
                      {"Tall grass ahead!", "Wild Brick Beasts love to hide in it. Keep your deck ready."}, NULL, wood, brown};
  case 2:
    return (Readable){-21.4f, 8.9f, "the garden sign", "GARDEN", {"Please do not step on the flowers."},
                      "Pip, that means YOU.", (HzUIColor){228, 205, 158, 255}, brown};
  case 3:
    return (Readable){13.0f, 18.6f, "the lab sign", "QUILL LAB",
                      {"Brick Beast research since 1978.", "Opening hours: whenever the Professor remembers to unlock the door."},
                      NULL, paper, (HzUIColor){30, 90, 168, 255}};
  case 4:
    return (Readable){-18.2f, -2.4f, "the mailbox", "MAILBOX",
                      {"A postcard: \"Welcome home! Drop by the lab when you are ready.\"", "- Prof. Quill"}, NULL, paper, ink};
  case 5:
    return (Readable){15.0f, -7.4f, "the park bench", "PARK BENCH",
                      {"A nice place to sit and listen to the birds.", "Someone carved \"D + L\" into the backrest."}, NULL, wood, brown};
  case 6:
    return (Readable){-2.7f, 18.5f, "the ducks", "THE DUCKS",
                      {"Quack! The duck family paddles around the pond.", "The little ones follow their mother everywhere."}, NULL,
                      paper, ink};
  case 7:
    return (Readable){205.0f, -4.6f, "the TV", "BRICKHOLLOW NEWS",
                      {"\"...and wild Brick Beasts were spotted in the tall grass north of the village.\"",
                       "\"Prof. Quill reminds everyone: never go there without a deck!\""},
                      NULL, (HzUIColor){20, 40, 70, 255}, (HzUIColor){160, 220, 255, 255}};
  case 8:
    return (Readable){269.0f, -4.4f, "the TV", "GAME PAUSED",
                      {"A racing game, paused on the final lap.", "A sticky note on the screen: \"DON'T TOUCH. - Dex\""}, NULL,
                      (HzUIColor){60, 20, 70, 255}, (HzUIColor){255, 150, 230, 255}};
  case 9:
    return (Readable){251.0f, -4.4f, "the piano", "LUMI'S PIANO",
                      {"The sheet music says: \"Brickhollow Theme - practice every day!\"",
                       "You play a few notes. Not bad!"},
                      NULL, paper, ink};
  case 10:
    return (Readable){201.4f, 36.6f, "your PC", "YOUR PC",
                      {"1 new message:", "\"Come to the lab when you wake up. I have something for you! - Q\""}, NULL,
                      (HzUIColor){20, 40, 70, 255}, (HzUIColor){160, 220, 255, 255}};
  case 11:
    return (Readable){196.2f, 38.6f, "your bed", "YOUR BED", {"Still warm. No time for a nap, there is a whole village outside!"},
                      NULL, paper, ink};
  default:
    return (Readable){342.0f, 0.6f, "the lab computer", "RESEARCH NOTES",
                      {"Brick Beasts love studs, sunshine and being collected on cards.",
                       "Note to self: buy more coffee bricks."},
                      NULL, (HzUIColor){10, 26, 36, 255}, (HzUIColor){64, 192, 255, 255}};
  }
}

typedef struct WorldState {
  i32 area;          // -1 outside, else the RoomId the player is in
  f32 door_cooldown; // seconds; after a teleport the doors rest a moment
  i32 nearby;          // the readable in reach, -1 none
  b32 reading;
  f32 title_t;       // seconds the area title still shows
  CString title;
} WorldState;

hz_internal i32 world_area_at(v3 studs) {
  if (studs.x < 150.0f)
    return -1;
  if (studs.z > 25.0f)
    return ROOM_BEDROOM;
  i32 best = ROOM_HOME;
  f32 best_d = 1e9f;
  for (i32 i = 0; i < ROOM_BEDROOM; i++) {
    f32 d = m_absf(room_def((RoomId)i).ox - studs.x);
    if (d < best_d) {
      best_d = d;
      best = i;
    }
  }
  return best;
}

// main lane, before the scope: a door or the stairs ask for a teleport (written into the frame's controls)
hz_internal void world_doors(WorldState *ws, v3 pos_m, PlayerControls *controls, f32 dt) {
  v3 p = v3_scale(pos_m, 1.0f / TOWN_STUD);
  ws->door_cooldown = m_maxf(0.0f, ws->door_cooldown - dt);
  ws->title_t = m_maxf(0.0f, ws->title_t - dt);
  i32 area = world_area_at(p);
  if (area != ws->area) {
    ws->area = area;
    ws->title = area < 0 ? "Brickhollow" : room_def((RoomId)area).label;
    ws->title_t = 3.5f;
  }
  if (ws->door_cooldown > 0.0f)
    return;
  v3 to = V3_ZERO;
  f32 yaw = 0.0f;
  b32 go = false;
  if (area < 0) {
    for (i32 i = 0; i < ROOM_BEDROOM && !go; i++) {
      RoomDef r = room_def((RoomId)i);
      if (m_absf(p.x - r.door_x) < 1.6f && p.z > r.door_z && p.z < r.door_z + 2.0f) {
        to = (v3){r.ox, 0.5f, r.oz + r.d * 0.5f - 2.6f};
        yaw = M_PI; // into the room (-z)
        go = true;
      }
    }
  } else if (area == ROOM_BEDROOM) {
    TownRect wl = BED_WELL;
    if (p.x > wl.x0 && p.x < wl.x1 && p.z > wl.z0 + 0.4f) {
      to = (v3){210.6f, 0.5f, 3.6f};
      go = true;
    }
  } else {
    RoomDef r = room_def((RoomId)area);
    TownRect st = HOME_STAIRS;
    if (m_absf(p.x - r.ox) < 1.8f && p.z > r.oz + r.d * 0.5f - 0.5f) {
      to = (v3){r.door_x, 0.5f, r.door_z + 3.0f};
      go = true;
    } else if (area == ROOM_HOME && p.x > st.x0 && p.x < st.x1 && p.z < st.z0 + 1.2f) {
      to = (v3){207.0f, 0.5f, 40.0f};
      yaw = M_PI;
      go = true;
    }
  }
  if (go) {
    controls->teleport = true;
    controls->teleport_pos = town_m(to.x, to.y, to.z);
    controls->teleport_yaw = yaw;
    ws->door_cooldown = 0.6f;
    ws->reading = false;
  }
}

// main lane: which readable is in reach; E (or the touch button) opens and closes it, walking away closes it
hz_internal void world_reading(WorldState *ws, v3 pos_m, b32 toggle, b32 cancel) {
  v3 p = v3_scale(pos_m, 1.0f / TOWN_STUD);
  i32 nearby = -1;
  f32 best = READ_RADIUS * READ_RADIUS;
  for (i32 i = 0; i < READABLE_COUNT; i++) {
    Readable r = readable(i);
    f32 dx = r.x - p.x, dz = r.z - p.z, d = dx * dx + dz * dz;
    if (d < best) {
      best = d;
      nearby = i;
    }
  }
  if (nearby != ws->nearby)
    ws->reading = false;
  ws->nearby = nearby;
  if (cancel)
    ws->reading = false;
  else if (toggle && nearby >= 0)
    ws->reading = !ws->reading;
}

hz_internal void world_text(CString s, u16 size, HzUIColor color, b32 wrap) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size,
                                       .textColor = color,
                                       .wrapMode = wrap ? UI_TEXT_WRAP_WORDS : UI_TEXT_WRAP_NONE,
                                       .textAlignment = UI_TEXT_ALIGN_CENTER}));
}

// the UI frame: the area title, the "Read" prompt and the open sign
hz_internal void world_draw(const WorldState *ws, b32 touch) {
  OsSafeAreaInsets safe = os_get_safe_area();
  if (ws->title_t > 0.0f && ws->title) {
    u8 a = (u8)(255.0f * m_minf(1.0f, ws->title_t * 2.0f)); // fades out over the last half second
    ui_element({.id = ui_id("AreaTitle"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP},
                             .offset = {0.0f, safe.top + 18.0f}},
                .layout = {.padding = {.left = 18, .right = 18, .top = 8, .bottom = 8}},
                .background_color = {16, 18, 24, (u8)(a * 0.66f)},
                .corner_radius = ui_corner_radius(8)}) {
      world_text(ws->title, 22, (HzUIColor){255, 255, 255, a}, false);
    }
  }
  if (ws->nearby < 0)
    return;
  Readable r = readable(ws->nearby);
  if (!ws->reading) {
    ui_element({.id = ui_id("ReadPrompt"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0.0f, -safe.bottom - (touch ? 150.0f : 28.0f)}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10,
                           .padding = {.left = 14, .right = 16, .top = 8, .bottom = 8}},
                .background_color = {16, 18, 24, 190},
                .corner_radius = ui_corner_radius(8)}) {
      if (!touch) {
        ui_element({.layout = {.padding = {.left = 8, .right = 8, .top = 2, .bottom = 2}},
                    .background_color = {242, 205, 55, 255},
                    .corner_radius = ui_corner_radius(4)}) {
          world_text("E", 16, (HzUIColor){27, 42, 52, 255}, false);
        }
      }
      world_text(touch ? "Tap Read:" : "Read", 16, (HzUIColor){240, 242, 246, 255}, false);
      world_text(r.label, 16, (HzUIColor){242, 205, 55, 255}, false);
    }
    return;
  }
  ui_element({.id = ui_id("SignPanel"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_CENTER_CENTER}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 10, .child_alignment = {.x = UI_ALIGN_X_CENTER},
                         .sizing = {.width = ui_sizing_fixed(480)},
                         .padding = {.left = 26, .right = 26, .top = 20, .bottom = 18}},
              .background_color = r.bg,
              .border = {.color = r.fg, .width = {.left = 4, .right = 4, .top = 4, .bottom = 4}},
              .corner_radius = ui_corner_radius(10)}) {
    world_text(r.title, 30, r.fg, true);
    ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(3)}}, .background_color = r.fg}) {}
    for (i32 i = 0; i < 3; i++)
      if (r.lines[i])
        world_text(r.lines[i], 18, r.fg, true);
    if (r.note)
      world_text(r.note, 16, r.fg, true);
    world_text(touch ? "(tap Read to close)" : "(E to close)", 13, (HzUIColor){r.fg.r, r.fg.g, r.fg.b, 150}, false);
  }
}
