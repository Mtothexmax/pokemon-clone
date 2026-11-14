// The card screens: one card widget (picture, cost, power, name, the upgrade stage's frame), the battle board, the deck
// and collection (with upgrades and the Voxelmon you walk with) and the rewards. Clicks are read while the UI is built
// and acted on the next frame. Included by starting_template.c above app_init.

typedef struct CardUi {
  CardArt art;
  // battle
  i32 selected;     // the hand card picked to play (-1)
  i32 hover_card;   // battle card index under the mouse
  i32 click_card, click_zone;
  b32 click_end, click_snap, click_retreat, click_done;
  // deck screen
  b32 deck_open;
  i32 deck_pick;    // the card shown big
  i32 click_coll, click_slot;
  b32 click_add, click_remove, click_upgrade, click_walk, click_close;
  // rewards
  i32 reward[3], reward_n;
  CString reward_title;
  b32 click_reward_ok;
  char text[10][96];
} CardUi;

hz_internal HzUIColor cu_rgb(u32 hex) { return (HzUIColor){(f32)((hex >> 16) & 255), (f32)((hex >> 8) & 255), (f32)(hex & 255), 255}; }

hz_internal void cu_text(CString s, u16 size, HzUIColor c) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size, .textColor = c, .wrapMode = UI_TEXT_WRAP_NONE}));
}
hz_internal void cu_text_wrap(CString s, u16 size, HzUIColor c) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size, .textColor = c, .wrapMode = UI_TEXT_WRAP_WORDS}));
}

hz_internal CString cu_num(char *buf, u32 cap, i32 v) {
  cstr_format(buf, cap, "%", fmt_i32(v));
  return buf;
}

// numbers that live as long as the program: many cards are drawn per frame and the UI reads its strings at the end
#define CU_NUM_MIN -60
#define CU_NUM_MAX 99
hz_internal CString cu_numstr(i32 v, b32 plus) {
  static char table[2][CU_NUM_MAX - CU_NUM_MIN + 1][6];
  static b32 made;
  if (!made) {
    made = true;
    for (i32 n = CU_NUM_MIN; n <= CU_NUM_MAX; n++) {
      cstr_format(table[0][n - CU_NUM_MIN], 6, "%", fmt_i32(n));
      cstr_format(table[1][n - CU_NUM_MIN], 6, "%%", fmt_cstr(n > 0 ? "+" : ""), fmt_i32(n));
    }
  }
  v = MIN(MAX(v, CU_NUM_MIN), CU_NUM_MAX);
  return table[plus ? 1 : 0][v - CU_NUM_MIN];
}

