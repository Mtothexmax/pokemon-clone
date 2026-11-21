# Porting the Kaigen version of Brickhollow to wasmOnly

Source: the Kaigen (HZ engine, C) port in `%LOCALAPPDATA%/KaigenProjects/pokemontest`
(`src/starting_template/st_*.c`, assets in `assets/brickhollow/`). The three.js original is this repo
(`C:/dev/pokemonsnap`, branch history `threejs-version`). Goal: everything the Kaigen port does, in this crate.

## Rules for every part
- Never patch the engine (`C:/dev/wasmOnly`). Missing engine features or engine bugs: write them down in the
  report (the main session forwards them to the engine session) and work around them in the game if possible.
- Dev loop: the watcher on http://localhost:8091/ is already running (do not start another one). Debug Chrome:
  `node C:/dev/wasmOnly/tools/bridge.mjs <cmd> --port 9223 --url http://localhost:8091/` (do not launch another
  Chrome, do not touch the servers on ports 8443, 8444, 8780).
- Every feature gets a scripted test in `tests/*.txt`; all tests pass before a commit.
- Commit after each finished part (source, tests, needed assets; never `out/` or `dist/`), never push.
- Phones: every action reachable by touch (`data-action` buttons, `ctx.ui.take_actions()`), keep the frame
  rate (phone profile in `phone_quality`).

## Status
- [x] Village (town.rs), minifigs, NPCs standing, collision, touch stick, day/night sky, phone profile
- [x] Sound: music (home, wild fight, battle intro / fight / victory / defeat), voices, effects; settings menu
      (voices, music, graphics); save (sound.rs, settings.rs, save.rs; tests menu.txt, touch.txt)
- [x] Dialogues with voices, village life (NPC routines, facing who you talk to), goal pill / markers
      (dialog.rs, talk.rs, life.rs, play.rs; Pip's hide and seek / tag / football, the ducks, ball, weather etc.
      of st_life.c come with their parts; tests dialog.txt, life.txt)
- [x] Rooms / interiors with doors (studded furniture, warm light), TV and computer screens, piano, mailbox,
      sign close-ups; the bedroom start with the intro thought, Mom / Lumi / the lab Prof inside, the
      Deck-o-Matic showing MISSING PART (rooms.rs, screens.rs, talk.rs; tests rooms.txt, screens.txt)
- [x] Cards and Voxelmon data, starter gift, foil packs, deck screen (collection, deck), upgrades with the upgrade
      moment, card faces as PNG / 3D texture (cards.rs, cardface.rs, deck.rs; tests cards.txt, screens.txt)
- [x] Card battle in 3D vs Dex, Lumi, Pip: rules (battle.rs), the table / slab cards / tiles / effects / HUD
      (arena.rs, table.rs), brick transition, battle music + taunts, rewards (tests battle.txt, battle_dex.txt,
      battle_touch.txt)
- [x] Wild Voxelmon fights in the tall grass, skills, upgrades, own music, no touch Jump/Use in fights (vox.rs: the
      follower, encounters, the real-time fight with its camera and HUD, rewards, the upgrade showcase in 3D, HP back on
      sleep; beasts `assets/beasts/*.glb`; tests vox.txt, vox_lose.txt, vox_upgrade.txt, vox_touch.txt)
- [x] Lab quest: diving suit, deck machine "missing part", pond dive into the cave (swim in 3D, darker with depth,
      seaweed, glowing plants, collectibles, marine snow, part deep down), deck built on return (dive.rs, cave.rs;
      tests dive.txt, dive_touch.txt)
- [x] Route 1 north and Stonecrest: the gate (Prof. Quill steps aside after the Voxelmon), the trail, the creek and
      bridge, three more grass patches with wild fights, the forest, Stonecrest (roads, six houses, fountain, castle,
      Marla, Sir Bricksworth), the two signs, area titles, walls; band layers + forest LODs (north.rs; test north.txt)
- [x] Tall grass that moves (grass.rs: tufts bend away from you / villagers / Voxelmon on the GPU (`Material::bend`),
      a short wake behind you, wind sway from two far pushers; test grass.txt), Stonecrest's fountain water (fountain.rs: engine water in a hollow basin + bowl, jet, falling
      streams, droplets, spray, ripple rings; test fountain.txt)
