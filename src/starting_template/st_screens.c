// The screens you can look at, as in the three.js version: the camera flies up to the TV or the computer and the
// picture is drawn right on its screen. The TV has four channels (the BBN news, the weather and the nature show are
// voiced) drawn from vector art (tools/make_screen_art.py: SVG layers cooked to glyphs, each with its own colour and
// motion), your PC runs "BrickOS" (messages, stats, your beasts, stickers), Prof. Quill's lab computer a read-only
// research terminal that types by itself. The piano stays a keyboard over the room. E or Esc closes.
// Included by starting_template.c above app_init.

#include "../../assets/brickhollow/screens/art.icons.gen.h"
hz_global HzHandleT(UIFontData) g_art;      // the screens' vector art: assets/brickhollow/screens/art/*.svg, cooked to glyphs
hz_global HzHandleT(UIFontFamily) g_mono;   // the terminal's monospaced face (the OS one natively; the web has Inter)

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

#define PIANO_WHITE 9 // C4 .. D5
#define PIANO_KEYS 16

// the PC's apps
typedef enum PcApp { APP_HOME, APP_MSG, APP_STATS, APP_BEASTS, APP_STICKERS, APP_COUNT } PcApp;

// the lab terminal: the lines on screen and the "session" it is typing (LabTerminal.svelte)
#define TERM_LINES 18
#define TERM_QUEUE 16
#define TERM_W 80
typedef enum TermCls { TC_PLAIN, TC_CMD, TC_DIM, TC_WARN, TC_OK, TC_GENE, TC_TITLE } TermCls;

typedef struct Screens {
  ScreenKind kind;
  f32 time;      // since it opened
  b32 has_board; // the camera flies to `board` (the TV, the PCs); the piano has none
  SignBoard board;
  i32 channel;
  u32 weather;
  u32 line;
  f32 line_t;    // seconds on this line
  b32 voiced;    // the line's voice started
  f32 noise_t;   // static after switching channels
  HzHandleT(AudioVoice) voice;
  f32 key_lit[PIANO_KEYS];
  PcApp app;     // the PC's open app
  f32 app_t;     // since it opened
  u32 fact;      // the beast fact shown this time
  // the lab terminal
  u32 rng;
  char term[TERM_LINES][TERM_W];
  u8 term_cls[TERM_LINES];
  u32 term_n;    // lines written (the last TERM_LINES are shown)
  char queue[TERM_QUEUE][TERM_W];
  u8 queue_cls[TERM_QUEUE];
  b32 queue_cmd[TERM_QUEUE];
  u32 queue_n, queue_i;
  char typing[TERM_W];
  u32 typed;     // letters of the command typed so far
  f32 term_wait; // until the next step
  // clicks the UI saw (read next frame)
  i32 click_key, click_zap, click_app;
  b32 click_close;
  char text[12][96];
} Screens;

// semitones above C4 of piano key k (white keys first, then black)
hz_internal i32 piano_semi(i32 k) {
  i32 white[PIANO_WHITE] = {0, 2, 4, 5, 7, 9, 11, 12, 14};
  i32 black[PIANO_KEYS - PIANO_WHITE] = {1, 3, 6, 8, 10, 13, 15};
  return k < PIANO_WHITE ? white[k] : black[k - PIANO_WHITE];
}

// the screens the camera flies to, in studs: centre, facing (yaw), size; the lab computers lean back 45 degrees.
// `x, z` is where the target stands (st_world.c targets), which picks the TV or computer
hz_internal b32 screen_board(ScreenKind kind, f32 x, f32 z, SignBoard *out) {
  switch (kind) {
  case SCREEN_TV: // st_rooms.c room_screen: the home TV, the one in Dex & Lumi's house
    *out = x < 240.0f ? (SignBoard){{205.0f, 4.35f, -7.11f}, 0.0f, 3.8f, 2.1f, 0.0f} : (SignBoard){{269.0f, 5.2f, -7.61f}, 0.0f, 3.0f, 1.8f, 0.0f};
    return true;
  case SCREEN_PC: *out = (SignBoard){{201.4f, 4.8f, 33.69f}, 0.0f, 2.6f, 1.6f, 0.0f}; return true;
  case SCREEN_LAB: { // the office computers on the lab desks (st_rooms.c, as the three.js labComputers)
    f32 dz = z < 0.5f ? -2.0f : 3.5f;
    *out = (SignBoard){{345.2f, 4.33f, dz - 0.17f}, 0.0f, 2.2f, 1.64f, M_PI * 0.25f};
    return true;
  }
  default: return false;
  }
}

hz_internal void scr_copy(char *dst, u32 cap, CString src) { cstr_copy_bounded(dst, cap, src, (u32)cstr_len(src)); }

hz_internal u32 scr_rand(Screens *sc) {
  u32 x = sc->rng ? sc->rng : 0x9e3779b9u;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  return sc->rng = x;
}
hz_internal f32 scr_randf(Screens *sc) { return (f32)(scr_rand(sc) & 0xffffff) / 16777216.0f; }

// ---------------------------------------------------------------- the lab terminal's sessions

hz_internal void term_push(Screens *sc, CString text, TermCls cls) {
  u32 i = sc->term_n % TERM_LINES;
  scr_copy(sc->term[i], TERM_W, text);
  sc->term_cls[i] = (u8)cls;
  sc->term_n++;
}

hz_internal char *term_q(Screens *sc, TermCls cls, b32 cmd) {
  u32 i = sc->queue_n < TERM_QUEUE ? sc->queue_n++ : TERM_QUEUE - 1;
  sc->queue_cls[i] = (u8)cls;
  sc->queue_cmd[i] = cmd;
  sc->queue[i][0] = 0;
  return sc->queue[i];
}

hz_internal CString term_kind(CardKind k) {
  switch (k) {
  case KIND_REVEAL: return "on reveal";
  case KIND_ONGOING: return "ongoing";
  case KIND_ACTIVATE: return "activate";
  case KIND_ENDTURN: return "end of turn";
  case KIND_START: return "game start";
  default: return "vanilla";
  }
}

// "[#######.............] 35%"
hz_internal void term_bar(char *out, u32 cap, f32 p) {
  char b[24];
  i32 n = (i32)(m_clampf(p, 0.0f, 1.0f) * 20.0f + 0.5f);
  for (i32 i = 0; i < 20; i++)
    b[i] = i < n ? '#' : '.';
  b[20] = 0;
  cstr_format(out, cap, "[%] %\x25", fmt_cstr(b), fmt_i32((i32)(m_clampf(p, 0.0f, 1.0f) * 100.0f + 0.5f)));
}

// one session: commands typed letter by letter, and what they print (LabTerminal.svelte script())
hz_internal void term_script(Screens *sc, const GameSave *g) {
  sc->queue_n = sc->queue_i = 0;
  i32 sp = (i32)(scr_rand(sc) % CARD_POOL);
  CardDef d = card_def(sp);
  char bar[40], gene[32], hex[2][32];
  static const char bases[5] = {'B', 'R', 'I', 'C', 'K'};
  for (i32 i = 0; i < 24; i++)
    gene[i] = bases[scr_rand(sc) % 5];
  gene[24] = 0;
  for (i32 r = 0; r < 2; r++) {
    char *o = hex[r];
    for (i32 i = 0; i < 8; i++) {
      u32 v = scr_rand(sc) & 255;
      static const char hx[] = "0123456789abcdef";
      *o++ = hx[v >> 4];
      *o++ = hx[v & 15];
      *o++ = i < 7 ? ' ' : 0;
    }
  }
  cstr_format(term_q(sc, TC_CMD, true), TERM_W, "./analyze --specimen %", fmt_cstr(d.id));
  cstr_format(term_q(sc, TC_DIM, false), TERM_W, "loading sample %.brk ...", fmt_cstr(d.name));
  cstr_format(term_q(sc, TC_PLAIN, false), TERM_W, "  cost % | power % | %", fmt_i32(d.cost), fmt_i32(d.power), fmt_cstr(term_kind(d.kind)));
  cstr_format(term_q(sc, TC_GENE, false), TERM_W, "  genome  %", fmt_cstr(gene));
  term_bar(bar, sizeof(bar), 0.3f + scr_randf(sc) * 0.7f);
  cstr_format(term_q(sc, TC_OK, false), TERM_W, "  % stud resonance", fmt_cstr(bar));
  scr_copy(term_q(sc, TC_CMD, true), TERM_W, "cat /var/log/route1.log | tail -3");
  cstr_format(term_q(sc, TC_WARN, false), TERM_W, "  [%] wild pack detected in tall grass", fmt_f32_p(scr_randf(sc) * 99.0f, 2));
  cstr_format(term_q(sc, TC_PLAIN, false), TERM_W, "  [%] glimmer flux % kBr", fmt_f32_p(scr_randf(sc) * 99.0f, 2), fmt_f32_p(scr_randf(sc) * 3.0f, 3));
  scr_copy(term_q(sc, TC_CMD, true), TERM_W, "hexdump -n 16 /dev/brickcore");
  cstr_format(term_q(sc, TC_DIM, false), TERM_W, "  0000  %", fmt_cstr(hex[0]));
  cstr_format(term_q(sc, TC_DIM, false), TERM_W, "  0008  %", fmt_cstr(hex[1]));
  scr_copy(term_q(sc, TC_CMD, true), TERM_W, "./collection --status");
  u32 own = owned_count(g);
  term_bar(bar, sizeof(bar), (f32)own / (f32)CARD_POOL);
  cstr_format(term_q(sc, TC_OK, false), TERM_W, "  % %/% beasts", fmt_cstr(bar), fmt_u32(own), fmt_i32(CARD_POOL));
  cstr_format(term_q(sc, TC_PLAIN, false), TERM_W, "  battles won: %  \xc2\xb7  rival defeated: %x", fmt_u32(g->wins), fmt_u32(g->rival_wins));
  static const CString jokes[4][2] = {
      {"ping dex-house.local", "  64 bytes from dex-house: \"SNAP!\" time=1ms"},
      {"sudo make coffee", "  [sudo] password for quill: ******  ACCESS DENIED (no coffee)"},
      {"ssh quill@moon-base", "  connection refused: the moon is made of bricks"},
      {"git log --oneline -1", "  a1b2c3d fix: beasts escaping through the vents again"}};
  u32 j = scr_rand(sc) % 4;
  scr_copy(term_q(sc, TC_CMD, true), TERM_W, jokes[j][0]);
  scr_copy(term_q(sc, TC_WARN, false), TERM_W, jokes[j][1]);
}

