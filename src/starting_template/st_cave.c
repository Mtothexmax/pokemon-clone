// The pond cave: under the pond lies a flooded basin of LEGO rock, built far east of everything and drawn only while
// you dive (draw group CAVE_GROUP). Inspired by abyssal-garden (github.com/Noniv/abyssal-garden): a kelp forest, a
// purple grotto under an overhang, coral and sponges on the ledges, a stone whale tail, a sunken boat and a deep trench
// where the Stud Core lies. The living parts (kelp swaying, fish, mantas, bubbles, glowing plants, the pearls, the
// lights) and the swimming are st_dive.c. Included by st_town.c after st_rooms.c; cave coordinates are studs from
// (CAVE_X, 0, CAVE_Z).

#define CAVE_X 640.0f
#define CAVE_Z 0.0f
#define CAVE_R 40.0f    // the basin: x and z in [-R, R]
#define CAVE_TOP 80.0f  // the ceiling; the hole up to the pond in its middle (|x|, |z| < CAVE_HOLE)
#define CAVE_HOLE 5.0f
#define CAVE_GROUP 5    // its draw group (TOWN_GROUPS)
#define CAVE_CELL 8.0f  // the floor's cells

// what the diver bumps into (cave coordinates), filled while building
typedef struct CaveBox {
  f32 x0, x1, y0, y1, z0, z1;
} CaveBox;
#define CAVE_SOLIDS 128
hz_global CaveBox g_cave_solid[CAVE_SOLIDS];
hz_global u32 g_cave_solid_n;

hz_internal f32 cave_hash(i32 a, i32 b) {
  u32 h = (u32)(a * 73856093) ^ (u32)(b * 19349663) ^ 0x9e3779b9u;
  h ^= h >> 13;
  h *= 0x5bd1e995u;
  h ^= h >> 15;
  return (f32)(h & 0xffffu) / 65535.0f;
}

// the floor under (x, z): low hills of sand and rock, the trench in the north (x within 12, z past 14) far below
hz_internal f32 cave_floor(f32 x, f32 z) {
  i32 i = (i32)m_floorf((x + CAVE_R) / CAVE_CELL), k = (i32)m_floorf((z + CAVE_R) / CAVE_CELL);
  f32 cx = -CAVE_R + ((f32)i + 0.5f) * CAVE_CELL, cz = -CAVE_R + ((f32)k + 0.5f) * CAVE_CELL;
  if (m_absf(cx) < 12.0f && cz > 14.0f)
    return cz > 30.0f ? -24.0f : -14.0f; // the trench, in two steps
  f32 rim = m_maxf(m_absf(cx), m_absf(cz)) > 30.0f ? 4.0f : 0.0f; // the floor rises towards the walls
  return 1.0f + 3.0f * cave_hash(i, k) + rim;
}

hz_internal void cave_solid(f32 x0, f32 x1, f32 y0, f32 y1, f32 z0, f32 z1) {
  if (g_cave_solid_n < CAVE_SOLIDS)
    g_cave_solid[g_cave_solid_n++] = (CaveBox){x0, x1, y0, y1, z0, z1};
}

// a solid brick in cave coordinates (bottom at y)
hz_internal void cave_rock(Town *t, u32 hex, f32 w, f32 h, f32 d, f32 x, f32 y, f32 z, b32 solid) {
  town_box(t, hex, w, h, d, CAVE_X + x, y, CAVE_Z + z, false);
  if (solid)
    cave_solid(x - w * 0.5f, x + w * 0.5f, y, y + h, z - d * 0.5f, z + d * 0.5f);
}

