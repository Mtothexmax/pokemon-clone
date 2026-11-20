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
- [ ] Rooms / interiors with doors (studded furniture, warm light), TV and computer screens, piano, mailbox,
      sign close-ups
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
