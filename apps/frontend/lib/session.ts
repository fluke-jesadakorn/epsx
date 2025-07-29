import { cookies } from 'next/headers';
import { ServerCookies, COOKIE_NAMES } from './cookies';

const SESSION_KEY = COOKIE_NAMES.SESSION;
const MAX_AGE = 60 * 60 * 24 * 5; // 5 days in seconds
const REFRESH_THRESHOLD = 60 * 60; // 1 hour before expiry

export interface SessionClaims {
  user_id: string;
  email: string;
  email_verified: boolean;
  name?: string;
  picture: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  package_tier: string;
  expires_at: string;
  session_type: string;
}

export interface SessionResult {
  success: boolean;
  claims?: SessionClaims;
  needsRefresh?: boolean;
}

/**
 * Create a new session using backend verification
 */
export async function createSession(sessionData: any): Promise<{ success: boolean; error?: string }> {
  try {
    if (!sessionData?.user_id) {
      return { success: false, error: 'Invalid session data: missing user ID' };
    }

    // Store session data as a JSON string in cookies
    await ServerCookies.set('SESSION', JSON.stringify(sessionData));

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
    const sessionData = await ServerCookies.get('SESSION');
    
    if (!sessionData) {
      return { success: false };
    }

    // Parse session data from JSON
    let parsedSession;
    try {
      parsedSession = JSON.parse(sessionData);
    } catch (parseError) {
      console.error('Failed to parse session data:', parseError);
      await destroySession(); // Clear invalid session
      return { success: false };
    }
    
    const claims: SessionClaims = {
      user_id: parsedSession.user_id,
      email: parsedSession.email || '',
      email_verified: true, // Backend sessions are considered verified
      name: parsedSession.name,
      picture: parsedSession.picture || '',
      role: parsedSession.role || 'user',
      permissions: parsedSession.permissions || [],
      subscription_tier: parsedSession.subscription_tier || 'free',
      package_tier: parsedSession.package_tier || 'free',
      expires_at: parsedSession.expires_at,
      session_type: parsedSession.session_type || 'user',
    };

    // Check if session needs refresh (within 1 hour of expiry)
    const needsRefresh = claims.expires_at ? 
      (new Date(claims.expires_at).getTime() - Date.now()) < (REFRESH_THRESHOLD * 1000) : false;

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
 * Refresh the current session with new session data
 */
export async function refreshSession(newSessionData: any): Promise<{ success: boolean; error?: string }> {
  try {
    if (!newSessionData?.user_id) {
      return { success: false, error: 'Invalid refresh data: missing user ID' };
    }

    // Update the session cookie using ServerCookies
    await ServerCookies.set('SESSION', JSON.stringify(newSessionData));

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
    await ServerCookies.clearAuthCookies();
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
    return await ServerCookies.has('SESSION');
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
  email: string | undefined;
  emailVerified: boolean | undefined;
  displayName: string | undefined;
  role: string | undefined;
  permissions: string[];
  packageTier: string | undefined;
}> {
  const result = await verifySession();
  
  if (!result.success || !result.claims) {
    return { 
      isAuthenticated: false,
      email: undefined,
      emailVerified: undefined,
      displayName: undefined,
      role: undefined,
      permissions: [],
      packageTier: undefined,
    };
  }

  return {
    isAuthenticated: true,
    email: result.claims.email ?? undefined,
    emailVerified: result.claims.email_verified ?? undefined,
    displayName: result.claims.name ?? undefined,
    role: result.claims.role ?? undefined,
    permissions: result.claims.permissions ?? [],
    packageTier: result.claims.package_tier ?? undefined,
  };
}
