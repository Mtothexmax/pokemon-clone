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
  ACT_GIVE_STARTER,    // Prof. Quill: the starter cards (then the rewards, then DLG_PROF_AFTER)
  ACT_BATTLE_RIVAL,    // Dex
  ACT_BATTLE_PRACTICE, // Lumi
  ACT_DEX_SCARED,      // Dex: "Not now"
  ACT_OPEN_GATE,       // Prof. Quill at the gap: you have a Voxelmon
  ACT_QUEST_PART,      // Prof. Quill: the Deck-o-Matic misses its Stud Core (find the suit, dive for it)
  ACT_BATTLE_PIP,      // Pip: a card battle (only after a round of hide and seek)
} DialogAction;

typedef struct DialogLine {
  CString text;
  CString voice; // NULL: not voiced
} DialogLine;

typedef struct Dialog {
  CString speaker;
  DialogLine lines[DIALOG_MAX_LINES];
  u32 count;
  CString choice[3];
  DialogAction action[3];
  DialogAction on_end; // when the dialog closes without a choice
} Dialog;

typedef enum DialogId {
  DLG_INTRO_KEYS,
  DLG_INTRO_TOUCH,
  DLG_INTRO_SWIPE,
  DLG_MOM,
  DLG_PROF,
  DLG_DEX,
  DLG_LUMI,
  DLG_PIP,
  DLG_GATE,
  DLG_PROF_AFTER, // after the starter cards
  DLG_DEX_SCARED,
  DLG_DEX_WON, // after a battle: you won
  DLG_DEX_LOST,
  DLG_LUMI_WON,
  DLG_LUMI_LOST,
  DLG_MACHINE, // the Deck-o-Matic in the lab
  DLG_KNIGHT,  // Stonecrest: Sir Bricksworth at the castle
  DLG_BAKER,   // Stonecrest: Marla by the bakery
  DLG_PIP_NOT_YET, // Pip: a card battle? hide and seek first
  DLG_PIP_WON,     // after a battle against Pip: you won
  DLG_PIP_LOST,
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
  case DLG_INTRO_SWIPE:
    return (Dialog){.speaker = "You", .lines = {L("*yawn* What a beautiful morning in Brickhollow.", VOICE("91b19939")),
                            L("I should go downstairs and talk to Mom in the kitchen. Swipe to walk, tap to stop, and tap the talk button.",
                              VOICE("4c5ccef1"))}, .count = 2};
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
  case DLG_KNIGHT: // (three.js +page.svelte: never voiced there either)
    return (Dialog){.speaker = "Sir Bricksworth",
                    .lines = {L("Welcome to Stonecrest, traveller! I guard the old castle.", NULL),
                              L("It was built from the very first bricks of the valley. Some say Brick Beasts still live in its towers.", NULL),
                              L("The gate is closed for now. Come back when you are a famous trainer!", NULL)},
                    .count = 3};
  case DLG_BAKER:
    return (Dialog){.speaker = "Marla",
                    .lines = {L("Oh, a visitor from Brickhollow! You came all the way through the forest?", NULL),
                              L("Have a look around. Our fountain is the prettiest in the whole valley.", NULL)},
                    .count = 2};
  case DLG_PIP:
    return (Dialog){.speaker = "Pip", .lines = {L("Psst! I saw something wiggle in the tall grass up north!", VOICE("afa6d9b1")),
                     L("Also\xe2\x80\xa6 there are 15 golden studs hidden around Brickhollow. Some are even inside houses! Find them all and "
                       "something legendary happens.",
                       VOICE("f19b68ac")),
                     L("My big brother says there are 34 different Brick Beasts. I only know the fishy ones.", VOICE("153eb44f")),
                     L("Hey, I'm bored! Wanna play hide and seek? Or tag? I'm super fast!", NULL)}, .count = 4, .choice = {"Play hide and seek", "Play tag"}, .action = {ACT_HIDE_SEEK, ACT_TAG}};
  default:
    return (Dialog){.speaker = "Prof. Hazel Quill", .lines = {L("Wait! Don't go out there!", VOICE("c77df687")),
                                          L("Wild Brick Beasts live in the tall grass. You can't go without any Beast cards!",
                                            VOICE("7cf5da3f")),
                                          L("Come to my lab first. I'll give you your very first cards.", VOICE("22139e70"))}, .count = 3};
  }
}

