#include "multiplayer.h"
#include "multiplayer_lobby_internal.h"
#include "ecs/ecs_netcode/ecs_netcode.h"
#include "hz_engine.h"
#include "gameplay/netcode_replay/netcode_replay_internal.h"
#include "lib/allocator_tlsf.h"
#include "lib/assert.h"
#include "lib/thread_context.h"
#include "lib/utf.h"
#include "os/os.h"

#define MP_PORT 4433
#define MP_END_MS 5000.0  // a quitting host waits this long for the service to take its session off
#define MP_NAME_MAX 63u   // bytes: a WorldUiLabel holds them whole
#define MP_ENV_MAX 64u

struct Multiplayer {
  MultiplayerDesc desc;
  TLSFAllocator *tlsf;
  Allocator alloc; // the netcode's and online_game's: connections come and go
  OnlineGame *og;
  Netcode *net;
  NetcodeRole role;
  NetcodeArgs args;
  EcsWorld *world;
  EcsWorld *server_world;
  const OnlineGameStart *start; // online_game's, until its next start
  b32 start_pending;
  b32 begun; // a netcode was started for the worlds as they are
  b32 ending;
  b32 end_session; // the ending waits for the hosted session's end
  b32 quitting;  // the block's own quit request, never vetoed
  b32 requeue;   // the ending holds a vetoed engine quit request: it is re-issued, not replaced by os_quit
  u64 end_from;
  NetcodeReplay *replay; // netcode_replay_init's
  f64 switch_ms[2];      // the last switch's main-lane teardown and rebuild (server world, on_role, netcode init)
  b32 switch_logged; // the last switch's times are logged (true until the first switch)
  MpLobby *lobby;
};

// the role's server world: a host's own, a server's is the world it draws in, none on a client
hz_internal void mp_server_world(Multiplayer *s) {
  s->server_world = s->role == NETCODE_SERVER ? s->world : s->role == NETCODE_HOST ? ecs_world_create() : NULL;
  if (s->server_world)
    ecs_resource_set(s->server_world, MultiplayerNet, {0});
}

// main lane, outside flushes, before the receive is scheduled: what the old role owned goes (its replay recording, its
// netcode, its server world), the new role's server world comes and the game fills it
hz_internal void mp_switch(Multiplayer *s, NetcodeRole role) {
  u64 at = os_time_now();
  if (s->replay)
    netcode_replay_reset(s->replay);
  if (s->net)
    ecs_netcode_destroy(s->world);
  s->net = NULL;
  if (s->server_world && s->server_world != s->world)
    ecs_world_destroy(s->server_world);
  s->server_world = NULL;
  u64 mid = os_time_now();
  s->role = role;
  mp_server_world(s);
  s->desc.on_role(s->desc.user, s, role);
  s->begun = false;
  s->switch_ms[0] = os_ticks_to_ms(os_time_diff(mid, at));
  s->switch_ms[1] = os_ticks_to_ms(os_time_diff(os_time_now(), mid));
  s->switch_logged = false;
}

hz_internal OnlinePolicy mp_policy(OnlinePolicy fallback) {
  const char *p = hz_arg_value("--online-policy");
  if (!p)
    return fallback;
  if (cstr_equal(p, "public"))
    return ONLINE_POLICY_PUBLIC;
  if (cstr_equal(p, "friends"))
    return ONLINE_POLICY_FRIENDS;
  if (cstr_equal(p, "invite"))
    return ONLINE_POLICY_INVITE;
  log_warn("multiplayer: --online-policy % is not public, friends or invite", fmt_cstr(p));
  return fallback;
}

// --port <n>: a run's UDP port n and WebSocket n + 1, beside another run on this machine
hz_internal void mp_ports(const MultiplayerDesc *d, u16 *port, u16 *ws_port) {
  u32 p = d->port ? d->port : MP_PORT;
  u32 ws = d->ws_port ? d->ws_port : p + 1;
  const char *arg = hz_arg_value("--port");
  if (arg) {
    u32 v = 0;
    if (cstr_to_u32((CString)arg, &v) && v > 0 && v < 65535) {
      p = v;
      ws = v + 1;
    } else {
      log_warn("multiplayer: --port % is not a port: % is used", fmt_cstr(arg), fmt_u32(p));
    }
  }
  *port = (u16)p;
  *ws_port = (u16)ws;
}

