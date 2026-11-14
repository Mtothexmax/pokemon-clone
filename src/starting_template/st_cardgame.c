// The card game around the village (three.js +page.svelte): the battles against Dex and Lumi with their music and
// taunts, what a battle earns (studs, EXP, cards), the reward screen, the deck menu (C) and the legendary card for
// all golden studs. Included by starting_template.c above app_init, after st_life.c.

typedef enum MusicTrack { MUSIC_NONE, MUSIC_HOME, MUSIC_INTRO, MUSIC_FIGHT, MUSIC_VICTORY, MUSIC_DEFEAT } MusicTrack;

typedef struct CardGame {
  CardUi ui;
  Battle b;
  b32 in_battle;
  f32 flash;        // the white flash before a battle (seconds left)
  BattleKind pending_kind;
  MusicTrack track;
  HzHandleT(AudioVoice) music;
  f32 track_t;      // seconds the track has played
  DialogId after;   // the dialogue once the rewards are closed
  b32 has_after;
  // the opponent's speech bubble
  CString taunt;
  f32 taunt_t;
  i32 last_turn;
  b32 ai_snap_said, celebrated;
  HzHandleT(AudioVoice) taunt_voice;
  char dlg_buf[128];
  char toast[96];
  u32 seed;
} CardGame;

hz_internal void cg_music(CardGame *cg, EcsWorld *w, AudioSystem *audio, MusicTrack t) {
  if (cg->track == t)
    return;
  CString path[] = {NULL, "brickhollow/music/home.hzt", "brickhollow/music/battle-intro.hzt", "brickhollow/music/fight.hzt",
                    "brickhollow/music/victory.hzt", "brickhollow/music/defeat.hzt"};
  f32 vol[] = {0.0f, 0.55f, 0.6f, 0.5f, 0.6f, 0.55f};
  audio_voice_stop(audio, cg->music, 0.4f);
  cg->track = t;
  cg->track_t = 0.0f;
  if (t != MUSIC_NONE)
    cg->music = ecs_audio_play(w, path[t], &(EcsAudioPlayDesc){.volume = vol[t], .loop = t == MUSIC_HOME || t == MUSIC_FIGHT,
                                                               .bus = AUDIO_BUS_MUSIC});
}

// ---- what the opponent says (BattleView.svelte LINES; Dex and Lumi are voiced) ----
typedef struct Taunt {
  CString text, voice;
} Taunt;

hz_internal Taunt cg_line(BattleKind k, i32 what, u32 r) {
  // what: 0 start, 1 turn, 2 snap, 3 the opponent won, 4 the opponent lost
  static const Taunt rival[5][4] = {
      {{"Grandma gave YOU a deck? This will be quick.", VOICE("e0b185df")}, {"Ready to lose?", VOICE("2ea3ea6a")}},
      {{"Is that all you got?", VOICE("38c41100")}, {"My Beasts are way cooler.", VOICE("241a4bbf")}, {"Watch this!", VOICE("dd0fe3a0")},
       {"Hmm\xe2\x80\xa6 lucky draw.", VOICE("f2108d20")}},
      {{"SNAP! Double or nothing, slowpoke!", VOICE("a5a25e9f")}, {"SNAP! I like my odds.", VOICE("335ad1f3")}},
      {{"Told you my deck was better!", VOICE("c31240c9")}},
      {{"No way\xe2\x80\xa6 my deck was perfect!", VOICE("e6a74568")}}};
  static const Taunt practice[5][4] = {
      {{"Let\xe2\x80\x99s have a fun practice round!", VOICE("7e9bc202")}},
      {{"Nice move!", VOICE("49205884")}, {"Ooh, careful with that zone.", VOICE("09c03c86")}, {"Remember: win 2 of 3!", VOICE("bdf62197")}},
      {{"Snap! Let\xe2\x80\x99s make it exciting!", VOICE("760dc0c6")}},
      {{"Good try! You\xe2\x80\x99re getting better.", VOICE("1f93971b")}},
      {{"Wow, you learn fast!", VOICE("45311098")}}};
  static const Taunt wild[5][4] = {{{"*rustle rustle*", NULL}, {"Grrrr!", NULL}},
                                   {{"*snarl*", NULL}, {"*brick noises*", NULL}, {"Rrraah!", NULL}},
                                   {{"*SNAP!* (it bares its teeth)", NULL}},
                                   {{"*victory howl*", NULL}},
                                   {{"*runs back into the grass*", NULL}}};
  const Taunt(*set)[4] = k == BATTLE_RIVAL ? rival : k == BATTLE_PRACTICE ? practice : wild;
  i32 n = 0;
  while (n < 4 && set[what][n].text)
    n++;
  return set[what][r % (u32)MAX(n, 1)];
}

