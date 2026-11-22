# Brickhollow on wasmOnly: reference

The LEGO village game (Brickhollow, "Snap Beasts") ported from the Kaigen version (HZ engine, C:
`%LOCALAPPDATA%/KaigenProjects/pokemontest`, `src/starting_template/st_*.c`, assets in `assets/brickhollow/`) to this
crate. The three.js original is the parent repo (`C:/dev/pokemonsnap`, branch history `threejs-version`). Every part of
the Kaigen port is in; the differences are listed at the end. Doc comments name the Kaigen / three.js function a piece
comes from.

## Rules for every part
- Never patch the engine (`C:/dev/wasmOnly`). Missing engine features or engine bugs: write them down in the
  report (the main session forwards them to the engine session) and work around them in the game if possible.
- Dev loop: the watcher on http://localhost:8091/ is already running (do not start another one). Debug Chrome:
  `node C:/dev/wasmOnly/tools/bridge.mjs <cmd> --port 9223 --url http://localhost:8091/` (do not launch another
  Chrome, do not touch the servers on ports 8443, 8444, 8780). The watcher rebuilds on any change in the game folder
  except `tests/`, `logs/`, `dist/`, `target/` (so don't edit files while a test runs).
- The build has no warnings from the game's code.
- Every feature gets a scripted test in `tests/*.txt`; all tests pass before a commit.
- Commit after each finished part (source, tests, needed assets; never `out/` or `dist/`), never push.
- Phones: every action reachable by touch (`data-action` buttons, `ctx.ui.take_actions()`), keep the frame
  rate (phone profile in `phone_quality`).

## Conventions
- **Units:** game coordinates are **studs** like the original (`town::STUD` m per stud; a brick is 1.2 studs high, a
  plate 0.4), x east, z south, house fronts face +z. `town::m(x, y, z)` turns studs into metres. `Life`, `Vox`, the
  cave and the dive keep studs too; engine positions (`hero_pos`, `npc_pos`) are metres.
- **Render layers** (`MeshRenderer::layer`, `ctx.set_layer`, `ParticleEmitter::layer`): 0 the hero, 1 the village
  (`rooms::VILLAGE_LAYER`), 2..5 the rooms (`rooms::layer_of`), 6 / 7 / 9 / 10 the north bands (`town::NORTH_BANDS`),
  8 the cave (`cave::CAVE_LAYER`), 11 / 12 / 13 the island's near chunks, far chunks and sea (`island::LAYER_NEAR`,
  `LAYER_FAR`, `LAYER_SEA`), 29 the battle table (`arena::LAYER`), 31 drawn by no camera (`rooms::HIDDEN_LAYER`);
  14..28 are free. Anything outside goes on `town::layer_at(x, z)`, or it shows everywhere / nowhere. The play camera
  draws `1 | town::bands_near(z, 30, 40)` outside (`PlayScreen::outdoor_zone`, every frame), `town::all_bands()` while
  it looks far (a close-up, a fight), `1 | room layer` inside (`show_area`). The engine's z-fighting check does not
  know layers: a hidden duplicate also moves away (the Deck-o-Matic's screens step 2 m down). Near the island
  (`IslandView::layers`) the mask adds 11 | 13 (12 too while looking far); on the island the strip's bands reach only
  24 studs south (its forest is heavy). `town::layer_at` gives `LAYER_NEAR` north of `island::SOUTH_Z` (z -194).
- **Randomness:** `crate::lcg` / `lcg_f` (the Kaigen LCG) for every game stream (town layout, `Life` seed 4242,
  battles, Voxelmon, sounds); particles use `ctx.particle_rng` / fixed seeds; `Data::cg_rand` mixes `Math.random`
  into the card game's seed (`battle {seed}` fixes one battle).
- **Fixed step vs frame:** movement, villagers, life, grass pushers, the gate walk run in `fixed_update`; read
  `ctx.time.fixed_dt` (on phones `lock_fixed_to_frames` makes it a whole number of 1/60 s steps per drawn frame). HUD,
  camera, sound, fades run in `update`.
- **HUD:** HTML slots via `ctx.ui.set(slot, html)`; each module puts its CSS in a style slot once
  (`Data::ensure_loaded`: `play::STYLE`, `screens::STYLE`, `deck::STYLE`, `arena::STYLE`, `vox::STYLE`, `dive::STYLE`,
  `debugpanel::STYLE`, `LOADBAR_STYLE`). Text goes through `crate::esc`. Slots of the play screen: `HUD_SLOTS`
  (cleared under a battle and on exit), `DIVE_HIDES`.
- **Materials:** name non-plastic materials with a `NOT_PLASTIC` word (wood, felt, card, cave, paper, rock, sand,
  sign_): `Data::plastic` gives every other new opaque lit material the LEGO plastic look (roughness `PLASTIC_ROUGH`
  0.3, clear coat, subsurface `PLASTIC_SSS`); rougher than 0.85, glowing, metal, water, unlit or blended stay as
  they are.

## Modules
| module | what it does |
|---|---|
| `lib.rs` | `main`, `Data` (shared by all screens: save, sound, settings, town assets, card faces, card UI, battle art, the battle transition, the card game's seed), `Title`, `setup_look` (sky, fog, sun, shadows, plastic, solid fade, day cycle: `DAY_MINUTES` 6, `START_HOUR`), `day_colours`, `phone_quality`, `loadbar` (asset bar + plastic pass), `lcg`, `esc` |
| `play.rs` | `PlayScreen`: the village, you (character controller + `Figure`), the villagers (`Npc`, kinematic capsules, markers), the top-down camera (`CAM_OFFSET`; phones: fixed, never turns; a PC turns and tilts it with the left or right mouse button, `cam_free` / `CAM_TURN` / `CAM_PITCH`, the keys walk the way it looks; tilted below `CAM_LOW` everything far is drawn; `far_view` pushes the fog back and divides `lod_distance_scale` and raises `view_distance`), talking, doors and rooms, indoor light, the colour grade (`set_grade`, `DAY` / `NIGHT`, `NIGHT_FILL`), the pond and the dive hand-off, the gate, the HUD (help line, goal, prompt, toast, title, studs, markers, touch buttons), battles in and out, the debug commands |
| `town.rs` | the town built in code (chunks of bricks per colour, instanced studs, glTF props, roads, the pond, lamps, clouds, tall-grass tufts), `TownAssets` (meshes / materials made once, `deferred` Stonecrest models via `load_deferred`), `NPCS`, `GRASSES`, `ROADS`, `POND`, `LAMPS`, zones (`zone_at`, `ZONE_NAMES`), layers (`layer_at`, `bands_near`, `all_bands`), the gate spots |
| `rooms.rs` (`town::rooms`) | the interiors far east (x >= 187): `RoomId`, `ROOMS`, `room_def`, `area_at`, `door_at`, `ROOM_NPCS` (Mom, Lumi, the lab Prof), `room_lights`, `MACHINE_SCREEN`, layers |
| `north.rs` | `Town::build_north`: Route 1 (trail, creek + bridge, three grass patches, forest with LODs `FOREST_LOD`), Stonecrest (roads, six houses, the fountain's stone, castle), signs, walls |
| `figure.rs` | minifigs from six glTF parts, the walk cycle, `Pose` (Sit, Dance, ArmsUp, Crouch, Wave), `spin`, `air`, `head_want`, talking mouths (`want_mouths`, `set_mouth`) |
| `talk.rs` | `TARGETS` (everything usable: talk / read / TV / piano / PC / lab / bed / suit / feed / sit / chest), `reach`, `goal_text`, `marker_big`, `readable` signs, `CamZoom` + `Board` (the camera eases to a face / sign / screen and back: eye on a line, look point sliding, `ZOOM_S`, `ZOOM_FOV`), `face_board`, `sign_board` |
| `dialog.rs` | `DialogState` (box, lines, voices, choices), `DialogId`, `dialog_for(id, save)` (all Kaigen dialogues by progress), `DialogAction` (handled in `PlayScreen::dialog_action`) |
| `screens.rs` | TV (four channels), your PC ("BrickOS"), the lab terminal, the piano: HTML / SVG over the projected 3D screen (a panel on an upright phone) |
| `life.rs` | `Life`: the engine-free simulation (golden studs, ducks + feeding, fish, butterflies, fireflies, ball + goal, chest, rain + puddles, bench, dancing, Pip: walk, hide and seek, tag, football); `LifeView`: its entities, lamp glow, rain loop (`Bus::Ambience`) |
| `grass.rs` | the tall grass bends away from walkers on the GPU (`Material::bend` = `TUFT_BEND`, `ctx.renderer.set_pushers`: the nearest 16 walkers + your wake), the engine's wind with a slow gust (`WIND_STRENGTH`); tuft transforms are never written (don't `batch_static` the tufts) |
| `fountain.rs` | Stonecrest's fountain water: engine water in the basin and bowl, jet, streams, droplets, spray (seeded particles on the square's band layer), ripple rings |
| `fade.rs` | `Fader`: buildings / trees between the camera and you fade (`MeshRenderer.opacity`; drawn as their front surface with `solid_fade`) |
| `island.rs` | the island north of Stonecrest (OPEN_WORLD.md): the height law `natural(x, z)` (coast as a signed distance of capsules between region centres + bays + noise warp, hills, ridged mountains in the north, mesas with cliffs, warmth, forest), `Island::bake` (the lattice of `CELL` 4 studs), `Island::cell` (one stud: lattice + terraces + the flat `PADS` with blend rings + the graded `ROADS` and their valley, rounded to plates; its colour), `PALETTE`, the north gate's spot (`GATE_X`), self-tests (debug builds) |
| `islandview.rs` | `IslandView` (in `PlayScreen::isle`): the island's 64-stud chunks streamed around you within `budget_ms` per frame (fine: 1-stud cells, colliders; coarse: 4-stud cells, kept until the fine one exists), greedy tops with the baseplate's stud texture per colour, risers as bricks (±3 %, rock strata on big drops) from a palette texture, skirts at chunk borders, real studs within 12 studs (10 on phones), deep water as a wall, the sea (`ocean_mesh`, 170 m, `Ocean` follows the camera) |
| `worldmap.rs` | the world map: painted once from the lattice (biome tint, hill-shade, inked coast, roads, the strip) on a canvas -> PNG data URL; names, "?" on unfound pads and your arrow as HTML (`map`, `map_me` slots, `worldmap::STYLE`) |
| `vox.rs` | your Voxelmon (follows you), wild fights in the tall grass (encounters, skills, HP, projectiles, the fight camera made main, rewards), the upgrade showcase, the HP pill; models `assets/beasts/<card id>.glb` |
| `dive.rs` / `cave.rs` | the lab quest: the suit, sinking into the pond, swimming in the cave (own camera, fog, light; touch: hold / steer / pinch), pearls, the Stud Core; `cave.rs` builds the static cave (basin, tunnel `TUN_*`, deep grotto `DEEP_*`, `Zone`) far east on layer 8 |
| `cards.rs` | card data (`CARDS`, `card(c)`, `by_id`, kinds, `ZONES`, `DECKS`, `STARTER`), stages and upgrades, EXP, rewards, the collection in the save, pictures from `src/beasts.bin` |
| `cardface.rs` | card faces drawn on a 2D canvas -> textures (`CardFaces::texture(ctx, Face)`, cached) and PNG URLs; compact / big / back / unknown / power variants |
| `deck.rs` | `CardUi`: the deck screen (collection, deck strip, replace chooser), foil packs (`show_rewards`; a revealed card tapped shows big at once: `reward_big`), the upgrade moment |
| `battle.rs` | the card battle's rules, engine-free: `Battle::new(opponent, my_deck, ai_deck, seed)`, `update(dt)`, `stage`, `end_turn`, `plan`, `leader`; `BattleKind`, taunts |
| `arena.rs` | `BattleScreen` (scene "battle", pushed over the village): the 3D table, slab cards, tiles, HUD, input, effects, the result (one shadow cascade reaching just past the table, no bloom / depth darkening; phones at full resolution without stud relief: `enter`, `fit_camera`; a tapped card shows big at once (no zoom); a card dragged by a finger rides above it (`drag_point`), a dragged card docks - sinks, clicks - where a let-go lays it: `landing`); the brick transition (`start_transition`, `transition_update`, `TRANS_BOARD`) |
| `table.rs` | `TableArt`: the battle's meshes and materials (slabs, tiles, number atlas badges, shards, table, glows) |
| `sound.rs` | synthesized effects (`Sfx`), music (`Track`, crossfades), voice lines (`assets/voices/<id>.ogg` on `Bus::Voice`), `voice_mouth` |
| `settings.rs` | the ☰ menu (voices, music, Graphics Auto / High / Medium / Low, controls on phones, debug panel, title screen), Auto's frame-rate watch, `apply_step`, `native_resolution` |
| `places.rs` | the island's towns: `Places` (built near, kept), Brickton / Port Saltwick / Dewmere plans, `tower`, `shop`, `mall`, `cottage`, `pier`, `boat`, `lighthouse` |
| `save.rs` | `GameSave` (serde, every field defaulted) in localStorage, written 0.5 s after `mark()` |
| `swipe.rs` | swipe walking on phones (`touch_mode` 0): keeps walking a direction around obstacles, a tap stops |
| `debugpanel.rs` | FPS, the picker, Copy buttons; shown with the engine's debug mode (`) or the menu row |

## Flow and hooks
- **Screens:** `Title` -> `PlayScreen` ("play"). A battle: `PlayScreen::start_battle` -> `arena::start_transition`
  (bricks + fanfare); at `TRANS_BOARD` the play screen pushes `BattleScreen` (its camera made main, stick off). When
  it pops, `PlayScreen::resume` -> `battle_done` pays out (studs, EXP, reward packs, `rival_wins` / `pip_wins`) and
  opens the after-dialogue. `BattleScreen::pauses_below` is false. Call `data.loadbar(ctx)` every frame from any
  new screen.
- **LOD:** props load with the engine's auto-LOD (`gltf_defaults.auto_lod`), minifig parts never (figure.rs).
  `lod_distance_scale` multiplies the switch distances (bigger = more detail): desktop 1.0, medium 0.75, phones 0.6; a
  low PC camera divides it by up to 2.
- **Round the strip:** the island's lattice reaches south past the village (`island::Z1`): meadows a plate below the
  plates (`strip_sd`, `STRIP_LAND`), a beach, the sea; the plates and the rooms far east are holes (`island::hole`,
  no tops, sides or colliders; a coarse cell only when all of it is). No table under the plates any more. The island
  view is active everywhere outdoors (not in the rooms), so the bands outside include layers 11 and 13.
- **Studs on the ground:** the engine's analytic stud relief (`StudRelief::lego(1.0)`, one stud per world UV) on the
  baseplate, the trail and every island top; the stud maps still show far away; phones use `stud_relief = 1` (3x3).
- **Start screen:** `Title` shows assets/title/title_wide.webp (landscape) / title_tall.webp (portrait), the title and
  Play over the sky; music `Track::Title` (assets/music/title.ogg).
- **Pond cave additions:** the user's LEGO coral (`cave::lego_coral`: base plates with studs, the green branching
  element swept along Catmull-Rom curves with closed round tips, four small companions) east of the hole; at the
  grotto's far end a green pipe (`PIPE_AT`, Super Mario Land), an open shaft you swim down through the grotto's
  floor into the core room under it (`CR_*`, the pipe hanging from its ceiling to `PIPE2_BOTTOM`), where the Stud
  Core lies (`DEEP_CORE`); `Zone::Pipe` keeps you in the shaft, its rims and body push you off outside. Every cave
  brick grows and shifts by its own 1..6 cm (`Build::bx`), so no two faces share a plane.
