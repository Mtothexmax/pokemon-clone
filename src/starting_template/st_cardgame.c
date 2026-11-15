// The card game around the village (three.js +page.svelte): the battles against Dex and Lumi with their music and
// taunts, what a battle earns (studs, EXP, cards), the reward screen, the deck menu (C) and the legendary card for
// all golden studs. Included by starting_template.c above app_init, after st_life.c.

typedef enum MusicTrack { MUSIC_NONE, MUSIC_HOME, MUSIC_INTRO, MUSIC_FIGHT, MUSIC_VICTORY, MUSIC_DEFEAT, MUSIC_WILD } MusicTrack;

typedef struct CardGame {
  CardUi ui;
  Battle b;
  b32 in_battle;
  f32 flash;        // the way into a battle (seconds left of BATTLE_TRANS; cg_transition_draw)
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
                    "brickhollow/music/victory.hzt", "brickhollow/music/defeat.hzt",
                    "brickhollow/music/wild.hzt"}; // wild: a Voxelmon fight in the tall grass (st_vox.c)
  // as quiet as in the three.js version (VOL 0.15): the voices (0.95) must stay clear above the music
  f32 vol[] = {0.0f, 0.15f, 0.15f, 0.15f, 0.15f, 0.15f, 0.15f};
  clip_stop(&cg->music, 0.4f);
  cg->track = t;
  cg->track_t = 0.0f;
  if (t != MUSIC_NONE)
    clip_play(w, path[t], &(EcsAudioPlayDesc){.volume = vol[t], .loop = t == MUSIC_HOME || t == MUSIC_FIGHT || t == MUSIC_WILD, .bus = AUDIO_BUS_MUSIC},
              &cg->music);
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
  clip_stop(&cg->taunt_voice, 0.05f);
  if (t.voice)
    clip_play(w, t.voice, &(EcsAudioPlayDesc){.volume = 0.95f, .bus = AUDIO_BUS_VOICE}, &cg->taunt_voice);
}

hz_internal void cg_init(CardGame *cg, EcsWorld *w, AudioSystem *audio, u32 seed) {
  cg->ui.selected = cg->ui.hover_card = cg->ui.click_card = cg->ui.click_zone = -1;
  cg->ui.deck_pick = cg->ui.click_coll = cg->ui.last_coll = cg->ui.click_slot = cg->ui.up_card = -1;
  cg->ui.drag = cg->ui.press_card = cg->ui.click_play_to = -1;
  cg->ui.confetti_t = -1.0f;
  cg->seed = seed | 1u;
  cg_music(cg, w, audio, MUSIC_HOME);
}

hz_internal b32 cg_busy(const CardGame *cg) { return cg->in_battle || cg->flash > 0.0f || cg->ui.deck_open || cg->ui.reward_n > 0; }

// ---- the way into a battle (BATTLE_TRANS s): a white flash, rows of LEGO bricks slide in from both sides and cover
// the town, "YOU vs <opponent>" slams in, the board is set up under the bricks, and they slide on and away ----
#define BATTLE_TRANS 2.1f
#define BATTLE_TRANS_BOARD 1.55f // the board starts here (under the cover)

hz_internal f32 cg_ease_back(f32 x) { // overshoots a little, then settles (css ease-out-back)
  x = m_clampf(x, 0.0f, 1.0f) - 1.0f;
  return 1.0f + 2.7f * x * x * x + 1.7f * x * x;
}

