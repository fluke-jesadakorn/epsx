// Import Firebase scripts for service worker (v10+ compatible)
importScripts('https://www.gstatic.com/firebasejs/10.7.0/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/10.7.0/firebase-messaging-compat.js');

// Firebase configuration for admin interface
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

// Cache for notification click tracking (admin-specific)
let adminNotificationQueue = [];

// Handle background messages for admin interface
messaging.onBackgroundMessage(function(payload) {
  console.log('Admin FCM background message received:', payload);

  const notificationTitle = payload.notification?.title || payload.data?.title || 'EPSX Admin Notification';
  const notificationOptions = {
    body: payload.notification?.body || payload.data?.body || 'You have a new admin notification',
    icon: payload.notification?.icon || payload.data?.icon || '/logo.png',
    badge: '/logo.png',
    tag: payload.data?.tag || `epsx-admin-${Date.now()}`,
    timestamp: Date.now(),
    data: {
      url: payload.data?.url || payload.fcmOptions?.link || '/notifications',
      notificationId: payload.data?.notificationId || Math.random().toString(36),
      type: payload.data?.type || 'admin',
      adminType: payload.data?.adminType || 'system', // user, system, security, analytics
      userId: payload.data?.userId,
      trackingEnabled: payload.data?.trackingEnabled !== 'false',
      isAdminNotification: true,
      ...payload.data
    },
    actions: payload.data?.actions ? JSON.parse(payload.data.actions) : [
      {
        action: 'view',
        title: 'Open Admin',
        icon: '/logo.png'
      },
      {
        action: 'dismiss',
        title: 'Dismiss'
      }
    ],
    requireInteraction: payload.data?.requireInteraction === 'true' || payload.data?.adminType === 'security',
    silent: payload.data?.silent === 'true',
    vibrate: payload.data?.vibrate ? JSON.parse(payload.data.vibrate) : [200, 100, 200],
    renotify: payload.data?.renotify === 'true',
    dir: 'ltr',
    lang: 'en'
  };

  // Store notification for analytics
  if (notificationOptions.data.trackingEnabled) {
    try {
      adminNotificationQueue.push({
        notificationId: notificationOptions.data.notificationId,
        action: 'received',
        timestamp: Date.now(),
        payload: payload,
        context: 'admin'
      });
    } catch (error) {
      console.error('Error storing admin notification tracking data:', error);
    }
  }

  // Show notification
  return self.registration.showNotification(notificationTitle, notificationOptions);
});

// Handle notification click events for admin interface
self.addEventListener('notificationclick', function(event) {
  console.log('Admin notification clicked:', event);
  
  const notification = event.notification;
  const action = event.action;
  const data = notification.data || {};

  // Track notification interaction
  if (data.trackingEnabled && data.notificationId) {
    try {
      adminNotificationQueue.push({
        notificationId: data.notificationId,
        action: action || 'click',
        timestamp: Date.now(),
        url: data.url,
        context: 'admin'
      });
      
      // Send tracking data to backend (async)
      sendAdminTrackingData(data.notificationId, action || 'click', data.url);
    } catch (error) {
      console.error('Error tracking admin notification click:', error);
    }
  }

  notification.close();

  if (action === 'dismiss') {
    return;
  }

  // Determine URL to open based on admin notification type
  let urlToOpen = data.url || '/notifications';
  
  // Smart routing for admin-specific notifications
  if (data.adminType === 'user' && !data.url) {
    urlToOpen = '/users';
  } else if (data.adminType === 'system' && !data.url) {
    urlToOpen = '/system';
  } else if (data.adminType === 'security' && !data.url) {
    urlToOpen = '/permissions';
  } else if (data.adminType === 'analytics' && !data.url) {
    urlToOpen = '/analytics';
  }

  event.waitUntil(
    handleAdminNotificationNavigation(urlToOpen, data)
  );
});

// Enhanced navigation handling for admin interface
async function handleAdminNotificationNavigation(urlToOpen, data) {
  try {
    const clientList = await clients.matchAll({
      type: 'window',
      includeUncontrolled: true
    });

    // Try to find existing admin window/tab (port 3001 or admin subdomain)
    for (const client of clientList) {
      if (client.url.includes('localhost:3001') || 
          client.url.includes('admin') ||
          client.url.includes(self.location.origin)) {
        
        // Focus existing admin window and navigate if needed
        await client.focus();
        
        // Send message to client to navigate to specific URL
        if (client.postMessage) {
          client.postMessage({
            type: 'ADMIN_FCM_NOTIFICATION_CLICK',
            url: urlToOpen,
            data: data,
            adminType: data.adminType
          });
        }
        
        return;
      }
    }
    
    // No existing admin window found, open new one
    if (clients.openWindow) {
      await clients.openWindow(urlToOpen);
    }
  } catch (error) {
    console.error('Error handling admin notification navigation:', error);
    
    // Fallback: try to open any window
    try {
      if (clients.openWindow) {
        await clients.openWindow(urlToOpen);
      }
    } catch (fallbackError) {
      console.error('Admin navigation fallback also failed:', fallbackError);
    }
  }
}