- **Character generator and townspeople:** `figure::FigSpec` (head / torso+arms / legs from the eight styles,
  clothes and hair tinted from `CLOTH` / `HAIR`, `None` keeps a part's colour; `FigSpec::random(seed)`;
  `Figure::spawn_spec`, `Figure::tint` once the parts loaded, `TintCache`). Debug `minifig {seed | spec, at}`. Each
  town's people walk their ways (`people_ways`: Brickton's sidewalks and plaza, Saltwick's street and pier, across
  Dewmere's green), resting at the ends; drawn and moved only within `PEOPLE_R` 90 studs. Test: people.
- **The day:** follows the clock by default (`GameSave.day_fast` false: a day lasts 1440 real minutes and starts at
  the local time, `local_hours`); the menu's DAY row (Clock / 6 min) switches to the quick day (`day_minutes`). The
  bench's fast-forward and sleeping work as before.
- **Lab and home after the key art:** the lab with battlements, a ribbed glass dome, corner turrets (antenna), the
  dish at the front corner, an entrance canopy; your home with facing bricks in shades (`Town::facing`, exact boxes),
  wooden window frames (`window_c`), flower boxes, a blue door canopy and a hip roof. The football goal's top net.
- **Card battle look:** card faces 480 x 672 (`cardface::FACE_W/H`); the camera 65 % of the way to straight down
  (76.7 / 80.5 degrees), fitted to the three locations' width (landscape: the rows may reach under the bars); the
  portraits are LEGO faces (assets/portraits/<style>_face_<0|1|2>.png, the face prints cropped by
  tools/portraits/faces.py), the opponent's
  mouth print following the voice (`foe_mouth`, hysteresis as figure.rs).
