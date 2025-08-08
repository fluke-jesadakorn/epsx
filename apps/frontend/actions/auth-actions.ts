'use server';

import { getCurrentUser, getBearerToken, checkFeatureAccess } from '@/lib/auth/server-auth';

/**
 * Server actions for authentication and authorization
 * Replaces client-side hooks with server-only validation
 */

/**
 * Get user session data (server action)
 */
export async function getUserSession() {
  const user = await getCurrentUser();
  
  if (!user) {
    return { authenticated: false, user: null };
  }
  
  return {
    authenticated: true,
    user: {
      id: user.user_id,
      email: user.email,
      role: user.role,
      subscription_tier: user.subscription_tier,
    },
  };
}

/**
 * Check permission for specific action/resource
 */
export async function checkPermission(
  action: string,
  resource: string
): Promise<{
  allowed: boolean;
  reason?: string;
  limits?: any;
}> {
  const user = await getCurrentUser();
  const token = await getBearerToken();
  
  if (!user || !token) {
    return { allowed: false, reason: 'Not authenticated' };
  }
  
  try {
    const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:8080'}/api/v1/permissions/check`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: user.user_id,
        action,
        resource,
      }),
    });
    
    if (!response.ok) {
      return { allowed: false, reason: 'Permission check failed' };
    }
    
    return await response.json();
  } catch (error) {
    console.error('Permission check error:', error);
    return { allowed: false, reason: 'Check error' };
  }
}

/**
 * Validate feature access with tier limitations
 */
export async function validateFeature(feature: string): Promise<{
  allowed: boolean;
  limits?: {
    maxResults?: number;
    features?: string[];
    refreshRate?: number;
  };
  reason?: string;
}> {
  return await checkFeatureAccess(feature, 'access');
}

/**
 * Get user's subscription limits
 */
export async function getSubscriptionLimits(): Promise<{
  tier: string;
  limits: {
    api_calls_per_day?: number;
    max_watchlist_items?: number;
    max_alerts?: number;
    analytics_history_days?: number;
    real_time_data?: boolean;
    export_data?: boolean;
  };
} | null> {
  const user = await getCurrentUser();
  const token = await getBearerToken();
  
  if (!user || !token) {
    return null;
  }
  
  try {
    const response = await fetch(`${process.env.BACKEND_URL || 'http://localhost:8080'}/api/v1/users/${user.user_id}/limits`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });
    
    if (!response.ok) {
      return null;
    }
    
    return await response.json();
  } catch (error) {
    console.error('Failed to get subscription limits:', error);
    return null;
  }
}