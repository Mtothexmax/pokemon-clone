// STARTING TEMPLATE: grid floor, day/night sky, character, follow camera, HUD; search wiki/index.md before writing a helper.
#include "app/app.h"
#include "assets/assets.h"
#include "ecs/ecs.h"
#include "ecs/ecs.gen.h"
#include "ecs/camera/ecs_camera.h"
#include "ecs/ecs_assets/ecs_assets.h"
#include "ecs/ecs_editor/ecs_editor.h"
#include "ecs/ecs_physics/ecs_physics.h"
#include "ecs/ecs_rigidbody/ecs_rigidbody.h"
#include "ecs/input/ecs_input.h"
#include "ecs/renderer/ecs_renderer.h"
#include "ecs/transform/ecs_transform.h"
#include "gameplay/camera_rig/camera_rig.h"
#include "gameplay/character/character.h"
#include "gameplay/day_night/day_night.h"
#include "gameplay/touch_controls/touch_controls.h"
#include "input.h"
#include "lib/hzmath.h"
#include "lib/thread_context.h"
#include "lib/typedefs.h"
#include "os/os.h"
#include "perf_stats.h"
#include "rendering/gizmos.h"
#include "rendering/gizmos_utils/gizmos_utils.h"
#include "rendering/renderer.h"
#include "scene/scene.h"
#include "ui/ui.h"
#include "generated/types_app.gen.h"
#include "starting_template.h"
#include "game_sim.h"
#include "starting_template/starting_template.gen.h"

// editor, picking and gizmos cost GPU memory a shipped build does not need
#ifdef DEBUG
#define GAME_DEV_TOOLS 1
#else
#define GAME_DEV_TOOLS 0
#endif

// the level is scene data: edit it with F1, save with F5, then `hz/hzbuild cook`
#define GAME_SCENE_PATH "starting_template/main.hzt"
#define GAME_GRAVITY 9.81f
#define GAME_FIXED_HZ 60.0f
#define GAME_MAX_SUBSTEPS 4
// wall-clock dt is not clamped by the engine: a hitch must not become one giant simulation step
#define GAME_MAX_DT (1.0f / 20.0f)
#define GAME_LOOK_SENSITIVITY 0.003f // radians per mouse pixel
#define GAME_TOUCH_LOOK 0.005f       // radians per dragged point
#define GAME_ZOOM_RATE 0.1f
#define GAME_PINCH_ZOOM 0.02f // zoom per point of pinch spread
#define GAME_SPAWN ((v3){0.0f, 0.1f, 0.0f})
#define GAME_FALL_Y -20.0f // below this the player respawns
// the player's avatar: the Kaigen bot, female (0, default) or male (1); same clips on both
#ifndef GAME_AVATAR_MALE
#define GAME_AVATAR_MALE 0
#endif
#if GAME_AVATAR_MALE
#define GAME_AVATAR "starting_template/characters/kaigen_bot"
#else
#define GAME_AVATAR "starting_template/characters/kaigen_bot_female"
#endif

ECS_COMPONENT_TYPEDEF(AppMemory);

// reflected components (editor, scene, set:) live in starting_template.h; a plain pointer write is invisible to change detection

// resource: the entity the camera follows
typedef struct PlayerRef {
  EcsEntity player;
} PlayerRef;
ECS_COMPONENT_TYPEDEF(PlayerRef);

// per entity, parallel: the frame's controls to the character's intent; a respawn or a fall puts it back at the spawn
ecs_system(player_input_system, EcsIn(Player), EcsIn(WorldTransform), EcsInOut(CharacterInput), EcsResIn(PlayerControls)) {
  const PlayerControls *c = ecs_sys_get_resource(PlayerControls);
  v2 move = v2_length(c->move) > 1.0f ? v2_normalize(c->move) : c->move;
  for (i32 i = 0; i < it->count; i++) {
    const Player *p = ecs_sys_get_component(Player, i);
    const WorldTransform *wt = ecs_sys_get_component(WorldTransform, i);
    CharacterInput *ci = ecs_sys_get_component_mut(CharacterInput, i);
    ci->move = move;
    ci->heading = c->heading;
    ci->run = p->sprint_enabled && c->run;
    ci->jump |= c->jump; // consumed by the next fixed step
    if (c->respawn || wt->position.y < GAME_FALL_Y) {
      ci->teleport = true;
      ci->teleport_pos = GAME_SPAWN;
      ci->teleport_yaw = 0.0f;
    }
  }
}

