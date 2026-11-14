// Lobbies on Kaigen Online: teams and ready per member, the countdown and the match's session as the owner's attributes.
#include "multiplayer_lobby_internal.h"
#include "lib/assert.h"
#include "lib/fmt.h"
#include "lib/thread_context.h"
#include "lib/utf.h"
#include "os/os.h"

#define MP_LOBBY_OPS 8u
#define MP_LOBBY_JOIN_TRIES 3u
#define MP_LOBBY_JOIN_RETRY_MS 3000.0 // a join of the lobby's match that did not happen is tried again after this

struct MpLobby {
  MultiplayerLobbyState state;
  b32 in;                      // a member of `id`
  OnlineOp *op;                // the create or join in flight
  b32 creating;                // ... a create
  b32 abandon;                 // ... left before it answered: leave once it does
  OnlineOp *leave_op;          // a leave in flight (a quit waits for it)
  OnlineOp *owner_op;          // the owner's state, session or countdown in flight: one at a time
  OnlineOp *team_op;           // this member's team in flight
  OnlineOp *ops[MP_LOBBY_OPS]; // ready, kick, privacy: released once settled
  String64Bytes id;
  MpLobbySnap snap; // the version last taken (0: none yet)
  b32 started;      // this process wrote the STARTING the lobby shows
  b32 host_asked;   // this process hosts the lobby's match
  String64Bytes joined;
  u32 join_tries;   // joins of `joined` so far
  u64 join_failed;  // os_time_now when the last one was seen to fail (0: not failed)
  u32 team_size;
  f32 countdown;
  MultiplayerLobbyMember members[MULTIPLAYER_LOBBY_MEMBERS];
  MpLobbyLine log[MULTIPLAYER_LOBBY_LOG]; // a ring by log_seq
  String log_view[MULTIPLAYER_LOBBY_LOG];
  u32 log_seq;
  u32 log_from; // log_seq when this lobby was entered: the view shows only its lines
  String256Bytes status;
};

// ---- the logic over a snapshot ----

// cut on a UTF-8 boundary to what the fixed string holds, NUL-terminated
hz_internal String64Bytes mp_fix64(String s) {
  String64Bytes out = {0};
  u32 len = MIN(s.len, 63u);
  while (len < s.len && len > 0 && utf8_is_continuation((u8)s.value[len]))
    len--;
  mem_cpy(out.value, s.value, len);
  out.len = len;
  out.value[len] = 0;
  return out;
}

hz_internal String mp_s64(const String64Bytes *s) { return STR(s->value, s->len); }

// "-" is the service's word for none
hz_internal String mp_dash(String v) { return v.len == 1 && v.value[0] == '-' ? (String){0} : v; }

hz_internal b32 mp_u32(String s, u32 *out) {
  if (!s.len || s.len > 9)
    return false;
  u32 v = 0;
  for (u32 i = 0; i < s.len; i++) {
    if (s.value[i] < '0' || s.value[i] > '9')
      return false;
    v = v * 10u + (u32)(s.value[i] - '0');
  }
  *out = v;
  return true;
}

hz_internal b32 mp_attr(const String *keys, const String *vals, u32 n, const char *key, String *out) {
  String k = cstr_to_str(key);
  for (u32 i = 0; i < n; i++)
    if (str_equal(keys[i], k)) {
      *out = vals[i];
      return true;
    }
  return false;
}

void mp_lobby_snap(const OnlineLobby *l, MpLobbySnap *out) {
  *out = (MpLobbySnap){.version = l->version, .policy = l->visibility, .max = l->max_members, .teams = 1};
  out->id = mp_fix64(l->id);
  out->owner = mp_fix64(l->owner);
  out->name = mp_fix64(l->name);
  String v;
  if (mp_attr(l->attr_keys, l->attr_values, l->attr_count, "mode", &v))
    out->mode = mp_fix64(mp_dash(v));
  u32 teams;
  if (mp_attr(l->attr_keys, l->attr_values, l->attr_count, "teams", &v) && mp_u32(v, &teams) && teams > 0)
    out->teams = teams;
  if (mp_attr(l->attr_keys, l->attr_values, l->attr_count, "state", &v))
    out->phase = str_equal(v, STR_FROM_CSTR("starting"))  ? MP_PHASE_STARTING
                 : str_equal(v, STR_FROM_CSTR("playing")) ? MP_PHASE_PLAYING
                                                          : MP_PHASE_LOBBY;
  if (mp_attr(l->attr_keys, l->attr_values, l->attr_count, "start_at", &v) && !str_to_f64(v, &out->start_at))
    out->start_at = 0.0;
  if (mp_attr(l->attr_keys, l->attr_values, l->attr_count, "session", &v))
    out->session = mp_fix64(mp_dash(v));
  out->count = MIN(l->member_count, MULTIPLAYER_LOBBY_MEMBERS);
  for (u32 i = 0; i < out->count; i++) {
    const OnlineLobbyMember *m = &l->members[i];
    MpLobbyMemberRec *r = &out->m[i];
    r->player = mp_fix64(m->player_id);
    r->name = mp_fix64(m->name);
    r->ready = m->ready;
    r->team = MULTIPLAYER_NO_TEAM;
    u32 t;
    if (mp_attr(m->attr_keys, m->attr_values, m->attr_count, "team", &v) && mp_u32(v, &t) && t < out->teams)
      r->team = t;
  }
}

