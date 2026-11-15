// The card screens: one card widget (the printed card of the three.js Card.svelte: white edge, the ability's colour,
// the art window, the name banner, cost and power badges, the upgrade stage's metal frame), the battle board
// (BattleView.svelte: background picture, three locations with their pictures, energy orb, deck pile, hand, drag and
// drop, the big card on tap, attack effects, confetti), the deck and collection and the rewards. Clicks are read while
// the UI is built and acted on the next frame. Included by starting_template.c above app_init.

#include "lib/image.h"
#include "st_battle_images.c"

typedef enum InspectKind { INSPECT_NONE, INSPECT_HAND, INSPECT_BOARD, INSPECT_ZONE } InspectKind;

typedef struct CardUi {
  CardArt art;
  f32 time; // the UI clock (glows, holo, confetti)
  f32 dpr;
  i16 z;    // z index of the card widgets' floating parts in the current layer
  // battle
  i32 selected;     // the hand card picked to play (-1)
  i32 hover_card;   // battle card index under the mouse
  i32 click_card, click_zone;
  b32 click_end, click_snap, click_retreat, click_done, click_undo, click_log;
  i32 press_card;   // a card the pointer went down on this frame (drag start)
  // the big card / location on tap
  InspectKind inspect;
  i32 inspect_i;
  i32 click_play_to; // "Play to:" zone (-1)
  b32 click_take_back, click_activate, click_cancel_act, click_close_insp, click_insp_bg;
  b32 log_open;
  // drag and drop
  i32 drag;          // the dragged card (-1)
  b32 drag_moved, drag_from_hand;
  f32 drag_x0, drag_y0, drag_dx, drag_dy, drag_tilt, drag_lastx;
  i32 drag_over;     // 0..2 a location, 3 the hand, -1 nothing
  // where every battle card was last frame (projectiles, shatter)
  Clay_BoundingBox box[B_MAX_CARDS];
  b32 has_box[B_MAX_CARDS];
  f32 confetti_t;    // seconds since the win (confetti), < 0 none
  // pictures
  GpuTexture zone_tex[ZONE_COUNT], back_tex[3];
  u32 zone_w[ZONE_COUNT], zone_h[ZONE_COUNT];
  b32 zone_made[ZONE_COUNT], back_made[3];
  // deck screen
  b32 deck_open;
  i32 deck_pick;    // the card shown big
  i32 click_coll, click_slot;
  b32 click_add, click_remove, click_upgrade, click_walk, click_close, click_unpick;
  // the upgrade moment (up_draw): the card, the stage before and after, seconds since it began
  i32 up_card; // -1 none
  i32 up_from, up_to;
  f32 up_t;
  b32 click_up_ok;
  // rewards
  i32 reward[3], reward_n;
  CString reward_title;
  b32 click_reward_ok;
  f32 reward_t;        // since the packs were shown (they are dealt in)
  f32 reward_tear[3];  // since each pack was torn open (PACK_SEALED: not yet; < 0: about to, "Reveal all")
  i32 click_pack;      // a pack tapped (-1)
  b32 click_reveal_all;
  char text[10][96];
  char sbuf[40][64]; // this frame's formatted strings (the UI reads them after the frame is built)
  i32 sbuf_n;
} CardUi;

hz_internal HzUIColor cu_rgb(u32 hex) { return (HzUIColor){(f32)((hex >> 16) & 255), (f32)((hex >> 8) & 255), (f32)(hex & 255), 255}; }
hz_internal HzUIColor cu_rgba(u32 hex, f32 a) {
  HzUIColor c = cu_rgb(hex);
  c.a = 255.0f * m_clampf(a, 0.0f, 1.0f);
  return c;
}
// a dark see-through colour as css shows it: the engine blends in linear light, where the same alpha darkens far less
hz_internal HzUIColor cu_shade(u32 hex, f32 a) { return cu_rgba(hex, 1.0f - m_powf(1.0f - m_clampf(a, 0.0f, 1.0f), 2.2f)); }
hz_internal HzUIColor cu_black(f32 a) { return cu_shade(0, a); }
hz_internal HzUIColor cu_alpha(HzUIColor c, f32 a) {
  c.a = 255.0f * m_clampf(a, 0.0f, 1.0f);
  return c;
}
// css color-mix(in srgb, a (1-t), b t)
hz_internal HzUIColor cu_mix(HzUIColor a, HzUIColor b, f32 t) {
  return (HzUIColor){a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t};
}
hz_internal b32 cu_light(HzUIColor c) { return (0.299f * c.r + 0.587f * c.g + 0.114f * c.b) / 255.0f > 0.62f; }
hz_internal HzUIColor cu_ink(HzUIColor bg) { return cu_light(bg) ? cu_rgb(0x1b2a34) : cu_rgb(0xffffff); }

hz_internal UIGradient cu_lin(f32 angle, u32 n, const f32 *t, const HzUIColor *c) {
  UIGradient g = {.kind = UI_GRADIENT_LINEAR, .angle = angle, .stop_count = n};
  for (u32 i = 0; i < n && i < UI_GRADIENT_MAX_STOPS; i++)
    g.stops[i] = (HzUIGradientStop){t[i], c[i]};
  return g;
}
hz_internal UIGradient cu_lin2(f32 angle, HzUIColor a, HzUIColor b) {
  return cu_lin(angle, 2, (f32[]){0.0f, 1.0f}, (HzUIColor[]){a, b});
}
// radial-gradient(circle at x y, a, b t1) farthest corner
hz_internal UIGradient cu_rad(f32 x, f32 y, HzUIColor a, HzUIColor b, f32 t1) {
  UIGradient g = {.kind = UI_GRADIENT_RADIAL, .stop_count = 2, .circle = true, .size = UI_RADIAL_FARTHEST_CORNER,
                  .at = opt_some(vec2, ((vec2){x, y}))};
  g.stops[0] = (HzUIGradientStop){0.0f, a};
  g.stops[1] = (HzUIGradientStop){t1, b};
  return g;
}

hz_internal Optional(HzUITransform) cu_tf(f32 tx, f32 ty, f32 sc, f32 rot) {
  if (tx == 0.0f && ty == 0.0f && sc == 1.0f && rot == 0.0f)
    return opt_none(HzUITransform);
  HzUITransform t = {.scale = {sc, sc}, .translate = {tx, ty}, .rotation = rot, .origin = {0.5f, 0.5f}};
  return opt_some(HzUITransform, t);
}

hz_internal f32 cu_ease_out(f32 x) { return 1.0f - (1.0f - x) * (1.0f - x); }
hz_internal f32 cu_hash(u32 i) { // 0..1, fixed per index
  i = (i ^ 61u) ^ (i >> 16);
  i *= 9u;
  i ^= i >> 4;
  i *= 0x27d4eb2du;
  i ^= i >> 15;
  return (f32)(i & 0xffffu) / 65535.0f;
}

hz_internal void cu_text(CString s, u16 size, HzUIColor c) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size, .textColor = c, .wrapMode = UI_TEXT_WRAP_NONE}));
}
hz_internal void cu_text_wrap(CString s, u16 size, HzUIColor c) {
  ui_text(ui_str_c(s), ui_text_config({.fontSize = size, .textColor = c, .wrapMode = UI_TEXT_WRAP_WORDS}));
}
// bold text (weight 700..900), optionally centred and wrapped, with an optional shadow
hz_internal void cu_textb(CString s, f32 size, HzUIColor c, u16 weight, b32 wrap, HzUIColor shadow) {
  HzUITextConfig cfg = {.fontSize = m_maxf(6.0f, size), .textColor = c, .weight = weight,
                        .wrapMode = wrap ? UI_TEXT_WRAP_WORDS : UI_TEXT_WRAP_NONE, .textAlignment = CLAY_TEXT_ALIGN_CENTER};
  if (shadow.a > 0.0f)
    cfg.shadows[0] = (HzUITextShadow){.color = shadow, .offset_y = 1.0f, .blur = 2.0f};
  ui_text(ui_str_c(s), cfg);
}

hz_internal CString cu_fmt_slot(CardUi *ui) {
  char *b = ui->sbuf[ui->sbuf_n % 40];
  ui->sbuf_n++;
  return b;
}
hz_internal CString cu_fmt_args(CardUi *ui, CString fmt, const FmtArgs *a) {
  char *buf = (char *)cu_fmt_slot(ui);
  fmt_cstr_to_buffer(buf, 64, fmt, a);
  return buf;
}
// a formatted string that lives until the end of the frame
#define CU_FMT(ui, fmt, ...)                                                                                                       cu_fmt_args((ui), (fmt), &(FmtArgs){.args = (FmtArg[]){__VA_ARGS__}, .args_size = sizeof((FmtArg[]){__VA_ARGS__}) / sizeof(FmtArg)})

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

// ---- the pictures: decoded from st_battle_images.c the first time they are drawn ----
hz_internal GpuTexture cu_decode(const u8 *data, u32 len, u32 up, u32 *w, u32 *h) {
  ImageInfo info = {0};
  ImageDecodeDesc desc = {0};
  u8 *px = image_decode_rgba8(data, len, &desc, &info);
  GpuTexture tex = {0};
  if (!px) {
    log_info("battle picture: %", fmt_cstr(info.error ? info.error : "decode failed"));
    return tex;
  }
  u32 W = info.width * up, H = info.height * up;
  if (up == 1) {
    tex = gpu_make_texture(&(GpuTextureDesc){.data = px, .size = W * H * 4u, .data_type = GPU_TEXTURE_DATA_RGBA8, .width = W,
                                             .height = H, .is_srgb = true});
  } else { // pixel art: each pixel a crisp block (the UI samples smoothly)
    tctx_temp_allocator_scope(ta, NULL) {
      u32 *dst = ALLOC_ARRAY(&ta.allocator, u32, W * H);
      const u32 *src = (const u32 *)px;
      for (u32 y = 0; y < H; y++)
        for (u32 x = 0; x < W; x++)
          dst[y * W + x] = src[(y / up) * info.width + x / up];
      tex = gpu_make_texture(&(GpuTextureDesc){.data = (u8 *)dst, .size = W * H * 4u, .data_type = GPU_TEXTURE_DATA_RGBA8,
                                               .width = W, .height = H, .is_srgb = true});
    }
  }
  *w = info.width;
  *h = info.height;
  image_free(px, &info);
  return tex;
}
hz_internal GpuTexture cu_zone_tex(CardUi *ui, i32 z) {
  if (!ui->zone_made[z]) {
    ui->zone_made[z] = true;
    ui->zone_tex[z] = cu_decode(ZONE_IMAGES[z].data, ZONE_IMAGES[z].len, 1, &ui->zone_w[z], &ui->zone_h[z]);
  }
  return ui->zone_tex[z];
}
hz_internal GpuTexture cu_back_tex(CardUi *ui, i32 kind) {
  if (!ui->back_made[kind]) {
    ui->back_made[kind] = true;
    u32 w, h;
    ui->back_tex[kind] = cu_decode(BACK_IMAGES[kind].data, BACK_IMAGES[kind].len, 3, &w, &h);
  }
  return ui->back_tex[kind];
}

// a picture covering a box (css object-fit: cover), as a floating child clipped to its parent
hz_internal void cu_cover(GpuTexture tex, f32 iw, f32 ih, f32 bw, f32 bh, i16 z) {
  if (!handle_is_valid(tex) || iw <= 0.0f || ih <= 0.0f)
    return;
  f32 s = m_maxf(bw / iw, bh / ih);
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .clip_to = UI_CLIP_TO_ATTACHED_PARENT, .z_index = z,
                           .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                           .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_CENTER_CENTER}},
              .layout = {.sizing = {.width = ui_sizing_fixed(iw * s), .height = ui_sizing_fixed(ih * s)}},
              .image = {.texture = tex},
              .background_color = {255, 255, 255, 255}}) {}
}

// the ability symbol (the three.js version uses Material icons; these are glyphs the UI font has)
hz_internal CString cu_kind_glyph(CardKind k) {
  switch (k) {
  case KIND_REVEAL: return "!";
  case KIND_ONGOING: return "\xe2\x88\x9e"; // infinity
  case KIND_ACTIVATE: return "\xe2\x96\xb6";  // play triangle
  case KIND_ENDTURN: return "\xe2\x86\xbb";   // clockwise arrow
  case KIND_START: return "\xe2\x98\x85";     // star
  default: return "";
  }
}

// a small round badge with a number (deck screen etc.)
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
  i32 power;      // shown power (may differ from the card's)
  i32 level;      // upgrade stage: 1 bronze, 2 silver, 3 gold + holo, 4 Voxelmon
  f32 w;          // width in px (height = 1.4 w)
  b32 lit;        // selected: lifted with a yellow glow
  b32 dim;        // can not be played now: greyed
  b32 unknown;    // not collected yet: a dark silhouette, "???"
  b32 hidden;     // face down: the card back
  u32 uid;        // for the element ids
  i32 fx_n;       // the last power change (+N / -N), 0 none
  f32 fx_t;       // seconds since it
  f32 fx_delay;   // the projectile's travel time before the change shows
  f32 pulse_t;    // seconds since it was revealed (it turns over)
  b32 hide_cost;  // cards on the table do not show their cost
  b32 staged;     // played this turn, not revealed yet: dashed outline
  b32 ready;      // an Activate that can be used now: green glow
  b32 aura;       // an Ongoing card in play: blue rings
  b32 big;        // the detail card: art on top, the kind and the text below
  f32 tx, ty, rot, scale; // extra placement (drag, lunge, draw)
  f32 opacity;    // 0 = opaque
  f32 glow;       // extra yellow glow 0..1 (drawn card)
} CardLook;

hz_internal void cu_card_back(f32 w, f32 h, f32 em, i16 z) {
  // cream edge, red lattice, a gold frame line and the medallion
  ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgb(0xa3140a),
              .backgrounds = ui_backgrounds(
                  {.gradient = cu_lin(45.0f, 4, (f32[]){0.0f, 0.44f, 0.5f, 0.56f},
                                      (HzUIColor[]){cu_rgba(0xe8604c, 0), cu_rgba(0xe8604c, 0), cu_rgba(0xe8604c, 1), cu_rgba(0xe8604c, 0)}),
                   .size = {1.3f * em, 1.3f * em}},
                  {.gradient = cu_lin(-45.0f, 4, (f32[]){0.0f, 0.44f, 0.5f, 0.56f},
                                      (HzUIColor[]){cu_rgba(0xe8604c, 0), cu_rgba(0xe8604c, 0), cu_rgba(0xe8604c, 1), cu_rgba(0xe8604c, 0)}),
                   .size = {1.3f * em, 1.3f * em}},
                  {.gradient = cu_rad(0.5f, 0.5f, cu_rgba(0xffffff, 0.18f), cu_rgba(0xffffff, 0), 0.62f)}),
              .corner_radius = ui_corner_radius(0.38f * em)}) {
    // the gold double frame line
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {0.3f * em, 0.3f * em},
                             .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(w - 1.24f * em), .height = ui_sizing_fixed(h - 1.24f * em)}},
                .border = {.color = cu_rgb(0xf2cd37), .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
                .corner_radius = ui_corner_radius(0.28f * em)}) {}
    // the medallion: a little brick and the name
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(0.25f * em),
                           .padding = {.left = (u16)(0.5f * em), .right = (u16)(0.5f * em), .top = (u16)(0.45f * em), .bottom = (u16)(0.45f * em)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = cu_rgb(0x8a1006),
                .border = {.color = cu_rgb(0xf2cd37), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                .corner_radius = ui_corner_radius(0.9f * em),
                .shadow = {.color = cu_black(0.35f), .offset_y = 0.1f * em, .blur = 0.3f * em}}) {
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(0.3f * em),
                             .padding = {.left = (u16)(0.2f * em), .right = (u16)(0.2f * em), .top = (u16)(0.25f * em), .bottom = (u16)(0.25f * em)}},
                  .background_color = cu_rgb(0xf2cd37),
                  .corner_radius = ui_corner_radius(0.15f * em)}) {
        for (i32 i = 0; i < 2; i++)
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(0.55f * em), .height = ui_sizing_fixed(0.4f * em)}},
                      .background_color = cu_rgb(0xffe27a), .corner_radius = ui_corner_radius(0.2f * em)}) {}
      }
      if (w >= 48.0f)
        cu_textb("BRICK\nBEASTS", 0.95f * em, cu_rgb(0xf2cd37), 900, false, (HzUIColor){0, 0, 0, 0});
    }
  }
}

