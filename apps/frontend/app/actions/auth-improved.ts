'use server';

import { redirect } from 'next/navigation';
import { createSession, destroySession, verifySession, refreshSession as refreshSessionInternal } from '@/lib/session-improved';
import { getPaymentDetails } from './payment';
import type { User } from '@/types/auth/user';
export async function handleSignIn(idToken: string): Promise<{ success: boolean; error?: string }> {
  try {
    const result = await createSession(idToken);
    
    if (!result.success) {
      throw new Error(result.error || 'Failed to create session');
    }
    
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
export async function handleSignOut(): Promise<{ success: boolean; error?: string }> {
  try {
    await destroySession();
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
export async function refreshSession(newToken: string): Promise<{ success: boolean; error?: string }> {
  try {
    const result = await refreshSessionInternal(newToken);
    return result;
  } catch (error) {
    console.error('Session refresh error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to refresh session' 
    };
  }
}

/**
 * Get current authenticated user with full profile data
 */
export async function getCurrentUser(): Promise<User | null> {
  try {
    const sessionResult = await verifySession();
    
    if (!sessionResult.success || !sessionResult.claims) {
      return null;
    }

    const { claims } = sessionResult;
    
    // Build user object from session claims
    const user: User = {
      id: claims.uid,
      email: claims.email || '',
      createdAt: new Date().toISOString(), // We don't have this in claims
      updatedAt: new Date().toISOString(),
      emailVerified: claims.email_verified || false,
      role: 'USER', // Default role, could be enhanced with custom claims
      displayName: claims.name || undefined,
      photoURL: claims.picture || undefined,
    };

    // Fetch additional user data (payment details, etc.)
    try {
      const usdtDetails = await getPaymentDetails(user.id);
      user.usdtDetails = usdtDetails;
    } catch (error) {
      console.warn('Failed to fetch payment details:', error);
      // Don't fail the whole request if payment details fail
    }

    return user;
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
}

/**
 * Check authentication status (lightweight)
 */
export async function getAuthStatus(): Promise<{
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
    const sessionResult = await verifySession();
    
    if (!sessionResult.success || !sessionResult.claims) {
      return { isAuthenticated: false };
    }

    return {
      isAuthenticated: true,
      needsRefresh: sessionResult.needsRefresh,
      user: {
        id: sessionResult.claims.uid,
        email: sessionResult.claims.email,
        emailVerified: sessionResult.claims.email_verified,
        displayName: sessionResult.claims.name,
      },
    };
  } catch (error) {
    console.error('Auth status check error:', error);
    return { isAuthenticated: false };
  }
}

/**
 * Require authentication - redirect to login if not authenticated
 */
export async function requireAuth(redirectTo?: string): Promise<User> {
  const user = await getCurrentUser();
  
  if (!user) {
    const loginUrl = `/login${redirectTo ? `?returnUrl=${encodeURIComponent(redirectTo)}` : ''}`;
    redirect(loginUrl);
  }
  
  return user;
}

/**
 * Require guest (unauthenticated) - redirect to dashboard if authenticated
 */
export async function requireGuest(): Promise<void> {
  const user = await getCurrentUser();
  
  if (user) {
    redirect('/my-data');
  }
}
