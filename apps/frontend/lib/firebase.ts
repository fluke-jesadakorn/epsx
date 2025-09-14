'use client'

/**
 * Firebase Authentication Integration
 * Phase 2 Day 8: Firebase Auth → OIDC tokens → HttpOnly cookies flow
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
import { 
  getRemoteConfig, 
  fetchAndActivate, 
  getValue, 
  getAll,
  RemoteConfig 
} from 'firebase/remote-config'

import { config } from '@/config/env';

// Firebase configuration validation - prevent initialization with undefined values
const validateFirebaseConfig = () => {
  const requiredFields = [
    { key: 'apiKey', value: config.firebase.apiKey, minLength: 35 },
    { key: 'projectId', value: config.firebase.projectId, minLength: 5 },
    { key: 'appId', value: config.firebase.appId, minLength: 15 }
  ];

  const missing = requiredFields.filter(field => 
    !field.value || 
    field.value === 'undefined' || 
    field.value.length < field.minLength
  );

  if (missing.length > 0) {
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
  console.warn('🔧 Firebase initialization skipped due to invalid configuration');
  console.warn('💡 To enable Firebase features, ensure NEXT_PUBLIC_FIREBASE_* environment variables are properly set');
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
    
    console.log('✅ Firebase Auth initialized successfully');
  } catch (error) {
    console.error('❌ Firebase Auth initialization failed:', error);
    console.warn('⚠️ Authentication features may be limited');
    // Reset variables to null on failure
    app = null;
    auth = null;
    googleProvider = null;
  }
} else {
  console.warn('⚠️ Firebase Auth not initialized - invalid or missing configuration');
  console.warn('📋 Required environment variables:');
  console.warn('  - NEXT_PUBLIC_FIREBASE_API_KEY (35+ chars)');
  console.warn('  - NEXT_PUBLIC_FIREBASE_PROJECT_ID (5+ chars)');
  console.warn('  - NEXT_PUBLIC_FIREBASE_APP_ID (15+ chars)');
}

export { auth, googleProvider };

// Initialize Firebase Remote Config with error handling
let remoteConfig: any = null;

if (app && isFirebaseConfigValid) {
  try {
    remoteConfig = getRemoteConfig(app);
    
    // Configure Remote Config settings
    remoteConfig.settings.minimumFetchIntervalMillis = 3600000 // 1 hour for production
    remoteConfig.settings.fetchTimeoutMillis = 60000 // 60 seconds
    
    console.log('✅ Firebase Remote Config initialized successfully');
  } catch (error) {
    console.error('❌ Firebase Remote Config initialization failed:', error);
    remoteConfig = null;
  }
} else {
  console.warn('⚠️ Remote Config not initialized - Firebase app not available or invalid configuration');
}

export { remoteConfig };

// Configure Google provider if initialized
if (googleProvider) {
  try {
    googleProvider.addScope('profile');
    googleProvider.addScope('email');
    console.log('✅ Google provider configured successfully');
  } catch (error) {
    console.error('❌ Google provider configuration failed:', error);
  }
} else {
  console.warn('⚠️ Google provider not configured - Firebase not initialized');
}

// ============================================================================
// Firebase Authentication Methods
// ============================================================================

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
 * Sign out from Firebase
 */
