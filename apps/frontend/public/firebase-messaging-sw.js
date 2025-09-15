// Firebase Cloud Messaging Service Worker - Dynamic Configuration
// This service worker receives Firebase configuration via postMessage from the main thread

let firebaseConfig = null;
let messaging = null;
let isFirebaseInitialized = false;

// Import Firebase scripts
importScripts('https://www.gstatic.com/firebasejs/12.2.1/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/12.2.1/firebase-messaging-compat.js');

// Listen for configuration from main thread
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'FIREBASE_CONFIG') {
    console.log('[SW] Received Firebase config from main thread');
    firebaseConfig = event.data.config;
    initializeFirebase();
  }
});

function initializeFirebase() {
  if (!firebaseConfig || isFirebaseInitialized) return;

  try {
    // Validate configuration - no fallback, must receive proper config
    if (!firebaseConfig || !firebaseConfig.apiKey) {
      console.warn('[SW] Firebase configuration not available - push notifications disabled');
      return;
    }
    
    const requiredFields = ['apiKey', 'authDomain', 'projectId', 'messagingSenderId', 'appId'];

    for (const field of requiredFields) {
      if (!firebaseConfig[field]) {
        console.warn(`[SW] Missing Firebase configuration field: ${field} - push notifications disabled`);
        return;
      }
    }

    // Initialize Firebase
    firebase.initializeApp(firebaseConfig);
    messaging = firebase.messaging();
    isFirebaseInitialized = true;

    console.log('[SW] Firebase messaging initialized successfully');

    // Handle background messages
    messaging.onBackgroundMessage((payload) => {
      console.log('[SW] Background message received:', payload);

      const notificationTitle = payload.notification?.title || 'EPSX Notification';
      const notificationOptions = {
        body: payload.notification?.body || 'You have a new notification',
        icon: payload.notification?.image || '/icons/icon-192x192.png',
        badge: '/icons/badge-72x72.png',
        image: payload.notification?.image,
        data: {
          ...payload.data,
          click_action: payload.data?.click_action || '/',
          notification_id: payload.data?.notification_id,
          type: payload.data?.type || 'notification'
        },
        actions: [
          {
            action: 'open',
            title: 'Open',
            icon: '/icons/open-icon.png'
          },
          {
            action: 'dismiss',
            title: 'Dismiss',
            icon: '/icons/dismiss-icon.png'
          }
        ],
        requireInteraction: payload.data?.priority === 'high',
        silent: payload.data?.priority === 'low',
        tag: payload.data?.type || 'default',
        timestamp: Date.now(),
        vibrate: payload.data?.priority === 'high' ? [200, 100, 200] : [100],
      };

      return self.registration.showNotification(notificationTitle, notificationOptions);
    });

  } catch (error) {
    console.warn('[SW] Firebase initialization failed - push notifications disabled:', error);
  }
}

// Handle notification click
self.addEventListener('notificationclick', (event) => {
  console.log('[SW] Notification clicked:', event);

  const { data } = event.notification;
  event.notification.close();

  if (event.action === 'dismiss') {
    return;
  }

  const clickAction = data?.click_action || '/';
  
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true })
      .then((clientList) => {
        for (const client of clientList) {
          if (client.url.includes(self.location.origin) && 'focus' in client) {
            client.navigate(clickAction);
            return client.focus();
          }
        }
        
        if (clients.openWindow) {
          return clients.openWindow(clickAction);
        }
      })
      .catch(err => {
        console.error('[SW] Error handling notification click:', err);
      })
  );

  // Track notification click
  if (data?.notification_id) {
    fetch('/api/v1/notifications/track', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        notification_id: data.notification_id,
        action: 'clicked',
        timestamp: new Date().toISOString()
      })
    }).catch(err => console.log('[SW] Failed to track notification click:', err));
  }
});

// Handle notification close
self.addEventListener('notificationclose', (event) => {
  console.log('[SW] Notification closed:', event);
  
  const { data } = event.notification;
  
  if (data?.notification_id) {
    fetch('/api/v1/notifications/track', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        notification_id: data.notification_id,
        action: 'dismissed',
        timestamp: new Date().toISOString()
      })
    }).catch(err => console.log('[SW] Failed to track notification dismissal:', err));
  }
});

// Service worker lifecycle events
self.addEventListener('install', (event) => {
  console.log('[SW] Service worker installing');
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  console.log('[SW] Service worker activating');
  event.waitUntil(self.clients.claim());
});

// Handle push event (fallback)
self.addEventListener('push', (event) => {
  console.log('[SW] Push received:', event);
  
  if (event.data) {
    const payload = event.data.json();
    const title = payload.notification?.title || 'EPSX Notification';
    const options = {
      body: payload.notification?.body || 'You have a new notification',
      icon: '/icons/icon-192x192.png',
      badge: '/icons/badge-72x72.png',
      data: payload.data
    };
    
    event.waitUntil(
      self.registration.showNotification(title, options)
    );
  }
});