// a card; returns true when it was clicked this frame
hz_internal b32 cu_card(CardUi *ui, CardLook k, const InputSystem *in, HzUIElementId id) {
  b32 clicked = false;
  f32 w = k.w, h = w * 1.4f, em = w / 10.0f;
  i16 z = ui->z;
  CardDef d = card_def(k.id);
  KindInfo ki = kind_info(d.kind);
  HzUIColor t = ki.color, td = ki.dark;
  if (k.unknown) {
    t = cu_rgb(0x7a7f86);
    td = cu_rgb(0x3c4046);
  }
  // ---- motion: selection, the turn-over when revealed, the hit shake / buff bounce ----
  f32 sc = k.scale > 0.0f ? k.scale : 1.0f, tx = k.tx, ty = k.ty, rot = k.rot;
  if (k.lit) {
    ty -= 14.0f;
    sc *= 1.06f;
  }
  f32 ft = k.fx_t - k.fx_delay;
  b32 fx_on = k.fx_n != 0 && ft >= 0.0f && ft < 1.0f && !k.hidden;
  if (fx_on && k.fx_n < 0 && ft < 0.5f) {
    f32 q = ft / 0.5f, a = (1.0f - q);
    tx += 0.5f * em * m_sinf(q * 4.0f * M_PI) * a;
    rot += 4.0f * m_sinf(q * 4.0f * M_PI) * a;
  }
  if (fx_on && k.fx_n > 0 && ft < 0.55f) {
    f32 q = ft / 0.55f;
    f32 up = q < 0.35f ? q / 0.35f : m_maxf(0.0f, 1.0f - (q - 0.35f) / 0.35f);
    sc *= q < 0.35f ? 1.0f + 0.18f * q / 0.35f : q < 0.7f ? 1.18f - 0.21f * (q - 0.35f) / 0.35f : 0.97f + 0.03f * (q - 0.7f) / 0.3f;
    ty -= 0.6f * em * up;
  }
  f32 ry = 0.0f;
  if (k.pulse_t > 0.0f && k.pulse_t < 0.45f && !k.hidden) {
    f32 q = k.pulse_t / 0.45f;
    ry = q < 0.6f ? 80.0f * (1.0f - q / 0.6f) : 0.0f;
    sc *= q < 0.6f ? 1.25f - 0.15f * q / 0.6f : 1.1f - 0.1f * (q - 0.6f) / 0.4f;
  }
  Optional(HzUITransform) tf = cu_tf(tx, ty, sc, rot);
  if (ry != 0.0f) {
    HzUITransform tt = {.scale = {sc, sc}, .translate = {tx, ty}, .rotation = rot, .origin = {0.5f, 0.5f}, .perspective = 500.0f,
                        .rotate_y = ry};
    tf = opt_some(HzUITransform, tt);
  }
  // ---- the frame: white printed edge, or the stage's metal ----
  f32 bw = (k.level >= 3 ? 0.26f : k.level == 2 ? 0.24f : k.level == 1 ? 0.22f : 0.16f) * em;
  if (k.hidden)
    bw = 0.32f * em;
  UIGradient metal = {0};
  if (!k.hidden && k.level == 1)
    metal = cu_lin(135.0f, 4, (f32[]){0.0f, 0.3f, 0.52f, 1.0f}, (HzUIColor[]){cu_rgb(0xf3c58f), cu_rgb(0xa8642a), cu_rgb(0xffdcae), cu_rgb(0x8a4e1c)});
  else if (!k.hidden && k.level == 2)
    metal = cu_lin(135.0f, 4, (f32[]){0.0f, 0.28f, 0.5f, 1.0f}, (HzUIColor[]){cu_rgb(0xffffff), cu_rgb(0x9aa6b2), cu_rgb(0xf4f8fb), cu_rgb(0x7d8995)});
  else if (!k.hidden && k.level >= 3)
    metal = cu_lin(135.0f, 4, (f32[]){0.0f, 0.26f, 0.48f, 1.0f}, (HzUIColor[]){cu_rgb(0xfff3a8), cu_rgb(0xd9a300), cu_rgb(0xfffbe0), cu_rgb(0xb07d00)});
  HzUIShadow glow = {0};
  f32 pulse = 0.5f + 0.5f * m_sinf(ui->time * 2.0f * M_PI / 2.4f);
  if (k.lit)
    glow = (HzUIShadow){.color = cu_rgb(0xffd23d), .blur = 18.0f, .spread = 6.0f};
  else if (k.ready)
    glow = (HzUIShadow){.color = cu_rgb(0x4fc06a), .blur = 14.0f + 8.0f * pulse, .spread = 4.0f + 4.0f * pulse};
  else if (k.glow > 0.0f)
    glow = (HzUIShadow){.color = cu_rgba(0xffe066, k.glow), .blur = 14.0f};
  else if (k.aura)
    glow = (HzUIShadow){.color = cu_rgba(0x3d9bff, 0.7f + 0.25f * pulse), .blur = (0.6f + 0.6f * pulse) * em, .spread = (0.2f + 0.25f * pulse) * em};
  else if (!k.hidden && k.level >= 4)
    glow = (HzUIShadow){.color = cu_rgba(0x78dcff, 0.85f * pulse), .blur = 1.3f * em * pulse};
  else if (!k.hidden && k.level == 3)
    glow = (HzUIShadow){.color = cu_rgba(0xffd23d, 0.55f), .blur = 0.7f * em};
  f32 op = k.opacity > 0.0f ? k.opacity : 1.0f;
  if (k.dim)
    op *= 0.45f;
  if (k.staged)
    op *= 0.75f;
  ui_element({.id = id,
              .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}, .padding = ui_padding_all((u16)m_maxf(1.0f, bw))},
              .background_color = k.hidden ? cu_rgb(0xfbf6ea) : cu_rgb(0xffffff),
              .gradient = metal,
              .corner_radius = ui_corner_radius(0.6f * em),
              .shadow = {.color = cu_black(0.35f), .offset_y = 0.25f * em, .blur = 0.6f * em},
              .shadows = {glow},
              .outline = {.width = k.lit ? 3.0f : 0.0f, .color = {255, 255, 255, 255}},
              .transform = tf,
              .opacity = op < 1.0f ? opt_some(f32, op) : opt_none(f32),
              .brightness = k.dim ? opt_some(f32, 0.8f) : opt_none(f32)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    if (k.hidden) {
      cu_card_back(w, h, em, z);
    } else {
      // the colour of the ability, light at the top left
      HzUIColor t_light = cu_mix(t, cu_rgb(0xffffff), 0.3f);
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}, .padding = ui_padding_all((u16)(0.3f * em))},
                  .gradient = cu_lin(150.0f, 3, (f32[]){0.0f, 0.38f, 1.0f}, (HzUIColor[]){t_light, t, td}),
                  .corner_radius = ui_corner_radius(0.45f * em)}) {
        f32 inner_w = w - 2.0f * bw - 0.6f * em;
        ui_element({.id = ui_idi("CardIn", k.uid),
                    .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                               .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                    .gradient = cu_lin2(180.0f, cu_rgb(0xfffdf7), cu_mix(cu_rgb(0xfffaf0), t, 0.18f)),
                    .corner_radius = ui_corner_radius(0.38f * em),
                    .clip = {.horizontal = true, .vertical = true}}) {
          // the art window: the beast on a little scene in the ability's colour
          f32 art_h = k.big ? inner_w * 5.0f / 6.0f : 0.0f;
          HzUIColor sky = k.unknown ? cu_rgb(0x3a3e44) : cu_mix(t, cu_rgb(0xffffff), 0.55f);
          HzUIColor ground = k.unknown ? cu_rgb(0x2a2d32) : cu_mix(t, td, 0.35f);
          ui_element({.id = ui_idi("CardArt", k.uid),
                      .layout = {.sizing = {.width = ui_sizing_grow(0), .height = k.big ? ui_sizing_fixed(art_h) : ui_sizing_grow(0)},
                                 .padding = {.bottom = (u16)(0.7f * em), .top = (u16)(0.6f * em)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                      .gradient = cu_lin(180.0f, 4, (f32[]){0.0f, 0.62f, 0.66f, 1.0f}, (HzUIColor[]){sky, cu_mix(sky, t, 0.6f), ground, cu_mix(ground, td, 0.5f)}),
                      .corner_radius = {.top_left = 0.3f * em, .top_right = 0.3f * em, .bottom_left = 0.15f * em, .bottom_right = 0.15f * em},
                      .shadow = {.color = cu_black(0.18f), .offset_y = 0.12f * em}}) {
            u32 pw = 0, ph = 0;
            GpuTexture tex = card_art(&ui->art, k.id, &pw, &ph);
            if (pw && ph && handle_is_valid(tex)) {
              f32 iw = inner_w * 0.86f, ih = (k.big ? art_h : (h - 2.0f * bw - 0.6f * em) * 0.62f) * 0.86f;
              f32 s = m_minf(iw / (f32)pw, ih / (f32)ph);
              ui_element({.layout = {.sizing = {.width = ui_sizing_fixed((f32)pw * s), .height = ui_sizing_fixed((f32)ph * s)}},
                          .image = {.texture = tex},
                          .background_color = k.unknown ? (HzUIColor){0, 0, 0, 255} : (HzUIColor){255, 255, 255, 255}}) {}
            }
            // gold: a rainbow holo band sweeping over the picture
            if (k.level >= 3) {
              f32 p = m_fmodf(ui->time / 3.6f, 1.0f) * 1.6f - 0.3f;
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                          .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                          .gradient = cu_lin(115.0f, 4, (f32[]){p - 0.14f, p - 0.04f, p + 0.05f, p + 0.15f},
                                             (HzUIColor[]){cu_rgba(0xff6ad5, 0), cu_rgba(0xffd23d, 0.4f), cu_rgba(0x3dbbff, 0.4f), cu_rgba(0xb06aff, 0)})}) {}
            }
            // the name on a banner overlapping the bottom of the art window
            ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {0.0f, 0.16f * em},
                                     .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                                     .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_CENTER_BOTTOM}},
                        .layout = {.sizing = {.width = ui_sizing_fixed(inner_w - 0.6f * em), .height = ui_sizing_fixed(1.5f * em)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                        .gradient = cu_lin2(180.0f, cu_rgb(0xffffff), cu_rgb(0xf1ebdc)),
                        .corner_radius = ui_corner_radius(0.35f * em),
                        .clip = {.horizontal = true},
                        .outline = {.width = m_maxf(1.0f, 0.07f * em), .color = cu_mix(td, cu_rgb(0x000000), 0.3f)},
                        .shadow = {.color = cu_black(0.30f), .offset_y = 0.1f * em, .blur = 0.2f * em}}) {
              cu_textb(k.unknown ? "???" : d.name, (w <= 96.0f && !k.big ? 1.2f : 1.05f) * em, cu_rgb(0x1b2a34), 800, false, (HzUIColor){0});
            }
          }
          if (k.big) {
            ui_element({.layout = {.sizing = {.height = ui_sizing_fixed(1.3f * em)}}}) {}
            ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(0.25f * em),
                                   .padding = {.left = (u16)(0.6f * em), .right = (u16)(0.6f * em), .top = 1, .bottom = 1},
                                   .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                        .background_color = t,
                        .corner_radius = ui_corner_radius(em)}) {
              cu_textb(cu_kind_glyph(d.kind), 0.85f * em, cu_ink(t), 800, false, (HzUIColor){0});
              cu_textb(ki.label, 0.75f * em, cu_ink(t), 700, false, (HzUIColor){0});
            }
            ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(inner_w)}, .padding = {.left = (u16)(0.5f * em), .right = (u16)(0.5f * em), .top = (u16)(0.3f * em)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
              // long texts get smaller (the three.js fitText)
              u32 len = (u32)cstr_len(d.text);
              f32 fs = len > 90 ? 0.6f : len > 60 ? 0.68f : 0.78f;
              cu_textb(k.unknown ? "" : d.text, fs * em, cu_rgb(0x333333), 500, true, (HzUIColor){0});
            }
          } else {
            // under the name: the ability symbol (its text on hover), or the same space empty
            ui_element({.layout = {.sizing = {.height = ui_sizing_fixed(0.95f * em)}}}) {}
            ui_element({.layout = {.sizing = {.height = ui_sizing_fixed(1.95f * em)}, .padding = {.bottom = (u16)(0.3f * em)}}}) {
              if (d.kind != KIND_NONE && !k.unknown) {
                ui_element({.id = ui_idi("CardBadge", k.uid),
                            .layout = {.sizing = {.width = ui_sizing_fixed(1.65f * em), .height = ui_sizing_fixed(1.65f * em)},
                                       .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                            .background_color = t,
                            .border = {.color = {255, 255, 255, 217}, .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
                            .corner_radius = ui_corner_radius(0.85f * em),
                            .shadow = {.color = cu_black(0.35f), .offset_y = 0.1f * em, .blur = 0.3f * em}}) {
                  cu_textb(cu_kind_glyph(d.kind), 1.0f * em, cu_ink(t), 800, false, (HzUIColor){0});
                }
              }
            }
          }
          // cost (round, blue) top left, power (rounded square, orange / green raised / red lowered) top right
          f32 bd = 2.15f * em;
          if (!k.hide_cost) {
            ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {0.15f * em, 0.15f * em},
                                     .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                        .layout = {.sizing = {.width = ui_sizing_fixed(bd), .height = ui_sizing_fixed(bd)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                        .gradient = cu_rad(0.35f, 0.3f, cu_rgb(0x3f86e0), cu_rgb(0x0a3478), 0.75f),
                        .border = {.color = {255, 255, 255, 255}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                        .corner_radius = ui_corner_radius(bd * 0.5f),
                        .shadow = {.color = cu_black(0.40f), .offset_y = 0.12f * em, .blur = 0.25f * em}}) {
              cu_textb(cu_numstr(d.cost, false), 1.15f * em, cu_rgb(0xffffff), 800, false, cu_black(0.5f));
            }
          }
          i32 diff = k.power - d.power;
          UIGradient pg = diff > 0 ? cu_rad(0.35f, 0.3f, cu_rgb(0xb6ff8f), cu_rgb(0x1f9a12), 1.0f)
                          : diff < 0 ? cu_rad(0.35f, 0.3f, cu_rgb(0xff9a9a), cu_rgb(0xc01010), 1.0f)
                                     : cu_rad(0.35f, 0.3f, cu_rgb(0xffc08a), cu_rgb(0xd9531b), 0.75f);
          ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {-0.15f * em, 0.15f * em},
                                   .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                                   .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP}},
                      .layout = {.sizing = {.width = ui_sizing_fixed(bd), .height = ui_sizing_fixed(bd)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                      .gradient = pg,
                      .border = {.color = {255, 255, 255, 255}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                      .corner_radius = ui_corner_radius(0.46f * em),
                      .shadow = {.color = cu_black(0.40f), .offset_y = 0.12f * em, .blur = 0.25f * em},
                      .shadows = {diff > 0 ? (HzUIShadow){.color = cu_rgb(0x5cff3a), .blur = 0.5f * em}
                                  : diff < 0 ? (HzUIShadow){.color = cu_rgb(0xff3a3a), .blur = 0.5f * em}
                                             : (HzUIShadow){0}}}) {
            cu_textb(cu_numstr(k.unknown ? d.power : k.power, false), 1.15f * em, cu_rgb(0xffffff), 800, false, cu_black(0.5f));
          }
        }
      }
      // a soft foil sheen over the whole card
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {-bw, -bw},
                               .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
                  .gradient = cu_lin(125.0f, 4, (f32[]){0.0f, 0.28f, 0.62f, 0.78f},
                                     (HzUIColor[]){cu_rgba(0xffffff, 0.35f), cu_rgba(0xffffff, 0), cu_rgba(0xffffff, 0), cu_rgba(0xffffff, 0.18f)}),
                  .corner_radius = ui_corner_radius(0.6f * em)}) {}
      // the Ongoing aura: rings radiating from the card
      if (k.aura) {
        for (i32 r = 0; r < 3; r++) {
          f32 q = m_fmodf(ui->time / 2.4f + (f32)r / 3.0f, 1.0f);
          f32 rs = 1.0f + 0.45f * q;
          ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {-bw, -bw},
                                   .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                      .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
                      .border = {.color = cu_rgba(0x6fb6ff, 0.9f * (1.0f - q)), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                      .corner_radius = ui_corner_radius(0.8f * em),
                      .transform = cu_tf(0.0f, 0.0f, rs, 0.0f)}) {}
        }
      }
      // staged this turn: a dashed white outline
      if (k.staged) {
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = z, .offset = {-bw - 4.0f, -bw - 4.0f},
                                 .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                    .layout = {.sizing = {.width = ui_sizing_fixed(w + 8.0f), .height = ui_sizing_fixed(h + 8.0f)}},
                    .border = {.color = {255, 255, 255, 255}, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}, .style = UI_BORDER_DASHED},
                    .corner_radius = ui_corner_radius(0.6f * em + 4.0f)}) {}
      }
    }
    // the power change: a flash and the number floating up
    if (fx_on) {
      b32 buff = k.fx_n > 0;
      f32 fa = 1.0f - ft / 0.7f;
      if (fa > 0.0f) {
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = (i16)(z + 1), .offset = {-bw, -bw},
                                 .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                    .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
                    .gradient = buff ? cu_rad(0.5f, 0.5f, cu_rgba(0x8cff6e, 0.85f), cu_rgba(0x28c828, 0.2f), 0.7f)
                                     : cu_rad(0.5f, 0.5f, cu_rgba(0xff5a5a, 0.85f), cu_rgba(0xc81414, 0.25f), 0.7f),
                    .shadow = {.color = buff ? cu_rgb(0x5cff3a) : cu_rgb(0xff3a3a), .blur = 1.5f * em, .spread = 0.4f * em},
                    .corner_radius = ui_corner_radius(0.6f * em),
                    .opacity = opt_some(f32, fa)}) {}
      }
      f32 nq = ft; // the number: pops in, rises and fades
      f32 ns = nq < 0.2f ? 0.4f + 0.85f * nq / 0.2f : 1.25f - 0.25f * m_minf(1.0f, (nq - 0.2f) / 0.8f);
      f32 ny = nq < 0.2f ? 0.4f * em - 0.6f * em * nq / 0.2f : -0.2f * em - 1.4f * em * (nq - 0.2f) / 0.8f;
      f32 na = nq < 0.2f ? nq / 0.2f : nq < 0.7f ? 1.0f : 1.0f - (nq - 0.7f) / 0.3f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = (i16)(z + 2), .offset = {0.0f, 0.3f * h + ny},
                               .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                               .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP}},
                  .transform = cu_tf(0.0f, 0.0f, ns, 0.0f),
                  .opacity = opt_some(f32, m_clampf(na, 0.0f, 1.0f))}) {
        HzUITextConfig cfg = {.fontSize = 3.4f * em, .textColor = buff ? cu_rgb(0x6dff4a) : cu_rgb(0xff4a4a), .weight = 900,
                              .outlineColor = cu_rgb(0x1b2a34), .outlineWidth = (u16)m_maxf(1.0f, 0.2f * em)};
        cfg.shadows[0] = (HzUITextShadow){.color = buff ? cu_rgb(0x2a9a1a) : cu_rgb(0xb01818), .blur = 0.6f * em};
        ui_text(ui_str_c(cu_numstr(k.fx_n, true)), cfg);
      }
    }
  }
  return clicked;
}

typedef enum CuBtn { BTN_BLUE, BTN_SNAP, BTN_GHOST, BTN_GREEN, BTN_DARK } CuBtn;

