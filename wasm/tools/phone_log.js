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
  setInterval(() => {
    let r = null;
    try {
      const s = window.engine?.call?.('stats');
      const v = s?.result ?? s;
      r = v?.render ? { draws: v.render.draw_calls, tris: v.render.triangles, scale: v.render.render_scale, size: v.render.size ?? v.render.resolution, gpu_ms: v.render.gpu_ms?.frame,
        pacing: v.render.pacing ? { interval: v.render.pacing.interval, cap: v.render.pacing.cap_interval, latency_ms: v.render.pacing.latency_ms, queued: v.render.pacing.queued, late: v.render.pacing.late, skipped: v.render.pacing.skipped } : null } : null;
    } catch {}
    // fps = animation frames; the engine draws every `pacing.interval`-th of them (pacing.skipped: held back by back-pressure)
    send('fps', { fps: +(frames / 5).toFixed(1), screen: (() => { try { const g = window.engine?.call?.('game_state'); return (g?.result ?? g)?.screen; } catch { return null; } })(), render: r });
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
      // (the measuring mode below traces on its own)
      if (traced || id == null || !(st?.state?.hero?.speed > 1) || /[?&]measure/.test(location.search)) return;
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
      // warnings and errors, plus the info lines about frame pacing and first-use hitches
      const r = window.engine?.call?.('logs', { since, level: 'info', limit: 200 });
      const v = r?.result ?? r;
      const all = Array.isArray(v) ? v : v?.entries || [];
      const list = all.filter((x) => x.level !== 'info' || /first use|prewarm|adaptive|frame pacing|graphics/.test(x.msg || ''));
      if (list.length) send('engine', list);
      if (all.length) since = Math.max(since, ...all.map((x) => (x.seq ?? 0) + 1));
      if (window.engine?.fatal) send('fatal', window.engine.fatal);
    } catch (e) { send('poll', String(e)); }
  }, 3000);
})();

// ---- Measuring mode (https://<pc>:8444/?measure): the player walks, the PC gets the timing ----
// A badge at the top counts the seconds walked. While the hero walks: every animation frame's interval (3 s windows:
// p50 / p95 / max, frames over 1.5x / 2.5x the median), the engine's render size, GPU time, frame pacing and CPU phases
// twice a second, and the drawn hero position over 240 frames at a time (engine `trace`: speed jitter, frames with
// 0 / 2+ fixed steps) - all as "measure" lines in logs/phone.log. Done after 60 s of walking.
(() => {
  if (!/[?&]measure\b/.test(location.search)) return;
  const send = (kind, data) => {
    try { fetch('/__log', { method: 'POST', body: JSON.stringify({ t: Math.round(performance.now()), kind, data }), keepalive: true }); } catch {}
  };
  const call = (n, a) => { try { const r = window.engine?.call?.(n, a); return r?.result ?? r; } catch { return null; } };
  const badge = document.createElement('div');
  badge.style.cssText = 'position:fixed;left:50%;top:6px;transform:translateX(-50%);z-index:99;background:#c21;color:#fff;font:600 13px system-ui;padding:4px 10px;border-radius:12px;pointer-events:none';
  badge.textContent = 'measuring: walk around';
  addEventListener('DOMContentLoaded', () => document.body.appendChild(badge));
  const GOAL = 60;
  let walked = 0, last = null, walking = false, win = [], winT = 0, traceOn = false, done = false;
  const q = (a, p) => a[Math.min(a.length - 1, Math.floor(p * a.length))];
  const flush = () => {
    if (win.length < 5) { win = []; return; }
    const s = [...win].sort((a, b) => a - b);
    const med = q(s, 0.5);
    send('measure', { what: 'frames', n: win.length, p50: +med.toFixed(1), p95: +q(s, 0.95).toFixed(1), max: +s[s.length - 1].toFixed(1),
      over1_5: win.filter((x) => x > med * 1.5).length, over2_5: win.filter((x) => x > med * 2.5).length,
      seq: win.map((x) => Math.round(x)).join(',') });
    win = [];
  };
  const frame = (t) => {
    if (last != null && walking && !done) {
      const dt = t - last;
      win.push(dt);
      walked += dt / 1000;
      winT += dt;
      if (winT > 3000) { winT = 0; flush(); }
    }
    last = t;
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
  setInterval(() => {
    if (done) return;
    const g = call('game_state');
    const h = g?.state?.hero;
    walking = !!h && h.speed > 1;
    badge.textContent = walked >= GOAL ? 'measured - thanks!' : `measuring: walk around (${Math.floor(walked)} / ${GOAL} s)`;
    if (walked >= GOAL) { done = true; flush(); send('measure', { what: 'done', walked: Math.round(walked) }); badge.style.background = '#2a2'; return; }
    if (!walking) return;
    const st = call('stats');
    const r = st?.render, tm = st?.time;
    send('measure', { what: 'stats', speed: +h.speed.toFixed(2), screen: g?.screen, area: g?.state?.area, zone: g?.state?.zone,
      scale: r?.render_scale, size: r?.width ? [r.width, r.height] : r?.size, gpu: r?.gpu_ms, draws: r?.draw_calls, tris: r?.triangles,
      hz: tm?.refresh_hz, interval: tm?.frame_interval,
      frame_ms: tm?.frame_ms, phases: tm?.phases, fps: tm?.fps, pacing: r?.pacing ? { mode: r.pacing.mode, interval: r.pacing.interval, auto: r.pacing.auto_interval, cap: r.pacing.cap_interval, latency: r.pacing.latency_ms, gpu: r.pacing.gpu_frame_ms,
        base: r.pacing.baseline_ms, jitter: r.pacing.jitter_ms, queue: r.pacing.queue_ms, queued: r.pacing.queued, late: r.pacing.late,
        skipped: r.pacing.skipped, miss: r.pacing.window_miss_pct } : null, steps: tm?.steps_per_frame });
    // the drawn position: 240 frames at a time while walking
    if (!traceOn && h.id != null) {
      traceOn = true;
      call('trace', { id: h.id, frames: 240 });
      const poll = setInterval(() => {
        const tr = call('trace', { samples: true });
        if (tr?.done) {
          clearInterval(poll);
          traceOn = false;
          const s = tr.samples || [];
          send('measure', { what: 'trace', jitter: tr.speed_jitter_pct, steps0: tr.frames_with_0_steps, steps2: tr.frames_with_2plus_steps,
            mean_speed: tr.mean_speed, frame_ms: s.map((x) => Math.round(x[0] * 10) / 10).join(','), steps: s.map((x) => x[1]).join('') });
        }
      }, 1000);
    }
  }, 500);
})();
