// The screens you can look at: the TV (four channels; the BBN news, the weather and the nature show are voiced like
// in the three.js version), the piano (play it with the mouse, a finger or the keys A..K and W E T Y U), your PC
// (messages and your stats) and the lab computer (research notes). Each is a full-screen overlay; E or Esc closes it.
// Included by starting_template.c above app_init.

typedef enum ScreenKind { SCREEN_NONE, SCREEN_TV, SCREEN_PIANO, SCREEN_PC, SCREEN_LAB } ScreenKind;

typedef struct TvLine {
  CString speaker; // the name on the subtitle
  CString text;
  CString voice;
} TvLine;

#define TV_CHANNELS 4
#define TV_ANCHOR "Rex Studwell"
#define TV_REPORTER "Penny Plate"
#define TV_NARRATOR "Narrator"

hz_internal CString tv_channel_name(i32 ch) {
  switch (ch) {
  case 0: return "BBN News";
  case 1: return "Weather";
  case 2: return "Cartoons";
  default: return "Wild Bricks";
  }
}

// weather: 0 day, 1 night, 2 rain
hz_internal u32 tv_line_count(i32 ch, u32 weather) {
  switch (ch) {
  case 0: return 7;
  case 1: return 2;
  case 2: return 4;
  default: return 6;
  }
  (void)weather;
}

hz_internal TvLine tv_line(i32 ch, u32 weather, u32 i) {
  if (ch == 0) {
    switch (i) {
    case 0: return (TvLine){TV_ANCHOR, "Hello, Brickhollow! This is BBN, the Brick Beast News. I'm Rex Studwell.", VOICE("0c57cbfe")};
    case 1: return (TvLine){TV_REPORTER, "And I'm Penny Plate. Our top story: Glimmerlings were spotted folding light into tiny bricks!", VOICE("b92a25ec")};
    case 2: return (TvLine){TV_ANCHOR, "Fascinating. Professor Quill says it is perfectly safe, as long as nobody sneezes.", VOICE("ecff39c8")};
    case 3: return (TvLine){TV_REPORTER, "In sports: young Dex claims his deck is unbeatable. Local trainers would like a word.", VOICE("d7de0462")};
    case 4: return (TvLine){TV_ANCHOR, "The ducks at the pond report record bread deliveries this week. Experts say: more, please.", VOICE("180b23f7")};
    case 5: return (TvLine){TV_REPORTER, "And golden studs are still hidden all over the village. If you find one, keep it shiny!", VOICE("626a02a6")};
    default: return (TvLine){TV_ANCHOR, "That's all for now. Stay snappy, Brickhollow!", VOICE("1206ef79")};
    }
  }
  if (ch == 1) {
    if (weather == 2)
      return i == 0 ? (TvLine){TV_REPORTER, "Rain, rain, and a little more rain! Grab your umbrella, Brickhollow.", VOICE("51d69dfd")}
                    : (TvLine){TV_REPORTER, "The good news: the puddles are perfect for splashing. Pip approves.", VOICE("f7f07197")};
    if (weather == 1)
      return i == 0 ? (TvLine){TV_REPORTER, "Good evening, Brickhollow. Tonight we expect a clear and starry sky.", VOICE("677efb60")}
                    : (TvLine){TV_REPORTER, "The fireflies are out by the pond. Catch them gently, and let them go again!", VOICE("7168691e")};
    return i == 0 ? (TvLine){TV_REPORTER, "Here's your Brickhollow weather! Sunny skies all day, perfect for a battle on Route 1.", VOICE("b511c369")}
                  : (TvLine){TV_REPORTER, "A little shower may still pop by later. Puddles ahead, so splash responsibly!", VOICE("691c329e")};
  }
  if (ch == 2) {
    switch (i) {
    case 0: return (TvLine){"Captain Click", "Captain Click to the rescue! Who left this 1x1 plate on the floor?!", NULL};
    case 1: return (TvLine){"Sidekick Stud", "Not me, Captain! I only step on 2x4s.", NULL};
    case 2: return (TvLine){"Captain Click", "Then justice will be... snapped together!", NULL};
    default: return (TvLine){"Announcer", "Tune in next week for: The Case of the Missing Instructions!", NULL};
    }
  }
  switch (i) {
  case 0: return (TvLine){TV_NARRATOR, "Here, in the tall grass of Route 1, a young Brick Beast takes its very first steps.", VOICE("c51f2db1")};
  case 1: return (TvLine){TV_NARRATOR, "Its studs are still soft. Within a week, they will click firmly into place.", VOICE("58850af2")};
  case 2: return (TvLine){TV_NARRATOR, "Look closely. It has found a lost two by four brick. A rare treasure.", VOICE("3c3b5ded")};
  case 3: return (TvLine){TV_NARRATOR, "Brick Beasts talk with a gentle clicking sound. Scientists still argue about what it means.", VOICE("8cd330e6")};
  case 4: return (TvLine){TV_NARRATOR, "Most of them agree: it means snack time.", VOICE("4c545f17")};
  default: return (TvLine){TV_NARRATOR, "Nature is patient. And so, it seems, is this little Beast.", VOICE("89d05b53")};
  }
}

