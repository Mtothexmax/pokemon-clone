---
title: World UI
description: world-anchored UI as ECS widgets (nametag, damage/value popup, proximity prompt, hotspot pin) with projection, occlusion ray, edge clamp and stacking, plus BillboardGui and SurfaceGui for world-space surfaces
resource: src/gameplay/world_ui/world_ui.h
tags: [gameplay, ui, world-ui, hud]
updated: 2026-11-03
---

# What it provides

- Block 2, `WorldUi` entities: `WorldUi` (kind, anchor offset, `max_distance`, `occlude`, `clamp_edge`, `fade_s`, stacking)
  + `WorldUiLabel` (text, sub line, colour, fill bar) on any entity; the anchor is the entity's `WorldTransform` plus
  `offset`, or a screen position when `screen_anchored`. `WorldUiScreen` holds the result each frame (`pos`, `distance`,
  `alpha`, `visible`, `occluded`, `clamped`, `angle`, `active`, `triggered`).
- Kinds: `WORLD_UI_NAMETAG` (name, sub line, optional fill bar always or `ON_HIT`), `WORLD_UI_POPUP`, `WORLD_UI_PROMPT`
  (`WorldUiPrompt {key, range, enabled, hold_s}`; the nearest enabled one in range of the `WorldUiInteractor` is active
  and reports `triggered` on its key press, or after `hold_s` seconds held with `hold_progress` 0..1), `WORLD_UI_PIN`.
- Aim select (`WorldUiDesc.aim_select`, `aim_layer`, `aim_filter`, `aim_reach`): the prompt whose collider the camera's
  centre ray hits is active instead of the nearest, hidden by the occluder layer when `occlusion` is on. Each prompt
  entity carries its own collider in `aim_layer`. `world_ui_active_prompt(w, &hit)` gives the entity and hit point.
- `world_ui_input(w, pressed, held)` (main lane, before the scope) hands world_ui the game's interact button (several
  keys, a pad button, a script) in place of each prompt's key; `press_buffer_s` keeps a press usable for a moment.
- `world_ui_popup(w, &desc)` spawns a self-destroying popup entity from any system (rise, drift, enter and fade
  phases, scatter, stack group); this replaces the value-popup and damage-number blocks.
- `world_ui_layout(w, ui, &style)` draws the kinds in `style.kinds`; a kind left out is yours to draw from
  `WorldUiScreen` (a hotspot pin with its own look).
- `BillboardGui` (block 1): a `UISurface` anchored at a world point. `billboard_create(ui, r, &BillboardGuiDesc)` (canvas size, `size_mode`
  `BILLBOARD_SIZE_STUDS` shrinks with distance or `BILLBOARD_SIZE_PIXELS` holds `pixel_height` on screen, world and camera offsets,
  `max_distance`, `always_on_top`), `billboard_place(&bg, &cam, screen_h, anchor)` per frame (false when culled), `billboard_destroy`.
- `SurfaceGui` (block 3): a panel on one face of an entity with a `WorldTransform`. `surface_gui_init(w, ui, r)` once;
  `surface_gui_add(w, e, &SurfaceGuiDesc)` (`face` `FACE_TOP/BOTTOM/FRONT/BACK/LEFT/RIGHT`, `pixels_per_stud`, face extent,
  `always_on_top`, `clear_color`); `surface_gui_adopt(w)` once after a scene spawns, for authored `SurfaceGui` components;
  `surface_gui_update(w, &cam)` per frame; `surface_gui_get_surface(w, e)` to draw into; `surface_gui_destroy`.
  `SurfaceGuiText` on the same entity is a centred label that `surface_gui_text_draw(w)` draws. `surface_gui_face_rotation(face)` is exported for tests.

# Example usage

```c
// init (main lane), after ecs_physics_init when occluding
world_ui_init(w, &(WorldUiDesc){.occlusion = true, .occluder_layer = product_layer, .occlusion_every = 3});
world_ui_add(w, pin, &(WorldUi){.kind = WORLD_UI_PIN, .occlude = true}, &(WorldUiLabel){0});
// in the scope, after ecs_camera_schedule (and ecs_physics_schedule when occluding)
world_ui_schedule(w);
// from a system: a kill popup stacked above the live ones
world_ui_popup(it->world, &(WorldUiPopupDesc){.screen_anchored = true, .screen_frac = {0.5f, 0.5f}, .text = text,
                                              .color = white, .life = 1.4f, .stack_group = 1, .stack_size = {120, 24}});
// UI frame (main lane)
world_ui_layout(w, ui, &style);
const WorldUiScreen *s = world_ui_screen(w, pin);

// world surfaces: main lane, end of frame, before renderer_update and outside the UI frame (header Usage lines)
BillboardGui bg = billboard_create(ui, r, &(BillboardGuiDesc){.canvas_w = 256, .canvas_h = 64, .pixels_per_unit = 128});
if (billboard_place(&bg, &cam, screen_h, anchor)) ui_surface_scope(ui, bg.surface) { /* ui_text(...) */ }
surface_gui_add(w, e, &(SurfaceGuiDesc){.face = FACE_FRONT, .pixels_per_stud = 40});
surface_gui_update(w, &cam);
ui_surface_update_all(ui, &cam);
ui_surface_scope(ui, surface_gui_get_surface(w, e)) { /* ui_text(...) */ }
```

# Gotchas

- `WorldUiScreen` appears one flush after `WorldUi`; `world_ui_screen` is NULL before the first projection.
- Occlusion rays are staggered per widget (`occlusion_every` frames) and stop `occlusion_margin` metres short of the
  anchor; without `ecs_physics_init` leave `occlusion` off.
- A screen-anchored prompt ignores range: it is active while `enabled`.
- `triggered` and `hold_progress` are this frame's consolidate result: a system scheduled before `world_ui_schedule`
  reads last frame's. Order a system after it by reading the `WorldUiView` resource.
- Stacking pushes overlapping boxes of one `stack_group` up and keeps the offset unless `stack_relax` settles it back.
- The view is the scene `Camera`'s matrices and the OS canvas size: a UI that renders to a smaller target needs its own
  projection.
- `SurfaceGui` calls assert single-thread and run at end of frame before `renderer_update`, outside the UI frame.
- Part-local face normals are Top=+Y, Bottom=-Y, Front=+Z, Back=-Z, Left=-X, Right=+X; `FACE_FRONT` and `FACE_BACK` are the opposite of `NormalId`.
- A world surface drawn inside `ui_begin_frame` .. `ui_finish_layout` renders nothing, silently.
