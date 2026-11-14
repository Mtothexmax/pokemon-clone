// A headless run: a bot plays through the player's own controls at a fixed dt; copy this shape to test a gameplay change.

#include "tests/test_framework.h"
#include "gameplay/sim_run/sim_run.h"
#include "lib/hash.h"
#include "lib/hzmath.h"
#include "lib/thread_context.h"
#include "starting_template/game_sim.h"

#define ST_ROUTE_POINTS 2
#define ST_ARRIVE_M 0.5f

// the bot's route: walk to each point in turn
typedef struct StBot {
  GameSim *sim;
  u32 next;
  v3 route[ST_ROUTE_POINTS];
} StBot;

hz_internal void *st_begin(SimRun *run, u64 seed, void *user) {
  UNUSED(seed);
  UNUSED(user);
  StBot *bot = ALLOC(run->alloc, StBot);
  bot->sim = game_sim_create();
  bot->route[0] = (v3){8.0f, 0.0f, 0.0f};
  bot->route[1] = (v3){8.0f, 0.0f, 8.0f};
  return bot;
}

hz_internal b32 st_ready(void *game, void *user) {
  UNUSED(user);
  return game_sim_ready(((StBot *)game)->sim);
}

hz_internal SimRunStatus st_step(SimRun *run, void *game, f32 dt, void *user) {
  UNUSED(user);
  StBot *bot = game;
  local_shared PlayerControls controls;
  local_shared SimRunStatus status;
  if (is_main_thread()) {
    controls = (PlayerControls){0};
    status = SIM_RUN_CONTINUE;
    v3 feet = game_sim_player_position(bot->sim);
    if (dt > 0.0f && bot->next < ST_ROUTE_POINTS) {
      v3 to = v3_sub(bot->route[bot->next], feet);
      to.y = 0.0f;
      if (v3_length(to) < ST_ARRIVE_M) {
        tctx_temp_allocator_scope(ta, NULL) { sim_run_beat(run, str_format(&ta.allocator, "reached:%", fmt_u32(bot->next))); }
        bot->next++;
      } else {
        // heading 0 faces -z; the move is relative to it
        controls = (PlayerControls){.move = {0.0f, 1.0f}, .heading = m_atan2f(-to.x, -to.z)};
      }
    }
    status = bot->next == ST_ROUTE_POINTS ? SIM_RUN_DONE : SIM_RUN_CONTINUE;
  }
  lane_sync(); // every lane steps with the controls the main lane chose
  game_sim_frame(bot->sim, &controls, dt);
  return status;
}

hz_internal void st_end(SimRun *run, void *game, void *user) {
  UNUSED(run);
  UNUSED(user);
  game_sim_destroy(((StBot *)game)->sim);
}

hz_internal u64 st_digest(void *game, void *user) {
  UNUSED(user);
  v3 p = game_sim_player_position(((StBot *)game)->sim);
  return fnv1a_hash_64_n((const char *)&p, (u32)sizeof(p));
}

// the bot walks the route at the design rate; both seeds arrive, the same way (no randomness in this game yet)
void test_st_sim_walks_the_route(void) {
  local_shared ArenaAllocator *arena;
  local_shared Allocator alloc;
  local_shared SimRun run;
  local_shared SimRunDesc desc;
  local_shared u64 *seeds;
  local_shared u32 finished;
  local_shared u32 arrived;
  local_shared b32 same;
  if (is_main_thread()) {
    arena = arena_create(MB(16), MB(1));
    alloc = make_arena_allocator(arena);
    seeds = ALLOC_ARRAY(&alloc, u64, 2);
    seeds[0] = 1;
    seeds[1] = 2;
    desc = (SimRunDesc){.name = "st sim route",
                        .dt = 1.0f / 60.0f,
                        .max_seconds = 60.0f,
                        .seeds = seeds,
                        .seed_count = 2,
                        .begin = st_begin,
                        .ready = st_ready,
                        .step = st_step,
                        .end = st_end,
                        .digest = st_digest};
  }
  lane_sync();
  SimRunReportBlobAsset *r = sim_run(&run, &desc, &alloc);
  if (is_main_thread()) {
    sim_run_log(r);
    Array(SimRunSeedStat) st = blob_array_get(SimRunSeedStat, &r->seeds);
    finished = 0;
    arr_foreach_ptr(st, SimRunSeedStat, s) finished += s->finished ? 1u : 0u;
    const SimRunBeatStat *last = sim_run_find_beat(r, STR_FROM_CSTR("reached:1"));
    arrived = last ? last->reached : 0;
    same = st.items[0].digest_trail == st.items[1].digest_trail;
  }
  lane_sync();
  if (is_main_thread()) {
    arena_destroy(arena);
    test_assert_eq(finished, 2u);
    test_assert_eq(arrived, 2u);
    test_assert_true(same);
  }
}