// a fsz round badge with a number (cost blue, power orange)
hz_internal void cu_badge(HzUIElementId id, CString num, HzUIColor bg, f32 d, u16 font) {
  ui_element({.id = id,
              .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = bg,
              .border = {.color = {255, 255, 255, 255}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
              .corner_radius = ui_corner_radius(d * 0.5f)}) {
    cu_text(num, font, (HzUIColor){255, 255, 255, 255});
  }
}

typedef struct CardLook {
  i32 id;
  i32 power;   // shown power (may differ from the card's)
  i32 level;   // upgrade stage
  f32 w;       // width in px (height = 1.4 w)
  b32 lit;     // selected / playable highlight
  b32 dim;     // can not be played now: greyed
  b32 unknown; // not collected yet: a dark silhouette, "???"
  b32 hidden;  // the AI's staged card: the back
  u32 uid;     // for the element ids
  i32 fx_n;    // a power change to show (+N / -N), 0 none
} CardLook;

// a card; returns true when it was clicked this frame
hz_internal b32 cu_card(CardUi *ui, CardLook k, const InputSystem *in, HzUIElementId id) {
  b32 clicked = false;
  f32 w = k.w, h = w * 1.4f;
  CardDef d = card_def(k.id);
  KindInfo ki = kind_info(d.kind);
  StageDef sd = stage_def(k.level);
  HzUIColor frame = k.level > 0 ? sd.frame : ki.dark;
  if (k.lit)
    frame = (HzUIColor){255, 255, 255, 255};
  u16 fsz = (u16)m_maxf(9.0f, w * 0.13f);
  ui_element({.id = id,
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)},
                         .padding = ui_padding_all((u16)m_maxf(2.0f, w * 0.04f)), .child_gap = 1,
                         .child_alignment = {.x = UI_ALIGN_X_CENTER}},
              .background_color = k.hidden ? (HzUIColor){60, 50, 90, 255} : frame,
              .corner_radius = ui_corner_radius(w * 0.08f)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    if (k.hidden) {
      cu_text("?", (u16)(w * 0.4f), (HzUIColor){200, 190, 255, 255});
    } else {
      // the picture on the kind's colour, the beast's pixel art on it
      u32 pw = 0, ph = 0;
      GpuTexture tex = card_art(&ui->art, k.id, &pw, &ph);
      f32 iw = w * 0.86f, ih = w * 0.78f;
      ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(iw), .height = ui_sizing_fixed(ih)},
                             .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}},
                  .background_color = k.unknown ? (HzUIColor){50, 54, 60, 255} : k.dim ? (HzUIColor){(f32)ki.color.r * 0.5f, (f32)ki.color.g * 0.5f, (f32)ki.color.b * 0.5f, 255} : ki.color,
                  .corner_radius = ui_corner_radius(w * 0.05f)}) {
        if (pw && ph && handle_is_valid(tex)) {
          f32 sc = m_minf(iw * 0.92f / (f32)pw, ih * 0.92f / (f32)ph);
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed((f32)pw * sc), .height = ui_sizing_fixed((f32)ph * sc)}},
                      .image = {.texture = tex},
                      .background_color = k.unknown ? (HzUIColor){0, 0, 0, 255} : k.dim ? (HzUIColor){130, 130, 130, 255} : (HzUIColor){255, 255, 255, 255}}) {}
        }
      }
      cu_text(k.unknown ? "???" : d.name, fsz, k.dim ? (HzUIColor){170, 170, 170, 255} : (HzUIColor){255, 255, 255, 255});
      if (w >= 90.0f)
        cu_text(ki.label, (u16)m_maxf(8.0f, w * 0.1f), (HzUIColor){255, 255, 255, 200});
      // cost top left, power top right
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT,
                               .attach_points = {.element = UI_ATTACH_POINT_LEFT_TOP, .parent = UI_ATTACH_POINT_LEFT_TOP},
                               .offset = {-w * 0.06f, -w * 0.06f}}}) {
        cu_badge(ui_idi("CostB", k.uid), cu_numstr(d.cost, false), (HzUIColor){0, 85, 191, 255}, w * 0.3f, fsz);
      }
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT,
                               .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                               .offset = {w * 0.06f, -w * 0.06f}}}) {
        HzUIColor pc = k.power > d.power ? (HzUIColor){60, 170, 60, 255} : k.power < d.power ? (HzUIColor){200, 40, 30, 255}
                                                                                              : (HzUIColor){240, 120, 30, 255};
        cu_badge(ui_idi("PowB", k.uid), cu_numstr(k.power, false), pc, w * 0.3f, fsz);
      }
      if (k.fx_n) {
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT,
                                 .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_CENTER_CENTER}},
                    .layout = {.padding = ui_padding_all(4)},
                    .background_color = k.fx_n > 0 ? (HzUIColor){40, 160, 60, 230} : (HzUIColor){200, 40, 30, 230},
                    .corner_radius = ui_corner_radius(6)}) {
          cu_text(cu_numstr(k.fx_n, true), (u16)(w * 0.22f), (HzUIColor){255, 255, 255, 255});
        }
      }
    }
  }
  return clicked;
}

