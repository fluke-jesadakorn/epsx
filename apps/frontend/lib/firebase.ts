import { getAnalytics, isSupported } from 'firebase/analytics';
import { initializeApp, getApps, getApp } from 'firebase/app';
import { getAuth, onAuthStateChanged  } from 'firebase/auth';

import type {User as FirebaseUser} from 'firebase/auth';

const firebaseConfig = {
  apiKey: "AIzaSyBof2MIWdFMfpvfl21Di2fOH08ElTgAurU",
  authDomain: "epsx-449804.firebaseapp.com",
  projectId: "epsx-449804",
  storageBucket: "epsx-449804.firebasestorage.app",
  messagingSenderId: "351896526537",
  appId: "1:351896526537:web:c8ebf7f209ca7430b9f718",
  measurementId: "G-FQWE9G37ZE"
};

const app = getApps().length ? getApp() : initializeApp(firebaseConfig);
const auth = getAuth(app);

// Initialize analytics only in browser environment and when supported
const analytics = typeof window !== 'undefined' 
  ? isSupported().then(() => getAnalytics(app)) 
  : null;

export { auth, app, analytics };

export type { FirebaseUser };

export const watchAuthState = (callback: (user: FirebaseUser | null) => void) => {
  return onAuthStateChanged(auth, callback);
};