hz_internal u32 cg_rand(CardGame *cg) {
  cg->seed = cg->seed * 1664525u + 1013904223u;
  return cg->seed >> 8;
}

hz_internal void cg_say(CardGame *cg, EcsWorld *w, AudioSystem *audio, i32 what, f32 secs) {
  Taunt t = cg_line(cg->b.kind, what, cg_rand(cg));
  cg->taunt = t.text;
  cg->taunt_t = secs;
  audio_voice_stop(audio, cg->taunt_voice, 0.05f);
  if (t.voice)
    cg->taunt_voice = ecs_audio_play(w, t.voice, &(EcsAudioPlayDesc){.volume = 1.0f, .bus = AUDIO_BUS_VOICE});
}

hz_internal void cg_init(CardGame *cg, EcsWorld *w, AudioSystem *audio, u32 seed) {
  cg->ui.selected = cg->ui.hover_card = cg->ui.click_card = cg->ui.click_zone = -1;
  cg->ui.deck_pick = cg->ui.click_coll = cg->ui.click_slot = -1;
  cg->ui.drag = cg->ui.press_card = cg->ui.click_play_to = -1;
  cg->ui.confetti_t = -1.0f;
  cg->seed = seed | 1u;
  cg_music(cg, w, audio, MUSIC_HOME);
}

hz_internal b32 cg_busy(const CardGame *cg) { return cg->in_battle || cg->flash > 0.0f || cg->ui.deck_open || cg->ui.reward_n > 0; }

// a dialogue chose a battle: the flash and the intro fanfare, then the board
hz_internal void cg_start_battle(CardGame *cg, BattleKind kind, EcsWorld *w, AudioSystem *audio) {
  cg->pending_kind = kind;
  cg->flash = 0.75f;
  cg_music(cg, w, audio, MUSIC_INTRO);
}

hz_internal void cg_begin_board(CardGame *cg, const GameSave *s, EcsWorld *w, AudioSystem *audio) {
  i32 ai[DECK_SIZE];
  i32 deck = cg->pending_kind == BATTLE_RIVAL ? (s->rival_wins ? DECK_RIVAL_2 : DECK_RIVAL) : DECK_PRACTICE;
  for (i32 i = 0; i < DECK_SIZE; i++)
    ai[i] = deck_card(deck, i);
  CString opp = cg->pending_kind == BATTLE_RIVAL ? (s->rival_wins ? "Dex \xe2\x98\x85 Rematch" : "Dex") : "Lumi";
  b_init(&cg->b, cg->pending_kind, opp, s->deck, ai, false, cg_rand(cg));
  cg->in_battle = true;
  cg->ui.selected = -1;
  cg->ui.drag = cg->ui.press_card = cg->ui.click_play_to = -1;
  cg->ui.inspect = INSPECT_NONE;
  cg->ui.log_open = false;
  cg->ui.confetti_t = -1.0f;
  for (i32 i = 0; i < B_MAX_CARDS; i++)
    cg->ui.has_box[i] = false;
  cg->last_turn = cg->b.turn;
  cg->ai_snap_said = cg->celebrated = false;
  cg_say(cg, w, audio, 0, 3.0f);
}

hz_internal void cg_show_rewards(CardGame *cg, CString title, const i32 *ids, i32 n) {
  cg->ui.reward_n = MIN(n, 3);
  for (i32 i = 0; i < cg->ui.reward_n; i++)
    cg->ui.reward[i] = ids[i];
  cstr_format(cg->ui.text[5], sizeof(cg->ui.text[5]), "%", fmt_cstr(title));
  cg->ui.reward_title = cg->ui.text[5];
}