// "You have N of 34 Brick Beast cards." as recorded for every N
hz_internal CString prof_count_voice(u32 n) {
  static const CString ids[35] = {VOICE("8b373374"), VOICE("925f7191"), VOICE("cacfe5d2"), VOICE("adec45f7"), VOICE("cfe9e390"), VOICE("3266cefd"), VOICE("fbb93e1e"), VOICE("7e9955e3"), VOICE("2df970fc"), VOICE("6ed80a59"), VOICE("dc71ff0d"), VOICE("9c57c1a0"), VOICE("ce9262b3"), VOICE("2c64cb2e"), VOICE("e2242121"), VOICE("aff263c4"), VOICE("a3629d07"), VOICE("4c501aa2"), VOICE("3bc9d495"), VOICE("6088ca08"), VOICE("c512a37c"), VOICE("05f13cd9"), VOICE("85280dda"), VOICE("6abac11f"), VOICE("ba89a838"), VOICE("8c9677c5"), VOICE("d7a87066"), VOICE("85ce9c2b"), VOICE("225065f4"), VOICE("2978a411"), VOICE("0f30fe4f"), VOICE("09b26b0a"), VOICE("6afa0609"), VOICE("b6afd02c"), VOICE("522b0d4b")};
  return ids[n < 35 ? n : 34];
}