- **Island towns** (`places.rs`): built on their pads the first time you come within 330 studs, then kept: Brickton
  (towers taller toward the middle, the mall north-east of the plaza, the TV tower `tv_tower` north-west of it, shops /
  offices south), Port Saltwick (two rows
  of houses, pier, boat, the striped `lighthouse` after the user's picture), Dewmere (cottages round a green, pond, trees). Houses (`house`, `HouseStyle`)
  in the Brick Ages manner: running-bond brick walls with openings, framed windows with shutters and flower boxes,
  doors with frame / step / lamp, stepped gable or hip roofs, chimneys, Dewmere's with picket-fenced gardens. The fishing cutter (`boat`, 47 bricks after the user's model, `BOAT_K`
  4.5 studs per unit) at the pier's end; Dewmere's firs (`fir_parts`: trunk, five tiers of a core and bevelled
  branch bricks, a hexagonal tip; meshes made once, placed as instances). Each building its own entity
  (fades, `Fader::add`) and a box collider; the pads' middles stay free. ~21k triangles for all three. Test: towns.
- **Map travel:** a click on the open map's picture (`hook_map_click`, a DOM click handler: the UI's actions carry no
  position) travels there (`map_travel` / `map_target`): the island once the gate is open (a town's pad: its middle),
  never the sea; on the strip the nearest safe spot (the village plate: the start). Debug `island {map_click: [x, z]}`.
  Test: map_travel.
- **Island music:** `island::music_at` picks the place's track (`sound::ISLE_MUSIC`: a town's on its pad + 20 studs, the
  north, cliffs, warm beaches, the night sea west / east, meadows; day / night); `PlayScreen::outdoor_music` switches
  once a place's track held 2 s (`MUSIC_HOLD`); `Track::Home` means the outdoor music where you are
  (`Sound::set_outdoor`), so a fight hands back to it. Test: isle_music.
