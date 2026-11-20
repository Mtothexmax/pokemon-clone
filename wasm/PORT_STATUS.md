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
- [ ] Wild Voxelmon fights in the tall grass, skills, upgrades, own music, no touch Jump/Use in fights
- [ ] Lab quest: diving suit, deck machine "missing part", pond dive into the cave (swim in 3D, darker with depth,
      seaweed, glowing plants, collectibles, marine snow, part deep down), deck built on return
- [ ] Route 1 north and Stonecrest
- [ ] Hide and seek with Pip (then he battles), night pill, gate, debug panel

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
  pushes `BattleScreen` (scene "battle"; the village stays loaded below, its camera's `Camera` is taken off while
  the battle shows and put back on pop). When it pops, `PlayScreen::resume` -> `battle_done` pays out (studs, EXP,
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
