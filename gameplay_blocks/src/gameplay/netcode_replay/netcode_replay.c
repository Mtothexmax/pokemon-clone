#include "netcode_replay.h"
#include "netcode_replay_internal.h"
#include "app/app.h"
#include "assets/assets.h"
#include "camera_controllers/camera_controllers.h"
#include "ecs/camera/ecs_camera.h"
#include "ecs/input/ecs_input.h"
#include "ecs/renderer/ecs_renderer.h"
#include "ecs/transform/ecs_transform.h"
#include "hz_command_registry.h"
#include "hz_engine.h"
#include "input.h"
#include "lib/allocator_tlsf.h"
#include "lib/assert.h"
#include "lib/string.h"
#include "lib/thread_context.h"
#include "rendering/renderer.h"

#define NRP_RING_SECONDS 10.0f
#define NRP_FOLLOW_BACK 4.0f   // metres behind the followed ghost
#define NRP_FOLLOW_UP 1.6f     // and above its position
#define NRP_FOLLOW_AIM 1.2f    // the point looked at, above its position
#define NRP_FREE_SPEED 8.0f    // m/s
#define NRP_FREE_BACK 7.0f     // the free camera starts this far behind the recorded player
#define NRP_FREE_UP 9.0f       // and this far above it: over the walls a free camera does not collide with
#define NRP_FOLLOW_EASE 6.0f   // 1/s: how fast the chase direction turns

struct NetcodeReplay {
  Multiplayer *mp;
  NetcodeReplayDesc desc;
  TLSFAllocator *tlsf;
  Allocator alloc; // the recording's, the clips' and the playback's
  EcsWorld *live;
  EcsWorld *world;
  RendererState *r;
  EcsQuery *live_camera;
  EcsEntity camera;
  NetcodeRecorder *rec;
  b32 tracked; // the netcode's client was taken (recorded, or refused); netcode_replay_reset clears it
  NetcodeClip *clip;
  NetcodePlayback *pb;
  NetcodeReplayCamera mode;
  f32 end;      // the clip's position it holds at
  u32 follow;
  f32 rate;
  b32 paused;
  b32 active;
  u8 live_mask; // the main view mask the replay replaced
  b32 fresh;    // the camera starts where the live one is on the replay's first frame
  v3 chase;     // FOLLOW: the horizontal direction the camera looks along
  FlyCameraCtrl fly;
};


hz_internal NetcodeReplay *nrp_get(const Multiplayer *mp) { return *multiplayer_replay_slot((Multiplayer *)mp); }

// main lane: the clip's playback in the replay world, the replay drawn in place of the live world
hz_internal b32 nrp_play(NetcodeReplay *r, NetcodeClip *clip, const NetcodeReplayStartDesc *d) {
  NetcodePlayback *pb = netcode_playback_begin(&(NetcodePlaybackDesc){.clip = clip, .world = r->world}, &r->alloc);
  if (!pb) {
    ALLOC_FREE(&r->alloc, clip);
    return false;
  }
  r->clip = clip;
  r->pb = pb;
  // a clip starts at the whole keyframe before the seconds asked for: begin at the seconds themselves
  f32 from = netcode_playback_duration(pb) - d->seconds_back;
  if (d->seconds_back > 0.0f && from > 0.0f)
    netcode_playback_seek(pb, from);
  r->end = MAX(netcode_playback_duration(pb) - MAX(d->end_back, 0.0f), netcode_playback_position(pb));
  r->mode = d->camera;
  r->follow = d->follow_ghost;
  r->rate = d->rate > 0.0f ? d->rate : 1.0f;
  r->paused = false;
  r->active = true;
  r->fresh = true;
  r->live_mask = renderer_main_view_mask(r->r);
  renderer_set_main_view_mask(r->r, ECS_RENDERER_VIEW_MASK_ATTACHED);
  return true;
}

hz_internal void nrp_end(NetcodeReplay *r) {
  if (!r->active)
    return;
  netcode_playback_end(r->pb);
  ALLOC_FREE(&r->alloc, r->clip);
  r->pb = NULL;
  r->clip = NULL;
  r->active = false;
  renderer_set_main_view_mask(r->r, r->live_mask);
}

