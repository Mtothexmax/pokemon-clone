// Multiplayer: players join the game and show up with their names, on Kaigen, Steam and Epic: the worlds for the role,
// the online services with the stores the game lists, the netcode started once the level is in, the --bots process, a
// host ending its session before it quits, a replicated name per player with its nametag, and lobbies.
#pragma once

#include "ecs/ecs.h"
#include "gameplay/world_ui/world_ui.h"
#include "lib/reflect.h"
#include "netcode/netcode.h"
#include "online/online_game.h"

// the game's part of the NetcodeDesc (field, tiles, caps, lag compensation, codebook), after the online services' (role, dev, tickets), so a game may adjust d->dev
typedef void MultiplayerFillFn(void *user, NetcodeDesc *d);
typedef struct Multiplayer Multiplayer;
// main lane, outside flushes: the role's server world is new (multiplayer_server_world; NULL on a client): the game
// fills it. Called from multiplayer_create for the first role, then on each switch before that role's netcode starts
typedef void MultiplayerRoleFn(void *user, Multiplayer *m, NetcodeRole role);

typedef struct MultiplayerDesc {
  const char *title;               // Kaigen Online's; NULL = ONLINE_DEV_TITLE
  u32 steam_appid;                 // Steam, as `stores` in hzproject.hzt lists it (0 = no Steam)
  const OnlineEosDesc *eos;        // Epic, as `stores` lists it (NULL = no Epic)
  u32 max_players;                 // the session's and the server's connections, bots included; 0 = 8
  const NetcodeRegistry *registry; // &app_netcode_registry
  u64 protocol_id;
  u16 port;                        // 0 = 4433; --port <n> overrides
  u16 ws_port;                     // 0 = port + 1
  OnlinePolicy policy;             // --online-policy public|friends|invite overrides
  NetScheduleFn simulate;          // the tick's systems, for the server world and a client's prediction
  void *user;                      // simulate's, bot_input's and fill's
  const char *rich_host;           // presence while hosting (NULL = "Hosting")
  const char *rich_play;           // presence while playing (NULL = "Playing")
  NetcodeBotInputFn bot_input;     // a --bots process's players (NULL: zeroed input)
  u32 bot_sight;                   // ghosts each bot sees
  MultiplayerFillFn *fill;         // NULL: none
  u64 alloc_reserve;               // the netcode's and the online services' TLSF reserve; 0 = 512 MB
  const EcsWorldInitDesc *world;   // the world the game draws in; NULL = ecs_world_create()
  MultiplayerRoleFn *on_role;      // NULL: the game plays its first role only (host, join and leave are refused)
} MultiplayerDesc;

// on the server world while the netcode runs, for tick systems that ask the server (identity, views)
typedef struct MultiplayerNet {
  Netcode *net;
} MultiplayerNet;
ECS_COMPONENT_TYPEDEF(MultiplayerNet);

// a player's name on the ghost its connection owns: its verified login's, else "Player <n>" (n = NetGhost.owner)
ECS_COMPONENT_TYPEDEF(MultiplayerName);
HZ_REFLECT()
NET_COMPONENT()
typedef struct MultiplayerName {
  u32 len;
  u32 text[16]; // UTF-8, four bytes a word in the platform's byte order (every target is little-endian)
  u32 verified; // 1: the signed-in player's name
} MultiplayerName;

// main lane, app_init: the role (the command line's, the online services' or the store's launch), its worlds and on_role; a bots process starts its netcode here
Multiplayer *multiplayer_create(const MultiplayerDesc *d);
// main lane, after the last frame: a hosted session ends without waiting, the netcode closes; the worlds stay the game's
void multiplayer_destroy(Multiplayer *m);
// each frame before the netcode receive, outside flushes, every lane in a bots process, else the main lane: the online
// services, a role switch (replay, netcode and server world out, the new ones in), the netcode's start once `ready`
void multiplayer_frame(Multiplayer *m, f32 dt, b32 ready);
// main lane: a host ends its session first (5 s at most), then quits, as for any quit request; a second quits now
void multiplayer_quit(Multiplayer *m);
// main lane: offline play becomes a listed online host (online_game_host); the switch comes in multiplayer_frame
void multiplayer_host(Multiplayer *m, OnlinePolicy policy);
// main lane: joins a session from offline or hosting: an id (online_session_find's) or an invite link (its kaigen-join)
void multiplayer_join(Multiplayer *m, String session);
// main lane: back to offline play, a local host as at startup
void multiplayer_leave(Multiplayer *m);

// the role the worlds are built for: the first one, then each switch's
NetcodeRole multiplayer_role(const Multiplayer *m);
// the world the game draws in; NULL in a bots process
EcsWorld *multiplayer_world(const Multiplayer *m);
// a host's second world, a server's own world, NULL on a client
EcsWorld *multiplayer_server_world(const Multiplayer *m);
// NULL until it started
Netcode *multiplayer_net(const Multiplayer *m);
// the online services' word for the player ("" when they are off)
String multiplayer_status(const Multiplayer *m);
// a host's invite: the play page's link once published, else the session id ("" otherwise)
String multiplayer_invite(const Multiplayer *m, Allocator *a);
// players connected to a host or server, bots included; 0 on a client
u32 multiplayer_players(const Multiplayer *m);
// main lane: the online services themselves, for a game's own panel (who signed in, a friend's invite); NULL when off
OnlineGame *multiplayer_online(const Multiplayer *m);

