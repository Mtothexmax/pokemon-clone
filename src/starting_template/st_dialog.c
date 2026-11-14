// Dialogues: a speaker box at the bottom, lines advanced with E / Space / Enter, a click or the touch button, up to
// two choices at the end. Every line the three.js version had voiced plays its recorded voice (Fish Audio, the ids of
// scripts/voice-lines.mjs: assets/brickhollow/voices/<id>.mp3). Included by starting_template.c above app_init.

#define DIALOG_MAX_LINES 10
#define VOICE(id) "brickhollow/voices/" id ".hzt"

typedef enum DialogAction {
  ACT_NONE,
  ACT_HIDE_SEEK, // Pip: play hide and seek
  ACT_TAG,       // Pip: play tag
  ACT_SLEEP,     // your bed
} DialogAction;

typedef struct DialogLine {
  CString text;
  CString voice; // NULL: not voiced
} DialogLine;

typedef struct Dialog {
  CString speaker;
  DialogLine lines[DIALOG_MAX_LINES];
  u32 count;
  CString choice[2];
  DialogAction action[2];
} Dialog;

typedef enum DialogId {
  DLG_INTRO_KEYS,
  DLG_INTRO_TOUCH,
  DLG_MOM,
  DLG_PROF,
  DLG_DEX,
  DLG_LUMI,
  DLG_PIP,
  DLG_GATE,
  DLG_COUNT
} DialogId;

#define L(text, voice) {text, voice}

// the lines as the three.js version had them before the first cards (the card rounds extend this)
hz_internal Dialog dialog_script(DialogId id) {
  switch (id) {
  case DLG_INTRO_KEYS:
    return (Dialog){.speaker = "You", .lines = {L("*yawn* What a beautiful morning in Brickhollow.", VOICE("91b19939")),
                            L("I should go downstairs and talk to Mom in the kitchen. WASD / arrow keys to walk, E or Space to talk.",
                              VOICE("4f858f02"))}, .count = 2};
  case DLG_INTRO_TOUCH:
    return (Dialog){.speaker = "You", .lines = {L("*yawn* What a beautiful morning in Brickhollow.", VOICE("91b19939")),
                            L("I should go downstairs and talk to Mom in the kitchen. Drag the stick to walk and tap the talk button.",
                              VOICE("459836e6"))}, .count = 2};
  case DLG_MOM:
    return (Dialog){.speaker = "Mom", .lines = {L("Morning, sweetie! Professor Quill was looking for you earlier.", VOICE("10a293f7")),
                            L("Her lab is the big tan building with the solar panels \xe2\x80\x94 just across the road to the south-east.",
                              VOICE("85d4a2ce"))}, .count = 2};
  case DLG_PROF:
    return (Dialog){.speaker = "Prof. Hazel Quill", .lines = {L("Ah, there you are! Welcome to Quill Lab.", VOICE("df9ece56")),
                     L("All around Brickhollow live BRICK BEASTS \xe2\x80\x94 little creatures snapped together from living bricks!",
                       VOICE("97e8d994")),
                     L("Trainers here battle with decks of 12 Beast cards, fighting over 3 zones for 6 turns.", VOICE("faa6ffcd")),
                     L("Did you know? Glimmerlings fold light into tiny bricks. Fascinating creatures.", VOICE("c83f36f4"))}, .count = 4};
  case DLG_DEX:
    return (Dialog){.speaker = "Dex", .lines = {L("Hey, slowpoke! You don't even have a deck yet.", VOICE("14add442")),
                            L("Go see Grandma Quill at the lab. Then I'll crush you!", VOICE("88739c53"))}, .count = 2};
  case DLG_LUMI:
    return (Dialog){.speaker = "Lumi", .lines = {L("Hi! I'm Lumi, Dex's sister. Want a quick lesson?", VOICE("dbc3cccf")),
                     L("Each turn you get energy equal to the turn number \xe2\x80\x94 1 on turn 1, up to 6 on turn 6.", VOICE("e99643e2")),
                     L("Play cards into the 3 zones. Zones reveal one by one on turns 1 to 3 and each has its own effect.",
                       VOICE("1e658f85")),
                     L("Both players plan in secret, then cards reveal. Whoever is ahead reveals first.", VOICE("e7930b4e")),
                     L("Cards come in ability types: On Reveal fires when the card flips, Ongoing works all the time.",
                       VOICE("7abe2838")),
                     L("Activate cards can be used once \xe2\x80\x94 tap them on the board. End of Turn cards trigger after every turn, "
                       "and Game Start cards do their thing before the first draw.",
                       VOICE("064e0ba2")),
                     L("Tap any card to see it big, and tap a location to read what it does!", VOICE("fb9d3419")),
                     L("After turn 6, win 2 of the 3 zones to win! Hit SNAP to double the stakes if you feel confident.",
                       VOICE("251412fd"))}, .count = 8, .choice = {"Got it!"}, .action = {ACT_NONE}};
  case DLG_PIP:
    return (Dialog){.speaker = "Pip", .lines = {L("Psst! I saw something wiggle in the tall grass up north!", VOICE("afa6d9b1")),
                     L("Also\xe2\x80\xa6 there are 15 golden studs hidden around Brickhollow. Some are even inside houses! Find them all and "
                       "something legendary happens.",
                       VOICE("f19b68ac")),
                     L("My big brother says there are 34 different Brick Beasts. I only know the fishy ones.", VOICE("153eb44f"))}, .count = 3, .choice = {"Play hide and seek", "Play tag"}, .action = {ACT_HIDE_SEEK, ACT_TAG}};
  default:
    return (Dialog){.speaker = "Prof. Hazel Quill", .lines = {L("Wait! Don't go out there!", VOICE("c77df687")),
                                          L("Wild Brick Beasts live in the tall grass. You can't go without any Beast cards!",
                                            VOICE("7cf5da3f")),
                                          L("Come to my lab first. I'll give you your very first cards.", VOICE("22139e70"))}, .count = 3};
  }
}