// main lane: the followed ghost's position, from its type's replicated position field as this frame drew it
hz_internal b32 nrp_ghost_position(NetcodeReplay *r, u32 ghost, v3 *out) {
  EcsEntity e = netcode_playback_ghost_entity(r->pb, ghost);
  const NetGhost *g = e ? ecs_component_get(r->world, e, NetGhost) : NULL;
  const NetcodeRegistry *reg = multiplayer_registry(r->mp);
  if (!g || g->type >= reg->ghost_count || !reg->ghosts[g->type]->position.enabled)
    return false;
  const NetGhostTypeDesc *t = reg->ghosts[g->type];
  const NetComponentDesc *c = t->components[t->position.component];
  const u8 *data = _ecs_component_get_id(r->world, e, ecs_component_ensure(r->world, c->size, c->align, c->name));
  if (!data)
    return false;
  mem_cpy(out, data + t->position.offset, sizeof(v3));
  return true;
}

// main lane: the ghost the recorded client owned (NetGhost.owner 1), 0 when the replay holds none
hz_internal u32 nrp_own_ghost(NetcodeReplay *r) {
  u32 own = 0;
  ecs_query_foreach(ecs_query(r->world, EcsIn(NetGhost)), it) {
    const NetGhost *g = ecs_iter_get_component(&it, NetGhost, 0);
    for (i32 i = 0; i < it.count && !own; i++)
      if (g[i].owner == 1)
        own = g[i].id;
  }
  return own;
}

// main lane: `back` behind the ghost along the chase direction and `up` above it, looking at it
hz_internal void nrp_camera_behind(NetcodeReplay *r, Camera *cam, v3 at, f32 back, f32 up) {
  v3 aim = v3_add(at, (v3){0.0f, NRP_FOLLOW_AIM, 0.0f});
  cam->pos = v3_add(v3_sub(at, v3_scale(r->chase, back)), (v3){0.0f, up, 0.0f});
  cam->rot = quat_look(v3_negate(v3_sub(aim, cam->pos)), V3_UP);
}

// main lane, the replay's first frame: the live camera's lens and heading; FREE starts over the recorded player
hz_internal void nrp_camera_from_live(NetcodeReplay *r, Camera *cam) {
  r->chase = (v3){0.0f, 0.0f, -1.0f};
  const Camera *live = r->live_camera ? ecs_query_get_singleton(Camera, r->live_camera) : NULL;
  if (live) {
    cam->pos = live->pos;
    cam->rot = live->rot;
    cam->fov = live->fov;
    cam->near_plane = live->near_plane;
    cam->far_plane = live->far_plane;
    v3 f = camera_forward(live);
    if (f.x * f.x + f.z * f.z > 1e-6f)
      r->chase = v3_normalize((v3){f.x, 0.0f, f.z});
  }
  v3 at;
  u32 own = r->mode == NETCODE_REPLAY_CAMERA_FREE ? nrp_own_ghost(r) : 0;
  if (own && nrp_ghost_position(r, own, &at))
    nrp_camera_behind(r, cam, at, NRP_FREE_BACK, NRP_FREE_UP);
  flycam_sync_from_camera(&r->fly, cam);
  r->fly.move_speed = NRP_FREE_SPEED;
}

// main lane: FOLLOW chases the ghost, FREE flies on the live world's input; GAME is the game's
hz_internal void nrp_camera(NetcodeReplay *r, f32 dt) {
  Camera *cam = ecs_component_get_mut(r->world, r->camera, Camera);
  if (r->fresh) {
    r->fresh = false;
    nrp_camera_from_live(r, cam);
  }
  if (r->mode == NETCODE_REPLAY_CAMERA_FREE) {
    const InputSystem *input = ecs_resource_get(r->live, InputSystem);
    if (input)
      flycam_update(&r->fly, cam, input, dt);
    return;
  }
  v3 at;
  if (r->mode != NETCODE_REPLAY_CAMERA_FOLLOW || !nrp_ghost_position(r, r->follow, &at))
    return;
  v3 aim = v3_add(at, (v3){0.0f, NRP_FOLLOW_AIM, 0.0f});
  v3 to = v3_sub(aim, cam->pos);
  to.y = 0.0f;
  if (v3_dot(to, to) > 1e-4f) {
    f32 k = 1.0f - m_expf(-NRP_FOLLOW_EASE * dt);
    v3 c = v3_add(v3_scale(r->chase, 1.0f - k), v3_scale(v3_normalize(to), k));
    if (v3_dot(c, c) > 1e-6f)
      r->chase = v3_normalize(c);
  }
  nrp_camera_behind(r, cam, at, NRP_FOLLOW_BACK, NRP_FOLLOW_UP);
}