- [x] Village life and the last Kaigen pieces (life.rs: golden studs, ducks + feeding, fish, butterflies, fireflies, the ball
      and the goal, the treasure chest, rain + puddles + TV rain, glowing lamps, the park bench, dancing, Pip's hide and seek
      (then his card battle), tag and football; figure poses, head turns, jump pose; R respawn; swipe walking (swipe.rs, menu
      Swipe / Stick); the debug panel (debugpanel.rs); the goal only outside; the night pill was already there; the castle's
      colours; tests village.txt, seek.txt, swipe.txt, debug_panel.txt, goal.txt, npc_block.txt)

## Hooks for later parts (sound / dialog / life slice)
- Save: add fields to `save::GameSave` (serde defaults), call `data.save.mark()`; quest flags are already there.
- Dialogues: `dialog::dialog_for(DialogId, &GameSave)` has every Kaigen dialogue; open with `self.dlg.open(..)`.
  Actions chosen at the end (battles, hide and seek, tag, starter cards, sleep) arrive in
  `PlayScreen::dialog_action`; unhandled ones are logged and kept in `pending_action` (game_state).
- Talk spots: `talk::TARGETS` (indoor villagers already listed; add their figures to `PlayScreen::npcs`).
- Music: `data.sound.music(ctx, Track::Wild)`; effects `data.sound.sfx(ctx, Sfx::Fanfare, 1.0, 0.8)`.
- Camera close-ups (signs, TV): `talk::CamZoom` + `talk::Board`.
- Pip: `life::PipMode` (Hiding / Flee / Football stubs), `Life::toast`, `Life::blocks` (town colliders).
- Debug commands: `talk`, `life {freeze, pip_home}`, `flags {..GameSave fields}`, `save_reset`, `play_sfx`.

## Hooks for later parts (rooms / screens slice)
- Rooms: `town::rooms` (`RoomId`, `room_def`, `area_at`, `door_at`, `ROOM_NPCS`, `MACHINE_SCREEN`). Each area has
  its own render layer (`layer_of`; 0 = the hero, `HIDDEN_LAYER` = drawn by no camera); `PlayScreen::area` is the
  room you are in, `show_area` switches the camera's layers, the ceiling lights and the area title. Things built
  into a room: `Town::rooms_build` (a group per room, `Town::area` tags props). New outdoor zones (Route 1,
  Stonecrest) can set the title in `show_area` (Kaigen `town_zone`; for now always "Brickhollow" outside).
- Moving the player anywhere (a cave, a gate): `PlayScreen::move_to(x, z, yaw)` snaps the camera; a faded
  move: set `PlayScreen::door = Some((0.0, to, yaw))` (debug command `enter {room}` does that).
- Card battles from dialogs: Dex (BattleRival), Lumi (BattlePractice), Pip (BattlePip) still arrive in
  `PlayScreen::dialog_action` -> `pending_action`; Mom / Lumi / the Prof stand in their rooms (`npcs` ids
  `mom`, `lumi`, `lab-prof`), the camera already looks at whom you talk to indoors.
- Lab quest: the Deck-o-Matic's screen follows `has_deck` (`machine_missing.glb` / `machine_ready.glb`, swapped by
  layer in `update`); the Prof's `QuestPart` / `GiveStarter` actions are in `dialog_action`. The diving suit is
  not built yet: Kaigen stands it at (314.2, 0.4, -9.4) studs, yaw 0.6, in the lab corner behind the bookshelves,
  with a target "Take the old diving suit" at (315.6, -7.2) while `!has_suit && !has_deck` (add a `talk::Use`
  variant and a case in `use_target`; set `has_suit`, fanfare, toast "Prof. Quill's old diving suit! ...").
- Screens (`screens.rs`): the Beasts app shows the count only and the lab terminal analyses stand-in specimens
  (`SPECIMENS`, random cost / power) until the cards part brings the card data (Kaigen `card_def`, owned card
  name chips); Stats shows butterflies = 0 until the life part has them; the TV weather gets rain (`weather` 2)
  from the life part's weather. `Sound::voice_active` tells whether a voice line still plays.
- Sleep: `PlayScreen::sleep` (bed, `DialogAction::Sleep`); the Voxelmon part restores its HP there (Kaigen).
- Debug: `intro` (new game's wake-up), `enter {room}`; `save_reset` now leaves `intro_seen` true (tests start in
  the village); `game_state.state` has `area`, `door`, `title`, `indoor`, `camera_layers`, `room_lights_on`,
  `machine`, `hero_hidden`, `screen {kind, channel, line, app, notes, term_lines}`, `reading`, `sleeps`.