- **No auto-LOD** for the Voxelmon (vox.rs: a simplified voxel model has holes) and the lab machine signs.
- **Far view:** trees (`town::is_tree`) are also engine impostors (`prop_opts`: one baked card beyond
  `TREE_IMPOSTOR` 30 x their radius, never nearer than `TREE_IMPOSTOR_MIN_M` 60 m, 8 views, 128 px); the village's bricks are batched under "town bricks" with an
  HLOD per 24 m cell (`HLOD_CELL`, beyond `HLOD_DISTANCE` 60 m, clustered at a stud; `town.hlods`). `view_distance`
  120 m at the fixed tilt .. 260 m at the lowest PC tilt (`VIEW_NEAR` / `VIEW_FAR`, faded into the haze). Village,
  PC camera lowest: 656k -> 197k triangles; the top-down views (PC and phone) show no cards.
- **Save:** the ☰ menu's SAVE GAME row exports the whole `GameSave` as a JSON download and imports one through the
  file picker (`savefile.rs`: refused unless it is a save; applied, written, page reloaded).
  Add fields to `save::GameSave` (serde default), change them, `data.save.mark()`. `save_reset` keeps
  `intro_seen` true (tests start in the village) and puts you at the start.
- **Where you are:** `GameSave.pos` [x, z studs, yaw], kept by `PlayScreen::keep_place` every 2 s while you stand
  safely (grounded; no dialogue, fight, dive, door, zoom), written at once when the page is hidden; a reload starts
  there (village or island). A new game has none (the intro in your room). Tests that need the start call `save_reset`.
