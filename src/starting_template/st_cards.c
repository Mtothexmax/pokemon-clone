// The Brick Beast cards (three.js src/lib/game/cards.ts, levels.ts): the card table, the zones, the decks, the upgrade
// stages, the collection in the save, and each card's picture as a texture (the Voxelmon's front pixel art,
// st_beasts_data.c). Included by starting_template.c above app_init.

#include "st_beasts_data.c"

typedef enum CardKind { KIND_NONE, KIND_REVEAL, KIND_ONGOING, KIND_ACTIVATE, KIND_ENDTURN, KIND_START } CardKind;

typedef enum CardId {
  CARD_PEBBIT, CARD_FLICKER, CARD_DRIZZLET, CARD_SPROUTLE, CARD_ZAPLING, CARD_QUICKHOP,
  CARD_VOLTAIL, CARD_CINDERPUP, CARD_MOSSCUB, CARD_GLOOMOTH, CARD_GUSTLING, CARD_SPARKMOLE, CARD_TICKBOT,
  CARD_TIDALOTL, CARD_ROCKODILE, CARD_FUNGOO, CARD_ZEPHYRROW, CARD_GLIMMERLING, CARD_ECHOWL, CARD_BUILDBEAVER,
  CARD_THUNDRAKE, CARD_CORALYTH, CARD_EMBERHORN, CARD_BRAMBLEWOLF, CARD_PHANTOMULE, CARD_BOOMSHROOM,
  CARD_GLACIOMOTH, CARD_MAGMAW, CARD_LUMIFOX, CARD_CRAGOLIATH, CARD_CLOCKCRAB,
  CARD_TITANOAK, CARD_STORMAW, CARD_ABYSSQUID,
  CARD_GOLDBRICK, // special reward: all golden studs
  CARD_BRICKLING, // token
  CARD_COUNT
} CardId;

#define CARD_POOL 34 // the cards you can collect (not the special reward, not the token)
#define DECK_SIZE 12

typedef struct CardDef {
  CString id, name;
  CardKind kind;
  i32 cost, power;
  CString text;
} CardDef;

