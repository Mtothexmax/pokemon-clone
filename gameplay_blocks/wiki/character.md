---
title: Character
description: does the engine have a walking, jumping, swimming character controller with moving and one-way platforms, conveyors, step-up and a skinned avatar driver — the kinematic capsule or rigidbody mover, its tuning, input and frame order, the world water query, and opt-in networking (CharacterNet, character_net_init, character_net_schedule) with owner prediction
resource: src/gameplay/character/character.h
tags: [gameplay, character, physics, controller]
updated: 2026-11-07
---

# What it provides

- Two movers on one component set: `CHARACTER_MOVER_KINEMATIC` (swept capsule, collide-and-slide over contact planes, steep slopes as
  walls, step-up, stick-to-floor) and `CHARACTER_MOVER_RIGIDBODY` (a rotation-locked dynamic body steered through its
  velocity, so `PhysicsSensor` and other bodies see it). The kinematic mover needs no rigidbody of its own, but with `ecs_rigidbody`
  initialised it moves in that world: dynamic bodies block it and it pushes them. `CharacterTuning.disable_push` keeps the blocking and
  drops the pushing.
- `CharacterTuning` (reflected, scene-authorable): speeds, ground control by `CHARACTER_MOVE_TIME_TO_SPEED`, `CHARACTER_MOVE_QUAKE` or
  `CHARACTER_MOVE_DAMPED` (exponential approach at `move_damping`, `air_control` off the ground), `disable_riding`,
  gravity, jump height, coyote time and jump buffer, slope limit, step height, facing mode (movement, heading, input), swim.
  Zero fields take the documented defaults, in meters.
- `CharacterInput` is the per-frame intent the game writes: `move` (in the heading frame), `heading`, `run`, `jump`, `teleport`,
  `add_velocity` (jump pad, knockback). `Character` is runtime state; its counters (`jumps`, `landings`, `footsteps`, `launches`,
  `teleports`) only grow, so a consumer diffs against the value it saw last.
- Crouch: `crouch_height` > 0 enables it. `CharacterInput.crouch` held drops the capsule to `crouch_height` at
  `crouch_speed` with no running; releasing it stands only when a capsule cast up finds room. `Character.crouched` and the
  eased `crouch_k` are the state; `character_eye_height(t, c)` is the eye the first-person rig uses.
- Platforms: the character rides the body it stands on and carries its pose delta; `CharacterOneWay` marks a
  platform solid from above only; `CharacterPassThrough` makes a body invisible to the character (held or thrown props never
  push, lift or carry the player); `CharacterSurface{local_velocity}` makes a conveyor or bounce pad.
- `character_avatar_add` attaches a skinned model with idle, run and fall clips driven from the `Character` state (default model in
  `assets/gameplay_blocks/character/`). `character_spawn` builds a character entity in code; a scene may author `CharacterTuning` instead.
- `character_spawn_height(w, x, z, from_y, max_drop, fallback)` finds the floor under a spawn point.
- Water: a `swim_enabled` character swims below `water_height` everywhere, or, once `character_water_set(w, fn, ctx)`
  gives the world a `CharacterWaterFn`, in whatever water the query reports at its feet (lakes at different heights,
  rivers, pools). The query returns false where there is no water to swim in and runs on any lane at once, so it only
  reads. `CharacterWorld.water` is the same for `character_kinematic_step_v2`.