// the lines as they depend on your progress (the three.js talk()): deck, wins, Voxelmon
hz_internal Dialog dialog_for(DialogId id, const GameSave *s, b32 has_voxelmon, char *buf, u32 cap) {
  Dialog d = dialog_script(id);
  switch (id) {
  case DLG_MOM:
    if (s->has_deck)
      d = (Dialog){.speaker = "Mom",
                   .lines = {s->wins ? (DialogLine)L("So many wins already? I'm so proud of you!", VOICE("2dcf3ee1"))
                                     : (DialogLine)L("Off on an adventure? Be careful in that tall grass!", VOICE("e212973d")),
                             L("Remember: you can rebuild your deck any time with the Deck button (C).", VOICE("33a0ab32")),
                             L("And snap only when you feel lucky. Your father lost a whole bucket of studs that way!", VOICE("79f91424"))},
                   .count = 3};
    break;
  case DLG_PROF:
    // the first deck: the Deck-o-Matic prints it, but it misses its Stud Core - lost in the pond cave
    if (!s->has_deck && !s->has_part) {
      if (!s->quest_part)
        d = (Dialog){.speaker = "Prof. Hazel Quill",
                     .lines = {L("Ah, there you are! Welcome to Quill Lab.", VOICE("df9ece56")),
                               L("All around Brickhollow live BRICK BEASTS \xe2\x80\x94 little creatures snapped together from living bricks!", VOICE("97e8d994")),
                               L("Trainers here battle with decks of 12 Beast cards, fighting over 3 zones for 6 turns.", VOICE("faa6ffcd")),
                               L("I wanted to print your very first deck with my Deck-o-Matic over there... but it only blinks MISSING PART!", NULL),
                               L("Its Stud Core is gone. I dropped it while diving in the pond. Yes, the pond: it is the way into a deep, flooded cave.", NULL),
                               L("My old diving suit must be somewhere in this lab. Find it, dive down and bring me the Stud Core!", NULL)},
                     .count = 6, .on_end = ACT_QUEST_PART};
      else if (!s->has_suit)
        d = (Dialog){.speaker = "Prof. Hazel Quill",
                     .lines = {L("Found my diving suit yet? It is in here somewhere... in a corner nobody ever looks into, I suppose.", NULL)},
                     .count = 1};
      else
        d = (Dialog){.speaker = "Prof. Hazel Quill",
                     .lines = {L("The suit fits you! Go to the pond and dive in.", NULL),
                               L("Deep down the Stud Core glows yellow. Follow the light, and mind the dark parts.", NULL)},
                     .count = 2};
    } else if (!s->has_deck) {
      d = (Dialog){.speaker = "Prof. Hazel Quill",
                   .lines = {L("The Stud Core! You really found it down there!", NULL),
                             L("Let me put it in... *clank* *whirr* ... the Deck-o-Matic is printing!", NULL),
                             L("Here are 18 cards to start your collection. I've already built you a deck of 12!", VOICE("83c29583")),
                             L("Wild Beasts hide in the tall grass on Route 1, north of the village. Beat them to collect new cards!", VOICE("a633cd6e"))},
                   .count = 4, .on_end = ACT_GIVE_STARTER};
    } else {
      u32 n = 0;
      for (i32 c = 0; c < CARD_POOL; c++)
        n += (u32)((s->collection >> c) & 1ull);
      cstr_format(buf, cap, "You have % of % Brick Beast cards. %", fmt_u32(n), fmt_i32(CARD_POOL),
                  fmt_cstr(n >= CARD_POOL ? "A complete collection \xe2\x80\x94 incredible!" : "Keep exploring Route 1!"));
      d = (Dialog){.speaker = "Prof. Hazel Quill",
                   .lines = {{buf, prof_count_voice(n)},
                             L("Did you know? Glimmerlings fold light into tiny bricks. Fascinating creatures.", VOICE("c83f36f4"))},
                   .count = 2};
    }
    break;
  case DLG_MACHINE:
    if (!s->has_deck && !s->has_part)
      d = (Dialog){.speaker = "Deck-o-Matic",
                   .lines = {L("*beep* ERROR: MISSING PART. Stud Core not found.", NULL),
                             L("Insert a Stud Core to print Beast decks. *beep*", NULL)},
                   .count = 2};
    else if (!s->has_deck)
      d = (Dialog){.speaker = "Deck-o-Matic", .lines = {L("*beep* Stud Core detected. Waiting for Prof. Quill.", NULL)}, .count = 1};
    else
      d = (Dialog){.speaker = "Deck-o-Matic", .lines = {L("*whirr* READY. Decks printed today: 1. Have fun!", NULL)}, .count = 1};
    break;
  case DLG_PROF_AFTER:
    d = (Dialog){.speaker = "Prof. Hazel Quill",
                 .lines = {L("Lumi, Dex's sister, knows the rules by heart \xe2\x80\x94 visit their big house in the east for a practice match.", VOICE("2a919bfb")),
                           L("And watch out for my grandson Dex. He has been itching for a battle all week!", VOICE("5facc525"))},
                 .count = 2};
    break;
  case DLG_DEX:
    if (s->has_deck && s->rival_wins)
      d = (Dialog){.speaker = "Dex",
                   .lines = {L("Back for more? This time I upgraded my deck!", VOICE("755b07b2")), L("Ready?", VOICE("bd46fed4"))},
                   .count = 2, .choice = {"Battle!", "Not now"}, .action = {ACT_BATTLE_RIVAL, ACT_DEX_SCARED}};
    else if (s->has_deck)
      d = (Dialog){.speaker = "Dex",
                   .lines = {L("So Grandma gave you a deck, huh? Mine is WAY cooler.", VOICE("676ff784")),
                             L("Let's battle \xe2\x80\x94 right here, right now!", VOICE("4493640a"))},
                   .count = 2, .choice = {"Battle!", "Not now"}, .action = {ACT_BATTLE_RIVAL, ACT_DEX_SCARED}};
    break;
  case DLG_DEX_SCARED:
    d = (Dialog){.speaker = "Dex", .lines = {L("Scared? Ha! Come back when you are ready.", VOICE("e4e22ae0"))}, .count = 1};
    break;
  case DLG_DEX_WON:
    d = (Dialog){.speaker = "Dex",
                 .lines = {L("WHAT?! No way... my deck was perfect!", VOICE("1d5e5e86")),
                           L("Fine. You win this time. I'm going to train on Route 1!", VOICE("e454c49a"))},
                 .count = 2};
    break;
  case DLG_DEX_LOST:
    d = (Dialog){.speaker = "Dex",
                 .lines = {L("Ha! Told you my deck was cooler.", VOICE("8c7da71a")), L("Come back when you have some better cards!", VOICE("d12633cc"))},
                 .count = 2};
    break;
  case DLG_PIP:
    if (s->has_deck) { // a card battle too (st_cardgame.c)
      d.lines[3] = (DialogLine)L("Hey, I'm bored! Hide and seek? Tag? Or... a card battle? My deck is secretly the best!", NULL);
      d.choice[0] = "Hide and seek";
      d.choice[1] = "Tag";
      d.choice[2] = "Card battle!";
      d.action[0] = ACT_HIDE_SEEK;
      d.action[1] = ACT_TAG;
      d.action[2] = ACT_BATTLE_PIP;
    }
    break;
  case DLG_PIP_NOT_YET:
    d = (Dialog){.speaker = "Pip",
                 .lines = {L("A card battle? Nooo, that's boring. Hide and seek first!", NULL),
                           L("Play one round with me, then I'll show you my deck. It's even better than my brother's!", NULL)},
                 .count = 2, .choice = {"Hide and seek", "Later"}, .action = {ACT_HIDE_SEEK, ACT_NONE}};
    break;
  case DLG_PIP_WON:
    d = (Dialog){.speaker = "Pip",
                 .lines = {L("Whaaat? You beat my super deck?!", NULL), L("Okay, okay... you're good. But I'm still faster than you!", NULL)},
                 .count = 2};
    break;
  case DLG_PIP_LOST:
    d = (Dialog){.speaker = "Pip",
                 .lines = {L("Hehe! Told you, my deck is the best!", NULL), L("Come back when you've got stronger cards. I'll be by the pond!", NULL)},
                 .count = 2};
    break;
  case DLG_LUMI_WON:
    d = (Dialog){.speaker = "Lumi", .lines = {L("Wow, you learn fast! Here, keep that card.", VOICE("f8de2d54"))}, .count = 1};
    break;
  case DLG_LUMI_LOST:
    d = (Dialog){.speaker = "Lumi", .lines = {L("Good try! Watch which zones you can still win and focus there.", VOICE("2a3279f7"))}, .count = 1};
    break;
  case DLG_LUMI:
    if (s->has_deck) {
      d.choice[0] = "Practice battle";
      d.choice[1] = "Thanks!";
      d.action[0] = ACT_BATTLE_PRACTICE;
      d.action[1] = ACT_NONE;
    }
    break;
  case DLG_GATE:
    if (s->has_deck && !has_voxelmon)
      d = (Dialog){.speaker = "Prof. Hazel Quill",
                   .lines = {L("Hold on! Cards alone won't protect you out there.", VOICE("eeac3b70")),
                             L("The wild Beasts on Route 1 are real Voxelmon, made of living bricks!", VOICE("5376fc4a")),
                             L("Win card battles to earn EXP, and upgrade your cards in the Deck menu.", VOICE("d85cccae")),
                             L("Once a six energy card reaches its final stage, it becomes a real Voxelmon. Then you may pass!", VOICE("13e542b8"))},
                   .count = 4};
    else if (s->has_deck)
      d = (Dialog){.speaker = "Prof. Hazel Quill",
                   .lines = {L("Oh my! A real Voxelmon! Then you are ready.", VOICE("e99f9d41")),
                             L("Press 1 and 2 in battle to use its attacks. Good luck out there!", VOICE("b9510f9e"))},
                   .count = 2, .on_end = ACT_OPEN_GATE};
    break;
  default:
    break;
  }
  return d;
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
  clip_stop(&ds->voice, 0.08f);
  CString v = ds->d.lines[ds->index].voice;
  if (v)
    clip_play(w, v, &(EcsAudioPlayDesc){.volume = 0.95f, .bus = AUDIO_BUS_VOICE}, &ds->voice);
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
  if (ds->chosen == ACT_NONE)
    ds->chosen = ds->d.on_end;
  clip_stop(&ds->voice, 0.15f);
}