hz_internal CardDef card_def(i32 c) {
  switch (c) {
  case CARD_PEBBIT: return (CardDef){"pebbit", "Pebbit", KIND_NONE, 1, 2, "A sturdy little pebble bunny."};
  case CARD_FLICKER: return (CardDef){"flicker", "Flicker", KIND_REVEAL, 1, 1, "On Reveal: Your other cards here get +1 Power."};
  case CARD_DRIZZLET: return (CardDef){"drizzlet", "Drizzlet", KIND_REVEAL, 1, 1, "On Reveal: Draw a card."};
  case CARD_SPROUTLE: return (CardDef){"sproutle", "Sproutle", KIND_ENDTURN, 1, 1, "End of Turn: This card gets +1 Power."};
  case CARD_ZAPLING: return (CardDef){"zapling", "Zapling", KIND_REVEAL, 1, 2, "On Reveal: A random enemy card here gets -1 Power."};
  case CARD_QUICKHOP: return (CardDef){"quickhop", "Quickhop", KIND_START, 1, 2, "Game Start: Put this card into your opening hand."};
  case CARD_VOLTAIL: return (CardDef){"voltail", "Voltail", KIND_REVEAL, 2, 2, "On Reveal: A random enemy card here gets -2 Power."};
  case CARD_CINDERPUP: return (CardDef){"cinderpup", "Cinderpup", KIND_REVEAL, 2, 2, "On Reveal: If an enemy card is here, +3 Power."};
  case CARD_MOSSCUB: return (CardDef){"mosscub", "Mosscub", KIND_ONGOING, 2, 1, "Ongoing: Your other cards here have +1 Power."};
  case CARD_GLOOMOTH: return (CardDef){"gloomoth", "Gloomoth", KIND_ONGOING, 2, 2, "Ongoing: Enemy cards here have -1 Power."};
  case CARD_GUSTLING: return (CardDef){"gustling", "Gustling", KIND_ACTIVATE, 2, 2, "Activate: Your other cards here get +1 Power."};
  case CARD_SPARKMOLE: return (CardDef){"sparkmole", "Sparkmole", KIND_ACTIVATE, 2, 3, "Activate: Draw a card."};
  case CARD_TICKBOT: return (CardDef){"tickbot", "Tickbot", KIND_ENDTURN, 2, 2, "End of Turn: A random other card of yours here gets +1 Power."};
  case CARD_TIDALOTL: return (CardDef){"tidalotl", "Tidalotl", KIND_REVEAL, 3, 3, "On Reveal: Your cards in other zones get +1 Power."};
  case CARD_ROCKODILE: return (CardDef){"rockodile", "Rockodile", KIND_NONE, 3, 5, "Big jaws, no tricks."};
  case CARD_FUNGOO: return (CardDef){"fungoo", "Fungoo", KIND_REVEAL, 3, 2, "On Reveal: All enemy cards here get -1 Power."};
  case CARD_ZEPHYRROW: return (CardDef){"zephyrrow", "Zephyrrow", KIND_ONGOING, 3, 3, "Ongoing: +4 Power if this is your only card here."};
  case CARD_GLIMMERLING:
    return (CardDef){"glimmerling", "Glimmerling", KIND_REVEAL, 3, 3, "On Reveal: Add a Brickling (1 cost, 2 Power) to your hand."};
  case CARD_ECHOWL: return (CardDef){"echowl", "Echowl", KIND_ONGOING, 3, 2, "Ongoing: On Reveal abilities of your cards here happen twice."};
  case CARD_BUILDBEAVER: return (CardDef){"buildbeaver", "Buildbeaver", KIND_START, 3, 3, "Game Start: This card gets +2 Power for this game."};
  case CARD_THUNDRAKE: return (CardDef){"thundrake", "Thundrake", KIND_REVEAL, 4, 5, "On Reveal: The strongest enemy card here gets -3 Power."};
  case CARD_CORALYTH: return (CardDef){"coralyth", "Coralyth", KIND_ONGOING, 4, 3, "Ongoing: Your cards in other zones have +1 Power."};
  case CARD_EMBERHORN: return (CardDef){"emberhorn", "Emberhorn", KIND_REVEAL, 4, 7, "On Reveal: Your other cards here get -1 Power."};
  case CARD_BRAMBLEWOLF: return (CardDef){"bramblewolf", "Bramblewolf", KIND_ONGOING, 4, 3, "Ongoing: +1 Power for each other card you have here."};
  case CARD_PHANTOMULE: return (CardDef){"phantomule", "Phantomule", KIND_REVEAL, 4, 3, "On Reveal: Destroy the weakest enemy card here."};
  case CARD_BOOMSHROOM: return (CardDef){"boomshroom", "Boomshroom", KIND_ACTIVATE, 4, 4, "Activate: All enemy cards here get -2 Power."};
  case CARD_GLACIOMOTH: return (CardDef){"glaciomoth", "Glaciomoth", KIND_REVEAL, 5, 4, "On Reveal: All enemy cards here get -2 Power."};
  case CARD_MAGMAW: return (CardDef){"magmaw", "Magmaw", KIND_NONE, 5, 8, "A walking volcano."};
  case CARD_LUMIFOX: return (CardDef){"lumifox", "Lumifox", KIND_REVEAL, 5, 3, "On Reveal: Your other cards here get +2 Power."};
  case CARD_CRAGOLIATH: return (CardDef){"cragoliath", "Cragoliath", KIND_ONGOING, 5, 6, "Ongoing: +3 Power if your side of this zone is full."};
  case CARD_CLOCKCRAB: return (CardDef){"clockcrab", "Clockcrab", KIND_ENDTURN, 5, 4, "End of Turn: This card gets +2 Power."};
  case CARD_TITANOAK: return (CardDef){"titanoak", "Titanoak", KIND_NONE, 6, 11, "An ancient tree-titan."};
  case CARD_STORMAW: return (CardDef){"stormaw", "Stormaw", KIND_REVEAL, 6, 7, "On Reveal: ALL enemy cards get -1 Power."};
  case CARD_ABYSSQUID: return (CardDef){"abyssquid", "Abyssquid", KIND_ONGOING, 6, 5, "Ongoing: Your other cards here have +2 Power."};
  case CARD_GOLDBRICK:
    return (CardDef){"goldbrick", "Goldbrick", KIND_START, 2, 3, "Game Start: Put this card into your opening hand. It gets +2 Power for this game."};
  default: return (CardDef){"brickling", "Brickling", KIND_NONE, 1, 2, "A tiny sparkle brick."};
  }
}