// the scene authors the camera rig without a target: it follows the player once the player exists
ecs_system(camera_target_system, EcsInOut(CameraFollow), EcsResIn(PlayerRef)) {
  const PlayerRef *ref = ecs_sys_get_resource(PlayerRef);
  for (i32 i = 0; i < it->count; i++) {
    if (ecs_sys_get_component(CameraFollow, i)->target != ref->player)
      ecs_sys_get_component_mut(CameraFollow, i)->target = ref->player;
  }
}

// colliders are code-built: every scene entity tagged SolidBox gets a static box the size of its scale
ecs_system(solid_box_system, EcsIn(SolidBox), EcsIn(WorldTransform), EcsNot(ColliderBody)) {
  for (i32 i = 0; i < it->count; i++) {
    const WorldTransform *wt = ecs_sys_get_component(WorldTransform, i);
    ecs_component_set(it->world, it->entities[i], ColliderBody,
                      {.collider = collider_box(V3_ZERO, v3_scale(wt->scale, 0.5f)),
                       .pos = wt->position,
                       .rot = wt->rotation,
                       .scale = 1.0f});
    ecs_component_add(it->world, it->entities[i], StaticBody);
  }
}

// ---- YOUR COMPONENTS AND SYSTEMS HERE: split into files when this gets long (#include "st_enemies.c" above app_init) ----

// ---- THE SIMULATION: all game state, shared by the app and headless runs (game_sim.h); no renderer, UI, audio or avatar ----

struct GameSim {
  EcsWorld *world;
  SceneSystem scene_sys;
  SceneHandle scene;
  EcsQuery *camera_query;
  EcsQuery *solid_query;
  FixedTimestep sim_clock;
  i32 steps; // this frame's fixed steps, written by the main lane before the sync
  EcsEntity player;
  ArenaAllocator *arena; // headless only: the simulation owns its world
  Allocator alloc;
};

// main lane, once: the simulation's engine modules, the level scene and the player
hz_internal void game_sim_init(GameSim *sim, EcsWorld *w, AssetSystem *assets, Allocator *alloc) {
  sim->world = w;
  // ---- engine modules: delete the ones your game does not use ----
  ecs_transform_init(w, NULL);
  // physics with its defaults (zero fields); the world it owns goes in ecs_rigidbody_shutdown
  ecs_rigidbody_init(w, &(EcsRigidBodyInitDesc){.v2 = {.gravity = {0, -GAME_GRAVITY, 0}}});
  ecs_physics_init(w);
  character_controller_init(w);
  camera_rig_init(w);
  sim->sim_clock = fixed_timestep_make(GAME_FIXED_HZ, GAME_MAX_SUBSTEPS);

  // ---- the level: the sun, the floor and the camera come from the scene file ----
  scene_system_create(&sim->scene_sys, w, assets, alloc);
  sim->scene = scene_load_file(&sim->scene_sys, GAME_SCENE_PATH);
  sim->camera_query = ecs_query_cached(w, EcsIn(Camera));
  sim->solid_query = ecs_query_cached(w, EcsIn(SolidBox), EcsIn(ColliderBody));

  // ---- the player: a kinematic character; see wiki gameplay_blocks/character ----
  CharacterTuning tuning = {.walk_speed = 4.0f, .run_speed = 7.0f}; // zero fields take the block defaults (jump, gravity)
  sim->player = character_spawn(w, "Player", GAME_SPAWN, &tuning);
  ecs_component_set(w, sim->player, Player, {.sprint_enabled = true});
  ecs_resource_set(w, PlayerRef, {.player = sim->player});
  ecs_resource_set(w, PlayerControls, {0});

  // ---- YOUR SIMULATION INIT HERE: resources (ecs_resource_set), runtime-spawned entities ----
}