hz_internal HzUIColor tv_screen_color(i32 ch) {
  switch (ch) {
  case 0: return (HzUIColor){18, 50, 110, 255};
  case 1: return (HzUIColor){60, 150, 225, 255};
  case 2: return (HzUIColor){230, 120, 40, 255};
  default: return (HzUIColor){40, 110, 50, 255};
  }
}

#define PIANO_WHITE 9 // C4 .. D5
#define PIANO_KEYS 16

typedef struct Screens {
  ScreenKind kind;
  i32 channel;
  u32 weather;
  u32 line;
  f32 line_t;    // seconds on this line
  b32 voiced;    // the line's voice started
  HzHandleT(AudioVoice) voice;
  f32 key_lit[PIANO_KEYS];
  i32 tab;       // PC / lab: the open app
  // clicks the UI saw (read next frame)
  i32 click_key, click_tab, click_zap;
  b32 click_close;
  char text[8][96];
} Screens;

// semitones above C4 of piano key k (white keys first, then black)
hz_internal i32 piano_semi(i32 k) {
  i32 white[PIANO_WHITE] = {0, 2, 4, 5, 7, 9, 11, 12, 14};
  i32 black[PIANO_KEYS - PIANO_WHITE] = {1, 3, 6, 8, 10, 13, 15};
  return k < PIANO_WHITE ? white[k] : black[k - PIANO_WHITE];
}

hz_internal void screens_tv_line(Screens *sc, EcsWorld *w, AudioSystem *audio) {
  audio_voice_stop(audio, sc->voice, 0.1f);
  TvLine ln = tv_line(sc->channel, sc->weather, sc->line);
  sc->voice = (HzHandleT(AudioVoice)){0};
  sc->voiced = ln.voice != NULL;
  if (ln.voice)
    sc->voice = ecs_audio_play(w, ln.voice, &(EcsAudioPlayDesc){.volume = 0.95f, .bus = AUDIO_BUS_VOICE});
  sc->line_t = 0.0f;
}

hz_internal void screens_open(Screens *sc, ScreenKind kind, EcsWorld *w, AudioSystem *audio, u32 weather) {
  *sc = (Screens){.kind = kind, .weather = weather, .click_key = -1, .click_tab = -1, .click_zap = 0};
  if (kind == SCREEN_TV)
    screens_tv_line(sc, w, audio);
}

hz_internal void screens_close(Screens *sc, AudioSystem *audio) {
  audio_voice_stop(audio, sc->voice, 0.15f);
  sc->kind = SCREEN_NONE;
}

