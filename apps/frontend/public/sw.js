const CACHE_NAME = 'epsx-v1';
const STATIC_ASSETS = [
  '/',
  '/analytics',
  '/analytics/eps',
  '/analytics/pattern-recognition',
  '/manifest.json',
  '/icons/icon-192x192.png',
  '/icons/icon-512x512.png'
];

const API_CACHE_NAME = 'epsx-api-v1';
const OFFLINE_PAGE = '/offline';

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
      await Promise.all(
        cacheNames
          .filter(cacheName => cacheName !== CACHE_NAME && cacheName !== API_CACHE_NAME)
          .map(cacheName => caches.delete(cacheName))
      );
      console.log('[SW] Old caches cleaned');
    })()
  );
  
  self.clients.claim();
});

// Fetch event - handle requests with caching strategies
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') return;

  // Skip external requests
  if (url.origin !== self.location.origin) return;

  // Handle different types of requests
  if (url.pathname.startsWith('/api/')) {
    // API requests - network first, cache as fallback
    event.respondWith(networkFirstStrategy(request));
  } else if (STATIC_ASSETS.includes(url.pathname) || url.pathname.includes('/icons/')) {
    // Static assets - cache first
    event.respondWith(cacheFirstStrategy(request));
  } else {
    // Pages - stale while revalidate
    event.respondWith(staleWhileRevalidateStrategy(request));
  }
});

// Network first strategy for API calls
async function networkFirstStrategy(request) {
  const cache = await caches.open(API_CACHE_NAME);
  
  try {
    const response = await fetch(request);
    
    if (response.ok) {
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
    // Sync any pending analytics data
    const pendingData = await getStoredAnalytics();
    if (pendingData.length > 0) {
      await fetch('/api/analytics/batch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(pendingData)
      });
      await clearStoredAnalytics();
    }
  } catch (error) {
    console.error('[SW] Analytics sync failed:', error);
  }
}

async function syncNotifications() {
  try {
    // Check for new notifications
    const response = await fetch('/api/notifications/check');
    const data = await response.json();
    
    if (data.hasNew) {
      self.registration.showNotification('EPSX', {
        body: 'You have new trading alerts',
        icon: '/icons/icon-192x192.png',
        badge: '/icons/badge-72x72.png',
        tag: 'new-alerts',
        requireInteraction: true,
        actions: [
          { action: 'view', title: 'View Alerts' },
          { action: 'dismiss', title: 'Dismiss' }
        ]
      });
    }
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