// "netcode_replay_save:<path>[:<seconds>]": a trailing number is the seconds, so a drive letter's colon stays in the path
hz_internal void nrp_cmd_save(const HzCommandCtx *ctx, void *user) {
  Multiplayer *mp = user;
  TempAllocator ta = tctx_temp_allocator_begin(NULL);
  String args = str_from_cstr_alloc(ctx->args ? ctx->args : "", &ta.allocator);
  f32 seconds = 0.0f;
  i32 colon = str_find_last(args, STR(":", 1));
  if (colon > 1) {
    if (cstr_to_f32(args.value + colon + 1, &seconds))
      args.value[colon] = 0;
    else
      seconds = 0.0f;
  }
  b32 ok = args.value[0] && netcode_replay_save(mp, args.value, seconds);
  if (!ok && ctx->result)
    ctx->result->ok = false;
  tctx_temp_allocator_end(ta);
}

void netcode_replay_init(Multiplayer *mp, const NetcodeReplayDesc *d) {
  assert_msg(is_main_thread(), "netcode_replay_init: main lane only"); // creates a world and registers a command outside scopes
  NetcodeReplay **slot = multiplayer_replay_slot(mp);
  EcsWorld *live = multiplayer_world(mp);
  RendererState *rs = live ? ecs_resource_get_mut(live, RendererState) : NULL;
  if (*slot || !rs || !ecs_resource_get(live, AssetSystem)) {
    log_error("netcode replay: init once, after ecs_assets_init and ecs_renderer_init on the multiplayer world");
    return;
  }
  TLSFAllocator *tlsf = tlsf_allocator_create(MB(512), MB(4));
  Allocator alloc = make_tlsf_allocator(tlsf);
  NetcodeReplay *r = ALLOC(&alloc, NetcodeReplay);
  r->mp = mp;
  r->desc = *d;
  r->desc.ring_seconds = d->ring_seconds > 0.0f ? d->ring_seconds : NRP_RING_SECONDS;
  r->tlsf = tlsf;
  r->alloc = alloc;
  r->live = live;
  r->r = rs;
  r->world = ecs_world_create();
  AppMemory *memory = ecs_resource_get_mut(live, AppMemory);
  if (memory)
    ecs_resource_set_ref(r->world, AppMemory, memory);
  ecs_resource_set_ref(r->world, AssetSystem, ecs_resource_get_mut(live, AssetSystem));
  ecs_renderer_attach(r->world, rs);
  ecs_transform_init(r->world, NULL);
  r->camera = ecs_entity_create(r->world);
  ecs_component_set(r->world, r->camera, Camera,
                    {.rot = quat_identity(), .fov = 60.0f, .near_plane = 0.05f, .far_plane = 1000.0f, .aspect = 16.0f / 9.0f});
  r->live_camera = ecs_query_cached(live, EcsIn(Camera));
  *slot = r;
  hz_command_register("netcode_replay_save", nrp_cmd_save, mp);
  const char *file = hz_arg_value("--netcode-replay");
  if (!file)
    return;
  NetcodeClip *clip = netcode_clip_load((CString)file, multiplayer_registry(mp), &r->alloc);
  if (!clip || !nrp_play(r, clip, &(NetcodeReplayStartDesc){.camera = NETCODE_REPLAY_CAMERA_FREE}))
    log_error("netcode replay: --netcode-replay % did not load", fmt_cstr(file));
  else
    log_info("netcode replay: playing % (% s)", fmt_cstr(file), fmt_f32_p(netcode_playback_duration(r->pb), 2));
}

// main lane, each frame: the netcode's client is recorded from the first frame it exists (a host's and a joiner's come with their ticket)
hz_internal void nrp_track(NetcodeReplay *r) {
  if (r->tracked)
    return;
  Netcode *net = multiplayer_net(r->mp);
  NetcodeClient *c = net ? netcode_client(net) : NULL;
  if (!c)
    return;
  r->tracked = true;
  r->rec = netcode_record_begin(c, &(NetcodeRecordDesc){.ring_seconds = r->desc.ring_seconds}, &r->alloc);
  if (!r->rec)
    log_warn("netcode replay: this client cannot be recorded");
  else
    log_info("netcode replay: recording the client, the last % s held", fmt_f32_p(r->desc.ring_seconds, 1));
}

