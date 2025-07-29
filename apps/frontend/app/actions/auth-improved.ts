'use server';

import { redirect } from 'next/navigation';
import { getServerAuth, createServerSession, destroyServerSession } from '@/lib/auth-server';
import { ServerCookies } from '@/lib/cookies';
import { createApiClient, isApiError } from '@epsx/api-client';
import type { User } from '@/types/auth/user';

// Get API client - will automatically use backend URL
const getApi = () => {
  return createApiClient();
};

/**
 * Enhanced Server Actions for authentication with backend integration
 * These functions use the Rust backend API directly
 */

/**
 * Handle user sign in with enhanced error handling
 */
export async function handleSignIn(
  email: string, 
  password: string
): Promise<{ success: boolean; error?: string; user?: any }> {
  try {
    const result = await createServerSession(email, password);
    
    if (!result.success) {
      return { 
        success: false, 
        error: result.error || 'Authentication failed' 
      };
    }
    
    return { 
      success: true, 
      user: result.user 
    };
  } catch (error) {
    console.error('Enhanced sign-in error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Authentication failed' 
    };
  }
}

/**
 * Handle user sign out with enhanced cleanup
 */
export async function handleSignOut(): Promise<{ success: boolean; error?: string }> {
  try {
    await destroyServerSession();
    return { success: true };
  } catch (error) {
    console.error('Enhanced sign-out error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Failed to sign out' 
    };
  }
}

/**
 * Refresh session token with backend validation
 */
export async function refreshSession(): Promise<{ 
  success: boolean; 
  error?: string; 
  needsRefresh?: boolean 
}> {
  try {
    const sessionId = await ServerCookies.get('SESSION');
    
    if (!sessionId) {
      return { success: false, needsRefresh: true, error: 'No session found' };
    }
    
    const api = getApi();
    const response = await api.refreshToken();
    
    if (isApiError(response)) {
      await ServerCookies.delete('SESSION');
      return { 
        success: false, 
        needsRefresh: true, 
        error: response.error || 'Session expired' 
      };
    }
    
    return { success: true, needsRefresh: false };
  } catch (error) {
    console.error('Session refresh error:', error);
    return { 
      success: false, 
      needsRefresh: true,
      error: error instanceof Error ? error.message : 'Failed to refresh session' 
    };
  }
}

/**
 * Get current authenticated user with full profile data
 */
export async function getCurrentUser(): Promise<User | null> {
  try {
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return null;
    }

    const { user } = authResult;
    
    const fullUser: User = {
      id: user.user_id,
      email: user.email,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      emailVerified: user.emailVerified || true,
      role: user.role?.toUpperCase() as any || 'USER',
      displayName: user.displayName || user.email?.split('@')[0],
      photoURL: user.photoURL,
    };

    // Fetch additional user data (payment details, etc.) if needed
    try {
      // Enhanced user data could be fetched from backend here
      // const additionalData = await getPaymentDetails(user.user_id);
      // if (additionalData) fullUser.usdtDetails = additionalData;
    } catch (error) {
      console.warn('Failed to fetch additional user data:', error);
    }

    return fullUser;
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
}

/**
 * Check authentication status with enhanced details
 */
export async function getAuthStatus(): Promise<{
  isAuthenticated: boolean;
  needsRefresh?: boolean;
  user?: {
    id: string;
    email?: string;
    emailVerified?: boolean;
    displayName?: string;
    role?: string;
    permissions?: string[];
    packageTier?: string;
  };
  sessionInfo?: {
    expiresAt?: string;
    sessionType?: string;
  };
}> {
  try {
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return { isAuthenticated: false };
    }

    const { user } = authResult;
    
    // Check if session needs refresh
    const refreshResult = await refreshSession();
    
    return {
      isAuthenticated: true,
      needsRefresh: refreshResult.needsRefresh,
      user: {
        id: user.user_id,
        email: user.email,
        emailVerified: user.emailVerified,
        displayName: user.displayName,
        role: user.role,
        permissions: user.permissions,
        packageTier: user.package_tier,
      },
      sessionInfo: {
        expiresAt: user.expires_at,
        sessionType: user.session_type,
      },
    };
  } catch (error) {
    console.error('Auth status check error:', error);
    return { isAuthenticated: false };
  }
}

/**
 * Require authentication with enhanced redirect handling
 */
export async function requireAuth(redirectTo?: string): Promise<User> {
  const user = await getCurrentUser();
  
  if (!user) {
    const loginUrl = `/login${redirectTo ? `?redirect=${encodeURIComponent(redirectTo)}` : ''}`;
    redirect(loginUrl);
  }
  
  return user;
}

/**
 * Require specific permission
 */
export async function requirePermission(
  permission: string, 
  redirectTo?: string
): Promise<User> {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated) {
    const loginUrl = `/login${redirectTo ? `?redirect=${encodeURIComponent(redirectTo)}` : ''}`;
    redirect(loginUrl);
  }
  
  const hasPermission = authResult.user?.permissions?.includes(permission) ||
    authResult.user?.permissions?.some(userPermission => {
      if (userPermission.endsWith('.*')) {
        const prefix = userPermission.slice(0, -2);
        return permission.startsWith(prefix + '.');
      }
      return false;
    });
  
  if (!hasPermission) {
    redirect('/access-denied?reason=' + encodeURIComponent(`Missing required permission: ${permission}`));
  }
  
  const user = await getCurrentUser();
  return user!;
}

/**
 * Require guest (unauthenticated) with enhanced redirect
 */
export async function requireGuest(): Promise<void> {
  try {
    const authResult = await getServerAuth();
    
    // Only redirect if user is actually authenticated (not just has stale cookies)
    if (authResult.isAuthenticated && authResult.user && !authResult.error) {
      redirect('/dashboard');
    }
  } catch (error) {
    // If there's an error checking auth, assume user is not authenticated (guest)
    console.log('Auth check failed in requireGuest, treating as guest:', error);
  }
}

/**
 * Get user permissions
 */
export async function getUserPermissions(): Promise<string[]> {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated || !authResult.user) {
    return [];
  }
  
  return authResult.user.permissions || [];
}

/**
 * Check if user has specific permission
 */
export async function hasPermission(permission: string): Promise<boolean> {
  const permissions = await getUserPermissions();
  
  // Check exact match
  if (permissions.includes(permission)) {
    return true;
  }
  
  // Check wildcard permissions
  return permissions.some(userPermission => {
    if (userPermission.endsWith('.*')) {
      const prefix = userPermission.slice(0, -2);
      return permission.startsWith(prefix + '.');
    }
    return false;
  });
}
