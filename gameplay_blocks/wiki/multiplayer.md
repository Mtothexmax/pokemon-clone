---
title: Multiplayer
description: multiplayer block — players join the game and show up with their names on Kaigen, Steam and Epic: the worlds for the role, the online services with the stores the game lists, the netcode started once the level is in, the --bots process, a host ending its session before it quits, a replicated name per player with its nametag, and lobbies (teams, ready, countdown, the match for everyone, host migration)
resource: src/gameplay/multiplayer/multiplayer.h
tags: [gameplay, netcode, online, multiplayer, steam, epic, nametag, lobby]
updated: 2026-11-11
---

# What it provides

- `multiplayer_create(&MultiplayerDesc)` (main lane, `app_init`): the role from the command line or the online
  services, the game's world, and the server world (the same world on a dedicated server, a second one on a host,
  NULL on a client). A `--bots` child process (`--netcode-bots` / `--bots-server`) starts its bot netcode here and
  never signs in.
- Stores, as `stores` in `hzproject.hzt` lists them: Kaigen is always on; `steam_appid` turns Steam on, `eos`
  (`OnlineEosDesc`) turns Epic on. At run time the engine uses the store the game was launched from for sign-in,
  friends, invites, overlay joins and, between two Steam players, Steam's relay (the online host keeps the
  `NetcodeServerOptions.paths` slot it needs). A launch by a listed store plays online without `--online`, and a
  Steam lobby join at launch (`+connect_lobby`) starts as a joiner. The game writes nothing per store.
- `MultiplayerDesc`: `registry`, `protocol_id`, `simulate` (the tick schedule), `max_players` (0 = 8), `port` (0 = 4433,
  `--port` overrides; WebSocket = port + 1), `policy` (`--online-policy` overrides), `rich_host`/`rich_play` presence,
  `bot_input` + `bot_sight` for `--bots N`, and `fill` for the game's part of the `NetcodeDesc` (play space, tiles,
  caps, lag compensation, codebook). A large server sets `alloc_reserve` (0 = 512 MB) and a big drawn crowd `world`.
- `multiplayer_frame(m, dt, ready)` each frame before the receive: the online services, then the netcode's start on
  the first frame `ready` is true. In a bots process every lane calls it; otherwise the main lane. `--bots` past the
  server's connections (`max_connections`, 0 = 64) are cut to fit at the start, with one warning.
