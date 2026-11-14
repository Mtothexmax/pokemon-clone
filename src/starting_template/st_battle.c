// The card battle (three.js src/lib/game/battle.svelte.ts): 3 zones out of 12, 6 turns, energy = turn, win 2 of 3
// zones. You plan in secret, then both sides reveal (whoever is ahead first), On Reveal / Activate / End of Turn
// abilities fire with short pauses (a step machine instead of the awaits), the AI plans like the three.js one.
// Included by starting_template.c above app_init.

#define B_MAX_CARDS 48
#define B_HAND_MAX 7
#define B_SIDE_P 0
#define B_SIDE_AI 1

typedef enum BattleKind { BATTLE_WILD, BATTLE_RIVAL, BATTLE_PRACTICE } BattleKind;
typedef enum BattlePhase { PHASE_PLAN, PHASE_REVEAL, PHASE_OVER } BattlePhase;
typedef enum BattleResult { RESULT_WIN, RESULT_LOSE, RESULT_DRAW, RESULT_RETREAT, RESULT_NONE } BattleResult;

typedef struct BCard {
  i32 id;
  i32 owner;
  i32 zone; // -1 hand / deck
  i32 mod;  // permanent power changes from abilities
  b32 revealed, staged, activated, alive;
  i32 order;     // play order this turn
  i32 act_order; // an Activate chosen this turn (-1 none)
  u32 hseq;      // when it entered the hand (the hand keeps this order)
  i32 fx_n;      // the last power change
  f32 fx_t;      // seconds since it (the UI shows a +N / -N)
  f32 pulse_t;   // seconds since it was revealed or activated
} BCard;

typedef enum StepKind { STEP_REVEAL, STEP_ACT } StepKind;
typedef struct BStep {
  StepKind kind;
  i32 card;
  i32 at;
} BStep;

typedef struct Battle {
  BattleKind kind;
  b32 shiny;
  CString opponent;
  BCard c[B_MAX_CARDS];
  i32 n;
  i32 zone[3];
  b32 zone_rev[3];
  i32 hand[2][B_HAND_MAX], hand_n[2];
  i32 deck[2][16], deck_n[2];
  i32 turn, max_turns, energy, energy_left;
  BattlePhase phase;
  BattleResult result;
  i32 stakes;
  b32 p_snapped, ai_snapped;
  i32 first;
  i32 play_counter;
  u32 hseq, seed;
  // the reveal, step by step
  BStep steps[40];
  i32 n_steps, step_i, reveal_repeat;
  i32 stage; // 0 reveals, 1 end of turn, 2 next turn
  i32 et_side, et_zone, et_idx;
  f32 wait;
  char log[6][120];
  i32 log_n;
  i32 sfx_hits; // power changes since the UI last looked (sounds)
} Battle;

hz_internal u32 b_rand_u(Battle *b) {
  b->seed = b->seed * 1664525u + 1013904223u;
  return b->seed >> 8;
}
hz_internal f32 b_rand(Battle *b) { return (f32)b_rand_u(b) / 16777216.0f; }

hz_internal void b_say(Battle *b, CString msg) {
  for (i32 i = 5; i > 0; i--)
    mem_cpy(b->log[i], b->log[i - 1], sizeof(b->log[0]));
  cstr_format(b->log[0], sizeof(b->log[0]), "%", fmt_cstr(msg));
  b->log_n = MIN(b->log_n + 1, 6);
}

#define B_SAY(b, ...)                                                                                                      \
  do {                                                                                                                     \
    char line_[120];                                                                                                       \
    cstr_format(line_, sizeof(line_), __VA_ARGS__);                                                                        \
    b_say((b), line_);                                                                                                     \
  } while (0)

hz_internal i32 b_other(i32 s) { return 1 - s; }
hz_internal CString b_name(const Battle *b, i32 ci) { return card_def(b->c[ci].id).name; }