i32 mp_lobby_find(const MpLobbySnap *s, String player) {
  for (u32 i = 0; i < s->count; i++)
    if (str_equal(mp_s64(&s->m[i].player), player))
      return (i32)i;
  return -1;
}

u32 mp_lobby_team_count(const MpLobbySnap *s, u32 team, i32 skip) {
  u32 n = 0;
  for (u32 i = 0; i < s->count; i++)
    if ((i32)i != skip && s->m[i].team == team)
      n++;
  return n;
}

u32 mp_lobby_team_pick(const MpLobbySnap *s, u32 self, u32 team_size) {
  u32 teams = MAX(s->teams, 1u);
  u32 own = s->m[self].team;
  if (own < teams) {
    // the first team_size by join order keep their team; a later one over it moves
    u32 before = 0;
    for (u32 i = 0; i < self; i++)
      if (s->m[i].team == own)
        before++;
    if (before < team_size)
      return own;
  }
  u32 best = MULTIPLAYER_NO_TEAM, best_n = 0;
  for (u32 t = 0; t < teams; t++) {
    if (t == own)
      continue;
    u32 n = mp_lobby_team_count(s, t, (i32)self);
    if (n < team_size && (best == MULTIPLAYER_NO_TEAM || n < best_n)) {
      best = t;
      best_n = n;
    }
  }
  return best != MULTIPLAYER_NO_TEAM ? best : own;
}

hz_internal void mp_line(MpLobbyLine *out, u32 *n, u32 max, String name, const char *what) {
  if (*n >= max)
    return;
  MpLobbyLine *l = &out[(*n)++];
  cstr_format(l->text.value, sizeof(l->text.value), "%%", fmt_str(name), fmt_cstr(what));
  l->text.len = cstr_len(l->text.value);
}

u32 mp_lobby_changes(const MpLobbySnap *prev, const MpLobbySnap *next, MpLobbyLine *out, u32 max) {
  u32 n = 0;
  for (u32 i = 0; i < next->count; i++)
    if (mp_lobby_find(prev, mp_s64(&next->m[i].player)) < 0)
      mp_line(out, &n, max, mp_s64(&next->m[i].name), " joined");
  for (u32 i = 0; i < prev->count; i++)
    if (mp_lobby_find(next, mp_s64(&prev->m[i].player)) < 0)
      mp_line(out, &n, max, mp_s64(&prev->m[i].name), " left");
  if (prev->owner.len && !str_equal(mp_s64(&prev->owner), mp_s64(&next->owner))) {
    i32 o = mp_lobby_find(next, mp_s64(&next->owner));
    if (o >= 0)
      mp_line(out, &n, max, mp_s64(&next->m[o].name), " is now the host");
  }
  return n;
}

