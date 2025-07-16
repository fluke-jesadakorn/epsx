'use server';

import { redirect } from 'next/navigation';
import { 
  createSession, 
  destroySession, 
  verifySession, 
  refreshSession as refreshSessionInternal,
  getSessionInfo
} from './session';
import type { SessionClaims, SessionConfig } from './types';

/**
 * Handle user sign in - create session from Firebase ID token
 */
export async function handleSignIn(
  idToken: string,
  config?: Partial<SessionConfig>
): Promise<{ success: boolean; error?: string }> {
  try {
    console.log('Creating session with token');
    const result = await createSession(idToken, config);
    if (!result.success) {
      throw new Error(result.error || 'Failed to create session');
    }
    console.log('Session created successfully');
    return { success: true };
  } catch (error) {
    console.error('Sign-in error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Authentication failed' 
    };
  }
}

/**
 * Handle user sign out - destroy session
 */
export async function handleSignOut(
  config?: Partial<SessionConfig>
): Promise<{ success: boolean; error?: string }> {
  try {
    console.log('Destroying session');
    await destroySession(config);
    console.log('Session destroyed successfully');
    return { success: true };
  } catch (error) {
    console.error('Sign-out error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to sign out' 
    };
  }
}

/**
 * Refresh the current session with a new token
 */
export async function refreshSession(
  newToken: string,
  config?: Partial<SessionConfig>
): Promise<{ success: boolean; error?: string }> {
  try {
    console.log('Refreshing session with new token');
    const result = await refreshSessionInternal(newToken, config);
    if (!result.success) {
      throw new Error(result.error || 'Failed to refresh session');
    }
    console.log('Session refreshed successfully');
    return { success: true };
  } catch (error) {
    console.error('Session refresh error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to refresh session' 
    };
  }
}

/**
 * Get current authenticated user with session claims
 */
export async function getCurrentUser(
  config?: Partial<SessionConfig>
): Promise<SessionClaims | null> {
  try {
    const result = await verifySession(config);
    return result.success ? result.claims || null : null;
  } catch (error) {
    console.error('Failed to get current user:', error);
    return null;
  }
}

/**
 * Check authentication status (lightweight)
 */
export async function getAuthStatus(
  config?: Partial<SessionConfig>
): Promise<{
  isAuthenticated: boolean;
  needsRefresh?: boolean;
  user?: {
    id: string;
    email?: string;
    emailVerified?: boolean;
    displayName?: string;
  };
}> {
  try {
    const result = await verifySession(config);
    
    if (!result.success || !result.claims) {
      return { isAuthenticated: false };
    }

    return {
      isAuthenticated: true,
      needsRefresh: result.needsRefresh,
      user: {
        id: result.claims.uid,
        email: result.claims.email,
        emailVerified: result.claims.email_verified,
        displayName: result.claims.name,
      },
    };
  } catch (error) {
    console.error('Failed to get auth status:', error);
    return { isAuthenticated: false };
  }
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(
  redirectTo?: string,
  config?: Partial<SessionConfig>
): Promise<SessionClaims> {
  const user = await getCurrentUser(config);
  
  if (!user) {
    redirect(redirectTo || '/login');
  }
  
  return user;
}

/**
 * Require guest (unauthenticated) - redirect to dashboard if authenticated
 */
export async function requireGuest(
  redirectTo: string = '/dashboard',
  config?: Partial<SessionConfig>
): Promise<void> {
  const user = await getCurrentUser(config);
  
  if (user) {
    redirect(redirectTo);
  }
}

/**
 * Get session info (lightweight version)
 */
export async function getSessionInfoAction(
  config?: Partial<SessionConfig>
): Promise<{
  isAuthenticated: boolean;
  email?: string;
  emailVerified?: boolean;
  displayName?: string;
}> {
  return await getSessionInfo(config);
}