// main lane, before the scope: this frame's controls and fixed steps; a paused frame (the editor) runs none
hz_internal void game_sim_begin_frame(GameSim *sim, const PlayerControls *controls, f32 dt, b32 paused) {
  EcsWorld *w = sim->world;
  *ecs_resource_get_mut(w, PlayerControls) = *controls;
  sim->steps = paused ? 0 : fixed_timestep_begin(&sim->sim_clock, dt);
  ecs_resource_get_mut(w, CharacterFrame)->alpha = fixed_timestep_alpha(&sim->sim_clock);
}

// all lanes, inside the frame's scope: the caller adds its presentation schedules, then flushes
hz_internal void game_sim_schedule(GameSim *sim, f32 dt) {
  EcsWorld *w = sim->world;
  i32 steps = sim->steps; // every lane runs the same number of fixed steps, or the scope's syncs diverge
  ecs_assets_schedule(w);

  // ---- per-frame input: the controls resource to intent components ----
  ecs_schedule(w, player_input_system);
  ecs_schedule(w, solid_box_system);
  ecs_schedule(w, camera_target_system);
  ecs_physics_schedule(w); // the static layer the character walks on and the camera boom queries
  ecs_flush(w, dt);

  // ---- FIXED STEP: physics-driven gameplay runs here, N times per frame at fixed_dt ----
  for (i32 k = 0; k < steps; k++) {
    character_schedule_fixed(w); // moves characters (kinematic sweep or rigidbody drive)
    // YOUR FIXED-STEP SYSTEMS HERE: anything that moves bodies or must be frame-rate independent
    ecs_flush_ex(w, &(EcsFlushDesc){.delta_time = sim->sim_clock.fixed_dt, .steps = 1});
    ecs_rigidbody_step(w, sim->sim_clock.fixed_dt); // all lanes, between flushes
  }

  // ---- PER FRAME, YOUR GAME GOES HERE: ecs_schedule(w, per_entity_system) / ecs_schedule_single(w, resource_writer) ----
  character_schedule_frame(w); // interpolates the visual root, drives animation
  camera_rig_schedule(w);      // writes Camera from the followed entity
  ecs_transform_schedule(w);
  ecs_camera_schedule(w, &(EcsCameraScheduleDesc){.transform_sync_off = true}); // the rig writes Camera
}

// ---- headless API (game_sim.h) ----

GameSim *game_sim_create(void) {
  // the ECS world outside a scope is main-lane state
  assert(is_main_thread());
  ArenaAllocator *arena = arena_create(MB(64), MB(1));
  Allocator alloc = make_arena_allocator(arena);
  GameSim *sim = ALLOC(&alloc, GameSim);
  sim->arena = arena;
  sim->alloc = alloc;
  EcsWorld *w = ecs_world_create();
  // the camera reads the canvas for its aspect
  AppMemory *memory = ALLOC(&sim->alloc, AppMemory);
  *memory = (AppMemory){.canvas_width = 1280, .canvas_height = 720, .dpr = 1.0f};
  ecs_resource_set_ref(w, AppMemory, memory);
  ecs_assets_init(w, NULL); // the scene loads through it
  game_sim_init(sim, w, ecs_resource_get_mut(w, AssetSystem), &sim->alloc);
  return sim;
}

void game_sim_destroy(GameSim *sim) {
  // the ECS world outside a scope is main-lane state
  assert(is_main_thread());
  scene_system_destroy(&sim->scene_sys);
  ecs_rigidbody_shutdown(sim->world);
  AssetSystem *assets = ecs_resource_get_mut(sim->world, AssetSystem);
  ecs_world_destroy(sim->world);
  asset_system_destroy(assets);
  arena_destroy(sim->arena);
}

b32 game_sim_ready(GameSim *sim) {
  return ecs_query_entity_count(sim->solid_query) > 0 && ecs_query_get_singleton_entity(sim->camera_query) != ECS_ENTITY_INVALID;
}

void game_sim_frame(GameSim *sim, const PlayerControls *controls, f32 dt) {
  if (is_main_thread())
    game_sim_begin_frame(sim, controls, dt, false);
  lane_sync(); // publishes the main-lane writes above to every lane
  ecs_scope(sim->world) {
    game_sim_schedule(sim, dt);
    ecs_flush(sim->world, dt);
  }
}