## Hooks for later parts (cards slice)
- Card data (`cards.rs`, Kaigen `st_cards.c`): cards are `usize` indices, `cards::card(c)` -> `CardDef {id, name, kind,
  cost, power, text}`, consts `cards::PEBBIT ..`, `by_id("pebbit")`, `kind_info(kind)` (label, colours, glyph),
  `ZONES` + `zone_picture(z)` (`assets/zones/<id>.jpg`, 400x225; battle backgrounds `assets/backgrounds/
  {forest,cave,lab}.png`, 480x270 pixel art, both from the three.js `static/`), `DECKS[DECK_STARTER|RIVAL|RIVAL_2|
  PRACTICE|PIP]`, `STARTER`, stages (`stage(lv)`, `upgrade_cost`, `upgrade`, `max_level`, `VOXEL_LEVEL`),
  `battle_exp(result, rival)`, `reward_card(save, &mut seed, rare)` (gives and returns a missing card), `WILD_EXP`,
  `picture(c)` (the Voxelmon front pixel art, RGBA, from `src/beasts.bin` = Kaigen `st_beasts_data.c` via
  `tools/make_beasts.py`).
- Save: `GameSave::collection` (bit per card; `cards::owns/give_card/owned_count`), `deck: Vec<u8>` (card indices,
  up to `DECK_SIZE`; `cards::in_deck`), `card_level: Vec<u8>` (may be shorter than `CARD_COUNT`: read with
  `cards::level(s, c)`), `exp`, `voxelmon` (CardId + 1), `wins/losses/rival_wins/pip_wins`.
- Card textures (`cardface.rs`): `data.faces.texture(ctx, Face::new(card, level))` -> `Option<TextureId>` (sRGB, mips,
  300x420 = 1 : 1.4, transparent rounded corners, cached per `Face`). Use it as
  `Material { texture: Some(t), ..Material::color(0xffffff).cutout(0.5) }` on `ctx.renderer.builtin.quad` scaled
  (w, 1.4 w, 1) (see the `card_preview` command in play.rs). Variants: `.big()` (art + kind + ability text: the
  detail card), small (ability symbol only; what the hand / board shows), `.unknown()`, `.power(p)` (green / red
  badge), `hide_cost`, `Face::back()` (the card back). Textures are never freed: keep `power` at the card's own
  value and draw battle power changes as separate badges/meshes. `data.faces.rgba(face)` gives the pixels;
  `data.faces.url(face)` a PNG blob URL for the DOM (None for a frame or two while it encodes).
- Card screens (`deck.rs`, `data.cards: CardUi`): `data.cards.show_rewards(title, &[c..3], Some(DialogId::DexWon))`
  shows foil packs (the battle part: after `reward_card`; the dialogue opens when they are closed); `busy()`;
  the deck screen (`deck_open`, C / Deck button / `cards {open}`), the upgrade moment (`up`; a Voxelmon card
  bursts and the screen opens onto the town at `UP_BURST + 0.3` s: the Voxelmon part adds the 3D beast rising there,
  Kaigen `vox_showcase`; "Walk with me" sets `voxelmon`).
- Starter gift: `PlayScreen::give_starter` (DialogAction::GiveStarter; packs Titanoak / Thundrake / Emberhorn, then
  DialogId::ProfAfter). The legendary Goldbrick is given by `CardUi::update` once `legendary` is set.
- Screens: the Beasts app shows owned name chips; the lab terminal analyses a random pool card.
- Debug: `cards {starter, give, exp, level, deck, rewards, open, pick}`, `card_face {id, level, big, texture}`,
  `card_preview {id, level}`; `game_state.state.cards` (owned, collection, deck, exp, levels, deck_open, pick,
  upgrade, rewards {cards, tear, revealed}, faces {drawn, urls, textures}).