// revealed cards of a side at a zone
hz_internal i32 b_cards_at(const Battle *b, i32 zone, i32 side, i32 *out) {
  i32 n = 0;
  if (zone < 0)
    return 0;
  for (i32 i = 0; i < b->n; i++) {
    const BCard *c = &b->c[i];
    if (c->alive && c->zone == zone && c->owner == side && c->revealed)
      out[n++] = i;
  }
  return n;
}
// every card on a side of a zone, staged ones too (for the capacity)
hz_internal i32 b_count_at(const Battle *b, i32 zone, i32 side) {
  i32 n = 0;
  for (i32 i = 0; i < b->n; i++)
    if (b->c[i].alive && b->c[i].zone == zone && b->c[i].owner == side)
      n++;
  return n;
}
hz_internal i32 b_capacity(const Battle *b, i32 zone) { return b->zone_rev[zone] && b->zone[zone] == ZONE_LEDGE ? 2 : 4; }

hz_internal i32 b_pick(Battle *b, const i32 *arr, i32 n) { return n ? arr[b_rand_u(b) % (u32)n] : -1; }

hz_internal void b_buff(Battle *b, i32 target, i32 n, i32 src) {
  b->c[target].mod += n;
  b->c[target].fx_n = n;
  b->c[target].fx_t = 0.0f;
  b->sfx_hits++;
  B_SAY(b, "%: % %% Power", fmt_cstr(b_name(b, src)), fmt_cstr(b_name(b, target)), fmt_cstr(n > 0 ? "+" : ""), fmt_i32(n));
}

hz_internal void b_destroy(Battle *b, i32 target, i32 src) {
  b->c[target].alive = false;
  b->sfx_hits++;
  B_SAY(b, "% destroyed %!", fmt_cstr(b_name(b, src)), fmt_cstr(b_name(b, target)));
}

hz_internal void b_draw(Battle *b, i32 side, b32 silent) {
  if (!b->deck_n[side] || b->hand_n[side] >= B_HAND_MAX)
    return;
  i32 ci = b->deck[side][0];
  for (i32 i = 1; i < b->deck_n[side]; i++)
    b->deck[side][i - 1] = b->deck[side][i];
  b->deck_n[side]--;
  b->c[ci].hseq = b->hseq++;
  b->hand[side][b->hand_n[side]++] = ci;
  if (!silent && side == B_SIDE_P)
    B_SAY(b, "You drew %.", fmt_cstr(b_name(b, ci)));
}

hz_internal i32 b_new_card(Battle *b, i32 id, i32 owner) {
  i32 ci = b->n++;
  b->c[ci] = (BCard){.id = id, .owner = owner, .zone = -1, .alive = true, .act_order = -1, .fx_t = 99.0f, .pulse_t = 99.0f};
  return ci;
}

hz_internal void b_add_to_hand(Battle *b, i32 side, i32 id) {
  if (b->hand_n[side] >= B_HAND_MAX || b->n >= B_MAX_CARDS)
    return;
  i32 ci = b_new_card(b, id, side);
  b->c[ci].hseq = b->hseq++;
  b->hand[side][b->hand_n[side]++] = ci;
  B_SAY(b, "% got a %.", fmt_cstr(side == B_SIDE_P ? "You" : b->opponent), fmt_cstr(card_def(id).name));
}

hz_internal void b_to_opening_hand(Battle *b, i32 ci) {
  i32 s = b->c[ci].owner;
  for (i32 i = 0; i < b->deck_n[s]; i++)
    if (b->deck[s][i] == ci) {
      for (i32 k = i; k > 0; k--)
        b->deck[s][k] = b->deck[s][k - 1];
      b->deck[s][0] = ci;
      return;
    }
}

hz_internal b32 b_doubles_reveal(i32 id) { return id == CARD_ECHOWL; }

// ---- power ----
hz_internal i32 b_power(const Battle *b, i32 ci);

hz_internal i32 b_self_bonus(const Battle *b, i32 ci) {
  const BCard *c = &b->c[ci];
  i32 tmp[B_MAX_CARDS];
  switch (c->id) {
  case CARD_ZEPHYRROW: return b_cards_at(b, c->zone, c->owner, tmp) == 1 ? 4 : 0;
  case CARD_BRAMBLEWOLF: return b_cards_at(b, c->zone, c->owner, tmp) - 1;
  case CARD_CRAGOLIATH: return b_cards_at(b, c->zone, c->owner, tmp) >= b_capacity(b, c->zone) ? 3 : 0;
  default: return 0;
  }
}