hz_internal b32 cu_button(HzUIElementId id, CString label, HzUIColor bg, b32 enabled, const InputSystem *in, u16 font) {
  b32 clicked = false;
  ui_element({.id = id,
              .layout = {.padding = {.left = 16, .right = 16, .top = 9, .bottom = 9}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
              .background_color = enabled ? bg : (HzUIColor){90, 94, 100, 255},
              .corner_radius = ui_corner_radius(9)}) {
    if (enabled && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    cu_text(label, font, (HzUIColor){255, 255, 255, enabled ? 255 : 150});
  }
  return clicked;
}

// ---------------------------------------------------------------- the battle board

hz_internal CardLook cu_look(const Battle *b, i32 ci, const GameSave *s, f32 w) {
  const BCard *c = &b->c[ci];
  return (CardLook){.id = c->id, .power = b_power(b, ci), .level = c->owner == B_SIDE_P ? s->card_level[c->id] : 0, .w = w,
                    .uid = (u32)ci, .fx_n = c->fx_t < 1.0f ? c->fx_n : 0};
}

hz_internal void battle_draw(CardUi *ui, Battle *b, const InputSystem *in, const GameSave *s, f32 sw, f32 shgt) {
  HzUIColor white = {255, 255, 255, 255}, dim = {200, 214, 205, 255};
  f32 cw = m_clampf(sw / 15.0f, 44.0f, 70.0f);   // board cards
  f32 hw = m_clampf(sw / 9.5f, 60.0f, 110.0f);   // hand cards
  ui->hover_card = -1;
  ui_element({.id = ui_id("Battle"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .padding = ui_padding_all(10), .child_gap = 8},
              .background_color = {24, 64, 44, 250}}) {
    // ---- the top line: opponent, turn, stakes ----
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 18, .child_alignment = {.y = UI_ALIGN_Y_CENTER},
                           .sizing = {.width = ui_sizing_grow(0)}}}) {
      cstr_format(ui->text[0], 96, "vs %", fmt_cstr(b->opponent));
      cu_text(ui->text[0], 20, white);
      cstr_format(ui->text[1], 96, "Turn %/%", fmt_i32(b->turn), fmt_i32(b->max_turns));
      cu_text(ui->text[1], 18, dim);
      cstr_format(ui->text[2], 96, "Stakes x%%", fmt_i32(b->stakes), fmt_cstr(b->p_snapped || b->ai_snapped ? "  SNAP!" : ""));
      cu_text(ui->text[2], 18, (HzUIColor){255, 210, 61, 255});
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
      cu_text(b->log_n ? b->log[0] : "", 14, dim);
    }
    // ---- the three zones ----
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
      for (i32 z = 0; z < 3; z++) {
        ZoneDef zd = zone_def(b->zone[z]);
        b32 playable = ui->selected >= 0 && b_can_play(b, ui->selected, z);
        ui_element({.id = ui_idi("Zone", (u32)z),
                    .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .padding = ui_padding_all(6),
                               .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                               .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                    .background_color = playable ? (HzUIColor){60, 120, 80, 255} : (HzUIColor){34, 82, 58, 255},
                    .border = {.color = playable ? (HzUIColor){255, 255, 255, 255} : (HzUIColor){20, 50, 34, 255},
                               .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                    .corner_radius = ui_corner_radius(10)}) {
          if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
            ui->click_zone = z;
          // the opponent's cards
          ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 4, .sizing = {.width = ui_sizing_grow(0),
                                                                                                       .height = ui_sizing_fixed(cw * 1.4f + 4.0f)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
            for (i32 i = 0; i < b->n; i++) {
              const BCard *c = &b->c[i];
              if (!c->alive || c->zone != z || c->owner != B_SIDE_AI)
                continue;
              CardLook k = cu_look(b, i, s, cw);
              k.hidden = !c->revealed;
              cu_card(ui, k, in, ui_idi("AiCard", (u32)i));
              if (ui_pointer_over(ui_idi("AiCard", (u32)i)) && c->revealed)
                ui->hover_card = i;
            }
          }
          // the zone: name, text, the two powers
          i32 ap = b_zone_power(b, z, B_SIDE_AI), pp = b_zone_power(b, z, B_SIDE_P);
          ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 3, .padding = ui_padding_all(6),
                                 .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                      .background_color = b->zone_rev[z] ? zd.color : (HzUIColor){70, 80, 90, 255},
                      .corner_radius = ui_corner_radius(8)}) {
            cu_text(cu_numstr(ap, false), 18, ap > pp ? (HzUIColor){255, 120, 100, 255} : white);
            cu_text(b->zone_rev[z] ? zd.name : "Unrevealed zone", 15, white);
            cu_text_wrap(b->zone_rev[z] ? zd.text : "It is revealed on a later turn.", 12, (HzUIColor){255, 255, 255, 220});
            cu_text(cu_numstr(pp, false), 18, pp > ap ? (HzUIColor){120, 255, 140, 255} : white);
          }
          // your cards
          ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 4, .sizing = {.width = ui_sizing_grow(0)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
            for (i32 i = 0; i < b->n; i++) {
              const BCard *c = &b->c[i];
              if (!c->alive || c->zone != z || c->owner != B_SIDE_P)
                continue;
              CardLook k = cu_look(b, i, s, cw);
              k.lit = c->act_order >= 0 || (b_can_activate(b, i) && b->phase == PHASE_PLAN);
              k.dim = c->staged;
              if (cu_card(ui, k, in, ui_idi("MyCard", (u32)i)))
                ui->click_card = i;
              if (ui_pointer_over(ui_idi("MyCard", (u32)i)))
                ui->hover_card = i;
            }
          }
        }
      }
    }
    // ---- the hand, the energy and the buttons ----
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .child_alignment = {.y = UI_ALIGN_Y_CENTER},
                           .sizing = {.width = ui_sizing_grow(0)}}}) {
      // energy: the big number left, "/turn" fsz (as in the three.js version)
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_alignment = {.y = UI_ALIGN_Y_BOTTOM}, .padding = ui_padding_all(8)},
                  .background_color = {0, 85, 191, 255},
                  .corner_radius = ui_corner_radius(30)}) {
        cu_text(cu_numstr(b->energy_left, false), 34, white);
        cstr_format(ui->text[1], 96, "/%", fmt_i32(b->turn));
        cu_text(ui->text[1], 14, (HzUIColor){200, 220, 255, 255});
      }
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .sizing = {.width = ui_sizing_grow(0)},
                             .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
        for (i32 h = 0; h < b->hand_n[B_SIDE_P]; h++) {
          i32 ci = b->hand[B_SIDE_P][h];
          CardLook k = cu_look(b, ci, s, hw);
          k.lit = ui->selected == ci;
          k.dim = card_def(b->c[ci].id).cost > b->energy_left;
          k.fx_n = 0;
          if (cu_card(ui, k, in, ui_idi("Hand", (u32)ci)))
            ui->click_card = ci;
          if (ui_pointer_over(ui_idi("Hand", (u32)ci)))
            ui->hover_card = ci;
        }
      }
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6}}) {
        b32 plan = b->phase == PHASE_PLAN;
        if (cu_button(ui_id("BtnEnd"), plan ? "End turn" : "Revealing\xe2\x80\xa6", (HzUIColor){0, 85, 191, 255}, plan, in, 18))
          ui->click_end = true;
        if (cu_button(ui_id("BtnSnap"), "SNAP", (HzUIColor){200, 60, 160, 255}, plan && !b->p_snapped, in, 15))
          ui->click_snap = true;
        if (cu_button(ui_id("BtnRetreat"), "Retreat", (HzUIColor){120, 60, 50, 255}, b->phase != PHASE_OVER, in, 14))
          ui->click_retreat = true;
      }
    }
  }
  // the card under the mouse, big, with its text
  if (ui->hover_card >= 0 && b->phase != PHASE_OVER) {
    i32 id = b->c[ui->hover_card].id;
    CardDef d = card_def(id);
    ui_element({.id = ui_id("CardTip"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                             .attach_points = {.element = UI_ATTACH_POINT_RIGHT_CENTER, .parent = UI_ATTACH_POINT_RIGHT_CENTER},
                             .offset = {-12.0f, -40.0f}, .z_index = 50},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .padding = ui_padding_all(10),
                           .sizing = {.width = ui_sizing_fixed(200)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = {16, 20, 26, 235},
                .corner_radius = ui_corner_radius(10)}) {
      CardLook k = cu_look(b, ui->hover_card, s, 150.0f);
      k.uid = 9000;
      k.fx_n = 0;
      cu_card(ui, k, in, ui_id("CardTipCard"));
      cu_text_wrap(d.text, 14, (HzUIColor){240, 240, 240, 255});
    }
  }
  // the end
  if (b->phase == PHASE_OVER) {
    CString title = b->result == RESULT_WIN ? "Victory!" : b->result == RESULT_LOSE ? "Defeat\xe2\x80\xa6" : b->result == RESULT_DRAW ? "Draw" : "You retreated";
    ui_element({.id = ui_id("BattleEnd"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 60},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = {0, 0, 0, 150}}) {
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .padding = ui_padding_all(24),
                             .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .background_color = {250, 250, 246, 255},
                  .corner_radius = ui_corner_radius(14)}) {
        cu_text(title, 36, b->result == RESULT_WIN ? (HzUIColor){30, 140, 60, 255} : (HzUIColor){40, 44, 52, 255});
        cstr_format(ui->text[2], 96, "Zones won: % - %   (stakes x%)", fmt_i32(b_winning_zones(b, B_SIDE_P)),
                    fmt_i32(b_winning_zones(b, B_SIDE_AI)), fmt_i32(b->stakes));
        cu_text(ui->text[2], 16, (HzUIColor){60, 64, 72, 255});
        if (cu_button(ui_id("BtnDone"), "Continue", (HzUIColor){0, 85, 191, 255}, true, in, 18))
          ui->click_done = true;
      }
    }
  }
  (void)shgt;
}

