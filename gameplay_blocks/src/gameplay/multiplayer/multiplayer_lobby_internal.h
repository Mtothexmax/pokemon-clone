// The multiplayer block's lobby logic as plain functions over a lobby snapshot: module-internal, and what the tests drive.
#pragma once

#include "gameplay/multiplayer/multiplayer.h"
#include "lib/string.h"

#define MP_LOBBY_COUNTDOWN_MS 5000.0

// the lobby's `state` attribute
typedef enum MpLobbyPhase {
  MP_PHASE_LOBBY = 0,
  MP_PHASE_STARTING, // `start_at`: the service's unix ms the countdown ends at
  MP_PHASE_PLAYING,  // `session`: the host's match
} MpLobbyPhase;

typedef struct MpLobbyMemberRec {
  String64Bytes player, name;
  u32 team; // MULTIPLAYER_NO_TEAM before its `team` attribute
  b32 ready;
} MpLobbyMemberRec;

// a lobby document as the block keeps it between pushes
typedef struct MpLobbySnap {
  u32 version;
  String64Bytes id, owner, name, mode, session;
  OnlinePolicy policy;
  u32 max, teams;
  u32 phase; // MpLobbyPhase
  f64 start_at;
  u32 count;
  MpLobbyMemberRec m[MULTIPLAYER_LOBBY_MEMBERS];
} MpLobbySnap;

// this player's side of a frame's decision
typedef struct MpLobbyLocal {
  String self;    // the player's id
  b32 started;    // this process wrote the STARTING the lobby shows (its own start)
  b32 hosting;    // its match is asked for, listing or listed
  String hosted;  // the listed session's id ("" until listed)
  b32 in_match;   // joining or playing `joined`
  b32 playing;    // ... and its netcode runs (a host: listed and running)
  String joined;  // the session last joined for this lobby ("" none)
  f64 now;        // the service's clock, unix ms (0: not known yet)
} MpLobbyLocal;

typedef enum MpLobbyDo {
  MP_DO_NOTHING = 0,
  MP_DO_HOST,        // the owner's countdown ended: host the match
  MP_DO_PUBLISH,     // the owner's match is listed: `session` and PLAYING into the lobby
  MP_DO_JOIN,        // a member joins the lobby's session
  MP_DO_LEAVE_MATCH, // the lobby is back: out of the match
  MP_DO_RESET,       // an owner that did not start what the lobby shows (the host left): back to LOBBY
} MpLobbyDo;

// a line of the lobby's log
typedef struct MpLobbyLine {
  String128Bytes text;
} MpLobbyLine;

// the document into a snapshot (the first MULTIPLAYER_LOBBY_MEMBERS members)
void mp_lobby_snap(const OnlineLobby *l, MpLobbySnap *out);
// the member's index, -1 when not in it
i32 mp_lobby_find(const MpLobbySnap *s, String player);
// the team member `self` belongs on: its own unless absent or past team_size by join order, else the emptiest with room
u32 mp_lobby_team_pick(const MpLobbySnap *s, u32 self, u32 team_size);
// members on `team`, `skip` (an index, or -1) not counted
u32 mp_lobby_team_count(const MpLobbySnap *s, u32 team, i32 skip);
// what changed from prev to next: joins, leaves, a new host; at most max lines, returns how many
u32 mp_lobby_changes(const MpLobbySnap *prev, const MpLobbySnap *next, MpLobbyLine *out, u32 max);
// what this player does this frame, and what its lobby screen shows
MpLobbyDo mp_lobby_decide(const MpLobbySnap *s, const MpLobbyLocal *l, MultiplayerLobbyState *view);
// a lobby id in a pasted text: its first run of exactly 24 lowercase hex digits ("" when none)
String mp_lobby_code(String text);

// ---- the block's own wiring between multiplayer.c and multiplayer_lobby.c ----

typedef struct MpLobby MpLobby;
// multiplayer.c: the lobby state multiplayer_create made
MpLobby *multiplayer_lobby_slot(const Multiplayer *m);
// multiplayer.c: MultiplayerDesc's max_players (after its default) and protocol_id
u32 multiplayer_max_players(const Multiplayer *m);
u64 multiplayer_protocol(const Multiplayer *m);
// main lane, multiplayer_create
MpLobby *mp_lobby_create(Allocator *a);
// main lane, each multiplayer_frame after the online services' update: ops, the snapshot, the decision
void mp_lobby_frame(Multiplayer *m, MpLobby *l);
// main lane, from online_game_update: a lobby event
void mp_lobby_event(Multiplayer *m, MpLobby *l, const OnlineEvent *e);
// main lane, a quit: the lobby is left; true while that leave is in flight
b32 mp_lobby_quitting(Multiplayer *m, MpLobby *l);