- `MultiplayerNet {net}` on the server world: tick systems that ask the server (identity, views) read it.
- `multiplayer_quit(m)`: a host ends its online session (5 s at most), then the app quits. `multiplayer_frame` does the same for every quit request while hosting (window close, Alt-F4, ctrl-c, a script `quit`): it vetoes it with `hz_cancel_quit` and ends the session first; a second request while ending quits at once.
- **Switching roles at runtime:** `multiplayer_host(m, policy)`, `multiplayer_join(m, text)` (a session id, or an invite link's `kaigen-join=` value) and `multiplayer_leave(m)` (back to the offline local host). Each needs `MultiplayerDesc.on_role(user, m, role)`. A game without it keeps its first role, and these calls are refused with an error.
  - `on_role` is called from `multiplayer_create` for the first role, then on each switch. It fills the new server world (`multiplayer_server_world`; NULL on a client), the way app_init did.
  - The switch runs in `multiplayer_frame` on the main lane, before the receive is scheduled and outside any flush. In order: the replay recorder resets (`netcode_replay_reset`), `ecs_netcode_destroy(world)` runs (the old ghosts leave the game's world), the old server world is destroyed, the new one is created, `on_role` runs, and the netcode starts once `ready`.
  - The log line `multiplayer: now a host (main lane: teardown X ms, rebuild Y ms)` gives both times.
  - A client whose host leaves is switched back to offline play by itself.
- Names: `MultiplayerName` (`NET_COMPONENT`, up to 63 UTF-8 bytes) in the avatar ghost. `multiplayer_schedule_server(w)`
  in the server's tick gives each owned ghost its connection's verified name (`netcode_connection_identity`: the
  Kaigen account's display name, which a Steam or Epic sign-in creates from the store's persona name), else
  "Player <owner>", upgraded when the identity lands later. `multiplayer_nametag(w, ghost, offset)` adds a
  `WORLD_UI_NAMETAG`; `multiplayer_schedule(w)` in a client's frame keeps labels on their ghost's name;
  `multiplayer_name(w, e)` reads it.
- Reads: `multiplayer_role`, `multiplayer_world`, `multiplayer_server_world`, `multiplayer_net`, `multiplayer_status`,
  `multiplayer_invite(m, alloc)`, `multiplayer_players`, `multiplayer_online` (the `OnlineGame`, for a panel).
- Does NOT: draw online UI, own the registry, the ghosts or the tick schedule, or let a player pick a name.

# Lobbies

Players gather in a Kaigen Online lobby before a match: no game connection while they wait. The lobby is the
service's (`online_lobby_*`); the block keeps its state, decides each frame what this player does, and plays the match
through its own `multiplayer_host` / `multiplayer_join` / `multiplayer_leave`.

- `multiplayer_lobby_create(m, &MultiplayerLobbyDesc{policy, name, mode, max_players, teams})`: a new lobby, this
  player its host (owner). `name` NULL = "<player>'s lobby"; `mode` is what Browse and the lobby show.
- `multiplayer_lobby_join(m, text)`: by id; the text may be the invite code or anything holding it (its first run of
  24 lowercase hex digits). A lobby whose match runs is joined straight into the match.
- `multiplayer_lobby_leave`, `_ready(m, b32)`, `_team(m, team)` (false, and a log line, when the team is full),
  host only: `_start` (a 5 s countdown on the service's clock, then the match), `_cancel` (stops the countdown; while
  the match runs it ends it and everyone returns), `_kick(m, player)`, `_privacy(m, policy)`.
- `multiplayer_lobby(m, &MultiplayerLobbyView)`: state (`NONE`, `JOINING`, `OPEN`, `STARTING`, `LOADING`, `PLAYING`),
  id (the invite code), name, mode, policy, teams and team size, `host` (this player hosts), the countdown, the members
  in joined order (name, team, ready, host, self, player id) and the log lines (`log_seq` grows by one per line: a
  toast per increase). In `NONE`, `status` says why the last lobby ended ("You were removed from the lobby") or why a
  create or join failed.
- `multiplayer_lobby_team_of(m, connection)` on the match's server: the team the lobby chose for a verified
  connection (`netcode_connection_identity`'s player is the lobby member), else `MULTIPLAYER_NO_TEAM`.

How it runs:
- Teams are member attributes (`team`), readiness the member's ready flag. A member without a team (a newcomer) takes
  the team with fewer players; a team past `max_players / teams` by join order (two joined at once) sends its later
  member to a team with room. Everyone's screen redraws from the pushed lobby (`ONLINE_EV_LOBBY`, polled by version).
- The owner's lobby attributes carry the match: `state` (`lobby`, `starting`, `playing`), `start_at` (the service's
  unix ms, from `online_server_time`), `session` (the match's id, `-` for none), plus `teams`, `mode` and `protocol`
  (`MultiplayerDesc.protocol_id` in decimal: a browser lists only lobbies whose match it can join,
  `OnlineSessionQuery{.lobbies = true, .attrs = {"protocol", ...}}`).
- START writes `starting` and `start_at = now + 5 s`. At 0 the host calls `multiplayer_host` (listed public: see
  Gotchas), and once listed writes `session` and `playing`; every member seeing a `session` it has not joined calls
  `multiplayer_join` (up to three tries, 3 s apart, when a join does not happen).
- The game ends the match with `multiplayer_lobby_cancel` (Kestrel: once its scoreboard is over): `state` back to
  `lobby`, every member leaves the match (`multiplayer_leave`) and clears its ready flag.
- Host migration: the service hands the lobby to its longest member when the owner leaves (and re-lists it under the
  new owner at once). A new owner that did not start what the lobby shows (a countdown, or a match it does not host)
  writes `lobby` back: everyone returns, the log says "<name> is now the host", and the new host can start again.
- A quit request while in a lobby leaves it first (with the hosted session's end, inside the same 5 s).
- Lobby events reach the block through `OnlineGameDesc.event`; a kick arrives as `ONLINE_EV_LOBBY_GONE` "kicked".

The logic is plain functions over a snapshot (`multiplayer_lobby_internal.h`: `mp_lobby_snap`, `mp_lobby_team_pick`,
`mp_lobby_changes`, `mp_lobby_decide`, `mp_lobby_code`), tested in `net_tests/tests/test_multiplayer_lobby.c`
(`--filter mp_lobby`). Kestrel's `ks_lobby.c` is the worked screen.

# Example usage

```c
s->mp = multiplayer_create(&(MultiplayerDesc){.registry = &app_netcode_registry, .protocol_id = GAME_PROTOCOL,
                                              .simulate = game_simulate, .user = s, .bot_input = game_bot_input,
                                              .bot_sight = 4, .fill = game_net_fill});
if (multiplayer_role(s->mp) == NETCODE_BOTS) return;
EcsWorld *w = multiplayer_world(s->mp); // init it (and multiplayer_server_world when it differs)
// frame, main lane, before the receive
multiplayer_frame(s->mp, dt, game_level_ready(&s->level));
// server tick: multiplayer_schedule_server(w); client, per new remote ghost: multiplayer_nametag(w, g, (v3){0, 2.1f, 0});
// client frame scope: multiplayer_schedule(w); world_ui_schedule(w);
```

A game on Steam and Epic adds `.steam_appid = 480` and `.eos = &(OnlineEosDesc){...}` to the desc. The worked example
is `templates/multiplayer-starting-template`.

# Gotchas

- Positioned ghost types need `NetcodeDesc.server.play_space`: set it in `fill`, or the netcode asserts at start.
- `fill` runs after the online services filled the desc: set single fields, never whole `d->client` / `d->server`
  structs, or the join ticket, token and path are lost.
- A host's server world has no scene: copy the level's static colliders into it before `ready`, and again in `on_role` on each switch (the new world is empty).
- State the game keeps beside the worlds (sequence counters it compared with ghosts, the own avatar's entity, a killcam) belongs to the old netcode: reset it in `on_role`. Kestrel's `ks_net_role_reset` is the example.
- Allocate a server world's per-world buffers once and reuse them across switches. Kestrel keeps `srv_gather` for this. Allocating per switch leaks.
- The name lands a tick after the ghost spawns: a tag added on spawn starts empty and `multiplayer_schedule` fills it.
- A game that never calls `multiplayer_frame` on a frame gets no veto on that frame: its quit goes through unended.
- A lobby's match is listed public, not invite only: the service admits an invite-only session's players only through
  `online_invite`, which goes to friends, and lobby members are not the host's friends. A game's browser that lists
  lobbies only (Kestrel's) never shows the match. Likewise an invite-only lobby admits only players invited through
  `online_invite` (friends): its code alone does not let a stranger in.
- The view's strings point into the block: copy them for text the UI reads after a lobby call in the same frame
  (Kestrel's `ks_lb_str`).