// one step of the terminal: output lines come quickly, commands are typed after the prompt
hz_internal void term_update(Screens *sc, const GameSave *g, SoundBank *snd, f32 dt) {
  sc->term_wait -= dt;
  while (sc->term_wait <= 0.0f) {
    if (sc->queue_i >= sc->queue_n)
      term_script(sc, g);
    u32 i = sc->queue_i;
    if (!sc->queue_cmd[i]) {
      term_push(sc, sc->queue[i], (TermCls)sc->queue_cls[i]);
      sc->queue_i++;
      sc->term_wait += 0.12f + scr_randf(sc) * 0.26f;
      continue;
    }
    u32 len = (u32)cstr_len(sc->queue[i]);
    if (sc->typed < len) { // the next letter
      sc->typed++;
      memcpy(sc->typing, sc->queue[i], sc->typed);
      sc->typing[sc->typed] = 0;
      if (sc->typed % 3 == 0)
        sound_play(snd, SND_BLIP, 1.6f + scr_randf(sc) * 0.3f, 0.12f);
      sc->term_wait += sc->typed < len ? 0.035f + scr_randf(sc) * 0.045f : 0.28f;
    } else { // enter
      char line[TERM_W + 16];
      cstr_format(line, sizeof(line), "quill@lab:~$ %", fmt_cstr(sc->queue[i]));
      term_push(sc, line, TC_CMD);
      sc->typing[0] = 0;
      sc->typed = 0;
      sc->queue_i++;
      sc->term_wait += 0.25f + 0.3f; // a breath before the next command starts typing
    }
  }
}

// ---------------------------------------------------------------- open, close, update

hz_internal void screens_tv_line(Screens *sc, EcsWorld *w, AudioSystem *audio) {
  clip_stop(&sc->voice, 0.1f);
  TvLine ln = tv_line(sc->channel, sc->weather, sc->line);
  sc->voice = (HzHandleT(AudioVoice)){0};
  sc->voiced = ln.voice != NULL;
  if (ln.voice)
    clip_play(w, ln.voice, &(EcsAudioPlayDesc){.volume = 0.95f, .bus = AUDIO_BUS_VOICE}, &sc->voice);
  sc->line_t = 0.0f;
}

// `x, z`: where the target that opened it stands (picks which TV or computer)
hz_internal void screens_open(Screens *sc, ScreenKind kind, EcsWorld *w, AudioSystem *audio, u32 weather, f32 x, f32 z, u32 seed) {
  *sc = (Screens){.kind = kind, .weather = weather, .click_key = -1, .click_app = -1, .rng = seed | 1u};
  sc->has_board = screen_board(kind, x, z, &sc->board);
  sc->fact = scr_rand(sc);
  if (kind == SCREEN_TV) {
    sc->line_t = -0.5f; // the first line after the picture is up
    screens_tv_line(sc, w, audio);
  }
  if (kind == SCREEN_LAB) {
    term_push(sc, "QuillOS 3.7 \xc2\xb7 Brick Beast Research Terminal", TC_TITLE);
    term_push(sc, "read-only session \xc2\xb7 logged by Prof. Quill", TC_DIM);
    sc->term_wait = 1.0f;
  }
}

hz_internal void screens_close(Screens *sc, AudioSystem *audio) {
  clip_stop(&sc->voice, 0.15f);
  sc->kind = SCREEN_NONE;
}

// main lane, before the scope
hz_internal void screens_update(Screens *sc, EcsWorld *w, AudioSystem *audio, SoundBank *snd, const InputSystem *in, const GameSave *g, f32 dt) {
  if (sc->kind == SCREEN_NONE)
    return;
  if (in->buttons[KEY_ESCAPE].pressed_this_frame || in->buttons[KEY_E].pressed_this_frame || sc->click_close) {
    screens_close(sc, audio);
    return;
  }
  sc->time += dt;
  sc->app_t += dt;
  sc->noise_t = m_maxf(0.0f, sc->noise_t - dt);
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
      sc->noise_t = 0.35f;
      sound_play(snd, SND_BLIP, 0.7f, 0.5f);
      screens_tv_line(sc, w, audio);
      return;
    }
    sc->line_t += dt;
    // next line: the voice is over (plus a breath), or a while for a line without one
    TvLine ln = tv_line(sc->channel, sc->weather, sc->line);
    b32 over = sc->voiced ? (sc->line_t > 1.2f && !clip_pending(&sc->voice) && !audio_voice_playing(audio, sc->voice))
                          : sc->line_t > 1.2f + (f32)cstr_len(ln.text) * 0.055f;
    if (over || sc->line_t > 14.0f) {
      sc->line = (sc->line + 1) % tv_line_count(sc->channel, sc->weather);
      screens_tv_line(sc, w, audio);
      if (sc->line == 0)
        sc->line_t = -1.9f; // a longer pause before the show starts over
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
  } else if (sc->kind == SCREEN_PC) {
    i32 app = sc->click_app;
    if (sc->app != APP_HOME && (in->buttons[KEY_BACKSPACE].pressed_this_frame || in->buttons[KEY_LEFT].pressed_this_frame))
      app = APP_HOME;
    if (sc->app == APP_HOME)
      for (i32 k = 1; k < APP_COUNT; k++)
        if (in->buttons[KEY_1 + k - 1].pressed_this_frame)
          app = k;
    sc->click_app = -1;
    if (app >= 0 && app != (i32)sc->app) {
      sc->app = (PcApp)app;
      sc->app_t = 0.0f;
      sound_play(snd, SND_BLIP, app == APP_HOME ? 1.1f : 1.4f, 0.35f);
    }
  } else if (sc->kind == SCREEN_LAB && sc->time > 0.5f) {
    term_update(sc, g, snd, dt);
  }
}

// ---------------------------------------------------------------- drawing helpers

#define SCR_Z 30 // the screens' layer (every floating piece inside carries it, or it would draw under the screen)

hz_internal void scr_text(CString s, f32 size, HzUIColor c, b32 wrap) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = m_maxf(6.0f, size), .textColor = c, .wrapMode = wrap ? UI_TEXT_WRAP_WORDS : UI_TEXT_WRAP_NONE}));
}
hz_internal void scr_textw(CString s, f32 size, HzUIColor c, u16 weight, b32 wrap) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = m_maxf(6.0f, size), .textColor = c, .weight = weight,
                                       .wrapMode = wrap ? UI_TEXT_WRAP_WORDS : UI_TEXT_WRAP_NONE}));
}

hz_internal Optional(HzUITransform) scr_tf(f32 tx, f32 ty, f32 sx, f32 sy, f32 rot, f32 ox, f32 oy) {
  return opt_some(HzUITransform, ((HzUITransform){.scale = {sx, sy}, .translate = {tx, ty}, .rotation = rot, .origin = {ox, oy}}));
}

// a glyph of the art set filling a w x h box at (x, y) of the parent (its viewBox scaled to the box), coloured c
hz_internal void scr_glyph(CString g, f32 x, f32 y, f32 w, f32 h, HzUIColor c, Optional(HzUITransform) tf) {
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {x, y}},
              .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
              .transform = tf}) {
    ui_text(ui_str_c(g), ui_text_config({.fontId = (u16)g_art.idx, .fontSize = h, .lineHeight = h, .textColor = c}));
  }
}

// an icon (176 x 176 art) of size `s` in the flow
hz_internal void scr_icon(CString g, f32 s, HzUIColor c) {
  ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(s), .height = ui_sizing_fixed(s)}}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {
    ui_text(ui_str_c(g), ui_text_config({.fontId = (u16)g_art.idx, .fontSize = s, .lineHeight = s, .textColor = c}));
  }
}

// the scene canvas: a viewBox of 320 x 176 drawn to cover the screen (svg "xMidYMid slice"); k = px per unit
typedef struct ScrCanvas {
  f32 k, ox, oy;
} ScrCanvas;

hz_internal ScrCanvas scr_canvas(f32 w, f32 h) {
  f32 k = m_maxf(w / 320.0f, h / 176.0f);
  return (ScrCanvas){k, (w - 320.0f * k) * 0.5f, (h - 176.0f * k) * 0.5f};
}

// one layer (part) of a scene, in its colour, moved by (tx, ty) viewBox units, scaled / turned about (px, py)
hz_internal void scr_layer(ScrCanvas cv, CString g, HzUIColor c, f32 tx, f32 ty, f32 sx, f32 sy, f32 rot, f32 px, f32 py) {
  b32 moved = tx != 0.0f || ty != 0.0f || sx != 1.0f || sy != 1.0f || rot != 0.0f;
  scr_glyph(g, cv.ox, cv.oy, 320.0f * cv.k, 176.0f * cv.k, c,
            moved ? scr_tf(tx * cv.k, ty * cv.k, sx, sy, rot, px / 320.0f, py / 176.0f) : opt_none(HzUITransform));
}
#define LAYER(cv, g, c) scr_layer((cv), (g), (c), 0, 0, 1, 1, 0, 160, 88)

// a rounded box in viewBox units
hz_internal void scr_box(ScrCanvas cv, f32 x, f32 y, f32 w, f32 h, f32 r, HzUIColor c, f32 rot) {
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + x * cv.k, cv.oy + y * cv.k}},
              .layout = {.sizing = {.width = ui_sizing_fixed(w * cv.k), .height = ui_sizing_fixed(h * cv.k)}},
              .background_color = c,
              .corner_radius = ui_corner_radius(r * cv.k),
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
              .transform = rot != 0.0f ? scr_tf(0, 0, 1, 1, rot, 0.5f, 0.5f) : opt_none(HzUITransform)}) {}
}

// a full-screen overlay piece over the w x h screen (gradients, scanlines)
hz_internal void scr_cover(f32 w, f32 h, UIGradient g, const HzUIBackgrounds *bg, HzUIColor c) {
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z},
              .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
              .gradient = g,
              .backgrounds = bg,
              .background_color = c,
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {}
}

// css repeating-linear-gradient(0deg, rgb(0 0 0 / a) 0 1px, transparent 1px 3px): CRT scanlines
hz_internal void scr_scanlines(f32 w, f32 h, f32 a) {
  UIGradient g = {.kind = UI_GRADIENT_LINEAR, .angle = 0.0f, .stop_count = 4, .repeating = true};
  g.stops[0] = (HzUIGradientStop){0.0f, cu_black(a)};
  g.stops[1] = (HzUIGradientStop){0.0f, cu_black(a)};
  g.stops[2] = (HzUIGradientStop){0.0f, cu_black(0.0f)};
  g.stops[3] = (HzUIGradientStop){0.0f, cu_black(0.0f)};
  g.stop_offset_px[1] = 1.0f;
  g.stop_offset_px[2] = 1.0f;
  g.stop_offset_px[3] = 3.0f;
  scr_cover(w, h, g, NULL, (HzUIColor){0});
}

