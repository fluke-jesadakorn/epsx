/**
 * Firebase Analytics Configuration (Minimal)
 * Only keeps analytics configuration - FCM and other services removed
 */

// ============================================================================
// Firebase Analytics Configuration
// ============================================================================

export interface FirebaseConfig {
  apiKey: string;
  authDomain: string;
  projectId: string;
  appId: string;
  measurementId?: string;
}

export function getFirebaseConfig(): FirebaseConfig {
  return {
    apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY || '',
    authDomain: `${process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID}.firebaseapp.com`,
    projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID || '',
    appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID || '',
    measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
  };
}

// ============================================================================
// Firebase Analytics (Disabled)
// ============================================================================

export function isFirebaseInitialized(): boolean {
  return false; // Analytics disabled for security
}

export function isServiceAvailable(service: string): boolean {
  return false; // All services disabled
}

export function trackEvent(eventName: string, parameters?: Record<string, any>): void {
  // No-op - analytics disabled
}

export function getRemoteConfigValue(key: string): any {
  return null; // Remote config disabled
}

export async function fetchRemoteConfig(): Promise<boolean> {
  return false; // Remote config disabled
}

