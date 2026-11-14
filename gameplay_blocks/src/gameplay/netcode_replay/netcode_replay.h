// Netcode replay: each client's received stream replayed into a world of its own, drawn instead of the live one, saved to .hzrep
#pragma once

#include "ecs/ecs.h"
#include "gameplay/multiplayer/multiplayer.h"
#include "netcode/netcode.h"

typedef struct NetcodeReplayDesc {
  f32 ring_seconds;                            // the recording's memory ring; 0 = 10
  void (*setup)(void *user, EcsWorld *replay); // each replay frame, inside the replay world's scope: its presentation
  void *user;
} NetcodeReplayDesc;

// main lane, app_init after ecs_renderer_init: the replay world, a recording on every client from the netcode's start; --netcode-replay <file.hzrep> plays one
void netcode_replay_init(Multiplayer *mp, const NetcodeReplayDesc *d);

typedef enum {
  NETCODE_REPLAY_CAMERA_FOLLOW, // behind follow_ghost
  NETCODE_REPLAY_CAMERA_FREE,   // fly: right mouse looks, WASD moves
  NETCODE_REPLAY_CAMERA_GAME,   // the game writes the replay world's Camera
} NetcodeReplayCamera;

typedef struct NetcodeReplayStartDesc {
  f32 seconds_back; // 0 = all the ring holds
  f32 end_back;     // the clip ends and holds this many seconds before the newest frame drawn; 0 = on it
  u32 follow_ghost; // ghost id for NETCODE_REPLAY_CAMERA_FOLLOW
  NetcodeReplayCamera camera;
  f32 rate; // 0 = 1
} NetcodeReplayStartDesc;

// main lane, outside scopes: the recording's last seconds, drawn from the next netcode_replay_frame; false before a whole keyframe
b32 netcode_replay_start(Multiplayer *mp, const NetcodeReplayStartDesc *d);
// main lane, outside scopes: the live world is drawn again
void netcode_replay_stop(Multiplayer *mp);
b32 netcode_replay_active(const Multiplayer *mp);
void netcode_replay_set_rate(Multiplayer *mp, f32 rate);
void netcode_replay_set_paused(Multiplayer *mp, b32 paused);
// main lane, outside scopes: seconds into the clip
void netcode_replay_seek(Multiplayer *mp, f32 seconds);
EcsWorld *netcode_replay_world(Multiplayer *mp);
// NULL while no replay plays
NetcodePlayback *netcode_replay_playback(Multiplayer *mp);
// main lane, between frames: the recording's last `seconds` (0 = all held) to a .hzrep
b32 netcode_replay_save(Multiplayer *mp, CString path, f32 seconds);
// every lane, each frame outside scopes, after the netcode receive and before renderer_update: advance (held at the end), camera, setup, flush
void netcode_replay_frame(Multiplayer *mp, f32 dt);