hz_internal b32 dialog_has_choices(const DialogState *ds) { return ds->d.choice[0] != NULL; }

// main lane, before the scope: next / choose / close. `advance` = E, Space, Enter or the touch button
hz_internal void dialog_update(DialogState *ds, EcsWorld *w, AudioSystem *audio, const InputSystem *in, b32 advance) {
  if (!ds->open)
    return;
  b32 last = ds->index + 1 >= ds->d.count;
  i32 nchoice = ds->d.choice[2] ? 3 : ds->d.choice[1] ? 2 : ds->d.choice[0] ? 1 : 0;
  if (last && nchoice) {
    if (in->buttons[KEY_LEFT].pressed_this_frame || in->buttons[KEY_A].pressed_this_frame || in->buttons[KEY_UP].pressed_this_frame)
      ds->choice_hover = MAX(0, ds->choice_hover - 1);
    if (in->buttons[KEY_RIGHT].pressed_this_frame || in->buttons[KEY_D].pressed_this_frame || in->buttons[KEY_DOWN].pressed_this_frame)
      ds->choice_hover = MIN(nchoice - 1, ds->choice_hover + 1);
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
  // day: a white card with dark ink; night (the game's clock): a dark card with light text (ui_night_mix)
  HzUIColor ink = {27, 42, 52, 255};
  HzUIColor text = ui_night_mix(ink, (HzUIColor){236, 241, 246, 255});
  ui_element({.id = ui_id("DialogBox"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                           .offset = {0.0f, -safe.bottom - (touch ? 140.0f : 22.0f)}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8,
                         .sizing = {.width = ui_sizing_fixed(m_minf(760.0f, screen_w - 32.0f))},
                         .padding = {.left = 20, .right = 20, .top = 14, .bottom = 14}},
              .background_color = ui_night_mix((HzUIColor){250, 250, 246, 245}, (HzUIColor){24, 32, 42, 242}),
              .border = {.color = ui_night_mix(ink, (HzUIColor){70, 86, 104, 255}), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
              .corner_radius = ui_corner_radius(12)}) {
    if (ui_hovered() && click && !(last && dialog_has_choices(ds)))
      ds->clicked = true;
    ui_element({.layout = {.padding = {.left = 10, .right = 10, .top = 3, .bottom = 3}},
                .background_color = {242, 205, 55, 255},
                .corner_radius = ui_corner_radius(6)}) {
      ui_text(ui_str_c(ds->d.speaker), ui_text_config({.fontSize = 16, .textColor = ink, .wrapMode = UI_TEXT_WRAP_NONE}));
    }
    ui_text(ui_str_c(ds->d.lines[ds->index].text),
            ui_text_config({.fontSize = 20, .textColor = text, .wrapMode = UI_TEXT_WRAP_WORDS}));
    if (last && dialog_has_choices(ds)) {
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 12, .sizing = {.width = ui_sizing_grow(0)}}}) {
        for (i32 i = 0; i < 3; i++) {
          if (!ds->d.choice[i])
            continue;
          b32 sel = ds->choice_hover == i;
          ui_element({.id = ui_idi("DialogChoice", (u32)i),
                      .layout = {.padding = {.left = 16, .right = 16, .top = 8, .bottom = 8}},
                      .background_color = sel ? (HzUIColor){0, 85, 191, 255} : ui_night_mix((HzUIColor){224, 230, 236, 255}, (HzUIColor){44, 56, 70, 255}),
                      .corner_radius = ui_corner_radius(8)}) {
            if (ui_hovered()) {
              ds->choice_hover = i;
              if (click)
                ds->clicked_choice = i;
            }
            ui_text(ui_str_c(ds->d.choice[i]),
                    ui_text_config({.fontSize = 18, .textColor = sel ? (HzUIColor){255, 255, 255, 255} : text,
                                    .wrapMode = UI_TEXT_WRAP_NONE}));
          }
        }
      }
    } else {
      ui_text(ui_str_c(touch ? "tap to continue" : "E / Space / click to continue"),
              ui_text_config({.fontSize = 13, .textColor = ui_night_mix((HzUIColor){120, 128, 136, 255}, (HzUIColor){140, 154, 170, 255}),
                              .wrapMode = UI_TEXT_WRAP_NONE}));
    }
  }
}