// a dark edge (a CRT's curve, a camera lens): radial, clear inside `t0`
hz_internal void scr_vignette(f32 w, f32 h, f32 t0, f32 a) {
  UIGradient g = {.kind = UI_GRADIENT_RADIAL, .stop_count = 2, .size = UI_RADIAL_FARTHEST_CORNER, .at = opt_some(vec2, ((vec2){0.5f, 0.5f}))};
  g.stops[0] = (HzUIGradientStop){t0, cu_black(0.0f)};
  g.stops[1] = (HzUIGradientStop){1.0f, cu_black(a)};
  scr_cover(w, h, g, NULL, (HzUIColor){0});
}

// a round button with an icon; true when clicked this frame
hz_internal b32 scr_round_button(HzUIElementId id, CString icon, f32 d, HzUIColor bg, HzUIColor fg, const InputSystem *in) {
  b32 clicked = false;
  ui_element({.id = id,
              .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = bg,
              .corner_radius = ui_corner_radius(d * 0.5f),
              .shadow = {.color = cu_black(0.3f), .offset_y = 2.0f, .blur = 6.0f},
              .brightness = ui_hovered() ? opt_some(f32, 1.15f) : opt_none(f32)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    scr_icon(icon, d * 0.56f, fg);
  }
  return clicked;
}

// ---------------------------------------------------------------- the TV

hz_internal u32 scr_golden(const GameSave *g) {
  u32 n = 0;
  for (u32 k = 0; k < 15; k++)
    n += (g->golden_mask >> k) & 1u;
  return n;
}

hz_internal f32 scr_wave(f32 t, f32 period) { return 0.5f - 0.5f * m_cosf(t * 2.0f * M_PI / period); } // 0 -> 1 -> 0

// static between channels: grey blocks that change every frame
hz_internal void tv_static(Screens *sc, f32 w, f32 h, f32 a) {
  u32 seed = (u32)(sc->time * 60.0f) * 2654435761u;
  i32 cols = 24, rows = 14;
  f32 cw = w / (f32)cols, ch = h / (f32)rows;
  for (i32 y = 0; y < rows; y++)
    for (i32 x = 0; x < cols; x++) {
      seed = seed * 1664525u + 1013904223u;
      f32 v = (f32)(seed >> 24) / 255.0f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {(f32)x * cw, (f32)y * ch}},
                  .layout = {.sizing = {.width = ui_sizing_fixed(cw + 1.0f), .height = ui_sizing_fixed(ch + 1.0f)}},
                  .background_color = cu_rgba(0x101010 + 0x010101 * (u32)(v * 220.0f), a),
                  .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {}
    }
}

// a shape in viewBox units: a box (r = corner radius; a circle when r is half its size) with an svg-like stroke `lw`
// centred on its edge; scaled (sx, sy) and turned (degrees) about (ox, oy), fractions of the box
hz_internal void scr_shape(ScrCanvas cv, f32 x, f32 y, f32 w, f32 h, f32 r, HzUIColor fill, HzUIColor line, f32 lw, f32 sx, f32 sy, f32 rot,
                           f32 ox, f32 oy) {
  f32 k = cv.k, e = lw * 0.5f;
  u16 b = (u16)m_maxf(lw > 0.0f ? 1.0f : 0.0f, lw * k + 0.5f);
  b32 moved = sx != 1.0f || sy != 1.0f || rot != 0.0f;
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + (x - e) * k, cv.oy + (y - e) * k}},
              .layout = {.sizing = {.width = ui_sizing_fixed((w + lw) * k), .height = ui_sizing_fixed((h + lw) * k)}},
              .background_color = fill,
              .corner_radius = ui_corner_radius((r + e) * k),
              .border = {.color = line, .width = {.left = b, .right = b, .top = b, .bottom = b}},
              .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
              .transform = moved ? scr_tf(0, 0, sx, sy, rot, ox, oy) : opt_none(HzUITransform)}) {}
}
#define SHAPE(cv, x, y, w, h, r, fill) scr_shape((cv), (x), (y), (w), (h), (r), (fill), (HzUIColor){0}, 0, 1, 1, 0, 0.5f, 0.5f)
hz_internal void scr_circle(ScrCanvas cv, f32 cx, f32 cy, f32 r, HzUIColor fill, HzUIColor line, f32 lw, f32 sy) {
  scr_shape(cv, cx - r, cy - r, r * 2.0f, r * 2.0f, r, fill, line, lw, 1, sy, 0, 0.5f, 0.5f);
}
#define CIRCLE(cv, x, y, r, fill) scr_circle((cv), (x), (y), (r), (fill), (HzUIColor){0}, 0, 1)

// BBN: the studio, the anchor (bobs, blinks, talks while a voice plays), the desk, the lower third and the ticker
hz_internal void tv_bbn(Screens *sc, ScrCanvas cv, f32 w, f32 h, const GameSave *g, b32 talking) {
  f32 t = sc->time, k = cv.k;
  scr_cover(w, h, cu_lin2(180.0f, cu_rgb(0x1e5aa8), cu_rgb(0x0b2550)), NULL, (HzUIColor){0});
  for (i32 i = 0; i < 6; i++) // studio lights
    CIRCLE(cv, 30.0f + (f32)i * 52.0f, 30, 14, cu_rgba(0xffffff, 0.07f + 0.05f * scr_wave(t + (f32)i * 0.5f, 3.0f)));
  // the big screen behind the anchor, with the turning star
  scr_shape(cv, 176, 22, 120, 70, 6, cu_rgb(0x0d1b2a), cu_rgb(0xffd23d), 2, 1, 1, 0, 0.5f, 0.5f);
  scr_layer(cv, ART_TV_BBN, cu_rgb(0xffd23d), 0, 0, 1, 1, t * 60.0f, 236, 57);
  // the anchor, a friendly brick beast
  f32 ax = 92.0f, ay = 66.0f - 2.0f * scr_wave(t, 1.6f);
  f32 bp = m_fmodf(t, 3.2f) / 3.2f; // blink: shut at 95% of every 3.2 s
  f32 blink = bp < 0.9f ? 1.0f : bp < 0.95f ? m_lerpf(1.0f, 0.12f, (bp - 0.9f) / 0.05f) : m_lerpf(0.12f, 1.0f, (bp - 0.95f) / 0.05f);
  f32 mp = m_fmodf(t, 0.7f) / 0.7f; // talking: 2 6 2 4 2 over 0.7 s
  f32 mouth = mp < 0.25f ? m_lerpf(2, 6, mp / 0.25f) : mp < 0.5f ? m_lerpf(6, 2, (mp - 0.25f) / 0.25f)
            : mp < 0.75f ? m_lerpf(2, 4, (mp - 0.5f) / 0.25f) : m_lerpf(4, 2, (mp - 0.75f) / 0.25f);
  if (!talking)
    mouth = 2.0f;
  HzUIColor green = cu_rgb(0x4fc06a), dark = cu_rgb(0x1e7a35), ink = cu_rgb(0x1b2a34);
  scr_shape(cv, ax - 30, ay, 60, 58, 10, green, dark, 3, 1, 1, 0, 0.5f, 0.5f);
  for (i32 i = -1; i <= 1; i++)
    scr_shape(cv, ax + (f32)i * 18.0f - 6.0f, ay - 6.0f, 12, 8, 3, green, dark, 2, 1, 1, 0, 0.5f, 0.5f);
  CIRCLE(cv, ax - 12, ay + 22, 7, cu_rgb(0xffffff));
  CIRCLE(cv, ax + 12, ay + 22, 7, cu_rgb(0xffffff));
  scr_circle(cv, ax - 11, ay + 23, 3.5f, ink, (HzUIColor){0}, 0, blink);
  scr_circle(cv, ax + 13, ay + 23, 3.5f, ink, (HzUIColor){0}, 0, blink);
  SHAPE(cv, ax - 8, ay + 40 - mouth, 16, mouth * 2.0f, mouth, ink);
  scr_shape(cv, ax - 5, ay + 50, 10, 8, 1.5f, cu_rgb(0xc91a09), (HzUIColor){0}, 0, 1, 1, 0, 0.5f, 0.5f); // the tie
  // the desk
  SHAPE(cv, 20, 118, 150, 40, 4, cu_rgb(0xc91a09));
  SHAPE(cv, 20, 118, 150, 8, 3, cu_rgb(0xff5a4a));
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + 20.0f * k, cv.oy + 120.0f * k}},
              .layout = {.sizing = {.width = ui_sizing_fixed(150.0f * k), .height = ui_sizing_fixed(18.0f * k)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}}}) {
    scr_textw("BBN", 13.0f * k, cu_rgb(0xffffff), 800, false);
  }
  // the lower third: LIVE and the headline, then the ticker
  CString news[5];
  news[0] = "Glimmerlings spotted folding light into tiny bricks!";
  cstr_format(sc->text[0], 96, "Local trainer owns % of % Brick Beast cards", fmt_u32(owned_count(g)), fmt_i32(CARD_POOL));
  news[1] = sc->text[0];
  news[2] = g->rival_wins ? "BREAKING: Dex loses again, demands a rematch" : "Dex claims his deck is \"unbeatable\"";
  cstr_format(sc->text[1], 96, "% golden studs found so far. Where are the rest?", fmt_u32(scr_golden(g)));
  news[3] = sc->text[1];
  news[4] = "Mom reminds everyone: jackets on Route 1!";
  f32 fs = h / 15.0f, bar_h = fs * 1.45f;
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {0, h - bar_h * 2.1f}},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(bar_h * 1.1f)},
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.3f), .padding = {.left = (u16)(fs * 0.5f), .right = (u16)(fs * 0.5f)},
                           .sizing = {.height = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_rgb(0xc91a09)}) {
      ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(fs * 0.45f), .height = ui_sizing_fixed(fs * 0.45f)}},
                  .background_color = cu_rgba(0xffffff, m_fmodf(t, 1.2f) < 0.7f ? 1.0f : 0.25f),
                  .corner_radius = ui_corner_radius(fs)}) {}
      scr_textw("LIVE", fs * 0.8f, cu_rgb(0xffffff), 800, false);
    }
    ui_element({.layout = {.padding = {.left = (u16)(fs * 0.5f), .right = (u16)(fs * 0.5f)}, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                           .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .gradient = cu_lin2(90.0f, cu_rgba(0xffffff, 0.97f), cu_rgba(0xe8eef6, 0.92f))}) {
      scr_textw(news[(u32)(t / 5.0f) % 5], fs * 0.8f, cu_rgb(0x0b2550), 700, false);
    }
  }
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {0, h - bar_h}},
              .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(bar_h)}},
              .background_color = cu_rgb(0x0b1d3a)}) {
    cstr_format(sc->text[2], 96, "%   \xe2\x98\x85   %   \xe2\x98\x85   ", fmt_cstr(news[0]), fmt_cstr(news[1]));
    cstr_format(sc->text[3], 96, "%   \xe2\x98\x85   %   \xe2\x98\x85   ", fmt_cstr(news[2]), fmt_cstr(news[3]));
    cstr_format(sc->text[4], 96, "%   \xe2\x98\x85   ", fmt_cstr(news[4]));
    HzUIElementData td = ui_get_element_data(ui_id("TvTicker"));
    f32 tw = td.found ? m_maxf(td.boundingBox.width, 100.0f) : 2000.0f;
    f32 x = w - m_fmodf(t * fs * 3.2f, tw + w);
    ui_element({.id = ui_id("TvTicker"),
                .floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {x, 0}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.height = ui_sizing_fixed(bar_h)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
      for (i32 i = 2; i <= 4; i++)
        scr_textw(sc->text[i], fs * 0.78f, cu_rgb(0xffd23d), 700, false);
    }
  }
}

