import { cookies } from 'next/headers';
import { getAuthAdmin } from './firebase-admin';

const SESSION_KEY = '__session';
const MAX_AGE = 60 * 60 * 24 * 5; // 5 days in seconds
const REFRESH_THRESHOLD = 60 * 60; // 1 hour before expiry

export interface SessionClaims {
  uid: string;
  email?: string;
  email_verified?: boolean;
  name?: string;
  picture?: string;
  exp?: number;
  iat?: number;
}

export interface SessionResult {
  success: boolean;
  claims?: SessionClaims;
  needsRefresh?: boolean;
}

/**
 * Create a new session with the provided Firebase ID token
 */
export async function createSession(token: string): Promise<{ success: boolean; error?: string }> {
  try {
    // Verify the token first to ensure it's valid
    const auth = getAuthAdmin();
    const decodedToken = await auth.verifyIdToken(token, true);
    
    if (!decodedToken.uid) {
      return { success: false, error: 'Invalid token: missing user ID' };
    }

    const cookieStore = await cookies();
    cookieStore.set(SESSION_KEY, token, {
      maxAge: MAX_AGE,
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
    });

    return { success: true };
  } catch (error) {
    console.error('Session creation failed:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Session creation failed' 
    };
  }
}

/**
 * Verify the current session and return claims
 */
export async function verifySession(): Promise<SessionResult> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get(SESSION_KEY)?.value;
    
    if (!token) {
      return { success: false };
    }

    // Verify the Firebase ID token
    const auth = getAuthAdmin();
    const decodedToken = await auth.verifyIdToken(token, true); // checkRevoked = true
    
    const claims: SessionClaims = {
      uid: decodedToken.uid,
      email: decodedToken.email,
      email_verified: decodedToken.email_verified,
      name: decodedToken.name,
      picture: decodedToken.picture,
      exp: decodedToken.exp,
      iat: decodedToken.iat,
    };

    // Check if token needs refresh (within 1 hour of expiry)
    const needsRefresh = decodedToken.exp ? 
      (decodedToken.exp - Math.floor(Date.now() / 1000)) < REFRESH_THRESHOLD : false;

    return { 
      success: true, 
      claims,
      needsRefresh 
    };
  } catch (error) {
    console.error('Session verification failed:', error);
    
    // Clear invalid session cookie
    try {
      await destroySession();
    } catch (destroyError) {
      console.error('Failed to clear invalid session:', destroyError);
    }
    
    return { success: false };
  }
}

/**
 * Refresh the current session with a new token
 */
export async function refreshSession(newToken: string): Promise<{ success: boolean; error?: string }> {
  try {
    // Verify the new token
    const auth = getAuthAdmin();
    const decodedToken = await auth.verifyIdToken(newToken, true);
    
    if (!decodedToken.uid) {
      return { success: false, error: 'Invalid refresh token' };
    }

    // Update the session cookie
    const cookieStore = await cookies();
    cookieStore.set(SESSION_KEY, newToken, {
      maxAge: MAX_AGE,
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      path: '/',
    });

    return { success: true };
  } catch (error) {
    console.error('Session refresh failed:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Session refresh failed' 
    };
  }
}

/**
 * Destroy the current session
 */
export async function destroySession(): Promise<void> {
  try {
    const cookieStore = await cookies();
    cookieStore.delete(SESSION_KEY);
  } catch (error) {
    console.error('Session destruction failed:', error);
    throw error;
  }
}

/**
 * Check if a session exists (without verification)
 */
export async function hasSession(): Promise<boolean> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get(SESSION_KEY)?.value;
    return !!token;
  } catch (error) {
    console.error('Session check failed:', error);
    return false;
  }
}

/**
 * Get session info for client-side use (minimal data)
 */
export async function getSessionInfo(): Promise<{
  isAuthenticated: boolean;
  email?: string;
  emailVerified?: boolean;
  displayName?: string;
}> {
  const result = await verifySession();
  
  if (!result.success || !result.claims) {
    return { isAuthenticated: false };
  }

  return {
    isAuthenticated: true,
    email: result.claims.email,
    emailVerified: result.claims.email_verified,
    displayName: result.claims.name,
  };
}