// main lane, before the scope
hz_internal void screens_update(Screens *sc, EcsWorld *w, AudioSystem *audio, SoundBank *snd, const InputSystem *in, f32 dt) {
  if (sc->kind == SCREEN_NONE)
    return;
  if (in->buttons[KEY_ESCAPE].pressed_this_frame || in->buttons[KEY_E].pressed_this_frame || sc->click_close) {
    screens_close(sc, audio);
    return;
  }
  for (i32 k = 0; k < PIANO_KEYS; k++)
    sc->key_lit[k] = m_maxf(0.0f, sc->key_lit[k] - dt);
  if (sc->kind == SCREEN_TV) {
    i32 zap = sc->click_zap;
    if (in->buttons[KEY_RIGHT].pressed_this_frame || in->buttons[KEY_D].pressed_this_frame)
      zap = 1;
    if (in->buttons[KEY_LEFT].pressed_this_frame || in->buttons[KEY_A].pressed_this_frame)
      zap = -1;
    sc->click_zap = 0;
    if (zap) {
      sc->channel = (sc->channel + zap + TV_CHANNELS) % TV_CHANNELS;
      sc->line = 0;
      sound_play(snd, SND_BLIP, 0.7f, 0.5f);
      screens_tv_line(sc, w, audio);
      return;
    }
    sc->line_t += dt;
    // next line: the voice is over (plus a breath), or 4 s for a line without one
    b32 over = sc->voiced ? (sc->line_t > 0.8f && !audio_voice_playing(audio, sc->voice) && sc->line_t > 1.2f) : sc->line_t > 4.0f;
    if (over || sc->line_t > 14.0f) {
      sc->line = (sc->line + 1) % tv_line_count(sc->channel, sc->weather);
      screens_tv_line(sc, w, audio);
    }
  } else if (sc->kind == SCREEN_PIANO) {
    App_InputButtonType white[PIANO_WHITE] = {KEY_A, KEY_S, KEY_D, KEY_F, KEY_G, KEY_H, KEY_J, KEY_K, KEY_L};
    App_InputButtonType black[PIANO_KEYS - PIANO_WHITE] = {KEY_W, KEY_E, KEY_T, KEY_Y, KEY_U, KEY_O, KEY_P};
    for (i32 k = 0; k < PIANO_KEYS; k++) {
      b32 hit = k < PIANO_WHITE ? in->buttons[white[k]].pressed_this_frame : in->buttons[black[k - PIANO_WHITE]].pressed_this_frame;
      if (k == PIANO_WHITE + 1)
        hit = false; // E closes; the D# key is mouse / touch only
      if (hit || sc->click_key == k) {
        sound_play(snd, SND_PIANO, m_powf(2.0f, (f32)piano_semi(k) / 12.0f), 0.9f);
        sc->key_lit[k] = 0.18f;
      }
    }
    sc->click_key = -1;
  } else {
    if (sc->click_tab >= 0)
      sc->tab = sc->click_tab;
    sc->click_tab = -1;
    if (in->buttons[KEY_RIGHT].pressed_this_frame || in->buttons[KEY_TAB].pressed_this_frame)
      sc->tab = (sc->tab + 1) % 2;
    if (in->buttons[KEY_LEFT].pressed_this_frame)
      sc->tab = (sc->tab + 1) % 2;
  }
}

hz_internal void scr_text(CString s, u16 size, HzUIColor c, b32 wrap) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size, .textColor = c, .wrapMode = wrap ? UI_TEXT_WRAP_WORDS : UI_TEXT_WRAP_NONE}));
}

