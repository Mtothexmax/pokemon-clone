# Pallet Quest

A tiny, entirely-procedural 2D Pokémon-style overworld rendered with the Canvas
2D API and hosted by SvelteKit. There are no image files in this project — every
tile, building, tree and character is generated in code.

**Repository:** https://github.com/Mtothexmax/pokemon-clone

## Play

- Live: https://mtothexmax.github.io/pokemon-clone/ (GitHub Pages, `docs/`)
- Local:

```bash
npm install
npm run dev
```

## Controls

| Action | Keyboard | Touch |
| --- | --- | --- |
| Move | Arrow keys / WASD | On-screen analog stick |
| A — talk / confirm / jump | `Z` / `J` / Space | **A** button |
| B — cancel / run | `X` / `K` / Shift | **B** button |
| Inspect a tile (debug) | `` ` `` | — |

On touch devices an analog stick (bottom-left) and A/B buttons (bottom-right)
appear automatically (force with `?touch=1`, hide with `?touch=0`). They scale
with the viewport so the thumb reach stays consistent from a small phone to a
4K display. The game view keeps its aspect ratio and letterboxes to fit.

## Development

```bash
node tools/shots.mjs --build --test   # build, run tests, capture screenshots
node test/headless.mjs                # a single test suite
```

## Static export

`Export.ps1` builds a static site into `docs/` for GitHub Pages
(`adapter-static`, base path `/pokemon-clone`):

```powershell
./Export.ps1
```