// a button like the three.js .btn: rounded, a darker bottom edge, brighter under the mouse
hz_internal b32 cu_btn(HzUIElementId id, CString label, CuBtn kind, b32 enabled, const InputSystem *in, f32 font, HzUISizingAxis wsz,
                       HzUISizingAxis hsz, b32 nudge, f32 time) {
  b32 clicked = false;
  UIGradient g = {0};
  HzUIColor bg = {0}, ink = cu_rgb(0xffffff);
  switch (kind) {
  case BTN_BLUE: g = cu_lin2(180.0f, cu_rgb(0x3d9bff), cu_rgb(0x1655b0)); break;
  case BTN_SNAP: g = cu_lin2(180.0f, cu_rgb(0xffd23d), cu_rgb(0xf0a020)); ink = cu_rgb(0x3a2a00); break;
  case BTN_GREEN: g = cu_lin2(180.0f, cu_rgb(0x5fd07a), cu_rgb(0x1e7a35)); break;
  case BTN_DARK: bg = cu_rgb(0x3c4048); break;
  default: bg = cu_rgba(0xffffff, 0.15f); break;
  }
  b32 hover = enabled && ui_pointer_over(id);
  f32 np = 0.5f + 0.5f * m_sinf(time * M_PI / 0.9f);
  ui_element({.id = id,
              .layout = {.sizing = {.width = wsz, .height = hsz},
                         .padding = {.left = font < 14.0f ? 7 : 11, .right = font < 14.0f ? 7 : 11, .top = font < 14.0f ? 5 : 7, .bottom = font < 14.0f ? 5 : 7},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = bg,
              .gradient = g,
              .border = {.color = cu_black(0.30f), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
              .corner_radius = ui_corner_radius(10),
              .shadow = {.color = cu_black(0.25f), .offset_y = -3.0f, .inset = true},
              .shadows = {nudge ? (HzUIShadow){.color = cu_rgb(0x3d9bff), .blur = 22.0f, .spread = 6.0f} : (HzUIShadow){0}},
              .outline = {.width = nudge ? 3.0f : 0.0f, .color = {255, 255, 255, 255}},
              .opacity = enabled ? opt_none(f32) : opt_some(f32, 0.4f),
              .brightness = hover ? opt_some(f32, 1.1f) : nudge ? opt_some(f32, 1.0f + 0.25f * np) : opt_none(f32)}) {
    if (enabled && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    cu_textb(label, font, ink, 700, false, (HzUIColor){0});
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

// ---------------------------------------------------------------- the battle board (BattleView.svelte)

hz_internal CardLook cu_look(const Battle *b, i32 ci, const GameSave *s, f32 w) {
  const BCard *c = &b->c[ci];
  return (CardLook){.id = c->id, .power = b_power(b, ci), .level = c->owner == B_SIDE_P ? s->card_level[c->id] : 0, .w = w,
                    .uid = (u32)ci, .fx_n = c->fx_n, .fx_t = c->fx_t, .pulse_t = c->pulse_t,
                    .fx_delay = c->fx_src >= 0 && c->fx_src != ci ? 0.44f : 0.0f};
}

hz_internal b32 cu_any_staged(const Battle *b) {
  for (i32 i = 0; i < b->n; i++)
    if (b->c[i].staged)
      return true;
  return false;
}

hz_internal b32 cu_playable(const Battle *b, i32 ci) { return b->phase == PHASE_PLAN && card_def(b->c[ci].id).cost <= b->energy_left; }

// nothing left to do this turn: no card can go anywhere and no Activate is ready
hz_internal b32 cu_no_moves(const Battle *b) {
  if (b->phase != PHASE_PLAN)
    return false;
  for (i32 h = 0; h < b->hand_n[B_SIDE_P]; h++)
    for (i32 z = 0; z < 3; z++)
      if (b_can_play(b, b->hand[B_SIDE_P][h], z))
        return false;
  for (i32 i = 0; i < b->n; i++)
    if (b->c[i].owner == B_SIDE_P && b_can_activate(b, i))
      return false;
  return true;
}

hz_internal Clay_BoundingBox cu_box_of(HzUIElementId id) {
  HzUIElementData d = ui_get_element_data(id);
  return d.found ? d.boundingBox : (Clay_BoundingBox){0};
}

// a coloured location tile's text colours: on a picture white with a shadow, else ink on the colour
// the cards of a side at a zone in the order they were put there (a new card never pushes an older one out of its spot)
hz_internal i32 cu_zone_order(const Battle *b, i32 z, i32 side, i32 *out) {
  i32 n = 0;
  for (i32 i = 0; i < b->n; i++) {
    const BCard *c = &b->c[i];
    if (!c->alive || c->zone != z || c->owner != side)
      continue;
    i32 k = n++;
    while (k > 0 && b->c[out[k - 1]].order > c->order) {
      out[k] = out[k - 1];
      k--;
    }
    out[k] = i;
  }
  return n;
}

hz_internal void cu_zone_texts(CardUi *ui, const Battle *b, i32 z, f32 name_px, f32 text_px, b32 big) {
  ZoneDef zd = zone_def(b->zone[z]);
  b32 rev = b->zone_rev[z];
  b32 pic = rev && handle_is_valid(cu_zone_tex(ui, b->zone[z]));
  HzUIColor ink = pic ? cu_rgb(0xffffff) : cu_ink(rev ? zd.color : cu_rgb(0x555555));
  HzUIColor sh = pic || !cu_light(rev ? zd.color : cu_rgb(0x555555)) ? cu_black(0.9f) : (HzUIColor){0};
  if (!rev)
    cu_textb("?", name_px * 1.8f, ink, 800, false, sh);
  cu_textb(rev ? zd.name : "Unrevealed", name_px, ink, 800, true, sh);
  CString txt = rev ? zd.text : CU_FMT(ui, big ? "This location reveals on turn %." : "Reveals on turn %", fmt_i32(z + 1));
  cu_textb(txt, text_px, cu_alpha(ink, 0.95f), 600, true, sh);
}

// the location's background: its picture with a dark fade (revealed), or grey dots (not yet)
// the location's picture with a dark fade, inside a frame of the tile's size whose rounded corners mask it (the tile's
// own clip is a rectangle: the picture's corners stuck out of the rounded frame). Nothing while it is unrevealed
hz_internal void cu_zone_fill(CardUi *ui, const Battle *b, i32 z, f32 bw, f32 bh, f32 radius, i16 zi) {
  if (!b->zone_rev[z])
    return;
  i32 id = b->zone[z];
  GpuTexture tex = cu_zone_tex(ui, id);
  if (!handle_is_valid(tex))
    return;
  HzUIClipPath round = {.round = ui_corner_radius(radius)};
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = zi, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
              .layout = {.sizing = {.width = ui_sizing_fixed(bw), .height = ui_sizing_fixed(bh)}},
              .clip = {.horizontal = true, .vertical = true},
              .clip_path = round}) {
    cu_cover(tex, (f32)ui->zone_w[id], (f32)ui->zone_h[id], bw, bh, zi);
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .clip_to = UI_CLIP_TO_ATTACHED_PARENT, .z_index = zi,
                             .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(bw), .height = ui_sizing_fixed(bh)}},
                .gradient = cu_lin(0.0f, 2, (f32[]){0.0f, 0.75f}, (HzUIColor[]){cu_black(0.72f), cu_black(0.15f)})}) {}
  }
}

hz_internal UIGradient cu_dots(void) {
  UIGradient g = {.kind = UI_GRADIENT_RADIAL, .stop_count = 3, .circle = true, .size = UI_RADIAL_CLOSEST_SIDE};
  g.stops[0] = (HzUIGradientStop){0.0f, cu_rgba(0xffffff, 0.25f)};
  g.stops[1] = (HzUIGradientStop){0.36f, cu_rgba(0xffffff, 0.25f)};
  g.stops[2] = (HzUIGradientStop){0.4f, cu_rgba(0xffffff, 0)};
  return g;
}

// a power pill on the location's top or bottom edge; the side that leads glows
hz_internal void cu_power_pill(i32 v, b32 lead, b32 enemy, b32 top) {
  HzUIColor bg = lead ? (enemy ? cu_rgb(0xd9361b) : cu_rgb(0xf0a020)) : cu_rgb(0x1b2a34);
  ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 36, .offset = {0.0f, top ? -13.0f : 13.0f},
                           .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                           .attach_points = {.element = top ? UI_ATTACH_POINT_CENTER_TOP : UI_ATTACH_POINT_CENTER_BOTTOM,
                                             .parent = top ? UI_ATTACH_POINT_CENTER_TOP : UI_ATTACH_POINT_CENTER_BOTTOM}},
              .layout = {.sizing = {.width = ui_sizing_fit(34.0f, 200.0f), .height = ui_sizing_fixed(26.0f)}, .padding = {.left = 7, .right = 7},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = bg,
              .border = {.color = cu_rgba(0xffffff, 0.5f), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
              .corner_radius = ui_corner_radius(13),
              .shadow = {.color = lead ? (enemy ? cu_rgb(0xff5a3d) : cu_rgb(0xffd23d)) : (HzUIColor){0}, .blur = 12.0f}}) {
    cu_textb(cu_numstr(v, false), 16, lead && !enemy ? cu_rgb(0x2a1a00) : cu_rgb(0xffffff), 800, false, (HzUIColor){0});
  }
}

hz_internal void cu_brick_icon(f32 sz, HzUIColor c) {
  ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)(sz * 0.18f)}}) {
      for (i32 i = 0; i < 2; i++)
        ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(sz * 0.3f), .height = ui_sizing_fixed(sz * 0.18f)}},
                    .background_color = c, .corner_radius = {.top_left = sz * 0.08f, .top_right = sz * 0.08f}}) {}
    }
    ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(sz), .height = ui_sizing_fixed(sz * 0.55f)}}, .background_color = c,
                .corner_radius = ui_corner_radius(sz * 0.1f)}) {}
  }
}

// the opponent's portrait in the top bar, with a badge: "1st" when they reveal first this turn (whoever wins more
// locations; a tie: a coin), lit gold and pulsing; "2nd" when you do
hz_internal void cu_portrait(CardUi *ui, HzUIElementId id, CString label, f32 label_size, u32 bg, f32 sz, b32 first, b32 show) {
  HzUIColor white = {255, 255, 255, 255}, none = {0};
  f32 pulse = 0.5f + 0.5f * m_sinf(ui->time * 5.0f);
  b32 lit = first && show;
  ui_element({.id = id,
              .layout = {.sizing = {.width = ui_sizing_fixed(sz), .height = ui_sizing_fixed(sz)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = cu_rgb(bg),
              .border = {.color = lit ? cu_rgb(0xffd23d) : cu_rgba(0xffffff, 0.6f), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
              .corner_radius = ui_corner_radius(sz * 0.28f),
              .shadow = {.color = lit ? cu_rgba(0xffd23d, 0.55f + 0.4f * pulse) : none, .blur = 10.0f + 8.0f * pulse, .spread = 2.0f + 2.0f * pulse}}) {
    cu_textb(label, label_size, white, 800, false, none);
    if (show) {
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 36, .offset = {9.0f, -4.0f}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                               .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP}},
                  .layout = {.padding = {.left = 4, .right = 4, .top = 1, .bottom = 1}},
                  .background_color = first ? cu_rgb(0xffd23d) : cu_rgb(0x3a3f4a),
                  .border = {.color = white, .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
                  .corner_radius = ui_corner_radius(7)}) {
        cu_textb(first ? "1st" : "2nd", 11, first ? cu_rgb(0x2a1a00) : white, 900, false, none);
      }
    }
  }
}