typedef struct KindInfo {
  CString label;
  HzUIColor color, dark;
} KindInfo;

hz_internal KindInfo kind_info(CardKind k) {
  switch (k) {
  case KIND_REVEAL: return (KindInfo){"On Reveal", {255, 138, 61, 255}, {184, 64, 15, 255}};
  case KIND_ONGOING: return (KindInfo){"Ongoing", {61, 155, 255, 255}, {22, 85, 176, 255}};
  case KIND_ACTIVATE: return (KindInfo){"Activate", {79, 192, 106, 255}, {30, 122, 53, 255}};
  case KIND_ENDTURN: return (KindInfo){"End of Turn", {154, 116, 214, 255}, {79, 47, 138, 255}};
  case KIND_START: return (KindInfo){"Game Start", {255, 201, 40, 255}, {168, 122, 0, 255}};
  default: return (KindInfo){"No ability", {184, 188, 192, 255}, {92, 97, 102, 255}};
  }
}

// ---- zones (locations) ----
typedef enum ZoneId {
  ZONE_MEADOW, ZONE_STAGE, ZONE_POND, ZONE_GROVE, ZONE_LEDGE, ZONE_BARN, ZONE_LIBRARY, ZONE_GYM, ZONE_TOYBOX, ZONE_CLOCK,
  ZONE_TOWER, ZONE_WORKSHOP, ZONE_COUNT
} ZoneId;

typedef struct ZoneDef {
  CString name, text;
  HzUIColor color;
} ZoneDef;

hz_internal ZoneDef zone_def(i32 z) {
  switch (z) {
  case ZONE_MEADOW: return (ZoneDef){"Sunny Meadow", "No effect. Just a nice day.", {124, 207, 90, 255}};
  case ZONE_STAGE: return (ZoneDef){"Spotlight Stage", "On Reveal abilities here happen twice.", {255, 138, 61, 255}};
  case ZONE_POND: return (ZoneDef){"Calm Pond", "Ongoing cards here have +2 Power.", {61, 155, 255, 255}};
  case ZONE_GROVE: return (ZoneDef){"Old Grove", "Cards with no ability have +2 Power here.", {62, 154, 50, 255}};
  case ZONE_LEDGE: return (ZoneDef){"Sky Ledge", "Only 2 cards per side fit here.", {143, 224, 230, 255}};
  case ZONE_BARN: return (ZoneDef){"Haunted Barn", "All cards here have -1 Power.", {122, 90, 160, 255}};
  case ZONE_LIBRARY: return (ZoneDef){"Quiet Library", "On Reveal abilities do not trigger here.", {176, 138, 90, 255}};
  case ZONE_GYM: return (ZoneDef){"Brick Gym", "All cards here have +1 Power.", {224, 64, 64, 255}};
  case ZONE_TOYBOX: return (ZoneDef){"Toy Box", "Cards that cost 1 or 2 have +2 Power here.", {255, 176, 48, 255}};
  case ZONE_CLOCK: return (ZoneDef){"Clock Tower", "End of Turn abilities here happen twice.", {154, 116, 214, 255}};
  case ZONE_TOWER: return (ZoneDef){"Thunder Tower", "Closed on the final turn.", {255, 210, 61, 255}};
  default: return (ZoneDef){"Workshop", "Activate cards here have +2 Power.", {79, 192, 106, 255}};
  }
}

// ---- decks ----
#define DECK_STARTER 0
#define DECK_RIVAL 1
#define DECK_RIVAL_2 2
#define DECK_PRACTICE 3
#define DECK_PIP 4 // Pip: stronger than Dex's rematch deck (and every card +1 Power, cg_begin_board)