// the weather: sun (or moon and stars, or rain), drifting clouds, the hills and the forecast
hz_internal void tv_weather(Screens *sc, ScrCanvas cv, f32 w, f32 h) {
  f32 t = sc->time;
  b32 night = sc->weather == 1, rain = sc->weather == 2;
  HzUIColor sky = night ? cu_rgb(0x16213e) : rain ? cu_rgb(0x7f97ab) : cu_rgb(0x7cc6ff);
  scr_cover(w, h, cu_lin2(180.0f, sky, night ? cu_rgb(0x24305a) : rain ? cu_rgb(0xa9bccb) : cu_rgb(0xc4e6ff)), NULL, (HzUIColor){0});
  if (night) {
    for (i32 i = 0; i < 18; i++) {
      f32 a = 0.2f + 0.8f * scr_wave(t + (f32)i * 0.37f, 2.0f + (f32)(i % 3));
      CIRCLE(cv, (f32)((i * 53) % 320), (f32)((i * 37) % 110), 1.4f, cu_rgba(0xffffff, a));
    }
    CIRCLE(cv, 230, 58, 26, cu_rgb(0xfff6c8));
    CIRCLE(cv, 242, 50, 24, sky);
  } else if (!rain) {
    for (i32 i = 0; i < 12; i++) { // the rays turn once in 14 s
      f32 a = ((f32)i * 30.0f + t * 360.0f / 14.0f) * M_PI / 180.0f;
      scr_shape(cv, 230.0f + 39.0f * m_sinf(a) - 3.0f, 58.0f - 39.0f * m_cosf(a) - 7.0f, 6, 14, 3, cu_rgb(0xffd23d), (HzUIColor){0}, 0, 1, 1,
                a * 180.0f / M_PI, 0.5f, 0.5f);
    }
    scr_circle(cv, 230, 58, 26, cu_rgb(0xffd23d), cu_rgb(0xf2a705), 3, 1);
  }
  static const f32 drift[3] = {18.0f, 24.0f, 16.0f};
  CString clouds[3] = {ART_TV_WEATHER_PART_0, ART_TV_WEATHER_PART_1, ART_TV_WEATHER_PART_2};
  for (i32 i = 0; i < 3; i++) {
    f32 dx = -20.0f + 40.0f * scr_wave(t, drift[i]);
    scr_layer(cv, clouds[i], rain ? cu_rgb(0xdfe6ec) : night ? cu_rgba(0xc8d0e8, 0.55f) : cu_rgba(0xffffff, 0.95f), dx, 0, 1, 1, 0, 160, 88);
  }
  if (rain)
    for (i32 i = 0; i < 46; i++) {
      f32 y = m_fmodf(t * 230.0f + (f32)(i * 37), 200.0f) - 20.0f;
      f32 x = (f32)((i * 61) % 340) - 10.0f - y * 0.27f;
      scr_shape(cv, x, y, 1.4f, 11.0f, 0.7f, cu_rgba(0xffffff, 0.55f), (HzUIColor){0}, 0, 1, 1, 15.0f, 0.5f, 0.5f);
    }
  LAYER(cv, ART_TV_WEATHER_PART_3, night ? cu_rgb(0x1f4d22) : cu_rgb(0x3e9a32));
  LAYER(cv, ART_TV_WEATHER_PART_4, night ? cu_rgb(0x2a6b2f) : cu_rgb(0x4fc06a));
  // the forecast card, top left
  f32 fs = h / 15.0f;
  i32 temp = night ? 12 : rain ? 15 : 21;
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {fs * 0.8f, fs * 0.8f}},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.6f), .padding = {.left = (u16)(fs * 0.7f), .right = (u16)(fs * 0.9f), .top = (u16)(fs * 0.4f), .bottom = (u16)(fs * 0.4f)},
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgba(0xffffff, 0.86f),
              .corner_radius = ui_corner_radius(fs * 0.7f),
              .shadow = {.color = cu_black(0.25f), .offset_y = fs * 0.15f, .blur = fs * 0.6f}}) {
    cstr_format(sc->text[5], 96, "%\xc2\xb0", fmt_i32(temp));
    scr_textw(sc->text[5], fs * 2.2f, night ? cu_rgb(0x24305a) : cu_rgb(0xe0781a), 800, false);
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 2}}) {
      scr_textw("Brickhollow", fs * 0.95f, cu_rgb(0x1b2a34), 800, false);
      scr_textw(night ? "Clear night \xc2\xb7 fireflies likely" : rain ? "Rain \xc2\xb7 puddles everywhere" : "Sunny \xc2\xb7 perfect for battles",
                fs * 0.7f, cu_rgb(0x5c6670), 600, false);
    }
  }
}

// cartoons: "Brick & Stud" - Brick bounces along the road, Stud rolls after him
hz_internal void tv_cartoon(Screens *sc, ScrCanvas cv, f32 w, f32 h) {
  f32 t = sc->time, k = cv.k;
  scr_cover(w, h, (UIGradient){0}, NULL, cu_rgb(0xffe7a8));
  scr_layer(cv, ART_TV_CARTOON, cu_rgba(0xf2a705, 0.5f), -160.0f * m_fmodf(t / 4.0f, 1.0f), 0, 1, 1, 0, 160, 88);
  SHAPE(cv, 0, 150, 320, 26, 0, cu_rgb(0xc96a2a));
  f32 sx = -40.0f * m_fmodf(t / 0.8f, 1.0f);
  for (i32 i = 0; i < 9; i++)
    SHAPE(cv, sx + (f32)i * 40.0f, 150, 20, 5, 0, cu_rgb(0xa85520));
  // Brick: a ball's arc, squashed a little where he lands
  f32 p = m_fmodf(t / 0.8f, 1.0f), up = 1.0f - (1.0f - 2.0f * p) * (1.0f - 2.0f * p);
  f32 sq = up < 0.1f ? 0.18f * (1.0f - up / 0.1f) : 0.0f;
  f32 bx = 130.0f, by = 120.0f - 56.0f * up;
  HzUIColor red = cu_rgb(0xc91a09), dark = cu_rgb(0x7a0f05), ink = cu_rgb(0x1b2a34);
  f32 bw = 48.0f * (1.0f + sq), bh = 30.0f * (1.0f - sq);
  for (i32 i = -1; i <= 1; i += 2)
    scr_shape(cv, bx + (f32)i * 12.0f * (1.0f + sq) - 7.0f, by - bh - 7.0f, 14, 8, 3, red, dark, 2, 1, 1, 0, 0.5f, 0.5f);
  scr_shape(cv, bx - bw * 0.5f, by - bh, bw, bh, 5, red, dark, 3, 1, 1, 0, 0.5f, 0.5f);
  CIRCLE(cv, bx - 8, by - bh * 0.57f, 4, cu_rgb(0xffffff));
  CIRCLE(cv, bx + 10, by - bh * 0.57f, 4, cu_rgb(0xffffff));
  CIRCLE(cv, bx - 7, by - bh * 0.57f, 2, ink);
  CIRCLE(cv, bx + 11, by - bh * 0.57f, 2, ink);
  // his smile: the bottom of a ring
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + (bx - 6.0f) * k, cv.oy + (by - bh * 0.4f - 4.0f) * k}},
              .layout = {.sizing = {.width = ui_sizing_fixed(14.0f * k), .height = ui_sizing_fixed(7.0f * k)}},
              .corner_radius = {.bottom_left = 7.0f * k, .bottom_right = 7.0f * k},
              .border = {.color = ink, .width = {.bottom = (u16)(2.5f * k), .left = (u16)(1.0f * k), .right = (u16)(1.0f * k)}}}) {}
  // Stud, rolling after him
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + 46.5f * k, cv.oy + 124.5f * k}},
              .layout = {.sizing = {.width = ui_sizing_fixed(27.0f * k), .height = ui_sizing_fixed(27.0f * k)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgb(0xffd23d),
              .corner_radius = ui_corner_radius(13.5f * k),
              .border = {.color = cu_rgb(0xa87a00), .width = {.left = (u16)(3.0f * k), .right = (u16)(3.0f * k), .top = (u16)(3.0f * k), .bottom = (u16)(3.0f * k)}, .overlay = true},
              .transform = scr_tf(0, 0, 1, 1, t * 300.0f, 0.5f, 0.5f)}) {
    ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(12.0f * k), .height = ui_sizing_fixed(4.0f * k)}}, .background_color = cu_rgb(0xa87a00),
                .corner_radius = ui_corner_radius(2.0f * k)}) {}
  }
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                           .offset = {-(w - (cv.ox + 300.0f * k)), cv.oy + 12.0f * k}}}) {
    HzUITextConfig c = {.fontSize = 15.0f * k, .textColor = red, .weight = 800};
    c.shadows[0] = (HzUITextShadow){.color = cu_rgb(0xffffff), .offset_y = 1.5f * k, .blur = 0.0f};
    ui_text(ui_str_c("BRICK & STUD"), c);
  }
}