- Step 0: voices play on `Bus::Voice` (menu Voices = that bus's volume); Graphics High stores MSAA 4 for the next
  start (`engine::set_msaa_pref`), other choices clear it (`menu.msaa_pref` in game_state).

## Hooks for later parts (card battle slice)
- Start a battle like the dialogues do: `arena::start_transition(ctx, data, BattleKind::Pip)` from the play screen
  (`PlayScreen::start_battle`): the bricks slide in with the intro fanfare and at `arena::TRANS_BOARD` the play screen
  pushes `BattleScreen` (scene "battle"; the village stays loaded below with its camera; the battle camera is made main
  with `ctx.set_main_camera` and the joystick is off (`set_stick_enabled(false)`) until the battle pops, then the
  village camera is main again). When it pops, `PlayScreen::resume` -> `battle_done` pays out (studs, EXP,
  `cards::reward_card` packs, `rival_wins` / `pip_wins`) and opens the after-dialogue (`DexWon`, `PipLost`, ...).
- Hide and seek -> Pip's battle: `DialogAction::BattlePip` already starts it once `seek_played` or `seek_wins` is set
  (else `PipNotYet`). The hide-and-seek part only has to count its rounds into the save (`seek_played += 1`, a win
  `seek_wins += 1`) and then either let Pip's dialogue offer "Card battle!" or call `start_battle(ctx, data,
  BattleKind::Pip)` itself at the end of a round (Kaigen: Pip asks for a battle after hide and seek). Pip's deck is
  `DECK_PIP` with +1 Power on every card (`BattleScreen::build`).
- Wild Voxelmon fights (the Voxelmon part) can reuse `BattleKind::Wild` (forest backdrop, "*snarl*" taunts, no
  voice) or keep their own screen; `battle::Battle` is engine-free (`Battle::new(kind, name, my_deck, ai_deck,
  seed)`, `update(dt)`, `stage`, `end_turn`, `plan(side)`), shiny battles would add the Kaigen `shiny` flag.
- Music: `data.sound.music(ctx, Track::Intro)` hands over to `Track::Fight` after 10.8 s (`Data::track_music`), the
  battle plays `Victory` / `Defeat`, `battle_done` goes back `Home`. `data.cg_seed` is the card game's generator
  (battle seeds, reward cards).
- 3D pieces in `data.art` (`table::TableArt`): `slab` card / tile meshes, `face_mat(tex)`, `number(style, v)` (atlas
  badge quads), the location tile textures (`zone_tex`, `hidden_tex`), the backdrops. Board layout and camera fit:
  `arena::Lay` (`portrait` = phone held upright: columns 1.2 m, steeper camera).
- Debug: play screen `battle {kind: rival|practice|pip, seed?, instant?, start?: false}`, game_state
  `battle_transition`, `battles`, `save {studs, exp, wins, losses, rival_wins, pip_wins}`; battle screen
  `battle_play {card|hand, zone}`, `battle_end_turn`, `battle_auto {end}`, `battle_ui {action}`,
  `battle_inspect {card|zone|close}`, `battle_speed {x}`, `battle_cheat {power, foe}`, `battle_continue`;
  `game_state.state.battle` = phase, turn, energy(_left), stakes, first, result, hand [{id, cost, playable, screen}],
  locations [{id, revealed, power [you, foe], winner, mine / theirs [{id, power, revealed, staged, y, flip, anim}],
  screen}], selected, inspect, dragging, drag_over, taunt, banner, flips, lands, balls, shatters, confetti, camera,
  layout, track, voice, last_sfx. HUD actions: `bt_end`, `bt_undo`, `bt_snap`, `bt_retreat`, `bt_log`, `bt_play:z`,
  `bt_take_back`, `bt_activate`, `bt_cancel_act`, `bt_close`, `bt_continue`.
- Tests: drive cards with touch input (`input {touch}`), not the mouse: the real mouse over the debug window moves the
  pointer between frames.

## Hooks for later parts (Voxelmon slice)
- `vox.rs` (`PlayScreen::vox`, Kaigen `st_vox.c`): the fight stays in the village (as Kaigen, no screen of its own).
  `Vox::update` every frame returns whether a fight holds you (`PlayScreen::fight`: no Jump / Use buttons, no
  joystick (`set_stick_enabled(false)`), no Deck button, goal, help line, area title, Esc = Run); `walk` / `face_yaw`
  steer you back from the arena in `fixed_update`. The fight camera is its own entity made main with
  `ctx.set_main_camera` and despawned afterwards (the village camera keeps following you, the way back ends where it is).
- Beast models: `assets/beasts/<card id>.glb` (Kaigen's, split per colour by `tools/glb-split-colors.mjs`; 1 unit high,
  feet at 0, facing +z). `Vox::model(ctx, card)` loads one (hidden on `HIDDEN_LAYER`, shown on layer 0); other parts
  (Pip's Voxelmon, the dive) can reuse `show_model`.
- Tall grass: `town::GRASSES` (all four patches; only patch 0 is built and walkable yet). Route 1 (next part) builds the
  other three and the land north of patch 0: the fight camera looks from the side away from you, at patch 0 that is
  north, where the map still ends (the edge shows at the bottom of the view until then).
- Debug: `vox {force: true|'id', shiny, skill: 0|1, run, hp: {me, wild}, tired, wild_idle, rate, seed}`;
  `game_state.state.vox` (phase, me / wild {id, hp, max, tired, shiny, x, y, z, act, shown, size, children}, skills,
  cool, result, center, walk, meter, rate, cam_on, cam_t, projectiles, numbers, show, fights, skills_used, hits,
  rustles), `fight`, `stick`; `save.vox_wins`, `save.voxelmon`. HUD slots `vox_wild`, `vox_mine` (actions
  `vox_skill:0|1`, `vox_run`), `vox_pill`.
- The gate (Prof. Quill steps aside once you have a Voxelmon): `save.voxelmon > 0`, as the dialogues already check.

## Hooks for later parts (lab quest / dive slice)
- `dive.rs` (`PlayScreen::dive`, Kaigen `st_dive.c`) and `cave.rs` (Kaigen `st_cave.c`): the cave is built once per play
  screen far east (x 600..680 studs, y -30..100) on its own render layer `cave::CAVE_LAYER` (8); the dive has its own camera
  (`ctx.set_main_camera`, layer 8 only, 105 degrees across) and owns fog / sun_scale / ambient_scale / shadows while
  `dive.on` (given back in `Dive::stop`). While diving `PlayScreen::update` returns early after the dive (no dialogs, menu,
  vox, doors); only toasts stay. Cave coordinates: studs from (`CAVE_X`, 0, `CAVE_Z`), `cave::world(c)` -> metres,
  `cave::floor(x, z)`, the diver's solids `Dive::solids`.
- The pond: `PlayScreen::pond` (jump onto it: `dive::can_dive` -> sink 0.9 s -> `Dive::start`; else back on the nearest
  bank, `climb_outs`); after a dive you stand at `POND_EXIT` (-5.5, 13.6; Kaigen -8 is Pip's home spot). Pip's football /
  the ducks (life part) must keep out of `town::POND` as before.
- The suit: `talk::Use::Suit` (`SUIT_TARGET`, skipped by `pick_target` unless `dive::suit_here`), the model in the lab
  (`Dive::lab` shows / hides it). Save: `has_suit`, `has_part`, `dive_found` (pearl bits, 8 pearls, +5 studs each).
- Kaigen's F2 (diver to the trench) is the debug command `dive {at: [0, -19, 38]}`; `dive {sink | start | at | yaw | pitch |
  skip_fade | surface}`; `game_state.state.dive` (on, t, fade, leaving, x/y/z, yaw, pitch, speed, az, fast, depth, metres,
  touch, pinch, squeeze, pearls, pearls_now, core_taken, dives, fov, cam_main, lights_on, fog, sun_scale, ambient,
  suit_shown, cave_tris, solids, near_core), `sink`, `pond_cd`, `climb_outs`, `save {has_suit, has_part, dive_found}`.
  HUD slots `dive` (tint, edge, pill, hint, Surface button = action `dive_surface`, fade) and `dive_fx` (helmet bubbles).
- The hide-and-seek / debug-panel part: the Kaigen debug panel's "dive" entries map to the `dive` command above.

## Hooks for later parts (Route 1 / Stonecrest slice)
- `north.rs` (`Town::build_north`, Kaigen `st_north.c`): everything north of the village plate (z < -47 studs). Consts in
  `town.rs`: `NORTH_PLATE`, `CREEK`, `BRIDGE`, `TRAIL`, `GRASSES` (all four built; `vox::in_grass` finds the patch, the fight
  stays in it), `ROADS[7..]` (Stonecrest's streets, `Town::road(i)`), `zone_at(z)` / `ZONE_NAMES` (0 Brickhollow, 1 Route 1,
  2 Stonecrest, three.js `zoneLabel`), `on_road`.
- Render layers: the north is cut into bands (`town::NORTH_BANDS`: layers 6, 7, 9, 10; the village stays on 1). `layer_at(x, z)`
  gives the layer of anything outside (chunks, studs, props, buildings, NPC figures use it). The play camera draws
  `1 | town::bands_near(z, 30, 40)` (`PlayScreen::outdoor_zone`, every frame) and `town::all_bands()` while the camera looks far
  (a close-up at a face or sign, a fight). New things north must go on `layer_at` too, or they show everywhere / nowhere.
  Layers 8 (cave), 31 (hidden), 2..5 (rooms) as before; 11+ are free.
- Forest LODs: `Prop::lod` > 0 loads the glTF with `GltfOpts { auto_lod: true, lod_distance }` (only the trees behind the route's
  and the town's walls, `FOREST_LOD` 0.2); everything in the village stays at full detail (`lods_village == 0` in town.txt).
- The gate: `PlayScreen::gate` (Kaigen app_update): while `!gate_open` walking into the gap (z -25.3..-27.5, |x| < 5) moves you
  back to z -22.6 and opens `DialogId::Gate` (not during a fight or while busy); `DialogAction::OpenGate` sets `gate_open`,
  then Prof. Quill walks to `town::GATE_PROF_OPEN` (`gate_walk`, fixed step); her talk spot `talk::GATE_TARGET` follows.
  Tests that teleport into patch 0 with a Voxelmon set `gate_open: true` (a fight's step back can lead into the gap).
- NPCs: `town::NPCS` has `knight` (Sir Bricksworth, on the square's plate: y 0.4 via `on_road`) and `baker` (Marla), npcs
  indices 3 and 4 (the room villagers moved to 5..7). Their lines are `DialogId::Knight` / `Baker` (unvoiced, as three.js),
  targets 22 / 23; the signs are readables 5 (route sign, `north.glb`) and 6 (town sign, `stonecrest.glb`), targets 20 / 21,
  with `sign_board` close-ups. The houses' doors lead nowhere yet (no rooms); Pip's hide and seek can use `town::GRASSES` and
  the forest scatter for spots.
- Titles: walking between zones shows "Route 1" / "Welcome to Stonecrest!" / "Back in Brickhollow"; the help line's heading is
  the zone (`PlayScreen::help`).
- Debug: `game_state.state` `zone`, `bands` (the outdoor layer mask drawn), `gate_stops`, `lods_village`; `save_reset` puts
  Prof. Quill back at once. The LEGO clouds (`Town::cloud`) hang beside and beyond the
  diorama now (x -98..-90 / 118..125, z -252), at the camera's height: the eye-level views see them, the top-down camera
  never has them under it (they used to hang over Route 1 below the camera: pale slabs and their shadows).

## Hooks for later parts (tall grass / fountain / cleanup slice)
- Tall grass (`grass.rs`, `PlayScreen::grass`): every tuft's blades are an instance of one mesh (`town::tuft_mesh`, origin at
  its foot; the round plate stays in the static bricks) with a bending material per green (`TownAssets::tuft_material`,
  `Material::bend` = `grass::TUFT_BEND`); the transforms never change (no per-frame upload, the tufts are static shadow
  casters, their shadows stay upright). Walkers are set every fixed step (`Grass::set_pushers(feet + radius m, dt)`: the
  hero first, outdoor NPCs, `Vox::pushers`; add Pip's spots or the ducks there); `Grass::update` hands the nearest 16 plus
  your wake (3 fading points) to `ctx.renderer.set_pushers` every frame (none indoors; cleared on exit) and sets the
  engine's wind (`renderer.grass_settings().wind = [dir x, dir z, strength]`, `grass::WIND_STRENGTH` 0.3 with a slow
  gust; engine 6401217 sways bend geometry with it). Don't `batch_static` the tufts.
  Debug: `tall_grass {wind}`, engine `bend` (pushers, bend materials); `game_state.state.grass` (tufts, patches, bent,
  max_bend, near [[dist m, lean rad]..] = the shader's lean recomputed from the walkers, pushers, wake, sent, wind,
  wind_strength, bend, writes = 0).
- Fountain (`fountain.rs`, `PlayScreen::fountain`): consts `town::FOUNTAIN`, `FOUNTAIN_IN`, `FOUNTAIN_WATER`, `BOWL_R`,
  `BOWL_WATER`, `NOZZLE_TOP` (studs); the stone in `north.rs` `fountain`. Particles are spawned with fixed seeds (`emitter`,
  `ctx.spawn_particles_seeded`) on the square's band layer (`ParticleEmitter::layer`): cameras without that band (village,
  rooms, dive, battle table) do not draw them, nothing switches them on/off; `set_visible` only stops the ripple rings. `game_state.state.fountain`.
  The village pond already was engine water on a grid: left as it was.
- Camera close-ups anywhere (debug): `look {at: [x, y, z] studs, yaw, w, h, tilt}`, `look {}` back (uses `talk::CamZoom`).
- Rooms: the home stairs are real steps with one box collider each (engine 3edabf4 climbs them; `Town::ramps` is gone);
  rooms.txt walks halfway up and down again. The lab's checker floor is one slab + inlays (no coplanar tile tops), the
  rooms' dark surrounds sit below 0, each room's a little lower.
- Indoor light: `PlayScreen::indoor_light` eases `settings.sun_scale` / `ambient_scale` (applied after the day cycle; the
  sky outside is untouched); the dive owns both while `dive.on` and gives them back in `Dive::stop`; the play screen's exit
  resets them to 1.
- The creek is one `spawn_water` strip (engine 1c65e8e tessellates each axis).

## Hooks for later parts (village life / last pieces slice)
- `life.rs`: `Life` is the engine-free simulation (its own LCG, seed 4242, as Kaigen; run in `PlayScreen::fixed_update` via
  `Life::step(&mut GameSave, &LifeIn, dt)`), it queues sounds in `Life::sfx` / `fish_splash` and sets `dirty` when the save
  changed (`PlayScreen::life_frame` plays them, marks the save). `LifeView` owns the entities (meshes made once:
  `TownAssets::life`), `sync` places them every fixed step, `frame` sets the lamps' glow and the rain loop's volume (its own
  voice on `Bus::Ambience`, see the last hooks). Pip's figure reads `pip`, `pip_yaw`, `pip_speed`, `pip_pose`.
- Targets: `talk::FEED_TARGET`, `SIT_TARGET`, `CHEST_TARGET` (`Use::Feed | Sit | Chest`); E at a hidden Pip finds him
  (`use_target`), the prompt says "Found you, Pip!", his marker is hidden while he hides.
- `figure.rs`: `Figure::pose` (`Pose::Sit | Dance | ArmsUp | Crouch | Wave`), `spin`, `air`, `head_want` (Kaigen fig_animate).
- Villagers: `fixed_update` drops the part of your move wish that points into a villager within 0.76 m (`npc_stops`).
- `swipe.rs` (phones, `GameSave::touch_mode` 0): the stick is off (`set_stick_enabled(false)`), `Swipe::touch` reads
  `ctx.input.touch` every frame, `Swipe::step` walks / dodges in the fixed step (rays ignore your capsule: `raycast_excluding`).
- `debugpanel.rs`: shown while the engine's debug mode is on (`ctx.debug_mode.on()`, the ` key; the engine consumes that key)
  or the menu's "Debug panel" row; `debug_pick {x, y | copy | menu}`.