// --bots past the server's connections are cut to fit, with one warning: netcode_init refuses the whole start otherwise
hz_internal void mp_bots_cap(NetcodeDesc *d) {
  u32 max = d->server.replication.max_connections ? d->server.replication.max_connections : 64u; // netcode_init's default
  u32 windows = d->role == NETCODE_HOST ? MAX(d->dev.clients, 1u) : d->dev.clients; // a host's own player takes one
  u32 room = windows < max ? max - windows : 0u;
  if (d->dev.bots <= room)
    return;
  log_warn("multiplayer: the server takes % players: % windows and % of the % bots join, % are refused", fmt_u32(max),
           fmt_u32(windows), fmt_u32(room), fmt_u32(d->dev.bots), fmt_u32(d->dev.bots - room));
  d->dev.bots = room;
}

hz_internal b32 mp_has_arg(u32 argc, const char *const *argv, const char *a) {
  for (u32 i = 0; i < argc; i++)
    if (cstr_equal(argv[i], a))
      return true;
  return false;
}

// a store the game lists started it (Steam sets SteamAppId and names a lobby to join, the Epic launcher passes an exchange code): it plays online
hz_internal b32 mp_store_launch(const MultiplayerDesc *d, u32 argc, const char *const *argv, b32 *steam_join) {
  *steam_join = d->steam_appid && mp_has_arg(argc, argv, "+connect_lobby");
  b32 steam = false;
  if (d->steam_appid) {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    steam = *steam_join || os_env_get("SteamAppId", ALLOC_ARRAY(&ta.allocator, char, MP_ENV_MAX), MP_ENV_MAX) > 0;
    tctx_temp_allocator_end(ta);
  }
  b32 epic = d->eos && mp_has_arg(argc, argv, "-AUTH_TYPE=exchangecode");
  return steam || epic;
}

// main lane: the netcode for the start online_game gave, into the worlds create built
hz_internal void mp_begin(Multiplayer *s) {
  s->start_pending = false;
  s->begun = true;
  const OnlineGameStart *start = s->start;
  const MultiplayerDesc *sd = &s->desc;
  NetcodeDesc d = {.server_world = s->role == NETCODE_HOST ? s->server_world : NULL,
                   .registry = sd->registry,
                   .simulate = sd->simulate,
                   .user = sd->user,
                   .protocol_id = sd->protocol_id,
                   .server = {.replication = {.max_connections = sd->max_players}},
                   .client = {.network_sim = true},
                   .alloc = &s->alloc};
  mp_ports(sd, &d.server.port, &d.server.ws_port);
  online_game_netcode_desc(s->og, start, &d);
  if (sd->fill)
    sd->fill(sd->user, &d);
  mp_bots_cap(&d);
  u64 at = os_time_now();
  s->net = ecs_netcode_init(s->world, &d);
  s->switch_ms[1] += os_ticks_to_ms(os_time_diff(os_time_now(), at));
  online_game_netcode(s->og, s->net);
  if (!s->net) {
    log_error("multiplayer: the netcode did not start");
    return;
  }
  if (s->server_world)
    ecs_resource_set(s->server_world, MultiplayerNet, {.net = s->net});
}

// main lane, from online_game_update: kept until the game is ready
hz_internal void mp_started(void *user, OnlineGame *g, const OnlineGameStart *start) {
  UNUSED(g);
  Multiplayer *s = user;
  s->start = start;
  s->start_pending = true;
}

// main lane, from online_game_update: the lobby's events
hz_internal void mp_event(void *user, OnlineGame *g, const OnlineEvent *e) {
  UNUSED(g);
  Multiplayer *s = user;
  mp_lobby_event(s, s->lobby, e);
}