// the power card `src` (revealed, Ongoing) gives `t`
hz_internal i32 b_aura(const Battle *b, i32 src, i32 t) {
  const BCard *s = &b->c[src], *c = &b->c[t];
  switch (s->id) {
  case CARD_MOSSCUB: return c->owner == s->owner && c->zone == s->zone && t != src ? 1 : 0;
  case CARD_GLOOMOTH: return c->owner != s->owner && c->zone == s->zone ? -1 : 0;
  case CARD_CORALYTH: return c->owner == s->owner && c->zone != s->zone ? 1 : 0;
  case CARD_ABYSSQUID: return c->owner == s->owner && c->zone == s->zone && t != src ? 2 : 0;
  default: return 0;
  }
}

hz_internal i32 b_zone_bonus(const Battle *b, i32 zone, i32 ci) {
  CardDef d = card_def(b->c[ci].id);
  switch (b->zone[zone]) {
  case ZONE_POND: return d.kind == KIND_ONGOING ? 2 : 0;
  case ZONE_GROVE: return d.kind == KIND_NONE ? 2 : 0;
  case ZONE_BARN: return -1;
  case ZONE_GYM: return 1;
  case ZONE_TOYBOX: return d.cost <= 2 ? 2 : 0;
  case ZONE_WORKSHOP: return d.kind == KIND_ACTIVATE ? 2 : 0;
  default: return 0;
  }
}

hz_internal i32 b_power(const Battle *b, i32 ci) {
  const BCard *c = &b->c[ci];
  i32 p = card_def(c->id).power + c->mod;
  if (c->zone < 0 || !c->revealed)
    return p;
  p += b_self_bonus(b, ci);
  if (b->zone_rev[c->zone])
    p += b_zone_bonus(b, c->zone, ci);
  for (i32 i = 0; i < b->n; i++)
    if (b->c[i].alive && b->c[i].revealed && b->c[i].zone >= 0)
      p += b_aura(b, i, ci);
  return p;
}

hz_internal i32 b_zone_power(const Battle *b, i32 zone, i32 side) {
  i32 tmp[B_MAX_CARDS], n = b_cards_at(b, zone, side, tmp), s = 0;
  for (i32 i = 0; i < n; i++)
    s += b_power(b, tmp[i]);
  return s;
}
hz_internal i32 b_zone_winner(const Battle *b, i32 zone) {
  i32 a = b_zone_power(b, zone, B_SIDE_P), d = b_zone_power(b, zone, B_SIDE_AI);
  return a > d ? B_SIDE_P : d > a ? B_SIDE_AI : -1;
}
hz_internal i32 b_winning_zones(const Battle *b, i32 side) {
  i32 n = 0;
  for (i32 z = 0; z < 3; z++)
    n += b_zone_winner(b, z) == side ? 1 : 0;
  return n;
}

// ---- abilities ----
hz_internal i32 b_enemies_here(const Battle *b, i32 ci, i32 *out) { return b_cards_at(b, b->c[ci].zone, b_other(b->c[ci].owner), out); }
hz_internal i32 b_friends_here(const Battle *b, i32 ci, i32 *out) {
  i32 tmp[B_MAX_CARDS], n = b_cards_at(b, b->c[ci].zone, b->c[ci].owner, tmp), k = 0;
  for (i32 i = 0; i < n; i++)
    if (tmp[i] != ci)
      out[k++] = tmp[i];
  return k;
}

