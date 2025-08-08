'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { auth } from '@/auth';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
  session_type: string;
  expires_at: string;
}

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

/**
 * Get current user from session (server-side only)
 */
export async function getCurrentUser(): Promise<AuthUser | null> {
  try {
    const session = await auth();
    
    if (session?.user?.id && session?.user?.email) {
      return {
        user_id: session.user.id,
        email: session.user.email,
        role: session.user.role || 'user',
        permissions: session.user.permissions || [],
        subscription_tier: session.user.subscription_tier || 'free',
        session_type: 'user',
        expires_at: session.user.expires_at || new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
      };
    }
  } catch (error) {
    console.error('Failed to get current user:', error);
  }
  
  return null;
}

/**
 * Get bearer token for API calls (server-side only)
 */
export async function getBearerToken(): Promise<string | null> {
  try {
    const session = await auth();
    
    if (session?.session_id) {
      return session.session_id;
    }
  } catch (error) {
    console.error('Failed to get bearer token:', error);
  }
  
  return null;
}

/**
 * Check if user is authenticated (server-side only)
 */
export async function isAuthenticated(): Promise<boolean> {
  const user = await getCurrentUser();
  return user !== null;
}

/**
 * Validate user permissions against backend
 */
export async function validatePermissions(
  permissions: string[]
): Promise<{ allowed: boolean; reason?: string }> {
  const user = await getCurrentUser();
  const token = await getBearerToken();
  
  if (!user || !token) {
    return { allowed: false, reason: 'Not authenticated' };
  }
  
  try {
    const response = await fetch(`${BACKEND_URL}/api/v1/permissions/validate`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: user.user_id,
        permissions,
      }),
    });
    
    if (!response.ok) {
      return { allowed: false, reason: 'Permission validation failed' };
    }
    
    const result = await response.json();
    return { allowed: result.allowed, reason: result.reason };
  } catch (error) {
    console.error('Permission validation error:', error);
    return { allowed: false, reason: 'Validation error' };
  }
}

/**
 * Check specific feature access
 */
export async function checkFeatureAccess(
  feature: string,
  action: string = 'read'
): Promise<{ allowed: boolean; limits?: any; reason?: string }> {
  const user = await getCurrentUser();
  const token = await getBearerToken();
  
  if (!user || !token) {
    return { allowed: false, reason: 'Not authenticated' };
  }
  
  try {
    const response = await fetch(`${BACKEND_URL}/api/v1/permissions/check`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: user.user_id,
        feature,
        action,
      }),
    });
    
    if (!response.ok) {
      return { allowed: false, reason: 'Feature check failed' };
    }
    
    const result = await response.json();
    return {
      allowed: result.allowed,
      limits: result.limits,
      reason: result.reason,
    };
  } catch (error) {
    console.error('Feature access check error:', error);
    return { allowed: false, reason: 'Check error' };
  }
}

/**
 * Require authentication (redirect if not authenticated)
 */
export async function requireAuth(): Promise<AuthUser> {
  const user = await getCurrentUser();
  
  if (!user) {
    redirect('/login');
  }
  
  return user;
}

/**
 * Require specific permissions (redirect if not authorized)
 */
export async function requirePermissions(permissions: string[]): Promise<AuthUser> {
  const user = await requireAuth();
  const validation = await validatePermissions(permissions);
  
  if (!validation.allowed) {
    redirect('/access-denied?reason=' + encodeURIComponent(validation.reason || 'Insufficient permissions'));
  }
  
  return user;
}