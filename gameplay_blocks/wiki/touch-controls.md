---
title: Touch Controls
description: does the engine have mobile touch controls - a dynamic thumbstick, a jump button, anchored action buttons (fire, aim, reload) with look-through drag, camera pan and pinch with the touch/camera exclusion rule, read-only views and default draws; TouchControls, TouchControlsConfig, TouchControlsFrame
resource: src/gameplay/touch_controls/touch_controls.h
tags: [gameplay, touch, mobile, input, controls]
updated: 2026-11-03
---

# What it provides

- `touch_controls_init(TouchControlsConfig)` returns a `TouchControls`; `touch_controls_update(&tc, input, now)` is a pure view over `InputSystem.touches`
  (no ECS system, no allocation, fixed slots). It claims the touches it uses, mutating `input`, and returns a `TouchControlsFrame`.
- Dynamic thumbstick: a touch that starts in the stick region spawns the stick at that point. `move` is camera-relative (x right, y forward, `|move| <= 1`),
  `stick_travel` is the finger travel over the max-speed radius, unclamped: above 1 is an over-push (sprint).
  `TOUCH_THUMBSTICK_FIXED` is reserved and not implemented.
- Jump: `jump_held` is a level; `TouchControlsConfig.disable_jump_button` removes the button.
- Action buttons: `TouchControlsConfig.buttons[TOUCH_BUTTON_MAX]` (`TouchButtonDesc`: `enabled`, `look_through`, `label`), indexed by your own enum,
  placed per orientation by `TouchControlsLayout.buttons[i]` (`TouchButtonPlacement`: `TouchAnchor` safe-area corner, `corner_offset_px` to the button centre, `diameter_px`; 0 = absent there).
  `TouchControlsFrame.buttons[i]` reports `held`, `pressed`, `released`. A `look_through` button's finger also adds to `camera_pan_px` (fire while aiming).
  `touch_controls_set_button(tc, i, visible, lit)` hides a button (a held touch stays until lifted) or draws it highlighted, per frame.
- Camera: a touch that starts in the stick region never reaches the camera; of the rest, one finger pans (`camera_pan_px`, logical px, not dt-scaled),
  two pinch (`camera_pinch_px`, positive spreading). `disable_pinch` keeps two free fingers panning with the earliest.
  `pinch_grace_s` revokes a stick claim when a second touch starts within that window, so two fingers pinch instead of walking.
- Layout: `touch_controls_layout_default(TOUCH_ORIENTATION_LANDSCAPE / _PORTRAIT)` returns every placement and feel number with `custom` set, to assign and edit.
  Landscape applies when width >= height. px fields are logical px scaled by `ui_scale` (auto below `small_screen_max_dim_px`); `_pct` fields are fractions of the safe-area rect.
- Drawing: `touch_controls_draw(tc, ui)` is the default look of every widget; `touch_controls_draw_stick`, `_draw_button(i)`, `_draw_jump_button` draw one.
  A game with its own look reads `touch_controls_stick`, `touch_controls_button(i)`, `touch_controls_jump_button` (`TouchStickView` / `TouchButtonView`: visible, held, lit, centre, diameter, the exact hit circle) and draws over them.
- `TouchControlsConfig.ui` keeps touches that start over pointer-capturing UI with the UI (`ui_point_over_ui`); its own circles are pointer-passthrough.
- `TouchControlsFrame.last_pan_time` is a seam for aim assist; no aim assist is implemented.

# Example usage

From `templates/proving-grounds/src/proving_grounds/gr_input.c` (a shooter layout; the starting template uses the defaults):

```c
TouchControlsLayout l = touch_controls_layout_default(o);
l.region_pos_pct = (v2){0.0f, 0.3f};
l.buttons[GR_TOUCH_FIRE] = (TouchButtonPlacement){TOUCH_ANCHOR_BOTTOM_RIGHT, {130.0f, 110.0f}, 84.0f};
// config: .ui = ui, .landscape / .portrait = the two layouts, .buttons[GR_TOUCH_FIRE] = {.enabled = true, .look_through = true, .label = "FIRE"}
TouchControls tc = touch_controls_init(cfg);

// per frame, main lane, after input_update and before the UI frame
TouchControlsFrame tf = touch_controls_update(&tc, input, now);
touch_controls_set_button(&tc, GR_TOUCH_ADS, true, ads_on);
in->forward = m_clampf(in->forward + tf.move.y, -1.0f, 1.0f);   // ADD to the keyboard intent, then reclamp
in->strafe = m_clampf(in->strafe + tf.move.x, -1.0f, 1.0f);
in->look = v2_add(in->look, v2_scale(tf.camera_pan_px, GR_TOUCH_LOOK_SCALE));
in->fire_held |= tf.buttons[GR_TOUCH_FIRE].held;
// UI frame: touch_controls_draw(&tc, ui);  (the starting template draws the stick and jump button with the _draw_ calls)
```

# Gotchas

- Main lane only, after `input_update` and before the UI frame; the draw calls go inside `ui_begin_frame` / `ui_finish_layout` and do nothing when not visible.
- The block feeds nothing itself: add `tf.move` and `tf.jump_held` into your `CharacterInput` and scale `camera_pan_px` / `camera_pinch_px` into `CameraRigInput.look` / `zoom` (see [[character]], [[camera-rig]]). Summed moves need a reclamp.
- `touch_controls_update` asserts `TOUCH_THUMBSTICK_DYNAMIC` in debug builds.
- `TouchControlsLayout.custom` is an explicit opt-in: with it false every other field is ignored and the defaults apply.
- `tf.active` is true when the last input category was touch, or `force_enabled` is set; gate on it so a mouse and keyboard session ignores the widgets.