hz_internal void b_on_reveal(Battle *b, i32 ci) {
  i32 arr[B_MAX_CARDS], n;
  BCard *c = &b->c[ci];
  switch (c->id) {
  case CARD_FLICKER: n = b_friends_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], 1, ci); break;
  case CARD_DRIZZLET: b_draw(b, c->owner, false); break;
  case CARD_ZAPLING: { i32 t = b_pick(b, arr, b_enemies_here(b, ci, arr)); if (t >= 0) b_buff(b, t, -1, ci); } break;
  case CARD_VOLTAIL: { i32 t = b_pick(b, arr, b_enemies_here(b, ci, arr)); if (t >= 0) b_buff(b, t, -2, ci); } break;
  case CARD_CINDERPUP: if (b_enemies_here(b, ci, arr)) b_buff(b, ci, 3, ci); break;
  case CARD_TIDALOTL:
    for (i32 i = 0; i < b->n; i++)
      if (b->c[i].alive && b->c[i].revealed && b->c[i].owner == c->owner && b->c[i].zone >= 0 && b->c[i].zone != c->zone)
        b_buff(b, i, 1, ci);
    break;
  case CARD_FUNGOO: n = b_enemies_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], -1, ci); break;
  case CARD_GLIMMERLING: b_add_to_hand(b, c->owner, CARD_BRICKLING); break;
  case CARD_THUNDRAKE: {
    n = b_enemies_here(b, ci, arr);
    i32 t = -1;
    for (i32 i = 0; i < n; i++)
      if (t < 0 || b_power(b, arr[i]) > b_power(b, t))
        t = arr[i];
    if (t >= 0)
      b_buff(b, t, -3, ci);
  } break;
  case CARD_EMBERHORN: n = b_friends_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], -1, ci); break;
  case CARD_PHANTOMULE: {
    n = b_enemies_here(b, ci, arr);
    i32 t = -1;
    for (i32 i = 0; i < n; i++)
      if (t < 0 || b_power(b, arr[i]) < b_power(b, t))
        t = arr[i];
    if (t >= 0)
      b_destroy(b, t, ci);
  } break;
  case CARD_GLACIOMOTH: n = b_enemies_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], -2, ci); break;
  case CARD_LUMIFOX: n = b_friends_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], 2, ci); break;
  case CARD_STORMAW:
    for (i32 i = 0; i < b->n; i++)
      if (b->c[i].alive && b->c[i].revealed && b->c[i].zone >= 0 && b->c[i].owner != c->owner)
        b_buff(b, i, -1, ci);
    break;
  default: break;
  }
}

hz_internal void b_activate_fx(Battle *b, i32 ci) {
  i32 arr[B_MAX_CARDS], n;
  switch (b->c[ci].id) {
  case CARD_GUSTLING: n = b_friends_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], 1, ci); break;
  case CARD_SPARKMOLE: b_draw(b, b->c[ci].owner, false); break;
  case CARD_BOOMSHROOM: n = b_enemies_here(b, ci, arr); for (i32 i = 0; i < n; i++) b_buff(b, arr[i], -2, ci); break;
  default: break;
  }
}

hz_internal void b_end_turn_fx(Battle *b, i32 ci) {
  i32 arr[B_MAX_CARDS];
  switch (b->c[ci].id) {
  case CARD_SPROUTLE: b_buff(b, ci, 1, ci); break;
  case CARD_TICKBOT: { i32 t = b_pick(b, arr, b_friends_here(b, ci, arr)); if (t >= 0) b_buff(b, t, 1, ci); } break;
  case CARD_CLOCKCRAB: b_buff(b, ci, 2, ci); break;
  default: break;
  }
}

hz_internal b32 b_has_end_turn(i32 id) { return id == CARD_SPROUTLE || id == CARD_TICKBOT || id == CARD_CLOCKCRAB; }
hz_internal b32 b_has_activate(i32 id) { return id == CARD_GUSTLING || id == CARD_SPARKMOLE || id == CARD_BOOMSHROOM; }

hz_internal i32 b_reveal_times(const Battle *b, i32 ci) {
  const BCard *c = &b->c[ci];
  b32 open = b->zone_rev[c->zone];
  if (open && b->zone[c->zone] == ZONE_LIBRARY)
    return 0;
  i32 n = 1;
  if (open && b->zone[c->zone] == ZONE_STAGE)
    n++;
  i32 arr[B_MAX_CARDS], k = b_cards_at(b, c->zone, c->owner, arr);
  for (i32 i = 0; i < k; i++)
    if (arr[i] != ci && b_doubles_reveal(b->c[arr[i]].id))
      n++;
  return n;
}

hz_internal b32 b_can_activate(const Battle *b, i32 ci) {
  const BCard *c = &b->c[ci];
  return b_has_activate(c->id) && c->alive && c->revealed && !c->activated && c->act_order < 0 && c->zone >= 0 &&
         (c->owner != B_SIDE_P || b->phase == PHASE_PLAN);
}