export async function signOut(): Promise<void> {
  try {
    console.log('🔄 Signing out from Firebase...')
    
    await firebaseSignOut(auth)
    
    console.log('✅ Firebase sign out successful')
  } catch (error) {
    console.error('❌ Firebase sign out failed:', error)
    throw new Error(`Sign out failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Get current Firebase user ID token
 * Used for refreshing OIDC tokens
 */
export async function getCurrentUserIdToken(): Promise<string | null> {
  try {
    const user = auth.currentUser
    if (!user) {
      console.log('ℹ️ No Firebase user currently signed in')
      return null
    }
    
    const idToken = await getIdToken(user, true) // Force refresh
    console.log('✅ Firebase ID token refreshed successfully')
    
    return idToken
  } catch (error) {
    console.error('❌ Failed to get Firebase ID token:', error)
    return null
  }
}

// ============================================================================
// Firebase Auth State Management
// ============================================================================

/**
 * Firebase auth state change listener
 * Integrates with OIDC token exchange flow
 */
export function onFirebaseAuthStateChanged(
  callback: (user: FirebaseUser | null) => Promise<void>
): () => void {
  return onAuthStateChanged(auth, async (user) => {
    console.log('🔄 Firebase auth state changed:', user ? 'signed in' : 'signed out')
    
    try {
      await callback(user)
    } catch (error) {
      console.error('❌ Auth state change callback error:', error)
    }
  })
}

/**
 * Check if Firebase is initialized and user is authenticated
 */
export function isFirebaseAuthenticated(): boolean {
  return !!auth.currentUser
}

/**
 * Get Firebase user display information
 */
export function getFirebaseUserInfo(): {
  uid: string
  email: string | null
  displayName: string | null
  photoURL: string | null
} | null {
  const user = auth.currentUser
  if (!user) return null
  
  return {
    uid: user.uid,
    email: user.email,
    displayName: user.displayName,
    photoURL: user.photoURL
  }
}

// ============================================================================
// OIDC Token Exchange Integration
// ============================================================================

/**
 * Exchange Firebase ID token for OIDC tokens
 * Core function for Phase 2 Day 8 implementation
 */
export async function exchangeFirebaseTokenForOIDC(
  firebaseIdToken: string
): Promise<{
  accessToken: string
  idToken: string
  refreshToken: string
}> {
  try {
    console.log('🔄 Exchanging Firebase ID token for OIDC tokens...')
    
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
    
    // Call backend OIDC token exchange endpoint
    const response = await fetch(`${backendUrl}/api/v1/oidc/token/exchange`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        firebase_id_token: firebaseIdToken,
        grant_type: 'firebase_token',
        scope: 'openid profile email'
      })
    })
    
    if (!response.ok) {
      const errorText = await response.text()
      console.error('❌ OIDC token exchange failed:', response.status, errorText)
      throw new Error(`OIDC token exchange failed: ${response.status} ${response.statusText}`)
    }
    
    const tokens = await response.json()
    
    console.log('✅ OIDC token exchange successful:', {
      accessToken: tokens.access_token ? 'received' : 'missing',
      idToken: tokens.id_token ? 'received' : 'missing',
      refreshToken: tokens.refresh_token ? 'received' : 'missing'
    })
    
    if (!tokens.access_token || !tokens.id_token || !tokens.refresh_token) {
      throw new Error('Incomplete OIDC token response')
    }
    
    return {
      accessToken: tokens.access_token,
      idToken: tokens.id_token,
      refreshToken: tokens.refresh_token
    }
    
  } catch (error) {
    console.error('❌ OIDC token exchange error:', error)
    throw new Error(`Failed to exchange Firebase token for OIDC tokens: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Store OIDC tokens in HttpOnly cookies
 * Final step of Firebase → OIDC → Cookies flow
 */
export async function storeOIDCTokensInCookies(tokens: {
  accessToken: string
  idToken: string
  refreshToken: string
}): Promise<void> {
  try {
    console.log('🔄 Storing OIDC tokens in HttpOnly cookies...')
    
    // Call frontend API to set OIDC cookies
    const response = await fetch('/api/auth/session', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        accessToken: tokens.accessToken,
        idToken: tokens.idToken,
        refreshToken: tokens.refreshToken
      }),
      credentials: 'include'
    })
    
    if (!response.ok) {
      const errorData = await response.json()
      throw new Error(`Failed to store OIDC cookies: ${errorData.error}`)
    }
    
    console.log('✅ OIDC tokens stored in HttpOnly cookies successfully')
    
  } catch (error) {
    console.error('❌ Failed to store OIDC tokens in cookies:', error)
    throw error
  }
}

/**
 * Complete Firebase → OIDC → Cookies authentication flow
 * Main integration function for Phase 2 Day 8
 */
export async function completeFirebaseOIDCFlow(firebaseIdToken: string): Promise<void> {
  try {
    console.log('🚀 Starting complete Firebase → OIDC → Cookies flow...')
    
    // Step 1: Exchange Firebase ID token for OIDC tokens
    const oidcTokens = await exchangeFirebaseTokenForOIDC(firebaseIdToken)
    
    // Step 2: Store OIDC tokens in HttpOnly cookies
    await storeOIDCTokensInCookies(oidcTokens)
    
    console.log('✅ Complete Firebase → OIDC → Cookies flow successful!')
    
  } catch (error) {
    console.error('❌ Firebase → OIDC → Cookies flow failed:', error)
    throw error
  }
}