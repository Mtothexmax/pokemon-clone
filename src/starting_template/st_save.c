// The save game: one small struct in the platform store (a file natively, IndexedDB in the browser) under
// "brickhollow.save". Loaded once at start (asynchronously: the game starts, the save lands a few frames later),
// written whenever something changed. Included by starting_template.c above app_init.

#define SAVE_KEY "brickhollow.save"
#define SAVE_VERSION 1u

typedef struct GameSave {
  u32 version;
  u32 golden_mask; // GOLDEN_COUNT bits
  u32 studs;
  u32 duck_feeds;
  u32 chests;
  u32 fireflies;
  u32 goals;
  u32 seek_wins;
  b32 legendary; // all golden studs found
  f32 time_of_day; // 0..1, saved so the day goes on where you left it
  u32 _reserved[16];
  // ---- cards and Voxelmon (appended: older saves load with these zero) ----
  b32 has_deck;
  u64 collection;    // a bit per card (CardId)
  u8 deck[12];       // the battle deck, CardIds
  u8 deck_count;
  u8 card_level[64]; // upgrade stage per CardId (0 basic .. 4 Voxelmon)
  u32 exp;
  u32 wins, losses, rival_wins;
  i32 voxelmon;      // the companion: CardId + 1, 0 none
  b32 gate_open;
  u32 vox_wins;
  b32 met_rival;
  // ---- the menu (appended) ----
  u8 gfx;        // GfxChoice: 0 Auto, 1 High, 2 Medium, 3 Low
  u8 voices_off, music_off;
  u8 auto_step;  // the step Auto reached + 1 (0: not chosen yet)
  u8 touch_mode; // phones: 0 Swipe (the default), 1 D-pad (the stick)
  // ---- the first deck's quest and the pond cave (appended) ----
  u8 quest_part; // Prof. Quill told you the Deck-o-Matic misses its Stud Core
  u8 has_suit;   // you took her old diving suit (in a corner of the lab)
  u8 has_part;   // you found the Stud Core deep in the pond cave
  u8 _quest_pad;
  u32 dive_found; // the pearls found in the cave (a bit each)
} GameSave;

typedef struct SaveState {
  GameSave data;
  b32 loaded;     // the read finished (found or not)
  b32 dirty;      // changed since the last write
  f32 write_wait; // seconds until a dirty save is written (coalesces bursts)
  OsStoreOp *op;
  b32 op_is_read;
  b32 no_write; // dev builds (F3): a trial of the first quest that is never written
} SaveState;

hz_internal void save_begin(SaveState *s) {
  s->data = (GameSave){.version = SAVE_VERSION, .time_of_day = 0.32f};
  s->op = os_store_read(SAVE_KEY);
  s->op_is_read = true;
}

hz_internal void save_mark(SaveState *s) {
  s->dirty = true;
  s->write_wait = 0.5f;
}

// main lane, every frame: finishes the read, writes a dirty save
hz_internal void save_update(SaveState *s, f32 dt) {
  if (s->op) {
    OsStoreState st = os_store_check(s->op);
    if (st == OS_STORE_PENDING)
      return;
    if (s->op_is_read) {
      const u8 *bytes = NULL;
      u32 len = os_store_value(s->op, &bytes);
      if (st == OS_STORE_DONE && bytes && len >= sizeof(u32)) {
        GameSave g = {0};
        mem_cpy(&g, bytes, MIN(len, (u32)sizeof(GameSave)));
        if (g.version == SAVE_VERSION)
          s->data = g;
      }
      s->loaded = true;
      log_info("save: % (golden % / studs %)", fmt_cstr(st == OS_STORE_DONE ? "loaded" : "new game"),
               fmt_u32(s->data.golden_mask), fmt_u32(s->data.studs));
    }
    os_store_release(s->op);
    s->op = NULL;
  }
  if (!s->loaded || !s->dirty || s->no_write)
    return;
  s->write_wait -= dt;
  if (s->write_wait > 0.0f)
    return;
  s->dirty = false;
  s->op = os_store_write(SAVE_KEY, (const u8 *)&s->data, (u32)sizeof(GameSave));
  s->op_is_read = false;
}