hz_internal b32 b_activate(Battle *b, i32 ci) {
  if (!b_can_activate(b, ci))
    return false;
  b->c[ci].act_order = b->play_counter++;
  b->c[ci].pulse_t = 0.0f;
  return true;
}

hz_internal void b_cancel_activate(Battle *b, i32 ci) {
  if (b->phase == PHASE_PLAN && b->c[ci].act_order >= 0)
    b->c[ci].act_order = -1;
}

hz_internal b32 b_zone_open(const Battle *b, i32 zone, i32 side) {
  if (b_count_at(b, zone, side) >= b_capacity(b, zone))
    return false;
  if (b->zone_rev[zone] && b->zone[zone] == ZONE_TOWER && b->turn >= b->max_turns)
    return false;
  return true;
}

hz_internal b32 b_can_play(const Battle *b, i32 ci, i32 zone) {
  return b->phase == PHASE_PLAN && card_def(b->c[ci].id).cost <= b->energy_left && b_zone_open(b, zone, B_SIDE_P);
}

hz_internal void b_remove_from_hand(Battle *b, i32 side, i32 ci) {
  i32 k = 0;
  for (i32 i = 0; i < b->hand_n[side]; i++)
    if (b->hand[side][i] != ci)
      b->hand[side][k++] = b->hand[side][i];
  b->hand_n[side] = k;
}

hz_internal b32 b_stage(Battle *b, i32 ci, i32 zone) {
  if (!b_can_play(b, ci, zone))
    return false;
  b_remove_from_hand(b, B_SIDE_P, ci);
  BCard *c = &b->c[ci];
  c->zone = zone;
  c->staged = true;
  c->order = b->play_counter++;
  b->energy_left -= card_def(c->id).cost;
  return true;
}

hz_internal void b_unstage(Battle *b, i32 ci) {
  BCard *c = &b->c[ci];
  if (b->phase != PHASE_PLAN || !c->staged)
    return;
  b->energy_left += card_def(c->id).cost;
  c->zone = -1;
  c->staged = false;
  // back into its old slot
  i32 at = b->hand_n[B_SIDE_P];
  for (i32 i = 0; i < b->hand_n[B_SIDE_P]; i++)
    if (b->c[b->hand[B_SIDE_P][i]].hseq > c->hseq) {
      at = i;
      break;
    }
  for (i32 i = b->hand_n[B_SIDE_P]; i > at; i--)
    b->hand[B_SIDE_P][i] = b->hand[B_SIDE_P][i - 1];
  b->hand[B_SIDE_P][at] = ci;
  b->hand_n[B_SIDE_P]++;
}

hz_internal void b_snap(Battle *b) {
  if (b->p_snapped || b->phase == PHASE_OVER)
    return;
  b->p_snapped = true;
  b->stakes *= 2;
  b_say(b, "You SNAPPED! Stakes doubled.");
}

hz_internal void b_retreat(Battle *b) {
  if (b->phase == PHASE_OVER)
    return;
  b->phase = PHASE_OVER;
  b->result = RESULT_RETREAT;
  b_say(b, "You retreated.");
}

hz_internal void b_ai_snap(Battle *b) {
  if (b->ai_snapped || b->turn < 3 || b->turn > 5)
    return;
  i32 lead = 0, strong = 0;
  for (i32 z = 0; z < 3; z++)
    lead += b_zone_power(b, z, B_SIDE_AI) - b_zone_power(b, z, B_SIDE_P);
  for (i32 i = 0; i < b->hand_n[B_SIDE_AI]; i++)
    strong += card_def(b->c[b->hand[B_SIDE_AI][i]].id).power >= 5 ? 1 : 0;
  f32 score = (f32)lead / 4.0f + (f32)strong * 0.5f + (f32)b_winning_zones(b, B_SIDE_AI) - 1.0f;
  if (score > 1.8f && b_rand(b) < 0.6f) {
    b->ai_snapped = true;
    b->stakes *= 2;
    B_SAY(b, "% SNAPPED! Stakes doubled.", fmt_cstr(b->opponent));
  }
}