hz_internal void cg_transition_draw(const CardGame *cg, const GameSave *s, f32 sw, f32 sh, f32 e) {
  static const u32 colors[5] = {0xc91a09, 0xf2cd37, 0x0055bf, 0x237841, 0xfe8a18};
  // the bricks, seen from above: 2 x 4 studs, rows alternately from the left and the right, in and on out again
  f32 rh = m_maxf(sh / 6.0f, 60.0f), bw = rh * 2.0f;
  i32 rows = (i32)m_ceilf(sh / rh), cols = (i32)m_ceilf(sw / bw) + 2;
  for (i32 r = 0; r < rows; r++) {
    f32 dir = (r & 1) ? 1.0f : -1.0f;
    f32 pin = cu_ease_out(m_clampf((e - 0.18f - 0.045f * (f32)r) / 0.4f, 0.0f, 1.0f));
    f32 pout = m_clampf((e - 1.65f - 0.035f * (f32)r) / 0.32f, 0.0f, 1.0f);
    pout = pout * pout;
    if (pin <= 0.0f || pout >= 1.0f)
      continue;
    f32 x = dir * (1.0f - pin) * (sw + bw) - dir * pout * (sw + bw) - ((r & 1) ? bw * 0.5f : 0.0f);
    for (i32 c = 0; c < cols; c++) {
      u32 hash = (u32)(r * 7 + c * 13 + r * c);
      HzUIColor col = cu_rgb(colors[hash % 5]);
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 80, .offset = {x + (f32)c * bw - bw * 0.5f, (f32)r * rh}},
                  .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(bw), .height = ui_sizing_fixed(rh)},
                             .padding = ui_padding_all((u16)(rh * 0.1f)), .child_gap = (u16)(rh * 0.1f)},
                  .background_color = col,
                  .corner_radius = ui_corner_radius(rh * 0.06f),
                  .border = {.color = cu_mix(col, cu_rgb(0x000000), 0.35f), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                  .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {
        for (i32 sr = 0; sr < 2; sr++)
          ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}, .child_gap = (u16)(rh * 0.12f)}}) {
            for (i32 st = 0; st < 4; st++)
              ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(rh * 0.28f), .height = ui_sizing_fixed(rh * 0.28f)}},
                          .gradient = cu_lin2(135.0f, cu_mix(col, cu_rgb(0xffffff), 0.35f), cu_mix(col, cu_rgb(0x000000), 0.12f)),
                          .corner_radius = ui_corner_radius(rh * 0.14f),
                          .shadow = {.color = cu_black(0.35f), .offset_x = rh * 0.025f, .offset_y = rh * 0.04f, .blur = rh * 0.04f}}) {}
          }
      }
    }
  }
  // YOU vs the opponent: two panels slide in, the VS slams down, everything leaves with the bricks
  f32 pv = cu_ease_out(m_clampf((e - 0.75f) / 0.22f, 0.0f, 1.0f)), out = m_clampf((e - 1.62f) / 0.25f, 0.0f, 1.0f);
  if (pv > 0.0f && out < 1.0f) {
    CString opp = cg->pending_kind == BATTLE_RIVAL ? (s->rival_wins ? "DEX \xe2\x98\x85" : "DEX") : "LUMI";
    f32 bh = m_clampf(sh * 0.2f, 90.0f, 170.0f), fs = bh * 0.38f, y = sh * 0.5f - bh * 0.5f;
    for (i32 side = 0; side < 2; side++) {
      f32 dir = side == 0 ? -1.0f : 1.0f;
      f32 px = side == 0 ? 0.0f : sw * 0.5f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 81,
                               .offset = {px + dir * (1.0f - pv) * sw * 0.6f + dir * out * sw * 0.6f, y + (side ? bh * 0.12f : -bh * 0.12f)}},
                  .layout = {.sizing = {.width = ui_sizing_fixed(sw * 0.5f), .height = ui_sizing_fixed(bh)},
                             .child_alignment = {.x = side == 0 ? UI_ALIGN_X_RIGHT : UI_ALIGN_X_LEFT, .y = UI_ALIGN_Y_CENTER},
                             .padding = {.left = (u16)(bh * 0.55f), .right = (u16)(bh * 0.55f)}},
                  .gradient = side == 0 ? cu_lin2(90.0f, cu_rgb(0x0b3d91), cu_rgb(0x1e6fe0)) : cu_lin2(90.0f, cu_rgb(0xe0301e), cu_rgb(0x8f1306)),
                  .shadow = {.color = cu_black(0.45f), .offset_y = 6.0f, .blur = 18.0f},
                  .transform = cu_tf(0, 0, 1, side == 0 ? -4.0f : -4.0f),
                  .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {
        HzUITextConfig c = {.fontSize = fs, .textColor = cu_rgb(0xffffff), .weight = 800};
        c.shadows[0] = (HzUITextShadow){.color = cu_black(0.5f), .offset_y = fs * 0.06f, .blur = fs * 0.1f};
        ui_text(ui_str_c(side == 0 ? "YOU" : opp), c);
      }
    }
    // the VS: falls in big, overshoots, settles; a white ring behind it
    f32 pvs = m_clampf((e - 0.95f) / 0.22f, 0.0f, 1.0f);
    if (pvs > 0.0f) {
      f32 sc = 2.6f - 1.6f * cg_ease_back(pvs);
      f32 d = bh * 1.15f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 82, .offset = {sw * 0.5f - d * 0.5f, sh * 0.5f - d * 0.5f}},
                  .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)},
                             .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                  .gradient = cu_rad(0.5f, 0.4f, cu_rgb(0xfff3a0), cu_rgb(0xf2a705), 1.0f),
                  .corner_radius = ui_corner_radius(d * 0.5f),
                  .border = {.color = cu_rgb(0xffffff), .width = {.left = 5, .right = 5, .top = 5, .bottom = 5}},
                  .shadow = {.color = cu_black(0.5f), .offset_y = 6.0f, .blur = 20.0f},
                  .opacity = opt_some(f32, m_minf(1.0f, pvs * 3.0f) * (1.0f - out)),
                  .transform = cu_tf(0, 0, sc * (1.0f - 0.6f * out), -8.0f + 8.0f * pvs),
                  .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {
        HzUITextConfig c = {.fontSize = d * 0.42f, .textColor = cu_rgb(0xc91a09), .weight = 800};
        c.shadows[0] = (HzUITextShadow){.color = cu_rgb(0xffffff), .offset_y = d * 0.025f, .blur = 0.0f};
        ui_text(ui_str_c("VS"), c);
      }
    }
  }
  // the flash: two quick white blinks first, one more when the VS lands
  f32 fl = 0.0f;
  if (e < 0.32f)
    fl = 0.85f * m_maxf(0.0f, m_sinf(e / 0.32f * 2.0f * M_PI));
  if (e > 1.17f && e < 1.35f)
    fl = 0.55f * (1.0f - (e - 1.17f) / 0.18f);
  if (fl > 0.0f)
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 83},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                .background_color = cu_rgba(0xffffff, fl),
                .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {}
}