Multiplayer *multiplayer_create(const MultiplayerDesc *d) {
  assert_msg(is_main_thread(), "multiplayer_create: main lane only"); // creates worlds outside a scope
  TLSFAllocator *tlsf = tlsf_allocator_create(d->alloc_reserve ? d->alloc_reserve : MB(512), MB(8));
  Allocator alloc = make_tlsf_allocator(tlsf);
  Multiplayer *s = ALLOC(&alloc, Multiplayer);
  s->tlsf = tlsf;
  s->alloc = alloc;
  s->desc = *d;
  s->desc.max_players = d->max_players ? d->max_players : 8;
  s->role = NETCODE_HOST;
  s->switch_logged = true;
  s->lobby = mp_lobby_create(&s->alloc);
  // a --bots child (or the remote form) gets the parent's arguments: it never signs in
  b32 bots = hz_arg_value("--netcode-bots") || hz_arg_value("--bots-server");
  if (bots && netcode_args_parse(hz_arg_count(), hz_argv(), &s->args, &s->alloc) && s->args.role == NETCODE_BOTS) {
    s->role = NETCODE_BOTS;
    s->net = netcode_init(&(NetcodeDesc){.role = NETCODE_BOTS,
                                         .registry = d->registry,
                                         .protocol_id = d->protocol_id,
                                         .dev = s->args.dev,
                                         .alloc = &s->alloc,
                                         .bot_input = d->bot_input,
                                         .bot_input_user = d->user,
                                         .bot_sight = d->bot_sight});
    if (!s->net)
      log_error("multiplayer: the bots did not start");
    return s;
  }
  u32 argc = hz_arg_count();
  const char *const *argv = hz_argv();
  b32 steam_join = false;
  b32 store = mp_store_launch(d, argc, argv, &steam_join);
  // a Steam lobby join at launch is a joiner from the start: its worlds are a client's
  if (steam_join && !mp_has_arg(argc, argv, "--online-join") && !hz_arg_value("--join")) {
    const char **with = ALLOC_ARRAY(&s->alloc, const char *, argc + 1u);
    for (u32 i = 0; i < argc; i++)
      with[i] = argv[i];
    with[argc++] = "--online-join";
    argv = with;
  }
  s->og = online_game_create(&(OnlineGameDesc){.title = d->title,
                                               .online = store,
                                               .alloc = &s->alloc,
                                               .argc = argc,
                                               .argv = argv,
                                               .max_players = s->desc.max_players,
                                               .policy = mp_policy(d->policy),
                                               .rich_host = d->rich_host,
                                               .rich_play = d->rich_play,
                                               .steam_appid = d->steam_appid,
                                               .eos = d->eos,
                                               .start = mp_started,
                                               .event = mp_event,
                                               .user = s});
  if (s->og)
    s->role = online_game_role(s->og);
  else
    log_error("multiplayer: the command line was refused, so the netcode will not start");
  s->world = d->world ? ecs_world_create_ex(d->world) : ecs_world_create();
  mp_server_world(s);
  if (d->on_role)
    d->on_role(d->user, s, s->role);
  return s;
}

void multiplayer_destroy(Multiplayer *s) {
  assert_msg(is_main_thread(), "multiplayer_destroy: main lane only"); // the online services and the netcode are main-lane state
  if (s->replay)
    netcode_replay_destroy(s->replay);
  s->replay = NULL;
  if (s->og)
    online_game_destroy(s->og);
  if (s->role == NETCODE_BOTS && s->net)
    netcode_destroy(s->net);
  else if (s->net)
    ecs_netcode_destroy(s->world);
  s->og = NULL;
  s->net = NULL;
}

// a listed (or listing) session of ours, which a quit ends first
hz_internal b32 mp_hosting(const Multiplayer *s) {
  OnlineGameState st = s->og ? online_game_state(s->og) : ONLINE_GAME_OFF;
  return st == ONLINE_GAME_HOSTING || st == ONLINE_GAME_LISTING;
}

// in a lobby (or joining one), which a quit leaves first
hz_internal b32 mp_lobby_member(const Multiplayer *s) {
  MultiplayerLobbyView v;
  return s->og && multiplayer_lobby(s, &v);
}

void multiplayer_frame(Multiplayer *s, f32 dt, b32 ready) {
  if (s->role == NETCODE_BOTS) {
    if (s->net)
      netcode_update(s->net, dt);
    return;
  }
  if (!is_main_thread() || !s->og)
    return;
  online_game_update(s->og);
  mp_lobby_frame(s, s->lobby);
  // any quit request (window close, Alt-F4, ctrl-c, a script) ends a hosted session and leaves a lobby first; a second one quits now
  if (hz_quit_requested() && !s->quitting) {
    if (s->ending) {
      log_info("multiplayer: quit again while ending the session: quitting now");
      s->ending = false;
      s->quitting = true;
    } else if (mp_hosting(s) || mp_lobby_member(s)) {
      hz_cancel_quit();
      s->requeue = true;
      multiplayer_quit(s);
    }
  }
  // a start for worlds already playing (a switch) rebuilds them first; the game's `ready` then gates the netcode
  if (s->start_pending && !s->ending && (s->begun || s->start->role != s->role)) {
    if (!s->desc.on_role) {
      log_warn("multiplayer: a % start, and no on_role to build its worlds: the first role plays on", fmt_u32(s->start->role));
      s->start_pending = false;
      online_game_netcode(s->og, NULL);
    } else {
      mp_switch(s, s->start->role);
    }
  }
  if (s->start_pending && ready && !s->ending)
    mp_begin(s);
  if (s->net && !s->switch_logged && s->begun) {
    s->switch_logged = true;
    log_info("multiplayer: now % (main lane: teardown % ms, rebuild % ms)",
             fmt_cstr(s->role == NETCODE_CLIENT ? "a client" : s->role == NETCODE_SERVER ? "a server" : "a host"),
             fmt_f64_p(s->switch_ms[0], 3), fmt_f64_p(s->switch_ms[1], 3));
  }
  if (!s->ending)
    return;
  OnlineGameState st = online_game_state(s->og);
  b32 ended = !s->end_session || st == ONLINE_GAME_ENDED || st == ONLINE_GAME_FAILED || st == ONLINE_GAME_OFF;
  b32 settled = ended && !mp_lobby_quitting(s, s->lobby);
  if (settled || os_ticks_to_ms(os_time_diff(os_time_now(), s->end_from)) >= MP_END_MS) {
    log_info("multiplayer: % the session and the lobby, quitting", fmt_cstr(settled ? "ended" : "gave up waiting to end"));
    s->ending = false;
    s->quitting = true;
    // os_quit does nothing in a browser; the vetoed request stops the engine loop there as everywhere
    if (s->requeue)
      hz_request_quit();
    else
      os_quit();
  }
}

