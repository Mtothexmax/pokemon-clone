// Route 1 north and Stonecrest (the three.js round 2, town.ts buildNorth/buildFountain): the dirt trail through the
// forest, the creek with its bridge, three more patches of tall grass, the town below the old castle with its six
// houses, the fountain on the square, Sir Bricksworth and Marla. Coordinates in studs (x east, z south), y the bottom
// of a box as in st_town.c. Included by st_town.c after st_cave.c.

// every patch of tall grass (wild Voxelmon: st_vox.c), the first one north of the village gap
#define TOWN_GRASS_COUNT 4
hz_global const TownRect g_town_grass[TOWN_GRASS_COUNT] = {
    {-8.6f, 8.6f, -42.0f, -28.2f},
    {-17.0f, -4.5f, -64.0f, -50.0f},
    {4.5f, 18.0f, -92.0f, -80.0f},
    {-16.0f, -4.5f, -106.0f, -94.0f},
};

hz_internal i32 town_grass_at(f32 x, f32 z) {
  for (i32 i = 0; i < TOWN_GRASS_COUNT; i++)
    if (x > g_town_grass[i].x0 && x < g_town_grass[i].x1 && z > g_town_grass[i].z0 && z < g_town_grass[i].z1)
      return i;
  return -1;
}

// the zone you are in (outside): 0 Brickhollow, 1 Route 1, 2 Stonecrest (three.js zoneLabel)
hz_internal i32 town_zone(f32 z) { return z < -111.0f ? 2 : z < -27.0f ? 1 : 0; }

// the villagers of Stonecrest (st_minifig.c spawns them, st_world.c talks to them)
#define KNIGHT_X 7.0f
#define KNIGHT_Z -146.2f
#define BAKER_X -8.0f
#define BAKER_Z -110.6f

#define NORTH_CREEK_Z0 -75.0f
#define NORTH_CREEK_Z1 -71.0f

hz_internal b32 north_near_creek(f32 z) { return z > -77.5f && z < -68.5f; }

// a sign on a post with its painted board (as the Route 1 sign in st_town.c)
hz_internal void north_sign(Town *t, f32 x, f32 z, u32 board, CString glb) {
  town_box(t, C_BROWN, 0.35f, 3.2f, 0.35f, x, 0.0f, z - 0.2f, false);
  town_sign_board(t, x, z, 3.4f, 1.8f, 1.3f, board);
  town_prop(t, glb, x, z + 0.21f, 2.2f, 1.0f, 0.0f, false);
  town_block(t, x - 0.5f, x + 0.5f, z - 0.7f, z + 0.3f);
}

// the fountain on the square (buildFountain): a stone basin with water, a pillar, a bowl and the spout
hz_internal void north_fountain(Town *t, f32 x, f32 z) {
  town_cyl(t, C_GRAY, x, 0.4f, z, 3.2f, 3.0f, 1.0f, 28, true);
  town_cyl(t, C_WATER, x, 1.32f, z, 2.6f, 2.6f, 0.12f, 28, true);
  town_cyl(t, C_GRAY, x, 1.1f, z, 0.6f, 0.45f, 2.6f, 16, true);
  town_cyl(t, C_GRAY, x, 3.55f, z, 0.6f, 1.3f, 0.5f, 20, true);
  town_cyl(t, 0xbfe6ffu, x, 3.9f, z, 0.35f, 0.18f, 1.4f, 12, true);
  town_block(t, x - 3.3f, x + 3.3f, z - 3.3f, z + 3.3f);
}