// a nature film: dusk, the camera pans over the meadow, a little beast wobbles to a lost 2x4 brick
hz_internal void tv_wild(Screens *sc, ScrCanvas cv, f32 w, f32 h) {
  f32 t = sc->time, k = cv.k;
  scr_cover(w, h, cu_lin2(180.0f, cu_rgb(0xffcf8a), cu_rgb(0xffefc9)), NULL, (HzUIColor){0});
  CIRCLE(cv, 250, 60, 22, cu_rgba(0xfff3c4, 0.9f));
  f32 pan = -30.0f * scr_wave(t, 24.0f);
  scr_layer(cv, ART_TV_WILD, cu_rgb(0x6bb04a), pan, 0, 1, 1, 0, 160, 88);
  // the little beast wobbles over to the brick and back
  f32 p = m_fmodf(t / 9.0f, 1.0f);
  f32 bx = 120.0f + pan + 16.0f * (p < 0.5f ? p * 2.0f : 2.0f - p * 2.0f), by = 118.0f - 3.0f * m_absf(m_sinf(p * 4.0f * M_PI));
  f32 look = m_sinf(t * 2.0f * M_PI / 5.0f);
  HzUIColor blue = cu_rgb(0x3d9bff), dark = cu_rgb(0x1e5aa8), ink = cu_rgb(0x1b2a34);
  for (i32 i = -1; i <= 1; i += 2)
    scr_shape(cv, bx + (f32)i * 7.0f - 4.0f, by - 23.0f, 8, 6, 2, blue, dark, 1.5f, 1, 1, 0, 0.5f, 0.5f);
  scr_shape(cv, bx - 14.0f, by - 18.0f, 28, 18, 4, blue, dark, 2, 1, 1, 0, 0.5f, 0.5f);
  CIRCLE(cv, bx - 5, by - 10, 3, cu_rgb(0xffffff));
  CIRCLE(cv, bx + 6, by - 10, 3, cu_rgb(0xffffff));
  CIRCLE(cv, bx - 4 + look, by - 10, 1.5f, ink);
  CIRCLE(cv, bx + 7 + look, by - 10, 1.5f, ink);
  // the lost 2x4 brick
  SHAPE(cv, 184 + pan, 110, 24, 7, 1.5f, cu_rgb(0xc91a09));
  for (i32 i = 0; i < 4; i++)
    SHAPE(cv, 185 + pan + (f32)i * 6.0f, 107, 4, 3, 1, cu_rgb(0xc91a09));
  // tall grass in front, each blade swaying about its root
  for (i32 i = 0; i < 26; i++) {
    f32 bh = 44.0f + (f32)((i * 11) % 26), lean = (i % 2 ? 7.0f : -5.0f) + 3.0f * m_sinf(t * 2.0f * M_PI / (3.0f + (f32)(i % 4)) + (f32)i);
    scr_shape(cv, (f32)i * 14.0f - 12.0f + pan, 176.0f - bh, 4, bh + 4, 2, i % 2 ? cu_rgb(0x3e8a2a) : cu_rgb(0x4f9e33), (HzUIColor){0}, 0, 1, 1, lean,
              0.5f, 1.0f);
  }
  scr_vignette(w, h, 0.62f, 0.7f); // the lens
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {cv.ox + 16.0f * k, cv.oy + 12.0f * k}}}) {
    scr_textw("WILD BRICKS", 12.0f * k, cu_rgba(0xffffff, 0.85f), 800, false);
  }
}

hz_internal void tv_draw(Screens *sc, f32 w, f32 h, const GameSave *g, b32 talking) {
  ScrCanvas cv = scr_canvas(w, h);
  switch (sc->channel) {
  case 0: tv_bbn(sc, cv, w, h, g, talking); break;
  case 1: tv_weather(sc, cv, w, h); break;
  case 2: tv_cartoon(sc, cv, w, h); break;
  default: tv_wild(sc, cv, w, h); break;
  }
  // the subtitle (the news has it at the top, over the studio)
  if (sc->line_t >= 0.0f) {
    TvLine ln = tv_line(sc->channel, sc->weather, sc->line);
    f32 fs = h / 17.0f;
    f32 pop = m_clampf(sc->line_t / 0.18f, 0.0f, 1.0f);
    b32 top = sc->channel == 0;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z,
                             .attach_points = top ? (HzUIFloatingAttachPoints){.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP}
                                                  : (HzUIFloatingAttachPoints){.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0, top ? fs * 0.6f : -fs * 0.6f}},
                .layout = {.padding = {.left = (u16)(fs * 0.6f), .right = (u16)(fs * 0.6f), .top = (u16)(fs * 0.3f), .bottom = (u16)(fs * 0.35f)},
                           .sizing = {.width = ui_sizing_fit(0, w * 0.88f)}},
                .background_color = cu_black(0.72f),
                .corner_radius = ui_corner_radius(fs * 0.35f),
                .opacity = opt_some(f32, pop),
                .transform = cu_tf(0, (1.0f - pop) * fs * 0.4f, 1, 0)}) {
      cstr_format(sc->text[6], 96, "%: ", fmt_cstr(ln.speaker));
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT}}) {
        HzUITextConfig c = {.fontSize = fs, .textColor = cu_rgb(0xffffff), .weight = 600, .wrapMode = UI_TEXT_WRAP_WORDS};
        HzUITextConfig b = c;
        b.weight = 800;
        b.textColor = cu_rgb(0xffd23d);
        ui_text(ui_str_c(sc->text[6]), b);
        ui_text(ui_str_c(ln.text), c);
      }
    }
  }
  if (sc->noise_t > 0.0f)
    tv_static(sc, w, h, m_clampf(sc->noise_t / 0.12f, 0.0f, 1.0f));
  scr_scanlines(w, h, 0.18f);
  scr_vignette(w, h, 0.7f, 0.35f);
  // a soft reflection on the glass
  scr_cover(w, h, cu_lin(135.0f, 3, (f32[]){0.0f, 0.35f, 0.36f}, (HzUIColor[]){cu_rgba(0xffffff, 0.10f), cu_rgba(0xffffff, 0.03f), cu_rgba(0xffffff, 0.0f)}),
            NULL, (HzUIColor){0});
}

// the remote under the TV: channel down / name / up, power
hz_internal void tv_osd(Screens *sc, const InputSystem *in, f32 fs) {
  ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.5f), .padding = ui_padding_all((u16)(fs * 0.35f)),
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgba(0x16191f, 0.92f),
              .corner_radius = ui_corner_radius(fs * 2.0f),
              .shadow = {.color = cu_black(0.4f), .offset_y = 3.0f, .blur = 12.0f}}) {
    f32 d = fs * 2.3f;
    if (scr_round_button(ui_id("TvPrev"), ART_IC_CHEVRON_LEFT, d, cu_rgb(0x2b313b), cu_rgb(0xffffff), in))
      sc->click_zap = -1;
    ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(fs * 9.0f)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
      cstr_format(sc->text[7], 96, "CH %  \xc2\xb7  %", fmt_i32(sc->channel + 1), fmt_cstr(tv_channel_name(sc->channel)));
      scr_textw(sc->text[7], fs, cu_rgb(0xffffff), 700, false);
    }
    if (scr_round_button(ui_id("TvNext"), ART_IC_CHEVRON_RIGHT, d, cu_rgb(0x2b313b), cu_rgb(0xffffff), in))
      sc->click_zap = 1;
    ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(fs * 0.6f)}}}) {}
    if (scr_round_button(ui_id("TvOff"), ART_IC_POWER, d, cu_rgb(0xc91a09), cu_rgb(0xffffff), in))
      sc->click_close = true;
  }
}

// ---------------------------------------------------------------- the lab terminal

hz_internal HzUIColor term_color(TermCls c) {
  switch (c) {
  case TC_CMD: return cu_rgb(0xc8ffe0);
  case TC_DIM: return cu_rgb(0x2f9e64);
  case TC_WARN: return cu_rgb(0xffd23d);
  case TC_GENE: return cu_rgb(0x7fd8ff);
  case TC_TITLE: return cu_rgb(0xffffff);
  default: return cu_rgb(0x5dff9d);
  }
}

hz_internal HzUITextConfig term_cfg(f32 fs, HzUIColor c, u16 weight) {
  HzUITextConfig cfg = {.fontSize = fs, .lineHeight = fs * 1.3f, .textColor = c, .font_family = g_mono, .weight = weight};
  cfg.shadows[0] = (HzUITextShadow){.color = cu_rgba(0x5dff9d, 0.45f), .blur = fs * 0.5f};
  return cfg;
}

// a framed panel with a small title
#define TERM_PANEL(title_fn, fs)                                                                                         \
  ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 2, .padding = {.left = (u16)((fs) * 0.4f), .right = (u16)((fs) * 0.4f), .top = (u16)((fs) * 0.25f), .bottom = (u16)((fs) * 0.3f)}, \
                         .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},                           \
              .background_color = cu_rgba(0x0a321e, 0.45f),                                                              \
              .border = {.color = cu_rgb(0x1f6f4a), .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},                                               \
              .corner_radius = ui_corner_radius((fs) * 0.3f)})

