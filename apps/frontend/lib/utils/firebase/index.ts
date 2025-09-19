/**
 * Firebase Utilities
 * Consolidated Firebase configuration, analytics, and FCM utilities
 */

// Simplified error handling to avoid circular imports
function safeError(error: unknown): { message: string } {
  if (error instanceof Error) {
    return { message: error.message };
  }
  if (typeof error === 'string') {
    return { message: error };
  }
  return { message: 'Unknown error occurred' };
}

// ============================================================================
// Firebase Configuration
// ============================================================================

export interface FirebaseConfig {
  apiKey: string;
  authDomain: string;
  projectId: string;
  storageBucket: string;
  messagingSenderId: string;
  appId: string;
  measurementId?: string;
}

export function getFirebaseConfig(): FirebaseConfig {
  return {
    apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY || '',
    authDomain: `${process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID}.firebaseapp.com`,
    projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID || '',
    storageBucket: `${process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID}.appspot.com`,
    messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID || '',
    appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID || '',
    measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
  };
}

// ============================================================================
// Firebase Analytics
// ============================================================================

export function initFirebaseAnalytics() {
  if (typeof window === 'undefined') return;
  
  // Analytics initialization would go here
  // Simplified for refactoring
}

export function logAnalyticsEvent(eventName: string, parameters?: Record<string, any>) {
  if (typeof window === 'undefined') return;
  
  try {
    // Analytics event logging would go here
  } catch (error) {
    console.warn('Failed to log analytics event:', safeError(error).message);
  }
}

// ============================================================================
// Firebase Cloud Messaging (FCM)
// ============================================================================

export interface FCMConfig {
  vapidKey: string;
  swUrl: string;
}

export class FCMClient {
  private config: FCMConfig;
  private registration: ServiceWorkerRegistration | null = null;

  constructor(config: FCMConfig) {
    this.config = config;
  }

  async initialize(): Promise<void> {
    if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
      throw new Error('Service Worker not supported');
    }

    try {
      this.registration = await navigator.serviceWorker.register(this.config.swUrl);
    } catch (error) {
      throw new Error(`Service Worker registration failed: ${safeError(error).message}`);
    }
  }

  async requestPermission(): Promise<NotificationPermission> {
    if (!('Notification' in window)) {
      throw new Error('Notifications not supported');
    }

    const permission = await Notification.requestPermission();
    return permission;
  }

  async getToken(): Promise<string | null> {
    if (!this.registration) {
      throw new Error('FCM not initialized');
    }

    try {
      // FCM token retrieval would go here
      // Simplified for refactoring
      return 'mock-fcm-token';
    } catch (error) {
      console.error('Failed to get FCM token:', safeError(error).message);
      return null;
    }
  }

  async subscribeToTopic(token: string, topic: string): Promise<void> {
    try {
      // Topic subscription would go here
    } catch (error) {
      throw new Error(`Failed to subscribe to topic: ${safeError(error).message}`);
    }
  }

  onMessage(callback: (payload: any) => void): void {
    // Message listener would go here
    // Simplified for refactoring
  }
}

// ============================================================================
// FCM Error Handling
// ============================================================================

export class FCMErrorHandler {
  private static instance: FCMErrorHandler;
  private errorCallback?: (error: Error) => void;

  private constructor() {}

  static getInstance(): FCMErrorHandler {
    if (!FCMErrorHandler.instance) {
      FCMErrorHandler.instance = new FCMErrorHandler();
    }
    return FCMErrorHandler.instance;
  }

  setErrorCallback(callback: (error: Error) => void): void {
    this.errorCallback = callback;
  }

  handleError(error: any, context: string): void {
    const processedError = new Error(`FCM Error in ${context}: ${safeError(error).message}`);
    
    console.error('FCM Error:', processedError);
    
    if (this.errorCallback) {
      this.errorCallback(processedError);
    }
  }

