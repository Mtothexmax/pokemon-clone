// coi-sw.js - makes the page cross-origin isolated on hosts that cannot send headers (GitHub Pages, python http.server).
// The engine's threaded web build needs SharedArrayBuffer (the audio worklet reads the mixer's ring buffer from the
// game's shared memory), which browsers only allow with COOP same-origin + COEP require-corp. This worker answers
// every request itself and adds those headers. index.html registers it and reloads once so it controls the page.

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));

self.addEventListener("fetch", (e) => {
    const req = e.request;
    if (req.cache === "only-if-cached" && req.mode !== "same-origin") return;
    e.respondWith(
        fetch(req).then((res) => {
            // opaque and body-less responses pass through untouched
            if (res.status === 0 || [101, 204, 205, 304].includes(res.status)) return res;
            const headers = new Headers(res.headers);
            headers.set("Cross-Origin-Embedder-Policy", "require-corp");
            headers.set("Cross-Origin-Opener-Policy", "same-origin");
            headers.set("Cross-Origin-Resource-Policy", "cross-origin");
            return new Response(res.body, { status: res.status, statusText: res.statusText, headers });
        })
    );
});
