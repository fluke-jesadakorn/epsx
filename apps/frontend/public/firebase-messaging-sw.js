// Import Firebase scripts for service worker (v9+ modular SDK)
importScripts('https://www.gstatic.com/firebasejs/10.7.0/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/10.7.0/firebase-messaging-compat.js');

// Firebase configuration - using environment variables pattern
const firebaseConfig = {
  apiKey: "AIzaSyBof2MIWdFMfpvfl21Di2fOH08ElTgAurU",
  authDomain: "epsx-449804.firebaseapp.com", 
  projectId: "epsx-449804",
  storageBucket: "epsx-449804.firebasestorage.app",
  messagingSenderId: "1000000000000",
  appId: "1:1000000000000:web:abcd1234567890",
  measurementId: "G-MEASUREMENT_ID"
};

// Initialize Firebase in service worker
firebase.initializeApp(firebaseConfig);

// Get Firebase Messaging instance
const messaging = firebase.messaging();

// Cache for notification click tracking
let notificationClickQueue = [];

// Handle background messages
messaging.onBackgroundMessage(function(payload) {
  console.log('Background message received:', payload);

  const notificationTitle = payload.notification?.title || payload.data?.title || 'EPSX Notification';
  const notificationOptions = {
    body: payload.notification?.body || payload.data?.body || 'You have a new notification from EPSX',
    icon: payload.notification?.icon || payload.data?.icon || '/logo.png',
    badge: '/logo.png',
    tag: payload.data?.tag || `epsx-${Date.now()}`,
    timestamp: Date.now(),
    data: {
      url: payload.data?.url || payload.fcmOptions?.link || '/',
      notificationId: payload.data?.notificationId || Math.random().toString(36),
      type: payload.data?.type || 'system',
      userId: payload.data?.userId,
      trackingEnabled: payload.data?.trackingEnabled !== 'false',
      ...payload.data
    },
    actions: payload.data?.actions ? JSON.parse(payload.data.actions) : [
      {
        action: 'view',
        title: 'Open',
        icon: '/logo.png'
      },
      {
        action: 'dismiss',
        title: 'Dismiss'
      }
    ],
    requireInteraction: payload.data?.requireInteraction === 'true',
    silent: payload.data?.silent === 'true',
    vibrate: payload.data?.vibrate ? JSON.parse(payload.data.vibrate) : [200, 100, 200],
    renotify: payload.data?.renotify === 'true',
    dir: 'ltr',
    lang: 'en'
  };

  // Store notification for analytics
  if (notificationOptions.data.trackingEnabled) {
    try {
      notificationClickQueue.push({
        notificationId: notificationOptions.data.notificationId,
        action: 'received',
        timestamp: Date.now(),
        payload: payload
      });
    } catch (error) {
      console.error('Error storing notification tracking data:', error);
    }
  }

  // Show notification
  return self.registration.showNotification(notificationTitle, notificationOptions);
});

// Handle notification click events
self.addEventListener('notificationclick', function(event) {
  console.log('Notification clicked:', event);
  
  const notification = event.notification;
  const action = event.action;
  const data = notification.data || {};

  // Track notification interaction
  if (data.trackingEnabled && data.notificationId) {
    try {
      notificationClickQueue.push({
        notificationId: data.notificationId,
        action: action || 'click',
        timestamp: Date.now(),
        url: data.url
      });
      
      // Send tracking data to backend (async)
      sendTrackingData(data.notificationId, action || 'click', data.url);
    } catch (error) {
      console.error('Error tracking notification click:', error);
    }
  }

  notification.close();

  if (action === 'dismiss') {
    return;
  }

  // Determine URL to open based on notification type
  let urlToOpen = data.url || '/notifications';
  
  // Smart routing based on notification type
  if (data.type === 'trading' && !data.url) {
    urlToOpen = '/analytics';
  } else if (data.type === 'account' && !data.url) {
    urlToOpen = '/settings';
  } else if (data.type === 'system' && !data.url) {
    urlToOpen = '/notifications';
  }

  event.waitUntil(
    handleNotificationNavigation(urlToOpen, data)
  );
});

