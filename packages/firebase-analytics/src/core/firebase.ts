import { initializeApp, getApps } from 'firebase/app';
import { getAnalytics, Analytics } from 'firebase/analytics';

const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
};

let analytics: Analytics | null = null;

export const initializeFirebaseAnalytics = (): Analytics | null => {
  if (typeof window !== 'undefined' && !getApps().length) {
    const app = initializeApp(firebaseConfig);
    analytics = getAnalytics(app);
    return analytics;
  }
  return analytics;
};

export const getFirebaseAnalytics = (): Analytics | null => {
  if (typeof window !== 'undefined' && !analytics) {
    analytics = initializeFirebaseAnalytics();
  }
  return analytics;
};