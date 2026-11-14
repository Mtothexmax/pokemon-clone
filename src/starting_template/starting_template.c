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
#include "gameplay/mesh_builder/mesh_builder.h"
#include "ecs/audio/ecs_audio.h"
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
#define GAME_MAX_SUBSTEPS 8 // catch-up at 60 Hz down to 7.5 fps: a slow phone must not slow the world down
// wall-clock dt is not clamped by the engine: a hitch must not become one giant simulation step. Only real hitches:
// a phone drawing at 10-20 fps must still move the player at full speed (fixed steps, not a slower clock)
#define GAME_MAX_DT (1.0f / 8.0f)
#define GAME_LOOK_SENSITIVITY 0.003f // radians per mouse pixel
#define GAME_TOUCH_LOOK 0.005f       // radians per dragged point
#define GAME_ZOOM_RATE 0.1f
#define GAME_PINCH_ZOOM 0.02f // zoom per point of pinch spread
#define GAME_SPAWN ((v3){-5.7f, 0.3f, 0.2f}) // on the road in front of your house (stud -15, 0.5)
#define GAME_FALL_Y -20.0f // below this the player respawns

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
    if (c->teleport) {
      ci->teleport = true;
      ci->teleport_pos = c->teleport_pos;
      ci->teleport_yaw = c->teleport_yaw;
    } else if (c->respawn || wt->position.y < GAME_FALL_Y) {
      ci->teleport = true;
      ci->teleport_pos = GAME_SPAWN;
      ci->teleport_yaw = 0.0f;
    }
  }
}

// resource: how much of the sun reaches you (1 outside; inside the rooms the warm ceiling lights take over)
typedef struct IndoorLight {
  f32 sun_scale;
} IndoorLight;
ECS_COMPONENT_TYPEDEF(IndoorLight);

// per sun, after day_night_schedule wrote this frame's light: dimmed indoors
ecs_system(indoor_light_system, EcsInOut(EcsDirectionalLight), EcsResIn(IndoorLight)) {
  const IndoorLight *il = ecs_sys_get_resource(IndoorLight);
  for (i32 i = 0; i < it->count; i++)
    ecs_sys_get_component_mut(EcsDirectionalLight, i)->intensity *= il->sun_scale;
}