// the battle is over and the result was seen: studs, EXP, cards, then what the opponent says (battleDone)
hz_internal void cg_battle_done(CardGame *cg, SaveState *save, Life *life, DialogState *dlg, EcsWorld *w, AudioSystem *audio) {
  GameSave *s = &save->data;
  Battle *b = &cg->b;
  cg->in_battle = false;
  audio_voice_stop(audio, cg->taunt_voice, 0.1f);
  cg->taunt_t = 0.0f;
  cg_music(cg, w, audio, MUSIC_HOME);
  BattleResult r = b->result;
  i32 delta = 0;
  if (r == RESULT_WIN) {
    delta = b->stakes;
    s->wins++;
  } else if (r == RESULT_LOSE) {
    delta = -b->stakes;
    s->losses++;
  } else if (r == RESULT_RETREAT)
    delta = -MAX(1, b->stakes / 2);
  s->studs = (u32)MAX(0, (i32)s->studs + delta);
  u32 exp = battle_exp((i32)r, b->kind == BATTLE_RIVAL);
  s->exp += exp;
  i32 ids[2], n = 0;
  if (r == RESULT_WIN) {
    i32 k = b->kind == BATTLE_RIVAL && s->rival_wins == 0 ? 2 : 1;
    for (i32 i = 0; i < k; i++) {
      i32 c = reward_card(s, &cg->seed, false);
      if (c >= 0)
        ids[n++] = c;
    }
    if (b->kind == BATTLE_RIVAL)
      s->rival_wins++;
  }
  save_mark(save);
  cg->has_after = true;
  cg->after = b->kind == BATTLE_RIVAL ? (r == RESULT_WIN ? DLG_DEX_WON : DLG_DEX_LOST) : (r == RESULT_WIN ? DLG_LUMI_WON : DLG_LUMI_LOST);
  cstr_format(cg->toast, sizeof(cg->toast), "+% EXP \xc2\xb7 upgrade your cards in the Deck menu (C)", fmt_u32(exp));
  life_toast(life, cg->toast);
  if (n) {
    char title[96];
    cstr_format(title, sizeof(title), "New card added to your collection! \xc2\xb7 +% EXP", fmt_u32(exp));
    cg_show_rewards(cg, title, ids, n);
  } else {
    cg->has_after = false;
    dialog_open(dlg, dialog_for(cg->after, s, s->voxelmon > 0, cg->dlg_buf, sizeof(cg->dlg_buf)), w, audio);
  }
}

