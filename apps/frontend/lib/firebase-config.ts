import { initializeApp, getApps, FirebaseApp } from 'firebase/app';
// Future analytics imports (commented for now):
// import { getAnalytics, Analytics } from 'firebase/analytics';
// import { getPerformance, Performance } from 'firebase/performance';

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
// Future analytics variables (commented for now):
// let analytics: Analytics | null = null;
// let performance: Performance | null = null;

// Initialize Firebase
if (!getApps().length) {
  firebaseApp = initializeApp(firebaseConfig);
} else {
  firebaseApp = getApps()[0];
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

// FCM functionality moved to backend - frontend only keeps Firebase app for future analytics
// All notification handling now done via server components and backend API