v3 game_sim_player_position(GameSim *sim) {
  const WorldTransform *wt = ecs_component_get(sim->world, sim->player, WorldTransform);
  return wt ? wt->position : V3_ZERO;
}

// ---- THE APP: input, presentation (sky, avatar, HUD, editor) and the frame order around the simulation ----

typedef struct AppState {
  Allocator allocator;
  GameSim sim;
  UISystem *ui;
  TouchControls touch;
  b32 touch_jump_held;
  b32 editor_open;
  b32 look_locked;
#if GAME_DEV_TOOLS
  EcsEditorDefault editor;
#endif
  b32 playable; // the first frame with nothing left to load has been logged
} AppState;

HZ_APP_API void app_init(AppMemory *memory) {
  // app_init runs on every lane; world setup is main-lane work
  if (!is_main_thread())
    return;
  AppState *s = memory->state;
  s->allocator = make_arena_allocator(arena_create(MB(16), MB(1)));

  EcsWorld *w = ecs_world_create();
  ecs_resource_set_ref(w, AppMemory, memory);

  // ---- engine modules the app adds to the simulation's: delete the ones your game does not use ----
  ecs_input_init(w);
  ecs_assets_init(w, NULL);
  ecs_renderer_init(w, &(RendererInitDesc){.shadows_enabled = true, .bloom_enabled = true});
  // the hardware's quality level: without one every feature keeps its highest default
  RendererHardware hw = renderer_hardware(ecs_resource_get(w, RendererState));
  renderer_set_quality(ecs_resource_get_mut(w, RendererState), renderer_quality_for_hardware(&hw));
  // the whole frame at min(dpr, 1.25): a phone's 3x density costs pixels the eye does not resolve
  hz_set_max_dpr(1.25f);
  AssetSystem *assets = ecs_resource_get_mut(w, AssetSystem);
  RendererState *renderer = ecs_resource_get_mut(w, RendererState);
  game_sim_init(&s->sim, w, assets, &s->allocator);

  // ---- the sky: day and night on the scene's Sun (its DayNightCycle: time, day length, latitude); see wiki gameplay_blocks/day_night ----
  // DAY_NIGHT_SKY_PHYSICAL swaps the stylized sky for the physical atmosphere
  day_night_init(w, renderer, &s->allocator, DAY_NIGHT_SKY_STYLIZED);
  day_night_stylized_base(w)->haze = 0.6f; // the floor's edge fades into the sky

  // ---- the player's look: the Kaigen bot avatar with the Quaternius idle/sprint/jump clips retargeted onto its rig ----
  // (art-studio kaigen_bot[_female]); pick it with GAME_AVATAR_MALE; {0} = the block's default avatar
  character_avatar_add(w, s->sim.player,
                       &(CharacterAvatarDesc){
                           .model_path = GAME_AVATAR ".glb",
                           .idle_clip_path = GAME_AVATAR "/Idle_Loop.hza",
                           .run_clip_path = GAME_AVATAR "/Sprint_Loop.hza",
                           .fall_clip_path = GAME_AVATAR "/Jump_Loop.hza",
                       });

  // ---- UI (Clay): the HUD is built every frame after the scope, see app_update_and_render ----
  s->ui = ui_create(&(UIInitDesc){.max_glyphs = 4096}, assets);
  ecs_resource_set_ref(w, UISystem, s->ui);
  ui_system_attach_to_renderer(s->ui, renderer);

  // ---- touch: a stick on the left, a jump button on the right, drag elsewhere to look; shown once a touch arrives ----
  s->touch = touch_controls_init((TouchControlsConfig){.ui = s->ui,
                                                       .landscape = touch_controls_layout_default(TOUCH_ORIENTATION_LANDSCAPE),
                                                       .portrait = touch_controls_layout_default(TOUCH_ORIENTATION_PORTRAIT)});

#if GAME_DEV_TOOLS
  ecs_editor_default_init(&s->editor, &(EcsEditorDefaultDesc){.world = w,
                                                              .assets = assets,
                                                              .ui = s->ui,
                                                              .allocator = &s->allocator,
                                                              .canvas_width = memory->canvas_width,
                                                              .canvas_height = memory->canvas_height});
  ecs_resource_set_ref(w, GizmoSystem, gizmos_create(&(GizmoInitDesc){.assets = assets, .renderer = renderer, .sample_count = 4}));
  gizmos_attach_to_renderer(ecs_resource_get_mut(w, GizmoSystem), renderer);
#endif

  // ---- YOUR APP INIT HERE: presentation only (materials, sounds, HUD state); game state goes in game_sim_init ----

  log_info("starting template initialized");
}