// main lane, before the scope. Returns true while the cards own the screen (no walking, no talking)
hz_internal b32 cg_update(CardGame *cg, SaveState *save, Life *life, DialogState *dlg, SoundBank *snd, EcsWorld *w, AudioSystem *audio,
                          const InputSystem *in, b32 busy_world, f32 dt) {
  GameSave *s = &save->data;
  cg->track_t += dt;
  cg->ui.time += dt;
  if (cg->ui.confetti_t >= 0.0f)
    cg->ui.confetti_t += dt;
  if (cg->taunt_t > 0.0f)
    cg->taunt_t -= dt;
  // the intro fanfare hands over to the fight music (as the three.js one at 10.8 s)
  if (cg->track == MUSIC_INTRO && cg->track_t > 10.8f)
    cg_music(cg, w, audio, cg->in_battle ? MUSIC_FIGHT : MUSIC_HOME);
  if (cg->flash > 0.0f) {
    cg->flash -= dt;
    if (cg->flash <= 0.0f)
      cg_begin_board(cg, s, w, audio);
    return true;
  }
  // the legendary card for all golden studs
  if (s->legendary && !owns(s, CARD_GOLDBRICK) && !cg_busy(cg) && !dlg->open) {
    give_card(s, CARD_GOLDBRICK);
    save_mark(save);
    i32 id = CARD_GOLDBRICK;
    cg_show_rewards(cg, "All golden studs found! A legendary card appears\xe2\x80\xa6", &id, 1);
  }
  if (cg->ui.reward_n) {
    if (cg->ui.click_reward_ok || in->buttons[KEY_ENTER].pressed_this_frame || in->buttons[KEY_E].pressed_this_frame ||
        in->buttons[KEY_SPACE].pressed_this_frame) {
      cg->ui.click_reward_ok = false;
      cg->ui.reward_n = 0;
      sound_play(snd, SND_BLIP, 1.2f, 0.4f);
      if (cg->has_after) {
        cg->has_after = false;
        dialog_open(dlg, dialog_for(cg->after, s, s->voxelmon > 0, cg->dlg_buf, sizeof(cg->dlg_buf)), w, audio);
      }
    }
    return true;
  }
  if (cg->in_battle) {
    Battle *b = &cg->b;
    if (b->phase != PHASE_OVER) {
      battle_input(&cg->ui, b, in, snd);
    } else {
      cg->ui.click_card = cg->ui.click_zone = cg->ui.press_card = cg->ui.drag = -1;
      cg->ui.inspect = INSPECT_NONE;
    }
    i32 before = b->phase;
    b_update(b, dt);
    // the sounds of the reveal: a card turns over, a hit, a boost, a card shatters
    if (b->sfx_reveals) {
      sound_play(snd, SND_POP, 1.1f, 0.45f);
      b->sfx_reveals = 0;
    }
    if (b->sfx_hits) {
      sound_play(snd, SND_KICK, 1.15f, 0.7f);
      b->sfx_hits = 0;
    }
    if (b->sfx_buffs) {
      sound_play(snd, SND_COIN, 1.25f, 0.4f);
      b->sfx_buffs = 0;
    }
    if (b->sfx_kills) {
      sound_play(snd, SND_SPLASH, 1.6f, 0.6f);
      sound_play(snd, SND_KICK, 0.7f, 0.8f);
      b->sfx_kills = 0;
    }
    if (b->turn > cg->last_turn) {
      if (b->turn < b->max_turns && (cg_rand(cg) % 100u) < 45u)
        cg_say(cg, w, audio, 1, 2.6f);
      cg->last_turn = b->turn;
      sound_play(snd, SND_BLIP, 0.9f, 0.4f);
    }
    if (b->ai_snapped && !cg->ai_snap_said) {
      cg->ai_snap_said = true;
      cg_say(cg, w, audio, 2, 3.2f);
      sound_play(snd, SND_ALERT, 1.0f, 0.6f);
    }
    if (b->phase == PHASE_OVER && !cg->celebrated) {
      cg->celebrated = true;
      if (b->result == RESULT_WIN) {
        cg_music(cg, w, audio, MUSIC_VICTORY);
        cg->ui.confetti_t = 0.0f;
        sound_play(snd, SND_FANFARE, 1.0f, 0.8f);
        cg_say(cg, w, audio, 4, 4.0f);
      } else if (b->result == RESULT_LOSE) {
        cg_music(cg, w, audio, MUSIC_DEFEAT);
        cg_say(cg, w, audio, 3, 4.0f);
      }
    }
    (void)before;
    if (cg->ui.click_done || (b->phase == PHASE_OVER && b->result == RESULT_RETREAT)) {
      cg->ui.click_done = false;
      cg_battle_done(cg, save, life, dlg, w, audio);
    }
    return true;
  }
  if (cg->ui.deck_open) {
    if (deck_input(&cg->ui, s, save, snd, in))
      cg->ui.deck_open = false;
    return true;
  }
  // C opens the deck menu once you have cards
  if (s->has_deck && !busy_world && in->buttons[KEY_C].pressed_this_frame) {
    cg->ui.deck_open = true;
    sound_play(snd, SND_BLIP, 1.4f, 0.4f);
    return true;
  }
  return false;
}

// a dialogue's choice or end that belongs to the cards; returns true when it was handled
hz_internal b32 cg_dialog_action(CardGame *cg, DialogAction act, SaveState *save, DialogState *dlg, SoundBank *snd, EcsWorld *w,
                                 AudioSystem *audio) {
  GameSave *s = &save->data;
  switch (act) {
  case ACT_GIVE_STARTER: {
    if (s->has_deck)
      return true;
    give_starter(s);
    save_mark(save);
    sound_play(snd, SND_FANFARE, 1.0f, 0.8f);
    i32 ids[3] = {CARD_TITANOAK, CARD_THUNDRAKE, CARD_EMBERHORN};
    cg_show_rewards(cg, "You received your starter collection! 18 cards, and a deck of 12 (C)", ids, 3);
    cg->after = DLG_PROF_AFTER;
    cg->has_after = true;
    return true;
  }
  case ACT_BATTLE_RIVAL: cg_start_battle(cg, BATTLE_RIVAL, w, audio); return true;
  case ACT_BATTLE_PRACTICE: cg_start_battle(cg, BATTLE_PRACTICE, w, audio); return true;
  case ACT_DEX_SCARED:
    dialog_open(dlg, dialog_for(DLG_DEX_SCARED, s, false, cg->dlg_buf, sizeof(cg->dlg_buf)), w, audio);
    return true;
  default: return false;
  }
}

