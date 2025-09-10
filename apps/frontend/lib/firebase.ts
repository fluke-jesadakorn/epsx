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

// Firebase configuration
const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID
}

// Initialize Firebase (singleton pattern)
const app = getApps().length === 0 ? initializeApp(firebaseConfig) : getApps()[0]
export const auth = getAuth(app)
export const googleProvider = new GoogleAuthProvider()

// Initialize Firebase Remote Config
export const remoteConfig = getRemoteConfig(app)

// Configure Remote Config settings
remoteConfig.settings.minimumFetchIntervalMillis = 3600000 // 1 hour for production
remoteConfig.settings.fetchTimeoutMillis = 60000 // 60 seconds

// Configure Google provider
googleProvider.addScope('profile')
googleProvider.addScope('email')

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