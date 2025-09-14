import { initializeApp, getApps, FirebaseApp } from 'firebase/app';
import { env } from '../../../shared/env/schema';
// Future analytics imports (commented for now):
// import { getAnalytics, Analytics } from 'firebase/analytics';
// import { getPerformance, Performance } from 'firebase/performance';

const firebaseConfig = {
  apiKey: env.FIREBASE_API_KEY,
  authDomain: env.FIREBASE_AUTH_DOMAIN,
  projectId: env.FIREBASE_PROJECT_ID,
  storageBucket: env.FIREBASE_STORAGE_BUCKET,
  messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
  appId: env.FIREBASE_APP_ID,
  measurementId: env.FIREBASE_MEASUREMENT_ID
};

// Debug logging for Firebase configuration
if (typeof window !== 'undefined') {
  console.log('🔥 Firebase Configuration:', {
    apiKey: env.FIREBASE_API_KEY ? `${env.FIREBASE_API_KEY.substring(0, 10)}...` : 'undefined',
    authDomain: env.FIREBASE_AUTH_DOMAIN,
    projectId: env.FIREBASE_PROJECT_ID,
    storageBucket: env.FIREBASE_STORAGE_BUCKET,
    messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
    appId: env.FIREBASE_APP_ID ? `${env.FIREBASE_APP_ID.substring(0, 15)}...` : 'undefined',
  });
  
  // Check if API key is valid before initializing
  if (!env.FIREBASE_API_KEY || env.FIREBASE_API_KEY.length < 20) {
    console.warn('🚨 Firebase API key appears invalid, skipping Firebase initialization');
  }
}

let firebaseApp: FirebaseApp | null = null;
// Future analytics variables (commented for now):
// let analytics: Analytics | null = null;
// let performance: Performance | null = null;

// Initialize Firebase only if API key is valid
try {
  if (!getApps().length && env.FIREBASE_API_KEY && env.FIREBASE_API_KEY.length > 20) {
    firebaseApp = initializeApp(firebaseConfig);
    console.log('✅ Firebase initialized successfully');
  } else if (getApps().length > 0) {
    firebaseApp = getApps()[0];
  } else {
    console.warn('⚠️ Firebase not initialized - invalid API key or missing configuration');
  }
} catch (error) {
  console.error('❌ Firebase initialization failed:', error);
  firebaseApp = null;
}

// Future analytics initialization (commented for now):
// if (typeof window !== 'undefined') {
//   try {
//     analytics = getAnalytics(firebaseApp);
//     performance = getPerformance(firebaseApp);
//   } catch (error) {
//     console.warn('Firebase analytics/performance not available:', error);
//   }
// }

export { firebaseApp };
// Future exports:
// export { analytics, performance };

// Safe Firebase app export - may be null if initialization failed
export const getFirebaseApp = () => firebaseApp;

// FCM functionality moved to backend - frontend only keeps Firebase app for future analytics
// All notification handling now done via server components and backend API