hz_internal void lab_draw(Screens *sc, const InputSystem *in, f32 w, f32 h) {
  f32 fs = m_minf(h / 30.0f, w / 28.0f), t = sc->time;
  b32 narrow = w < 520.0f; // a phone upright: the panels go under the console, side by side
  // the title bar
  ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.45f), .padding = {.left = (u16)(fs * 0.7f), .right = (u16)(fs * 0.4f), .top = (u16)(fs * 0.3f), .bottom = (u16)(fs * 0.3f)},
                         .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgb(0x0b2a1b)}) {
    u32 dots[3] = {0xff5f56, 0xffbd2e, 0x27c93f};
    for (i32 i = 0; i < 3; i++)
      ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(fs * 0.7f), .height = ui_sizing_fixed(fs * 0.7f)}}, .background_color = cu_rgb(dots[i]),
                  .corner_radius = ui_corner_radius(fs)}) {}
    ui_element({.layout = {.padding = {.left = (u16)(fs * 0.3f)}}}) {
      ui_text(ui_str_c("quill@lab \xe2\x80\x94 research"), term_cfg(fs * 0.95f, cu_rgb(0x9dffc6), 700));
    }
    ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
    ui_element({.id = ui_id("LabOff"),
                .layout = {.padding = {.left = (u16)(fs * 0.35f), .right = (u16)(fs * 0.35f), .top = (u16)(fs * 0.2f), .bottom = (u16)(fs * 0.2f)}},
                .background_color = cu_rgb(0xc91a09),
                .corner_radius = ui_corner_radius(fs * 0.35f),
                .brightness = ui_hovered() ? opt_some(f32, 1.2f) : opt_none(f32)}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
        sc->click_close = true;
      scr_icon(ART_IC_POWER, fs * 1.3f, cu_rgb(0xffffff));
    }
  }
  ui_element({.layout = {.layout_direction = narrow ? UI_TOP_TO_BOTTOM : UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.5f), .padding = {.left = (u16)(fs * 0.7f), .right = (u16)(fs * 0.6f), .top = (u16)(fs * 0.4f), .bottom = (u16)(fs * 0.5f)},
                         .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
    // the console: the last lines, newest at the bottom, then the prompt with what is being typed
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM,
                           .sizing = {.width = narrow ? ui_sizing_grow(0) : ui_sizing_percent(0.64f), .height = ui_sizing_grow(0)},
                           .child_alignment = {.y = UI_ALIGN_Y_BOTTOM}},
                .clip = {.horizontal = true, .vertical = true}}) {
      u32 n = sc->term_n < TERM_LINES - 1 ? sc->term_n : TERM_LINES - 1;
      for (u32 i = sc->term_n - n; i < sc->term_n; i++) {
        u32 j = i % TERM_LINES;
        ui_text(ui_str_c(sc->term[j]), term_cfg(fs, term_color((TermCls)sc->term_cls[j]), sc->term_cls[j] == TC_TITLE ? 700 : 400));
      }
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
        cstr_format(sc->text[8], 96, "quill@lab:~$ %", fmt_cstr(sc->typing));
        ui_text(ui_str_c(sc->text[8]), term_cfg(fs, term_color(TC_CMD), 400));
        ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(fs * 0.55f), .height = ui_sizing_fixed(fs * 1.05f)}},
                    .background_color = cu_rgba(0x5dff9d, m_fmodf(t, 1.0f) < 0.5f ? 0.9f : 0.0f),
                    .shadow = {.color = cu_rgba(0x5dff9d, 0.5f), .blur = fs * 0.5f}}) {}
      }
    }
    // the science panels
    ui_element({.layout = {.layout_direction = narrow ? UI_LEFT_TO_RIGHT : UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.5f),
                           .sizing = {.width = ui_sizing_grow(0), .height = narrow ? ui_sizing_fixed(h * 0.3f) : ui_sizing_grow(0)}}}) {
      TERM_PANEL(0, fs) {
        ui_text(ui_str_c("GENOME HELIX"), term_cfg(fs * 0.7f, cu_rgb(0x9dffc6), 700));
        // a double helix of studs, turning (a 100 x 100 box)
        ui_element({.id = ui_id("LabHelix"), .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
          HzUIElementData hb = ui_get_element_data(ui_id("LabHelix"));
          f32 bw = hb.found ? hb.boundingBox.width : 100.0f, bh = hb.found ? hb.boundingBox.height : 100.0f;
          f32 s = m_minf(bw, bh) / 100.0f, ox = (bw - 100.0f * s) * 0.5f, oy = (bh - 100.0f * s) * 0.5f;
          for (i32 i = 0; i < 14; i++) {
            f32 y = 8.0f + (f32)i * 6.4f, a = t * 1.6f + (f32)i * 0.55f;
            f32 x1 = 50.0f + m_sinf(a) * 30.0f, x2 = 50.0f - m_sinf(a) * 30.0f, z = m_cosf(a);
            f32 l = m_minf(x1, x2);
            ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {ox + l * s, oy + (y - 0.6f) * s}},
                        .layout = {.sizing = {.width = ui_sizing_fixed(m_absf(x2 - x1) * s), .height = ui_sizing_fixed(1.2f * s)}},
                        .background_color = cu_rgb(0x1f6f4a)}) {}
            for (i32 e = 0; e < 2; e++) {
              f32 zz = e == 0 ? z : -z, r = (2.4f + zz * 0.9f) * s, cx = (e == 0 ? x1 : x2) * s;
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {ox + cx - r, oy + y * s - r}},
                          .layout = {.sizing = {.width = ui_sizing_fixed(r * 2.0f), .height = ui_sizing_fixed(r * 2.0f)}},
                          .background_color = zz > 0.0f ? cu_rgb(0x5dff9d) : cu_rgb(0x2a9d60),
                          .corner_radius = ui_corner_radius(r),
                          .shadow = {.color = zz > 0.0f ? cu_rgba(0x5dff9d, 0.5f) : (HzUIColor){0}, .blur = r}}) {}
            }
          }
        }
      }
      TERM_PANEL(1, fs) {
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.4f)}}) {
          ui_text(ui_str_c("STUD RESONANCE"), term_cfg(fs * 0.7f, cu_rgb(0x9dffc6), 700));
          ui_text(ui_str_c("\xe2\x97\x8f LIVE"), term_cfg(fs * 0.7f, cu_rgba(0xff5f56, m_fmodf(t, 1.2f) < 0.6f ? 1.0f : 0.15f), 700));
        }
        // the wave (a 100 x 40 box, stretched): grid lines and 59 segments
        ui_element({.id = ui_id("LabWave"), .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
          HzUIElementData wb = ui_get_element_data(ui_id("LabWave"));
          f32 bw = wb.found ? wb.boundingBox.width : 100.0f, bh = wb.found ? wb.boundingBox.height : 40.0f;
          f32 sx = bw / 100.0f, sy = bh / 40.0f;
          for (i32 gy = 1; gy <= 3; gy++)
            ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {0, (f32)gy * 10.0f * sy}},
                        .layout = {.sizing = {.width = ui_sizing_fixed(bw), .height = ui_sizing_fixed(1.0f)}},
                        .background_color = cu_rgb(0x14402c)}) {}
          f32 px = 0, py = 0;
          for (i32 i = 0; i < 60; i++) {
            f32 x = (f32)i / 59.0f * 100.0f;
            f32 y = 20.0f + m_sinf((f32)i * 0.35f + t * 4.0f) * 9.0f * m_sinf(t * 0.7f + (f32)i * 0.05f) + m_sinf((f32)i * 1.7f + t * 9.0f) * 2.0f;
            x *= sx;
            y *= sy;
            if (i > 0) {
              f32 dx = x - px, dy = y - py, len = m_sqrtf(dx * dx + dy * dy);
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z, .offset = {(px + x) * 0.5f - len * 0.5f, (py + y) * 0.5f - fs * 0.07f}},
                          .layout = {.sizing = {.width = ui_sizing_fixed(len + fs * 0.1f), .height = ui_sizing_fixed(fs * 0.14f)}},
                          .background_color = cu_rgb(0x5dff9d),
                          .corner_radius = ui_corner_radius(fs * 0.07f),
                          .shadow = {.color = cu_rgba(0x5dff9d, 0.6f), .blur = fs * 0.3f},
                          .transform = scr_tf(0, 0, 1, 1, m_atan2f(dy, dx) * 180.0f / M_PI, 0.5f, 0.5f)}) {}
            }
            px = x;
            py = y;
          }
        }
      }
    }
  }
}

// ---------------------------------------------------------------- BrickOS (your PC)

typedef struct PcAppDef {
  CString label, icon;
  u32 color;
} PcAppDef;

hz_internal PcAppDef pc_app(PcApp a) {
  switch (a) {
  case APP_MSG: return (PcAppDef){"Messages", ART_IC_MAIL, 0x3d9bff};
  case APP_STATS: return (PcAppDef){"Stats", ART_IC_CHART, 0x27b05a};
  case APP_BEASTS: return (PcAppDef){"Beasts", ART_IC_BEAST, 0xf2a705};
  case APP_STICKERS: return (PcAppDef){"Stickers", ART_IC_STICKER, 0xe0457b};
  default: return (PcAppDef){"Home", ART_IC_BRICK, 0x1b2a34};
  }
}

typedef struct Sticker {
  CString name, hint;
  u32 color;
  b32 got;
} Sticker;

#define STICKERS 10
hz_internal Sticker pc_sticker(const GameSave *g, i32 i) {
  switch (i) {
  case 0: return (Sticker){"Card Trainer", "Get a deck from Prof. Quill", 0x3d9bff, g->has_deck};
  case 1: return (Sticker){"Rival Crusher", "Beat Dex in a card battle", 0xc91a09, g->rival_wins > 0};
  case 2: return (Sticker){"Gold Digger", "Find 5 golden studs", 0xf2a705, scr_golden(g) >= 5};
  case 3: return (Sticker){"Duck Friend", "Feed the ducks 3 times", 0x27b05a, g->duck_feeds >= 3};
  case 4: return (Sticker){"Treasure Hunter", "Open a treasure chest", 0x8a5a2b, g->chests >= 1};
  case 5: return (Sticker){"Firefly Catcher", "Catch 5 fireflies", 0x9bd23d, g->fireflies >= 5};
  case 6: return (Sticker){"Seeker", "Find Pip at hide and seek", 0xff8ac0, g->seek_wins >= 1};
  case 7: return (Sticker){"Striker", "Score a goal", 0x1b2a34, g->goals >= 1};
  case 8: return (Sticker){"Collector", "Own 20 different cards", 0x7a4fd1, owned_count(g) >= 20};
  default: return (Sticker){"Voxelmon!", "Raise a card into a Voxelmon", 0x16b5c9, g->voxelmon > 0};
  }
}

hz_internal u32 pc_stickers_done(const GameSave *g) {
  u32 n = 0;
  for (i32 i = 0; i < STICKERS; i++)
    n += pc_sticker(g, i).got ? 1u : 0u;
  return n;
}

// the messages in your inbox (newest first), from what has happened so far
hz_internal u32 pc_messages(const GameSave *g, CString from[6], CString text[6]) {
  u32 n = 0;
#define MSG(f, t) (from[n] = (f), text[n] = (t), n++)
  if (g->voxelmon)
    MSG("Prof. Quill", "A real Voxelmon! Show it to me at the village exit, then Route 1 is yours.");
  if (g->rival_wins)
    MSG("Dex", "That was LUCK. Rematch. Tomorrow. Bring snacks.");
  if (g->has_deck)
    MSG("Prof. Quill", "Your deck suits you. Dex is waiting at his big house - go show him!");
  MSG("Mom", "Breakfast is in the kitchen. Don't forget your jacket!");
  MSG("Dex", "SNAP! snap. SNAP!!!");
  MSG("Prof. Quill", "Come to the lab when you wake up. I have something for you!");
#undef MSG
  return n;
}

