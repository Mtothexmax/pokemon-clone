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
- [x] Sound: music (home, wild fight), voices, effects; settings menu (voices, music, graphics); save
      (sound.rs, settings.rs, save.rs; battle tracks still to convert, see sound.rs; tests menu.txt, touch.txt)
- [x] Dialogues with voices, village life (NPC routines, facing who you talk to), goal pill / markers
      (dialog.rs, talk.rs, life.rs, play.rs; Pip's hide and seek / tag / football, the ducks, ball, weather etc.
      of st_life.c come with their parts; tests dialog.txt, life.txt)
- [x] Rooms / interiors with doors (studded furniture, warm light), TV and computer screens, piano, mailbox,
      sign close-ups; the bedroom start with the intro thought, Mom / Lumi / the lab Prof inside, the
      Deck-o-Matic showing MISSING PART (rooms.rs, screens.rs, talk.rs; tests rooms.txt, screens.txt)
- [ ] Cards and Voxelmon data, starter gift, foil packs, deck screen (collection, deck)
- [ ] Card battle in 3D (cards are 3D objects lowered onto a 3D playing field) vs Dex, Lumi, Pip
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