void multiplayer_quit(Multiplayer *s) {
  assert_msg(is_main_thread(), "multiplayer_quit: main lane only"); // online_game is main-lane state
  b32 hosting = mp_hosting(s);
  b32 lobby = s->og && mp_lobby_quitting(s, s->lobby);
  if (!hosting && !lobby) {
    s->quitting = true;
    os_quit();
    return;
  }
  if (s->ending)
    return;
  log_info("multiplayer: % before quitting", fmt_cstr(hosting && lobby ? "ending the hosted session and leaving the lobby"
                                                      : hosting        ? "ending the hosted session"
                                                                       : "leaving the lobby"));
  s->ending = true;
  s->end_session = hosting;
  s->end_from = os_time_now();
  if (hosting)
    online_game_end(s->og);
}

// a game with no on_role plays its first role: switching is refused, logged
hz_internal b32 mp_switchable(const Multiplayer *s, const char *what) {
  if (s->og && s->desc.on_role)
    return true;
  log_error("multiplayer: % needs MultiplayerDesc.on_role (and the online services)", fmt_cstr(what));
  return false;
}

void multiplayer_host(Multiplayer *s, OnlinePolicy policy) {
  assert_msg(is_main_thread(), "multiplayer_host: main lane only"); // online_game is main-lane state
  if (mp_switchable(s, "multiplayer_host"))
    online_game_host(s->og, policy);
}

hz_internal b32 mp_blank(char c) { return c == ' ' || c == '\t' || c == '\r' || c == '\n'; }

// a session id as typed or pasted: an invite link's kaigen-join value, else the text without its surrounding blanks
hz_internal String mp_session_of(String text) {
  String key = STR_FROM_CSTR("kaigen-join=");
  i32 at = str_indexof(text, key);
  if (at >= 0) {
    String v = {text.value + at + key.len, text.len - (u32)at - key.len};
    i32 end = str_find_any(v, "&# \t\r\n");
    return (String){v.value, end >= 0 ? (u32)end : v.len};
  }
  u32 b = 0, e = text.len;
  while (b < e && mp_blank(text.value[b]))
    b++;
  while (e > b && mp_blank(text.value[e - 1]))
    e--;
  return (String){text.value + b, e - b};
}

void multiplayer_join(Multiplayer *s, String session) {
  assert_msg(is_main_thread(), "multiplayer_join: main lane only"); // online_game is main-lane state
  String id = mp_session_of(session);
  if (!id.len) {
    log_warn("multiplayer: nothing to join in \"%\"", fmt_str(session));
    return;
  }
  if (mp_switchable(s, "multiplayer_join"))
    online_game_join(s->og, id, (String){0});
}

void multiplayer_leave(Multiplayer *s) {
  assert_msg(is_main_thread(), "multiplayer_leave: main lane only"); // online_game is main-lane state
  if (mp_switchable(s, "multiplayer_leave"))
    online_game_leave(s->og);
}

NetcodeRole multiplayer_role(const Multiplayer *s) { return s->role; }
EcsWorld *multiplayer_world(const Multiplayer *s) { return s->world; }
EcsWorld *multiplayer_server_world(const Multiplayer *s) { return s->server_world; }
Netcode *multiplayer_net(const Multiplayer *s) { return s->net; }

