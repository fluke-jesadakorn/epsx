import { initializeApp, getApps, FirebaseApp } from 'firebase/app';
import { getMessaging, getToken, onMessage, Messaging, deleteToken } from 'firebase/messaging';

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

// Initialize Firebase for admin interface
if (!getApps().length) {
  firebaseApp = initializeApp(firebaseConfig);
} else {
  firebaseApp = getApps()[0];
}

// Initialize Firebase Cloud Messaging for admin interface
// Only initialize messaging in the browser and if supported
if (typeof window !== 'undefined' && 'serviceWorker' in navigator && vapidKey) {
  try {
    messaging = getMessaging(firebaseApp);
    console.log('✅ Admin Firebase messaging initialized successfully');
  } catch (error) {
    console.warn('⚠️  Admin Firebase messaging not available:', error);
    messaging = null;
  }
} else {
  console.log('🔍 Admin Firebase messaging initialization skipped:', {
    hasWindow: typeof window !== 'undefined',
    hasServiceWorker: typeof window !== 'undefined' && 'serviceWorker' in navigator,
    hasVapidKey: !!vapidKey
  });
}

export { firebaseApp, messaging };

export const vapidKey = process.env.NEXT_PUBLIC_FIREBASE_VAPID_KEY;

export interface AdminFCMToken {
  token: string;
  createdAt: Date;
  userAgent: string;
  platform: string;
  isActive: boolean;
  adminPermissions?: string[];
}

export interface AdminFCMSubscription {
  token: string;
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
  adminContext: {
    permissions: string[];
    role: string;
  };
}

// Request notification permission and get FCM token for admin
export async function requestAdminNotificationPermission(): Promise<string | null> {
  if (!messaging) {
    console.warn('Admin Firebase messaging not initialized');
    return null;
  }

  try {
    // Check if notification permissions are already granted
    if (Notification.permission === 'granted') {
      return await getAdminFCMToken();
    }

    // Request permission with admin-specific message
    const permission = await Notification.requestPermission();
    
    if (permission === 'granted') {
      return await getAdminFCMToken();
    } else {
      console.warn('Admin notification permission denied');
      return null;
    }
  } catch (error) {
    console.error('Error requesting admin notification permission:', error);
    return null;
  }
}

// Get FCM registration token for admin
export async function getAdminFCMToken(): Promise<string | null> {
  if (!messaging || !vapidKey) {
    console.warn('Admin Firebase messaging or VAPID key not available');
    return null;
  }

  try {
    const token = await getToken(messaging, {
      vapidKey: vapidKey
    });
    
    if (token) {
      console.log('Admin FCM registration token:', token);
      return token;
    } else {
      console.warn('No admin FCM registration token available');
      return null;
    }
  } catch (error) {
    console.error('Error getting admin FCM token:', error);
    return null;
  }
}

// Set up foreground message handler for admin
export function onAdminForegroundMessage(callback: (payload: any) => void) {
  if (!messaging) {
    console.warn('Admin Firebase messaging not initialized');
    return () => {};
  }

  try {
    return onMessage(messaging, (payload) => {
      console.log('Admin foreground message received:', payload);
      // Add admin-specific context to the payload
      const adminPayload = {
        ...payload,
        adminContext: true,
        receivedAt: new Date().toISOString()
      };
      callback(adminPayload);
    });
  } catch (error) {
    console.error('Error setting up admin foreground message handler:', error);
    return () => {};
  }
}

// Delete admin FCM token
export async function deleteAdminFCMToken(): Promise<boolean> {
  if (!messaging) {
    console.warn('Admin Firebase messaging not initialized');
    return false;
  }

  try {
    const result = await deleteToken(messaging);
    console.log('Admin FCM token deleted:', result);
    return result;
  } catch (error) {
    console.error('Error deleting admin FCM token:', error);
    return false;
  }
}

// Set up token refresh listener for admin
export function onAdminTokenRefresh(callback: (token: string) => void) {
  if (!messaging) {
    console.warn('Admin Firebase messaging not initialized');
    return () => {};
  }

  try {
    // Implement periodic token check instead of onTokenRefresh (which doesn't exist)
    const intervalId = setInterval(async () => {
      try {
        const newToken = await getAdminFCMToken();
        if (newToken) {
          callback(newToken);
        }
      } catch (error) {
        console.error('Error checking admin token refresh:', error);
      }
    }, 60000); // Check every minute

    return () => clearInterval(intervalId);
  } catch (error) {
    console.error('Error setting up admin token refresh listener:', error);
    return () => {};
  }
}

// Check if push notifications are supported for admin
export function isAdminPushNotificationSupported(): boolean {
  return (
    typeof window !== 'undefined' &&
    'serviceWorker' in navigator &&
    'PushManager' in window &&
    'Notification' in window &&
    messaging !== null
  );
}

// Get current notification permission status
export function getAdminNotificationPermissionStatus(): NotificationPermission | null {
  if (typeof window === 'undefined' || !('Notification' in window)) {
    return null;
  }
  return Notification.permission;
}

// Check if admin FCM is supported
export function isAdminFCMSupported(): boolean {
  return isAdminPushNotificationSupported() && !!vapidKey;
}

// Admin-specific notification categories
export type AdminNotificationType = 
  | 'user_management'
  | 'system_alert'
  | 'security_warning'
  | 'analytics_report'
  | 'permission_change'
  | 'admin_message';

// Admin notification payload interface
export interface AdminNotificationPayload {
  notification?: {
    title?: string;
    body?: string;
    icon?: string;
  };
  data?: {
    adminType?: AdminNotificationType;
    userId?: string;
    permissionLevel?: string;
    url?: string;
    priority?: 'low' | 'normal' | 'high' | 'critical';
    [key: string]: any;
  };
  fcmOptions?: {
    link?: string;
  };
}

// Register admin service worker
export async function registerAdminServiceWorker(): Promise<ServiceWorkerRegistration | null> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
    console.warn('Service workers not supported for admin interface');
    return null;
  }

  try {
    const registration = await navigator.serviceWorker.register('/firebase-messaging-sw.js', {
      scope: '/firebase-cloud-messaging-push-scope'
    });

    console.log('Admin FCM service worker registered:', registration);
    return registration;
  } catch (error) {
    console.error('Admin service worker registration failed:', error);
    return null;
  }
}

// Admin-specific notification sound
export function playAdminNotificationSound(notificationType: AdminNotificationType) {
  if (typeof window === 'undefined' || !('Audio' in window)) {
    return;
  }

  try {
    let soundFile = '/sounds/notification.mp3'; // Default sound

    // Different sounds for different admin notification types
    switch (notificationType) {
      case 'security_warning':
        soundFile = '/sounds/alert.mp3';
        break;
      case 'system_alert':
        soundFile = '/sounds/system.mp3';
        break;
      case 'user_management':
        soundFile = '/sounds/user.mp3';
        break;
      default:
        soundFile = '/sounds/notification.mp3';
    }

    const audio = new Audio(soundFile);
    audio.volume = 0.5; // 50% volume
    audio.play().catch(error => {
      console.warn('Could not play admin notification sound:', error);
    });
  } catch (error) {
    console.error('Error playing admin notification sound:', error);
  }
}