// a small button; true when clicked this frame
hz_internal b32 scr_button(HzUIElementId id, CString label, b32 lit, const InputSystem *in) {
  b32 clicked = false;
  ui_element({.id = id,
              .layout = {.padding = {.left = 14, .right = 14, .top = 7, .bottom = 7}},
              .background_color = lit ? (HzUIColor){0, 85, 191, 255} : (HzUIColor){226, 232, 238, 255},
              .corner_radius = ui_corner_radius(7)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    scr_text(label, 16, lit ? (HzUIColor){255, 255, 255, 255} : (HzUIColor){27, 42, 52, 255}, false);
  }
  return clicked;
}

// the UI frame
hz_internal void screens_draw(Screens *sc, const InputSystem *in, const GameSave *save) {
  if (sc->kind == SCREEN_NONE)
    return;
  HzUIColor ink = {27, 42, 52, 255}, white = {255, 255, 255, 255};
  ui_element({.id = ui_id("ScreenDim"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = {6, 8, 12, 200}}) {
    if (sc->kind == SCREEN_TV) {
      TvLine ln = tv_line(sc->channel, sc->weather, sc->line);
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 10, .sizing = {.width = ui_sizing_percent(0.8f)},
                             .padding = ui_padding_all(18)},
                  .background_color = {30, 30, 34, 255},
                  .corner_radius = ui_corner_radius(16)}) {
        ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 14, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(300)},
                               .padding = ui_padding_all(22), .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                    .background_color = tv_screen_color(sc->channel),
                    .corner_radius = ui_corner_radius(10)}) {
          scr_text(tv_channel_name(sc->channel), 34, white, false);
          ui_element({.layout = {.sizing = {.height = ui_sizing_grow(0)}}}) {}
          ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .padding = ui_padding_all(10),
                                 .sizing = {.width = ui_sizing_grow(0)}},
                      .background_color = {0, 0, 0, 170},
                      .corner_radius = ui_corner_radius(6)}) {
            scr_text(ln.speaker, 15, (HzUIColor){242, 205, 55, 255}, false);
            scr_text(ln.text, 20, white, true);
          }
        }
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
          if (scr_button(ui_id("TvPrev"), "\xe2\x97\x80 channel", false, in))
            sc->click_zap = -1;
          if (scr_button(ui_id("TvNext"), "channel \xe2\x96\xb6", false, in))
            sc->click_zap = 1;
          if (scr_button(ui_id("TvOff"), "Turn off (E)", false, in))
            sc->click_close = true;
        }
      }
    } else if (sc->kind == SCREEN_PIANO) {
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .padding = ui_padding_all(20),
                             .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .background_color = {27, 27, 30, 255},
                  .corner_radius = ui_corner_radius(14)}) {
        scr_text("Lumi's piano", 26, white, false);
        // black keys row, then white keys row (a simple two-row keyboard)
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .padding = {.left = 30}}}) {
          i32 gaps[PIANO_KEYS - PIANO_WHITE] = {0, 0, 1, 0, 0, 1, 0}; // a gap after E and B
          for (i32 b = 0; b < PIANO_KEYS - PIANO_WHITE; b++) {
            i32 k = PIANO_WHITE + b;
            ui_element({.id = ui_idi("PianoBlack", (u32)b),
                        .layout = {.sizing = {.width = ui_sizing_fixed(46), .height = ui_sizing_fixed(110)}},
                        .background_color = sc->key_lit[k] > 0.0f ? (HzUIColor){0, 85, 191, 255} : (HzUIColor){12, 12, 14, 255},
                        .corner_radius = ui_corner_radius(4)}) {
              if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
                sc->click_key = k;
            }
            if (gaps[b])
              ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(52)}}}) {}
          }
        }
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 4}}) {
          CString labels[PIANO_WHITE] = {"A", "S", "D", "F", "G", "H", "J", "K", "L"};
          for (i32 k = 0; k < PIANO_WHITE; k++) {
            ui_element({.id = ui_idi("PianoWhite", (u32)k),
                        .layout = {.sizing = {.width = ui_sizing_fixed(56), .height = ui_sizing_fixed(170)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}, .padding = {.bottom = 8}},
                        .background_color = sc->key_lit[k] > 0.0f ? (HzUIColor){150, 200, 255, 255} : (HzUIColor){248, 248, 244, 255},
                        .corner_radius = ui_corner_radius(5)}) {
              if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
                sc->click_key = k;
              scr_text(labels[k], 14, (HzUIColor){120, 128, 136, 255}, false);
            }
          }
        }
        scr_text("Keys A..L (white) and W, T, Y, U, O, P (black), or tap the keys. E / Esc to stop.", 14,
                 (HzUIColor){190, 196, 204, 255}, false);
      }
    } else {
      b32 lab = sc->kind == SCREEN_LAB;
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .padding = ui_padding_all(18),
                             .sizing = {.width = ui_sizing_fixed(620)}},
                  .background_color = lab ? (HzUIColor){10, 26, 36, 255} : (HzUIColor){70, 130, 200, 255},
                  .corner_radius = ui_corner_radius(12)}) {
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8}}) {
          CString tabs0 = lab ? "Research notes" : "Messages", tabs1 = lab ? "Brick Beast database" : "My stats";
          if (scr_button(ui_id("PcTab0"), tabs0, sc->tab == 0, in))
            sc->click_tab = 0;
          if (scr_button(ui_id("PcTab1"), tabs1, sc->tab == 1, in))
            sc->click_tab = 1;
          ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
          if (scr_button(ui_id("PcClose"), "Close (E)", false, in))
            sc->click_close = true;
        }
        HzUIColor fg = lab ? (HzUIColor){64, 192, 255, 255} : white;
        ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .padding = ui_padding_all(14),
                               .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(280)}},
                    .background_color = lab ? (HzUIColor){0, 10, 16, 255} : (HzUIColor){20, 40, 70, 230},
                    .corner_radius = ui_corner_radius(8)}) {
          if (lab && sc->tab == 0) {
            scr_text("> notes/brick_beasts.txt", 15, fg, false);
            scr_text("Brick Beasts love studs, sunshine and being collected on cards.", 17, fg, true);
            scr_text("They hide in tall grass and only come out for trainers with a deck.", 17, fg, true);
            scr_text("Six-energy cards that reach their final stage become real Voxelmon.", 17, fg, true);
            scr_text("Note to self: buy more coffee bricks.", 17, fg, true);
          } else if (lab) {
            scr_text("> db/species --count", 15, fg, false);
            scr_text("34 species registered. 0 seen by you (start your collection with the Professor!).", 17, fg, true);
            scr_text("(The card collection comes with the next update of Brickhollow.)", 15, fg, true);
          } else if (sc->tab == 0) {
            scr_text("Dex: \"SNAP!\"", 18, fg, false);
            scr_text("Dex: \"snap.\"", 18, fg, false);
            scr_text("Dex: \"SNAP!!!\"", 18, fg, false);
            scr_text("Prof. Quill: \"Come to the lab when you wake up. I have something for you!\"", 18, fg, true);
          } else {
            u32 n = 0;
            for (u32 k = 0; k < 15; k++)
              n += (save->golden_mask >> k) & 1u;
            cstr_format(sc->text[0], sizeof(sc->text[0]), "Studs: %", fmt_u32(save->studs));
            cstr_format(sc->text[1], sizeof(sc->text[1]), "Golden studs: % / 15", fmt_u32(n));
            cstr_format(sc->text[2], sizeof(sc->text[2]), "Ducks fed: %   Treasure chests: %", fmt_u32(save->duck_feeds), fmt_u32(save->chests));
            cstr_format(sc->text[3], sizeof(sc->text[3]), "Fireflies: %   Butterflies: %", fmt_u32(save->fireflies), fmt_u32(save->_reserved[0]));
            cstr_format(sc->text[4], sizeof(sc->text[4]), "Goals: %   Pip found: %   Pip tagged: %", fmt_u32(save->goals), fmt_u32(save->seek_wins),
                        fmt_u32(save->_reserved[1]));
            for (i32 i = 0; i < 5; i++)
              scr_text(sc->text[i], 18, fg, false);
          }
        }
      }
    }
  }
  (void)ink;
}
