import { initializeApp, getApp, getApps } from 'firebase/app';
import { Analytics, getAnalytics, isSupported } from 'firebase/analytics';

const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID,
};

// Initialize Firebase
const app = getApps().length > 0 ? getApp() : initializeApp(firebaseConfig);

if (process.env.NODE_ENV !== 'production') {
  if (!firebaseConfig.apiKey) {
    console.error('Firebase API Key is missing. Make sure to set NEXT_PUBLIC_FIREBASE_API_KEY in your environment variables.');
  }
}

// Initialize analytics if supported (not available during SSR)
let analytics: Analytics | null = null;
if (typeof window !== 'undefined') {
  isSupported()
    .then(yes => {
      if (yes) {
        analytics = getAnalytics(app);
      }
    })
    .catch(err => {
      console.error('Failed to initialize Firebase Analytics:', err);
    });
}

export { app, analytics };