// coral: a stem and branches going up and out
hz_internal void cave_coral(Town *t, f32 x, f32 y, f32 z, u32 hex, f32 s) {
  town_box(t, hex, 0.5f * s, 1.6f * s, 0.5f * s, CAVE_X + x, y, CAVE_Z + z, false);
  for (i32 b = 0; b < 4; b++) {
    f32 a = (f32)b * 1.57f + cave_hash((i32)(x * 7.0f), b) * 0.8f;
    f32 len = (1.0f + cave_hash(b, (i32)(z * 5.0f))) * s;
    v3 c = {CAVE_X + x + m_sinf(a) * len * 0.4f, y + 1.4f * s + len * 0.4f, CAVE_Z + z + m_cosf(a) * len * 0.4f};
    town_box_rot(t, hex, (v3){0.35f * s, len, 0.35f * s}, c, (v3){m_cosf(a) * 0.6f, 0.0f, -m_sinf(a) * 0.6f});
    town_box(t, hex, 0.55f * s, 0.55f * s, 0.55f * s, c.x + m_sinf(a) * len * 0.3f, c.y + len * 0.4f, c.z + m_cosf(a) * len * 0.3f, false);
  }
}

// a barrel sponge: a tube with a dark throat
hz_internal void cave_sponge(Town *t, f32 x, f32 y, f32 z, f32 h, u32 hex) {
  town_cyl(t, hex, CAVE_X + x, y, CAVE_Z + z, 1.1f, 1.5f, h, 10, false);
  town_cyl(t, 0x2a120au, CAVE_X + x, y + h - 0.2f, CAVE_Z + z, 1.0f, 1.0f, 0.25f, 10, false);
}