// main lane, before the scope: the clicks of the last UI frame on the battle (plus keys: Enter ends the turn)
hz_internal void battle_input(CardUi *ui, Battle *b, const InputSystem *in) {
  if (ui->click_card >= 0) {
    i32 ci = ui->click_card;
    BCard *c = &b->c[ci];
    if (c->owner == B_SIDE_P && c->zone < 0) // a hand card: pick it (again: drop it)
      ui->selected = ui->selected == ci ? -1 : ci;
    else if (c->owner == B_SIDE_P && c->staged)
      b_unstage(b, ci);
    else if (c->owner == B_SIDE_P && c->act_order >= 0)
      b_cancel_activate(b, ci);
    else if (c->owner == B_SIDE_P)
      b_activate(b, ci);
  } else if (ui->click_zone >= 0 && ui->selected >= 0) {
    if (b_stage(b, ui->selected, ui->click_zone))
      ui->selected = -1;
  }
  if (ui->click_end || in->buttons[KEY_ENTER].pressed_this_frame) {
    ui->selected = -1;
    b_end_turn(b);
  }
  if (ui->click_snap)
    b_snap(b);
  if (ui->click_retreat)
    b_retreat(b);
  ui->click_card = ui->click_zone = -1;
  ui->click_end = ui->click_snap = ui->click_retreat = false;
}

