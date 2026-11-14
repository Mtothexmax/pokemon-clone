// The game's simulation, callable without a window or GPU: a headless run steps it at a fixed dt (tests/, sim_run block).
#pragma once

#include "ecs/ecs.h"
#include "lib/hzmath.h"
#include "lib/typedefs.h"

// one frame of player controls: the app reads them from keys, pad and touch; a bot in a headless run writes them itself
typedef struct PlayerControls {
  v2 move;     // -1..1 strafe, forward, relative to heading
  f32 heading; // radians; the app passes the camera's yaw
  b32 jump;
  b32 run;
  b32 respawn;
} PlayerControls;
ECS_COMPONENT_TYPEDEF(PlayerControls);

typedef struct GameSim GameSim;

// main lane: a world with physics, the character and the camera rig; loads the level scene (see game_sim_ready)
GameSim *game_sim_create(void);
// main lane
void game_sim_destroy(GameSim *sim);
// main lane: the level has loaded and is solid; frames before this load assets only
b32 game_sim_ready(GameSim *sim);
// all lanes, lock step: one frame of exactly dt
void game_sim_frame(GameSim *sim, const PlayerControls *controls, f32 dt);
// main lane: the player's feet
v3 game_sim_player_position(GameSim *sim);