hz_internal void battle_draw(CardUi *ui, Battle *b, const InputSystem *in, const GameSave *s, f32 sw, f32 sh) {
  HzUIColor white = {255, 255, 255, 255}, none = {0};
  ui->sbuf_n = 0;
  ui->hover_card = -1;
  b32 phone = sw <= 640.0f;
  b32 short_land = sh <= 520.0f && sw > sh;
  b32 modal = ui->inspect != INSPECT_NONE || b->phase == PHASE_OVER;
  b32 dragging = ui->drag >= 0 && ui->drag_moved;
  b32 no_moves = cu_no_moves(b);
  OsSafeAreaInsets safe = os_get_safe_area(); // the notch and the home bar
  // ---- sizes (the three.js css, in px) ----
  f32 top_h = 50.0f + safe.top;
  f32 hw = phone ? m_clampf(sw * 0.19f, 52.0f, 96.0f) : short_land ? m_clampf(sw * 0.10f, 48.0f, 80.0f) : m_clampf(m_minf(sw * 0.15f, sh * 0.17f), 58.0f, 128.0f);
  f32 rowh = hw * 1.4f + 18.0f;
  f32 bottom_h = (phone ? rowh + 54.0f + 18.0f : rowh + 16.0f) + safe.bottom;
  f32 pad = (phone ? 4.0f : m_clampf(sw * 0.012f, 4.0f, 14.0f)) + m_maxf(safe.left, safe.right);
  f32 board_h = sh - top_h - bottom_h - 16.0f;
  // a short screen: smaller hand cards first, so the board keeps room for two rows of cards per side
  if (board_h < 300.0f && hw > 48.0f) {
    f32 cut = m_minf(hw - 48.0f, (300.0f - board_h) / 1.4f);
    hw -= cut;
    rowh = hw * 1.4f + 18.0f;
    bottom_h -= cut * 1.4f;
    board_h += cut * 1.4f;
  }
  f32 tile_h = short_land ? 64.0f : m_clampf(sh * 0.15f, 78.0f, 130.0f);
  f32 zgap = phone ? 14.0f : short_land ? 12.0f : 16.0f;
  f32 slot_h = m_maxf(40.0f, (board_h - tile_h - 2.0f * zgap) * 0.5f);
  b32 wide = sw / sh >= 1.6f;
  f32 colgap = wide ? 18.0f : pad;
  f32 colw = wide ? m_maxf(170.0f, (slot_h - 14.0f) * 0.5f * 0.714f * 2.0f + 5.0f + 14.0f) : (sw - 2.0f * pad - 2.0f * colgap) / 3.0f;
  colw = m_minf(colw, (sw - 2.0f * pad - 2.0f * colgap) / 3.0f);
  f32 cw = m_minf((colw - 8.0f - 14.0f) * 0.5f, (slot_h - 8.0f - 14.0f) * 0.5f * 0.714f);
  cw = m_maxf(cw, 20.0f);
  // a tall screen (a phone upright): the slots need less height than they get; the rest makes the hand cards and the
  // location pictures bigger, so the board fills the screen instead of leaving empty bands
  {
    f32 slot_need = 2.0f * cw * 1.4f + 5.0f + 14.0f;
    if (slot_h > slot_need + 8.0f) {
      f32 spare = (slot_h - slot_need) * 2.0f;
      f32 hw_cap = phone ? m_minf((sw - 20.0f) / 4.6f, 110.0f) : 128.0f; // 4 to 5 cards side by side
      f32 dh = m_clampf(spare * 0.5f / 1.4f, 0.0f, m_maxf(0.0f, hw_cap - hw));
      hw += dh;
      rowh += dh * 1.4f;
      bottom_h += dh * 1.4f;
      spare -= dh * 1.4f;
      f32 grow = m_minf(spare * 0.6f, tile_h * 0.7f);
      tile_h += grow;
      spare -= grow;
      slot_h = slot_need + spare * 0.5f;
    }
  }

  // ---- the background: the place's picture (wild forest, Dex's cave, Lumi's lab), a little darker ----
  i32 bg_kind = b->kind == BATTLE_WILD ? 0 : b->kind == BATTLE_RIVAL ? 1 : 2;
  GpuTexture bgt = cu_back_tex(ui, bg_kind);
  ui_element({.id = ui_id("BattleBg"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 30},
              .layout = {.sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)}},
              .background_color = cu_rgb(0x1a4a28),
              .clip = {.horizontal = true, .vertical = true}}) {
    cu_cover(bgt, 480.0f, 270.0f, sw, sh, 30);
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 31, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)}},
                .gradient = cu_lin2(180.0f, cu_shade(0x08100c, 0.35f), cu_shade(0x08100c, 0.55f))}) {}
    if (b->shiny) { // a golden shimmer over a shiny battle
      f32 p = 1.5f - m_fmodf(ui->time / 4.0f, 1.0f) * 3.0f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 31, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)}},
                  .gradient = cu_lin(115.0f, 3, (f32[]){p * 0.4f + 0.2f, p * 0.4f + 0.35f, p * 0.4f + 0.5f},
                                     (HzUIColor[]){cu_rgba(0xfff0aa, 0), cu_rgba(0xfff0aa, 0.25f), cu_rgba(0xfff0aa, 0)})}) {}
    }
  }

  ui->z = 33;
  ui_element({.id = ui_id("Battle"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 32},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)}}}) {
    // ---- the top bar: the opponent, the stakes and the turn, log / SNAP / Retreat ----
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = phone ? 5 : 8,
                           .padding = {.left = (u16)((phone ? 6 : 10) + safe.left), .right = (u16)((phone ? 6 : 10) + safe.right), .top = (u16)(6 + safe.top), .bottom = 6},
                           .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(top_h)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_black(0.3f)}) {
      {
        char *ini = (char *)cu_fmt_slot(ui);
        ini[0] = b->kind == BATTLE_WILD ? '?' : b->opponent[0];
        ini[1] = 0;
        cu_portrait(ui, ui_id("FoePortrait"), ini, 18, 0x7b2fbe, 38.0f, b->first == B_SIDE_AI, b->phase != PHASE_OVER);
      }
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 1, .sizing = {.width = ui_sizing_fit(0.0f, phone ? 64.0f : 400.0f)}},
                  .clip = {.horizontal = true}}) {
        cu_textb(b->opponent, phone ? 13 : 17, white, 700, false, none);
        if (!phone)
          cu_textb(CU_FMT(ui, "Hand %  \xc2\xb7  Deck %", fmt_i32(b->hand_n[B_SIDE_AI]), fmt_i32(b->deck_n[B_SIDE_AI])), 11, cu_rgba(0xffffff, 0.8f), 600, false, none);
      }
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
      b32 hot = b->stakes > 1;
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 5, .padding = {.left = 10, .right = 10, .top = 2, .bottom = 2},
                             .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                  .background_color = hot ? none : cu_black(0.35f),
                  .gradient = hot ? cu_lin2(180.0f, cu_rgb(0xff9a3d), cu_rgb(0xd9361b)) : (UIGradient){0},
                  .border = {.color = cu_rgba(0xffffff, 0.4f), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                  .corner_radius = ui_corner_radius(14),
                  .shadow = {.color = hot ? cu_rgb(0xff9a3d) : none, .blur = 14.0f}}) {
        cu_brick_icon(16.0f, cu_rgb(0xf2cd37));
        cu_textb(cu_numstr(b->stakes, false), 19, white, 800, false, none);
      }
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 3, .child_alignment = {.y = UI_ALIGN_Y_BOTTOM}}}) {
        cu_textb("T", 14, white, 500, false, none);
        cu_textb(cu_numstr(b->turn, false), 19, white, 800, false, none);
        cu_textb(CU_FMT(ui, "/%", fmt_i32(b->max_turns)), 14, white, 500, false, none);
      }
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
      if (cu_btn(ui_id("BtnLog"), "Log", BTN_GHOST, true, in, phone ? 12 : 14, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false, ui->time) && !modal)
        ui->click_log = true;
      if (cu_btn(ui_id("BtnSnap"), "SNAP", BTN_SNAP, !b->p_snapped && b->phase != PHASE_OVER, in, phone ? 13 : 16, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false,
                 ui->time) && !modal)
        ui->click_snap = true;
      if (cu_btn(ui_id("BtnRetreat"), phone ? "Flee" : "Retreat", BTN_GHOST, b->phase != PHASE_OVER, in, phone ? 12 : 15, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999),
                 false, ui->time) && !modal)
        ui->click_retreat = true;
    }

    // ---- the board: three locations, the opponent's cards above, yours below ----
    i32 sel = ui->selected;
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = (u16)colgap, .padding = {.left = (u16)pad, .right = (u16)pad, .top = 8, .bottom = 8},
                           .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
      for (i32 z = 0; z < 3; z++) {
        ZoneDef zd = zone_def(b->zone[z]);
        i32 ap = b_zone_power(b, z, B_SIDE_AI), pp = b_zone_power(b, z, B_SIDE_P);
        i32 win = b_zone_winner(b, z);
        b32 ok = (sel >= 0 && b_can_play(b, sel, z)) ||
                 (dragging && !ui->drag_from_hand && b->c[ui->drag].zone != z && b_zone_open(b, z, B_SIDE_P));
        b32 over = dragging && ui->drag_over == z;
        ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)zgap, .sizing = {.width = ui_sizing_fixed(colw), .height = ui_sizing_grow(0)}}}) {
          // the opponent's cards (2 x 2, from the bottom)
          ui_element({.id = ui_idi("AiSlots", (u32)z),
                      .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(slot_h)},
                                 .padding = ui_padding_all(4), .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}},
                      .background_color = cu_black(0.14f),
                      .corner_radius = ui_corner_radius(10)}) {
            ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 5, .sizing = {.width = ui_sizing_fixed(cw * 2.0f + 5.0f)}}}) {
              i32 zl[B_MAX_CARDS], zn = cu_zone_order(b, z, B_SIDE_AI, zl);
              for (i32 zi = 0; zi < zn; zi++) {
                i32 i = zl[zi];
                const BCard *c = &b->c[i];
                CardLook k = cu_look(b, i, s, cw);
                k.hidden = !c->revealed;
                k.hide_cost = true;
                k.aura = c->revealed && card_def(c->id).kind == KIND_ONGOING;
                if (cu_card(ui, k, in, ui_idi("BCard", (u32)i)) && !modal && c->revealed)
                  ui->click_card = i;
              }
            }
          }
          // the location: its picture, name and rule, the two powers on its edges
          f32 tsc = ok ? 1.03f : 1.0f;
          ui_element({.id = ui_idi("ZoneTile", (u32)z),
                      .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(tile_h)},
                                 .padding = {.left = 4, .right = 4, .top = 16, .bottom = 16}, .child_gap = 1,
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}},
                      .background_color = b->zone_rev[z] ? zd.color : cu_rgb(0x555555),
                      .backgrounds = b->zone_rev[z] ? NULL : ui_backgrounds({.gradient = cu_dots(), .size = {18.0f, 18.0f}}),
                      .border = {.color = b->zone_rev[z] ? zd.color : cu_black(0.35f), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}, .overlay = true},
                      .corner_radius = ui_corner_radius(14),
                      .clip = {.horizontal = true, .vertical = true},
                      .shadow = {.color = cu_black(0.3f), .offset_y = 4.0f, .blur = 10.0f},
                      .shadows = {ok || over ? (HzUIShadow){.color = over ? cu_rgba(0xffffff, 0.55f) : cu_rgb(0xffd23d), .blur = 22.0f, .spread = 6.0f}
                                             : (HzUIShadow){.color = cu_black(0.2f), .offset_y = -6.0f, .inset = true}},
                      .outline = {.width = ok || over ? 3.0f : 0.0f, .color = white},
                      .transform = cu_tf(0, 0, tsc, 0)}) {
            if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame && !modal)
              ui->click_zone = z;
            cu_zone_fill(ui, b, z, colw, tile_h, 14.0f, 33);
            ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 34, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                                     .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                                     .offset = {0.0f, -14.0f}},
                        .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 1, .sizing = {.width = ui_sizing_fixed(colw - 10.0f)},
                                   .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
              cu_zone_texts(ui, b, z, m_clampf(sw * 0.026f, 11.5f, 17.0f), m_clampf(sw * 0.022f, 9.0f, 12.0f), false);
            }
            // the powers on the tile's top and bottom edge (floating: not clipped by the tile)
            cu_power_pill(ap, win == B_SIDE_AI, true, true);
            cu_power_pill(pp, win == B_SIDE_P, false, false);
          }
          // your cards (2 x 2, from the top)
          ui_element({.id = ui_idi("MySlots", (u32)z),
                      .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(slot_h)},
                                 .padding = ui_padding_all(4), .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_TOP}},
                      .background_color = cu_rgba(0xffffff, 0.07f),
                      .corner_radius = ui_corner_radius(10),
                      .outline = {.width = over ? 3.0f : 0.0f, .color = white},
                      .shadow = {.color = over ? cu_rgba(0xffffff, 0.55f) : none, .blur = 24.0f, .spread = 6.0f}}) {
            if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame && !modal)
              ui->click_zone = z;
            ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 5, .sizing = {.width = ui_sizing_fixed(cw * 2.0f + 5.0f)}}}) {
              i32 zl[B_MAX_CARDS], zn = cu_zone_order(b, z, B_SIDE_P, zl);
              for (i32 zi = 0; zi < zn; zi++) {
                i32 i = zl[zi];
                const BCard *c = &b->c[i];
                CardLook k = cu_look(b, i, s, cw);
                b32 plan = b->phase == PHASE_PLAN;
                k.hidden = !c->revealed && !plan;   // after End Turn your new cards lie face down until revealed
                k.hide_cost = true;
                k.staged = (c->staged || c->act_order >= 0) && plan;
                k.aura = c->revealed && card_def(c->id).kind == KIND_ONGOING;
                k.ready = b_can_activate(b, i) && plan;
                if (dragging && ui->drag == i)
                  k.opacity = 0.001f; // the dragged card follows the pointer instead
                // the attacker lunges at its target
                for (i32 j = 0; j < b->n; j++) {
                  const BCard *t = &b->c[j];
                  if (j == i || t->fx_src != i || (t->fx_t > 0.42f && t->die_t > 0.42f) || !ui->has_box[i] || !ui->has_box[j])
                    continue;
                  f32 q = m_minf(t->fx_t, t->die_t) / 0.42f;
                  f32 dx = (ui->box[j].x + ui->box[j].width * 0.5f) - (ui->box[i].x + ui->box[i].width * 0.5f);
                  f32 dy = (ui->box[j].y + ui->box[j].height * 0.5f) - (ui->box[i].y + ui->box[i].height * 0.5f);
                  f32 len = m_maxf(1.0f, m_sqrtf(dx * dx + dy * dy));
                  f32 a = q < 0.25f ? -0.3f * q / 0.25f : q < 0.55f ? -0.3f + 1.3f * (q - 0.25f) / 0.3f : 1.0f - (q - 0.55f) / 0.45f;
                  k.tx = dx / len * 22.0f * a;
                  k.ty = dy / len * 22.0f * a;
                  k.scale = q < 0.25f ? 1.0f - 0.05f * q / 0.25f : q < 0.55f ? 0.95f + 0.2f * (q - 0.25f) / 0.3f : 1.15f - 0.15f * (q - 0.55f) / 0.45f;
                  break;
                }
                HzUIElementId cid = ui_idi("BCard", (u32)i);
                if (cu_card(ui, k, in, cid) && !modal) {
                  if (c->staged && plan)
                    ui->press_card = i; // drag it elsewhere, or tap it
                  else
                    ui->click_card = i;
                }
              }
            }
          }
        }
      }
    }

    // ---- the hand row: the energy orb with the deck pile, the hand, Undo / End Turn ----
    f32 orb = phone ? 54.0f : rowh;
    f32 ctrl_w = phone ? sw - orb - 40.0f - safe.left - safe.right : 150.0f;
    f32 hand_w = (phone ? sw - 20.0f : sw - orb - ctrl_w - 20.0f - 20.0f) - safe.left - safe.right;
    i32 hn = b->hand_n[B_SIDE_P];
    f32 hgap = hn < 2 ? 6.0f : m_minf(6.0f, (hand_w - (f32)hn * hw) / (f32)(hn - 1));
    ui_element({.id = ui_id("BattleBottom"),
                .layout = {.layout_direction = phone ? UI_TOP_TO_BOTTOM : UI_LEFT_TO_RIGHT, .child_gap = phone ? 4 : 10,
                           .padding = {.left = (u16)(10 + safe.left), .right = (u16)(10 + safe.right), .top = phone ? 2 : 6, .bottom = (u16)(10 + safe.bottom)},
                           .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_fixed(bottom_h)}, .child_alignment = {.y = UI_ALIGN_Y_BOTTOM}},
                .background_color = cu_black(0.2f),
                .outline = {.width = dragging && ui->drag_over == 3 ? 3.0f : 0.0f, .color = cu_rgba(0xffffff, 0.6f)}}) {
      // energy left (big) / this turn's energy (= the turn), and your deck: drawn cards fly out of it
#define CU_ORB()                                                                                                                                      \
  ui_element({.id = ui_id("EnergyOrb"),                                                                                                               \
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .sizing = {.width = ui_sizing_fixed(orb), .height = ui_sizing_fixed(orb)},             \
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},                                                         \
              .gradient = cu_rad(0.35f, 0.3f, cu_rgb(0x7ac0ff), cu_rgb(0x1655b0), 1.0f),                                                            \
              .border = {.color = cu_rgba(0xffffff, 0.7f), .width = {.left = 4, .right = 4, .top = 4, .bottom = 4}},                                  \
              .corner_radius = ui_corner_radius(orb * 0.5f),                                                                                          \
              .shadow = {.color = cu_rgba(0x3d9bff, 0.6f), .blur = 20.0f}}) {                                                                        \
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_alignment = {.y = UI_ALIGN_Y_BOTTOM}}}) {                                     \
      cu_textb(cu_numstr(b->energy_left, false), phone ? 22.0f : orb * 0.42f, white, 900, false, cu_black(0.3f));                                  \
      cu_textb(CU_FMT(ui, "/%", fmt_i32(b->energy)), phone ? 10.0f : orb * 0.17f, cu_rgba(0xffffff, 0.75f), 700, false, none);                      \
    }                                                                                                                                                 \
    cu_textb("ENERGY", phone ? 8.0f : orb * 0.1f, cu_rgba(0xffffff, 0.85f), 700, false, none);                                                       \
    f32 pw_ = m_maxf(26.0f, orb * 0.34f);                                                                                                            \
    ui_element({.id = ui_id("DeckPile"),                                                                                                              \
                .floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 34, .offset = {orb * 0.14f, -orb * 0.1f},                                  \
                             .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP}},                          \
                .layout = {.sizing = {.width = ui_sizing_fixed(pw_), .height = ui_sizing_fixed(pw_ * 1.4f)},                                         \
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},                                                       \
                .background_color = cu_rgb(0x7b2fbe),                                                                                                 \
                .backgrounds = ui_backgrounds({.gradient = {.kind = UI_GRADIENT_LINEAR, .angle = 45.0f, .stop_count = 4, .repeating = true,          \
                                                            .stops = {{0.0f, {123, 47, 190, 255}}, {0.5f, {123, 47, 190, 255}},                     \
                                                                      {0.5f, {90, 31, 148, 255}}, {1.0f, {90, 31, 148, 255}}},                      \
                                                            .stop_offset_px = {0, 0, 0, 0}},                                                          \
                                               .size = {12.0f, 12.0f}}),                                                                             \
                .border = {.color = white, .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},                                                  \
                .corner_radius = ui_corner_radius(6),                                                                                                 \
                .shadow = {.color = cu_rgb(0xd8d8e8), .offset_x = 2.0f, .offset_y = 2.0f},                                                           \
                .shadows = {{.color = cu_rgb(0x9a9ab0), .offset_x = 4.0f, .offset_y = 4.0f}, {.color = cu_black(0.5f), .offset_y = 4.0f, .blur = 10.0f}}}) { \
      ui_element({.layout = {.padding = {.left = 4, .right = 4}}, .background_color = cu_black(0.55f), .corner_radius = ui_corner_radius(9)}) {    \
        cu_textb(cu_numstr(b->deck_n[B_SIDE_P], false), m_maxf(10.0f, orb * 0.13f), white, 900, false, none);                                       \
      }                                                                                                                                               \
    }                                                                                                                                                 \
  }
#define CU_CONTROLS(dir)                                                                                                                             \
  ui_element({.layout = {.layout_direction = dir, .child_gap = 6, .sizing = {.width = ui_sizing_fixed(ctrl_w), .height = ui_sizing_fixed(orb)}}}) { \
    if (cu_btn(ui_id("BtnUndo"), "Undo", BTN_GHOST, b->phase == PHASE_PLAN && cu_any_staged(b), in, 15,                                              \
               phone ? ui_sizing_fit(0, 999) : ui_sizing_grow(0), ui_sizing_grow(0), false, ui->time) && !modal)                                    \
      ui->click_undo = true;                                                                                                                          \
    if (cu_btn(ui_id("BtnEnd"), b->phase == PHASE_REVEAL ? "\xe2\x80\xa6" : "End Turn", BTN_BLUE, b->phase == PHASE_PLAN, in, phone ? 17 : 20,   \
               ui_sizing_grow(0), phone ? ui_sizing_grow(0) : ui_sizing_fixed(orb * 0.62f), no_moves, ui->time) && !modal)                         \
      ui->click_end = true;                                                                                                                           \
  }
      if (!phone)
        CU_ORB();
      // the hand: the cards overlap when there is not enough room
      ui_element({.id = ui_id("Hand"),
                  .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.width = ui_sizing_fixed(hand_w), .height = ui_sizing_fixed(rowh)},
                             .padding = {.top = 18}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}}}) {
        Clay_BoundingBox pile = cu_box_of(ui_id("DeckPile"));
        i32 fresh = 0;
        for (i32 h = 0; h < hn; h++) {
          i32 ci = b->hand[B_SIDE_P][h];
          const BCard *c = &b->c[ci];
          f32 slot_w = h < hn - 1 ? hw + hgap : hw;
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(m_maxf(8.0f, slot_w)), .height = ui_sizing_fixed(hw * 1.4f)}}}) {
            CardLook k = cu_look(b, ci, s, hw);
            k.fx_n = 0;
            k.lit = ui->selected == ci && !dragging;
            k.dim = !cu_playable(b, ci);
            if (dragging && ui->drag == ci)
              k.opacity = 0.001f;
            // a new card flies out of the deck pile, one after the other
            f32 delay = 0.12f + 0.17f * (f32)fresh;
            if (c->draw_t < delay + 0.52f) {
              fresh++;
              f32 q = (c->draw_t - delay) / 0.52f;
              if (q < 0.0f || !ui->has_box[ci] || pile.width <= 0.0f) {
                k.opacity = q < 0.0f && pile.width > 0.0f ? 0.001f : k.opacity;
                if (pile.width > 0.0f && ui->has_box[ci] && q < 0.0f) {
                  k.opacity = 0.6f;
                  k.tx = pile.x + pile.width * 0.5f - (ui->box[ci].x + ui->box[ci].width * 0.5f);
                  k.ty = pile.y + pile.height * 0.5f - (ui->box[ci].y + ui->box[ci].height * 0.5f);
                  k.scale = pile.width / hw;
                  k.rot = -12.0f;
                }
              } else {
                f32 dx = pile.x + pile.width * 0.5f - (ui->box[ci].x + ui->box[ci].width * 0.5f);
                f32 dy = pile.y + pile.height * 0.5f - (ui->box[ci].y + ui->box[ci].height * 0.5f);
                f32 s0 = pile.width / hw;
                f32 e = q < 0.65f ? cu_ease_out(q / 0.65f) : 1.0f;
                if (q < 0.65f) {
                  k.tx = dx * (1.0f - e);
                  k.ty = dy * (1.0f - e) - 34.0f * e;
                  k.scale = s0 + (1.12f - s0) * e;
                  k.rot = -12.0f + 14.0f * e;
                  k.glow = e;
                } else {
                  f32 r = cu_ease_out((q - 0.65f) / 0.35f);
                  k.ty = -34.0f * (1.0f - r);
                  k.scale = 1.12f - 0.12f * r;
                  k.rot = 2.0f * (1.0f - r);
                  k.glow = 1.0f - r;
                }
              }
            }
            if (cu_card(ui, k, in, ui_idi("BCard", (u32)ci)) && !modal)
              ui->press_card = ci;
          }
        }
      }
      if (phone) {
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
          CU_ORB();
          CU_CONTROLS(UI_LEFT_TO_RIGHT);
        }
      } else {
        CU_CONTROLS(UI_TOP_TO_BOTTOM);
      }
