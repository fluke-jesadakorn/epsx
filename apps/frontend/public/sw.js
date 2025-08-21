const CACHE_NAME = 'epsx-v2-mobile';
const STATIC_ASSETS = [
  '/',
  '/analytics',
  '/analytics/eps',
  '/analytics/pattern-recognition',
  '/trading',
  '/ranking',
  '/dashboard',
  '/manifest.json',
  '/icons/icon-192x192.png',
  '/icons/icon-512x512.png'
];

const API_CACHE_NAME = 'epsx-api-v2';
const IMAGE_CACHE_NAME = 'epsx-images-v1';
const FONT_CACHE_NAME = 'epsx-fonts-v1';
const OFFLINE_PAGE = '/offline';

// Mobile-specific cache limits
const CACHE_LIMITS = {
  images: 50,
  api: 30,
  static: 20
};

// Install event - cache static assets
self.addEventListener('install', (event) => {
  console.log('[SW] Installing...');
  
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_NAME);
      try {
        await cache.addAll(STATIC_ASSETS);
        console.log('[SW] Static assets cached');
      } catch (error) {
        console.error('[SW] Failed to cache static assets:', error);
      }
    })()
  );
  
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating...');
  
  event.waitUntil(
    (async () => {
      const cacheNames = await caches.keys();
      const currentCaches = [CACHE_NAME, API_CACHE_NAME, IMAGE_CACHE_NAME, FONT_CACHE_NAME];
      await Promise.all(
        cacheNames
          .filter(cacheName => !currentCaches.includes(cacheName))
          .map(cacheName => caches.delete(cacheName))
      );
      console.log('[SW] Old caches cleaned');
    })()
  );
  
  self.clients.claim();
});

// Fetch event - handle requests with mobile-optimized caching strategies
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') return;

  // Skip external requests (except fonts)
  if (url.origin !== self.location.origin && !url.hostname.includes('fonts.googleapis.com') && !url.hostname.includes('fonts.gstatic.com')) return;

  // Handle different types of requests with mobile-optimized strategies
  if (url.pathname.startsWith('/api/')) {
    // API requests - network first, cache as fallback with cleanup
    event.respondWith(networkFirstWithCleanup(request));
  } else if (isImageRequest(url)) {
    // Images - cache first with size limits for mobile
    event.respondWith(imageStrategy(request));
  } else if (isFontRequest(url)) {
    // Fonts - cache first with long TTL
    event.respondWith(fontStrategy(request));
  } else if (STATIC_ASSETS.includes(url.pathname) || url.pathname.includes('/icons/')) {
    // Static assets - cache first
    event.respondWith(cacheFirstStrategy(request));
  } else {
    // Pages - stale while revalidate
    event.respondWith(staleWhileRevalidateStrategy(request));
  }
});

// Helper functions to identify request types
function isImageRequest(url) {
  return /\.(png|jpg|jpeg|gif|webp|avif|svg)$/i.test(url.pathname);
}

function isFontRequest(url) {
  return /\.(woff|woff2|ttf|otf)$/i.test(url.pathname) || url.hostname.includes('fonts.gstatic.com');
}

// Network first strategy for API calls with mobile cleanup
async function networkFirstWithCleanup(request) {
  const cache = await caches.open(API_CACHE_NAME);
  
  try {
    const response = await fetch(request);
    
    if (response.ok) {
      // Clean up cache before adding new entry
      await limitCacheSize(API_CACHE_NAME, CACHE_LIMITS.api);
      cache.put(request, response.clone());
    }
    
    return response;
  } catch (error) {
    console.log('[SW] Network failed, trying cache:', error);
    
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
      return cachedResponse;
    }
    
    // Return offline page for navigation requests
    if (request.destination === 'document') {
      return caches.match(OFFLINE_PAGE) || new Response('Offline', { status: 503 });
    }
    
    throw error;
  }
}

// Cache first strategy for static assets
async function cacheFirstStrategy(request) {
  const cache = await caches.open(CACHE_NAME);
  const cachedResponse = await cache.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.error('[SW] Failed to fetch static asset:', error);
    throw error;
  }
}