String multiplayer_status(const Multiplayer *s) { return s->og ? online_game_status(s->og) : (String){0}; }

String multiplayer_invite(const Multiplayer *s, Allocator *a) {
  return s->og ? online_game_invite_link(s->og, a) : (String){0};
}

OnlineGame *multiplayer_online(const Multiplayer *s) { return s->og; }

NetcodeReplay **multiplayer_replay_slot(Multiplayer *s) { return &s->replay; }
MpLobby *multiplayer_lobby_slot(const Multiplayer *s) { return s->lobby; }
u32 multiplayer_max_players(const Multiplayer *s) { return s->desc.max_players; }
u64 multiplayer_protocol(const Multiplayer *s) { return s->desc.protocol_id; }
const NetcodeRegistry *multiplayer_registry(const Multiplayer *s) { return s->desc.registry; }

u32 multiplayer_players(const Multiplayer *s) {
  return s->net && s->server_world ? netcode_stats(s->net).connections : 0;
}

// ---- names: the verified identity's (a store sign-in makes the Kaigen account under the store's name), else "Player <n>" ----

hz_internal String mp_name_str(const MultiplayerName *p) { return STR((char *)p->text, MIN(p->len, MP_NAME_MAX)); }

// cut on a UTF-8 boundary to what the label holds
hz_internal MultiplayerName mp_name_make(String name, b32 verified) {
  MultiplayerName p = {.verified = verified};
  u32 len = MIN(name.len, MP_NAME_MAX);
  while (len < name.len && len > 0 && utf8_is_continuation((u8)name.value[len]))
    len--;
  mem_cpy(p.text, name.value, len);
  p.len = len;
  return p;
}

// server, per owned ghost, parallel; the identity is read in the tick, after the receive phase that verifies connections
ecs_system(multiplayer_name_server_system, EcsIn(NetGhost), EcsInOut(MultiplayerName), EcsResIn(MultiplayerNet)) {
  const Netcode *net = ecs_sys_get_resource(MultiplayerNet)->net;
  for (i32 i = 0; i < it->count; i++) {
    u32 owner = ecs_sys_get_component(NetGhost, i)->owner;
    const MultiplayerName *now = ecs_sys_get_component(MultiplayerName, i);
    if (now->verified || !owner)
      continue;
    NetcodeIdentity id;
    if (net && netcode_connection_identity(net, owner - 1, &id) && id.name.len) {
      *ecs_sys_get_component_mut(MultiplayerName, i) = mp_name_make(id.name, true);
    } else if (!now->len) {
      tctx_temp_allocator_scope(ta, NULL) {
        *ecs_sys_get_component_mut(MultiplayerName, i) =
            mp_name_make(str_format(&ta.allocator, "Player %", fmt_u32(owner)), false);
      }
    }
  }
}

// client, per nametag, parallel: only tables whose names were written
ecs_system(multiplayer_tag_system, EcsIn(MultiplayerName), EcsInOut(WorldUiLabel)) {
  if (!ecs_iter_field_changed(it, 0))
    return;
  for (i32 i = 0; i < it->count; i++) {
    String name = mp_name_str(ecs_sys_get_component(MultiplayerName, i));
    const WorldUiLabel *l = ecs_sys_get_component(WorldUiLabel, i);
    if (l->text.len != name.len || mem_cmp(l->text.value, name.value, name.len) != 0)
      ecs_sys_get_component_mut(WorldUiLabel, i)->text = fixedstr64_from_len(name.value, name.len);
  }
}

void multiplayer_schedule_server(EcsWorld *w) { ecs_schedule(w, multiplayer_name_server_system); }

void multiplayer_schedule(EcsWorld *w) { ecs_schedule(w, multiplayer_tag_system); }

String multiplayer_name(EcsWorld *w, EcsEntity e) {
  const MultiplayerName *p = ecs_component_get(w, e, MultiplayerName);
  return p ? mp_name_str(p) : (String){0};
}

void multiplayer_nametag(EcsWorld *w, EcsEntity e, v3 offset) {
  assert_msg(is_main_thread(), "multiplayer_nametag: main lane only"); // adds components outside a scope
  String name = multiplayer_name(w, e);
  world_ui_add(w, e, &(WorldUi){.kind = WORLD_UI_NAMETAG, .offset = offset, .max_distance = 60.0f, .fade_s = 0.2f},
               &(WorldUiLabel){.text = fixedstr64_from_len(name.value, name.len), .color = {1.0f, 1.0f, 1.0f, 1.0f}});
}

#include "multiplayer_lobby.c"