#undef CU_ORB
#undef CU_CONTROLS
    }
  }
  // remember where every card is: projectiles and the shatter start there
  for (i32 i = 0; i < b->n; i++) {
    if (!b->c[i].alive)
      continue;
    HzUIElementData d = ui_get_element_data(ui_idi("BCard", (u32)i));
    ui->has_box[i] = d.found;
    if (d.found)
      ui->box[i] = d.boundingBox;
  }

  // ---- the attacks: a ball flies from the attacker, then the impact; destroyed cards burst into shards ----
  for (i32 j = 0; j < b->n; j++) {
    const BCard *t = &b->c[j];
    i32 src = t->fx_src;
    b32 kill = !t->alive && t->die_t < 1.6f;
    f32 at = kill ? t->die_t : t->fx_t;
    if (src < 0 || src == j || at > 0.85f || !ui->has_box[src] || !ui->has_box[j] || (!kill && t->fx_n == 0))
      continue;
    b32 buff = !kill && t->fx_n > 0;
    f32 ax = ui->box[src].x + ui->box[src].width * 0.5f, ay = ui->box[src].y + ui->box[src].height * 0.5f;
    f32 bx = ui->box[j].x + ui->box[j].width * 0.5f, by = ui->box[j].y + ui->box[j].height * 0.5f;
    f32 q = (at - 0.12f) / 0.32f;
    if (q >= 0.0f && q < 1.0f) {
      f32 arc = buff ? -40.0f : -18.0f;
      f32 e = q * q; // ease-in
      f32 px = ax + (bx - ax) * e, py = ay + (by - ay) * e + arc * 4.0f * e * (1.0f - e);
      f32 ps = e < 0.5f ? 0.4f + 1.6f * e : 1.2f - 0.6f * (e - 0.5f);
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 45, .offset = {px - 17.0f, py - 17.0f},
                               .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.sizing = {.width = ui_sizing_fixed(34), .height = ui_sizing_fixed(34)}},
                  .backgrounds = ui_backgrounds({.gradient = {.kind = UI_GRADIENT_RADIAL, .circle = true, .stop_count = 4, .at = opt_some(vec2, ((vec2){0.4f, 0.4f})),
                                                              .stops = {{0.0f, buff ? cu_rgb(0xffffff) : cu_rgb(0xfff6c8)}, {0.3f, buff ? cu_rgb(0xb6ff8f) : cu_rgb(0xffb020)},
                                                                        {0.6f, buff ? cu_rgb(0x2fd02a) : cu_rgb(0xff3a1a)}, {0.75f, buff ? cu_rgba(0x28c828, 0) : cu_rgba(0xff2800, 0)}}}}),
                  .corner_radius = ui_corner_radius(17),
                  .shadow = {.color = buff ? cu_rgba(0x5aff3c, 0.7f) : cu_rgba(0xff5014, 0.7f), .blur = 18.0f, .spread = 8.0f},
                  .transform = cu_tf(0, 0, ps, 0)}) {}
    } else if (q >= 1.0f) {
      f32 r = (at - 0.44f) / (kill ? 0.5f : 0.38f);
      if (r < 1.0f) {
        f32 sz = kill ? 90.0f * (0.4f + 2.2f * r) : 90.0f * (0.3f + 1.5f * r);
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 44, .offset = {bx - sz * 0.5f, by - sz * 0.5f},
                                 .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                    .layout = {.sizing = {.width = ui_sizing_fixed(sz), .height = ui_sizing_fixed(sz)}},
                    .backgrounds = ui_backgrounds({.gradient = {.kind = UI_GRADIENT_RADIAL, .circle = true, .stop_count = 3,
                                                                .stops = {{0.0f, buff ? cu_rgba(0xffffff, 0.95f) : cu_rgba(0xffe678, 0.95f)},
                                                                          {0.45f, buff ? cu_rgba(0x78ff5a, 0.7f) : cu_rgba(0xff461e, 0.7f)},
                                                                          {0.7f, buff ? cu_rgba(0x00ff00, 0) : cu_rgba(0xff0000, 0)}}}}),
                    .corner_radius = ui_corner_radius(sz * 0.5f),
                    .opacity = opt_some(f32, 1.0f - r)}) {}
      }
    }
  }
  ui->z = 47;
  for (i32 j = 0; j < b->n; j++) {
    const BCard *t = &b->c[j];
    if (t->alive || t->die_t > 1.6f || !ui->has_box[j])
      continue;
    Clay_BoundingBox r = ui->box[j];
    CardLook k = cu_look(b, j, s, r.width);
    k.hide_cost = true;
    k.fx_n = 0;
    k.uid = 10000u + (u32)j * 8u;
    if (t->die_t < 0.56f) { // frozen until the ball hits
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 46, .offset = {r.x, r.y}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}}) {
        cu_card(ui, k, in, ui_idi("Dying", (u32)j));
      }
      continue;
    }
    f32 q = (t->die_t - 0.56f) / 1.0f;
    for (i32 gx = 0; gx < 2; gx++)
      for (i32 gy = 0; gy < 3; gy++) {
        u32 hs = (u32)(j * 31 + gx * 7 + gy * 3);
        f32 jit[8];
        for (i32 m = 0; m < 8; m++)
          jit[m] = (cu_hash(hs * 8u + (u32)m) - 0.5f) * 0.08f;
        f32 x0 = gx / 2.0f, x1 = (gx + 1) / 2.0f, y0 = gy / 3.0f, y1 = (gy + 1) / 3.0f;
        HzUIClipPath cp = {.count = 4};
        cp.points[0].fraction = (vec2){x0 + jit[0], y0 + jit[1]};
        cp.points[1].fraction = (vec2){x1 + jit[2], y0 + jit[3]};
        cp.points[2].fraction = (vec2){x1 + jit[4], y1 + jit[5]};
        cp.points[3].fraction = (vec2){x0 + jit[6], y1 + jit[7]};
        f32 cx = (x0 + x1) * 0.5f - 0.5f, cy = (y0 + y1) * 0.5f - 0.5f;
        f32 dx = cx * r.width * (2.5f + 2.0f * cu_hash(hs + 101u)) + (cu_hash(hs + 7u) - 0.5f) * 40.0f;
        f32 dy = cy * r.height * (2.5f + 2.0f * cu_hash(hs + 202u)) - 40.0f - 40.0f * cu_hash(hs + 9u);
        f32 rot = (cu_hash(hs + 303u) - 0.5f) * 540.0f;
        f32 e = cu_ease_out(m_minf(1.0f, q));
        f32 fall = q > 0.45f ? 140.0f * (q - 0.45f) / 0.55f : 0.0f;
        ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 46, .offset = {r.x, r.y}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                    .clip_path = cp,
                    .transform = cu_tf(dx * e, dy * e + fall, 1.0f, rot * e),
                    .opacity = opt_some(f32, m_clampf(1.0f - (q - 0.45f) / 0.55f, 0.0f, 1.0f)),
                    .brightness = opt_some(f32, 1.0f + 0.6f * (1.0f - e))}) {
          k.uid = 10000u + (u32)j * 8u + 1u + (u32)(gx * 3 + gy);
          cu_card(ui, k, in, ui_idi("Shard", (u32)(j * 6 + gx * 3 + gy)));
        }
      }
  }

  // ---- the dragged card follows the pointer, tilted with its speed ----
  if (dragging) {
    i32 ci = ui->drag;
    Clay_BoundingBox r = ui->box[ci];
    CardLook k = cu_look(b, ci, s, r.width > 0.0f ? r.width : hw);
    k.fx_n = 0;
    k.hide_cost = !ui->drag_from_hand;
    k.level = s->card_level[b->c[ci].id];
    k.uid = 9500u;
    ui->z = 71;
    f32 lift = ui->drag_over < 0 || ui->drag_over == 3 ? 1.08f : 0.92f;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 70, .offset = {r.x + ui->drag_dx, r.y + ui->drag_dy},
                             .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .transform = cu_tf(0, 0, lift, ui->drag_tilt),
                .shadow = {.color = cu_black(0.55f), .offset_y = 14.0f, .blur = 22.0f}}) {
      cu_card(ui, k, in, ui_id("DragCard"));
    }
    if (!ui->drag_from_hand) {
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 69, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                               .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                               .offset = {0.0f, -(hw * 1.4f + 46.0f)}},
                  .layout = {.padding = {.left = 14, .right = 14, .top = 6, .bottom = 6}},
                  .background_color = cu_black(0.7f), .corner_radius = ui_corner_radius(99)}) {
        cu_textb("Drop on your hand to take it back \xc2\xb7 or on another location to move it", 13, white, 600, false, none);
      }
    }
  }

  // ---- hints ----
  if (no_moves && ui->inspect == INSPECT_NONE) {
    f32 np = 0.5f + 0.5f * m_sinf(ui->time * M_PI);
    ui_element({.id = ui_id("Suggest"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 48,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0.0f, -(hw * 0.55f) - 3.0f * np}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8, .padding = {.left = 22, .right = 22, .top = 12, .bottom = 12},
                           .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
                .gradient = cu_lin2(180.0f, cu_rgb(0x3d9bff), cu_rgb(0x1655b0)),
                .border = {.color = white, .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
                .corner_radius = ui_corner_radius(99),
                .shadow = {.color = cu_rgba(0x3d9bff, 0.35f - 0.2f * np), .spread = 4.0f + 6.0f * np},
                .shadows = {{.color = cu_black(0.4f), .offset_y = 8.0f, .blur = 24.0f}}}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
        ui->click_end = true;
      cu_textb("\xe2\x9c\x93", 18, white, 800, false, none);
      cu_textb("No more moves \xe2\x80\x94", 17, white, 500, false, none);
      cu_textb("End Turn", 17, white, 800, false, none);
    }
  } else if (b->phase == PHASE_PLAN && b->turn == 1 && !cu_any_staged(b) && ui->selected < 0 && ui->inspect == INSPECT_NONE && !dragging) {
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 48, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0.0f, -(bottom_h + 8.0f)}},
                .layout = {.padding = {.left = 12, .right = 12, .top = 5, .bottom = 5}},
                .background_color = cu_black(0.65f), .corner_radius = ui_corner_radius(99)}) {
      cu_textb(phone ? "Tap a card, then a location to play it." : "Tap a card to see it, then pick a location to play it \xc2\xb7 or drag it there.", 14,
               white, 600, false, none);
    }
  }

  // ---- the reveal begins: who reveals first, big in the middle for a moment ----
  if (b->phase == PHASE_REVEAL && b->first_t < 1.4f) {
    f32 a = m_minf(1.0f, m_minf(b->first_t / 0.15f, (1.4f - b->first_t) / 0.3f));
    b32 me_first = b->first == B_SIDE_P;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 49, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_CENTER_CENTER}},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_alignment = {.x = UI_ALIGN_X_CENTER}, .child_gap = 2,
                           .padding = {.left = 26, .right = 26, .top = 10, .bottom = 12}},
                .background_color = me_first ? cu_rgba(0x1f6fd1, 0.92f * a) : cu_rgba(0x7b2fbe, 0.92f * a),
                .border = {.color = cu_rgba(0xffffff, 0.7f * a), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
                .corner_radius = ui_corner_radius(16)}) {
      cu_textb(me_first ? "You reveal first" : CU_FMT(ui, "% reveals first", fmt_cstr(b->kind == BATTLE_WILD ? "The wild Beast" : b->opponent)),
               phone ? 20 : 26, cu_rgba(0xffffff, a), 900, false, none);
      cu_textb("(winning more locations \xc2\xb7 a tie: a coin flip)", phone ? 11 : 13, cu_rgba(0xffffff, 0.8f * a), 500, false, none);
    }
  }

  // ---- the battle log ----
  if (ui->log_open) {
    ui_element({.id = ui_id("LogDrawer"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 50,
                             .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP}, .offset = {-8.0f, 52.0f}},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .padding = {.left = 12, .right = 12, .top = 10, .bottom = 10},
                           .sizing = {.width = ui_sizing_fixed(m_minf(320.0f, sw * 0.86f))}},
                .background_color = cu_rgba(0x101c24, 0.95f),
                .border = {.color = cu_rgba(0xffffff, 0.25f), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                .corner_radius = ui_corner_radius(14)}) {
      cu_textb("Battle log", 15, white, 800, false, none);
      for (i32 i = 0; i < b->log_n; i++)
        ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {
          ui_text(ui_str_c(CU_FMT(ui, "\xe2\x80\xa2 %", fmt_cstr(b->log[i]))),
                  ui_text_config({.fontSize = 12, .textColor = cu_rgba(0xffffff, 0.9f), .wrapMode = UI_TEXT_WRAP_WORDS}));
        }
    }
  }

  // ---- the big card or location on tap ----
  if (ui->inspect != INSPECT_NONE && b->phase != PHASE_OVER) {
    ui->z = 57;
    ui_element({.id = ui_id("Inspect"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 55},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)},
                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_black(0.65f),
                .backdrop_blur = 3.0f}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame && !ui_pointer_over(ui_id("InspPanel")))
        ui->click_insp_bg = true;
      if (ui->inspect == INSPECT_ZONE) {
        i32 z = ui->inspect_i;
        ZoneDef zd = zone_def(b->zone[z]);
        f32 zw = m_minf(sw * 0.88f, 380.0f), zh = 300.0f;
        ui_element({.id = ui_id("InspPanel"),
                    .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .padding = {.left = 20, .right = 20, .top = 28, .bottom = 22},
                               .sizing = {.width = ui_sizing_fixed(zw), .height = ui_sizing_fixed(zh)},
                               .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_BOTTOM}},
                    .background_color = b->zone_rev[z] ? zd.color : cu_rgb(0x555555),
                    .backgrounds = b->zone_rev[z] ? NULL : ui_backgrounds({.gradient = cu_dots(), .size = {22.0f, 22.0f}}),
                    .border = {.color = b->zone_rev[z] ? zd.color : cu_black(0.35f), .width = {.left = 4, .right = 4, .top = 4, .bottom = 4}, .overlay = true},
                    .corner_radius = ui_corner_radius(22),
                    .clip = {.horizontal = true, .vertical = true},
                    .shadow = {.color = cu_black(0.4f), .offset_y = 12.0f, .blur = 30.0f}}) {
          cu_zone_fill(ui, b, z, zw, zh, 22.0f, 56);
          ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 57, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH,
                                   .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM}, .offset = {0, -20}},
                      .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .sizing = {.width = ui_sizing_fixed(zw - 40.0f)},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
            cu_zone_texts(ui, b, z, 28, 17, true);
          }
        }
        cu_textb("Tap anywhere to close", 13, cu_rgba(0xffffff, 0.8f), 500, false, none);
      } else {
        i32 ci = ui->inspect_i;
        const BCard *c = &b->c[ci];
        b32 in_hand = ui->inspect == INSPECT_HAND && c->zone < 0;
        ui_element({.id = ui_id("InspPanel"),
                    .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 12, .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
          CardLook k = cu_look(b, ci, s, m_minf(m_minf(sw * 0.72f, sh * 0.42f), 290.0f));
          k.big = true;
          k.uid = 9000u + (u32)ci;
          k.fx_n = 0;
          k.pulse_t = 99.0f;
          if (in_hand)
            k.power = card_def(c->id).power + c->mod;
          k.level = c->owner == B_SIDE_P ? s->card_level[c->id] : 0;
          cu_card(ui, k, in, ui_id("InspCard"));
          ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .sizing = {.width = ui_sizing_fixed(m_minf(sw * 0.92f, 420.0f))},
                                 .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
            if (in_hand && b->phase == PHASE_PLAN) {
              if (card_def(c->id).cost > b->energy_left) {
                cu_textb(CU_FMT(ui, "Not enough energy (% needed)", fmt_i32(card_def(c->id).cost)), 15, cu_rgba(0xffffff, 0.9f), 500, false, none);
              } else {
                cu_textb("Play to:", 15, cu_rgba(0xffffff, 0.9f), 500, false, none);
                ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .sizing = {.width = ui_sizing_grow(0)}}}) {
                  for (i32 z = 0; z < 3; z++) {
                    HzUIColor zc = b->zone_rev[z] ? zone_def(b->zone[z]).color : cu_rgb(0x555555);
                    b32 can = b_can_play(b, ci, z);
                    HzUIElementId zid = ui_idi("PlayTo", (u32)z);
                    ui_element({.id = zid,
                                .layout = {.sizing = {.width = ui_sizing_grow(0)}, .padding = {.left = 6, .right = 6, .top = 10, .bottom = 10},
                                           .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                                .background_color = zc,
                                .border = {.color = cu_black(0.35f), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}},
                                .corner_radius = ui_corner_radius(12),
                                .shadow = {.color = cu_black(0.2f), .offset_y = -4.0f, .inset = true},
                                .opacity = can ? opt_none(f32) : opt_some(f32, 0.35f),
                                .brightness = can && ui_pointer_over(zid) ? opt_some(f32, 1.1f) : opt_none(f32)}) {
                      if (can && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
                        ui->click_play_to = z;
                      cu_textb(b->zone_rev[z] ? zone_def(b->zone[z]).name : CU_FMT(ui, "Location %", fmt_i32(z + 1)), 14, cu_ink(zc), 800, true,
                               cu_light(zc) ? none : cu_black(0.6f));
                    }
                  }
                }
              }
            } else if (ui->inspect == INSPECT_BOARD && c->owner == B_SIDE_P && c->staged && b->phase == PHASE_PLAN) {
              if (cu_btn(ui_id("InspBack"), "\xe2\x86\xba  Take back", BTN_GHOST, true, in, 16, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false, ui->time))
                ui->click_take_back = true;
            } else if (ui->inspect == INSPECT_BOARD && c->owner == B_SIDE_P && b_can_activate(b, ci) && b->phase == PHASE_PLAN) {
              if (cu_btn(ui_id("InspAct"), "\xe2\x96\xb6  Activate", BTN_GREEN, true, in, 17, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false, ui->time))
                ui->click_activate = true;
            } else if (ui->inspect == INSPECT_BOARD && c->owner == B_SIDE_P && c->act_order >= 0 && b->phase == PHASE_PLAN) {
              cu_textb("Activates when the turn is revealed, in the order you played", 14, cu_rgba(0xffffff, 0.9f), 500, true, none);
              if (cu_btn(ui_id("InspCancel"), "\xe2\x86\xba  Cancel activate", BTN_GHOST, true, in, 16, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false,
                         ui->time))
                ui->click_cancel_act = true;
            } else if (card_def(c->id).kind == KIND_ACTIVATE && c->activated) {
              cu_textb("Already activated", 14, cu_rgba(0xffffff, 0.9f), 500, false, none);
            }
            if (cu_btn(ui_id("InspClose"), "\xc3\x97  Close", BTN_GHOST, true, in, 16, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false, ui->time))
              ui->click_close_insp = true;
          }
        }
      }
    }
  }

  // ---- the result ----
  if (b->phase == PHASE_OVER && b->result != RESULT_RETREAT) {
    CString title = b->result == RESULT_WIN ? "VICTORY!" : b->result == RESULT_LOSE ? "DEFEAT" : b->result == RESULT_DRAW ? "DRAW" : "RETREATED";
    i32 delta = b->result == RESULT_WIN ? b->stakes : b->result == RESULT_LOSE ? -b->stakes : 0;
    ui_element({.id = ui_id("BattleEnd"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 60},
                .layout = {.sizing = {.width = ui_sizing_fixed(sw), .height = ui_sizing_fixed(sh)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = cu_black(0.6f)}) {
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 14, .padding = {.left = 34, .right = 34, .top = 24, .bottom = 24},
                             .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .gradient = cu_lin2(180.0f, cu_rgb(0x2a3a5a), cu_rgb(0x1b2a34)),
                  .border = {.color = cu_rgba(0xffffff, 0.3f), .width = {.left = 4, .right = 4, .top = 4, .bottom = 4}},
                  .corner_radius = ui_corner_radius(20)}) {
        cu_textb(title, phone ? 36 : 46, b->result == RESULT_WIN ? cu_rgb(0xfde047) : white, 900, false, none);
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = phone ? 16 : 24}}) {
          for (i32 z = 0; z < 3; z++) {
            i32 w = b_zone_winner(b, z);
            ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 2, .child_alignment = {.x = UI_ALIGN_X_CENTER}}}) {
              cu_textb(zone_def(b->zone[z]).name, 13, cu_rgba(0xffffff, 0.8f), 500, false, none);
              cu_textb(CU_FMT(ui, "% : %", fmt_i32(b_zone_power(b, z, B_SIDE_P)), fmt_i32(b_zone_power(b, z, B_SIDE_AI))), 24, white, 800, false, none);
              cu_textb(w == B_SIDE_P ? "\xe2\x9c\x93" : w == B_SIDE_AI ? "\xe2\x9c\x97" : "\xe2\x80\x93", 20,
                       w == B_SIDE_P ? cu_rgb(0x4ade80) : w == B_SIDE_AI ? cu_rgb(0xf87171) : cu_rgba(0xffffff, 0.6f), 900, false, none);
            }
          }
        }
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 8, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(20), .height = ui_sizing_fixed(20)}},
                      .gradient = cu_rad(0.35f, 0.3f, cu_rgb(0xfff3a8), cu_rgb(0xd9a300), 1.0f), .corner_radius = ui_corner_radius(10)}) {}
          cu_textb(CU_FMT(ui, "%% Studs", fmt_cstr(delta >= 0 ? "+" : ""), fmt_i32(delta)), 20, white, 700, false, none);
        }
        if (cu_btn(ui_id("BtnDone"), "Continue", BTN_BLUE, true, in, 20, ui_sizing_fit(0, 999), ui_sizing_fit(0, 999), false, ui->time))
          ui->click_done = true;
      }
    }
  }

  // ---- victory: LEGO brick confetti ----
  if (ui->confetti_t >= 0.0f && ui->confetti_t < 4.5f) {
    static const u32 cols[] = {0xc91a09, 0xf2cd37, 0x0055bf, 0x237841, 0xfe8a18, 0xf4f4f4, 0xff8ac0};
    for (u32 i = 0; i < 70; i++) {
      f32 delay = cu_hash(i * 5u + 1u) * 0.7f, dur = 1.8f + cu_hash(i * 5u + 2u) * 1.6f;
      f32 q = (ui->confetti_t - delay) / dur;
      if (q < 0.0f || q > 1.0f)
        continue;
      f32 e = q < 0.5f ? 2.0f * q * q : 1.0f - 2.0f * (1.0f - q) * (1.0f - q) * 0.5f - 0.0f;
      e = m_clampf(0.3f * q + 0.7f * e, 0.0f, 1.0f);
      f32 bw2 = cu_hash(i * 5u + 3u) < 0.5f ? 28.0f : 14.0f;
      f32 x = cu_hash(i * 5u + 4u) * sw + (cu_hash(i * 7u) - 0.5f) * 240.0f * e;
      f32 y = -40.0f + (sh * 1.1f + 40.0f) * e;
      f32 rot = (cu_hash(i * 11u) - 0.5f) * 900.0f * e;
      HzUIColor col = cu_rgb(cols[i % 7]);
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 90, .offset = {x, y}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .transform = cu_tf(0, 0, 1, rot)}) {
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 4}}) {
          for (i32 st = 0; st < (bw2 > 20.0f ? 2 : 1); st++)
            ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(7), .height = ui_sizing_fixed(4)}}, .background_color = cu_mix(col, white, 0.12f),
                        .corner_radius = {.top_left = 2, .top_right = 2}}) {}
        }
        ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(bw2), .height = ui_sizing_fixed(10)}}, .background_color = col,
                    .corner_radius = ui_corner_radius(2), .shadow = {.color = cu_black(0.2f), .offset_y = -3.0f, .inset = true},
                    .shadows = {{.color = cu_black(0.25f), .offset_y = 2.0f, .blur = 4.0f}}}) {}
      }
    }
  }
  (void)sh;
}