// the UI frame
hz_internal void cg_draw(CardGame *cg, const GameSave *s, const InputSystem *in, f32 sw, f32 sh, f32 dpr, b32 show_button) {
  cg->ui.dpr = dpr;
  if (cg->flash > 0.0f) {
    ui_element({.id = ui_id("BattleFlash"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 80},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                .background_color = {255, 255, 255, (f32)(255.0f * m_clampf(1.0f - cg->flash / 0.75f, 0.0f, 1.0f))}}) {}
  }
  if (cg->in_battle) {
    battle_draw(&cg->ui, &cg->b, in, s, sw, sh);
    if (cg->taunt_t > 0.0f && cg->taunt) {
      // the opponent's speech bubble under its name, with a little tail
      f32 age = 3.0f - cg->taunt_t, pop = m_clampf(age / 0.25f, 0.0f, 1.0f);
      ui_element({.id = ui_id("Taunt"),
                  .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 58, .offset = {14.0f, 48.0f + 6.0f * (1.0f - pop)}},
                  .layout = {.padding = {.left = 14, .right = 14, .top = 8, .bottom = 8}, .sizing = {.width = ui_sizing_fit(0.0f, m_minf(sw * 0.7f, 340.0f))}},
                  .background_color = {255, 255, 255, 255},
                  .border = {.color = {27, 42, 52, 255}, .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
                  .corner_radius = ui_corner_radius(14),
                  .shadow = {.color = {0, 0, 0, 90}, .offset_y = 6.0f, .blur = 16.0f},
                  .transform = cu_tf(0, 0, 0.9f + 0.1f * pop, 0),
                  .opacity = opt_some(f32, pop)}) {
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 58, .offset = {16.0f, -8.0f}},
                    .layout = {.sizing = {.width = ui_sizing_fixed(13), .height = ui_sizing_fixed(13)}},
                    .background_color = {27, 42, 52, 255},
                    .transform = cu_tf(0, 0, 1, 45)}) {}
        HzUITextConfig cfg = {.fontSize = 15, .textColor = {27, 42, 52, 255}, .weight = 700, .wrapMode = UI_TEXT_WRAP_WORDS};
        ui_text(ui_str_c(cg->taunt), cfg);
      }
    }
  } else if (cg->ui.deck_open) {
    deck_draw(&cg->ui, (GameSave *)s, in, sw, s->voxelmon > 0);
  } else if (show_button && s->has_deck) {
    // the Deck button, top right
    OsSafeAreaInsets safe = os_get_safe_area();
    ui_element({.id = ui_id("DeckButton"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                             .offset = {-safe.right - 14.0f, safe.top + 14.0f}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8, .padding = {.left = 14, .right = 14, .top = 8, .bottom = 8},
                           .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .background_color = {0, 85, 191, 235},
                .corner_radius = ui_corner_radius(10)}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
        cg->ui.deck_open = true;
      cstr_format(cg->ui.text[6], sizeof(cg->ui.text[6]), "Deck (C)  % cards \xc2\xb7 % EXP", fmt_u32(owned_count(s)), fmt_u32(s->exp));
      cu_text(cg->ui.text[6], 15, (HzUIColor){255, 255, 255, 255});
    }
  }
  rewards_draw(&cg->ui, s, in, sw);
}

// ---------------------------------------------------------------- the goal and the markers