// the scene's ambient light: indoors darker and warm, so the ceiling lights carry the room
ecs_system(indoor_ambient_system, EcsInOut(EcsSceneSettings), EcsResIn(IndoorLight)) {
  const IndoorLight *il = ecs_sys_get_resource(IndoorLight);
  f32 in = 1.0f - il->sun_scale; // 0 outside .. ~1 inside
  for (i32 i = 0; i < it->count; i++) {
    EcsSceneSettings *ss = ecs_sys_get_component_mut(EcsSceneSettings, i);
    ss->ambient_intensity *= 1.0f - 0.7f * in;
    ss->ambient_color.r = m_lerpf(ss->ambient_color.r, 1.0f, in * 0.5f);
    ss->ambient_color.g = m_lerpf(ss->ambient_color.g, 0.86f, in * 0.5f);
    ss->ambient_color.b = m_lerpf(ss->ambient_color.b, 0.7f, in * 0.5f);
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
#include "st_town.c"
#include "st_minifig.c"
#include "st_sound.c"
#include "st_save.c"
#include "st_cards.c"
#include "st_dialog.c"
#include "st_battle.c"
#include "st_cardui.c"
#include "st_world.c"
#include "st_life.c"
#include "st_cardgame.c"
#include "st_vox.c"
#include "st_screens.c"
#include "st_debug.c"

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
  sim->sim_clock.max_frame_dt = GAME_MAX_DT; // the accumulator keeps the whole (clamped) frame, see GAME_MAX_DT

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
  AudioSystem audio;
  MinifigSet figs; // the LEGO player and the villagers
  WorldState world; // doors, stairs, signs, the area title
  SoundBank snd;
  SaveState save;
  DialogState dlg;
  Life life;       // golden studs, ducks, ball, weather, Pip... (st_life.c)
  Screens scr;     // TV, piano, PCs
  TownRect blocks[LIFE_MAX_BLOCKS];
  TownGroups groups;   // what is drawn per area (st_town.c)
  TownTag tags[TOWN_MAX_TAGS]; // names for the debug picker
  u32 tag_count;
  SignZoom zoom;       // the camera at a sign's board
  DebugPick dbg;
  i32 shown_area;      // the area the groups were last shown for
  f32 cam_lift;        // radians the camera is raised because something hides the player
  u32 block_count;
  b32 intro_checked;
  b32 dancing;
  CardGame cards;      // battles, rewards, the deck menu (st_cardgame.c)
  Vox vox;             // your Voxelmon and the fights in the tall grass (st_vox.c)
  b32 gate_shown;      // the gate prof stands where the save says
  b32 mom_talked, kid_talked; // for the "!" markers
  char dlg_buf[128];   // a dialogue line built from the save
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
  ecs_renderer_init(w, &(RendererInitDesc){.shadows_enabled = true, .bloom_enabled = true, .max_lights = 32}); // the rooms' ceiling lights
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

  // ---- the player's look: a LEGO minifigure (st_minifig.c) hung under the character, posed every frame ----
  fig_attach(&s->figs, w, s->sim.player, FIG_STYLE_PLAYER, true, 0.0f);
  // the villagers, where they stood in the three.js village (studs * TOWN_STUD; the road plates are 0.4 studs high)
  fig_spawn_villager(&s->figs, w, FIG_STYLE_DEX, town_m(52.5f, 0.0f, -3.4f), -M_PI * 0.5f);
  Minifig *pip = fig_spawn_villager(&s->figs, w, FIG_STYLE_PIP, town_m(-8.0f, 0.0f, 13.2f), 0.0f);
  fig_spawn_villager(&s->figs, w, FIG_STYLE_QUILL, town_m(-5.4f, 0.0f, -24.2f), 0.4f);
  // in the rooms: Mom in the kitchen, Lumi at home, the Professor in her lab (the floors are 0.4 studs high)
  fig_spawn_villager(&s->figs, w, FIG_STYLE_MOM, town_m(193.4f, 0.4f, -4.4f), 0.0f);
  fig_spawn_villager(&s->figs, w, FIG_STYLE_LUMI, town_m(254.5f, 0.4f, 2.0f), M_PI * 0.5f);
  fig_spawn_villager(&s->figs, w, FIG_STYLE_QUILL, town_m(337.0f, 0.4f, -1.2f), -M_PI * 0.5f);

  // ---- UI (Clay): the HUD is built every frame after the scope, see app_update_and_render ----
  s->ui = ui_create(&(UIInitDesc){.max_glyphs = 4096}, assets);
  ecs_resource_set_ref(w, UISystem, s->ui);
  ui_system_attach_to_renderer(s->ui, renderer);

  // ---- touch: a stick on the left, a jump button on the right, drag elsewhere to look; shown once a touch arrives ----
  // touch button 0 reads a sign (shown only in reach of one), left of the jump button / above it in portrait
  TouchControlsConfig touch = {.ui = s->ui,
                               .landscape = touch_controls_layout_default(TOUCH_ORIENTATION_LANDSCAPE),
                               .portrait = touch_controls_layout_default(TOUCH_ORIENTATION_PORTRAIT)};
  touch.buttons[0] = (TouchButtonDesc){.enabled = true, .label = "Use"};
  touch.landscape.buttons[0] = (TouchButtonPlacement){.anchor = TOUCH_ANCHOR_BOTTOM_RIGHT, .corner_offset_px = {200.0f, 70.0f},
                                                      .diameter_px = 86.0f};
  touch.portrait.buttons[0] = (TouchButtonPlacement){.anchor = TOUCH_ANCHOR_BOTTOM_RIGHT, .corner_offset_px = {80.0f, 210.0f},
                                                     .diameter_px = 86.0f};
  s->touch = touch_controls_init(touch);
  s->world = (WorldState){.area = -2, .nearby = -1};

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
  // the town: one merged vertex-coloured mesh plus its static colliders
  town_build(w, renderer, &s->allocator, s->blocks, &s->block_count, LIFE_MAX_BLOCKS, &s->groups, s->tags, &s->tag_count);
  // the villagers in the rooms belong to their room's group (figs 4, 5, 6: Mom, Lumi, the Professor)
  RoomId fig_room[3] = {ROOM_HOME, ROOM_DEX, ROOM_LAB};
  for (u32 i = 0; i < 3; i++)
    for (u32 k = 0; k < FIG_PART_COUNT; k++)
      town_group_add(&s->groups, 1 + fig_room[i], s->figs.figs[4 + i].part[k]);
  s->shown_area = -99;
  ecs_resource_set(w, IndoorLight, {.sun_scale = 1.0f});
  rooms_lights(w);
  // background music: the town theme, looping on the music bus (the web starts it on the first tap or key)
  s->audio = audio_system_init(&s->allocator, NULL);
  ecs_audio_init(&s->audio, w);
  sound_init(&s->snd, &s->audio, &s->allocator);
  save_begin(&s->save);
  life_init(&s->life, w, renderer, &s->snd, &s->save, &s->audio, &s->allocator, s->blocks, s->block_count, pip);
  cg_init(&s->cards, w, &s->audio, (u32)(os_time_unix_ms() & 0xffffffffu));
  vox_init(&s->vox, &s->life, (u32)(os_time_unix_ms() >> 3));

  log_info("brickhollow initialized");
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
    hud_text(ui_string("Brickhollow"), 18, ink);
    hud_text(ui_string("WASD move   Shift run   Space jump   E use / talk   Q dance"), 14, ink);
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
    if (input->buttons[KEY_F9].pressed_this_frame) { // dev builds: 200 EXP to try the upgrades and the Voxelmon
      s->save.data.exp += 200;
      save_mark(&s->save);
    }
    if (input->buttons[KEY_F8].pressed_this_frame && !cg_busy(&s->cards)) { // dev builds: a practice battle (starter deck if needed)
      if (!s->save.data.has_deck)
        give_starter(&s->save.data);
      cg_start_battle(&s->cards, BATTLE_PRACTICE, w, &s->audio);
    }
#endif
    // touch claims its fingers before anything else reads the pointer
    touch_controls_set_button(&s->touch, 0, s->world.nearby >= 0 || s->dlg.open || s->scr.kind != SCREEN_NONE, s->world.reading);
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
    // ---- the game around the walking: the save, dialogues, screens, things to use, doors, village life ----
    save_update(&s->save, dt);
    const WorldTransform *pwt = ecs_component_get(w, s->sim.player, WorldTransform);
    const Character *pch = ecs_component_get(w, s->sim.player, Character);
    Minifig *me = &s->figs.figs[0];
    b32 use = input->buttons[KEY_E].pressed_this_frame || (touch_on && tf.buttons[0].pressed);
    b32 advance = use || input->buttons[KEY_SPACE].pressed_this_frame || input->buttons[KEY_ENTER].pressed_this_frame;
    // a new game wakes up in your room upstairs, with a thought about the day
    if (!s->intro_checked && s->save.loaded && s->playable && memory->total_time > 1.0) {
      s->intro_checked = true;
      if (!s->save.data._reserved[3]) {
        s->save.data._reserved[3] = 1; // the intro was seen
        save_mark(&s->save);
        controls.teleport = true;
        controls.teleport_pos = town_m(197.4f, 0.5f, 41.2f);
        controls.teleport_yaw = M_PI;
        dialog_open(&s->dlg, dialog_script(s->touch.last_active ? DLG_INTRO_TOUCH : DLG_INTRO_KEYS), w, &s->audio);
      }
    }
    b32 busy = s->dlg.open || s->scr.kind != SCREEN_NONE || s->life.fade;
    if (pwt && !s->editor_open) {
      if (s->dlg.open) {
        dialog_update(&s->dlg, w, &s->audio, input, advance);
        if (!s->dlg.open) {
          DialogAction act = s->dlg.chosen;
          s->dlg.chosen = ACT_NONE;
          if (act == ACT_HIDE_SEEK)
            life_hide_seek_start(&s->life);
          else if (act == ACT_TAG)
            life_toast(&s->life, "Tag! Sprint into Pip (hold Shift) and run!");
          else if (act == ACT_OPEN_GATE) {
            s->save.data.gate_open = true;
            save_mark(&s->save);
            sound_play(&s->snd, SND_FANFARE, 1.0f, 0.8f);
            life_toast(&s->life, "The way to Route 1 is open!");
            s->gate_shown = false;
          } else
            cg_dialog_action(&s->cards, act, &s->save, &s->dlg, &s->snd, w, &s->audio);
        }
        use = false;
      } else if (s->scr.kind != SCREEN_NONE) {
        screens_update(&s->scr, w, &s->audio, &s->snd, input, dt);
        use = false;
      }
      // the cards: a battle, the rewards or the deck menu own the screen
      b32 cards = cg_update(&s->cards, &s->save, &s->life, &s->dlg, &s->snd, w, &s->audio, input, busy, dt);
      if (cards)
        use = false;
      busy |= cards;
      // your Voxelmon: it follows you; in the tall grass a fight holds you in place
      b32 fight = vox_update(&s->vox, &s->save, &s->cards, w, &s->audio, input, pwt->position, controls.heading, s->world.area, busy, dt);
      vox_reward(&s->vox, &s->save, &s->cards);
      vox_after(&s->vox, &s->cards, dt);
      if (fight)
        use = false;
      busy |= fight;
      // the gap north: Prof. Quill blocks it until you have a Voxelmon (then she steps aside)
      if (!s->gate_shown && s->save.loaded) {
        s->gate_shown = true;
        b32 open = s->save.data.gate_open;
        g_gate_prof_x = open ? -5.4f : 0.0f;
        fig_place(w, &s->figs.figs[3], open ? town_m(-5.4f, 0.0f, -24.2f) : town_m(0.0f, 0.0f, -24.4f), open ? 0.4f : 0.0f);
      }
      {
        f32 gx = pwt->position.x / TOWN_STUD, gz = pwt->position.z / TOWN_STUD;
        if (!s->save.data.gate_open && gz < -25.3f && gz > -27.5f && m_absf(gx) < 5.0f && !s->dlg.open) {
          controls.teleport = true;
          controls.teleport_pos = town_m(gx, 0.2f, -22.6f);
          controls.teleport_yaw = 0.0f;
          dialog_open(&s->dlg, dialog_for(DLG_GATE, &s->save.data, s->save.data.voxelmon > 0, s->dlg_buf, sizeof(s->dlg_buf)), w, &s->audio);
        }
      }
      v3 pip_studs = s->life.pip_p;
      const Target *t = busy ? NULL
                             : world_pick(&s->world, pwt->position, use, input->buttons[KEY_ESCAPE].pressed_this_frame, pip_studs,
                                          s->life.chest_live && !s->life.chest_open, s->life.chest_p);
      if (t) {
        u32 weather = s->life.rain > 0.5f ? 2u : s->life.night > 0.5f ? 1u : 0u;
        switch (t->kind) {
        case T_TALK:
          dialog_open(&s->dlg, dialog_for((DialogId)t->arg, &s->save.data, s->save.data.voxelmon > 0, s->dlg_buf, sizeof(s->dlg_buf)), w,
                      &s->audio);
          s->mom_talked |= t->arg == DLG_MOM;
          s->kid_talked |= t->arg == DLG_PIP;
          if (t->arg == DLG_PIP)
            s->life.pip_yaw = m_atan2f(pwt->position.x / TOWN_STUD - s->life.pip_p.x, pwt->position.z / TOWN_STUD - s->life.pip_p.z);
          break;
        case T_FEED: life_feed(&s->life, v3_scale(pwt->position, 1.0f / TOWN_STUD)); break;
        case T_SIT:
          life_sit(&s->life, true);
          controls.teleport = true;
          controls.teleport_pos = town_m(15.0f, 0.1f, -8.3f);
          controls.teleport_yaw = 0.0f;
          break;
        case T_SLEEP:
          life_sleep(&s->life);
          s->vox.me.hp = s->vox.me.max; // a good night: your Voxelmon is fit again
          s->vox.me.tired = false;
          break;
        case T_TV: screens_open(&s->scr, SCREEN_TV, w, &s->audio, weather); break;
        case T_PIANO: screens_open(&s->scr, SCREEN_PIANO, w, &s->audio, weather); break;
        case T_PC: screens_open(&s->scr, SCREEN_PC, w, &s->audio, weather); break;
        case T_LAB: screens_open(&s->scr, SCREEN_LAB, w, &s->audio, weather); break;
        case T_CHEST: life_open_chest(&s->life); break;
        default: break;
        }
        sound_play(&s->snd, SND_BLIP, 1.2f, 0.4f);
      }
      busy = s->dlg.open || s->scr.kind != SCREEN_NONE || s->life.fade || cg_busy(&s->cards) || fight;
      b32 moving = v2_length(controls.move) > 0.2f;
      // the bench: any move gets you up; Q dances (moving stops it)
      if (s->life.sitting && moving && !busy)
        life_sit(&s->life, false);
      if (input->buttons[KEY_Q].pressed_this_frame && !busy && !s->life.sitting) {
        s->dancing = !s->dancing;
        if (s->dancing) {
          sound_play(&s->snd, SND_FANFARE, 1.5f, 0.5f);
          life_toast(&s->life, "Dance party! Everyone joins in.");
        }
      }
      if (moving || busy)
        s->dancing = false;
      if (busy || s->life.sitting) {
        controls.move = (v2){0};
        controls.jump = false;
      }
      if (fight && v2_length(s->vox.walk) > 0.01f) { // step back from the fight (a world direction to camera-relative)
        f32 hh = controls.heading;
        v2 fw = {-m_sinf(hh), -m_cosf(hh)}, rt = {m_cosf(hh), -m_sinf(hh)};
        controls.move = (v2){v2_dot(s->vox.walk, rt), v2_dot(s->vox.walk, fw)};
      }
      me->pose = s->life.sitting ? FIG_POSE_SIT : s->dancing ? FIG_POSE_DANCE : FIG_POSE_NORMAL;
      me->spin = s->dancing ? me->spin + dt * 2.6f : 0.0f;
      // the villagers within 10 studs dance along (Pip spins)
      for (u32 i = 1; i < s->figs.count; i++) {
        Minifig *v = &s->figs.figs[i];
        if (v == s->life.pip)
          continue;
        const WorldTransform *vw = ecs_component_get(w, v->root, WorldTransform);
        b32 close_by = vw && v3_length(v3_sub(vw->position, pwt->position)) < 10.0f * TOWN_STUD;
        v->pose = s->dancing && close_by ? FIG_POSE_ARMS_UP : FIG_POSE_NORMAL;
      }
      // where the player walks (for kicking the ball) and the village around
      f32 h = controls.heading;
      v2 fwd = {-m_sinf(h), -m_cosf(h)}, right = {m_cosf(h), -m_sinf(h)};
      v2 dir = v2_add(v2_scale(right, controls.move.x), v2_scale(fwd, controls.move.y));
      f32 dl = v2_length(dir);
      if (dl > 1e-3f)
        dir = v2_scale(dir, 1.0f / dl);
      f32 speed = pch ? character_planar_speed(pch) : 0.0f;
      b32 outside = s->world.area < 0;
      life_update(&s->life, pwt->position, dir, speed, controls.run && moving, outside, dt);
      if (s->dancing && s->life.pip_mode == PIP_WALK) {
        s->life.pip->pose = FIG_POSE_ARMS_UP;
        s->life.pip->spin += dt * 4.0f;
      } else
        s->life.pip->spin = 0.0f;
      if (!busy && !s->life.sitting)
        world_doors(&s->world, pwt->position, &controls, dt);
      EcsEntity cam_e = ecs_query_get_singleton_entity(s->sim.camera_query);
      const CameraFollow *follow = cam_e != ECS_ENTITY_INVALID ? ecs_component_get(w, cam_e, CameraFollow) : NULL;
      // reading a sign with a board: the camera flies to it (moving closes it)
      if (s->world.reading && v2_length(controls.move) > 0.3f)
        s->world.reading = false;
      b32 zoomed = sign_zoom_update(&s->zoom, w, s->sim.camera_query, s->world.reading && s->world.target.kind == T_READ,
                                    s->world.target.arg, dt);
      if (zoomed) {
        controls.move = (v2){0};
        controls.jump = false;
      }
      // the figure would stand between the camera and the board
      {
        b32 show_me = !(s->zoom.active && s->zoom.t > 0.25f);
        for (u32 k = 0; k < FIG_PART_COUNT; k++) {
          const EcsVisibility *v = ecs_component_get(w, me->part[k], EcsVisibility);
          if (v && v->visible != show_me) {
            EcsVisibility nv = *v;
            nv.visible = show_me;
            ecs_component_set(w, me->part[k], EcsVisibility, nv);
          }
        }
      }
      // debug: ` toggles the picker, a click names what is under the mouse
      b32 dbg_toggled = input->buttons[KEY_BACKQUOTE].pressed_this_frame;
      if (dbg_toggled)
        s->dbg.on = !s->dbg.on;
      // a click names the thing under the mouse; turning debug on names what is in the middle of the screen
      if (s->dbg.on && (input->buttons[MOUSE_LEFT].pressed_this_frame || dbg_toggled)) {
        Camera *pick_cam = ecs_query_get_singleton(Camera, s->sim.camera_query);
        f32 mx = dbg_toggled ? (f32)memory->canvas_width * 0.5f : input->mouse_pos.x;
        f32 my = dbg_toggled ? (f32)memory->canvas_height * 0.5f : input->mouse_pos.y;
        if (pick_cam)
          debug_pick(&s->dbg, w, pick_cam, mx, my, memory->canvas_width, memory->canvas_height, s->tags, s->tag_count, &s->life,
                     &s->figs, s->sim.player);
      }
      // the camera never zooms in on an obstacle: when something hides the player it rises and looks down over it,
      // and sinks back once the view is free (a ray from the head to the camera, checked a step lower)
      const CameraRigState *rs = ecs_resource_get(w, CameraRigState);
      const EcsPhysicsWorld *pw = ecs_resource_get(w, EcsPhysicsWorld);
      if (follow && !s->zoom.active) {
        CameraFollow f = *follow;
        f32 base = f.pitch - s->cam_lift;
        f32 lift = s->cam_lift;
        if (rs && rs->ready && pw && !controls.teleport) {
          v3 head = v3_add(pwt->position, (v3){0.0f, f.focus_height, 0.0f});
          EcsPhysicsQuery q = {.ignore = s->sim.player};
          f32 dist = m_maxf(rs->distance, 1.0f);
          f32 sy = m_sinf(f.yaw), cy = m_cosf(f.yaw);
          v3 at, at_lower;
          {
            f32 p = base + lift, cp = m_cosf(p);
            at = v3_add(head, v3_scale((v3){sy * cp, m_sinf(p), cy * cp}, dist));
            f32 pl = base + m_maxf(0.0f, lift - 0.15f), cpl = m_cosf(pl);
            at_lower = v3_add(head, v3_scale((v3){sy * cpl, m_sinf(pl), cy * cpl}, dist));
          }
          EcsPhysicsHit hit;
          b32 blocked = ecs_physics_raycast(pw, head, at, &q, &hit);
          if (blocked)
            lift = m_minf(lift + dt * 0.8f, m_maxf(0.0f, 1.48f - base)); // rises gently (was 1.8 rad/s)
          else if (lift > 0.0f && !ecs_physics_raycast(pw, head, at_lower, &q, &hit))
            lift = m_maxf(0.0f, lift - dt * 0.7f);
        }
        if (controls.teleport && !s->life.sitting) {
          f.yaw = 0.0f; // the rooms and the houses face the camera (south); look north again
          f.snap = true;
          lift = 0.0f;
        }
        f.boom_radius = 0.0f;
        f.pitch = base + lift;
        s->cam_lift = lift;
        ecs_component_set(w, cam_e, CameraFollow, f);
      }
      // inside a room the sun is only a soft remainder: the warm ceiling lights light it
      IndoorLight *il = ecs_resource_get_mut(w, IndoorLight);
      f32 want_sun = s->world.area >= 0 ? 0.04f : 1.0f;
      il->sun_scale += (want_sun - il->sun_scale) * m_minf(1.0f, dt * 6.0f);
      // draw only the village outside, only this room inside
      if (s->world.area != s->shown_area) {
        s->shown_area = s->world.area;
        for (u32 g = 0; g < TOWN_GROUPS; g++) {
          b32 on = s->world.area < 0 ? g == 0 : g == (u32)(s->world.area + 1);
          for (u32 i = 0; i < s->groups.count[g]; i++) {
            const EcsVisibility *v = ecs_component_get(w, s->groups.ents[g][i], EcsVisibility);
            if (v && v->visible != on) {
              EcsVisibility nv = *v;
              nv.visible = on;
              ecs_component_set(w, s->groups.ents[g][i], EcsVisibility, nv);
            }
          }
        }
      }
    }
    // anti-aliasing follows the automatic quality: 4x MSAA on HIGH and ULTRA, off below (phones keep their frame rate)
    {
      RendererState *rs = ecs_resource_get_mut(w, RendererState);
      RendererQuality q = renderer_quality(rs);
      u32 want = q >= RENDERER_QUALITY_HIGH && q < RENDERER_QUALITY_COUNT ? 4u : 1u;
      if ((renderer_supported_sample_counts(rs) & want) && want != renderer_sample_count(rs) && want != renderer_pending_sample_count(rs))
        renderer_set_sample_count(rs, want);
    }
    // the editor pauses the simulation
    game_sim_begin_frame(&s->sim, &controls, dt, s->editor_open);
    fig_animate(&s->figs, w, s->sim.player, dt);
  }
  lane_sync(); // publishes the main-lane writes above to every lane

  // ALL lanes enter the scope. Never call is_main_thread() in here: schedule systems instead.
  ecs_scope(w) {
    day_night_schedule(w); // turns the Sun, sets the light, ambient and fog
    ecs_schedule(w, indoor_light_system);
    ecs_schedule(w, indoor_ambient_system);
    ecs_audio_schedule(w);
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
      f32 ui_w = (f32)memory->canvas_width / m_maxf(memory->dpr, 0.5f), ui_h = (f32)memory->canvas_height / m_maxf(memory->dpr, 0.5f);
      b32 cards_screen = s->cards.in_battle || s->cards.ui.deck_open;
      if (!cards_screen) {
        hud_draw(&s->touch);
        b32 busy_ui = s->dlg.open || s->scr.kind != SCREEN_NONE || cg_busy(&s->cards);
        world_draw(&s->world, s->touch.last_active, busy_ui);
        life_draw(&s->life);
      }
      if (!cards_screen && !s->dlg.open && s->scr.kind == SCREEN_NONE && s->vox.phase == VOX_NONE && s->world.area < 0 && !s->zoom.active)
        cg_quest_draw(&s->cards, &s->save.data);
      if (!cards_screen && !s->zoom.active) {
        const WorldTransform *mp = ecs_component_get(w, s->sim.player, WorldTransform);
        u32 gold = 0;
        for (u32 k = 0; k < GOLDEN_COUNT; k++)
          gold += (s->save.data.golden_mask >> k) & 1u;
        if (mp)
          cg_markers_draw(&s->save.data, &s->figs, w, cam, mp->position, ui_w, ui_h, m_maxf(memory->dpr, 0.5f), s->mom_talked, s->kid_talked, gold);
      }
      if (!cards_screen)
        vox_draw(&s->vox, input, cam, ui_w, ui_h, m_maxf(memory->dpr, 0.5f), s->world.area < 0 && !s->dlg.open);
      cg_draw(&s->cards, &s->save.data, input, ui_w, ui_h, m_maxf(memory->dpr, 0.5f), !s->dlg.open && s->scr.kind == SCREEN_NONE && s->vox.phase == VOX_NONE);
      dialog_draw(&s->dlg, input, s->touch.last_active, ui_w);
      screens_draw(&s->scr, input, &s->save.data);
      debug_draw(&s->dbg);
      touch_controls_draw_stick(&s->touch);
      touch_controls_draw_jump_button(&s->touch);
      touch_controls_draw_button(&s->touch, 0);
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