// main lane, before the scope: the clicks of the last UI frame on the battle, the drag, the keys (Enter, Escape)
hz_internal void battle_input(CardUi *ui, Battle *b, const InputSystem *in, SoundBank *snd) {
  f32 dpr = ui->dpr > 0.0f ? ui->dpr : 1.0f;
  f32 mx = in->mouse_pos.x / dpr, my = in->mouse_pos.y / dpr;
  // ---- the big card's buttons ----
  if (ui->inspect != INSPECT_NONE) {
    i32 ci = ui->inspect_i;
    b32 close = false;
    if (ui->click_play_to >= 0 && ui->inspect == INSPECT_HAND) {
      if (b_stage(b, ci, ui->click_play_to)) {
        ui->selected = -1;
        sound_play(snd, SND_POP, 0.8f, 0.6f);
      }
      close = true;
    } else if (ui->click_take_back) {
      b_unstage(b, ci);
      close = true;
    } else if (ui->click_activate) {
      b_activate(b, ci);
      sound_play(snd, SND_ALERT, 1.3f, 0.4f);
      close = true;
    } else if (ui->click_cancel_act) {
      b_cancel_activate(b, ci);
      close = true;
    } else if (ui->click_close_insp || ui->click_insp_bg || in->buttons[KEY_ESCAPE].pressed_this_frame) {
      close = true;
    }
    if (close)
      ui->inspect = INSPECT_NONE;
    ui->click_play_to = -1;
    ui->click_take_back = ui->click_activate = ui->click_cancel_act = ui->click_close_insp = ui->click_insp_bg = false;
    ui->click_card = ui->click_zone = ui->press_card = -1;
    ui->click_end = ui->click_snap = ui->click_retreat = ui->click_undo = ui->click_log = false;
    return;
  }
  // ---- drag and drop: hand cards (play them) and the cards played this turn (move them or take them back) ----
  if (ui->press_card >= 0 && b->phase == PHASE_PLAN && ui->drag < 0) {
    ui->drag = ui->press_card;
    ui->drag_moved = false;
    ui->drag_from_hand = b->c[ui->press_card].zone < 0;
    ui->drag_x0 = mx;
    ui->drag_y0 = my;
    ui->drag_dx = ui->drag_dy = ui->drag_tilt = 0.0f;
    ui->drag_lastx = mx;
    ui->drag_over = -1;
  }
  ui->press_card = -1;
  if (ui->drag >= 0) {
    i32 ci = ui->drag;
    // follow the finger only while it is down: on the frame it lifts, the pointer can jump (a touch's end restores the
    // last mouse position), so the drop goes where the finger was last seen
    if (in->buttons[MOUSE_LEFT].is_pressed) {
      ui->drag_dx = mx - ui->drag_x0;
      ui->drag_dy = my - ui->drag_y0;
      f32 vx = mx - ui->drag_lastx;
      ui->drag_lastx = mx;
      ui->drag_tilt = m_clampf(ui->drag_tilt * 0.75f + vx * 0.6f, -14.0f, 14.0f);
      b32 can_drag = !ui->drag_from_hand || cu_playable(b, ci);
      if (!ui->drag_moved && can_drag && m_sqrtf(ui->drag_dx * ui->drag_dx + ui->drag_dy * ui->drag_dy) > 8.0f) {
        ui->drag_moved = true;
        if (ui->drag_from_hand)
          ui->selected = ci; // light up the locations it can go to
        sound_play(snd, SND_BLIP, 1.3f, 0.3f);
      }
      if (ui->drag_moved) {
        ui->drag_over = -1;
        for (i32 z = 0; z < 3 && ui->drag_over < 0; z++) {
          Clay_BoundingBox r1 = cu_box_of(ui_idi("ZoneTile", (u32)z)), r2 = cu_box_of(ui_idi("MySlots", (u32)z));
          if ((mx >= r1.x && mx <= r1.x + r1.width && my >= r1.y && my <= r1.y + r1.height) ||
              (mx >= r2.x && mx <= r2.x + r2.width && my >= r2.y && my <= r2.y + r2.height))
            ui->drag_over = z;
        }
        Clay_BoundingBox hb = cu_box_of(ui_id("BattleBottom"));
        if (ui->drag_over < 0 && my >= hb.y && hb.height > 0.0f)
          ui->drag_over = 3;
      }
    }
    if (!in->buttons[MOUSE_LEFT].is_pressed) {
      if (!ui->drag_moved) { // a tap: the card big (a hand card is also selected)
        ui->inspect = ui->drag_from_hand ? INSPECT_HAND : INSPECT_BOARD;
        ui->inspect_i = ci;
        if (ui->drag_from_hand)
          ui->selected = b->phase == PHASE_PLAN ? ci : -1;
        sound_play(snd, SND_BLIP, 1.2f, 0.35f);
      } else {
        b32 placed = false;
        i32 t = ui->drag_over;
        if (ui->drag_from_hand) {
          if (t >= 0 && t < 3 && b_stage(b, ci, t))
            placed = true;
          ui->selected = -1;
        } else {
          i32 orig = b->c[ci].zone;
          if (t >= 0 && t < 3 && t != orig) {
            b_unstage(b, ci);
            if (b_stage(b, ci, t))
              placed = true;
            else
              b_stage(b, ci, orig); // not allowed there: back where it was
          } else if (t == 3) {
            b_unstage(b, ci);
            placed = true;
          }
        }
        if (placed)
          sound_play(snd, SND_POP, 0.8f, 0.6f);
      }
      ui->drag = -1;
      ui->drag_moved = false;
    }
  }
  // ---- taps on the board ----
  if (ui->click_card >= 0) {
    i32 ci = ui->click_card;
    const BCard *c = &b->c[ci];
    if (c->owner == B_SIDE_P && ui->selected >= 0 && c->zone >= 0 && b_can_play(b, ui->selected, c->zone)) {
      if (b_stage(b, ui->selected, c->zone))
        sound_play(snd, SND_POP, 0.8f, 0.6f);
      ui->selected = -1;
    } else if (c->revealed || c->owner == B_SIDE_P) {
      ui->inspect = INSPECT_BOARD;
      ui->inspect_i = ci;
    }
  } else if (ui->click_zone >= 0) {
    i32 z = ui->click_zone;
    if (ui->selected >= 0 && b_can_play(b, ui->selected, z)) {
      b_stage(b, ui->selected, z);
      sound_play(snd, SND_POP, 0.8f, 0.6f);
      ui->selected = -1;
    } else if (ui->drag < 0) {
      ui->inspect = INSPECT_ZONE;
      ui->inspect_i = z;
    }
  }
  if (ui->click_undo) {
    for (i32 i = 0; i < b->n; i++)
      if (b->c[i].staged)
        b_unstage(b, i);
  }
  if (ui->click_log)
    ui->log_open = !ui->log_open;
  if (in->buttons[KEY_ESCAPE].pressed_this_frame) {
    if (ui->log_open)
      ui->log_open = false;
    else
      ui->selected = -1;
  }
  if ((ui->click_end || in->buttons[KEY_ENTER].pressed_this_frame) && b->phase == PHASE_PLAN && ui->drag < 0) {
    ui->selected = -1;
    b_end_turn(b);
  }
  if (ui->click_snap)
    b_snap(b);
  if (ui->click_retreat)
    b_retreat(b);
  ui->click_card = ui->click_zone = -1;
  ui->click_end = ui->click_snap = ui->click_retreat = ui->click_undo = ui->click_log = false;
}


// ---------------------------------------------------------------- the deck and the collection