MpLobbyDo mp_lobby_decide(const MpLobbySnap *s, const MpLobbyLocal *l, MultiplayerLobbyState *view) {
  b32 owner = str_equal(mp_s64(&s->owner), l->self);
  String session = mp_s64(&s->session);
  switch (s->phase) {
  case MP_PHASE_STARTING: {
    if (owner && !l->started) {
      *view = MULTIPLAYER_LOBBY_OPEN;
      return MP_DO_RESET;
    }
    b32 over = l->now > 0.0 && l->now >= s->start_at;
    *view = over ? MULTIPLAYER_LOBBY_LOADING : MULTIPLAYER_LOBBY_STARTING;
    if (owner && over && !l->hosting)
      return MP_DO_HOST;
    if (owner && l->hosting && l->hosted.len)
      return MP_DO_PUBLISH;
    return MP_DO_NOTHING;
  }
  case MP_PHASE_PLAYING:
    if (owner) {
      // a host that left took its match with it: the new one brings everyone back
      if (!l->hosting || !str_equal(l->hosted, session)) {
        *view = MULTIPLAYER_LOBBY_OPEN;
        return MP_DO_RESET;
      }
      *view = l->playing ? MULTIPLAYER_LOBBY_PLAYING : MULTIPLAYER_LOBBY_LOADING;
      return MP_DO_NOTHING;
    }
    if (l->in_match && str_equal(l->joined, session)) {
      *view = l->playing ? MULTIPLAYER_LOBBY_PLAYING : MULTIPLAYER_LOBBY_LOADING;
      return MP_DO_NOTHING;
    }
    *view = MULTIPLAYER_LOBBY_LOADING;
    return session.len && !str_equal(l->joined, session) ? MP_DO_JOIN : MP_DO_NOTHING;
  default:
    *view = MULTIPLAYER_LOBBY_OPEN;
    return l->hosting || l->in_match ? MP_DO_LEAVE_MATCH : MP_DO_NOTHING;
  }
}