// a dialogue chose a battle: the transition and the intro fanfare, then the board
hz_internal void cg_start_battle(CardGame *cg, BattleKind kind, EcsWorld *w, AudioSystem *audio) {
  cg->pending_kind = kind;
  cg->flash = BATTLE_TRANS;
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
  cg->ui.reward_t = 0.0f; // every card comes in a sealed foil pack, torn open with a tap
  for (i32 i = 0; i < 3; i++)
    cg->ui.reward_tear[i] = PACK_SEALED;
  cg->ui.click_pack = -1;
  cg->ui.click_reveal_all = false;
}

// the battle is over and the result was seen: studs, EXP, cards, then what the opponent says (battleDone)
hz_internal void cg_battle_done(CardGame *cg, SaveState *save, Life *life, DialogState *dlg, EcsWorld *w, AudioSystem *audio) {
  GameSave *s = &save->data;
  Battle *b = &cg->b;
  cg->in_battle = false;
  clip_stop(&cg->taunt_voice, 0.1f);
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
  if (cg->ui.up_card >= 0) { // the upgrade moment's sounds: the flip (or the burst), then a jingle
    f32 t0 = cg->ui.up_t;
    cg->ui.up_t += dt;
    f32 t1 = cg->ui.up_t;
    b32 vox = cg->ui.up_to >= VOXEL_LEVEL;
    if (t0 < 0.3f && t1 >= 0.3f)
      sound_play(snd, SND_BLIP, 0.6f, 0.4f);
    if (!vox && t0 < UP_SWAP && t1 >= UP_SWAP) {
      sound_play(snd, SND_POP, 1.2f, 0.8f);
      sound_play(snd, SND_COIN, 1.0f, 0.8f);
    }
    if (vox && t0 < UP_BURST && t1 >= UP_BURST) {
      sound_play(snd, SND_KICK, 0.5f, 1.0f);
      sound_play(snd, SND_FANFARE, 1.0f, 0.9f);
    }
  }
  cg->ui.time += dt;
  if (cg->ui.confetti_t >= 0.0f)
    cg->ui.confetti_t += dt;
  if (cg->taunt_t > 0.0f)
    cg->taunt_t -= dt;
  // the intro fanfare hands over to the fight music (as the three.js one at 10.8 s)
  if (cg->track == MUSIC_INTRO && cg->track_t > 10.8f)
    cg_music(cg, w, audio, cg->in_battle ? MUSIC_FIGHT : MUSIC_HOME);
  if (cg->flash > 0.0f) {
    f32 e0 = BATTLE_TRANS - cg->flash;
    cg->flash = m_maxf(0.0f, cg->flash - dt);
    f32 e1 = BATTLE_TRANS - cg->flash;
    if (e0 < 0.2f && e1 >= 0.2f)
      sound_play(snd, SND_KICK, 0.7f, 0.5f); // the bricks rush in
    if (e0 < 1.12f && e1 >= 1.12f)
      sound_play(snd, SND_POP, 0.55f, 0.9f); // the VS lands
    if (!cg->in_battle && e1 >= BATTLE_TRANS_BOARD)
      cg_begin_board(cg, s, w, audio); // under the bricks; they slide away from it
    if (!cg->in_battle)
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
    // the packs: a tap tears one open (a crinkle; the card pops out at PACK_REVEAL with a jingle), "Reveal all" or a key
    // opens the rest one after another; once every card is out, the same keys or "Awesome!" close the screen
    CardUi *u = &cg->ui;
    u->reward_t += dt;
    b32 keys = in->buttons[KEY_ENTER].pressed_this_frame || in->buttons[KEY_E].pressed_this_frame || in->buttons[KEY_SPACE].pressed_this_frame;
    b32 all = true;
    for (i32 i = 0; i < u->reward_n; i++) {
      f32 *t = &u->reward_tear[i];
      if (*t > PACK_SEALED + 1.0f) {
        f32 before = *t;
        *t += dt;
        if (before < 0.0f && *t >= 0.0f && i > 0)
          sound_play(snd, SND_SPLASH, 1.9f, 0.35f);
        if (before < PACK_REVEAL && *t >= PACK_REVEAL)
          sound_play(snd, pack_rare(u->reward[i]) ? SND_FANFARE : SND_COIN, 1.0f + 0.12f * (f32)i, 0.7f);
      }
      all &= *t >= PACK_REVEAL;
    }
    if (u->click_pack >= 0 && u->click_pack < u->reward_n && u->reward_tear[u->click_pack] <= PACK_SEALED + 1.0f) {
      u->reward_tear[u->click_pack] = 0.0f;
      sound_play(snd, SND_SPLASH, 1.9f, 0.45f);
    } else if (!all && (u->click_reveal_all || keys)) {
      i32 k = 0;
      for (i32 i = 0; i < u->reward_n; i++)
        if (u->reward_tear[i] <= PACK_SEALED + 1.0f)
          u->reward_tear[i] = -0.26f * (f32)k++;
      sound_play(snd, SND_SPLASH, 1.9f, 0.45f);
      keys = false;
    }
    u->click_pack = -1;
    u->click_reveal_all = false;
    if (all && (u->click_reward_ok || keys)) {
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
  if (cg->flash > 0.0f)
    cg_transition_draw(cg, s, sw, sh, BATTLE_TRANS - cg->flash);
  if (cg->in_battle) {
    battle_draw(&cg->ui, &cg->b, in, s, sw, sh);
    if (cg->taunt_t > 0.0f && cg->taunt) {
      // the opponent's speech bubble under its name, with a little tail
      f32 age = 3.0f - cg->taunt_t, pop = m_clampf(age / 0.25f, 0.0f, 1.0f);
      ui_element({.id = ui_id("Taunt"),
                  .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 58, .offset = {14.0f + os_get_safe_area().left, 48.0f + os_get_safe_area().top + 6.0f * (1.0f - pop)}},
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
    // the upgrade moment over the deck; a new Voxelmon opens the view onto the town, where it rises in 3D
    if (!(cg->ui.up_card >= 0 && cg->ui.up_to >= VOXEL_LEVEL && cg->ui.up_t > UP_BURST + 0.3f))
      deck_draw(&cg->ui, (GameSave *)s, in, sw, sh, s->voxelmon > 0);
    up_draw(&cg->ui, s, in, sw, sh);
  } else if (show_button && s->has_deck) {
    // the Deck button, top right left of the menu button (narrow screens: the second row, right)
    OsSafeAreaInsets safe = os_get_safe_area();
    b32 narrow = g_hud.narrow;
    ui_element({.id = ui_id("DeckButton"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                             .offset = narrow ? (v2){-safe.right - 14.0f, g_hud.row2} : (v2){-safe.right - 62.0f, safe.top + 14.0f}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8, .padding = {.left = 14, .right = 14, .top = 8, .bottom = 8},
                           .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .background_color = {0, 85, 191, 235},
                .corner_radius = ui_corner_radius(10)}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
        cg->ui.deck_open = true;
      if (narrow) // short: the EXP is in the goal and the deck screen
        cstr_format(cg->ui.text[6], sizeof(cg->ui.text[6]), "Deck \xc2\xb7 % cards", fmt_u32(owned_count(s)));
      else
        cstr_format(cg->ui.text[6], sizeof(cg->ui.text[6]), "Deck (C)  % cards \xc2\xb7 % EXP", fmt_u32(owned_count(s)), fmt_u32(s->exp));
      cu_text(cg->ui.text[6], narrow ? 14 : 15, (HzUIColor){255, 255, 255, 255});
    }
  }
  rewards_draw(&cg->ui, s, in, sw);
}

// ---------------------------------------------------------------- the goal and the markers

// the current goal (the three.js quest line), top left
hz_internal void cg_quest_draw(CardGame *cg, const GameSave *s) {
  CString text;
  if (!s->has_deck && !s->quest_part)
    text = "Visit Prof. Quill in her lab (south-east)";
  else if (!s->has_deck && !s->has_part && !s->has_suit)
    text = "Find Prof. Quill's old diving suit somewhere in the lab";
  else if (!s->has_deck && !s->has_part)
    text = "Dive into the pond and find the Stud Core, deep in the cave";
  else if (!s->has_deck)
    text = "Bring the Stud Core to Prof. Quill";
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
                         .sizing = {.width = ui_sizing_fit(0.0f, g_hud.quest_w)},
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = ui_night_mix((HzUIColor){250, 250, 246, 225}, (HzUIColor){24, 32, 42, 225}), // dark at night
              .corner_radius = ui_corner_radius(10)}) {
    ui_element({.layout = {.padding = {.left = 7, .right = 7, .top = 1, .bottom = 1}}, .background_color = {242, 205, 55, 255},
                .corner_radius = ui_corner_radius(5)}) {
      cu_text("GOAL", 12, (HzUIColor){27, 42, 52, 255});
    }
    cu_text_wrap(text, g_hud.narrow ? 13 : 15, ui_night_mix((HzUIColor){27, 42, 52, 255}, (HzUIColor){232, 238, 244, 255})); // wraps
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
  mark[6] = !s->has_deck && (!s->quest_part || s->has_part); // Prof. Quill: the start, and the Stud Core
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