// Enhanced navigation handling with fallback mechanisms
async function handleNotificationNavigation(urlToOpen, data) {
  try {
    const clientList = await clients.matchAll({
      type: 'window',
      includeUncontrolled: true
    });

    // Try to find existing EPSX window/tab
    for (const client of clientList) {
      if (client.url.includes('localhost:3000') || 
          client.url.includes(self.location.origin) ||
          client.url.includes('epsx')) {
        
        // Focus existing window and navigate if needed
        await client.focus();
        
        // Send message to client to navigate to specific URL
        if (client.postMessage) {
          client.postMessage({
            type: 'FCM_NOTIFICATION_CLICK',
            url: urlToOpen,
            data: data
          });
        }
        
        return;
      }
    }
    
    // No existing window found, open new one
    if (clients.openWindow) {
      await clients.openWindow(urlToOpen);
    }
  } catch (error) {
    console.error('Error handling notification navigation:', error);
    
    // Fallback: try to open any window
    try {
      if (clients.openWindow) {
        await clients.openWindow(urlToOpen);
      }
    } catch (fallbackError) {
      console.error('Fallback navigation also failed:', fallbackError);
    }
  }
}

// Send tracking data to backend
async function sendTrackingData(notificationId, action, url) {
  try {
    const trackingData = {
      notificationId,
      action,
      url,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent
    };

    await fetch('/api/v1/notifications/analytics/track', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(trackingData)
    });
  } catch (error) {
    console.error('Failed to send notification tracking data:', error);
  }
}

// Handle notification close events
self.addEventListener('notificationclose', function(event) {
  console.log('Notification closed:', event);
  
  const notification = event.notification;
  const data = notification.data || {};
  
  // Optional: Track notification close events
  if (data.trackClose && data.notificationId) {
    // Send tracking data to analytics
    fetch('/api/v1/notifications/analytics/close', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        notificationId: data.notificationId,
        closedAt: new Date().toISOString()
      })
    }).catch(error => {
      console.error('Failed to track notification close:', error);
    });
  }
});

// Handle push events (for custom push handling)
self.addEventListener('push', function(event) {
  console.log('Push event received:', event);
  
  if (!event.data) {
    console.log('Push event has no data');
    return;
  }

  try {
    const data = event.data.json();
    console.log('Push data:', data);
    
    // This will be handled by Firebase messaging's onBackgroundMessage
    // but we can add custom logic here if needed
  } catch (error) {
    console.error('Error parsing push data:', error);
  }
});

// Periodic sync for notification queue (if supported)
if ('serviceWorker' in navigator && 'sync' in window.ServiceWorkerRegistration.prototype) {
  self.addEventListener('sync', function(event) {
    if (event.tag === 'fcm-tracking-sync') {
      event.waitUntil(syncNotificationData());
    }
  });
}

// Sync notification tracking data
async function syncNotificationData() {
  if (notificationClickQueue.length === 0) return;

  try {
    const dataToSync = [...notificationClickQueue];
    notificationClickQueue = []; // Clear the queue

    for (const trackingData of dataToSync) {
      await sendTrackingData(
        trackingData.notificationId, 
        trackingData.action, 
        trackingData.url
      );
    }
  } catch (error) {
    console.error('Error syncing notification data:', error);
    // Re-add failed items to queue (with limit)
    if (notificationClickQueue.length < 50) {
      notificationClickQueue.unshift(...dataToSync.slice(0, 10));
    }
  }
}

// Listen for messages from main thread
self.addEventListener('message', function(event) {
  console.log('Service worker received message:', event.data);
  
  if (event.data && event.data.type) {
    switch (event.data.type) {
      case 'SKIP_WAITING':
        self.skipWaiting();
        break;
      case 'GET_VERSION':
        event.ports[0].postMessage({
          type: 'VERSION_RESPONSE',
          version: '1.0.0',
          timestamp: Date.now()
        });
        break;
      case 'SYNC_TRACKING_DATA':
        syncNotificationData();
        break;
      case 'CLEAR_TRACKING_QUEUE':
        notificationClickQueue = [];
        break;
    }
  }
});

// Service worker activation
self.addEventListener('activate', function(event) {
  console.log('Firebase messaging service worker activated');
  
  event.waitUntil(
    Promise.all([
      // Clean up old caches if any
      self.clients.claim(),
      // Sync any pending tracking data
      syncNotificationData()
    ])
  );
});

// Service worker installation
self.addEventListener('install', function(event) {
  console.log('Firebase messaging service worker installed');
  
  event.waitUntil(
    // Pre-cache essential assets if needed
    Promise.resolve().then(() => {
      self.skipWaiting();
    })
  );
});

// Handle service worker errors
self.addEventListener('error', function(event) {
  console.error('Service worker error:', event.error);
});

// Handle unhandled promise rejections
self.addEventListener('unhandledrejection', function(event) {
  console.error('Service worker unhandled promise rejection:', event.reason);
  event.preventDefault();
});