hz_internal b32 mp_hex(char c) { return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'); }

String mp_lobby_code(String text) {
  u32 i = 0;
  while (i < text.len) {
    u32 j = i;
    while (j < text.len && mp_hex(text.value[j]))
      j++;
    if (j - i == 24)
      return STR(text.value + i, 24);
    i = j > i ? j : i + 1;
  }
  return (String){0};
}

// ---- the block's lobby ----

MpLobby *mp_lobby_create(Allocator *a) { return ALLOC(a, MpLobby); }

hz_internal OnlineServices *mpl_online(const Multiplayer *m) {
  OnlineGame *og = multiplayer_online(m);
  return og ? online_game_services(og) : NULL;
}

hz_internal void mpl_log(MpLobby *l, String text) {
  MpLobbyLine *line = &l->log[l->log_seq % MULTIPLAYER_LOBBY_LOG];
  u32 len = MIN(text.len, (u32)sizeof(line->text.value) - 1u);
  while (len < text.len && len > 0 && utf8_is_continuation((u8)text.value[len]))
    len--;
  mem_cpy(line->text.value, text.value, len);
  line->text.value[len] = 0;
  line->text.len = len;
  l->log_seq++;
  log_info("multiplayer lobby: %", fmt_str(STR(line->text.value, len)));
}

hz_internal void mpl_status(MpLobby *l, String text) {
  u32 len = MIN(text.len, (u32)sizeof(l->status.value) - 1u);
  mem_cpy(l->status.value, text.value, len);
  l->status.value[len] = 0;
  l->status.len = len;
  if (len)
    log_info("multiplayer lobby: %", fmt_str(text));
}

// an op released once it settles; a failure is logged
hz_internal b32 mpl_settled(OnlineServices *o, OnlineOp **op, const char *what) {
  if (!*op || online_op_state(*op) == ONLINE_OP_PENDING)
    return false;
  if (online_op_state(*op) == ONLINE_OP_FAILED)
    log_warn("multiplayer lobby: % failed: %", fmt_cstr(what), fmt_cstr(online_op_error(*op)));
  online_op_release(o, *op);
  *op = NULL;
  return true;
}

hz_internal void mpl_track(OnlineServices *o, MpLobby *l, OnlineOp *op) {
  for (u32 i = 0; i < MP_LOBBY_OPS; i++)
    if (!l->ops[i]) {
      l->ops[i] = op;
      return;
    }
  log_warn("multiplayer lobby: % requests in flight already: the oldest is no longer awaited", fmt_u32(MP_LOBBY_OPS));
  online_op_release(o, l->ops[0]);
  for (u32 i = 1; i < MP_LOBBY_OPS; i++)
    l->ops[i - 1] = l->ops[i];
  l->ops[MP_LOBBY_OPS - 1] = op;
}

// in no lobby: what was kept of the last one goes, its log stays readable
hz_internal void mpl_clear(MpLobby *l) {
  l->in = false;
  l->state = MULTIPLAYER_LOBBY_NONE;
  l->snap = (MpLobbySnap){0};
  l->id = (String64Bytes){0};
  l->started = l->host_asked = false;
  l->joined = (String64Bytes){0};
  l->join_tries = 0;
  l->join_failed = 0;
  l->countdown = 0.0f;
}

hz_internal b32 mpl_hosting(const Multiplayer *m) {
  OnlineGame *og = multiplayer_online(m);
  OnlineGameState st = og ? online_game_state(og) : ONLINE_GAME_OFF;
  return st == ONLINE_GAME_LISTING || st == ONLINE_GAME_HOSTING;
}

hz_internal b32 mpl_in_match(const Multiplayer *m, const MpLobby *l) {
  OnlineGame *og = multiplayer_online(m);
  OnlineGameState st = og ? online_game_state(og) : ONLINE_GAME_OFF;
  return l->joined.len && (st == ONLINE_GAME_JOINING || st == ONLINE_GAME_PLAYING);
}

// out of the lobby's match (the host's session ends, a client leaves it)
hz_internal void mpl_leave_match(Multiplayer *m, MpLobby *l) {
  if ((l->host_asked && mpl_hosting(m)) || mpl_in_match(m, l)) {
    log_info("multiplayer lobby: out of the match");
    multiplayer_leave(m);
  }
  l->host_asked = false;
  l->joined = (String64Bytes){0};
  l->join_tries = 0;
  l->join_failed = 0;
}

// the lobby left (and its match, unless a quit ends that itself); a create or join in flight is left once it answers
hz_internal void mpl_leave(Multiplayer *m, MpLobby *l, b32 match) {
  OnlineServices *o = mpl_online(m);
  if (match)
    mpl_leave_match(m, l);
  if (l->op)
    l->abandon = true;
  if (l->in && o) {
    if (l->leave_op)
      online_op_release(o, l->leave_op);
    l->leave_op = online_lobby_leave(o, l->id.value);
    log_info("multiplayer lobby: leaving %", fmt_cstr(l->id.value));
  }
  mpl_clear(l);
}

hz_internal b32 mpl_owner_write(OnlineServices *o, MpLobby *l, const OnlineAttr *a, u32 n) {
  if (l->owner_op)
    return false;
  l->owner_op = online_lobby_update(o, l->id.value, &(OnlineLobbyDesc){.attrs = a, .attr_count = n}, NULL, 0, -1);
  return true;
}

hz_internal b32 mpl_owner(const MpLobby *l, String self) {
  return l->in && l->snap.version && str_equal(mp_s64(&l->snap.owner), self);
}

hz_internal String mpl_self(OnlineServices *o) {
  OnlinePlayer me;
  return o && online_player(o, &me) ? me.player_id : (String){0};
}

// the view's members from the snapshot
hz_internal void mpl_view(MpLobby *l, String self) {
  for (u32 i = 0; i < l->snap.count; i++) {
    const MpLobbyMemberRec *r = &l->snap.m[i];
    l->members[i] = (MultiplayerLobbyMember){.player = mp_s64(&r->player),
                                             .name = mp_s64(&r->name),
                                             .team = r->team,
                                             .ready = r->ready,
                                             .host = str_equal(mp_s64(&r->player), mp_s64(&l->snap.owner)),
                                             .self = str_equal(mp_s64(&r->player), self)};
  }
}

hz_internal void mpl_team_send(OnlineServices *o, MpLobby *l, u32 team) {
  if (l->team_op)
    online_op_release(o, l->team_op);
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  char *value = str_to_cstr(str_format(&ta.allocator, "%", fmt_u32(team)), &ta.allocator);
  l->team_op = online_lobby_update(o, l->id.value, NULL, &(OnlineAttr){"team", value}, 1, -1);
  tctx_temp_allocator_end(ta);
}

// a create or join answered
hz_internal void mpl_answered(Multiplayer *m, MpLobby *l, OnlineServices *o) {
  OnlineOp *op = l->op;
  l->op = NULL;
  if (online_op_state(op) == ONLINE_OP_DONE) {
    String id = online_op_text(op);
    if (l->abandon) {
      l->abandon = false;
      if (l->leave_op)
        online_op_release(o, l->leave_op);
      TempAllocator ta = tctx_temp_allocator_begin(NULL);
      l->leave_op = online_lobby_leave(o, str_to_cstr(id, &ta.allocator));
      tctx_temp_allocator_end(ta);
    } else {
      mpl_clear(l);
      l->id = mp_fix64(id);
      l->in = true;
      l->state = MULTIPLAYER_LOBBY_OPEN;
      log_info("multiplayer lobby: % %", fmt_cstr(l->creating ? "created" : "joined"), fmt_cstr(l->id.value));
    }
  } else if (!l->abandon) {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    mpl_status(l, str_format(&ta.allocator, "Could not % the lobby: %", fmt_cstr(l->creating ? "create" : "join"),
                             fmt_cstr(online_op_error(op))));
    tctx_temp_allocator_end(ta);
    l->state = MULTIPLAYER_LOBBY_NONE;
  } else {
    l->abandon = false;
  }
  online_op_release(o, op);
  UNUSED(m);
}

// a new version of the lobby: its log lines, the ready reset after a match
hz_internal void mpl_take(MpLobby *l, OnlineServices *o, const OnlineLobby *doc, String self) {
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  MpLobbySnap *next = ALLOC(&ta.allocator, MpLobbySnap);
  mp_lobby_snap(doc, next);
  if (l->snap.version) {
    MpLobbyLine *lines = ALLOC_ARRAY(&ta.allocator, MpLobbyLine, 2u * MULTIPLAYER_LOBBY_MEMBERS + 1u);
    u32 n = mp_lobby_changes(&l->snap, next, lines, 2u * MULTIPLAYER_LOBBY_MEMBERS + 1u);
    for (u32 i = 0; i < n; i++)
      mpl_log(l, STR(lines[i].text.value, lines[i].text.len));
  } else {
    i32 me = mp_lobby_find(next, self);
    if (me >= 0)
      mpl_log(l, str_format(&ta.allocator, "%%", fmt_str(mp_s64(&next->m[me].name)),
                            fmt_cstr(str_equal(mp_s64(&next->owner), self) ? " created the lobby" : " joined")));
  }
  u32 was = l->snap.phase;
  b32 first = !l->snap.version;
  l->snap = *next;
  tctx_temp_allocator_end(ta);
  i32 me = mp_lobby_find(&l->snap, self);
  // back from a match: everyone readies again
  if (!first && was == MP_PHASE_PLAYING && l->snap.phase == MP_PHASE_LOBBY && me >= 0 && l->snap.m[me].ready)
    mpl_track(o, l, online_lobby_update(o, l->id.value, NULL, NULL, 0, 0));
  if (l->snap.phase == MP_PHASE_LOBBY)
    l->started = false;
}

void mp_lobby_frame(Multiplayer *m, MpLobby *l) {
  OnlineServices *o = mpl_online(m);
  if (!o)
    return;
  for (u32 i = 0; i < MP_LOBBY_OPS; i++)
    mpl_settled(o, &l->ops[i], "a lobby request");
  // compact the in-flight list
  u32 k = 0;
  for (u32 i = 0; i < MP_LOBBY_OPS; i++)
    if (l->ops[i])
      l->ops[k++] = l->ops[i];
  for (; k < MP_LOBBY_OPS; k++)
    l->ops[k] = NULL;
  mpl_settled(o, &l->owner_op, "the lobby's state");
  mpl_settled(o, &l->team_op, "a team change");
  mpl_settled(o, &l->leave_op, "leaving the lobby");
  if (l->op && online_op_state(l->op) != ONLINE_OP_PENDING)
    mpl_answered(m, l, o);
  if (!l->in)
    return;
  OnlineLobby doc;
  String self = mpl_self(o);
  if (!self.len || !online_lobby(o, l->id.value, &doc))
    return;
  if (doc.version != l->snap.version)
    mpl_take(l, o, &doc, self);
  i32 me = mp_lobby_find(&l->snap, self);
  if (me < 0)
    return;
  u32 teams = MAX(l->snap.teams, 1u);
  l->team_size = MAX(l->snap.max / teams, 1u);
  u32 team = mp_lobby_team_pick(&l->snap, (u32)me, l->team_size);
  if (team != MULTIPLAYER_NO_TEAM && team != l->snap.m[me].team && !l->team_op)
    mpl_team_send(o, l, team);
  // the lobby's word against this player's game
  OnlineGame *og = multiplayer_online(m);
  OnlineGameState st = online_game_state(og);
  b32 hosting = l->host_asked && mpl_hosting(m);
  String hosted = hosting && st == ONLINE_GAME_HOSTING ? online_session_id(o) : (String){0};
  b32 in_match = mpl_in_match(m, l);
  // a join of the lobby's match that did not happen (the host was not up yet, a lost answer) is tried again
  b32 rejoin = false;
  if (l->joined.len && !in_match && str_equal(mp_s64(&l->joined), mp_s64(&l->snap.session)) && l->join_tries < MP_LOBBY_JOIN_TRIES) {
    if (!l->join_failed)
      l->join_failed = os_time_now();
    else
      rejoin = os_ticks_to_ms(os_time_diff(os_time_now(), l->join_failed)) >= MP_LOBBY_JOIN_RETRY_MS;
  }
  b32 playing = hosting ? st == ONLINE_GAME_HOSTING && multiplayer_net(m) != NULL : st == ONLINE_GAME_PLAYING && in_match;
  f64 now = online_server_time(o);
  MpLobbyLocal loc = {.self = self, .started = l->started, .hosting = hosting, .hosted = hosted, .in_match = in_match,
                      .playing = playing, .joined = rejoin ? (String){0} : mp_s64(&l->joined), .now = now};
  // a host whose match did not come up gives the lobby back
  if (l->host_asked && !hosting && mpl_owner(l, self) && l->snap.phase != MP_PHASE_LOBBY && !l->owner_op) {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    mpl_log(l, str_format(&ta.allocator, "The match could not be hosted: %", fmt_str(multiplayer_status(m))));
    tctx_temp_allocator_end(ta);
    l->host_asked = l->started = false;
    mpl_owner_write(o, l, (OnlineAttr[]){{"state", "lobby"}, {"session", "-"}, {"start_at", "0"}}, 3);
  }
  MultiplayerLobbyState view = MULTIPLAYER_LOBBY_OPEN;
  MpLobbyDo act = mp_lobby_decide(&l->snap, &loc, &view);
  l->state = view;
  l->countdown = l->snap.phase == MP_PHASE_STARTING && now > 0.0 ? (f32)(MAX(l->snap.start_at - now, 0.0) / 1000.0) : 0.0f;
  switch (act) {
  case MP_DO_HOST:
    log_info("multiplayer lobby: the countdown ended: hosting the match");
    l->host_asked = true;
    // listed as public: the service admits invited players only through friend invites, and the lobby's members are no friends of the host
    multiplayer_host(m, ONLINE_POLICY_PUBLIC);
    break;
  case MP_DO_PUBLISH: {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    if (mpl_owner_write(o, l, (OnlineAttr[]){{"state", "playing"}, {"session", str_to_cstr(hosted, &ta.allocator)}}, 2))
      log_info("multiplayer lobby: the match % is up: everyone joins it", fmt_str(hosted));
    tctx_temp_allocator_end(ta);
    break;
  }
  case MP_DO_JOIN:
    l->join_tries = str_equal(mp_s64(&l->joined), mp_s64(&l->snap.session)) ? l->join_tries + 1u : 1u;
    l->joined = l->snap.session;
    l->join_failed = 0;
    log_info("multiplayer lobby: joining the match % (try %)", fmt_cstr(l->joined.value), fmt_u32(l->join_tries));
    multiplayer_join(m, mp_s64(&l->joined));
    break;
  case MP_DO_LEAVE_MATCH:
    mpl_leave_match(m, l);
    break;
  case MP_DO_RESET:
    if (mpl_owner_write(o, l, (OnlineAttr[]){{"state", "lobby"}, {"session", "-"}, {"start_at", "0"}}, 3))
      log_info("multiplayer lobby: the lobby is back: its match is over");
    break;
  default:
    break;
  }
  mpl_view(l, self);
}

void mp_lobby_event(Multiplayer *m, MpLobby *l, const OnlineEvent *e) {
  if (e->type != ONLINE_EV_LOBBY_GONE || !l->in || !str_equal(e->lobby, mp_s64(&l->id)))
    return;
  b32 kicked = str_equal(e->text, STR_FROM_CSTR("kicked"));
  log_info("multiplayer lobby: out of % (%)", fmt_cstr(l->id.value), fmt_str(e->text));
  mpl_leave_match(m, l);
  mpl_clear(l);
  mpl_status(l, kicked ? STR_FROM_CSTR("You were removed from the lobby")
                       : STR_FROM_CSTR("You lost the lobby: disconnected from Kaigen Online"));
}

b32 mp_lobby_quitting(Multiplayer *m, MpLobby *l) {
  if (l->in || l->op)
    mpl_leave(m, l, false);
  return (l->leave_op && online_op_state(l->leave_op) == ONLINE_OP_PENDING) || l->op;
}

// ---- the API ----

void multiplayer_lobby_create(Multiplayer *m, const MultiplayerLobbyDesc *d) {
  assert_msg(is_main_thread(), "multiplayer_lobby_create: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  OnlinePlayer me;
  if (!o || !online_player(o, &me)) {
    mpl_status(l, !o ? STR_FROM_CSTR("Lobbies need Kaigen Online") : STR_FROM_CSTR("Not signed in to Kaigen Online yet"));
    return;
  }
  if (l->in || l->op) {
    log_warn("multiplayer lobby: create refused: in a lobby already (leave it first)");
    return;
  }
  u32 max = MIN(d->max_players ? d->max_players : multiplayer_max_players(m), MULTIPLAYER_LOBBY_MEMBERS);
  u32 teams = MAX(d->teams, 1u);
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  String name = d->name ? cstr_to_str(d->name) : str_format(&ta.allocator, "%'s lobby", fmt_str(me.display_name));
  String64Bytes cut = mp_fix64(name);
  char *teams_s = str_to_cstr(str_format(&ta.allocator, "%", fmt_u32(teams)), &ta.allocator);
  char *proto_s = str_to_cstr(str_format(&ta.allocator, "%", fmt_u64(multiplayer_protocol(m))), &ta.allocator);
  OnlineAttr attrs[6] = {{"state", "lobby"}, {"session", "-"}, {"start_at", "0"}, {"teams", teams_s}, {"protocol", proto_s},
                         {"mode", d->mode && d->mode[0] ? d->mode : "-"}};
  l->log_from = l->log_seq;
  l->op = online_lobby_create(o, &(OnlineLobbyDesc){.name = cut.value, .max_members = max, .visibility = d->policy,
                                                     .attrs = attrs, .attr_count = 6});
  tctx_temp_allocator_end(ta);
  l->creating = true;
  l->abandon = false;
  l->state = MULTIPLAYER_LOBBY_JOINING;
  l->status = (String256Bytes){0};
  log_info("multiplayer lobby: creating \"%\" (% players, % teams)", fmt_cstr(cut.value), fmt_u32(max), fmt_u32(teams));
}

void multiplayer_lobby_join(Multiplayer *m, String id_or_code) {
  assert_msg(is_main_thread(), "multiplayer_lobby_join: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  String code = mp_lobby_code(id_or_code);
  if (!o || !code.len) {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    mpl_status(l, !o ? STR_FROM_CSTR("Lobbies need Kaigen Online")
                     : str_format(&ta.allocator, "No lobby code in \"%\"", fmt_str(id_or_code)));
    tctx_temp_allocator_end(ta);
    return;
  }
  if ((l->in && str_equal(mp_s64(&l->id), code)) || l->op)
    return;
  if (l->in)
    mpl_leave(m, l, true);
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  l->log_from = l->log_seq;
  l->op = online_lobby_join(o, str_to_cstr(code, &ta.allocator));
  tctx_temp_allocator_end(ta);
  l->creating = false;
  l->abandon = false;
  l->state = MULTIPLAYER_LOBBY_JOINING;
  l->status = (String256Bytes){0};
  log_info("multiplayer lobby: joining %", fmt_str(code));
}

void multiplayer_lobby_leave(Multiplayer *m) {
  assert_msg(is_main_thread(), "multiplayer_lobby_leave: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  mpl_leave(m, l, true);
  l->status = (String256Bytes){0};
}

void multiplayer_lobby_ready(Multiplayer *m, b32 ready) {
  assert_msg(is_main_thread(), "multiplayer_lobby_ready: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  if (!l->in || !o)
    return;
  mpl_track(o, l, online_lobby_update(o, l->id.value, NULL, NULL, 0, ready ? 1 : 0));
}

b32 multiplayer_lobby_team(Multiplayer *m, u32 team) {
  assert_msg(is_main_thread(), "multiplayer_lobby_team: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  i32 me = o ? mp_lobby_find(&l->snap, mpl_self(o)) : -1;
  if (!l->in || me < 0 || team >= MAX(l->snap.teams, 1u)) {
    log_warn("multiplayer lobby: no team % to switch to", fmt_u32(team));
    return false;
  }
  if (l->snap.m[me].team == team)
    return true;
  if (mp_lobby_team_count(&l->snap, team, me) >= l->team_size) {
    TempAllocator ta = tctx_temp_allocator_begin(NULL);
    mpl_log(l, str_format(&ta.allocator, "Team % is full", fmt_u32(team + 1u)));
    tctx_temp_allocator_end(ta);
    return false;
  }
  mpl_team_send(o, l, team);
  return true;
}

void multiplayer_lobby_start(Multiplayer *m) {
  assert_msg(is_main_thread(), "multiplayer_lobby_start: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  f64 now = o ? online_server_time(o) : 0.0;
  if (!o || !mpl_owner(l, mpl_self(o)) || l->snap.phase != MP_PHASE_LOBBY) {
    log_warn("multiplayer lobby: start refused: only the host starts, from the lobby");
    return;
  }
  if (now <= 0.0 || l->owner_op) {
    log_warn("multiplayer lobby: start refused: % ", fmt_cstr(now <= 0.0 ? "the service's clock is not known yet" : "the lobby is busy"));
    return;
  }
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  char *at = str_to_cstr(str_format(&ta.allocator, "%", fmt_u64((u64)(now + MP_LOBBY_COUNTDOWN_MS))), &ta.allocator);
  mpl_owner_write(o, l, (OnlineAttr[]){{"state", "starting"}, {"start_at", at}, {"session", "-"}}, 3);
  l->started = true;
  log_info("multiplayer lobby: starting at % (the service's clock)", fmt_cstr(at));
  tctx_temp_allocator_end(ta);
}

void multiplayer_lobby_cancel(Multiplayer *m) {
  assert_msg(is_main_thread(), "multiplayer_lobby_cancel: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  if (!o || !mpl_owner(l, mpl_self(o)) || l->snap.phase == MP_PHASE_LOBBY)
    return;
  if (l->owner_op) {
    online_op_release(o, l->owner_op);
    l->owner_op = NULL;
  }
  log_info("multiplayer lobby: % cancelled: back to the lobby", fmt_cstr(l->snap.phase == MP_PHASE_PLAYING ? "the match" : "the countdown"));
  mpl_owner_write(o, l, (OnlineAttr[]){{"state", "lobby"}, {"session", "-"}, {"start_at", "0"}}, 3);
  l->started = false;
}

void multiplayer_lobby_kick(Multiplayer *m, String player) {
  assert_msg(is_main_thread(), "multiplayer_lobby_kick: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  if (!o || !mpl_owner(l, mpl_self(o)) || mp_lobby_find(&l->snap, player) < 0)
    return;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  mpl_track(o, l, online_lobby_kick(o, l->id.value, str_to_cstr(player, &ta.allocator)));
  tctx_temp_allocator_end(ta);
  log_info("multiplayer lobby: kicking %", fmt_str(player));
}

void multiplayer_lobby_privacy(Multiplayer *m, OnlinePolicy policy) {
  assert_msg(is_main_thread(), "multiplayer_lobby_privacy: main lane only"); // the online services are main-lane state
  MpLobby *l = multiplayer_lobby_slot(m);
  OnlineServices *o = mpl_online(m);
  if (!o || !mpl_owner(l, mpl_self(o)) || l->snap.policy == policy)
    return;
  mpl_track(o, l, online_lobby_update(o, l->id.value, &(OnlineLobbyDesc){.visibility = policy, .set_visibility = true}, NULL, 0, -1));
}

b32 multiplayer_lobby(const Multiplayer *m, MultiplayerLobbyView *out) {
  MpLobby *l = multiplayer_lobby_slot(m);
  u32 n = MIN(l->log_seq - l->log_from, MULTIPLAYER_LOBBY_LOG);
  for (u32 i = 0; i < n; i++) {
    const MpLobbyLine *line = &l->log[(l->log_seq - n + i) % MULTIPLAYER_LOBBY_LOG];
    l->log_view[i] = STR(line->text.value, line->text.len);
  }
  // a member whose first push has not come yet is still joining
  b32 in = l->in && l->snap.version;
  MultiplayerLobbyState st = in ? l->state : l->in || (l->op && !l->abandon) ? MULTIPLAYER_LOBBY_JOINING : MULTIPLAYER_LOBBY_NONE;
  *out = (MultiplayerLobbyView){.state = st, .log_count = n, .log = l->log_view, .log_seq = l->log_seq,
                                .status = STR(l->status.value, l->status.len)};
  if (!in)
    return out->state != MULTIPLAYER_LOBBY_NONE;
  OnlineServices *o = mpl_online(m);
  out->id = mp_s64(&l->id);
  out->name = mp_s64(&l->snap.name);
  out->mode = mp_s64(&l->snap.mode);
  out->policy = l->snap.policy;
  out->teams = MAX(l->snap.teams, 1u);
  out->team_size = l->team_size;
  out->max_players = l->snap.max;
  out->host = mpl_owner(l, mpl_self(o));
  out->countdown = l->countdown;
  out->member_count = l->snap.count;
  out->members = l->members;
  return true;
}

u32 multiplayer_lobby_team_of(const Multiplayer *m, u32 connection) {
  MpLobby *l = multiplayer_lobby_slot(m);
  Netcode *n = multiplayer_net(m);
  NetcodeIdentity id;
  if (!l->in || !l->snap.version || !n || !netcode_connection_identity(n, connection, &id))
    return MULTIPLAYER_NO_TEAM;
  i32 i = mp_lobby_find(&l->snap, id.player);
  return i < 0 ? MULTIPLAYER_NO_TEAM : l->snap.m[i].team;
}
