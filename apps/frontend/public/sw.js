// PWA Service Worker for EPSX Trading Platform
// Provides offline functionality, caching, and background sync

const CACHE_NAME = 'epsx-v1';
const RUNTIME_CACHE = 'epsx-runtime-v1';
const OFFLINE_PAGE = '/offline';

// Assets to cache on install
const PRECACHE_ASSETS = [
  '/',
  '/offline',
  '/favicon.ico',
  '/logo.png',
  // Add critical CSS and JS files here
];

// Routes that should work offline
const OFFLINE_FALLBACK_ROUTES = [
  '/notifications',
  '/dashboard',
  '/analytics',
  '/ranking'
];

// Cache first strategy for these file types
const CACHE_FIRST_PATTERNS = [
  /\.(?:png|jpg|jpeg|svg|gif|webp|avif)$/,
  /\.(?:woff|woff2|eot|ttf|otf)$/,
  /\.(?:css|js)$/
];

// Network first strategy for API calls
const NETWORK_FIRST_PATTERNS = [
  /\/api\//,
  /\/auth\//
];

// Install event - precache assets
self.addEventListener('install', (event) => {
  console.log('PWA Service worker installing...');
  
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      console.log('Precaching assets...');
      return cache.addAll(PRECACHE_ASSETS).catch((error) => {
        console.error('Precaching failed for some assets:', error);
        // Don't fail installation if some assets fail to cache
        return Promise.resolve();
      });
    })
  );
  
  // Force the waiting service worker to become the active service worker
  self.skipWaiting();
});

// Activate event - cleanup old caches
self.addEventListener('activate', (event) => {
  console.log('PWA Service worker activating...');
  
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME && cacheName !== RUNTIME_CACHE) {
            console.log('Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
      // Take control of all open clients
      return self.clients.claim();
    })
  );
});

// Fetch event - handle network requests
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);
  
  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }
  
  // Skip chrome-extension and other non-http requests
  if (!request.url.startsWith('http')) {
    return;
  }
  
  // Handle different caching strategies based on request type
  if (CACHE_FIRST_PATTERNS.some(pattern => pattern.test(url.pathname))) {
    // Cache first strategy for static assets
    event.respondWith(cacheFirstStrategy(request));
  } else if (NETWORK_FIRST_PATTERNS.some(pattern => pattern.test(url.pathname))) {
    // Network first strategy for API calls
    event.respondWith(networkFirstStrategy(request));
  } else {
    // Stale while revalidate for pages
    event.respondWith(staleWhileRevalidateStrategy(request));
  }
});

// Cache first strategy
async function cacheFirstStrategy(request) {
  try {
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      const cache = await caches.open(RUNTIME_CACHE);
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.error('Cache first strategy failed:', error);
    return new Response('Network error', { status: 503 });
  }
}

// Network first strategy
async function networkFirstStrategy(request) {
  try {
    const networkResponse = await fetch(request);
    
    // Cache successful API responses for offline use
    if (networkResponse.ok && request.url.includes('/api/')) {
      const cache = await caches.open(RUNTIME_CACHE);
      cache.put(request, networkResponse.clone());
    }
    
    return networkResponse;
  } catch (error) {
    console.error('Network request failed:', error);
    
    // Try to serve from cache if network fails
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Return offline response for API calls
    if (request.url.includes('/api/')) {
      return new Response(
        JSON.stringify({
          error: 'Offline',
          message: 'This feature requires an internet connection'
        }),
        {
          status: 503,
          headers: { 'Content-Type': 'application/json' }
        }
      );
    }
    
    return new Response('Network error', { status: 503 });
  }
}

// Stale while revalidate strategy
async function staleWhileRevalidateStrategy(request) {
  const cache = await caches.open(RUNTIME_CACHE);
  const cachedResponse = await cache.match(request);
  
  // Fetch from network in the background
  const networkResponsePromise = fetch(request).then((networkResponse) => {
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  }).catch((error) => {
    console.error('Background fetch failed:', error);
    return null;
  });
  
  // Return cached response immediately if available
  if (cachedResponse) {
    return cachedResponse;
  }
  
  // If no cached response, wait for network
  try {
    const networkResponse = await networkResponsePromise;
    if (networkResponse) {
      return networkResponse;
    }
  } catch (error) {
    console.error('Network response failed:', error);
  }
  
  // Fallback to offline page for navigation requests
  if (request.mode === 'navigate') {
    const offlinePage = await caches.match(OFFLINE_PAGE);
    if (offlinePage) {
      return offlinePage;
    }
  }
  
  return new Response('Offline', { status: 503 });
}

// Background sync for failed requests
self.addEventListener('sync', (event) => {
  console.log('Background sync triggered:', event.tag);
  
  if (event.tag === 'notification-sync') {
    event.waitUntil(syncFailedNotifications());
  } else if (event.tag === 'analytics-sync') {
    event.waitUntil(syncAnalyticsData());
  }
});

// Sync failed notifications
async function syncFailedNotifications() {
  try {
    console.log('Syncing failed notifications...');
    // Implement notification sync logic here
    // This could retry failed notification API calls
  } catch (error) {
    console.error('Notification sync failed:', error);
  }
}

// Sync analytics data
async function syncAnalyticsData() {
  try {
    console.log('Syncing analytics data...');
    // Implement analytics sync logic here
    // This could send cached analytics events
  } catch (error) {
    console.error('Analytics sync failed:', error);
  }
}

// Handle messages from the main thread
self.addEventListener('message', (event) => {
  console.log('Service worker received message:', event.data);
  
  if (event.data?.type === 'SKIP_WAITING') {
    self.skipWaiting();
  } else if (event.data?.type === 'GET_VERSION') {
    event.ports[0].postMessage({
      type: 'VERSION',
      version: CACHE_NAME
    });
  } else if (event.data?.type === 'CACHE_URLS') {
    event.waitUntil(cacheUrls(event.data.urls));
  }
});

// Cache specific URLs
async function cacheUrls(urls) {
  const cache = await caches.open(RUNTIME_CACHE);
  try {
    await cache.addAll(urls);
    console.log('URLs cached successfully:', urls);
  } catch (error) {
    console.error('Failed to cache URLs:', error);
  }
}

// Periodic cleanup
self.addEventListener('periodicsync', (event) => {
  if (event.tag === 'cache-cleanup') {
    event.waitUntil(cleanupOldCaches());
  }
});

// Cleanup old cache entries
async function cleanupOldCaches() {
  const cache = await caches.open(RUNTIME_CACHE);
  const requests = await cache.keys();
  
  const now = Date.now();
  const maxAge = 7 * 24 * 60 * 60 * 1000; // 7 days
  
  for (const request of requests) {
    const response = await cache.match(request);
    const cachedDate = response?.headers.get('date');
    
    if (cachedDate && now - new Date(cachedDate).getTime() > maxAge) {
      await cache.delete(request);
      console.log('Removed old cache entry:', request.url);
    }
  }
}

console.log('PWA Service worker loaded successfully');