- Debug: `life {rain, wet, chest, chest_at, seek, seek_left, tag, ball, dance, sit, fish, golden, to_fly, to_firefly, to_pip}`;
  `game_state.state.life` (golden, ducks, crumbs, fish_jump, butterflies, fireflies(_shown), ball, goals, chest, rain(_level),
  drops, wet, puddles, splashes, day, sitting, dancing, seek_*, hidden, tag_wins, pip_pose), `debug_panel`, `swipe`,
  `npc_stops`, `respawns`, `day`, `day_length`, `fog_density`, `hero_pose`, npcs[].head / pose; `screen.weather`.

## Hooks for later parts (GPU grass / particle layers / room z-fighting / engine 79aba87 slice)
- Particles: put every emitter on a render layer (`ParticleEmitter::layer`, like `MeshRenderer::layer`): outdoor ones on
  `town::layer_at(x, z)`, the battle's bursts (`arena::burst`) on `arena::LAYER`. Particles only step in fixed steps, so a
  screen that pauses the simulation below it freezes them: `BattleScreen::pauses_below` is false (the village gets no
  fixed_update meanwhile and just stands).
- Z-fighting: rooms.txt expects `zfight {center, radius}` == 0 pairs in all four rooms. Bricks (`Town::brick`) jitter their size
  in steps of 0.012 studs (2.3 mm per face, the check's eps is 2 mm): two touching bricks with the same jitter index can still
  share a face; thin decals (rugs, mats, steps) need ~0.03 studs of their own height offset. Studs have no bottom disc and
  are 0.01 studs lower than `STUD_H` (plates laid at `y + STUD_H` clear them). The minifig leg glTFs' hip cylinders and hips end
  2 mm short of x = 0 (they reached into each other). The lab computer's keyboard plate and the flowerpot's red flower were
  nudged in their glTFs. The hidden Deck-o-Matic sign also moves 2 m down (`zfight` does not know layers).
  Outdoors the full check still reports ~21k pairs, nearly all inside the glTF props (trees, bushes): not looked at.
- Rain loop (`LifeView::frame`): its own voice's volume follows `rain_level` (`ctx.audio.set_voice_volume`, 0.15 s ramps) and
  fades out when dry, indoors or under a battle (`LifeView::stop(ctx, fade)`); `Bus::Ambience` stays at 1.
  `game_state.state.rain_volume` (None = not playing).
- Rays from inside the hero (swipe walking's `Swipe::free`, the debug picker) use `physics.raycast_excluding(.., &[hero])`.

## Hooks for later parts (loading bar / plastic / phone pacing, engine a928c1f)

- Page: `web/index.html` replaces the engine's (dev.mjs takes it): the engine's styles plus no text selection /
  long-press callout in the game (`user-select: none`; inputs keep it) and a brick loading bar for the game download
  (fetch with progress, then `init({module_or_path: bytes})`), faded out on `engine-ready`. Keep the engine's page
  changes in sync by hand.
- Talk close-up (`talk::CamZoom`): the eye moves on a line, the look point slides from the orbit camera's to the
  board's centre (`looking_at` every frame; a slerped rotation let the face swing out), smootherstep over `ZOOM_S`
  0.85 s, the lens opens to `ZOOM_FOV` 55 degrees (wide angle from ~1 m) and is restored. Touch help (`bh-thelp`) stays left of Use / Jump.
- Fine heads: `tools/subdivide.mjs --heads` makes `assets/minifig/*/head_hi.glb` (auto smooth at 46 degrees, then
  PN-triangle patches, 4x4 per triangle, hard edges kept) and `head.glb` (the same triangles smooth shaded) from
  `head_flat.glb` (the flat-shaded original, the source: change that one and rerun). `Figure::want_detail` loads it once (out of sight) and makes
  it LOD 0 of the head within `HI_DIST` 3 m (matched by material name); play.rs calls it for villagers within 4 m
  and the one you talk to; `game_state.state.fine_heads`.
- Asset bar: `Data::loadbar(ctx)` every frame from Title, PlayScreen and BattleScreen (call it from any new screen):
  slot `loadbar` (top centre) while `ctx.assets_pending() > 0`, percent of the most pending at once.
- Plastic: `Data::plastic` (inside `loadbar`) gives every new opaque lit material the plastic look (engine 12f937f):
  roughness `PLASTIC_ROUGH` 0.3, clear coat 1 (`PLASTIC_COAT_ROUGH` 0.06), subsurface `PLASTIC_SSS` 0.3 (the glTF
  minifigs / props come with 0.72); engine `material` lists them; materials named with `NOT_PLASTIC` words (wood,
  felt, card, cave, paper, rock, sand), rougher than 0.85, glowing, metal, water, unlit or transparent stay as they
  are: name new non-plastic materials accordingly. `setup_look` sets `ambient_specular` = `AMBIENT_SPECULAR` 0.8
  (sky sheen and rim).
- Phones: `phone_quality` sets `max_fps` = `PHONE_MAX_FPS` 45 (every 2nd refresh at 90 Hz: walking juddered with
  11 / 22 ms frames in turn); `Settings::update` lifts it on a display under 80 Hz; desktop steps set 0.

## Not ported / differences from Kaigen
- The debug panel's toggle: Kaigen used ` itself; here the engine owns ` (its debug mode, which the panel follows) and phones get
  a menu row. The picker names models by their glTF file and roads / the baseplate; Kaigen's per-building tag registry
  (`town_tag`, "Dex's house", "The old castle") does not exist in this town builder.
- Pip hiding has no marker over his head (Kaigen kept drawing it, which gave him away).
- Tag on phones: Kaigen's tag needs Shift (no run on touch); here the stick pushed far also sprints (swipe walking cannot run).
- The goal pill hides in the rooms and while talking (as Kaigen); the studs pill sits under the menu button (wide) or under the
  goal (narrow), Kaigen's rows.
- The castle (`castle.glb`): its stone colours are now the values the three.js version effectively showed (its vertex colours
  were used as linear), lighter than the dark sRGB-decoded greys Kaigen and this port had.
- Right-mouse camera look / Kaigen's editor (F1, F5) are not part of this port (the camera keeps the three.js fixed view);
  Kaigen's dev keys F2/F3/F6..F9 are debug commands here (`dive {at}`, `flags`, `vox {force}`, `cards {exp}`, `battle`, `cards {rewards}`).