// the content of an app, sliding in
hz_internal void pc_content(Screens *sc, const InputSystem *in, const GameSave *g, f32 w, f32 h, f32 fs) {
  HzUIColor ink = cu_rgb(0x1b2a34), dim = cu_rgb(0x5c6670);
  f32 in_t = cu_ease_out(m_clampf(sc->app_t / 0.25f, 0.0f, 1.0f));
  if (sc->app == APP_HOME) {
    // the desktop: a big welcome and the app icons
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.8f), .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}}}) {
      scr_textw("Welcome back, trainer!", fs * 1.25f, cu_rgb(0xffffff), 800, false);
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 1.1f)}}) {
        for (i32 a = 1; a < APP_COUNT; a++) {
          PcAppDef d = pc_app((PcApp)a);
          f32 pop = cu_ease_out(m_clampf((sc->app_t - 0.05f * (f32)a) / 0.25f, 0.0f, 1.0f));
          ui_element({.id = ui_idi("PcIcon", (u32)a),
                      .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.3f), .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                      .opacity = opt_some(f32, pop),
                      .transform = cu_tf(0, (1.0f - pop) * fs, ui_hovered() ? 1.06f : 1.0f, 0)}) {
            if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
              sc->click_app = a;
            f32 s = fs * 3.4f;
            ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(s), .height = ui_sizing_fixed(s)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                        .gradient = cu_lin2(160.0f, cu_mix(cu_rgb(d.color), cu_rgb(0xffffff), 0.25f), cu_rgb(d.color)),
                        .corner_radius = ui_corner_radius(s * 0.26f),
                        .shadow = {.color = cu_black(0.3f), .offset_y = fs * 0.2f, .blur = fs * 0.6f}}) {
              scr_icon(d.icon, s * 0.55f, cu_rgb(0xffffff));
              // the badges: unread messages, stickers earned
              CString badge = NULL;
              if (a == APP_MSG) {
                CString f[6], tx[6];
                cstr_format(sc->text[9], 96, "%", fmt_u32(pc_messages(g, f, tx)));
                badge = sc->text[9];
              } else if (a == APP_STICKERS) {
                cstr_format(sc->text[10], 96, "%/%", fmt_u32(pc_stickers_done(g)), fmt_i32(STICKERS));
                badge = sc->text[10];
              }
              if (badge)
                ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z,
                                         .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_RIGHT_TOP}},
                            .layout = {.padding = {.left = (u16)(fs * 0.35f), .right = (u16)(fs * 0.35f), .top = 1, .bottom = 1}},
                            .background_color = cu_rgb(0xc91a09),
                            .corner_radius = ui_corner_radius(fs),
                            .border = {.color = cu_rgb(0xffffff), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}}}) {
                  scr_textw(badge, fs * 0.7f, cu_rgb(0xffffff), 800, false);
                }
            }
            HzUITextConfig c = {.fontSize = fs * 0.85f, .textColor = cu_rgb(0xffffff), .weight = 700};
            c.shadows[0] = (HzUITextShadow){.color = cu_black(0.5f), .offset_y = 1.0f, .blur = 2.0f};
            ui_text(ui_str_c(d.label), c);
          }
        }
      }
    }
    return;
  }
  // an app: a white sheet
  ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.45f), .padding = ui_padding_all((u16)(fs * 0.8f)),
                         .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
              .background_color = cu_rgba(0xfafaf6, 0.97f),
              .corner_radius = ui_corner_radius(fs * 0.6f),
              .clip = {.vertical = true}, // (a horizontal clip would stop the rows from shrinking and wrapping)
              .opacity = opt_some(f32, in_t),
              .transform = cu_tf(0, (1.0f - in_t) * fs * 1.5f, 1, 0)}) {
    if (sc->app == APP_MSG) {
      CString from[6], text[6];
      u32 n = pc_messages(g, from, text);
      for (u32 i = 0; i < n; i++) {
        f32 pop = cu_ease_out(m_clampf((sc->app_t - 0.07f * (f32)i) / 0.25f, 0.0f, 1.0f));
        b32 narrow = w < 520.0f; // a phone upright: the sender above the text
        ui_element({.layout = {.layout_direction = narrow ? UI_TOP_TO_BOTTOM : UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * (narrow ? 0.1f : 0.5f)),
                               .padding = {.left = (u16)(fs * 0.6f), .right = (u16)(fs * 0.6f), .top = (u16)(fs * 0.35f), .bottom = (u16)(fs * 0.35f)},
                               .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                    .background_color = i == 0 ? cu_rgb(0xe3efff) : cu_rgb(0xeef1f4),
                    .corner_radius = ui_corner_radius(fs * 0.5f),
                    .opacity = opt_some(f32, pop),
                    .transform = cu_tf((1.0f - pop) * fs * 2.0f, 0, 1, 0)}) {
          scr_textw(from[i], fs * 0.85f, cu_rgb(0x1655b0), 800, false);
          ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { scr_textw(text[i], fs * 0.85f, ink, 500, true); }
        }
      }
    } else if (sc->app == APP_STATS) {
      CString labels[8] = {"Battles won", "Battles lost", "Beat Dex", "Golden studs", "Butterflies caught", "Ducks fed", "Treasure chests", "Pip found"};
      u32 vals[8] = {g->wins, g->losses, g->rival_wins, scr_golden(g), g->_reserved[0], g->duck_feeds, g->chests, g->seek_wins};
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.8f), .sizing = {.width = ui_sizing_grow(0)}}}) {
        i32 ncol = w < 520.0f ? 1 : 2, nrow = 8 / ncol; // a phone upright: one column
        for (i32 col = 0; col < ncol; col++)
          ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.35f), .sizing = {.width = ui_sizing_grow(0)}}}) {
            for (i32 r = 0; r < nrow; r++) {
              i32 i = col * nrow + r;
              ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .padding = {.left = (u16)(fs * 0.5f), .right = (u16)(fs * 0.6f), .top = (u16)(fs * 0.3f), .bottom = (u16)(fs * 0.3f)},
                                     .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                          .background_color = cu_rgb(0xeef1f4),
                          .corner_radius = ui_corner_radius(fs * 0.4f)}) {
                scr_textw(labels[i], fs * 0.85f, dim, 600, false);
                ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
                cstr_format(sc->text[i], 96, i == 3 ? "%/15" : i == 2 ? "%\xc3\x97" : "%", fmt_u32(vals[i]));
                scr_textw(sc->text[i], fs * 1.05f, ink, 800, false);
              }
            }
          }
      }
    } else if (sc->app == APP_BEASTS) {
      u32 own = owned_count(g);
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.4f), .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
        scr_textw("Collection", fs * 1.1f, ink, 700, false);
        cstr_format(sc->text[0], 96, "%/%", fmt_u32(own), fmt_i32(CARD_POOL));
        scr_textw(sc->text[0], fs * 1.1f, cu_rgb(0x1655b0), 800, false);
      }
      f32 p = (f32)own / (f32)CARD_POOL * cu_ease_out(m_clampf(sc->app_t / 0.6f, 0.0f, 1.0f));
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(fs * 0.8f)}}, .background_color = cu_rgb(0xe3e7eb),
                  .corner_radius = ui_corner_radius(fs)}) {
        ui_element({.layout = {.sizing = {.width = ui_sizing_percent(m_maxf(p, 0.02f)), .height = ui_sizing_grow(0)}},
                    .gradient = cu_lin2(90.0f, cu_rgb(0x3d9bff), cu_rgb(0x27b05a)), .corner_radius = ui_corner_radius(fs)}) {}
      }
      // the beasts you own, as name chips
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.3f), .sizing = {.width = ui_sizing_grow(0)}},
                  .clip = {.horizontal = true}}) {
        i32 shown = 0;
        for (i32 c = CARD_POOL - 1; c >= 0 && shown < 5; c--)
          if (owns(g, c)) {
            ui_element({.layout = {.padding = {.left = (u16)(fs * 0.45f), .right = (u16)(fs * 0.45f), .top = 2, .bottom = 2}}, .background_color = cu_rgb(0xfff3c4),
                        .corner_radius = ui_corner_radius(fs)}) {
              scr_textw(card_def(c).name, fs * 0.75f, cu_rgb(0x8a5a00), 700, false);
            }
            shown++;
          }
      }
      static const CString facts[5] = {
          "Brick Beasts love studs, sunshine and being collected on cards.",
          "They hide in tall grass and only come out for trainers with a deck.",
          "Six-energy cards that reach their final stage become real Voxelmon.",
          "A Beast's studs are soft at first. Within a week they click into place.",
          "Shiny Voxelmon are rare. Prof. Quill has seen exactly one. Maybe."};
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.5f), .padding = ui_padding_all((u16)(fs * 0.5f)),
                             .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                  .background_color = cu_rgb(0xfff8e1), .corner_radius = ui_corner_radius(fs * 0.5f),
                  .border = {.color = cu_rgb(0xf2d27a), .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}}}) {
        scr_icon(ART_IC_STICKER, fs * 1.2f, cu_rgb(0xf2a705));
        ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { scr_textw(facts[sc->fact % 5], fs * 0.85f, ink, 500, true); }
      }
    } else { // stickers: what you have done, and what is still locked
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(fs * 0.4f), .sizing = {.width = ui_sizing_grow(0)}}}) {
        i32 cols = w < 520.0f ? 3 : 5; // a phone upright: three a row
        for (i32 row = 0; row * cols < STICKERS; row++)
          ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.4f), .sizing = {.width = ui_sizing_grow(0)}}}) {
            for (i32 c = 0; c < cols; c++) {
              i32 i = row * cols + c;
              if (i >= STICKERS) { // keep the last row's stickers as wide as the others
                ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
                continue;
              }
              Sticker st = pc_sticker(g, i);
              f32 pop = cu_ease_out(m_clampf((sc->app_t - 0.04f * (f32)i) / 0.3f, 0.0f, 1.0f));
              ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 2, .padding = ui_padding_all((u16)(fs * 0.35f)),
                                     .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                          .background_color = st.got ? cu_mix(cu_rgb(st.color), cu_rgb(0xffffff), 0.85f) : cu_rgb(0xeef1f4),
                          .corner_radius = ui_corner_radius(fs * 0.5f),
                          .border = {.color = st.got ? cu_rgb(st.color) : cu_rgb(0xd5dae0), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                          .opacity = opt_some(f32, pop),
                          .transform = cu_tf(0, 0, 0.8f + 0.2f * pop, st.got ? ((f32)((i * 37) % 9) - 4.0f) * pop : 0)}) {
                scr_icon(st.got ? ART_IC_STICKER : ART_IC_LOCK, fs * 1.4f, st.got ? cu_rgb(st.color) : cu_rgb(0xa0a8b0));
                scr_textw(st.got ? st.name : "???", fs * 0.65f, st.got ? ink : dim, 800, false);
                ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {
                  ui_text(ui_str_c(st.hint), ui_text_config({.fontSize = fs * 0.55f, .textColor = dim, .wrapMode = UI_TEXT_WRAP_WORDS,
                                                             .textAlignment = CLAY_TEXT_ALIGN_CENTER}));
                }
              }
            }
          }
      }
    }
  }
}