hz_internal void town_north(Town *t) {
  // ---- the dirt trail from the grass to the town ----
  town_box(t, 0xc8a46au, 5.0f, 0.04f, 69.0f, 0.0f, 0.0f, -76.5f, false);

  // ---- the creek across the forest: a dark bed, the water, stone banks; only the bridge crosses ----
  town_box(t, 0x1d4a5eu, 102.0f, 0.02f, 4.0f, 5.0f, 0.0f, -73.0f, false);
  town_box(t, C_WATER, 102.0f, 0.1f, 3.8f, 5.0f, 0.02f, -73.0f, false);
  town_box(t, C_GRAY, 102.0f, 0.35f, 0.6f, 5.0f, 0.0f, -75.3f, true);
  town_box(t, C_GRAY, 102.0f, 0.35f, 0.6f, 5.0f, 0.0f, -70.7f, true);
  town_block(t, -46.0f, -3.0f, -75.6f, -70.4f);
  town_block(t, 3.0f, 56.0f, -75.6f, -70.4f);
  // the bridge: a deck you walk onto (0.5 high, solid as the roads), planks, posts and rails
  {
    MeshBuilderStyle st = town_style(0x8a5a32u);
    st.collider = MESH_BUILDER_COLLIDER_BOX;
    mesh_builder_box(t->b, &st, m4_trs(town_m(0.0f, 0.25f, -73.0f), quat_identity(), V3_ONE), town_m(6.8f, 0.5f, 6.4f));
  }
  for (f32 z = -75.7f; z < -69.8f; z += 1.0f)
    town_box(t, 0x6b4426u, 6.6f, 0.06f, 0.12f, 0.0f, 0.5f, z, false);
  for (i32 s = -1; s <= 1; s += 2) {
    f32 x = 3.2f * (f32)s;
    f32 zs[3] = {-75.9f, -73.0f, -70.1f};
    for (i32 k = 0; k < 3; k++)
      town_box(t, 0x6b4426u, 0.35f, 1.6f, 0.35f, x, 0.5f, zs[k], false);
    town_box(t, 0x8a5a32u, 0.3f, 0.25f, 6.4f, x, 1.75f, -73.0f, false);
    town_block(t, x - 0.25f, x + 0.25f, -76.2f, -69.8f); // the rails keep you on the deck
  }

  // ---- the two signs ----
  north_sign(t, 5.5f, -49.2f, 0xfff6c8u, "brickhollow/signs/north.glb");
  north_sign(t, 5.5f, -108.6f, C_WHITE, "brickhollow/signs/stonecrest.glb");

  // ---- the forest along the route (spruces and round trees beside it, a scatter of trees and bushes inside) ----
  for (f32 z = -48.5f; z >= -110.0f; z -= 3.6f) {
    for (f32 ax = 21.5f; ax <= 55.0f; ax += ax < 30.0f ? 3.6f : 4.6f) {
      for (i32 s = -1; s <= 1; s += 2) {
        f32 x = (f32)s * ax + (town_rand(&t->rng) - 0.5f) * 1.2f;
        if (x < -45.0f || north_near_creek(z) || (ax > 30.0f && town_rand(&t->rng) < 0.25f))
          continue;
        f32 zz = z + (town_rand(&t->rng) - 0.5f) * 1.2f;
        if (town_rand(&t->rng) < 0.3f)
          town_round_tree(t, x, zz, 6.0f + town_rand(&t->rng) * 1.5f);
        else
          town_spruce(t, x, zz, 8.5f + town_rand(&t->rng) * 2.5f);
      }
    }
  }
  {
    u32 seed = 7u;
    for (f32 z = -50.0f; z >= -108.0f; z -= 5.5f)
      for (f32 x = -17.0f; x <= 17.0f; x += 5.5f) {
        seed = (u32)(((u64)seed * 16807u) % 2147483647u);
        f32 px = x + ((f32)seed / 2147483647.0f - 0.5f) * 2.0f;
        seed = (u32)(((u64)seed * 16807u) % 2147483647u);
        f32 pz = z + ((f32)seed / 2147483647.0f - 0.5f) * 2.0f;
        if (m_absf(px) < 6.0f || north_near_creek(pz))
          continue;
        b32 by_grass = false;
        for (i32 g = 1; g < TOWN_GRASS_COUNT; g++) {
          TownRect r = g_town_grass[g];
          if (px > r.x0 - 2.5f && px < r.x1 + 2.5f && pz > r.z0 - 2.5f && pz < r.z1 + 2.5f)
            by_grass = true;
        }
        if (by_grass || (m_absf(px - 5.5f) < 3.0f && (m_absf(pz + 49.2f) < 4.0f || m_absf(pz + 108.6f) < 4.0f)))
          continue;
        seed = (u32)(((u64)seed * 16807u) % 2147483647u);
        if ((f32)seed / 2147483647.0f < 0.45f)
          continue;
        seed = (u32)(((u64)seed * 16807u) % 2147483647u);
        f32 r01 = (f32)seed / 2147483647.0f;
        if (r01 < 0.5f) {
          town_round_tree(t, px, pz, 6.0f + r01 * 1.5f);
          town_block(t, px - 1.3f, px + 1.3f, pz - 1.3f, pz + 1.3f);
        } else {
          town_bush(t, px, pz, 2.2f + r01 * 0.6f);
          town_block(t, px - 1.1f, px + 1.1f, pz - 1.1f, pz + 1.1f);
        }
      }
  }
  // around Stonecrest
  for (f32 z = -112.0f; z >= -192.0f; z -= 3.6f) {
    town_spruce(t, -44.2f + (town_rand(&t->rng) - 0.5f) * 1.2f, z, 8.5f + town_rand(&t->rng) * 2.5f);
    town_spruce(t, 53.5f + (town_rand(&t->rng) - 0.5f) * 1.2f, z + 1.8f, 8.5f + town_rand(&t->rng) * 2.5f);
  }
  for (f32 x = -40.0f; x <= 50.0f; x += 3.6f) {
    town_spruce(t, x + (town_rand(&t->rng) - 0.5f) * 1.2f, -191.5f, 8.5f + town_rand(&t->rng) * 2.5f);
    if (x < -24.0f || x > 24.0f)
      town_spruce(t, x + 1.8f + (town_rand(&t->rng) - 0.5f) * 1.2f, -187.5f, 8.5f + town_rand(&t->rng) * 2.5f);
  }

  // ---- Stonecrest: six houses (their doors lead nowhere), the fountain, the old castle ----
  town_house(t, -17.0f, -116.0f, 12.0f, 8.0f, C_RED, C_TAN, C_BLUE, C_RED, "brickhollow/signs/house_bakery.glb");
  town_house(t, 17.0f, -116.0f, 12.0f, 8.0f, C_BLUE, C_WHITE, C_RED, C_RED, "brickhollow/signs/house_inn.glb");
  town_house(t, 33.0f, -116.0f, 10.0f, 8.0f, C_DARK_GREEN, C_YELLOW, C_RED_BROWN, C_RED, "brickhollow/signs/house_shop.glb");
  town_house(t, -22.0f, -137.0f, 12.0f, 9.0f, C_ORANGE, C_WHITE, C_DARK_GREEN, C_RED, "brickhollow/signs/house_market.glb");
  town_house(t, 22.0f, -137.0f, 12.0f, 9.0f, C_RED_BROWN, C_GRAY, C_YELLOW, C_RED, "brickhollow/signs/house_smith.glb");
  town_house(t, 36.0f, -137.0f, 9.0f, 8.0f, C_BLUE, C_TAN, C_RED, C_RED, "brickhollow/signs/house_post.glb");
  north_fountain(t, 0.0f, -140.0f);
  {
    const f32 cx = 0.0f, cz = -166.0f, sc = 1.6f;
    f32 hx = 27.7f * sc * 0.5f, hz = 21.9f * sc * 0.5f; // turned by 90 degrees
    town_prop(t, PROP("castle"), cx, cz, 0.0f, sc, M_PI * 0.5f, true);
    town_block_h(t, cx - hx, cx + hx, cz - hz, cz + hz, 12.0f);
  }
  // Sir Bricksworth and Marla stand still (a block each, as the villagers in the village)
  town_block(t, KNIGHT_X - 0.9f, KNIGHT_X + 0.9f, KNIGHT_Z - 0.7f, KNIGHT_Z + 0.7f);
  town_block(t, BAKER_X - 0.9f, BAKER_X + 0.9f, BAKER_Z - 0.7f, BAKER_Z + 0.7f);

  // ---- where you can walk: Route 1 (x -20..20) and Stonecrest (x -42..48), walls along their edges ----
  {
    TownRect walls[] = {
        {-21.0f, -20.0f, -112.0f, -41.6f}, // Route 1, west
        {20.0f, 21.0f, -112.0f, -41.6f},   //          east
        {-20.0f, -9.4f, -42.6f, -41.6f},   //          south, beside the first patch
        {9.4f, 20.0f, -42.6f, -41.6f},
        {-43.0f, -42.0f, -191.0f, -111.0f}, // Stonecrest, west
        {48.0f, 49.0f, -191.0f, -111.0f},   //             east
        {-42.0f, 48.0f, -191.0f, -190.0f},  //             north
        {-42.0f, -20.0f, -112.0f, -111.0f}, //             south, beside the route
        {20.0f, 48.0f, -112.0f, -111.0f},
    };
    for (u32 i = 0; i < sizeof(walls) / sizeof(walls[0]); i++)
      town_block(t, walls[i].x0, walls[i].x1, walls[i].z0, walls[i].z1);
  }

  town_tag(t, "Route 1 (north)", 0.0f, -76.0f, 20.0f);
  town_tag(t, "The creek", 20.0f, -73.0f, 3.0f);
  town_tag(t, "The bridge", 0.0f, -73.0f, 3.4f);
  town_tag(t, "Stonecrest sign", 5.5f, -108.6f, 2.0f);
  town_tag(t, "Route sign (Stonecrest)", 5.5f, -49.2f, 2.0f);
  town_tag(t, "Bakery", -17.0f, -116.0f, 6.0f);
  town_tag(t, "Inn", 17.0f, -116.0f, 6.0f);
  town_tag(t, "Shop", 33.0f, -116.0f, 5.0f);
  town_tag(t, "Market", -22.0f, -137.0f, 6.0f);
  town_tag(t, "Smith", 22.0f, -137.0f, 6.0f);
  town_tag(t, "Post office", 36.0f, -137.0f, 4.5f);
  town_tag(t, "The fountain", 0.0f, -140.0f, 3.3f);
  town_tag(t, "The old castle", 0.0f, -166.0f, 20.0f);
}
