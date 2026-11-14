# Brickhollow

**Play: https://mtothexmax.github.io/pokemon-clone/**

A small starter town built with the Kaigen (HZ) engine: walk around between the houses, the lab, the pond and the tall
grass while the town theme plays. Runs in the browser with WebGPU (current Chrome, Edge, Safari), on desktop and phone.

- Keyboard: WASD to walk, Shift to run, Space to jump, hold the right mouse button to look, wheel to zoom.
- Phone: the stick on the left to walk, drag anywhere else to look, pinch to zoom.

The earlier three.js/SvelteKit version (cards, Voxelmon, battles, Route 1, Stonecrest) lives on the
[`threejs-version`](https://github.com/Mtothexmax/pokemon-clone/tree/threejs-version) branch.

## Project

- `src/starting_template/starting_template.c` – the app: player, camera, HUD, music.
- `src/starting_template/st_town.c` – the town, built in code with the mesh builder (one merged vertex-coloured mesh
  plus static colliders): houses, lab, pond, fences, trees, flowers, paths.
- `assets/brickhollow/music/home.mp3` – the background music (cooked to QOA, looped on the music bus).
- `assets/starting_template/main.hzt` – the scene: sun, camera rig, floor.

## Build and run (Windows)

```powershell
hz\hzbuild                     # build
hz\hzbuild run -- --window     # build and run in a window
```

`hz` links to the installed engine (`%LOCALAPPDATA%\hz\engines\0.7.0.002`).

## Web export for GitHub Pages

```powershell
powershell -ExecutionPolicy Bypass -File .\export.ps1
```

Builds the single-threaded web bundle (no SharedArrayBuffer, so no COOP/COEP headers needed) into `docs/`, which
GitHub Pages serves from the `main` branch. See the comments in `export.ps1` for the details (brotli siblings, LFS).