// ---------------------------------------------------------------- the deck and the collection

hz_internal void deck_draw(CardUi *ui, GameSave *s, const InputSystem *in, f32 sw, b32 voxel_walks) {
  HzUIColor ink = {27, 42, 52, 255}, white = {255, 255, 255, 255};
  f32 gw = m_clampf(sw / 12.0f, 52.0f, 84.0f);
  ui_element({.id = ui_id("DeckScreen"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .padding = ui_padding_all(12), .child_gap = 10},
              .background_color = {236, 232, 222, 250}}) {
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 16, .child_alignment = {.y = UI_ALIGN_Y_CENTER},
                           .sizing = {.width = ui_sizing_grow(0)}}}) {
      cu_text("Your cards", 24, ink);
      cstr_format(ui->text[0], 96, "% / % collected", fmt_u32(owned_count(s)), fmt_i32(CARD_POOL));
      cu_text(ui->text[0], 16, ink);
      cstr_format(ui->text[1], 96, "Deck % / %", fmt_i32(s->deck_count), fmt_i32(DECK_SIZE));
      cu_text(ui->text[1], 16, s->deck_count == DECK_SIZE ? (HzUIColor){30, 130, 60, 255} : (HzUIColor){200, 60, 40, 255});
      cstr_format(ui->text[2], 96, "% EXP", fmt_u32(s->exp));
      cu_text(ui->text[2], 18, (HzUIColor){120, 60, 200, 255});
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
      if (cu_button(ui_id("DeckClose"), "Close (C)", (HzUIColor){60, 64, 72, 255}, true, in, 15))
        ui->click_close = true;
    }
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 12, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}}}) {
      // the collection: every card, the ones you do not have as dark "???"
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                  .clip = {.vertical = true, .child_offset = ui_scroll_offset()}}) {
        cu_text("Collection (click a card)", 14, ink);
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 8, .sizing = {.width = ui_sizing_grow(0)}}}) {
          for (i32 c = 0; c < CARD_COUNT - 1; c++) {
            b32 have = owns(s, c);
            if (c == CARD_GOLDBRICK && !have)
              continue;
            b32 in_deck = false;
            for (i32 i = 0; i < s->deck_count; i++)
              in_deck |= s->deck[i] == c;
            CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = gw, .unknown = !have,
                          .lit = ui->deck_pick == c, .uid = 500u + (u32)c};
            if (cu_card(ui, k, in, ui_idi("Coll", (u32)c)) && have)
              ui->click_coll = c;
            (void)in_deck;
          }
        }
      }
      // the picked card big, with its stage and what you can do
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .padding = ui_padding_all(12),
                             .sizing = {.width = ui_sizing_fixed(m_minf(260.0f, sw * 0.32f))}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .background_color = {255, 255, 255, 255},
                  .corner_radius = ui_corner_radius(12)}) {
        i32 c = ui->deck_pick;
        if (c >= 0) {
          CardDef d = card_def(c);
          i32 lv = s->card_level[c];
          CardLook k = {.id = c, .power = d.power, .level = lv, .w = m_minf(170.0f, sw * 0.2f), .uid = 900u};
          cu_card(ui, k, in, ui_id("DeckBig"));
          cu_text_wrap(d.text, 14, ink);
          cstr_format(ui->text[3], 96, "Stage: % (%)", fmt_cstr(stage_def(lv).name), fmt_cstr(stage_def(lv).blurb));
          cu_text_wrap(ui->text[3], 13, (HzUIColor){90, 94, 100, 255});
          u32 cost = upgrade_cost(s, c);
          if (cost) {
            cstr_format(ui->text[4], 96, "Upgrade to % (% EXP)", fmt_cstr(stage_def(lv + 1).name), fmt_u32(cost));
            if (cu_button(ui_id("BtnUpgrade"), ui->text[4], (HzUIColor){120, 60, 200, 255}, s->exp >= cost, in, 14))
              ui->click_upgrade = true;
          } else if (!can_become_voxel(c)) {
            cu_text_wrap("Final stage. Only 6-energy cards become a Voxelmon.", 12, (HzUIColor){90, 94, 100, 255});
          }
          if (lv >= VOXEL_LEVEL) {
            b32 walking = s->voxelmon == c + 1;
            if (cu_button(ui_id("BtnWalk"), walking ? "Walks with you" : "Walk with me", (HzUIColor){60, 160, 80, 255}, !walking, in, 14))
              ui->click_walk = true;
          }
          b32 in_deck = false;
          for (i32 i = 0; i < s->deck_count; i++)
            in_deck |= s->deck[i] == c;
          if (in_deck) {
            if (cu_button(ui_id("BtnRemove"), "Remove from deck", (HzUIColor){180, 70, 50, 255}, true, in, 14))
              ui->click_remove = true;
          } else if (cu_button(ui_id("BtnAdd"), "Add to deck", (HzUIColor){0, 85, 191, 255}, s->deck_count < DECK_SIZE, in, 14)) {
            ui->click_add = true;
          }
        } else {
          cu_text_wrap("Pick a card on the left to see it big, upgrade it with EXP or change your deck.", 14, ink);
        }
        (void)voxel_walks;
      }
    }
    // the deck: 12 slots
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .sizing = {.width = ui_sizing_grow(0)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = {40, 44, 52, 255},
                .corner_radius = ui_corner_radius(10)}) {
      for (i32 i = 0; i < DECK_SIZE; i++) {
        if (i < s->deck_count) {
          i32 c = s->deck[i];
          CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = gw * 0.82f, .uid = 700u + (u32)i};
          if (cu_card(ui, k, in, ui_idi("Slot", (u32)i)))
            ui->click_slot = i;
        } else {
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(gw * 0.82f), .height = ui_sizing_fixed(gw * 0.82f * 1.4f)}},
                      .background_color = {70, 74, 82, 255},
                      .corner_radius = ui_corner_radius(6)}) {}
        }
      }
    }
  }
  (void)white;
}