typedef struct DialogState {
  b32 open;
  Dialog d;
  u32 index;
  i32 choice_hover; // keyboard selection of a choice
  DialogAction chosen; // set when the dialog closed with a choice (the caller acts on it, then clears)
  HzHandleT(AudioVoice) voice;
  b32 clicked; // the UI saw a click on the box (read next frame)
  i32 clicked_choice;
} DialogState;

hz_internal void dialog_voice(DialogState *ds, EcsWorld *w, AudioSystem *audio) {
  audio_voice_stop(audio, ds->voice, 0.08f);
  CString v = ds->d.lines[ds->index].voice;
  if (v)
    ds->voice = ecs_audio_play(w, v, &(EcsAudioPlayDesc){.volume = 1.0f, .bus = AUDIO_BUS_VOICE});
}

hz_internal void dialog_open(DialogState *ds, Dialog d, EcsWorld *w, AudioSystem *audio) {
  ds->open = true;
  ds->d = d;
  ds->index = 0;
  ds->choice_hover = 0;
  ds->clicked = false;
  ds->clicked_choice = -1;
  dialog_voice(ds, w, audio);
}

hz_internal void dialog_close(DialogState *ds, AudioSystem *audio) {
  ds->open = false;
  audio_voice_stop(audio, ds->voice, 0.15f);
}

hz_internal b32 dialog_has_choices(const DialogState *ds) { return ds->d.choice[0] != NULL; }