// a full-width button whose label wraps (and an optional smaller second line), so long labels always fit
hz_internal b32 deck_button(HzUIElementId id, CString label, CString sub, HzUIColor bg, b32 enabled, const InputSystem *in, f32 font) {
  b32 clicked = false;
  ui_element({.id = id,
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 1, .padding = {.left = 10, .right = 10, .top = 8, .bottom = 8},
                         .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
              .background_color = enabled ? bg : (HzUIColor){150, 154, 160, 255},
              .corner_radius = ui_corner_radius(9),
              .brightness = enabled && ui_hovered() ? opt_some(f32, 1.12f) : opt_none(f32)}) {
    if (enabled && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {
      cu_textb(label, font, (HzUIColor){255, 255, 255, enabled ? 255 : 210}, 700, true, (HzUIColor){0});
    }
    if (sub)
      ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {
        cu_textb(sub, font * 0.8f, (HzUIColor){255, 255, 255, enabled ? 220 : 190}, 500, true, (HzUIColor){0});
      }
  }
  return clicked;
}

// the picked card: its text, stage and what you can do with it (the side panel, or the sheet on phones)
hz_internal void deck_actions(CardUi *ui, GameSave *s, const InputSystem *in, i32 c) {
  HzUIColor dim = {90, 94, 100, 255};
  CardDef d = card_def(c);
  i32 lv = s->card_level[c];
  ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { cu_text_wrap(d.text, 14, (HzUIColor){27, 42, 52, 255}); }
  cstr_format(ui->text[3], 96, "Stage: % \xe2\x80\x94 %", fmt_cstr(stage_def(lv).name), fmt_cstr(stage_def(lv).blurb));
  ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { cu_text_wrap(ui->text[3], 13, dim); }
  u32 cost = upgrade_cost(s, c);
  if (cost) {
    cstr_format(ui->text[4], 96, "Upgrade to %", fmt_cstr(stage_def(lv + 1).name));
    cstr_format(ui->text[5], 96, s->exp >= cost ? "% EXP" : "% EXP (you have %)", fmt_u32(cost), fmt_u32(s->exp));
    if (deck_button(ui_id("BtnUpgrade"), ui->text[4], ui->text[5], (HzUIColor){120, 60, 200, 255}, s->exp >= cost, in, 14))
      ui->click_upgrade = true;
  } else if (!can_become_voxel(c)) {
    ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { cu_text_wrap("Final stage. Only 6-energy cards become a Voxelmon.", 12, dim); }
  }
  if (lv >= VOXEL_LEVEL) {
    b32 walking = s->voxelmon == c + 1;
    if (deck_button(ui_id("BtnWalk"), walking ? "Walks with you" : "Walk with me", NULL, (HzUIColor){60, 160, 80, 255}, !walking, in, 14))
      ui->click_walk = true;
  }
  b32 in_deck = false;
  for (i32 i = 0; i < s->deck_count; i++)
    in_deck |= s->deck[i] == c;
  if (in_deck) {
    if (deck_button(ui_id("BtnRemove"), "Remove from deck", NULL, (HzUIColor){180, 70, 50, 255}, true, in, 14))
      ui->click_remove = true;
  } else if (deck_button(ui_id("BtnAdd"), "Add to deck", s->deck_count < DECK_SIZE ? NULL : "The deck is full", (HzUIColor){0, 85, 191, 255},
                         s->deck_count < DECK_SIZE, in, 14)) {
    ui->click_add = true;
  }
}

// The deck screen: the header and the deck strip are panels of their own over the scrolling collection (the cards'
// floating parts can never show through them); on a narrow screen (phones) the collection takes the whole width,
// the deck wraps into two rows and the picked card opens as a sheet from the bottom. The x always closes it.
hz_internal void deck_draw(CardUi *ui, GameSave *s, const InputSystem *in, f32 sw, f32 sh, b32 voxel_walks) {
  HzUIColor ink = {27, 42, 52, 255}, paper = {206, 199, 185, 255}; // a darker sand: the cards stand out from it
  OsSafeAreaInsets safe = os_get_safe_area();
  b32 narrow = sw < 640.0f;
  f32 pad = 12.0f;
  f32 side_w = narrow ? 0.0f : m_minf(280.0f, sw * 0.32f);
  f32 coll_w = sw - pad * 2.0f - safe.left - safe.right - (narrow ? 0.0f : side_w + 12.0f);
  i32 per_row = narrow ? 4 : (i32)m_maxf(4.0f, m_floorf((coll_w + 8.0f) / 92.0f));
  f32 gw = m_clampf((coll_w - 8.0f * (f32)(per_row - 1)) / (f32)per_row - 1.0f, 52.0f, 84.0f);
  i32 cols = narrow ? 6 : 12;
  f32 strip_inner = sw - pad * 2.0f - safe.left - safe.right - 16.0f;
  f32 sw_card = m_minf(narrow ? 80.0f : 64.0f, (strip_inner - 6.0f * (f32)(cols - 1)) / (f32)cols);
  HzUIElementData hd = ui_get_element_data(ui_id("DeckHeader")), sd = ui_get_element_data(ui_id("DeckStrip"));
  f32 head_h = hd.found ? hd.boundingBox.height : 70.0f, strip_h = sd.found ? sd.boundingBox.height : 120.0f;
  ui->z = 1;
  ui_element({.id = ui_id("DeckScreen"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .padding = {.left = (u16)(pad + safe.left), .right = (u16)(pad + safe.right), .top = (u16)(head_h + 8.0f), .bottom = (u16)(strip_h + 8.0f)},
                         .child_gap = 12},
              .background_color = paper}) {
    // the collection: every card, the ones you do not have as dark "???"
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                .clip = {.vertical = true, .child_offset = ui_scroll_offset()}}) {
      cu_text(narrow ? "Collection (tap a card)" : "Collection (click a card)", 14, ink);
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 8, .sizing = {.width = ui_sizing_grow(0)}}}) {
        for (i32 c = 0; c < CARD_COUNT - 1; c++) {
          b32 have = owns(s, c);
          if (c == CARD_GOLDBRICK && !have)
            continue;
          CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = gw, .unknown = !have,
                        .lit = ui->deck_pick == c, .uid = 500u + (u32)c};
          b32 in_deck = false;
          for (i32 i = 0; i < s->deck_count; i++)
            in_deck |= s->deck[i] == c;
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(gw)}}}) {
            if (cu_card(ui, k, in, ui_idi("Coll", (u32)c)) && have)
              ui->click_coll = c;
            if (in_deck) { // in your deck: a green frame and a tick in the corner
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 4, .offset = {-3.0f, -3.0f},
                                       .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                          .layout = {.sizing = {.width = ui_sizing_fixed(gw + 6.0f), .height = ui_sizing_fixed(gw * 1.4f + 6.0f)}},
                          .corner_radius = ui_corner_radius(gw * 0.08f),
                          .border = {.color = cu_rgb(0x27b05a), .width = {.left = 3, .right = 3, .top = 3, .bottom = 3}}}) {}
              f32 d = m_clampf(gw * 0.3f, 18.0f, 26.0f);
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 5,
                                       .attach_points = {.element = UI_ATTACH_POINT_CENTER_CENTER, .parent = UI_ATTACH_POINT_RIGHT_BOTTOM},
                                       .offset = {-d * 0.25f, -d * 0.25f}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                          .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)},
                                     .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                          .background_color = cu_rgb(0x27b05a),
                          .corner_radius = ui_corner_radius(d * 0.5f),
                          .border = {.color = cu_rgb(0xffffff), .width = {.left = 2, .right = 2, .top = 2, .bottom = 2}},
                          .shadow = {.color = cu_black(0.35f), .offset_y = 1.0f, .blur = 3.0f}}) {
                cu_textb("\xe2\x9c\x93", d * 0.62f, cu_rgb(0xffffff), 800, false, (HzUIColor){0}); // a tick
              }
            }
          }
        }
      }
      ui_element({.layout = {.sizing = {.height = ui_sizing_fixed(narrow && ui->deck_pick >= 0 ? sh * 0.4f : 8.0f)}}}) {} // room to scroll past the sheet
    }
    if (!narrow) { // the picked card big, with what you can do
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .padding = ui_padding_all(12),
                             .sizing = {.width = ui_sizing_fixed(side_w)}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                  .background_color = {255, 255, 255, 255},
                  .corner_radius = ui_corner_radius(12),
                  .clip = {.vertical = true}}) {
        i32 c = ui->deck_pick;
        if (c >= 0) {
          CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = m_minf(170.0f, side_w * 0.62f), .uid = 900u};
          cu_card(ui, k, in, ui_id("DeckBig"));
          deck_actions(ui, s, in, c);
        } else {
          ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {
            cu_text_wrap("Pick a card on the left to see it big, upgrade it with EXP or change your deck.", 14, ink);
          }
        }
      }
    }
  }
  // the header: title, counts, and the way out
  ui_element({.id = ui_id("DeckHeader"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 12},
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 12, .sizing = {.width = ui_sizing_fixed(sw)},
                         .padding = {.left = (u16)(pad + safe.left), .right = (u16)(pad + safe.right), .top = (u16)(10.0f + safe.top), .bottom = 10},
                         .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = paper,
              .shadow = {.color = cu_black(0.18f), .offset_y = 2.0f, .blur = 8.0f}}) {
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 2, .sizing = {.width = ui_sizing_grow(0)}}}) {
      cu_textb("Your cards", narrow ? 20 : 24, ink, 800, false, (HzUIColor){0});
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT_WRAP, .child_gap = 12, .sizing = {.width = ui_sizing_grow(0)}}}) {
        cstr_format(ui->text[0], 96, "% / % collected", fmt_u32(owned_count(s)), fmt_i32(CARD_POOL));
        cu_text(ui->text[0], 14, ink);
        cstr_format(ui->text[1], 96, "Deck % / %", fmt_i32(s->deck_count), fmt_i32(DECK_SIZE));
        cu_text(ui->text[1], 14, s->deck_count == DECK_SIZE ? (HzUIColor){30, 130, 60, 255} : (HzUIColor){200, 60, 40, 255});
        cstr_format(ui->text[2], 96, "% EXP", fmt_u32(s->exp));
        cu_text(ui->text[2], 14, (HzUIColor){120, 60, 200, 255});
      }
    }
    ui_element({.id = ui_id("DeckClose"),
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6, .padding = {.left = 12, .right = 14, .top = 8, .bottom = 8},
                           .sizing = {.height = ui_sizing_fixed(42)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                .background_color = (HzUIColor){60, 64, 72, 255},
                .corner_radius = ui_corner_radius(21),
                .brightness = ui_hovered() ? opt_some(f32, 1.2f) : opt_none(f32)}) {
      if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
        ui->click_close = true;
      cu_textb("\xc3\x97", 24, (HzUIColor){255, 255, 255, 255}, 700, false, (HzUIColor){0});
      cu_textb(narrow ? "Close" : "Close (C)", 15, (HzUIColor){255, 255, 255, 255}, 700, false, (HzUIColor){0});
    }
  }
  // the deck: 12 slots (two rows of six on phones)
  ui->z = 13;
  ui_element({.id = ui_id("DeckStrip"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 12,
                           .attach_points = {.element = UI_ATTACH_POINT_LEFT_BOTTOM, .parent = UI_ATTACH_POINT_LEFT_BOTTOM}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .sizing = {.width = ui_sizing_fixed(sw)},
                         .padding = {.left = (u16)(pad + safe.left + 8.0f), .right = (u16)(pad + safe.right + 8.0f), .top = 8, .bottom = (u16)(10.0f + safe.bottom)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER}},
              .background_color = {40, 44, 52, 255},
              .shadow = {.color = cu_black(0.3f), .offset_y = -2.0f, .blur = 10.0f}}) {
    for (i32 row = 0; row * cols < DECK_SIZE; row++)
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 6}}) {
        for (i32 i = row * cols; i < (row + 1) * cols && i < DECK_SIZE; i++) {
          if (i < s->deck_count) {
            i32 c = s->deck[i];
            CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = sw_card, .uid = 700u + (u32)i,
                          .lit = ui->deck_pick == c};
            if (cu_card(ui, k, in, ui_idi("Slot", (u32)i)))
              ui->click_slot = i;
          } else {
            ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(sw_card), .height = ui_sizing_fixed(sw_card * 1.4f)}},
                        .background_color = {70, 74, 82, 255},
                        .corner_radius = ui_corner_radius(6)}) {}
          }
        }
      }
  }
  // phones: the picked card as a sheet over the deck strip
  if (narrow && ui->deck_pick >= 0) {
    i32 c = ui->deck_pick;
    ui->z = 21;
    ui_element({.id = ui_id("DeckSheet"),
                .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 20,
                             .attach_points = {.element = UI_ATTACH_POINT_LEFT_BOTTOM, .parent = UI_ATTACH_POINT_LEFT_BOTTOM}},
                .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 12, .sizing = {.width = ui_sizing_fixed(sw)},
                           .padding = {.left = (u16)(14.0f + safe.left), .right = (u16)(14.0f + safe.right), .top = 16, .bottom = (u16)(16.0f + safe.bottom)}},
                .background_color = {255, 255, 255, 255},
                .corner_radius = {.top_left = 18, .top_right = 18},
                .shadow = {.color = cu_black(0.35f), .offset_y = -4.0f, .blur = 20.0f}}) {
      CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = m_minf(120.0f, sw * 0.3f), .uid = 900u};
      cu_card(ui, k, in, ui_id("DeckBig"));
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 8, .sizing = {.width = ui_sizing_grow(0)}}}) {
        ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}}}) {
          ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) { cu_textb(card_def(c).name, 18, ink, 800, false, (HzUIColor){0}); }
          ui_element({.id = ui_id("DeckSheetClose"),
                      .layout = {.sizing = {.width = ui_sizing_fixed(34), .height = ui_sizing_fixed(34)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
                      .background_color = {226, 230, 234, 255},
                      .corner_radius = ui_corner_radius(17)}) {
            if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
              ui->click_unpick = true;
            cu_textb("\xc3\x97", 20, ink, 700, false, (HzUIColor){0});
          }
        }
        deck_actions(ui, s, in, c);
      }
    }
  }
  ui->z = 0;
  (void)voxel_walks;
}

// ---------------------------------------------------------------- the upgrade moment
// Every upgrade shows the card big in a spotlight: light gathers, the card flips and comes back in its new frame with
// rays and sparkles. A 6-energy card that becomes a Voxelmon shakes, glows and bursts into voxel cubes instead, the
// screen opens onto the town and the real 3D beast rises spinning out of a glowing ring (vox_showcase, st_vox.c).
#define UP_FLIP 1.15f  // the card turns edge-on
#define UP_SWAP 1.3f   // ... and comes back in its new frame
#define UP_BURST 1.25f // a Voxelmon: the card bursts, the 3D beast appears
#define UP_DONE 2.0f   // the button appears (a Voxelmon: UP_DONE + 0.7)

hz_internal f32 up_ease_back(f32 x) {
  x = m_clampf(x, 0.0f, 1.0f) - 1.0f;
  return 1.0f + 2.7f * x * x * x + 1.7f * x * x;
}

hz_internal void up_draw(CardUi *ui, const GameSave *s, const InputSystem *in, f32 sw, f32 sh) {
  if (ui->up_card < 0)
    return;
  i32 c = ui->up_card;
  f32 t = ui->up_t;
  b32 vox = ui->up_to >= VOXEL_LEVEL;
  StageDef st = stage_def(ui->up_to);
  HzUIColor sc = st.frame, white = {255, 255, 255, 255};
  f32 cw = m_minf(m_minf(sw * 0.42f, sh * 0.4f / 1.4f), 230.0f), ch = cw * 1.4f;
  f32 cx = sw * 0.5f, cy = sh * (vox ? 0.45f : 0.42f);
  // the backdrop: a dark spotlight; for a Voxelmon it opens onto the town once the card has burst
  f32 bg = m_clampf(t / 0.3f, 0.0f, 1.0f);
  if (vox)
    bg *= 1.0f - 0.85f * m_clampf((t - UP_BURST) / 0.45f, 0.0f, 1.0f);
  ui_element({.id = ui_id("UpShade"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 30},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
              .gradient = cu_rad(0.5f, 0.42f, cu_shade(0x2a3546, 0.9f * bg), cu_shade(0x05070b, 0.97f * bg), 1.0f)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame && t > (vox ? UP_DONE + 0.7f : UP_DONE))
      ui->click_up_ok = true; // a tap anywhere once it is done
  }
  // rays turning behind the card, brighter once it has changed
  f32 ray_a = m_clampf((t - 0.2f) / 0.5f, 0.0f, 1.0f) * (t > UP_SWAP ? 0.24f : 0.1f) * (vox ? 1.0f - m_clampf((t - UP_BURST) / 0.4f, 0.0f, 0.7f) : 1.0f);
  for (i32 i = 0; i < 6; i++) {
    f32 len = m_maxf(sw, sh) * 1.2f;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 31, .offset = {cx - cw * 0.16f, cy - len * 0.5f},
                             .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(cw * 0.32f), .height = ui_sizing_fixed(len)}},
                .gradient = cu_lin(0.0f, 3, (f32[]){0.0f, 0.5f, 1.0f}, (HzUIColor[]){cu_alpha(sc, 0.0f), cu_alpha(sc, ray_a), cu_alpha(sc, 0.0f)}),
                .transform = cu_tf(0, 0, 1, (f32)i * 30.0f + t * 18.0f)}) {}
  }
  // light gathers: sparks spiral in towards the card
  if (t > 0.25f && t < UP_SWAP) {
    for (i32 i = 0; i < 16; i++) {
      f32 p = m_clampf((t - 0.25f - 0.04f * (f32)(i % 6)) / 0.9f, 0.0f, 1.0f);
      f32 r = (1.0f - p) * cw * 1.6f + cw * 0.2f, a = (f32)i * 0.3927f + t * 3.0f;
      f32 d = 6.0f + 6.0f * (1.0f - p);
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 33, .offset = {cx + m_cosf(a) * r - d * 0.5f, cy + m_sinf(a) * r - d * 0.5f},
                               .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)}},
                  .background_color = i % 3 ? cu_rgba(0xffffff, 0.9f * p) : cu_alpha(sc, p),
                  .shadow = {.color = cu_alpha(sc, 0.8f * p), .blur = d},
                  .transform = cu_tf(0, 0, 1, 45.0f)}) {}
    }
  }
  // the card: in, a little wobble while the light gathers, the flip (a Voxelmon shakes harder and bursts)
  b32 show_card = !vox || t < UP_BURST;
  if (show_card) {
    f32 in_k = cu_ease_out(m_clampf(t / 0.35f, 0.0f, 1.0f));
    f32 sx = 1.0f, rot = 0.0f, scale = 0.6f + 0.4f * in_k, glow = m_clampf((t - 0.3f) / 0.8f, 0.0f, 1.0f);
    i32 level = ui->up_from;
    if (!vox) {
      rot = m_sinf(t * 14.0f) * 2.0f * glow * (t < UP_FLIP ? 1.0f : 0.0f);
      if (t > UP_FLIP - 0.15f && t < UP_SWAP)
        sx = m_maxf(0.02f, 1.0f - (t - (UP_FLIP - 0.15f)) / 0.3f);
      if (t >= UP_SWAP) {
        level = ui->up_to;
        f32 q = m_clampf((t - UP_SWAP) / 0.3f, 0.0f, 1.0f);
        sx = m_maxf(0.02f, q);
        scale *= 1.0f + 0.12f * m_sinf(m_clampf((t - UP_SWAP) / 0.45f, 0.0f, 1.0f) * M_PI);
      }
    } else {
      f32 shake = glow * glow;
      rot = m_sinf(t * 47.0f) * 4.0f * shake;
      scale *= 1.0f + 0.08f * shake;
      level = t > 0.9f ? VOXEL_LEVEL - 1 : ui->up_from;
    }
    ui->z = 35;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 34, .offset = {cx - cw * 0.5f, cy - ch * 0.5f},
                             .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.sizing = {.width = ui_sizing_fixed(cw), .height = ui_sizing_fixed(ch)}},
                .corner_radius = ui_corner_radius(cw * 0.06f),
                .shadow = {.color = cu_alpha(vox ? cu_rgb(0xb07aff) : sc, 0.9f * glow), .blur = cw * (0.15f + 0.25f * glow), .spread = cw * 0.02f * glow},
                .transform = opt_some(HzUITransform, ((HzUITransform){.scale = {scale * sx, scale}, .rotation = rot, .origin = {0.5f, 0.5f}})),
                .opacity = opt_some(f32, in_k)}) {
      CardLook k = {.id = c, .power = card_def(c).power, .level = level, .w = cw, .uid = 950u, .big = true};
      cu_card(ui, k, in, ui_id("UpCard"));
    }
    ui->z = 0;
  }
  // a Voxelmon: the card bursts into voxel cubes
  if (vox && t >= UP_BURST && t < UP_BURST + 1.1f) {
    f32 q = (t - UP_BURST) / 1.1f;
    static const u32 pal[6] = {0xa05aff, 0xffd23d, 0xffffff, 0x3d9bff, 0xff6a5a, 0x5ccb4c};
    for (i32 i = 0; i < 48; i++) {
      f32 gx = (f32)(i % 6) / 5.0f - 0.5f, gy = (f32)(i / 6) / 7.0f - 0.5f;
      f32 h1 = cu_hash((u32)i), h2 = cu_hash((u32)i * 7u + 3u);
      f32 vx = gx * 2.2f + (h1 - 0.5f) * 1.4f, vy = gy * 2.2f - 0.6f - h2 * 0.8f;
      f32 px = cx + gx * cw + vx * cw * q * 1.6f, py = cy + gy * ch + vy * cw * q * 1.6f + q * q * cw * 1.4f;
      f32 d = cw * (0.13f + 0.05f * h2) * (1.0f - 0.4f * q);
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 36, .offset = {px - d * 0.5f, py - d * 0.5f},
                               .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                  .layout = {.sizing = {.width = ui_sizing_fixed(d), .height = ui_sizing_fixed(d)}},
                  .gradient = cu_lin2(135.0f, cu_mix(cu_rgb(pal[i % 6]), white, 0.35f), cu_rgb(pal[i % 6])),
                  .corner_radius = ui_corner_radius(d * 0.12f),
                  .border = {.color = cu_black(0.25f), .width = {.left = 1, .right = 1, .top = 1, .bottom = 1}},
                  .opacity = opt_some(f32, 1.0f - q * q),
                  .transform = cu_tf(0, 0, 1, (h1 - 0.5f) * 720.0f * q)}) {}
    }
  }
  // the flash: when the card changes, or bursts
  f32 at = vox ? UP_BURST : UP_SWAP;
  if (t > at - 0.05f && t < at + 0.35f) {
    f32 fl = t < at ? (t - at + 0.05f) / 0.05f : 1.0f - (t - at) / 0.35f;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 37},
                .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
                .background_color = cu_rgba(0xffffff, 0.55f * m_clampf(fl, 0.0f, 1.0f)),
                .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {}
  }
  // the words: the new stage (or the Voxelmon, its skills), then the button
  f32 tt = vox ? UP_BURST + 0.6f : UP_SWAP + 0.2f;
  if (t > tt) {
    f32 pop = up_ease_back(m_clampf((t - tt) / 0.35f, 0.0f, 1.0f));
    f32 y = vox ? sh * 0.06f : cy + ch * 0.5f + 18.0f;
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 38,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_TOP, .parent = UI_ATTACH_POINT_CENTER_TOP},
                             .offset = {0, y}, .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 6, .child_alignment = {.x = UI_ALIGN_X_CENTER},
                           .sizing = {.width = ui_sizing_fixed(m_minf(sw - 24.0f, 560.0f))}},
                .opacity = opt_some(f32, m_minf(1.0f, pop)),
                .transform = cu_tf(0, 0, 0.6f + 0.4f * pop, 0)}) {
      HzUITextConfig big = {.fontSize = m_clampf(sw / 13.0f, 26.0f, 44.0f), .textColor = vox ? cu_rgb(0xd9b8ff) : cu_mix(sc, white, 0.25f),
                            .weight = 800, .textAlignment = CLAY_TEXT_ALIGN_CENTER, .wrapMode = UI_TEXT_WRAP_WORDS};
      big.shadows[0] = (HzUITextShadow){.color = cu_black(0.6f), .offset_y = 2.0f, .blur = 6.0f};
      HzUITextConfig note = {.fontSize = 16, .textColor = cu_rgba(0xffffff, 0.92f), .weight = 600, .textAlignment = CLAY_TEXT_ALIGN_CENTER,
                              .wrapMode = UI_TEXT_WRAP_WORDS};
      note.shadows[0] = (HzUITextShadow){.color = cu_black(0.7f), .offset_y = 1.0f, .blur = 4.0f};
      if (vox) {
        ui_text(ui_str_c(CU_FMT(ui, "% is a real Voxelmon now!", fmt_cstr(card_def(c).name))), big);
        ui_text(ui_str_c("It walks with you. Show it to Prof. Quill at the village exit!"), note);
      } else {
        ui_text(ui_str_c(CU_FMT(ui, "%!", fmt_cstr(st.name))), big);
        ui_text(ui_str_c(CU_FMT(ui, "% \xc2\xb7 %", fmt_cstr(card_def(c).name), fmt_cstr(st.blurb))), note);
      }
    }
  }
  f32 done = vox ? UP_DONE + 0.7f : UP_DONE;
  if (t > done) {
    f32 pop = up_ease_back(m_clampf((t - done) / 0.3f, 0.0f, 1.0f));
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 39,
                             .attach_points = {.element = UI_ATTACH_POINT_CENTER_BOTTOM, .parent = UI_ATTACH_POINT_CENTER_BOTTOM},
                             .offset = {0, -os_get_safe_area().bottom - sh * 0.07f}},
                .transform = cu_tf(0, 0, pop, 0)}) {
      if (cu_button(ui_id("UpOk"), vox ? "Let's go!" : "Nice!", vox ? cu_rgb(0x7a3fd6) : cu_rgb(0x0055bf), true, in, 20))
        ui->click_up_ok = true;
    }
  }
}

