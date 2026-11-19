# AGENTS.md — brickhollow

Engine checkout: `C:/dev/wasmOnly` (tools + docs live there; the engine crate is a path dependency: ../../wasmOnly/crates/engine).

## Loop
```bash
node C:/dev/wasmOnly/tools/dev.mjs watch --game-dir .          # build + serve http://localhost:8080 + rebuild/reload (keep running)
node C:/dev/wasmOnly/tools/bridge.mjs launch                   # debug Chrome (CDP :9222), muted
node C:/dev/wasmOnly/tools/bridge.mjs reload                   # after a rebuild: errors/warnings as JSON
node C:/dev/wasmOnly/tools/bridge.mjs call help                # every engine + game command
node C:/dev/wasmOnly/tools/bridge.mjs run tests/smoke.txt      # scripted test (input, step, expect, screenshot)
node C:/dev/wasmOnly/tools/bridge.mjs check                    # build check: errors, z-fighting
node C:/dev/wasmOnly/tools/bridge.mjs perf                     # frame times, CPU phases, GPU passes, verdict
```
MCP: `.mcp.json` registers the bridge as server `wasmonly` (engine_call, reload, screenshot, run_script, perf, check).

## Rules
- Verify with JSON (`game_state`, `entity`, `logs`) and test scripts, not screenshots alone.
- Put checks you will run again into `tests/*.txt`.
- `debug_state` must expose what a test needs; add game commands in `debug_command`.
- Input: `input.key` = held; `key_pressed` = edge (in `fixed_update` latched since the last step, so taps are never lost);
  jump/fire-style actions: `key_down_or_pressed`. Every action needs a scripted test with a short tap (`key Space` + `step`).
- When the user reports a bug: ask them to press ` (on German keyboards ^) before and after reproducing it; read the
  newest `logs/debug-*.log` (keys, camera, per-second perf + characters + game_state).
- Engine bugs/missing features: fix them in the engine checkout (C:/dev/wasmOnly) so every game benefits;
  keep game-specific content (generators, levels) in this repo.