- **Dialogues:** `self.dlg.open(ctx, &mut data.sound, dialog_for(id, &save))`; a closing choice arrives in
  `PlayScreen::dialog_action` (the last one is `pending_action` in game_state).
- **Something to use:** add a `talk::TARGETS` entry (and a `Use` variant + a case in `PlayScreen::use_target`);
  `pick_target` picks the nearest in `talk::reach`. Moving targets (Pip, the chest, the gate Prof) are special-cased
  in `pick_target`.
- **Villagers:** `town::NPCS` (outside) and `rooms::ROOM_NPCS`; `PlayScreen::npcs` ids `rival`, `kid` (Pip),
  `gate-prof`, `knight`, `baker`, `mom`, `lumi`, `lab-prof`. They block you (`npc_stop`: the wish into one within
  0.76 m is dropped), turn their heads (`animate_villagers`), and the one you talk to turns to you.
- **Moving the player:** `PlayScreen::move_to(x, z, yaw)` (camera snaps along); a faded move through a door:
  `self.door = Some((0.0, to, yaw))`.
- **Camera close-ups:** a `talk::Board` for `self.zoom` (`CamZoom::update` with the board each frame; `None` eases back).
- **Music / effects:** `data.sound.music(ctx, Track::Wild)`; `data.sound.sfx(ctx, Sfx::Fanfare, pitch, volume)`.
  `Track::Intro` hands over to `Fight` after 10.8 s (`Data::track_music`).
