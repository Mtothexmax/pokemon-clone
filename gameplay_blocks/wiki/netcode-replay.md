---
title: Netcode replay
description: netcode_replay block — every client records what it receives from the netcode's start; the last seconds play back into a replay world of their own (drawn into the live world's renderer while the live world keeps simulating undrawn) with a follow, free or game-driven camera, rate, pause and seek; .hzrep clips saved by call or command and watched with --netcode-replay
resource: src/gameplay/netcode_replay/netcode_replay.h
tags: [gameplay, netcode, replay, killcam, recording, playback]
updated: 2026-11-09
---

# What it provides
- `netcode_replay_init(mp, &NetcodeReplayDesc{ring_seconds, setup, user})` (main lane, `app_init` after `ecs_assets_init` and `ecs_renderer_init` on the multiplayer world): the replay world, attached to the live world's renderer (`ecs_renderer_attach`, sharing its `AssetSystem` and `AppMemory`), with a `Camera` entity; a recording of the netcode's client from the first frame it exists (`ring_seconds` 0 = 10), started again when the client changes.
- `netcode_replay_start(mp, &NetcodeReplayStartDesc{seconds_back, end_back, follow_ghost, camera, rate})` (main lane, outside scopes): the recording's last `seconds_back` play from the next frame, up to `end_back` seconds before the newest frame drawn (0: up to it), where the clip holds; the main view mask switches to `ECS_RENDERER_VIEW_MASK_ATTACHED`, so the live world's models, particles and trails are not drawn. False before the recording holds a whole keyframe. `netcode_replay_stop` restores the live view.
- Cameras: `NETCODE_REPLAY_CAMERA_FOLLOW` chases `follow_ghost` (its type's replicated position, any game); `_FREE` flies on the live world's input (right mouse looks, WASD moves), starting over the recorded player; `_GAME` leaves the replay world's `Camera` to the game.
- `netcode_replay_frame(mp, dt)` (every lane, each frame outside scopes, after the netcode receive and before `renderer_update`): advances at the rate (held at the end, `end_back` before the newest frame), places FOLLOW/FREE, then inside the replay world's scope calls `setup(user, replay)` (the game's presentation systems and the renderer's stages; without one, transforms and `ecs_renderer_schedule`) and flushes on replay time (the rate's step, 0 paused; held at the end, the presentation plays on, so a death resolves). The game draws with the replay world's camera while `netcode_replay_active`.
- `set_rate`, `set_paused`, `seek` (main lane), `netcode_replay_world` (fill it once after init: the level, the game's resources), `netcode_replay_playback` (the engine playback: tick drawn, ghost entities, received states).
- `netcode_replay_save(mp, path, seconds)` and the command `netcode_replay_save:<path>[:<seconds>]` (a trailing number is the seconds); `--netcode-replay <file.hzrep>` plays a saved clip with the free camera from the first frame (refused, logged, on another protocol or other types).

# Example usage
```c
// app_init, after the renderer: Kestrel's replay world shows the ship, its soldiers posed between the pose and skin stages
netcode_replay_init(mp, &(NetcodeReplayDesc){.setup = game_replay_setup, .user = game});
scene_load_file(&replay_scenes, "game/level.hzt"); // a SceneSystem made on netcode_replay_world(mp)
// main lane, after the death beat: the 3 s up to the kill drawn `lag` seconds ago, the camera the game's
netcode_replay_start(mp, &(NetcodeReplayStartDesc){.seconds_back = 3.0f + lag, .end_back = lag, .follow_ghost = killer, .camera = NETCODE_REPLAY_CAMERA_GAME});
// every lane, every frame, after the receive and the live scope
netcode_replay_frame(mp, dt);
Camera *cam = netcode_replay_active(mp) ? replay_camera : live_camera;
```
Kestrel's killcam (`templates/kestrel/src/kestrel/ks_killcam.c`) is the reference user: its replay world is a whole presentation world for the killer (the weapon and viewmodel from the server's reel, the effects with `ECS_RENDERER_VIEW_MASK_ATTACHED` and their own feature names, the soldiers, the sound on `AUDIO_BUS_REPLAY` heard from the killer's eye while the live mix is ducked).

# Gotchas
- The replay world holds only what the playback spawns and what the game puts in it: load the level into it yourself; lights, sun, sky settings, post, probes and decals stay the live world's (an attached world never syncs them), and a sky the game spawns at runtime must be spawned there too.
- A replay draws only what carries the attached view: the live world's particles, trails, vfx decals and lights, casings batch and models are hidden; level decals and lamps (view mask 0) show in both. A game whose replay needs effects gives the replay world effect systems of its own with the attached mask.
- The clip ends on the newest frame this client drew, its interpolation tick, unless `end_back` moves the end earlier: to end on a killing tick drawn `lag` seconds ago, start with `seconds_back = length + lag` and `end_back = lag`. Kestrel starts 1.2 s after the death this way, with `lag` from the interpolation tick past the killing tick. `netcode_playback_duration` stays the whole clip's: the hold is at `duration - end_back`.
- Scene ghosts and the field are not recorded (the engine refuses or stops the recording, logged).
- A role switch resets the block: `netcode_replay_reset` (internal, from `multiplayer_frame` before the netcode closes) ends a playback and the recording. The next netcode's client is recorded from its first frame. A pointer compare is not used, because a new client can reuse the old one's address.
- `start` is main lane: lane_sync before the frame's `netcode_replay_frame`, which every lane must reach with the same answer to `netcode_replay_active`.