hz_internal i32 deck_card(i32 deck, i32 i) {
  i32 starter[DECK_SIZE] = {CARD_QUICKHOP, CARD_FLICKER, CARD_DRIZZLET, CARD_ZAPLING, CARD_VOLTAIL, CARD_CINDERPUP,
                            CARD_GUSTLING, CARD_TIDALOTL, CARD_ECHOWL, CARD_THUNDRAKE, CARD_EMBERHORN, CARD_TITANOAK};
  i32 rival[DECK_SIZE] = {CARD_FLICKER, CARD_ZAPLING, CARD_CINDERPUP, CARD_GLOOMOTH, CARD_SPARKMOLE, CARD_TICKBOT,
                          CARD_TIDALOTL, CARD_PHANTOMULE, CARD_THUNDRAKE, CARD_BOOMSHROOM, CARD_LUMIFOX, CARD_STORMAW};
  i32 rival2[DECK_SIZE] = {CARD_FLICKER, CARD_VOLTAIL, CARD_CINDERPUP, CARD_TICKBOT, CARD_ECHOWL, CARD_CORALYTH,
                           CARD_THUNDRAKE, CARD_PHANTOMULE, CARD_GLACIOMOTH, CARD_LUMIFOX, CARD_STORMAW, CARD_ABYSSQUID};
  i32 practice[DECK_SIZE] = {CARD_PEBBIT, CARD_SPROUTLE, CARD_QUICKHOP, CARD_MOSSCUB, CARD_GUSTLING, CARD_ROCKODILE,
                             CARD_BUILDBEAVER, CARD_BRAMBLEWOLF, CARD_CORALYTH, CARD_CLOCKCRAB, CARD_CRAGOLIATH, CARD_TITANOAK};
  i32 pip[DECK_SIZE] = {CARD_QUICKHOP, CARD_ZAPLING, CARD_VOLTAIL, CARD_CINDERPUP, CARD_GLOOMOTH, CARD_TIDALOTL,
                        CARD_THUNDRAKE, CARD_PHANTOMULE, CARD_GLACIOMOTH, CARD_MAGMAW, CARD_STORMAW, CARD_ABYSSQUID};
  switch (deck) {
  case DECK_PIP: return pip[i];
  case DECK_RIVAL: return rival[i];
  case DECK_RIVAL_2: return rival2[i];
  case DECK_PRACTICE: return practice[i];
  default: return starter[i];
  }
}

// the 18 cards Prof. Quill gives you
#define STARTER_COUNT 18
hz_internal i32 starter_card(i32 i) {
  i32 s[STARTER_COUNT] = {CARD_PEBBIT, CARD_FLICKER, CARD_DRIZZLET, CARD_SPROUTLE, CARD_ZAPLING, CARD_QUICKHOP,
                          CARD_VOLTAIL, CARD_CINDERPUP, CARD_MOSSCUB, CARD_GUSTLING, CARD_TIDALOTL, CARD_ROCKODILE,
                          CARD_FUNGOO, CARD_THUNDRAKE, CARD_EMBERHORN, CARD_BRAMBLEWOLF, CARD_TITANOAK, CARD_ECHOWL};
  return s[i];
}

// ---- upgrade stages (levels.ts) ----
#define STAGE_COUNT 5
#define VOXEL_LEVEL 4
#define WILD_EXP 20

typedef struct StageDef {
  CString name;
  u32 cost; // EXP from the stage before
  CString blurb;
  HzUIColor frame;
} StageDef;

hz_internal StageDef stage_def(i32 s) {
  switch (s) {
  case 1: return (StageDef){"Bronze", 15, "A shiny bronze frame.", {205, 127, 50, 255}};
  case 2: return (StageDef){"Silver", 30, "Silver frame with glitter.", {200, 208, 216, 255}};
  case 3: return (StageDef){"Gold", 45, "Gold frame and rainbow holo foil.", {255, 204, 51, 255}};
  case 4: return (StageDef){"Voxelmon", 80, "The beast steps out of the card: a real 3D Voxelmon!", {160, 90, 255, 255}};
  default: return (StageDef){"Basic", 0, "A fresh card.", {40, 44, 52, 255}};
  }
}

hz_internal b32 can_become_voxel(i32 c) { return card_def(c).cost >= 6; }
hz_internal i32 max_level(i32 c) { return can_become_voxel(c) ? VOXEL_LEVEL : VOXEL_LEVEL - 1; }

