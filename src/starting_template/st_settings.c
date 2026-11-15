// The menu behind the top-right button (the three.js +page.svelte menu): progress, Voices on/off, Music on/off and
// Graphics Auto / High / Medium / Low. Auto measures the frame rate and steps down while the game runs below ~38 fps:
// first the anti-aliasing goes, then the render resolution and the renderer's quality level (shadows, effects). The
// choice and the level Auto reached are kept in the save. Included by starting_template.c above app_init.

typedef enum GfxChoice { GFX_AUTO, GFX_HIGH, GFX_MEDIUM, GFX_LOW, GFX_COUNT } GfxChoice;

// the steps Auto walks down (and what the fixed choices use)
typedef struct GfxStep {
  CString name;
  RendererQuality quality;
  u32 msaa;    // 4 or 1 (off)
  f32 max_dpr; // render resolution cap (css px to device px)
} GfxStep;
#define GFX_STEPS 4
hz_internal GfxStep gfx_step(i32 i) {
  switch (i) {
  case 0: return (GfxStep){"High", RENDERER_QUALITY_HIGH, 4u, 1.25f};
  case 1: return (GfxStep){"High (no anti-aliasing)", RENDERER_QUALITY_HIGH, 1u, 1.25f};
  case 2: return (GfxStep){"Medium", RENDERER_QUALITY_MEDIUM, 1u, 1.0f};
  default: return (GfxStep){"Low", RENDERER_QUALITY_LOW, 1u, 0.75f};
  }
}

typedef struct Settings {
  b32 open;
  b32 click_button, click_voices, click_music, click_shade;
  i32 click_touch; // touch mode clicked (-1)
  i32 click_gfx; // GfxChoice clicked (-1)
  // what is applied to the renderer now
  i32 applied; // step index, -1 none yet
  // Auto's frame-rate watch
  f32 win_t, settle;
  u32 win_frames;
  f32 fps; // the last window's frame rate (shown in the menu)
  // the device: phones get no MSAA (a phone's Vulkan driver failed to build a 4x pipeline and the game hung), and
  // Auto never starts above the level the engine picks for the hardware
  b32 hw_known, mobile;
  i32 hw_step;
  i32 buses; // the voice / music switches applied to the mixer (-1 none yet)
} Settings;

hz_internal i32 settings_step(const Settings *st, const GameSave *g) {
  switch ((GfxChoice)g->gfx) {
  case GFX_HIGH: return st->mobile ? 1 : 0; // phones: High without MSAA
  case GFX_MEDIUM: return 2;
  case GFX_LOW: return 3;
  default: return MAX(g->auto_step ? (i32)g->auto_step - 1 : 0, st->hw_step);
  }
}