// Send tracking data to backend (admin context)
async function sendAdminTrackingData(notificationId, action, url) {
  try {
    const trackingData = {
      notificationId,
      action,
      url,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      context: 'admin',
      platform: 'web'
    };

    await fetch('/api/v1/notifications/analytics/track', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(trackingData)
    });
  } catch (error) {
    console.error('Failed to send admin notification tracking data:', error);
  }
}

// Handle notification close events
self.addEventListener('notificationclose', function(event) {
  console.log('Admin notification closed:', event);
  
  const notification = event.notification;
  const data = notification.data || {};
  
  // Track notification close events for admin notifications
  if (data.trackingEnabled && data.notificationId) {
    sendAdminTrackingData(data.notificationId, 'close', data.url);
  }
});

// Handle push events (for custom push handling)
self.addEventListener('push', function(event) {
  console.log('Admin push event received:', event);
  
  if (!event.data) {
    console.log('Admin push event has no data');
    return;
  }

  try {
    const data = event.data.json();
    console.log('Admin push data:', data);
    
    // This will be handled by Firebase messaging's onBackgroundMessage
    // but we can add admin-specific logic here if needed
  } catch (error) {
    console.error('Error parsing admin push data:', error);
  }
});

// Periodic sync for admin notification queue (if supported)
if ('serviceWorker' in navigator && 'sync' in window.ServiceWorkerRegistration.prototype) {
  self.addEventListener('sync', function(event) {
    if (event.tag === 'admin-fcm-tracking-sync') {
      event.waitUntil(syncAdminNotificationData());
    }
  });
}

// Sync admin notification tracking data
async function syncAdminNotificationData() {
  if (adminNotificationQueue.length === 0) return;

  try {
    const dataToSync = [...adminNotificationQueue];
    adminNotificationQueue = []; // Clear the queue

    for (const trackingData of dataToSync) {
      await sendAdminTrackingData(
        trackingData.notificationId, 
        trackingData.action, 
        trackingData.url
      );
    }
  } catch (error) {
    console.error('Error syncing admin notification data:', error);
    // Re-add failed items to queue (with limit)
    if (adminNotificationQueue.length < 50) {
      adminNotificationQueue.unshift(...dataToSync.slice(0, 10));
    }
  }
}

// Listen for messages from admin main thread
self.addEventListener('message', function(event) {
  console.log('Admin service worker received message:', event.data);
  
  if (event.data && event.data.type) {
    switch (event.data.type) {
      case 'SKIP_WAITING':
        self.skipWaiting();
        break;
      case 'GET_VERSION':
        event.ports[0].postMessage({
          type: 'VERSION_RESPONSE',
          version: '1.0.0-admin',
          timestamp: Date.now()
        });
        break;
      case 'SYNC_ADMIN_TRACKING_DATA':
        syncAdminNotificationData();
        break;
      case 'CLEAR_ADMIN_TRACKING_QUEUE':
        adminNotificationQueue = [];
        break;
    }
  }
});

// Service worker activation
self.addEventListener('activate', function(event) {
  console.log('Admin Firebase messaging service worker activated');
  
  event.waitUntil(
    Promise.all([
      // Clean up old caches if any
      self.clients.claim(),
      // Sync any pending admin tracking data
      syncAdminNotificationData()
    ])
  );
});

// Service worker installation
self.addEventListener('install', function(event) {
  console.log('Admin Firebase messaging service worker installed');
  
  event.waitUntil(
    // Pre-cache essential assets if needed
    Promise.resolve().then(() => {
      self.skipWaiting();
    })
  );
});

// Handle service worker errors
self.addEventListener('error', function(event) {
  console.error('Admin service worker error:', event.error);
});

// Handle unhandled promise rejections
self.addEventListener('unhandledrejection', function(event) {
  console.error('Admin service worker unhandled promise rejection:', event.reason);
  event.preventDefault();
});