void netcode_replay_reset(NetcodeReplay *r) {
  nrp_end(r);
  if (r->rec)
    netcode_record_end(r->rec);
  r->rec = NULL;
  r->tracked = false;
}

void netcode_replay_destroy(NetcodeReplay *r) {
  netcode_replay_reset(r);
  ecs_world_destroy(r->world);
  tlsf_allocator_destroy(r->tlsf);
}

b32 netcode_replay_start(Multiplayer *mp, const NetcodeReplayStartDesc *d) {
  assert_msg(is_main_thread(), "netcode_replay_start: main lane only"); // begins a playback outside scopes
  NetcodeReplay *r = nrp_get(mp);
  if (!r || !r->rec)
    return false;
  NetcodeClip *clip = netcode_clip_from_recorder(r->rec, d->seconds_back, &r->alloc);
  if (!clip)
    return false;
  nrp_end(r);
  return nrp_play(r, clip, d);
}

void netcode_replay_stop(Multiplayer *mp) {
  assert_msg(is_main_thread(), "netcode_replay_stop: main lane only"); // ends the playback outside scopes
  NetcodeReplay *r = nrp_get(mp);
  if (r)
    nrp_end(r);
}

b32 netcode_replay_active(const Multiplayer *mp) {
  const NetcodeReplay *r = nrp_get(mp);
  return r && r->active;
}

void netcode_replay_set_rate(Multiplayer *mp, f32 rate) {
  NetcodeReplay *r = nrp_get(mp);
  if (r && rate > 0.0f)
    r->rate = rate;
}

void netcode_replay_set_paused(Multiplayer *mp, b32 paused) {
  NetcodeReplay *r = nrp_get(mp);
  if (r)
    r->paused = paused;
}

void netcode_replay_seek(Multiplayer *mp, f32 seconds) {
  assert_msg(is_main_thread(), "netcode_replay_seek: main lane only"); // restores a keyframe outside scopes
  NetcodeReplay *r = nrp_get(mp);
  if (r && r->active)
    netcode_playback_seek(r->pb, seconds);
}

EcsWorld *netcode_replay_world(Multiplayer *mp) {
  NetcodeReplay *r = nrp_get(mp);
  return r ? r->world : NULL;
}

NetcodePlayback *netcode_replay_playback(Multiplayer *mp) {
  NetcodeReplay *r = nrp_get(mp);
  return r && r->active ? r->pb : NULL;
}

b32 netcode_replay_save(Multiplayer *mp, CString path, f32 seconds) {
  NetcodeReplay *r = nrp_get(mp);
  if (!r || !r->rec) {
    log_error("netcode replay: nothing is recorded to save to %", fmt_cstr(path));
    return false;
  }
  NetcodeClip *clip = netcode_clip_from_recorder(r->rec, seconds, &r->alloc);
  b32 ok = clip && netcode_clip_save(clip, path);
  if (clip)
    ALLOC_FREE(&r->alloc, clip);
  if (ok)
    log_info("netcode replay: saved %", fmt_cstr(path));
  else
    log_error("netcode replay: could not save % (no whole keyframe yet, or the file did not open)", fmt_cstr(path));
  return ok;
}

void netcode_replay_frame(Multiplayer *mp, f32 dt) {
  NetcodeReplay *r = nrp_get(mp);
  if (r && is_main_thread())
    nrp_track(r);
  if (!r || !r->active)
    return;
  f32 left = r->end - netcode_playback_position(r->pb);
  f32 step = r->paused ? 0.0f : MIN(r->rate * dt, MAX(left, 0.0f));
  netcode_playback_update(r->pb, step);
  if (is_main_thread())
    nrp_camera(r, dt);
  lane_sync();
  EcsWorld *w = r->world;
  ecs_scope(w) {
    if (r->desc.setup) {
      r->desc.setup(r->desc.user, w);
    } else {
      ecs_transform_schedule(w);
      ecs_renderer_schedule(w);
    }
    ecs_camera_schedule(w, &(EcsCameraScheduleDesc){.transform_sync_off = true}); // the block or the game placed it
    ecs_flush(w, r->paused ? 0.0f : r->rate * dt); // replay time: the rate's, 0 paused; held at the end, the presentation plays on
  }
}