// one line of HUD text
hz_internal void hud_text(HzUIString text, u16 size, HzUIColor color) {
  ui_text(text, ui_text_config({.fontSize = size, .textColor = color, .wrapMode = UI_TEXT_WRAP_NONE}));
}

// the HUD: the controls bottom left (keyboard only: touch players see the controls themselves)
hz_internal void hud_draw(const TouchControls *touch) {
  OsSafeAreaInsets safe = os_get_safe_area();
  HzUIColor panel = {16, 18, 24, 170};
  HzUIColor ink = {240, 242, 246, 255};
  HzUIColor dim = {176, 182, 194, 255};
  if (touch->last_active)
    return;
  ui_element({.id = ui_id("HudControls"),
              .floating = {.attach_to = UI_ATTACH_TO_ROOT,
                           .attach_points = {.element = UI_ATTACH_POINT_LEFT_BOTTOM, .parent = UI_ATTACH_POINT_LEFT_BOTTOM},
                           .offset = {safe.left + 16.0f, -safe.bottom - 16.0f}},
              .layout = {.layout_direction = UI_TOP_TO_BOTTOM, .padding = {.left = 14, .right = 14, .top = 10, .bottom = 10},
                         .child_gap = 4},
              .background_color = panel,
              .corner_radius = ui_corner_radius(6)}) {
    hud_text(ui_string("WASD move   Shift run   Space jump"), 14, ink);
    hud_text(ui_string("Hold right mouse to look   Wheel to zoom   R respawn"), 14, dim);
#if GAME_DEV_TOOLS
    hud_text(ui_string("F1 editor   F5 save the level"), 14, dim);
#endif
  }
}