// inside the server's tick schedule: each owned ghost's MultiplayerName, upgraded to the verified one once its connection has it
void multiplayer_schedule_server(EcsWorld *w);
// inside a client's frame scope, before world_ui_schedule: nametags follow their ghost's name
void multiplayer_schedule(EcsWorld *w);
// main lane, or a system that reads MultiplayerName: e's name ("" without one), valid while e lives
String multiplayer_name(EcsWorld *w, EcsEntity e);
// main lane: a nametag `offset` above e, a ghost with MultiplayerName and a WorldTransform; world_ui_layout draws it
void multiplayer_nametag(EcsWorld *w, EcsEntity e, v3 offset);

// ---- lobbies: players gather on Kaigen Online, pick teams and ready up; the host starts and everyone plays its match ----

#define MULTIPLAYER_LOBBY_MEMBERS 16u
#define MULTIPLAYER_LOBBY_LOG 8u
#define MULTIPLAYER_NO_TEAM 0xffffffffu

typedef enum MultiplayerLobbyState {
  MULTIPLAYER_LOBBY_NONE = 0, // in no lobby (status: why the last one ended or failed)
  MULTIPLAYER_LOBBY_JOINING,  // creating or joining one
  MULTIPLAYER_LOBBY_OPEN,     // gathered: teams and ready
  MULTIPLAYER_LOBBY_STARTING, // the host started: countdown runs
  MULTIPLAYER_LOBBY_LOADING,  // the countdown ended: the host's match is listed and joined
  MULTIPLAYER_LOBBY_PLAYING,  // in the lobby's match
} MultiplayerLobbyState;

typedef struct MultiplayerLobbyDesc {
  OnlinePolicy policy;
  const char *name; // NULL = "<player>'s lobby"
  const char *mode; // what Browse and the lobby read ("Team Deathmatch"); NULL = none
  u32 max_players;  // 0 = MultiplayerDesc.max_players; at most MULTIPLAYER_LOBBY_MEMBERS
  u32 teams;        // 0 = 1
} MultiplayerLobbyDesc;

typedef struct MultiplayerLobbyMember {
  String player; // Kaigen Online's player id: multiplayer_lobby_kick's
  String name;
  u32 team;      // MULTIPLAYER_NO_TEAM until its first pick lands
  b32 ready, host, self;
} MultiplayerLobbyMember;

// the lobby as this frame's multiplayer_frame left it; strings and arrays valid until the next multiplayer_frame
typedef struct MultiplayerLobbyView {
  MultiplayerLobbyState state;
  String id;   // the invite code: multiplayer_lobby_join takes it
  String name;
  String mode;
  OnlinePolicy policy;
  u32 teams, team_size, max_players;
  b32 host;      // this player is the host
  f32 countdown; // STARTING: seconds left on the service's clock
  u32 member_count;
  const MultiplayerLobbyMember *members; // joined order: the longest member first
  u32 log_count;                         // the newest lines, oldest first ("Ana joined", "Ana is now the host")
  const String *log;
  u32 log_seq;   // lines written since the block was made: a new one per increase
  String status; // NONE: why the last lobby ended or a create or join failed ("" = nothing to say)
} MultiplayerLobbyView;

// main lane, signed in: a new lobby of this player's, hosted by it; JOINING, then OPEN
void multiplayer_lobby_create(Multiplayer *m, const MultiplayerLobbyDesc *d);
// main lane: a lobby by its id (Browse's row, the invite code, a pasted text holding it); a running match is joined at once
void multiplayer_lobby_join(Multiplayer *m, String id_or_code);
// main lane: out of the lobby and its match; the longest member becomes the host. Also clears NONE's status
void multiplayer_lobby_leave(Multiplayer *m);
void multiplayer_lobby_ready(Multiplayer *m, b32 ready);
// main lane: this player onto team `team`; false (logged) when it is full or there is no such team
b32 multiplayer_lobby_team(Multiplayer *m, u32 team);
// main lane, the host: the countdown (5 s on the service's clock), then the match for everyone, each on its team
void multiplayer_lobby_start(Multiplayer *m);
// main lane, the host: STARTING stops the countdown; PLAYING ends the match and everyone returns to the lobby, ready reset
void multiplayer_lobby_cancel(Multiplayer *m);
// main lane, the host: the player (MultiplayerLobbyMember.player) leaves; it reads why
void multiplayer_lobby_kick(Multiplayer *m, String player);
// main lane, the host: who may find and join the lobby
void multiplayer_lobby_privacy(Multiplayer *m, OnlinePolicy policy);
// the lobby's view; false in none
b32 multiplayer_lobby(const Multiplayer *m, MultiplayerLobbyView *out);
// the server's tick (or main lane) of the lobby's match: the team the lobby chose for a verified connection, else MULTIPLAYER_NO_TEAM
u32 multiplayer_lobby_team_of(const Multiplayer *m, u32 connection);