// the clicks of the deck screen; returns true when it was closed
hz_internal b32 deck_input(CardUi *ui, GameSave *s, SaveState *save, SoundBank *snd, const InputSystem *in) {
  if (ui->up_card >= 0) { // the upgrade moment owns the screen: once done, a tap, its button or a key goes back to the deck
    b32 done = ui->up_t > (ui->up_to >= VOXEL_LEVEL ? UP_DONE + 0.7f : UP_DONE);
    b32 keys = in->buttons[KEY_ENTER].pressed_this_frame || in->buttons[KEY_SPACE].pressed_this_frame || in->buttons[KEY_E].pressed_this_frame ||
               in->buttons[KEY_ESCAPE].pressed_this_frame || in->buttons[KEY_C].pressed_this_frame;
    if (done && (ui->click_up_ok || keys))
      ui->up_card = -1;
    ui->click_up_ok = false;
    ui->click_coll = ui->click_slot = -1;
    ui->click_add = ui->click_remove = ui->click_upgrade = ui->click_walk = ui->click_close = ui->click_unpick = false;
    return false;
  }
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
  if (c >= 0 && ui->click_upgrade && upgrade(s, c)) { // the moment plays (up_draw; its sounds in cg_update)
    ui->up_card = c;
    ui->up_from = s->card_level[c] - 1;
    ui->up_to = s->card_level[c];
    ui->up_t = 0.0f;
    save_mark(save);
  }
  if (c >= 0 && ui->click_walk && s->card_level[c] >= VOXEL_LEVEL) {
    s->voxelmon = c + 1;
    save_mark(save);
  }
  if (ui->click_unpick)
    ui->deck_pick = -1; // the sheet's x (phones)
  if (ui->click_close || in->buttons[KEY_C].pressed_this_frame || in->buttons[KEY_ESCAPE].pressed_this_frame)
    closed = true;
  ui->click_coll = ui->click_slot = -1;
  ui->click_add = ui->click_remove = ui->click_upgrade = ui->click_walk = ui->click_close = ui->click_unpick = false;
  return closed;
}

// ---------------------------------------------------------------- rewards: foil packs to tear open

#define PACK_SEALED -100.0f // reward_tear of a pack nobody has opened yet
#define PACK_REVEAL 0.42f   // the card is out (the strip is off, the halves fly)
#define PACK_GONE 1.1f      // the foil is gone

hz_internal b32 pack_rare(i32 c) { return card_def(c).cost >= 5 || c == CARD_GOLDBRICK; }

hz_internal HzUIClipPath pack_poly(i32 n, const f32 (*p)[2]) {
  HzUIClipPath cp = {.count = (u32)n};
  for (i32 i = 0; i < n && i < UI_CLIP_PATH_MAX_POINTS; i++)
    cp.points[i] = (HzUIClipPoint){.fraction = {p[i][0], p[i][1]}};
  return cp;
}

// the shiny wrapper of one pack (the three.js version's foil): the art is drawn whole in each piece, each piece cut out by
// its clip path - the strip that rips off first, and the two halves meeting along a jagged tear
hz_internal void pack_foil_art(f32 w, f32 h, b32 rare, f32 time, i32 piece) {
  HzUIColor a = rare ? cu_rgb(0xc99a2e) : cu_rgb(0xb9c0cc), b = rare ? cu_rgb(0xfff2b8) : cu_rgb(0xf6f8fb);
  HzUIColor c = rare ? cu_rgb(0xb8862a) : cu_rgb(0x98a2b3), d = rare ? cu_rgb(0xffe68a) : cu_rgb(0xeef1f6);
  ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .gradient = cu_lin(115.0f, 4, (f32[]){0.0f, 0.3f, 0.5f, 1.0f}, (HzUIColor[]){a, b, c, d}),
              .corner_radius = ui_corner_radius(w * 0.035f)}) {
    // the crimped seams top and bottom
    for (i32 k = 0; k < 2; k++) {
      UIGradient g = {.kind = UI_GRADIENT_LINEAR, .angle = 90.0f, .stop_count = 4, .repeating = true};
      g.stops[0] = (HzUIGradientStop){0.0f, cu_black(0.18f)};
      g.stops[1] = (HzUIGradientStop){0.0f, cu_black(0.18f)};
      g.stops[2] = (HzUIGradientStop){0.0f, cu_rgba(0xffffff, 0.35f)};
      g.stops[3] = (HzUIGradientStop){0.0f, cu_rgba(0xffffff, 0.35f)};
      g.stop_offset_px[1] = 2.0f;
      g.stop_offset_px[2] = 2.0f;
      g.stop_offset_px[3] = 4.0f;
      ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = (i16)(74 + piece), .offset = {0, k ? h * 0.91f : 0.0f}},
                  .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h * 0.09f)}},
                  .gradient = g}) {}
    }
    // the logo: a star, BRICK BEASTS, how many cards
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = (u16)(w * 0.02f), .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .transform = cu_tf(0, 0, 1, -8.0f)}) {
      HzUITextConfig star = {.fontSize = w * 0.2f, .textColor = cu_rgb(0xffd23d), .textAlignment = CLAY_TEXT_ALIGN_CENTER};
      star.shadows[0] = (HzUITextShadow){.color = cu_rgb(0xa87a00), .offset_y = 1.0f};
      ui_text(ui_str_c(rare ? "\xe2\x9c\xa6" : "\xe2\x98\x85"), star);
      HzUITextConfig t = {.fontSize = w * 0.13f, .textColor = rare ? cu_rgb(0x8a4b00) : cu_rgb(0xc91a09), .weight = 800,
                          .textAlignment = CLAY_TEXT_ALIGN_CENTER, .lineHeight = w * 0.14f};
      t.shadows[0] = (HzUITextShadow){.color = cu_rgb(0xffffff), .offset_y = 1.0f};
      ui_text(ui_str_c("BRICK\nBEASTS"), t);
      ui_text(ui_str_c(rare ? "RARE \xc2\xb7 1 CARD" : "1 CARD"),
              ui_text_config({.fontSize = w * 0.075f, .textColor = cu_rgb(0x1b2a34), .weight = 700}));
    }
    // the rainbow sheen sweeping over it
    f32 sweep = m_fmodf(time / 2.4f, 1.0f);
    UIGradient holo = cu_lin(115.0f, 4, (f32[]){0.0f, 0.33f, 0.66f, 1.0f},
                             (HzUIColor[]){cu_rgba(0xff50a0, 0.0f), cu_rgba(0xff50a0, 0.22f), cu_rgba(0x50c8ff, 0.25f), cu_rgba(0x78ff8c, 0.0f)});
    ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = (i16)(74 + piece), .offset = {(sweep * 2.2f - 1.1f) * w, 0}},
                .layout = {.sizing = {.width = ui_sizing_fixed(w * 0.9f), .height = ui_sizing_fixed(h)}},
                .gradient = holo,
                .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH}) {}
  }
}

hz_internal void rewards_draw(CardUi *ui, const GameSave *s, const InputSystem *in, f32 sw) {
  if (!ui->reward_n)
    return;
  HzUIColor ink = {27, 42, 52, 255};
  b32 all = true;
  for (i32 i = 0; i < ui->reward_n; i++)
    all &= ui->reward_tear[i] >= PACK_REVEAL;
  ui_element({.id = ui_id("Rewards"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 70},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)},
                         .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}, .padding = ui_padding_all(12)},
              .background_color = cu_black(0.67f)}) {
    ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 14, .padding = ui_padding_all(22),
                           .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                .background_color = {250, 250, 246, 255},
                .corner_radius = ui_corner_radius(14),
                .shadow = {.color = cu_black(0.4f), .offset_y = 10.0f, .blur = 30.0f}}) {
      ui_element({.layout = {.sizing = {.width = ui_sizing_fit(0, m_minf(sw - 70.0f, 520.0f))}}}) {
        cu_textb(ui->reward_title, 20, ink, 800, true, (HzUIColor){0});
      }
      f32 w = m_clampf((sw - 100.0f) / 3.2f, 90.0f, 150.0f), h = w * 1.4f;
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 14, .padding = {.top = 8, .bottom = 8}}}) {
        for (i32 i = 0; i < ui->reward_n && i < 3; i++) {
          i32 c = ui->reward[i];
          b32 rare = pack_rare(c);
          f32 t = ui->reward_tear[i];
          b32 sealed = t <= PACK_SEALED + 1.0f;
          // dealt in: drop and turn from above, one after another
          f32 deal = cu_ease_out(m_clampf((ui->reward_t - 0.08f * (f32)i) / 0.35f, 0.0f, 1.0f));
          ui_element({.id = ui_idi("RewardPack", (u32)i),
                      .layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
                      .opacity = opt_some(f32, deal),
                      .transform = cu_tf(0, -(1.0f - deal) * h * 0.4f, 1.0f, (1.0f - deal) * (i == 1 ? 4.0f : -6.0f))}) {
            if (sealed && ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
              ui->click_pack = i;
            // the card: pops out when the foil opens, with a burst of light (rare ones glow)
            f32 r = t - PACK_REVEAL;
            f32 pop = 1.0f, rot = 0.0f;
            if (r >= 0.0f && r < 0.55f) {
              f32 q = r / 0.55f;
              pop = q < 0.6f ? m_lerpf(0.85f, 1.1f, cu_ease_out(q / 0.6f)) : m_lerpf(1.1f, 1.0f, (q - 0.6f) / 0.4f);
              rot = q < 0.6f ? -2.0f * q / 0.6f : -2.0f * (1.0f - (q - 0.6f) / 0.4f);
            }
            ui->z = 71;
            ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(w), .height = ui_sizing_fixed(h)}},
                        .shadow = {.color = rare && r >= 0.0f ? cu_rgba(0xffd23d, 0.9f) : (HzUIColor){0}, .blur = 18.0f, .spread = 2.0f},
                        .corner_radius = ui_corner_radius(w * 0.06f),
                        .transform = r >= 0.0f ? cu_tf(0, 0, pop, rot) : opt_none(HzUITransform)}) {
              CardLook k = {.id = c, .power = card_def(c).power, .level = s->card_level[c], .w = w, .uid = 800u + (u32)i};
              cu_card(ui, k, in, ui_idi("Reward", (u32)i));
            }
            if (r >= 0.0f && r < 0.9f) { // the sparkle
              f32 q = r / 0.9f, sz = w * 1.3f * (0.6f + 0.8f * q);
              ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = 73, .offset = {(w - sz) * 0.5f, (h - sz) * 0.5f},
                                       .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                          .layout = {.sizing = {.width = ui_sizing_fixed(sz), .height = ui_sizing_fixed(sz)}},
                          .backgrounds = ui_backgrounds((HzUIBackgroundLayer){.gradient = {.kind = UI_GRADIENT_RADIAL, .stop_count = 3, .circle = true,
                                                                                          .size = UI_RADIAL_FARTHEST_SIDE,
                                                                                          .stops = {{0.0f, cu_rgba(0xffffff, 0.95f)}, {0.4f, cu_rgba(0xffe678, 0.55f)}, {0.7f, cu_rgba(0xffe678, 0.0f)}}}}),
                          .corner_radius = ui_corner_radius(sz * 0.5f),
                          .opacity = opt_some(f32, 1.0f - q)}) {}
            }
            // the foil: the strip rips off, then the halves fly apart
            if (t < PACK_GONE) {
              f32 fw = w * 1.03f, fh = h * 1.06f, ox = -w * 0.015f, oy = -h * 0.03f;
              f32 ps = sealed ? 0.0f : m_clampf(t / 0.35f, 0.0f, 1.0f);
              ps *= ps;
              f32 ph = sealed ? 0.0f : m_clampf((t - 0.25f) / 0.55f, 0.0f, 1.0f);
              ph *= ph;
              b32 hover = sealed && ui_pointer_over(ui_idi("RewardPack", (u32)i));
              static const f32 left[7][2] = {{0, 0}, {0.52f, 0}, {0.46f, 0.25f}, {0.54f, 0.5f}, {0.46f, 0.75f}, {0.52f, 1}, {0, 1}};
              static const f32 right[7][2] = {{0.52f, 0}, {1, 0}, {1, 1}, {0.52f, 1}, {0.46f, 0.75f}, {0.54f, 0.5f}, {0.46f, 0.25f}};
              static const f32 strip[7][2] = {{0, 0}, {1, 0}, {1, 0.11f}, {0.75f, 0.135f}, {0.5f, 0.11f}, {0.25f, 0.135f}, {0, 0.115f}};
              for (i32 piece = 0; piece < 3; piece++) {
                f32 tx = 0, ty = 0, rotp = 0, op = 1.0f;
                if (piece == 0) { tx = -0.75f * fw * ph; rotp = -16.0f * ph; op = 1.0f - ph; }
                if (piece == 1) { tx = 0.75f * fw * ph; rotp = 16.0f * ph; op = 1.0f - ph; }
                if (piece == 2) { tx = 0.35f * fw * ps; ty = -0.5f * fh * ps; rotp = -24.0f * ps; op = 1.0f - ps; }
                ui_element({.floating = {.attach_to = UI_ATTACH_TO_PARENT, .z_index = (i16)(74 + piece), .offset = {ox, oy},
                                         .pointer_capture_mode = UI_POINTER_CAPTURE_MODE_PASSTHROUGH},
                            .layout = {.sizing = {.width = ui_sizing_fixed(fw), .height = ui_sizing_fixed(fh)}},
                            .clip_path = pack_poly(7, piece == 0 ? left : piece == 1 ? right : strip),
                            .opacity = opt_some(f32, op),
                            .brightness = hover ? opt_some(f32, 1.12f) : opt_none(f32),
                            .transform = cu_tf(tx, ty, 1.0f, rotp)}) {
                  pack_foil_art(fw, fh, rare, ui->time, piece);
                }
              }
            }
          }
        }
      }
      ui->z = 0;
      if (!all) {
        cu_text("Tap a pack to tear it open!", 14, (HzUIColor){90, 94, 100, 255});
        if (cu_button(ui_id("RewardAll"), "Reveal all", (HzUIColor){120, 60, 200, 255}, true, in, 17))
          ui->click_reveal_all = true;
      } else if (cu_button(ui_id("RewardOk"), "Awesome!", (HzUIColor){0, 85, 191, 255}, true, in, 18)) {
        ui->click_reward_ok = true;
      }
    }
  }
}