- **Light:** indoors `PlayScreen::indoor_light` eases `sun_scale` / `ambient_scale` (`INDOOR_SUN`, `INDOOR_SKY`);
  room lamps `ROOM_LAMP`; street lamps 8 x night. The dive owns fog / sun / ambient while `dive.on` and gives them
  back in `Dive::stop`. Rain haze: `life_frame` scales `fog_density` / `fog_start`.
- **Grass pushers:** add walkers in `PlayScreen::grass_pushers` (feet in metres, radius).
- **Particles:** every emitter on a render layer (outside: `town::layer_at`; battle: `arena::LAYER`).
- **Island:** `Data::island` (the baked law, `Rc`, once per game), `PlayScreen::isle.update` every frame before
  `outdoor_zone`; `teleport` north of z -194 builds the chunk under you at once and puts you on its ground
  (`Island::ground_y`). The north gate: `north_gate` (doors, the blocker, walking into it closed -> the guard's
  `DialogId::NorthGate`; `DialogAction::OpenNorthGate`), save `north_gate`; pads found by walking onto them
  (`discover`, save `isle_found` bits). The map: M / the menu's Map row (`map`), M / Esc / a tap (`map_close`) close it
  (`set_map`); it makes you busy.
- **Phones:** `Data::ensure_loaded` detects a phone (coarse pointer, or touch points + a mobile UA) for the phone
  profile from the first frame (`phone_quality`: 2 cascades, half resolution, 20 ms budget, no screen-space
  extras). No frame cap: the engine's frame pacing picks the cadence (`stats.render.pacing`). `ctx.renderer.prewarm()`
  runs once the village is loaded. The play camera never turns on phones (`sensitivity` 0, yaw reset every step); on a PC it turns and tilts with the left or right mouse button held (`cam_free` = not `Settings::phone`, known at the start; fingers never turn it): the keys' axis is rotated into the world, the island chunk rectangles and the strip's bands reach as far in every direction, island chunks get their north-facing risers (`islandview::set_turning`). `PlayScreen::talking()` hides the
  touch buttons and the stick.
