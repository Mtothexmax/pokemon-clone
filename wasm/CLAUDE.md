# brickhollow

A game on the **wasmOnly** engine (Rust -> wasm32, WebGPU, Chrome only, AI-first).
Read AGENTS.md here first, then the engine's own docs:

- C:/dev/wasmOnly/AGENTS.md — dev loop, AI runtime API (every state is JSON via `window.engine.call`), code map, rules
- C:/dev/wasmOnly/.claude/README.md — skills: engine loop, verify gameplay, new game (Game/Ctx/Screen API,
  open world, generators, animation, particles, ...), debug rendering, assets
- C:/dev/wasmOnly/HANDOFF.md — engine overview for a new project, porting notes (three.js -> wasmOnly)
- Reference game: C:/dev/wasmOnly/games/demo (title / play / pause screens, open world in src/world.rs)
