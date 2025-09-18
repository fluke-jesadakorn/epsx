'use client'

/**
 * Firebase Analytics and Remote Config Only
 * Pure analytics implementation without authentication
 * Preserves Firebase Analytics, Remote Config, and FCM functionality
 */

import { initializeApp, getApps, FirebaseApp } from 'firebase/app'
import { 
  getRemoteConfig, 
  fetchAndActivate, 
  getValue, 
  getAll,
  RemoteConfig 
} from 'firebase/remote-config'
import { getAnalytics, Analytics, logEvent } from 'firebase/analytics'
import { getMessaging, Messaging, getToken, onMessage } from 'firebase/messaging'

import { config } from '@/config/env';

// Firebase configuration validation - prevent initialization with undefined or placeholder values
const validateFirebaseConfig = () => {
  const requiredFields = [
    { key: 'apiKey', value: config.firebase.apiKey, minLength: 35 },
    { key: 'projectId', value: config.firebase.projectId, minLength: 5 },
    { key: 'appId', value: config.firebase.appId, minLength: 15 }
  ];

  // Check for placeholder values that indicate misconfiguration
  const invalidPlaceholders = ['placeholder', 'undefined', 'null', 'test', 'dev'];

  const missing = requiredFields.filter(field => 
    !field.value || 
    field.value === 'undefined' || 
    field.value.length < field.minLength ||
    invalidPlaceholders.some(placeholder => field.value?.toLowerCase().includes(placeholder))
  );

  if (missing.length > 0) {
    const isDev = process.env.NODE_ENV === 'development';
    if (isDev) {
      console.error('❌ Firebase configuration validation failed. Missing or invalid fields:', 
        missing.map(f => f.key).join(', '));
      console.error('🔧 Firebase config values:', {
        apiKey: config.firebase.apiKey ? `${config.firebase.apiKey.substring(0, 10)}...` : 'undefined',
        authDomain: config.firebase.authDomain || 'undefined',
        projectId: config.firebase.projectId || 'undefined',
        storageBucket: config.firebase.storageBucket || 'undefined',
        messagingSenderId: config.firebase.messagingSenderId || 'undefined',
        appId: config.firebase.appId ? `${config.firebase.appId.substring(0, 15)}...` : 'undefined'
      });
    } else {
      console.warn('Firebase configuration incomplete - some features may be limited');
    }
    return false;
  }

  return true;
};

// Firebase configuration using unified environment schema (only if valid)
let firebaseConfig: any = null;
const isFirebaseConfigValid = validateFirebaseConfig();

if (isFirebaseConfigValid) {
  firebaseConfig = {
    apiKey: config.firebase.apiKey,
    authDomain: config.firebase.authDomain,
    projectId: config.firebase.projectId,
    storageBucket: config.firebase.storageBucket,
    messagingSenderId: config.firebase.messagingSenderId,
    appId: config.firebase.appId
  };
} else {
  if (process.env.NODE_ENV === 'development') {
    console.warn('🔧 Firebase initialization skipped due to invalid configuration');
    console.warn('💡 To enable Firebase features, ensure NEXT_PUBLIC_FIREBASE_* environment variables are properly set');
  }
}

// Initialize Firebase (singleton pattern) with error handling
let app: FirebaseApp | null = null;
let analytics: Analytics | null = null;
let messaging: Messaging | null = null;

if (firebaseConfig && isFirebaseConfigValid) {
  try {
    app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApps()[0];
    
    if (process.env.NODE_ENV === 'development') {
      console.log('✅ Firebase App initialized successfully (Analytics only)');
    }

    // Initialize Analytics (browser only)
    if (typeof window !== 'undefined') {
      try {
        analytics = getAnalytics(app);
        console.log('✅ Firebase Analytics initialized');
      } catch (error) {
        console.warn('⚠️ Firebase Analytics initialization failed:', error);
      }

      // Initialize Messaging (browser only)
      try {
        messaging = getMessaging(app);
        console.log('✅ Firebase Messaging initialized');
      } catch (error) {
        console.warn('⚠️ Firebase Messaging initialization failed:', error);
      }
    }
    
  } catch (error) {
    console.error('❌ Firebase initialization failed:', error);
    if (process.env.NODE_ENV === 'development') {
      console.warn('⚠️ Firebase features may be limited');
    }
    // Reset variables to null on failure
    app = null;
    analytics = null;
    messaging = null;
  }
} else {
  if (process.env.NODE_ENV === 'development') {
    console.warn('⚠️ Firebase not initialized - invalid or missing configuration');
    console.warn('📋 Required environment variables:');
    console.warn('  - NEXT_PUBLIC_FIREBASE_API_KEY (35+ chars)');
    console.warn('  - NEXT_PUBLIC_FIREBASE_PROJECT_ID (5+ chars)');
    console.warn('  - NEXT_PUBLIC_FIREBASE_APP_ID (15+ chars)');
  }
}

export { app, analytics, messaging };

// Initialize Firebase Remote Config with enhanced error handling
let remoteConfig: RemoteConfig | null = null;