hz_internal void b_next_turn(Battle *b) {
  b->turn++;
  if (b->turn <= 3) {
    b->zone_rev[b->turn - 1] = true;
    B_SAY(b, "Zone revealed: %", fmt_cstr(zone_def(b->zone[b->turn - 1]).name));
  }
  b->energy = b->energy_left = b->turn;
  b_draw(b, B_SIDE_P, false);
  b_draw(b, B_SIDE_AI, false);
  if (b->turn == b->max_turns) {
    b->stakes *= 2;
    b_say(b, "Final turn! Stakes doubled.");
  }
  b->phase = PHASE_PLAN;
  b_ai_snap(b);
}

hz_internal void b_shuffle(Battle *b, i32 *a, i32 n) {
  for (i32 i = n - 1; i > 0; i--) {
    i32 j = (i32)(b_rand_u(b) % (u32)(i + 1));
    i32 t = a[i];
    a[i] = a[j];
    a[j] = t;
  }
}

hz_internal void b_init(Battle *b, BattleKind kind, CString opponent, const u8 *player_deck, const i32 *ai_deck, b32 shiny, u32 seed) {
  *b = (Battle){.kind = kind, .shiny = shiny, .opponent = opponent, .max_turns = 6, .stakes = 1, .seed = seed | 1u, .hseq = 1,
                .result = RESULT_NONE};
  for (i32 s = 0; s < 2; s++) {
    for (i32 i = 0; i < DECK_SIZE; i++) {
      i32 id = s == B_SIDE_P ? (i32)player_deck[i] : ai_deck[i];
      b->deck[s][b->deck_n[s]++] = b_new_card(b, id, s);
    }
    b_shuffle(b, b->deck[s], b->deck_n[s]);
  }
  i32 zones[ZONE_COUNT];
  for (i32 i = 0; i < ZONE_COUNT; i++)
    zones[i] = i;
  b_shuffle(b, zones, ZONE_COUNT);
  for (i32 z = 0; z < 3; z++)
    b->zone[z] = zones[z];
  // Game Start abilities (the cards are still in the decks)
  for (i32 s = 0; s < 2; s++)
    for (i32 i = 0; i < b->deck_n[s]; i++) {
      i32 ci = b->deck[s][i];
      i32 id = b->c[ci].id;
      if (id == CARD_QUICKHOP || id == CARD_GOLDBRICK)
        b_to_opening_hand(b, ci);
      if (id == CARD_BUILDBEAVER || id == CARD_GOLDBRICK)
        b->c[ci].mod += 2;
      if (card_def(id).kind == KIND_START && s == B_SIDE_P)
        B_SAY(b, "Game Start: %", fmt_cstr(card_def(id).name));
    }
  for (i32 i = 0; i < 3; i++) {
    b_draw(b, B_SIDE_P, true);
    b_draw(b, B_SIDE_AI, true);
  }
  b_next_turn(b);
}

