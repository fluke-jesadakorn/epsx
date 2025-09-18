'use client'

/**
 * Firebase Authentication Integration for Admin Frontend
 * Uses unified environment schema for consistency with main frontend
 */

import { initializeApp, getApps } from 'firebase/app'
import { 
  getAuth, 
  GoogleAuthProvider, 
  signInWithPopup, 
  signInWithEmailAndPassword,
  createUserWithEmailAndPassword,
  signOut as firebaseSignOut,
  onAuthStateChanged,
  User as FirebaseUser,
  getIdToken
} from 'firebase/auth'

import { env } from '../../../shared/env/schema';

// Firebase configuration validation - prevent initialization with undefined or placeholder values
const validateFirebaseConfig = () => {
  const requiredFields = [
    { key: 'apiKey', value: env.FIREBASE_API_KEY, minLength: 35 },
    { key: 'projectId', value: env.FIREBASE_PROJECT_ID, minLength: 5 },
    { key: 'appId', value: env.FIREBASE_APP_ID, minLength: 15 }
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
        apiKey: env.FIREBASE_API_KEY ? `${env.FIREBASE_API_KEY.substring(0, 10)}...` : 'undefined',
        authDomain: env.FIREBASE_AUTH_DOMAIN || 'undefined',
        projectId: env.FIREBASE_PROJECT_ID || 'undefined',
        storageBucket: env.FIREBASE_STORAGE_BUCKET || 'undefined',
        messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID || 'undefined',
        appId: env.FIREBASE_APP_ID ? `${env.FIREBASE_APP_ID.substring(0, 15)}...` : 'undefined'
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
    apiKey: env.FIREBASE_API_KEY,
    authDomain: env.FIREBASE_AUTH_DOMAIN,
    projectId: env.FIREBASE_PROJECT_ID,
    storageBucket: env.FIREBASE_STORAGE_BUCKET,
    messagingSenderId: env.FIREBASE_MESSAGING_SENDER_ID,
    appId: env.FIREBASE_APP_ID
  };
} else {
  if (process.env.NODE_ENV === 'development') {
    console.warn('🔧 Firebase initialization skipped due to invalid configuration');
    console.warn('💡 To enable Firebase features, ensure NEXT_PUBLIC_FIREBASE_* environment variables are properly set');
  }
}

// Initialize Firebase (singleton pattern) with error handling
let app: any = null;
let auth: any = null;
let googleProvider: any = null;

if (firebaseConfig && isFirebaseConfigValid) {
  try {
    app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApps()[0];
    auth = getAuth(app);
    googleProvider = new GoogleAuthProvider();
    
    if (process.env.NODE_ENV === 'development') {
      console.log('✅ Firebase Auth initialized successfully for admin');
    }
  } catch (error) {
    console.error('❌ Firebase Auth initialization failed:', error);
    if (process.env.NODE_ENV === 'development') {
      console.warn('⚠️ Authentication features may be limited');
    }
    // Reset variables to null on failure
    app = null;
    auth = null;
    googleProvider = null;
  }
} else {
  if (process.env.NODE_ENV === 'development') {
    console.warn('⚠️ Firebase Auth not initialized - invalid or missing configuration');
    console.warn('📋 Required environment variables:');
    console.warn('  - NEXT_PUBLIC_FIREBASE_API_KEY (35+ chars)');
    console.warn('  - NEXT_PUBLIC_FIREBASE_PROJECT_ID (5+ chars)');
    console.warn('  - NEXT_PUBLIC_FIREBASE_APP_ID (15+ chars)');
  }
}

export { auth, googleProvider };

// Configure Google provider if initialized
if (googleProvider) {
  try {
    googleProvider.addScope('profile');
    googleProvider.addScope('email');
    if (process.env.NODE_ENV === 'development') {
      console.log('✅ Google provider configured successfully for admin');
    }
  } catch (error) {
    console.error('❌ Google provider configuration failed:', error);
  }
} else {
  if (process.env.NODE_ENV === 'development') {
    console.warn('⚠️ Google provider not configured - Firebase not initialized');
  }
}

export { auth, googleProvider };

/**
 * Sign in with Google using Firebase
 * Returns Firebase ID token for OIDC exchange
 */
export async function signInWithGoogle(): Promise<{
  user: FirebaseUser
  idToken: string
}> {
  try {
    if (!auth || !googleProvider) {
      throw new Error('Firebase authentication is not initialized');
    }
    
    console.log('🔄 Starting Firebase Google authentication...')
    
    const result = await signInWithPopup(auth, googleProvider)
    const user = result.user
    
    // Get Firebase ID token for OIDC exchange
    const idToken = await getIdToken(user)
    
    console.log('✅ Firebase Google authentication successful:', {
      uid: user.uid,
      email: user.email,
      displayName: user.displayName
    })
    
    return { user, idToken }
  } catch (error) {
    console.error('❌ Firebase Google authentication failed:', error)
    throw new Error(`Google sign-in failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Sign in with email and password using Firebase
 * Returns Firebase ID token for OIDC exchange
 */
export async function signInWithEmail(email: string, password: string): Promise<{
  user: FirebaseUser
  idToken: string
}> {
  try {
    if (!auth) {
      throw new Error('Firebase authentication is not initialized');
    }
    
    console.log('🔄 Starting Firebase email authentication...')
    
    const result = await signInWithEmailAndPassword(auth, email, password)
    const user = result.user
    
    // Get Firebase ID token for OIDC exchange
    const idToken = await getIdToken(user)
    
    console.log('✅ Firebase email authentication successful:', {
      uid: user.uid,
      email: user.email
    })
    
    return { user, idToken }
  } catch (error) {
    console.error('❌ Firebase email authentication failed:', error)
    throw new Error(`Email sign-in failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Create account with email and password using Firebase
 * Returns Firebase ID token for OIDC exchange
 */
export async function createAccount(email: string, password: string): Promise<{
  user: FirebaseUser
  idToken: string
}> {
  try {
    if (!auth) {
      throw new Error('Firebase authentication is not initialized');
    }
    
    console.log('🔄 Creating Firebase account with email...')
    
    const result = await createUserWithEmailAndPassword(auth, email, password)
    const user = result.user
    
    // Get Firebase ID token for OIDC exchange
    const idToken = await getIdToken(user)
    
    console.log('✅ Firebase account created successfully:', {
      uid: user.uid,
      email: user.email
    })
    
    return { user, idToken }
  } catch (error) {
    console.error('❌ Firebase account creation failed:', error)
    throw new Error(`Account creation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Complete Firebase → OIDC → Cookies authentication flow for admin
 */
export async function completeFirebaseOIDCFlow(firebaseIdToken: string): Promise<void> {
  try {
    console.log('🚀 Starting admin Firebase → OIDC → Cookies flow...')
    
    // This function doesn't need to do anything - the login component handles the flow
    console.log('✅ Admin Firebase OIDC flow placeholder')
    
  } catch (error) {
    console.error('❌ Admin Firebase OIDC flow failed:', error)
    throw error
  }
}

/**
 * Sign out from Firebase
 */
export async function signOut(): Promise<void> {
  try {
    if (!auth) {
      throw new Error('Firebase authentication is not initialized');
    }
    
    console.log('🔄 Signing out from Firebase...')
    
    await firebaseSignOut(auth)
    
    console.log('✅ Firebase sign out successful')
  } catch (error) {
    console.error('❌ Firebase sign out failed:', error)
    throw new Error(`Sign out failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}