  handleTokenError(error: any): void {
    this.handleError(error, 'Token Management');
  }

  handleMessageError(error: any): void {
    this.handleError(error, 'Message Handling');
  }

  handleSubscriptionError(error: any): void {
    this.handleError(error, 'Topic Subscription');
  }
}

// ============================================================================
// Remote Config
// ============================================================================

export interface RemoteConfigValue {
  asBoolean(): boolean;
  asString(): string;
  asNumber(): number;
  getSource(): 'static' | 'default' | 'remote';
}

export class RemoteConfigManager {
  private config: Record<string, any> = {};
  private initialized = false;

  async initialize(): Promise<void> {
    try {
      // Remote config initialization would go here
      // Simplified for refactoring
      this.config = {
        'feature_analytics_v2': true,
        'max_rankings_per_page': 50,
        'cache_duration_minutes': 15
      };
      
      this.initialized = true;
    } catch (error) {
      throw new Error(`Remote Config initialization failed: ${safeError(error).message}`);
    }
  }

  getValue(key: string): RemoteConfigValue {
    if (!this.initialized) {
      console.warn('Remote Config not initialized, returning default value');
    }

    const value = this.config[key];
    
    return {
      asBoolean: () => Boolean(value),
      asString: () => String(value),
      asNumber: () => Number(value),
      getSource: () => this.initialized ? 'remote' : 'default'
    };
  }

  async fetchAndActivate(): Promise<boolean> {
    try {
      // Config fetching would go here
      // Simplified for refactoring
      return true;
    } catch (error) {
      console.error('Failed to fetch remote config:', safeError(error).message);
      return false;
    }
  }

  getAll(): Record<string, RemoteConfigValue> {
    const result: Record<string, RemoteConfigValue> = {};
    
    for (const [key, value] of Object.entries(this.config)) {
      result[key] = this.getValue(key);
    }
    
    return result;
  }
}

// ============================================================================
// Remote Config Types and Defaults
// ============================================================================

export interface RemoteUserSettings {
  maxRankingsPerPage: number;
  enableAnalyticsV2: boolean;
  cacheDurationMinutes: number;
  enableNotifications: boolean;
  darkModeDefault: boolean;
}

export const defaultConfig: RemoteUserSettings = {
  maxRankingsPerPage: 50,
  enableAnalyticsV2: true,
  cacheDurationMinutes: 15,
  enableNotifications: true,
  darkModeDefault: false
};

// Remote Config API functions
export async function fetchRemoteConfig(): Promise<boolean> {
  return await remoteConfigManager.fetchAndActivate();
}

export function getAllRemoteSettings(): RemoteUserSettings {
  const all = remoteConfigManager.getAll();
  return {
    maxRankingsPerPage: all.max_rankings_per_page?.asNumber() || defaultConfig.maxRankingsPerPage,
    enableAnalyticsV2: all.feature_analytics_v2?.asBoolean() || defaultConfig.enableAnalyticsV2,
    cacheDurationMinutes: all.cache_duration_minutes?.asNumber() || defaultConfig.cacheDurationMinutes,
    enableNotifications: all.enable_notifications?.asBoolean() || defaultConfig.enableNotifications,
    darkModeDefault: all.dark_mode_default?.asBoolean() || defaultConfig.darkModeDefault
  };
}

export function getRemoteConfigStatus(): { initialized: boolean; lastFetch: number | null } {
  return {
    initialized: remoteConfigManager['initialized'] || false,
    lastFetch: Date.now() // Simplified
  };
}

// ============================================================================
// Exports
// ============================================================================

export const fcmClient = new FCMClient({
  vapidKey: process.env.NEXT_PUBLIC_FIREBASE_VAPID_KEY || '',
  swUrl: '/firebase-messaging-sw.js'
});

export const fcmErrorHandler = FCMErrorHandler.getInstance();
export const remoteConfigManager = new RemoteConfigManager();