// the AI: the subset of its hand that spends the most energy (then power), each card to the zone that needs it
hz_internal i32 b_ai_plan(Battle *b, i32 *placed) {
  i32 n = MIN(b->hand_n[B_SIDE_AI], 7), best_mask = 0;
  f32 best_score = -1.0f;
  for (i32 mask = 1; mask < (1 << n); mask++) {
    i32 cost = 0, pow = 0;
    for (i32 i = 0; i < n; i++)
      if (mask & (1 << i)) {
        CardDef d = card_def(b->c[b->hand[B_SIDE_AI][i]].id);
        cost += d.cost;
        pow += d.power;
      }
    if (cost > b->energy)
      continue;
    f32 score = (f32)(cost * 10 + pow) + b_rand(b);
    if (score > best_score) {
      best_score = score;
      best_mask = mask;
    }
  }
  i32 set[B_HAND_MAX], ns = 0;
  for (i32 i = 0; i < n; i++)
    if (best_mask & (1 << i))
      set[ns++] = b->hand[B_SIDE_AI][i];
  for (i32 i = 0; i < ns; i++) // most expensive first
    for (i32 k = i + 1; k < ns; k++)
      if (card_def(b->c[set[k]].id).cost > card_def(b->c[set[i]].id).cost) {
        i32 t = set[i];
        set[i] = set[k];
        set[k] = t;
      }
  i32 pending[3] = {0, 0, 0}, counts[3], np = 0;
  for (i32 z = 0; z < 3; z++)
    counts[z] = b_count_at(b, z, B_SIDE_AI);
  for (i32 i = 0; i < ns; i++) {
    i32 ci = set[i];
    CardDef d = card_def(b->c[ci].id);
    i32 best_z = -1;
    f32 best_s = -1e9f;
    for (i32 z = 0; z < 3; z++) {
      if (counts[z] >= b_capacity(b, z))
        continue;
      if (b->zone_rev[z] && b->zone[z] == ZONE_TOWER && b->turn >= b->max_turns)
        continue;
      i32 diff = b_zone_power(b, z, B_SIDE_AI) + pending[z] - b_zone_power(b, z, B_SIDE_P);
      f32 s = -m_absf((f32)(diff + d.power - 3));
      if (diff > 10)
        s -= 6.0f;
      if (diff < -14)
        s -= 6.0f;
      if (b->zone_rev[z]) {
        b->c[ci].zone = z;
        b->c[ci].revealed = true;
        s += (f32)b_zone_bonus(b, z, ci) * 2.0f;
        b->c[ci].zone = -1;
        b->c[ci].revealed = false;
      }
      if (b->c[ci].id == CARD_ZEPHYRROW)
        s += counts[z] == 0 ? 6.0f : -6.0f;
      if (d.kind == KIND_REVEAL && b->zone_rev[z] && b->zone[z] == ZONE_LIBRARY)
        s -= 4.0f;
      i32 id = b->c[ci].id;
      if (id == CARD_VOLTAIL || id == CARD_THUNDRAKE || id == CARD_PHANTOMULE || id == CARD_CINDERPUP || id == CARD_GLACIOMOTH) {
        i32 tmp[B_MAX_CARDS];
        if (b_cards_at(b, z, B_SIDE_P, tmp))
          s += 4.0f;
      }
      s += b_rand(b) * 2.0f;
      if (s > best_s) {
        best_s = s;
        best_z = z;
      }
    }
    if (best_z < 0)
      continue;
    counts[best_z]++;
    pending[best_z] += d.power;
    b_remove_from_hand(b, B_SIDE_AI, ci);
    b->c[ci].zone = best_z;
    b->c[ci].staged = true;
    b->c[ci].order = b->play_counter++;
    placed[np++] = ci;
  }
  return np;
}

// End Turn: the AI chooses its Activates and plays, then everything is revealed step by step (b_update)
hz_internal void b_end_turn(Battle *b) {
  if (b->phase != PHASE_PLAN)
    return;
  b->phase = PHASE_REVEAL;
  for (i32 i = 0; i < b->n; i++)
    if (b->c[i].owner == B_SIDE_AI && b_can_activate(b, i))
      b_activate(b, i);
  i32 ai_plays[B_HAND_MAX];
  i32 nai = b_ai_plan(b, ai_plays);
  i32 pw = b_winning_zones(b, B_SIDE_P), aw = b_winning_zones(b, B_SIDE_AI);
  b->first = pw > aw ? B_SIDE_P : aw > pw ? B_SIDE_AI : (b_rand(b) < 0.5f ? B_SIDE_P : B_SIDE_AI);
  // each side: its plays and Activates in the order they were done; the first side first
  b->n_steps = 0;
  for (i32 pass = 0; pass < 2; pass++) {
    i32 side = pass == 0 ? b->first : b_other(b->first);
    i32 start = b->n_steps;
    for (i32 i = 0; i < b->n; i++) {
      BCard *c = &b->c[i];
      if (c->owner != side || !c->alive)
        continue;
      if (c->staged && c->zone >= 0)
        b->steps[b->n_steps++] = (BStep){STEP_REVEAL, i, c->order};
      if (c->act_order >= 0)
        b->steps[b->n_steps++] = (BStep){STEP_ACT, i, c->act_order};
    }
    for (i32 i = start; i < b->n_steps; i++)
      for (i32 k = i + 1; k < b->n_steps; k++)
        if (b->steps[k].at < b->steps[i].at) {
          BStep t = b->steps[i];
          b->steps[i] = b->steps[k];
          b->steps[k] = t;
        }
  }
  (void)nai;
  b->step_i = 0;
  b->reveal_repeat = 0;
  b->stage = 0;
  b->wait = 0.35f;
}