// the current goal (the three.js quest line), top left
hz_internal void cg_quest_draw(CardGame *cg, const GameSave *s) {
  CString text;
  if (!s->has_deck)
    text = "Visit Prof. Quill in her lab (south-east)";
  else if (!s->rival_wins)
    text = "Beat Dex \xe2\x80\x94 he waits at his big house (east)";
  else if (!s->voxelmon) {
    cstr_format(cg->ui.text[7], sizeof(cg->ui.text[7]), "Win card battles for EXP and upgrade a 6-energy card into a Voxelmon \xc2\xb7 % EXP",
                fmt_u32(s->exp));
    text = cg->ui.text[7];
  } else if (!s->gate_open)
    text = "Show your Voxelmon to Prof. Quill at the village exit (north)";
  else if (owned_count(s) < CARD_POOL) {
    cstr_format(cg->ui.text[7], sizeof(cg->ui.text[7]), "Battle wild Voxelmon on Route 1 \xc2\xb7 %/% cards", fmt_u32(owned_count(s)), fmt_i32(CARD_POOL));
    text = cg->ui.text[7];
  } else
    text = "Collection complete! You are a Brick Beast master.";
  OsSafeAreaInsets safe = os_get_safe_area();
  ui_element({.id = ui_id("Quest"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .offset = {safe.left + 14.0f, safe.top + 14.0f}},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8, .padding = {.left = 12, .right = 14, .top = 7, .bottom = 7},
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = {250, 250, 246, 225},
              .corner_radius = ui_corner_radius(10)}) {
    ui_element({.layout = {.padding = {.left = 7, .right = 7, .top = 1, .bottom = 1}}, .background_color = {242, 205, 55, 255},
                .corner_radius = ui_corner_radius(5)}) {
      cu_text("GOAL", 12, (HzUIColor){27, 42, 52, 255});
    }
    cu_text(text, 15, (HzUIColor){27, 42, 52, 255});
  }
}

// "!" over whoever matters for the goal, "\xe2\x80\xa6" for small talk (updateMarkers): figs 1 Dex, 2 Pip, 3 Prof at the gap,
// 4 Mom, 5 Lumi, 6 Prof in the lab
hz_internal void cg_markers_draw(const GameSave *s, const MinifigSet *figs, EcsWorld *w, const Camera *cam, v3 player, f32 sw, f32 sh,
                                 f32 dpr, b32 mom_talked, b32 kid_talked, u32 golden_n) {
  if (!cam)
    return;
  b32 mark[7] = {0};
  mark[1] = s->has_deck && !s->rival_wins;
  mark[2] = golden_n < 15 && !kid_talked;
  mark[3] = s->voxelmon > 0 && !s->gate_open;
  mark[4] = !s->has_deck && !mom_talked;
  mark[5] = s->has_deck && !s->wins;
  mark[6] = !s->has_deck;
  v3 fwd = camera_forward(cam);
  for (u32 i = 1; i < figs->count && i < 7; i++) {
    const WorldTransform *wt = ecs_component_get(w, figs->figs[i].root, WorldTransform);
    if (!wt)
      continue;
    v3 head = v3_add(wt->position, (v3){0.0f, 2.45f, 0.0f});
    v3 to = v3_sub(head, cam->pos);
    f32 d = v3_length(v3_sub(wt->position, player));
    if (v3_dot(to, fwd) < 0.5f || d > 40.0f * TOWN_STUD || d < 1.2f)
      continue;
    f32 sp[2];
    camera_project_to_screen(cam->view_proj, head, sw * dpr, sh * dpr, sp);
    b32 big = mark[i];
    ui_element({.id = ui_idi("Marker", i),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .offset = {sp[0] / dpr - (big ? 14.0f : 16.0f), sp[1] / dpr - 30.0f}, .z_index = 5},
                .layout = {.sizing = {.width = ui_sizing_fixed(big ? 28.0f : 32.0f), .height = ui_sizing_fixed(big ? 28.0f : 20.0f)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = big ? (HzUIColor){242, 205, 55, 255} : (HzUIColor){250, 250, 246, 220},
                .border = {.color = {27, 42, 52, 255}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                .corner_radius = ui_corner_radius(big ? 14.0f : 10.0f)}) {
      cu_text(big ? "!" : "\xe2\x80\xa6", big ? 20 : 14, (HzUIColor){27, 42, 52, 255});
    }
  }
}