// EXP for a card battle: a win is worth the most, but you always learn something
hz_internal u32 battle_exp(i32 result, b32 rival) {
  switch (result) {
  case 0: return rival ? 40u : 30u; // win
  case 2: return 15u;               // draw
  case 1: return 10u;               // lose
  default: return 5u;               // retreat, fold
  }
}

// ---- the collection in the save ----
hz_internal b32 owns(const GameSave *s, i32 c) { return (s->collection >> c) & 1ull; }
hz_internal u32 owned_count(const GameSave *s) {
  u32 n = 0;
  for (i32 c = 0; c < CARD_POOL; c++)
    n += owns(s, c) ? 1u : 0u;
  return n;
}
hz_internal void give_card(GameSave *s, i32 c) { s->collection |= 1ull << c; }

// Prof. Quill's gift: the 18 starter cards and the starter deck
hz_internal void give_starter(GameSave *s) {
  for (i32 i = 0; i < STARTER_COUNT; i++)
    give_card(s, starter_card(i));
  for (i32 i = 0; i < DECK_SIZE; i++)
    s->deck[i] = (u8)deck_card(DECK_STARTER, i);
  s->deck_count = DECK_SIZE;
  s->has_deck = true;
}

// a random card the player does not have yet (rare: cost 5+ first); -1 when the collection is complete
hz_internal i32 reward_card(GameSave *s, u32 *seed, b32 rare) {
  i32 missing[CARD_POOL], n = 0, nr = 0, rares[CARD_POOL];
  for (i32 c = 0; c < CARD_POOL; c++)
    if (!owns(s, c)) {
      missing[n++] = c;
      if (card_def(c).cost >= 5)
        rares[nr++] = c;
    }
  if (!n)
    return -1;
  *seed = *seed * 1664525u + 1013904223u;
  u32 r = *seed >> 8;
  i32 c = rare && nr ? rares[r % (u32)nr] : missing[r % (u32)n];
  give_card(s, c);
  return c;
}

hz_internal u32 upgrade_cost(const GameSave *s, i32 c) {
  i32 lv = s->card_level[c];
  return lv >= max_level(c) ? 0u : stage_def(lv + 1).cost;
}

// spends EXP on the next stage; the first Voxelmon walks with you at once
hz_internal b32 upgrade(GameSave *s, i32 c) {
  u32 cost = upgrade_cost(s, c);
  if (!cost || s->exp < cost)
    return false;
  s->exp -= cost;
  s->card_level[c]++;
  if (s->card_level[c] == VOXEL_LEVEL && !s->voxelmon)
    s->voxelmon = c + 1;
  return true;
}

// ---- the pictures: each card's pixel art as a texture, made the first time it is drawn ----
typedef struct CardArt {
  GpuTexture tex[CARD_COUNT];
  u32 w[CARD_COUNT], h[CARD_COUNT];
  b32 made[CARD_COUNT];
} CardArt;

#define ART_SCALE 4 // pixels per voxel: the UI scales the picture smoothly, so each voxel is a crisp block

hz_internal GpuTexture card_art(CardArt *a, i32 c, u32 *w, u32 *h) {
  if (!a->made[c]) {
    a->made[c] = true;
    BeastPicture pic = beast_picture(card_def(c).id);
    if (pic.rgba) {
      u32 W = pic.w * ART_SCALE, H = pic.h * ART_SCALE;
      tctx_temp_allocator_scope(ta, NULL) {
        u32 *px = ALLOC_ARRAY(&ta.allocator, u32, W * H);
        for (u32 y = 0; y < H; y++)
          for (u32 x = 0; x < W; x++)
            px[y * W + x] = pic.rgba[(y / ART_SCALE) * pic.w + x / ART_SCALE];
        a->tex[c] = gpu_make_texture(&(GpuTextureDesc){.data = (u8 *)px, .size = W * H * 4u, .data_type = GPU_TEXTURE_DATA_RGBA8,
                                                       .width = W, .height = H, .is_srgb = true});
      }
      a->w[c] = pic.w;
      a->h[c] = pic.h;
    }
  }
  *w = a->w[c];
  *h = a->h[c];
  return a->tex[c];
}