HZ_APP_API void app_update_and_render(AppMemory *memory) {
  AppState *s = memory->state;
  EcsWorld *w = s->sim.world;
  perf_stats_begin_frame();
  // the "playable" mark publishing measures: the first frame with nothing left to load
  if (is_main_thread() && !s->playable && asset_system_idle(ecs_resource_get_mut(w, AssetSystem))) {
    s->playable = true;
    log_info("playable after % ms", fmt_u32((u32)(memory->total_time * 1000.0)));
  }
  ui_font_update(s->ui);
  f32 dt = m_minf(memory->dt, GAME_MAX_DT);

  // ---- main lane, BEFORE the scope: OS input to the simulation's controls, and dev keys. Game logic is systems ----
  if (is_main_thread()) {
    InputSystem *input = ecs_resource_get_mut(w, InputSystem);
    input_update(input, &memory->input_events, memory->total_time);
#if GAME_DEV_TOOLS
    if (input->buttons[KEY_F1].pressed_this_frame)
      s->editor_open = !s->editor_open;
    if (s->editor_open && input->buttons[KEY_F5].pressed_this_frame)
      s->sim.scene = scene_save(&s->sim.scene_sys, s->sim.scene, GAME_SCENE_PATH);
#endif
    // touch claims its fingers before anything else reads the pointer
    TouchControlsFrame tf = touch_controls_update(&s->touch, input, memory->total_time);
    b32 touch_on = tf.active && !s->editor_open;
    // keys, pad and touch to camera-relative controls; R respawns
    const CameraRigState *rig = ecs_resource_get(w, CameraRigState);
    v2 move = {(f32)input->buttons[KEY_D].is_pressed - (f32)input->buttons[KEY_A].is_pressed,
               (f32)input->buttons[KEY_W].is_pressed - (f32)input->buttons[KEY_S].is_pressed};
    move = v2_add(move, input_gamepad_stick(input->gamepads[input->active_gamepad].left_stick, 0.15f));
    move = v2_add(move, touch_on ? tf.move : (v2){0});
    PlayerControls controls = {
        .move = move,
        .heading = rig ? rig->yaw : 0.0f,
        .jump = input->buttons[KEY_SPACE].pressed_this_frame || (touch_on && tf.jump_held && !s->touch_jump_held) ||
                input->gamepads[input->active_gamepad].buttons[GAMEPAD_A].pressed_this_frame,
        .run = input->buttons[KEY_LEFT_SHIFT].is_pressed,
        .respawn = input->buttons[KEY_R].pressed_this_frame,
    };
    s->touch_jump_held = tf.jump_held;
    // hold the right mouse button to look around; the warp on the lock frame is not a look
    b32 look = !s->editor_open && input->buttons[MOUSE_RIGHT].is_pressed;
    os_lock_mouse(look);
    v2 mouse = look && s->look_locked ? (v2){input->mouse_delta.x, -input->mouse_delta.y} : (v2){0};
    v2 pan = touch_on ? (v2){tf.camera_pan_px.x, -tf.camera_pan_px.y} : (v2){0};
    *ecs_resource_get_mut(w, CameraRigInput) = (CameraRigInput){
        .look = v2_add(v2_scale(mouse, GAME_LOOK_SENSITIVITY), v2_scale(pan, GAME_TOUCH_LOOK)),
        .zoom = s->editor_open ? 0.0f
                               : -input->scroll_delta.y * GAME_ZOOM_RATE - (touch_on ? tf.camera_pinch_px * GAME_PINCH_ZOOM : 0.0f),
    };
    s->look_locked = look;
    // the editor pauses the simulation
    game_sim_begin_frame(&s->sim, &controls, dt, s->editor_open);
  }
  lane_sync(); // publishes the main-lane writes above to every lane

  // ALL lanes enter the scope. Never call is_main_thread() in here: schedule systems instead.
  ecs_scope(w) {
    day_night_schedule(w); // turns the Sun, sets the light, ambient and fog
    game_sim_schedule(&s->sim, dt);
    ecs_renderer_schedule(w);
    ecs_flush(w, dt);
  }

  // null until the scene's camera entity has loaded
  Camera *cam = ecs_query_get_singleton(Camera, s->sim.camera_query);

  // ---- main lane, AFTER the scope: renderer settings and UI. Clay is main-lane state; read what the systems published ----
  if (is_main_thread()) {
    day_night_sync(w, ecs_resource_get_mut(w, RendererState)); // the sky follows the clock
    const InputSystem *input = ecs_resource_get(w, InputSystem);
    ui_begin_frame(s->ui, input, memory->dt);
#if GAME_DEV_TOOLS
    if (s->editor_open && cam)
      ecs_editor_default_update(&s->editor, &(EcsEditorDefaultUpdateDesc){.camera = cam,
                                                                         .canvas_width = memory->canvas_width,
                                                                         .canvas_height = memory->canvas_height});
    if (cam)
      gizmos_prepare(ecs_resource_get_mut(w, GizmoSystem), &(GizmoFlushDesc){.camera = cam,
                                                                             .line_width = 2.0f,
                                                                             .screen_width = memory->canvas_width,
                                                                             .screen_height = memory->canvas_height});
#endif
    // YOUR HUD HERE: ui_* calls; read game state from resources (ecs_resource_get), never mutate the world
    if (!s->editor_open) {
      hud_draw(&s->touch);
      touch_controls_draw_stick(&s->touch);
      touch_controls_draw_jump_button(&s->touch);
#ifndef HZ_PUBLISH
      perf_stats_draw(input, memory->dt); // the editor's hierarchy sits where the stats draw
#endif
    }
    ui_finish_layout(s->ui);
  }
  lane_sync();

  // ALL lanes: the renderer records and submits in parallel
  if (cam) {
    renderer_update(ecs_resource_get_mut(w, RendererState), &(RendererBeginFrameDesc){
                                                                .camera = cam,
                                                                .clear_color = (ColorF32){0.6f, 0.75f, 0.9f, 1.0f},
                                                                .time = memory->total_time,
                                                            });
  }
  perf_stats_end_frame();
}

// main lane, after the last frame: the physics world lives outside the ECS world
void app_shutdown(AppMemory *memory) {
  AppState *s = memory->state;
  if (s->sim.world)
    ecs_rigidbody_shutdown(s->sim.world);
}

HZ_APP_API size_t app_state_size(void) { return sizeof(AppState); }