// Stale while revalidate strategy for pages
async function staleWhileRevalidateStrategy(request) {
  const cache = await caches.open(CACHE_NAME);
  const cachedResponse = await cache.match(request);
  
  const fetchPromise = fetch(request).then(response => {
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  }).catch(error => {
    console.error('[SW] Failed to fetch page:', error);
    
    // Return offline page for navigation requests if no cache
    if (request.destination === 'document' && !cachedResponse) {
      return caches.match(OFFLINE_PAGE) || new Response('Offline', { status: 503 });
    }
    
    throw error;
  });
  
  return cachedResponse || fetchPromise;
}

// Mobile-optimized image strategy with size limits
async function imageStrategy(request) {
  const cache = await caches.open(IMAGE_CACHE_NAME);
  const cachedResponse = await cache.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      // Check cache size and clean up if needed
      await limitCacheSize(IMAGE_CACHE_NAME, CACHE_LIMITS.images);
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.error('[SW] Failed to fetch image:', error);
    throw error;
  }
}

// Font strategy with long-term caching
async function fontStrategy(request) {
  const cache = await caches.open(FONT_CACHE_NAME);
  const cachedResponse = await cache.match(request);
  
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      // Fonts are cached for a very long time
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.error('[SW] Failed to fetch font:', error);
    throw error;
  }
}

// Cache size limiter for mobile optimization
async function limitCacheSize(cacheName, maxItems) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();
  
  if (keys.length > maxItems) {
    // Remove oldest entries (FIFO)
    const itemsToDelete = keys.slice(0, keys.length - maxItems);
    await Promise.all(itemsToDelete.map(key => cache.delete(key)));
    console.log(`[SW] Cleaned ${itemsToDelete.length} items from ${cacheName}`);
  }
}

// Background sync for analytics and notifications
self.addEventListener('sync', (event) => {
  console.log('[SW] Background sync:', event.tag);
  
  if (event.tag === 'background-analytics') {
    event.waitUntil(syncAnalytics());
  }
  
  if (event.tag === 'background-notifications') {
    event.waitUntil(syncNotifications());
  }
});

async function syncAnalytics() {
  try {
    // Analytics sync disabled - API endpoints have been migrated to server actions
    console.log('[SW] Analytics sync skipped - migrated to server actions');
  } catch (error) {
    console.error('[SW] Analytics sync failed:', error);
  }
}

async function syncNotifications() {
  try {
    // Notification sync disabled - API endpoints have been migrated to server actions
    console.log('[SW] Notification sync skipped - migrated to server actions');
  } catch (error) {
    console.error('[SW] Notification sync failed:', error);
  }
}

// Push notifications
self.addEventListener('push', (event) => {
  console.log('[SW] Push received:', event);
  
  const options = {
    body: 'You have a new trading alert',
    icon: '/icons/icon-192x192.png',
    badge: '/icons/badge-72x72.png',
    vibrate: [200, 100, 200],
    requireInteraction: true,
    actions: [
      { action: 'view', title: 'View' },
      { action: 'dismiss', title: 'Dismiss' }
    ]
  };
  
  if (event.data) {
    try {
      const data = event.data.json();
      options.body = data.message || options.body;
      options.tag = data.tag || 'default';
    } catch (error) {
      console.error('[SW] Failed to parse push data:', error);
    }
  }
  
  event.waitUntil(
    self.registration.showNotification('EPSX Alert', options)
  );
});

// Notification click handler
self.addEventListener('notificationclick', (event) => {
  console.log('[SW] Notification clicked:', event);
  
  event.notification.close();
  
  if (event.action === 'view') {
    event.waitUntil(
      clients.openWindow('/notifications')
    );
  }
});

// Helper functions
async function getStoredAnalytics() {
  // Implementation would depend on IndexedDB storage
  return [];
}

async function clearStoredAnalytics() {
  // Implementation would depend on IndexedDB storage
}

// Message handler for client communication
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});

console.log('[SW] Service Worker loaded');