hz_internal void pc_draw(Screens *sc, const InputSystem *in, const GameSave *g, f32 w, f32 h) {
  f32 fs = m_minf(h / 17.0f, w / 22.0f); // a tall panel (phones) gets more rows, not text too wide for it
  // the wallpaper: a sky gradient with a few big soft bricks
  scr_cover(w, h, cu_lin(160.0f, 3, (f32[]){0.0f, 0.55f, 1.0f}, (HzUIColor[]){cu_rgb(0x2f7fe0), cu_rgb(0x5b9cf0), cu_rgb(0x9b7be8)}), NULL, (HzUIColor){0});
  ScrCanvas cv = scr_canvas(w, h);
  for (i32 i = 0; i < 5; i++) {
    f32 x = (f32)((i * 71 + 20) % 300), y = (f32)((i * 43 + 30) % 150), s = 28.0f + (f32)((i * 13) % 20);
    f32 drift = 6.0f * m_sinf(sc->time * 0.4f + (f32)i);
    scr_box(cv, x, y + drift, s * 1.8f, s, s * 0.18f, cu_rgba(0xffffff, 0.07f), (f32)(i * 17 % 30) - 15.0f);
  }
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = SCR_Z},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}}}) {
    // the bar: BrickOS, back, studs, power
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.4f), .padding = {.left = (u16)(fs * 0.5f), .right = (u16)(fs * 0.35f), .top = (u16)(fs * 0.25f), .bottom = (u16)(fs * 0.25f)},
                           .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_rgba(0x0b1424, 0.55f),
                .backdrop_blur = 6.0f}) {
      scr_icon(ART_IC_BRICK, fs * 1.1f, cu_rgb(0xffd23d));
      scr_textw("BrickOS", fs * 0.9f, cu_rgb(0xffffff), 800, false);
      if (sc->app != APP_HOME) {
        ui_element({.id = ui_id("PcBack"),
                    .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(fs * 0.2f), .padding = {.left = (u16)(fs * 0.3f), .right = (u16)(fs * 0.5f), .top = 2, .bottom = 2},
                               .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                    .background_color = ui_hovered() ? cu_rgba(0xffffff, 0.28f) : cu_rgba(0xffffff, 0.16f),
                    .corner_radius = ui_corner_radius(fs)}) {
          if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
            sc->click_app = APP_HOME;
          scr_icon(ART_IC_BACK, fs * 0.9f, cu_rgb(0xffffff));
          scr_textw(pc_app(sc->app).label, fs * 0.8f, cu_rgb(0xffffff), 700, false);
        }
      }
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
      scr_icon(ART_IC_COIN, fs * 0.95f, cu_rgb(0xffd23d));
      cstr_format(sc->text[11], 96, "%", fmt_u32(g->studs));
      scr_textw(sc->text[11], fs * 0.85f, cu_rgb(0xffffff), 700, false);
      ui_element({.id = ui_id("PcOff"),
                  .layout = {.padding = {.left = (u16)(fs * 0.25f), .right = (u16)(fs * 0.25f), .top = (u16)(fs * 0.15f), .bottom = (u16)(fs * 0.15f)}},
                  .background_color = cu_rgb(0xc91a09),
                  .corner_radius = ui_corner_radius(fs * 0.35f),
                  .brightness = ui_hovered() ? opt_some(f32, 1.2f) : opt_none(f32)}) {
        if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
          sc->click_close = true;
        scr_icon(ART_IC_POWER, fs * 0.95f, cu_rgb(0xffffff));
      }
    }
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .padding = ui_padding_all((u16)(fs * 0.6f)),
                           .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
      pc_content(sc, in, g, w, h, fs);
    }
  }
}

// ---------------------------------------------------------------- the frame

// where the screen is on the UI now (the board projected; the camera flies there), else a box in the middle
hz_internal b32 screens_rect(Screens *sc, const Camera *cam, f32 sw, f32 sh, f32 r[4]) {
  if (!sc->has_board || !cam)
    return false;
  SignBoard b = sc->board;
  f32 ct = m_cosf(b.tilt), st = m_sinf(b.tilt);
  v3 n = {m_sinf(b.yaw) * ct, st, m_cosf(b.yaw) * ct};
  v3 right = {m_cosf(b.yaw), 0.0f, -m_sinf(b.yaw)};
  v3 up = v3_cross(n, right);
  v3 c = town_m(b.c.x, b.c.y, b.c.z);
  f32 x0 = 1e9f, y0 = 1e9f, x1 = -1e9f, y1 = -1e9f;
  for (i32 i = 0; i < 4; i++) {
    f32 sx = (i & 1) ? 0.5f : -0.5f, sy = (i & 2) ? 0.5f : -0.5f;
    v3 p = v3_add(c, v3_add(v3_scale(right, sx * b.w * TOWN_STUD), v3_scale(up, sy * b.h * TOWN_STUD)));
    f32 s[2];
    camera_project_to_screen(cam->view_proj, p, sw, sh, s);
    x0 = m_minf(x0, s[0]);
    x1 = m_maxf(x1, s[0]);
    y0 = m_minf(y0, s[1]);
    y1 = m_maxf(y1, s[1]);
  }
  if (x1 - x0 < 40.0f || y1 - y0 < 30.0f || x1 - x0 > sw * 3.0f)
    return false;
  r[0] = x0, r[1] = y0, r[2] = x1 - x0, r[3] = y1 - y0;
  return true;
}

// the UI frame. `zoom_t`: how far the camera has flown to the screen (0..1)
hz_internal void screens_draw(Screens *sc, const InputSystem *in, const GameSave *save, const Camera *cam, f32 sw, f32 sh, f32 zoom_t,
                              b32 talking) {
  if (sc->kind == SCREEN_NONE)
    return;
  HzUIColor white = {255, 255, 255, 255};
  if (sc->kind == SCREEN_PIANO) {
    ui_element({.id = ui_id("ScreenDim"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_black(0.55f)}) {
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .padding = ui_padding_all(20),
                             .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .gradient = cu_lin2(180.0f, cu_rgb(0x2b2622), cu_rgb(0x15120f)),
                  .corner_radius = ui_corner_radius(16),
                  .shadow = {.color = cu_black(0.5f), .offset_y = 10.0f, .blur = 30.0f}}) {
        scr_textw("Lumi's piano", 26, white, 700, false);
        // black keys row, then white keys row (a simple two-row keyboard)
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .padding = {.left = 30}}}) {
          i32 gaps[PIANO_KEYS - PIANO_WHITE] = {0, 0, 1, 0, 0, 1, 0}; // a gap after E and B
          for (i32 b = 0; b < PIANO_KEYS - PIANO_WHITE; b++) {
            i32 k = PIANO_WHITE + b;
            ui_element({.id = ui_idi("PianoBlack", (u32)b),
                        .layout = {.sizing = {.width = ui_sizing_fixed(46), .height = ui_sizing_fixed(110)}},
                        .gradient = sc->key_lit[k] > 0.0f ? cu_lin2(180.0f, cu_rgb(0x3d9bff), cu_rgb(0x0055bf)) : cu_lin2(180.0f, cu_rgb(0x3a3a40), cu_rgb(0x0c0c0e)),
                        .corner_radius = ui_corner_radius(5),
                        .shadow = {.color = cu_black(0.5f), .offset_y = 3.0f, .blur = 4.0f}}) {
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
                        .gradient = sc->key_lit[k] > 0.0f ? cu_lin2(180.0f, cu_rgb(0xd6ebff), cu_rgb(0x96c8ff)) : cu_lin2(180.0f, cu_rgb(0xffffff), cu_rgb(0xe6e4dc)),
                        .corner_radius = ui_corner_radius(6),
                        .shadow = {.color = cu_black(0.4f), .offset_y = 3.0f, .blur = 3.0f}}) {
              if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
                sc->click_key = k;
              scr_text(labels[k], 14, (HzUIColor){120, 128, 136, 255}, false);
            }
          }
        }
        scr_text("Keys A..L (white) and W, T, Y, U, O, P (black), or tap the keys. E / Esc to stop.", 14, (HzUIColor){190, 196, 204, 255}, false);
      }
    }
    return;
  }
  f32 r[4];
  b32 on_board = screens_rect(sc, cam, sw, sh, r);
  if (!on_board) { // no camera flight (or not visible yet): a 16:10 screen in the middle
    f32 w = m_minf(sw * 0.9f, 760.0f), h = w * (sc->kind == SCREEN_TV ? 0.55f : 0.62f);
    r[0] = (sw - w) * 0.5f, r[1] = (sh - h) * 0.45f, r[2] = w, r[3] = h;
  }
  // it switches on once the camera is nearly there: a line opening up from the middle, bright at first
  f32 shown = sc->has_board ? m_clampf((zoom_t - 0.75f) / 0.25f, 0.0f, 1.0f) : 1.0f;
  f32 since = sc->time - (sc->has_board ? 0.45f : 0.0f);
  f32 boot = m_clampf(since / 0.28f, 0.0f, 1.0f);
  if (shown <= 0.0f || since < 0.0f)
    return;
  // a phone held upright: the real screen is too small to read, so the picture grows out of it to a panel that fits
  // the phone (the TV keeps its shape; the computers get taller)
  if (sh > sw * 1.15f) {
    OsSafeAreaInsets safe = os_get_safe_area();
    f32 pw = sw - 16.0f - safe.left - safe.right;
    f32 ph = sc->kind == SCREEN_TV ? pw * 0.62f : m_minf(pw * 1.25f, (sh - safe.top - safe.bottom) * 0.72f);
    f32 to[4] = {(sw - pw) * 0.5f, safe.top + (sh - safe.top - safe.bottom - ph) * 0.42f, pw, ph};
    f32 e = cu_ease_out(boot);
    for (i32 i = 0; i < 4; i++)
      r[i] = m_lerpf(r[i], to[i], e);
  }
  f32 w = r[2], h = r[3];
  HzUIColor bg = sc->kind == SCREEN_LAB ? cu_rgb(0x03130c) : sc->kind == SCREEN_PC ? cu_rgb(0x2f7fe0) : cu_rgb(0x05070a);
  ui_element({.id = ui_id("Screen"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = SCR_Z, .offset = {r[0], r[1]}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
              .background_color = bg,
              .corner_radius = ui_corner_radius(h * 0.02f),
              .clip_path = {.round = ui_corner_radius(h * 0.02f)},
              .opacity = opt_some(f32, shown),
              .brightness = opt_some(f32, 1.0f + 2.5f * (1.0f - boot)),
              .transform = scr_tf(0, 0, 1.0f, 0.02f + 0.98f * cu_ease_out(boot), 0, 0.5f, 0.5f)}) {
    if (sc->kind == SCREEN_TV)
      tv_draw(sc, w, h, save, talking);
    else if (sc->kind == SCREEN_LAB) {
      lab_draw(sc, in, w, h);
      scr_scanlines(w, h, 0.25f);
      scr_vignette(w, h, 0.55f, 0.65f);
    } else {
      pc_draw(sc, in, save, w, h);
      scr_scanlines(w, h, 0.06f);
    }
  }
  // the TV's remote, under the screen
  if (sc->kind == SCREEN_TV) {
    f32 fs = m_clampf(h / 18.0f, 12.0f, 17.0f);
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = SCR_Z,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_LEFT_TOP},
                             .offset = {r[0] + w * 0.5f, m_minf(r[1] + h + fs * 0.9f, sh - fs * 4.0f)}},
                .opacity = opt_some(f32, shown * boot)}) {
      tv_osd(sc, in, fs);
    }
  }
}