// main lane, before the scope: next / choose / close. `advance` = E, Space, Enter or the touch button
hz_internal void dialog_update(DialogState *ds, EcsWorld *w, AudioSystem *audio, const InputSystem *in, b32 advance) {
  if (!ds->open)
    return;
  b32 last = ds->index + 1 >= ds->d.count;
  i32 nchoice = ds->d.choice[1] ? 2 : ds->d.choice[0] ? 1 : 0;
  if (last && nchoice) {
    if (in->buttons[KEY_LEFT].pressed_this_frame || in->buttons[KEY_A].pressed_this_frame || in->buttons[KEY_UP].pressed_this_frame)
      ds->choice_hover = 0;
    if (in->buttons[KEY_RIGHT].pressed_this_frame || in->buttons[KEY_D].pressed_this_frame || in->buttons[KEY_DOWN].pressed_this_frame)
      ds->choice_hover = nchoice - 1;
    i32 pick = ds->clicked_choice >= 0 ? ds->clicked_choice : advance ? ds->choice_hover : -1;
    ds->clicked_choice = -1;
    ds->clicked = false;
    if (pick >= 0) {
      ds->chosen = ds->d.action[pick];
      dialog_close(ds, audio);
    }
    if (in->buttons[KEY_ESCAPE].pressed_this_frame)
      dialog_close(ds, audio);
    return;
  }
  if (in->buttons[KEY_ESCAPE].pressed_this_frame) {
    dialog_close(ds, audio);
    return;
  }
  if (advance || ds->clicked) {
    ds->clicked = false;
    if (last)
      dialog_close(ds, audio);
    else {
      ds->index++;
      dialog_voice(ds, w, audio);
    }
  }
}

// the UI frame: the speaker box and the choices
hz_internal void dialog_draw(DialogState *ds, const InputSystem *in, b32 touch, f32 screen_w) {
  if (!ds->open)
    return;
  OsSafeAreaInsets safe = os_get_safe_area();
  b32 last = ds->index + 1 >= ds->d.count;
  b32 click = in->buttons[MOUSE_LEFT].pressed_this_frame;
  HzUIColor ink = {27, 42, 52, 255};
  ui_element({.id = ui_id("DialogBox"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                           .offset = {0.0f, -safe.bottom - (touch ? 140.0f : 22.0f)}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8,
                         .sizing = {.width = ui_sizing_fixed(m_minf(760.0f, screen_w - 32.0f))},
                         .padding = {.left = 20, .right = 20, .top = 14, .bottom = 14}},
              .background_color = {250, 250, 246, 245},
              .border = {.color = ink, .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
              .corner_radius = ui_corner_radius(12)}) {
    if (ui_hovered() && click && !(last && dialog_has_choices(ds)))
      ds->clicked = true;
    ui_element({.layout = {.padding = {.left = 10, .right = 10, .top = 3, .bottom = 3}},
                .background_color = {242, 205, 55, 255},
                .corner_radius = ui_corner_radius(6)}) {
      ui_text(ui_str_c(ds->d.speaker), ui_text_config({.fontSize = 16, .textColor = ink, .wrapMode = UI_TEXT_WRAP_NONE}));
    }
    ui_text(ui_str_c(ds->d.lines[ds->index].text),
            ui_text_config({.fontSize = 20, .textColor = ink, .wrapMode = UI_TEXT_WRAP_WORDS}));
    if (last && dialog_has_choices(ds)) {
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 12}}) {
        for (i32 i = 0; i < 2; i++) {
          if (!ds->d.choice[i])
            continue;
          b32 sel = ds->choice_hover == i;
          ui_element({.id = ui_idi("DialogChoice", (u32)i),
                      .layout = {.padding = {.left = 16, .right = 16, .top = 8, .bottom = 8}},
                      .background_color = sel ? (HzUIColor){0, 85, 191, 255} : (HzUIColor){224, 230, 236, 255},
                      .corner_radius = ui_corner_radius(8)}) {
            if (ui_hovered()) {
              ds->choice_hover = i;
              if (click)
                ds->clicked_choice = i;
            }
            ui_text(ui_str_c(ds->d.choice[i]),
                    ui_text_config({.fontSize = 18, .textColor = sel ? (HzUIColor){255, 255, 255, 255} : ink,
                                    .wrapMode = UI_TEXT_WRAP_NONE}));
          }
        }
      }
    } else {
      ui_text(ui_str_c(touch ? "tap to continue" : "E / Space / click to continue"),
              ui_text_config({.fontSize = 13, .textColor = {120, 128, 136, 255}, .wrapMode = UI_TEXT_WRAP_NONE}));
    }
  }
}