// main lane, before the renderer runs: the menu's clicks, the audio buses, the frame-rate watch and the graphics level
hz_internal void settings_update(Settings *st, SaveState *save, RendererState *rs, AudioSystem *audio, const InputSystem *in, f32 dt,
                                 b32 focused) {
  GameSave *g = &save->data;
  if (!st->hw_known) {
    st->hw_known = true;
    RendererHardware hw = renderer_hardware(rs);
    st->mobile = hw.platform == RENDERER_PLATFORM_MOBILE || hw.platform == RENDERER_PLATFORM_MOBILE_WEB;
    RendererQuality q = renderer_quality_for_hardware(&hw);
    // desktops may start at the top (Auto steps down when slow); phones start where the engine says, without MSAA
    st->hw_step = !st->mobile ? 0 : q >= RENDERER_QUALITY_HIGH ? 1 : q == RENDERER_QUALITY_MEDIUM ? 2 : 3;
    log_info("graphics: % device, Auto starts at %", fmt_cstr(st->mobile ? "mobile" : "desktop"), fmt_cstr(gfx_step(st->hw_step).name));
  }
  if (!save->loaded)
    return;
  // a new save: Auto starts at High (as the three.js version) and steps down only when it really is slow
  if (!g->auto_step) {
    g->auto_step = 1;
    save_mark(save);
  }
  // ---- the menu ----
  if (st->click_button) {
    st->open = !st->open;
  } else if (st->open && (st->click_shade || in->buttons[KEY_ESCAPE].pressed_this_frame)) {
    st->open = false;
  }
  if (st->click_voices) {
    g->voices_off = !g->voices_off;
    save_mark(save);
  }
  if (st->click_touch >= 0) {
    g->touch_mode = (u8)st->click_touch;
    save_mark(save);
  }
  if (st->click_music) {
    g->music_off = !g->music_off;
    save_mark(save);
  }
  if (st->click_gfx >= 0) {
    g->gfx = (u8)st->click_gfx;
    if (st->click_gfx == GFX_AUTO)
      g->auto_step = 1; // Auto starts from High again and steps down if needed
    st->settle = 0.0f;
    save_mark(save);
  }
  st->click_button = st->click_voices = st->click_music = st->click_shade = false;
  st->click_gfx = st->click_touch = -1;
  // the buses only when the switches change: set every frame, the engine restarts the bus fade each time and the
  // voices and the music never got off zero (silent)
  i32 buses = (g->voices_off ? 1 : 0) | (g->music_off ? 2 : 0);
  if (buses != st->buses) {
    st->buses = buses;
    audio_bus_set_volume(audio, AUDIO_BUS_VOICE, g->voices_off ? 0.0f : 1.0f);
    audio_bus_set_volume(audio, AUDIO_BUS_MUSIC, g->music_off ? 0.0f : 1.0f);
  }

  // ---- Auto: measure 2 s windows; below 38 fps one step down (not right after a change, a load or a stall) ----
  st->settle += dt;
  if (!focused || dt > 0.25f || st->settle < 4.0f) {
    st->win_t = 0.0f;
    st->win_frames = 0;
  } else {
    st->win_t += dt;
    st->win_frames++;
    if (st->win_t >= 2.0f) {
      st->fps = (f32)st->win_frames / st->win_t;
      i32 cur = settings_step(st, g);
      if (g->gfx == GFX_AUTO && st->fps < 38.0f && cur + 1 < GFX_STEPS) {
        g->auto_step = (u8)(cur + 2); // one below the step in use (+1: 0 means not chosen)
        save_mark(save);
        st->settle = 0.0f;
        log_info("graphics: % fps, Auto steps down to %", fmt_f32_p(st->fps, 1), fmt_cstr(gfx_step(cur + 1).name));
      }
      st->win_t = 0.0f;
      st->win_frames = 0;
    }
  }

  // ---- apply the level: renderer quality (fixed: we pick it), anti-aliasing, resolution ----
  i32 step = settings_step(st, g);
  GfxStep gs = gfx_step(step);
  if (st->applied != step) {
    st->applied = step;
    renderer_set_quality_mode(rs, RENDERER_QUALITY_FIXED);
    renderer_set_quality(rs, gs.quality);
    hz_set_max_dpr(gs.max_dpr);
  }
  u32 want = !st->mobile && (renderer_supported_sample_counts(rs) & gs.msaa) ? gs.msaa : 1u;
  if (want != renderer_sample_count(rs) && want != renderer_pending_sample_count(rs))
    renderer_set_sample_count(rs, want);
}

hz_internal b32 settings_row(HzUIElementId id, CString label, CString state, const InputSystem *in) {
  b32 clicked = false;
  b32 hover = ui_pointer_over(id);
  ui_element({.id = id,
              .layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 10, .padding = {.left = 12, .right = 12, .top = 9, .bottom = 9},
                         .sizing = {.width = ui_sizing_grow(0)}, .child_alignment = {.y = UI_ALIGN_Y_CENTER}},
              .background_color = hover ? cu_rgb(0xeef2f5) : cu_rgba(0xffffff, 0),
              .corner_radius = ui_corner_radius(10)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      clicked = true;
    cu_textb(label, 16, cu_rgb(0x1b2a34), 600, false, (HzUIColor){0});
    ui_element({.layout = {.sizing = {.width = ui_sizing_grow(0)}}}) {}
    if (state)
      cu_textb(state, 14, cu_rgb(0x5c6670), 600, false, (HzUIColor){0});
  }
  return clicked;
}