- Networking, opt-in: `character_net_init(w, &tuning)` on each world that steps (the server's and a predicting
  client's) and `character_net_schedule(w, server)` in the tick, after the game writes the ghost's `CharacterInput`.
  The ghost carries `CharacterNet` (`NET_COMPONENT`: pose, ground, crouch and swim for everyone; velocity, normal,
  timers, crouch ease and the moving platform's pose for the owner) and a `NET_LOCAL CharacterInput`. Every value
  snaps to its wire grid (`CHARACTER_NET_*_GRID`) before and after each step, so the owner's prediction restores the
  server's state bit for bit. Walking, running, jumping, crouch, swimming, one-way and moving platforms, conveyors and
  stairs all work networked. `character_net_step` is one step for a system of the game's own;
  `character_net_at(feet, yaw)` is the server's spawn state.

- Without a scope: `character_step(w, dt)` and `character_frame(w)` are `character_schedule_fixed` / `character_schedule_frame` with no
  flush, called by every lane outside an `ecs_scope`. Put a `lane_sync()` before `character_step` and before the `ecs_rigidbody_step`
  that follows it; `character_frame` goes after the fixed steps.
- One kinematic step for a netcode system: `character_kinematic_step(w, resolved, &c, &in, &wt, dt, pushes)` runs one character from
  state you own and writes back `Character`; `pushes` (may be NULL) is a `CharacterV2Pushes` that `character_v2_apply_pushes(world,
  pushes, count)` applies to the dynamic bodies in character order, one caller, after every character stepped.
  `character_kinematic_step_v2(&CharacterWorld, ...)` is the same step on a physics world you pass in, with `one_way` and
  `surface_velocity` callbacks (either may be NULL).
- Helpers: `character_tuning_resolve(t)` (every zero field replaced by its default), `character_stand_height`,
  `character_planar_speed(c)` (relative to the ground), `character_grounded(c)`.

# Example usage

```c
// init, main lane, after ecs_assets_init and ecs_physics_init: mandatory
character_controller_init(w);
EcsEntity player = character_spawn(w, "Player", (v3){0, 0.1f, 0}, &(CharacterTuning){.walk_speed = 4, .run_speed = 7});
character_avatar_add(w, player, &(CharacterAvatarDesc){0});

// frame, one ecs_scope: the input system writes CharacterInput (heading from CameraRigState.yaw)
ecs_schedule(w, player_input_system);
ecs_physics_schedule(w);   // the layers the character collides with
ecs_flush(w, dt);          // BEFORE the loop: ecs_flush_ex re-runs every pending system
for (i32 k = 0; k < steps; k++) {
  character_schedule_fixed(w);
  ecs_flush_ex(w, &(EcsFlushDesc){.delta_time = fixed_dt, .steps = 1});
  ecs_rigidbody_step(w, fixed_dt);   // all lanes
}
character_schedule_frame(w);         // pose interpolation, avatar animation, before ecs_transform_schedule
```

`CharacterFrame.alpha` (resource) carries the render interpolation: write `fixed_timestep_alpha` into it on the main lane before the scope.
Outside a scope (lock-step, from `hz_tests/tests/logic/test_character_ecs_push.c`):

```c
lane_sync();
ecs_physics_schedule(w);
ecs_flush(w, dt);
character_step(w, dt);
ecs_rigidbody_step(w, dt);
character_frame(w);
lane_sync();
```

Networked (`templates/multiplayer-starting-template`):

```c
NET_GHOST(.mode = NET_GHOST_OWNER_PREDICTED, .input = MyInput, .position = motion.position)
typedef struct MyAvatarGhost {
  CharacterNet motion;
  NET_LOCAL CharacterInput intent; // the game's input system writes it each tick
} MyAvatarGhost;
// init, each stepping world
character_net_init(w, &(CharacterTuning){.walk_speed = 4.0f, .swim_enabled = true});
character_water_set(w, my_lakes, &lakes);   // optional: the level's water
// server join: ecs_component_set(w, e, CharacterNet, character_net_at(spawn, 0.0f));
// tick schedule
ecs_schedule(w, my_intent_system);
character_net_schedule(w, server);
```

The starting template (`src/starting_template/starting_template.c`) is the worked use. A project without the
module gets it with `hz/hzbuild module add gameplay_blocks`.

# Gotchas

- `character_controller_init` is required: `character_schedule_*`, `character_step` and `character_frame` assert "call character_controller_init
  first" without it. It creates the `CharacterFrame` resource and the cached queries, and runs after `ecs_assets_init` and `ecs_physics_init`.

- `character_schedule_fixed` is per fixed step, `character_schedule_frame` once per frame; both inside the scope. The avatar system
  is `ecs_schedule_single` (it writes other entities' animation layers).
- Kinematic: `Character.position` is authoritative and the root `WorldTransform` is the interpolated render pose. Writing the root
  `WorldTransform` from outside (the `--commands teleport`) teleports the character; `CharacterInput.teleport` does it with a facing.
- The collision layers are a per-frame snapshot from `ecs_physics_schedule`: schedule it before the fixed loop, and a moving platform's
  pose lags the character by one frame.
- `jump` is a request consumed when a step arms the jump buffer: write `ci->jump |= pressed_this_frame` so a frame with zero steps does
  not drop the press.
- `heading` is the view yaw (forward = (-sin, 0, -cos)); `Character.yaw` 0 faces +Z. Feed the heading from `CameraRigState.yaw`.
- The rigidbody mover reads gravity from the physics world's settings (`ecs_rigidbody_world`) and needs `ecs_rigidbody_step` after the fixed systems.
  Its body never rotates: `character_avatar_add` hangs the avatar under a `CharacterVisual` pivot that carries the facing.
- A networked ghost has no `Character`: the step builds one from `CharacterNet` each tick. Draw a body of your own
  (the template's `PlayerBody` copies the motion onto a `Character` the avatar animates from).
- Networked characters use the kinematic mover in `ecs_physics`' query world and never push rigidbodies; a host's
  server world needs the level's colliders copied in, and a moving platform must move the same on the server and in
  a client's prediction.
- Does NOT: pathfind, drive vehicles, ragdoll, or move a camera (see [[camera-rig]]).
