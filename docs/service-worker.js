const CACHE_NAME = "garnet-web-v5";
const OFFLINE_ASSETS = [
  "./",
  "getting-started.html",
  "index.html",
  "install.sh",
  "ladder.html",
  "manifest.webmanifest",
  "minispec.html",
  "novel.html",
  "playground.html",
  "status.html",
  "stdlib.html",
  "synthesis.html",
  "blog/index.html",
  "blog/feed.xml",
  "releases.xml",
  "assets/garnet-hero.webp",
  "assets/garnet-demonstration-poster.jpg",
  "icons/garnet-192.png",
  "icons/garnet-512.png"
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(OFFLINE_ASSETS))
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    )
  );
  self.clients.claim();
});

// Video and audio bypass this worker entirely: a <video> fetches in ranges, and a
// whole 5 MB file is neither an offline asset nor something to serve stale.
const MEDIA_PATH = /\.(mp4|webm|m4v|mov|mp3|ogg|wav)(\?|#|$)/i;
function isMedia(request) {
  const destination = request.destination || "";
  return destination === "video" || destination === "audio" || MEDIA_PATH.test(String(request.url || ""));
}

self.addEventListener("fetch", (event) => {
  const request = event.request;

  // Media goes straight to the network: never cached, never served from cache.
  if (isMedia(request)) {
    return;
  }

  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, copy));
          return response;
        })
        .catch(() => caches.match(request).then((hit) => hit || caches.match("index.html")))
    );
    return;
  }

  event.respondWith(
    caches.match(request).then((hit) => {
      if (hit) {
        return hit;
      }
      return fetch(request).then((response) => {
        // A 206 is a partial body and cannot be cached.
        const ranged = Boolean(request.headers && request.headers.has && request.headers.has("range"));
        if (request.method === "GET" && response.ok && response.status !== 206 && !ranged) {
          const copy = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, copy));
        }
        return response;
      });
    })
  );
});