- **Page:** `web/index.html` replaces the engine's page (no text selection / callout, a brick loading bar for the
  wasm download): keep the engine's page changes in sync by hand.

## Debug commands (`bridge call help` lists them with their args)
- Play screen: `teleport {x, z}` (studs), `clock {hours}`, `talk {dialog?}`, `life {freeze, pip_home, rain, chest,
  chest_at, seek, tag, ball, dance, sit, fish, golden, ...}`, `debug_pick {x, y | copy | menu}`, `save_reset`, `save_export` -> {json}, `save_import {json}`, `intro`,
  `enter {room}`, `flags {any GameSave field}`, `play_sfx {name, pitch}`, `cards {starter, give, exp, level, deck,
  rewards, open, pick}`, `card_face {id, level, big, unknown, back, power, texture}`, `battle {kind, seed, instant,
  start}`, `look {at, yaw, w, h, tilt}` / `look {}`, `tall_grass {wind}`, `vox {force, shiny, skill, run, hp, tired,
  wild_idle, rate, seed}`, `dive {sink, start, at, yaw, pitch, skip_fade, surface}`, `island {gate: open|closed, map,
  budget, found, cell: [x, z], line: [x0, z0, x1, z1, n], reset_stats}`, `card_preview {id, level, big,
  back} | {clear}`.
- Battle screen: `battle_play {card|hand, zone}`, `battle_end_turn`, `battle_auto {end}`, `battle_ui {action}`,
  `battle_inspect {card|zone|close}`, `battle_speed {x}`, `battle_cheat {power, foe}`, `battle_continue`.
- `game_state.state` (play screen) has the hero, npcs (head, pose), area, door, title, zone, bands, indoor,
  camera_layers, island (law: bake_ms, pads, selftest; view: chunks, near tris, studs, colliders, build times; gate;
  map; found; ground_gap), machine, reading, screen, sleeps, mouths, life, grass, fountain, vox, fight, dive, cards, battle
  transition, battles, save fields, swipe, debug_panel, npc_stops, respawns, gate_stops, rain_volume, menu, and the
  last `pending_action`; the battle screen's `state.battle` has the table (phase, hand, locations, rank, ...).
- Kaigen's dev keys map to commands: F2 `dive {at}`, F3 `flags`, F6..F9 `vox {force}`, `cards {exp}`, `battle`,
  `cards {rewards}`.

## Tests (`tests/*.txt`)
Run one: `node C:/dev/wasmOnly/tools/bridge.mjs run tests/<name>.txt --port 9223 --url http://localhost:8091/`; all:
`for t in tests/*.txt; do node C:/dev/wasmOnly/tools/bridge.mjs run "$t" --port 9223 --url http://localhost:8091/; done`
(check `"passed": true`; "Inspected target navigated or closed" / "engine not ready" means a rebuild landed: rerun).
Drive cards with touch input (`input {touch}`), not the mouse.

