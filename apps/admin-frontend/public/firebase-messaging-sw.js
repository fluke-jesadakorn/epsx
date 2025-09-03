// Firebase Cloud Messaging Service Worker for Admin Frontend - Dynamic Configuration
// This service worker receives Firebase configuration via postMessage from the main thread

let firebaseConfig = null;
let messaging = null;
let isFirebaseInitialized = false;

// Import Firebase scripts with error handling
try {
  importScripts('https://www.gstatic.com/firebasejs/12.2.1/firebase-app-compat.js');
  importScripts('https://www.gstatic.com/firebasejs/12.2.1/firebase-messaging-compat.js');
  console.log('[Admin SW] Firebase scripts imported successfully');
} catch (error) {
  console.error('[Admin SW] Failed to import Firebase scripts:', error);
}

// Listen for configuration from main thread
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'FIREBASE_CONFIG') {
    console.log('[Admin SW] Received Firebase config from main thread');
    firebaseConfig = event.data.config;
    initializeFirebase();
  }
});

function initializeFirebase() {
  if (!firebaseConfig || isFirebaseInitialized) return;

  try {
    // Check if Firebase is available
    if (typeof firebase === 'undefined') {
      console.error('[Admin SW] Firebase not available - scripts may not have loaded');
      return;
    }

    // Validate configuration - no fallback, must receive proper config
    if (!firebaseConfig || !firebaseConfig.apiKey) {
      console.error('[Admin SW] No Firebase configuration received from main thread');
      return;
    }
    
    const requiredFields = ['apiKey', 'authDomain', 'projectId', 'messagingSenderId', 'appId'];

    for (const field of requiredFields) {
      if (!firebaseConfig[field]) {
        console.error(`[Admin SW] Missing Firebase configuration field: ${field}`);
        return;
      }
    }

    // Initialize Firebase
    firebase.initializeApp(firebaseConfig);
    messaging = firebase.messaging();
    isFirebaseInitialized = true;

    console.log('[Admin SW] Firebase messaging initialized successfully');

    // Handle background messages for admin
    messaging.onBackgroundMessage((payload) => {
      console.log('[Admin SW] Background message received:', payload);

      const notificationTitle = payload.notification?.title || 'EPSX Admin Alert';
      const notificationOptions = {
        body: payload.notification?.body || 'New admin notification',
        icon: payload.notification?.image || '/logo.png',
        badge: '/logo.png', 
        image: payload.notification?.image,
        data: {
          ...payload.data,
          click_action: payload.data?.click_action || '/admin',
          notification_id: payload.data?.notification_id,
          type: payload.data?.type || 'admin_notification'
        },
        actions: [
          {
            action: 'view',
            title: 'View in Admin'
          },
          {
            action: 'dismiss',
            title: 'Dismiss'
          }
        ],
        requireInteraction: true, // Admin notifications always require interaction
        tag: `admin-${payload.data?.type || 'default'}`,
        timestamp: Date.now(),
        vibrate: [300, 100, 300], // Stronger vibration for admin notifications
      };

      return self.registration.showNotification(notificationTitle, notificationOptions);
    });

  } catch (error) {
    console.error('[Admin SW] Firebase initialization failed:', error);
  }
}

// Handle admin notification click
self.addEventListener('notificationclick', (event) => {
  console.log('[Admin SW] Admin notification clicked:', event);

  const { data } = event.notification;
  event.notification.close();

  if (event.action === 'dismiss') {
    return;
  }

  // Ensure admin URLs start with /admin or are absolute admin URLs
  let clickAction = data?.click_action || '/admin';
  if (!clickAction.startsWith('/admin') && !clickAction.startsWith('http')) {
    clickAction = '/admin' + clickAction;
  }

  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true })
      .then((clientList) => {
        // Look for admin window specifically
        for (const client of clientList) {
          if (client.url.includes('/admin') && 'focus' in client) {
            client.navigate(clickAction);
            return client.focus();
          }
        }
        
        // Open admin window
        if (clients.openWindow) {
          return clients.openWindow(clickAction);
        }
      })
      .catch(err => {
        console.error('[Admin SW] Error handling admin notification click:', err);
      })
  );

  // Track admin notification click
  if (data?.notification_id) {
    fetch('/admin/api/v1/notifications/track', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        notification_id: data.notification_id,
        action: 'clicked',
        timestamp: new Date().toISOString(),
        context: 'admin'
      })
    }).catch(err => console.log('[Admin SW] Failed to track admin notification click:', err));
  }
});

// Handle notification close
self.addEventListener('notificationclose', (event) => {
  console.log('[Admin SW] Admin notification closed:', event);
  
  const { data } = event.notification;
  
  if (data?.notification_id) {
    fetch('/admin/api/v1/notifications/track', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        notification_id: data.notification_id,
        action: 'dismissed',
        timestamp: new Date().toISOString(),
        context: 'admin'
      })
    }).catch(err => console.log('[Admin SW] Failed to track admin notification dismissal:', err));
  }
});

// Service worker lifecycle events
self.addEventListener('install', (event) => {
  console.log('[Admin SW] Admin service worker installing');
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  console.log('[Admin SW] Admin service worker activating');
  event.waitUntil(self.clients.claim());
});

// Handle push event (fallback)
self.addEventListener('push', (event) => {
  console.log('[Admin SW] Push received:', event);
  
  if (event.data) {
    const payload = event.data.json();
    const title = payload.notification?.title || 'EPSX Admin Alert';
    const options = {
      body: payload.notification?.body || 'New admin notification',
      icon: '/logo.png',
      badge: '/logo.png',
      data: payload.data
    };
    
    event.waitUntil(
      self.registration.showNotification(title, options)
    );
  }
});