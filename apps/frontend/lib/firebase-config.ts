import { initializeApp, getApps, FirebaseApp } from 'firebase/app';
import { getMessaging, getToken, onMessage, Messaging, deleteToken, onTokenRefresh } from 'firebase/messaging';

const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
};

let firebaseApp: FirebaseApp;
let messaging: Messaging | null = null;

// Initialize Firebase
if (!getApps().length) {
  firebaseApp = initializeApp(firebaseConfig);
} else {
  firebaseApp = getApps()[0];
}

// Initialize Firebase Cloud Messaging and get a reference to the service
// Only initialize messaging in the browser and if supported
if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
  try {
    messaging = getMessaging(firebaseApp);
  } catch (error) {
    console.warn('Firebase messaging not available:', error);
  }
}

export { firebaseApp, messaging };

export const vapidKey = process.env.NEXT_PUBLIC_FIREBASE_VAPID_KEY;

export interface FCMToken {
  token: string;
  createdAt: Date;
  userAgent: string;
}

export interface FCMSubscription {
  token: string;
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

// Request notification permission and get FCM token
export async function requestNotificationPermission(): Promise<string | null> {
  if (!messaging) {
    console.warn('Firebase messaging not initialized');
    return null;
  }

  try {
    // Check if notification permissions are already granted
    if (Notification.permission === 'granted') {
      return await getFCMToken();
    }

    // Request permission
    const permission = await Notification.requestPermission();
    
    if (permission === 'granted') {
      return await getFCMToken();
    } else {
      console.warn('Notification permission denied');
      return null;
    }
  } catch (error) {
    console.error('Error requesting notification permission:', error);
    return null;
  }
}

// Get FCM registration token
export async function getFCMToken(): Promise<string | null> {
  if (!messaging || !vapidKey) {
    console.warn('Firebase messaging or VAPID key not available');
    return null;
  }

  try {
    const token = await getToken(messaging, {
      vapidKey: vapidKey
    });
    
    if (token) {
      console.log('FCM registration token:', token);
      return token;
    } else {
      console.warn('No FCM registration token available');
      return null;
    }
  } catch (error) {
    console.error('Error getting FCM token:', error);
    return null;
  }
}

// Set up foreground message handler
export function onForegroundMessage(callback: (payload: any) => void) {
  if (!messaging) {
    console.warn('Firebase messaging not initialized');
    return () => {};
  }

  try {
    return onMessage(messaging, (payload) => {
      console.log('Foreground message received:', payload);
      callback(payload);
    });
  } catch (error) {
    console.error('Error setting up foreground message handler:', error);
    return () => {};
  }
}

// Check if push notifications are supported
export function isPushNotificationSupported(): boolean {
  return (
    typeof window !== 'undefined' &&
    'serviceWorker' in navigator &&
    'PushManager' in window &&
    'Notification' in window &&
    messaging !== null
  );
}

// Get current notification permission status
export function getNotificationPermissionStatus(): NotificationPermission | null {
  if (typeof window === 'undefined' || !('Notification' in window)) {
    return null;
  }
  return Notification.permission;
}

// Check if the browser supports FCM
export function isFCMSupported(): boolean {
  return isPushNotificationSupported() && !!vapidKey;
}