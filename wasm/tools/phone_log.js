// Wi-Fi build only (tools/lan.ps1 injects it): sends what the phone's console sees to tools/serve_lan.py,
// which writes it to logs/phone.log. The phone has no dev tools, so this is how its errors reach the PC.
(() => {
  const send = (kind, data) => {
    try {
      fetch('/__log', { method: 'POST', body: JSON.stringify({ t: Math.round(performance.now()), kind, data }), keepalive: true });
    } catch {}
  };
  const str = (a) => a.map((x) => (x instanceof Error ? x.stack || String(x) : typeof x === 'object' ? (() => { try { return JSON.stringify(x); } catch { return String(x); } })() : String(x))).join(' ');
  for (const level of ['error', 'warn']) {
    const orig = console[level].bind(console);
    console[level] = (...a) => { send(level, str(a)); orig(...a); };
  }
  addEventListener('error', (e) => send('onerror', `${e.message} @ ${e.filename}:${e.lineno} ${e.error?.stack || ''}`));
  addEventListener('unhandledrejection', (e) => send('rejection', str([e.reason])));
  send('start', { ua: navigator.userAgent, secure: isSecureContext, gpu: !!navigator.gpu, w: innerWidth, h: innerHeight, dpr: devicePixelRatio });
  (async () => {
    try {
      const a = await navigator.gpu?.requestAdapter();
      if (!a) return send('adapter', 'none');
      const l = {};
      for (const k of ['maxStorageBuffersPerShaderStage', 'maxStorageBuffersInVertexStage', 'maxStorageBuffersInFragmentStage', 'maxStorageTexturesPerShaderStage', 'maxComputeWorkgroupStorageSize', 'maxBindGroups', 'maxSampledTexturesPerShaderStage', 'maxColorAttachmentBytesPerSample', 'maxTextureDimension2D', 'maxBufferSize', 'maxStorageBufferBindingSize', 'maxVertexBuffers', 'maxInterStageShaderVariables']) l[k] = a.limits[k];
      send('adapter', { info: a.info ? { vendor: a.info.vendor, arch: a.info.architecture, desc: a.info.description } : null, features: [...a.features], limits: l });
    } catch (e) { send('adapter', 'error ' + e); }
  })();
  // frame rate (requestAnimationFrame) and the engine's render size every 5 s
  let frames = 0;
  const tick = () => { frames++; requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
  let lastSkipped = null;
  setInterval(() => {
    let r = null;
    try {
      const s = window.engine?.call?.('stats');
      const v = s?.result ?? s;
      r = v?.render ? { draws: v.render.draw_calls, tris: v.render.triangles, scale: v.render.render_scale, size: v.render.size ?? v.render.resolution, gpu_ms: v.render.gpu_ms?.frame, lat_ms: v.render.gpu_latency_ms, in_flight: v.render.frames_in_flight, skipped: v.render.frames_skipped } : null;
    } catch {}
    // frames the engine actually rendered: animation frames minus the ones its frame limiter skipped
    const sk = r?.skipped;
    const drawn = sk != null && lastSkipped != null ? +((frames - (sk - lastSkipped)) / 5).toFixed(1) : null;
    lastSkipped = sk ?? lastSkipped;
    send('fps', { fps: +(frames / 5).toFixed(1), drawn, screen: (() => { try { const g = window.engine?.call?.('game_state'); return (g?.result ?? g)?.screen; } catch { return null; } })(), render: r });
    frames = 0;
  }, 5000);
  // smoothness: once in play, record the hero's drawn position for 120 frames, then send the samples
  let traced = false;
  const tracer = setInterval(() => {
    try {
      const g = window.engine?.call?.('game_state');
      const st = g?.result ?? g;
      const id = st?.state?.hero?.id;
      // start once the hero walks (that is what should look smooth)
      if (traced || id == null || !(st?.state?.hero?.speed > 1)) return;
      traced = true;
      clearInterval(tracer);
      window.engine.call('trace', { id, frames: 120 });
      setTimeout(() => { try { send('trace', window.engine.call('trace', { samples: true })); } catch (e) { send('trace', String(e)); } }, 8000);
    } catch {}
  }, 1000);
  // the engine's own log (validation errors, warnings) every 3 s once it is up
  let since = 0;
  setInterval(() => {
    try {
      const r = window.engine?.call?.('logs', { since, level: 'warn', limit: 50 });
      const v = r?.result ?? r;
      const list = Array.isArray(v) ? v : v?.entries || [];
      if (list.length) { send('engine', list); since = Math.max(since, ...list.map((x) => (x.seq ?? 0) + 1)); }
      if (window.engine?.fatal) send('fatal', window.engine.fatal);
    } catch (e) { send('poll', String(e)); }
  }, 3000);
})();
