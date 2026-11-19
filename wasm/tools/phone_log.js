// Wi-Fi build only (tools/lan.ps1 injects it): sends what the phone's console sees to tools/serve_lan.py,
// which writes it to out/phone.log. The phone has no dev tools, so this is how its errors reach the PC.
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