if (app && isFirebaseConfigValid) {
  try {
    remoteConfig = getRemoteConfig(app);
    
    // Configure Remote Config settings with error handling
    if (remoteConfig && remoteConfig.settings) {
      remoteConfig.settings.minimumFetchIntervalMillis = 3600000 // 1 hour for production
      remoteConfig.settings.fetchTimeoutMillis = 60000 // 60 seconds
    }
    
    if (process.env.NODE_ENV === 'development') {
      console.log('✅ Firebase Remote Config initialized successfully');
    }
  } catch (error) {
    console.error('❌ Firebase Remote Config initialization failed:', error);
    remoteConfig = null;
  }
} else {
  // Ensure remoteConfig is explicitly null when Firebase is not available
  remoteConfig = null;
  if (process.env.NODE_ENV === 'development') {
    console.warn('⚠️ Remote Config not initialized - Firebase app not available or invalid configuration');
  }
}

export { remoteConfig };

// ============================================================================
// Firebase Analytics Methods
// ============================================================================

/**
 * Track custom analytics event
 */
export function trackEvent(eventName: string, parameters?: Record<string, any>): void {
  try {
    if (!analytics) {
      console.warn('⚠️ Firebase Analytics not available');
      return;
    }
    
    logEvent(analytics, eventName, parameters);
    console.log(`📊 Analytics event tracked: ${eventName}`, parameters);
  } catch (error) {
    console.error('❌ Failed to track analytics event:', error);
  }
}

/**
 * Track page view
 */
export function trackPageView(pageName: string, additionalParams?: Record<string, any>): void {
  trackEvent('page_view', {
    page_title: pageName,
    page_location: window.location.href,
    ...additionalParams
  });
}

/**
 * Track user action
 */
export function trackUserAction(action: string, category?: string, label?: string): void {
  trackEvent('user_action', {
    action,
    category,
    label,
    timestamp: Date.now()
  });
}

// ============================================================================
// Firebase Remote Config Methods
// ============================================================================

/**
 * Fetch and activate remote config
 */
export async function fetchRemoteConfig(): Promise<boolean> {
  try {
    if (!remoteConfig) {
      console.warn('⚠️ Remote Config not available');
      return false;
    }
    
    await fetchAndActivate(remoteConfig);
    console.log('✅ Remote Config fetched and activated');
    return true;
  } catch (error) {
    console.error('❌ Failed to fetch Remote Config:', error);
    return false;
  }
}

/**
 * Get remote config value
 */
export function getRemoteConfigValue(key: string): string {
  try {
    if (!remoteConfig) {
      console.warn('⚠️ Remote Config not available');
      return '';
    }
    
    const value = getValue(remoteConfig, key);
    return value.asString();
  } catch (error) {
    console.error(`❌ Failed to get Remote Config value for key: ${key}`, error);
    return '';
  }
}

/**
 * Get all remote config values
 */
export function getAllRemoteConfigValues(): Record<string, string> {
  try {
    if (!remoteConfig) {
      console.warn('⚠️ Remote Config not available');
      return {};
    }
    
    const allValues = getAll(remoteConfig);
    const result: Record<string, string> = {};
    
    Object.keys(allValues).forEach(key => {
      result[key] = allValues[key].asString();
    });
    
    return result;
  } catch (error) {
    console.error('❌ Failed to get all Remote Config values:', error);
    return {};
  }
}

// ============================================================================
// Firebase Cloud Messaging Methods
// ============================================================================

/**
 * Get FCM registration token
 */
export async function getFCMToken(): Promise<string | null> {
  try {
    if (!messaging) {
      console.warn('⚠️ Firebase Messaging not available');
      return null;
    }
    
    const token = await getToken(messaging, {
      vapidKey: process.env.NEXT_PUBLIC_FIREBASE_VAPID_KEY
    });
    
    if (token) {
      console.log('✅ FCM registration token obtained');
      return token;
    } else {
      console.warn('⚠️ No FCM registration token available');
      return null;
    }
  } catch (error) {
    console.error('❌ Failed to get FCM token:', error);
    return null;
  }
}

/**
 * Listen for FCM messages
 */
export function onFCMMessage(callback: (payload: any) => void): (() => void) | null {
  try {
    if (!messaging) {
      console.warn('⚠️ Firebase Messaging not available');
      return null;
    }
    
    const unsubscribe = onMessage(messaging, (payload) => {
      console.log('📩 FCM message received:', payload);
      callback(payload);
    });
    
    return unsubscribe;
  } catch (error) {
    console.error('❌ Failed to set up FCM message listener:', error);
    return null;
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Check if Firebase is properly initialized
 */
export function isFirebaseInitialized(): boolean {
  return !!app;
}

/**
 * Check if specific Firebase service is available
 */
export function isServiceAvailable(service: 'analytics' | 'messaging' | 'remoteConfig'): boolean {
  switch (service) {
    case 'analytics':
      return !!analytics;
    case 'messaging':
      return !!messaging;
    case 'remoteConfig':
      return !!remoteConfig;
    default:
      return false;
  }
}

/**
 * Get Firebase app instance
 */
export function getFirebaseApp(): FirebaseApp | null {
  return app;
}