// the ☰ button (top right, left of it the Deck button) and the menu under it
hz_internal void settings_draw(Settings *st, CardUi *ui, const GameSave *g, const InputSystem *in, f32 sw, b32 touch, b32 show_button,
                               u32 golden_n,
                               u32 golden_total) {
  if (!show_button && !st->open)
    return;
  OsSafeAreaInsets safe = os_get_safe_area();
  HzUIColor ink = cu_rgb(0x1b2a34);
  ui_element({.id = ui_id("MenuButton"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 20,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                           .offset = {-safe.right - 14.0f, safe.top + 14.0f}},
              .layout = {.sizing = {.width = ui_sizing_fixed(40), .height = ui_sizing_fixed(36)}, .child_alignment = {.x = UI_ALIGN_X_CENTER, .y = UI_ALIGN_Y_CENTER}},
              .background_color = st->open ? cu_rgb(0xffffff) : cu_rgba(0xfafaf6, 0.92f),
              .corner_radius = ui_corner_radius(10),
              .shadow = {.color = cu_black(0.25f), .offset_y = 2.0f, .blur = 6.0f}}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
      st->click_button = true;
    if (st->open) {
      cu_textb("\xc3\x97", 22, ink, 700, false, (HzUIColor){0});
    } else { // three bars (the font has no menu glyph)
      ui_element({.layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4}}) {
        for (i32 i = 0; i < 3; i++)
          ui_element({.layout = {.sizing = {.width = ui_sizing_fixed(18), .height = ui_sizing_fixed(2.5f)}}, .background_color = ink,
                      .corner_radius = ui_corner_radius(1.25f)}) {}
      }
    }
  }
  if (!st->open)
    return;
  // a shade to click away
  ui_element({.id = ui_id("MenuShade"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 18},
              .layout = {.sizing = {.width = ui_sizing_grow(0), .height = ui_sizing_grow(0)}},
              .background_color = cu_black(0.15f)}) {
    if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame && !ui_pointer_over(ui_id("Menu")) && !ui_pointer_over(ui_id("MenuButton")))
      st->click_shade = true;
  }
  ui_element({.id = ui_id("Menu"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT, .z_index = 19,
                           .attach_points = {.element = UI_ATTACH_POINT_RIGHT_TOP, .parent = UI_ATTACH_POINT_RIGHT_TOP},
                           .offset = {-safe.right - 14.0f, safe.top + 58.0f}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .child_gap = 4, .padding = ui_padding_all(10),
                         .sizing = {.width = ui_sizing_fixed(m_minf(320.0f, sw - 28.0f))}},
              .background_color = cu_rgb(0xfafaf6),
              .corner_radius = ui_corner_radius(16),
              .shadow = {.color = cu_black(0.3f), .offset_y = 8.0f, .blur = 24.0f}}) {
    // progress
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 14, .padding = {.left = 12, .right = 12, .top = 4, .bottom = 8}}}) {
      cu_textb(CU_FMT(ui, "\xe2\x98\x85 %/%", fmt_u32(golden_n), fmt_u32(golden_total)), 14, cu_rgb(0xb08600), 700, false, (HzUIColor){0});
      if (g->has_deck) {
        cu_textb(CU_FMT(ui, "Cards %/%", fmt_u32(owned_count(g)), fmt_i32(CARD_POOL)), 14, cu_rgb(0x1655b0), 700, false, (HzUIColor){0});
        cu_textb(CU_FMT(ui, "Wins % / %", fmt_u32(g->wins), fmt_u32(g->losses)), 14, cu_rgb(0x5c6670), 700, false, (HzUIColor){0});
      }
    }
    if (settings_row(ui_id("MenuVoices"), "Voices", g->voices_off ? "off" : "on", in))
      st->click_voices = true;
    if (settings_row(ui_id("MenuMusic"), "Music", g->music_off ? "off" : "on", in))
      st->click_music = true;
    ui_element({.layout = {.padding = {.left = 12, .top = 8, .bottom = 2}}}) {
      cu_textb("GRAPHICS", 12, cu_rgb(0x5c6670), 800, false, (HzUIColor){0});
    }
    // segmented Auto / High / Medium / Low
    ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 3, .padding = ui_padding_all(3), .sizing = {.width = ui_sizing_grow(0)}},
                .background_color = cu_rgb(0xe6e9ec),
                .corner_radius = ui_corner_radius(10)}) {
      static const CString names[GFX_COUNT] = {"Auto", "High", "Medium", "Low"};
      for (i32 i = 0; i < GFX_COUNT; i++) {
        b32 on = g->gfx == i;
        HzUIElementId id = ui_idi("MenuGfx", (u32)i);
        ui_element({.id = id,
                    .layout = {.sizing = {.width = ui_sizing_grow(0)}, .padding = {.top = 7, .bottom = 7}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                    .background_color = on ? cu_rgb(0xffffff) : ui_pointer_over(id) ? cu_rgb(0xf2f4f6) : cu_rgba(0xffffff, 0),
                    .corner_radius = ui_corner_radius(8),
                    .shadow = {.color = on ? cu_black(0.18f) : (HzUIColor){0}, .offset_y = 1.0f, .blur = 3.0f}}) {
          if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
            st->click_gfx = i;
          cu_textb(names[i], 14, on ? ink : cu_rgb(0x5c6670), on ? 800 : 600, false, (HzUIColor){0});
        }
      }
    }
    ui_element({.layout = {.padding = {.left = 12, .right = 12, .top = 6, .bottom = 6}, .sizing = {.width = ui_sizing_grow(0)}}}) {
      CString hint = g->gfx == GFX_AUTO
                         ? CU_FMT(ui, "Auto: now %. It turns down if the game gets slow.", fmt_cstr(gfx_step(settings_step(st, g)).name))
                         : "Lower settings run smoother on slow devices.";
      ui_text(ui_str_c(hint), ui_text_config({.fontSize = 13, .textColor = cu_rgb(0x5c6670), .wrapMode = UI_TEXT_WRAP_WORDS}));
    }
    if (touch) { // phones: swipe walking or the stick
      ui_element({.layout = {.padding = {.left = 12, .top = 8, .bottom = 2}}}) {
        cu_textb("CONTROLS", 12, cu_rgb(0x5c6670), 800, false, (HzUIColor){0});
      }
      ui_element({.layout = {.layout_direction = UI_LEFT_TO_RIGHT, .child_gap = 3, .padding = ui_padding_all(3), .sizing = {.width = ui_sizing_grow(0)}},
                  .background_color = cu_rgb(0xe6e9ec),
                  .corner_radius = ui_corner_radius(10)}) {
        static const CString tnames[2] = {"Swipe", "D-pad"};
        for (i32 i = 0; i < 2; i++) {
          b32 on = g->touch_mode == i;
          HzUIElementId id = ui_idi("MenuTouch", (u32)i);
          ui_element({.id = id,
                      .layout = {.sizing = {.width = ui_sizing_grow(0)}, .padding = {.top = 7, .bottom = 7}, .child_alignment = {.x = UI_ALIGN_X_CENTER}},
                      .background_color = on ? cu_rgb(0xffffff) : cu_rgba(0xffffff, 0),
                      .corner_radius = ui_corner_radius(8),
                      .shadow = {.color = on ? cu_black(0.18f) : (HzUIColor){0}, .offset_y = 1.0f, .blur = 3.0f}}) {
            if (ui_hovered() && in->buttons[MOUSE_LEFT].pressed_this_frame)
              st->click_touch = i;
            cu_textb(tnames[i], 14, on ? ink : cu_rgb(0x5c6670), on ? 800 : 600, false, (HzUIColor){0});
          }
        }
      }
      ui_element({.layout = {.padding = {.left = 12, .right = 12, .top = 6, .bottom = 6}, .sizing = {.width = ui_sizing_grow(0)}}}) {
        ui_text(ui_str_c(g->touch_mode == 0 ? "Swipe once to walk that way, tap to stop." : "Hold and drag the stick to walk."),
                ui_text_config({.fontSize = 13, .textColor = cu_rgb(0x5c6670), .wrapMode = UI_TEXT_WRAP_WORDS}));
      }
    }
    if (st->fps > 0.0f) {
      ui_element({.layout = {.padding = {.left = 12, .bottom = 4}}}) {
        cu_textb(CU_FMT(ui, "% fps", fmt_f32_p(st->fps, 0)), 12, cu_rgb(0x8a929a), 600, false, (HzUIColor){0});
      }
    }
  }
}