| test | covers |
|---|---|
| camera_turn | the PC camera: the mouse alone doesn't turn it, left / right button drags turn and tilt it, W walks the way it looks |
| savepos | the place survives a reload (village and island), a new game has none |
| savefile | the menu's Export / Import row, export JSON, an import persists over a reload, non-saves refused |
| smoke, town | the village loads, walking, jumping, collision, studs, LODs (props have them: `lods_village > 0`; minifigs never) |
| touch, phone_start, swipe | the stick, Jump, the phone profile from the first frame, swipe walking |
| menu | sound, the ☰ menu, graphics steps |
| dialog, life, village, seek, npc_block, goal | dialogues + voices, Pip and village life, hide and seek / tag, solid villagers, the quest chain |
| rooms, screens, fade | doors, stairs, rooms (`zfight` 0 pairs), TV / PC / piano / signs, see-through occluders |
| cards, battle, battle_dex, battle_touch | starter gift, deck screen, card battles (desktop and upright phone) |
| vox, vox_lose, vox_upgrade, vox_touch | Voxelmon, wild fights, the upgrade showcase |
| dive, dive_touch | the lab quest and the cave |
| north, grass, fountain | Route 1 / Stonecrest, the gate, bending grass, the fountain |
| island, island_touch | the north gate (closed, forced, opened by the guard), the law's self-tests, streaming, ground, terraces, a cliff, the shallows, `zfight` 0 and < 60k triangles at sample spots, the map (desktop and upright phone) |
| debug_panel | the panel and its picker |

## Tools (`tools/`)
- `lan.ps1` + `serve_lan.py` + `phone_log.js`: a release build for a phone in the Wi-Fi. `lan.ps1` builds
  `--release` into `out/lan/` and injects `phone_log.js`; `serve_lan.py [port]` serves it over HTTPS on :8444 (the
  Kaigen port's local CA; gzip with `X-Raw-Length` for the page's bar, .ogg streamed) and writes what the phone sends
  to `logs/phone.log` (console errors, fps + `stats.render.pacing` every 5 s, engine warnings / first-use / prewarm
  lines, a smoothness trace). `https://<pc>:8444/?measure` is the measuring mode: walk 60 s, "measure" lines with
  frame intervals, stats, pacing and traces.
- `smooth.mjs [--port --url --frames --key --from]`: the hero walks along the road while the engine traces its drawn
  position; prints the trace (speed jitter, frames with 0 / 2+ steps) and frame times.
- `smooth_heads.mjs`: `assets/minifig/*/head.glb` smooth shaded (auto smooth at 46 degrees) from `head_flat.glb`
  (the flat-shaded original and the source: change that one and rerun).
- `mouths.py` (Pillow): `head_mouth1.glb` / `head_mouth2.glb` per figure (head_flat.glb with the printed mouth half /
  fully open), used by `Figure::set_mouth`.
- `glb-split-colors.mjs`: vertex-coloured glTFs (the Kaigen beasts / props) -> one primitive and material per colour.
- `make_beasts.py <st_beasts_data.c>`: the card pictures -> `src/beasts.bin`.

## Not ported / differences from Kaigen
- The debug panel's toggle: Kaigen used ` itself; here the engine owns ` (its debug mode, which the panel follows) and
  phones get a menu row. The picker names models by their glTF file and roads / the baseplate; Kaigen's per-building
  tag registry (`town_tag`) does not exist in this town builder.
- Pip hiding has no marker over his head (Kaigen kept drawing it, which gave him away).
- Tag on phones: Kaigen's tag needs Shift (no run on touch); here the stick pushed far also sprints (swipe walking
  cannot run).
- The goal pill hides in the rooms and while talking (as Kaigen); the studs pill sits under the menu button (wide) or
  under the goal (narrow).
- The castle (`castle.glb`): its stone colours are the values the three.js version effectively showed (its vertex
  colours were used as linear), lighter than Kaigen's sRGB-decoded greys.
- Right-mouse camera look / Kaigen's editor (F1, F5) are not part of this port (the camera keeps the three.js fixed
  view).
- Outdoors the full `zfight` check still reports pairs inside the glTF props (trees, bushes); rooms are clean.