hz_internal void cave_build(Town *t) {
  town_group_switch(t, CAVE_GROUP);
  g_cave_solid_n = 0;
  const u32 rock_a = 0x3a4452u, rock_b = 0x2b3440u, rock_c = 0x4a5361u, sand_a = 0xb9a370u, sand_b = 0x8f8058u;
  // ---- the floor: a column per cell, sand on the low middle, rock towards the walls ----
  for (f32 x = -CAVE_R; x < CAVE_R - 0.1f; x += CAVE_CELL)
    for (f32 z = -CAVE_R; z < CAVE_R - 0.1f; z += CAVE_CELL) {
      f32 cx = x + CAVE_CELL * 0.5f, cz = z + CAVE_CELL * 0.5f, fh = cave_floor(cx, cz);
      b32 deep = fh < 0.0f;
      u32 c = deep ? (fh < -20.0f ? 0x1f2733u : rock_b) : m_maxf(m_absf(cx), m_absf(cz)) > 28.0f ? rock_c : (cave_hash((i32)cx, (i32)cz) > 0.5f ? sand_a : sand_b);
      town_box(t, c, CAVE_CELL, fh + 30.0f, CAVE_CELL, CAVE_X + cx, -30.0f, CAVE_Z + cz, false);
      if (!deep && cave_hash((i32)cz, (i32)cx) > 0.7f) // a loose brick or two on the floor
        town_box(t, cave_hash((i32)cx, 3) > 0.5f ? rock_c : sand_b, 2.0f, 1.2f, 4.0f, CAVE_X + cx + 1.0f, fh, CAVE_Z + cz - 1.0f, true);
    }
  // ---- the walls: banded rock, with ledges sticking out ----
  for (i32 side = 0; side < 4; side++) {
    for (i32 band = 0; band < 10; band++) {
      f32 y = -30.0f + (f32)band * 11.6f;
      u32 c = band % 2 ? rock_a : rock_b;
      f32 len = 2.0f * CAVE_R + 12.0f;
      if (side < 2)
        town_box(t, c, len, 11.6f, 6.0f, CAVE_X, y, CAVE_Z + (side ? 1.0f : -1.0f) * (CAVE_R + 3.0f), false);
      else
        town_box(t, c, 6.0f, 11.6f, len, CAVE_X + (side == 3 ? 1.0f : -1.0f) * (CAVE_R + 3.0f), y, CAVE_Z, false);
    }
    for (i32 l = 0; l < 7; l++) {
      f32 along = -32.0f + (f32)l * 10.5f + cave_hash(side, l) * 4.0f, y = 10.0f + cave_hash(l, side) * 58.0f;
      f32 out = 3.0f + cave_hash(side * 7, l) * 3.0f;
      f32 w = 6.0f + cave_hash(l * 3, side) * 5.0f;
      f32 x, z, sx, sz;
      if (side < 2) {
        x = along;
        z = (side ? 1.0f : -1.0f) * (CAVE_R - out * 0.5f);
        sx = w;
        sz = out;
      } else {
        z = along;
        x = (side == 3 ? 1.0f : -1.0f) * (CAVE_R - out * 0.5f);
        sx = out;
        sz = w;
      }
      cave_rock(t, l % 2 ? rock_c : rock_a, sx, 2.0f, sz, x, y, z, true);
      // something grows on most ledges
      u32 corals[4] = {0xd63a3au, 0xff7aa8u, 0xff9a3cu, 0x9b4dd6u};
      if (cave_hash(l, side * 5) > 0.35f)
        cave_coral(t, x, y + 2.0f, z, corals[(side + l) % 4], 0.8f + cave_hash(l, l) * 0.5f);
      else
        cave_sponge(t, x, y + 2.0f, z, 2.0f + cave_hash(side, l * 9) * 2.0f, l % 2 ? 0x9a3a14u : 0xa35a1au);
    }
  }
  // ---- the ceiling around the hole up to the pond, and the short shaft above it ----
  f32 R = CAVE_R + 6.0f, H = CAVE_HOLE;
  town_box(t, rock_b, R - H, 6.0f, 2.0f * R, CAVE_X - (R + H) * 0.5f, CAVE_TOP, CAVE_Z, false);
  town_box(t, rock_b, R - H, 6.0f, 2.0f * R, CAVE_X + (R + H) * 0.5f, CAVE_TOP, CAVE_Z, false);
  town_box(t, rock_a, 2.0f * H, 6.0f, R - H, CAVE_X, CAVE_TOP, CAVE_Z - (R + H) * 0.5f, false);
  town_box(t, rock_a, 2.0f * H, 6.0f, R - H, CAVE_X, CAVE_TOP, CAVE_Z + (R + H) * 0.5f, false);
  for (i32 k = 0; k < 4; k++) { // the shaft's walls, a brick wider than the hole
    f32 sx = k < 2 ? 2.0f * H + 4.0f : 2.0f, sz = k < 2 ? 2.0f : 2.0f * H + 4.0f;
    f32 ox = k < 2 ? 0.0f : (k == 2 ? -1.0f : 1.0f) * (H + 1.0f), oz = k < 2 ? (k == 0 ? -1.0f : 1.0f) * (H + 1.0f) : 0.0f;
    town_box(t, rock_c, sx, 14.0f, sz, CAVE_X + ox, CAVE_TOP + 6.0f, CAVE_Z + oz, false);
  }
  // ---- rock pillars, some with an arch between them ----
  f32 pil[7][3] = {{-20, -8, 52}, {-20, 8, 46}, {22, 10, 60}, {-6, -30, 38}, {30, 30, 30}, {8, 22, 44}, {-30, 28, 56}};
  for (i32 i = 0; i < 7; i++) {
    f32 px = pil[i][0], pz = pil[i][1], top = pil[i][2], y = cave_floor(px, pz);
    f32 w = 5.0f;
    while (y < top) {
      f32 h = 5.0f + cave_hash(i, (i32)y) * 4.0f;
      f32 jx = (cave_hash((i32)y, i) - 0.5f) * 1.6f, jz = (cave_hash(i * 3, (i32)y) - 0.5f) * 1.6f;
      cave_rock(t, ((i32)y / 5) % 2 ? rock_a : rock_c, w, h, w, px + jx, y, pz + jz, true);
      y += h;
      w = m_maxf(3.0f, w - 0.4f + cave_hash(i, (i32)h) * 0.8f);
    }
  }
  cave_rock(t, rock_c, 6.0f, 4.0f, 22.0f, -20.0f, 40.0f, 0.0f, true); // the arch over the kelp forest's way
  // ---- the grotto: an overhang in the south-east, its roof on two pillars (the glow plants: st_dive.c) ----
  cave_rock(t, rock_b, 28.0f, 4.0f, 26.0f, 26.0f, 26.0f, -27.0f, true);
  cave_rock(t, rock_a, 30.0f, 2.0f, 28.0f, 26.0f, 30.0f, -27.0f, false);
  for (i32 i = 0; i < 2; i++)
    cave_rock(t, rock_c, 4.0f, 26.0f - cave_floor(14.0f + (f32)i * 24.0f, -16.0f), 4.0f, 14.0f + (f32)i * 24.0f,
              cave_floor(14.0f + (f32)i * 24.0f, -16.0f), -16.0f, true);
  // ---- the stone whale tail rising out of the sand ----
  {
    f32 x = 2.0f, z = -18.0f, y = cave_floor(x, z);
    u32 stone = 0x9aa3a6u, streak = 0x6b7477u;
    cave_rock(t, stone, 3.0f, 10.0f, 3.0f, x, y, z, true);
    town_box(t, streak, 3.1f, 0.6f, 3.1f, CAVE_X + x, y + 3.0f, CAVE_Z + z, false);
    town_box_rot(t, stone, (v3){9.0f, 1.2f, 4.0f}, (v3){CAVE_X + x - 4.0f, y + 12.0f, CAVE_Z + z}, (v3){0.0f, 0.0f, 0.45f});
    town_box_rot(t, stone, (v3){9.0f, 1.2f, 4.0f}, (v3){CAVE_X + x + 4.0f, y + 12.0f, CAVE_Z + z}, (v3){0.0f, 0.0f, -0.45f});
    cave_solid(x - 8.0f, x + 8.0f, y + 9.0f, y + 15.0f, z - 2.0f, z + 2.0f);
  }
  // ---- a sunken boat of brown bricks, on its side ----
  {
    f32 x = -26.0f, z = 18.0f, y = cave_floor(x, z);
    u32 wood = 0x5c3a1eu, dark = 0x3a2412u;
    town_box_rot(t, wood, (v3){14.0f, 1.0f, 6.0f}, (v3){CAVE_X + x, y + 1.0f, CAVE_Z + z}, (v3){0.0f, 0.3f, 0.25f});
    town_box_rot(t, wood, (v3){14.0f, 4.0f, 1.0f}, (v3){CAVE_X + x - 0.6f, y + 3.0f, CAVE_Z + z + 3.0f}, (v3){0.0f, 0.3f, 0.25f});
    town_box_rot(t, dark, (v3){14.0f, 3.0f, 1.0f}, (v3){CAVE_X + x + 0.6f, y + 2.0f, CAVE_Z + z - 3.0f}, (v3){0.0f, 0.3f, 0.25f});
    town_box_rot(t, dark, (v3){0.8f, 14.0f, 0.8f}, (v3){CAVE_X + x + 2.0f, y + 7.0f, CAVE_Z + z}, (v3){0.0f, 0.0f, 0.5f}); // the mast
    cave_solid(x - 7.0f, x + 7.0f, y, y + 5.0f, z - 3.5f, z + 3.5f);
  }
  // ---- sponges and coral on the floor ----
  for (i32 i = 0; i < 18; i++) {
    f32 x = -34.0f + cave_hash(i, 1) * 68.0f, z = -34.0f + cave_hash(i, 2) * 68.0f;
    if (m_absf(x) < 13.0f && z > 13.0f)
      continue; // not in the trench
    f32 y = cave_floor(x, z);
    if (i % 3 == 0)
      cave_sponge(t, x, y, z, 2.0f + cave_hash(i, 3) * 3.0f, i % 2 ? 0x9a3a14u : 0xa35a1au);
    else
      cave_coral(t, x, y, z, (u32[]){0xd63a3au, 0xff7aa8u, 0xff9a3cu, 0x9b4dd6u}[i % 4], 0.9f + cave_hash(i, 4) * 0.6f);
  }
  // ---- the bottom of the trench: a pedestal for the Stud Core ----
  cave_rock(t, 0x55606cu, 5.0f, 2.0f, 5.0f, 0.0f, -24.0f, 34.0f, true);
  cave_rock(t, 0x6a7582u, 3.6f, 1.0f, 3.6f, 0.0f, -22.0f, 34.0f, true);
  town_tag(t, "The pond cave", CAVE_X, CAVE_Z, CAVE_R);
}