// the clicks of the deck screen; returns true when it was closed
hz_internal b32 deck_input(CardUi *ui, GameSave *s, SaveState *save, SoundBank *snd, const InputSystem *in) {
  b32 closed = false;
  if (ui->click_coll >= 0)
    ui->deck_pick = ui->click_coll;
  if (ui->click_slot >= 0)
    ui->deck_pick = s->deck[ui->click_slot];
  i32 c = ui->deck_pick;
  if (c >= 0 && ui->click_add && s->deck_count < DECK_SIZE) {
    s->deck[s->deck_count++] = (u8)c;
    save_mark(save);
  }
  if (c >= 0 && ui->click_remove) {
    i32 k = 0;
    for (i32 i = 0; i < s->deck_count; i++)
      if (s->deck[i] != c)
        s->deck[k++] = s->deck[i];
    s->deck_count = (u8)k;
    save_mark(save);
  }
  if (c >= 0 && ui->click_upgrade && upgrade(s, c)) {
    sound_play(snd, s->card_level[c] == VOXEL_LEVEL ? SND_FANFARE : SND_COIN, 1.0f, 0.8f);
    save_mark(save);
  }
  if (c >= 0 && ui->click_walk && s->card_level[c] >= VOXEL_LEVEL) {
    s->voxelmon = c + 1;
    save_mark(save);
  }
  if (ui->click_close || in->buttons[KEY_C].pressed_this_frame || in->buttons[KEY_ESCAPE].pressed_this_frame)
    closed = true;
  ui->click_coll = ui->click_slot = -1;
  ui->click_add = ui->click_remove = ui->click_upgrade = ui->click_walk = ui->click_close = false;
  return closed;
}

// ---------------------------------------------------------------- rewards

hz_internal void rewards_draw(CardUi *ui, const GameSave *s, const InputSystem *in, f32 sw) {
  if (!ui->reward_n)
    return;
  ui_element({.id = ui_id("Rewards"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 70},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = {0, 0, 0, 170}}) {
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 14, .padding = ui_padding_all(22),
                           .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = {250, 250, 246, 255},
                .corner_radius = ui_corner_radius(14)}) {
      cu_text_wrap(ui->reward_title, 22, (HzUIColor){27, 42, 52, 255});
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 12}}) {
        f32 w = m_clampf(sw / 6.0f, 90.0f, 150.0f);
        for (i32 i = 0; i < ui->reward_n && i < 3; i++) {
          i32 c = ui->reward[i];
          CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = w, .uid = 800u + (u32)i};
          cu_card(ui, k, in, ui_idi("Reward", (u32)i));
        }
      }
      if (cu_button(ui_id("RewardOk"), "Great!", (HzUIColor){0, 85, 191, 255}, true, in, 18))
        ui->click_reward_ok = true;
    }
  }
}