hz_internal void b_finish(Battle *b) {
  b->phase = PHASE_OVER;
  i32 pw = b_winning_zones(b, B_SIDE_P), aw = b_winning_zones(b, B_SIDE_AI);
  if (pw != aw)
    b->result = pw > aw ? RESULT_WIN : RESULT_LOSE;
  else {
    i32 tp = 0, ta = 0;
    for (i32 z = 0; z < 3; z++) {
      tp += b_zone_power(b, z, B_SIDE_P);
      ta += b_zone_power(b, z, B_SIDE_AI);
    }
    b->result = tp > ta ? RESULT_WIN : ta > tp ? RESULT_LOSE : RESULT_DRAW;
  }
  b_say(b, b->result == RESULT_WIN ? "Victory!" : b->result == RESULT_LOSE ? "Defeat..." : "Draw.");
}

// every frame: the reveal's next step once its pause is over
hz_internal void b_update(Battle *b, f32 dt) {
  for (i32 i = 0; i < b->n; i++) {
    b->c[i].fx_t += dt;
    b->c[i].pulse_t += dt;
  }
  if (b->phase != PHASE_REVEAL)
    return;
  b->wait -= dt;
  if (b->wait > 0.0f)
    return;
  if (b->stage == 0) {
    if (b->step_i >= b->n_steps) {
      b->stage = 1;
      b->et_side = 0;
      b->et_zone = 0;
      b->et_idx = 0;
      for (i32 i = 0; i < b->n; i++)
        b->c[i].staged = false;
      return;
    }
    BStep st = b->steps[b->step_i];
    BCard *c = &b->c[st.card];
    if (st.kind == STEP_ACT) {
      c->act_order = -1;
      b->step_i++;
      if (!c->activated && c->alive && c->zone >= 0) {
        c->activated = true;
        c->pulse_t = 0.0f;
        B_SAY(b, "% activated %.", fmt_cstr(c->owner == B_SIDE_P ? "You" : b->opponent), fmt_cstr(b_name(b, st.card)));
        b_activate_fx(b, st.card);
        b->wait = 0.65f;
      }
      return;
    }
    if (b->reveal_repeat == 0) {
      if (!c->alive || c->zone < 0) { // destroyed meanwhile
        b->step_i++;
        return;
      }
      c->staged = false;
      c->revealed = true;
      c->pulse_t = 0.0f;
    }
    i32 times = card_def(c->id).kind == KIND_REVEAL ? b_reveal_times(b, st.card) : 0;
    if (b->reveal_repeat < times) {
      b_on_reveal(b, st.card);
      b->reveal_repeat++;
      b->wait = b->reveal_repeat < times ? 0.3f : 0.65f;
      if (b->reveal_repeat >= times) {
        b->reveal_repeat = 0;
        b->step_i++;
      }
      return;
    }
    b->reveal_repeat = 0;
    b->step_i++;
    b->wait = 0.65f;
    return;
  }
  if (b->stage == 1) { // End of Turn: the first side, then the other, zone by zone
    while (b->et_side < 2) {
      i32 side = b->et_side == 0 ? b->first : b_other(b->first);
      i32 arr[B_MAX_CARDS], n = b_cards_at(b, b->et_zone, side, arr);
      while (b->et_idx < n) {
        i32 ci = arr[b->et_idx++];
        if (!b_has_end_turn(b->c[ci].id))
          continue;
        i32 times = b->zone_rev[b->et_zone] && b->zone[b->et_zone] == ZONE_CLOCK ? 2 : 1;
        for (i32 k = 0; k < times; k++)
          b_end_turn_fx(b, ci);
        b->wait = 0.4f;
        return;
      }
      b->et_idx = 0;
      if (++b->et_zone >= 3) {
        b->et_zone = 0;
        b->et_side++;
      }
    }
    b->stage = 2;
  }
  if (b->turn >= b->max_turns)
    b_finish(b);
  else
    b_next_turn(b);
}
