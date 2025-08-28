'use server';

import { cookies } from 'next/headers';
import { verifyJWT, type JWTUser } from './auth-utils';

export interface User extends JWTUser {
  id: string;
  name?: string;
  image?: string;
  tier?: string;
  level?: string;
}

export interface FeatureAccess {
  hasAccess: boolean;
  tier: string;
  feature: string;
  reason?: string;
}

/**
 * Get current authenticated user from session
 */
export async function getCurrentUser(params?: any): Promise<{ success: boolean; data?: User; error?: string } | User | null> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get('auth-token')?.value;
    
    if (!token) {
      // Return null for backward compatibility, or success format if params provided
      return params ? { success: false, error: 'No authentication token found' } : null;
    }

    const secret = process.env.NEXTAUTH_SECRET || 'default-secret';
    const payload = await verifyJWT(token, secret);
    
    if (!payload) {
      return params ? { success: false, error: 'Invalid authentication token' } : null;
    }

    const user: User = {
      id: payload.uid,
      uid: payload.uid,
      email: payload.email,
      firebaseUid: payload.firebaseUid,
      role: payload.role,
      name: payload.email?.split('@')[0], // Fallback name from email
    };

    // Return success format if params provided, otherwise just the user
    return params ? { success: true, data: user } : user;
  } catch (error) {
    console.error('Failed to get current user:', error);
    return params ? { success: false, error: error instanceof Error ? error.message : 'Unknown error' } : null;
  }
}

/**
 * Check if user has access to a specific feature
 */
export async function checkFeatureAccess(feature: string): Promise<FeatureAccess> {
  const user = await getCurrentUser();
  
  if (!user) {
    return {
      hasAccess: false,
      tier: 'guest',
      feature,
      reason: 'User not authenticated'
    };
  }

  // Basic feature access logic - can be expanded
  const userTier = user.role || 'basic';
  const basicFeatures = ['dashboard', 'basic-analytics'];
  const premiumFeatures = ['advanced-analytics', 'alerts', 'export'];
  const adminFeatures = ['admin-panel', 'user-management'];

  let hasAccess = false;

  if (adminFeatures.includes(feature)) {
    hasAccess = userTier === 'admin' || userTier === 'moderator';
  } else if (premiumFeatures.includes(feature)) {
    hasAccess = userTier === 'premium' || userTier === 'admin' || userTier === 'moderator';
  } else if (basicFeatures.includes(feature)) {
    hasAccess = true; // All authenticated users
  }

  return {
    hasAccess,
    tier: userTier,
    feature,
    reason: hasAccess ? undefined : `Feature requires ${feature.includes('admin') ? 'admin' : 'premium'} access`
  };
}

/**
 * Get user session info
 */
export async function getSession(): Promise<{ user: User | null }> {
  const user = await getCurrentUser();
  return { user };
}

/**
 * Sign out user
 */
export async function signOut(): Promise<void> {
  const cookieStore = await cookies();
  cookieStore.delete('auth-token');
  cookieStore.delete('refresh-token');
}

/**
 * Get transaction history for current user
 */
export async function getTransactionHistory(excludePending?: boolean): Promise<PaymentTransaction[]> {
  try {
    // This is a stub implementation - replace with actual API call
    // For now, return empty array to prevent build errors
    return [];
  } catch (error) {
    console.error('Failed to get transaction history:', error);
    return [];
  }
}

export type { PaymentStatus, PaymentTransaction } from './api-client';

// Analytics Server Actions
interface AnalyticsFilterParams {
  page?: number;
  limit?: number;
  country?: string;
  sector?: string;
  sort_by?: string;
  min_eps?: number;
  min_growth?: number;
}

export async function updateAnalyticsFilters(params: AnalyticsFilterParams) {
  const { redirect } = await import('next/navigation');
  const searchParams = new URLSearchParams();
  
  // Always include page and limit
  searchParams.set('page', String(params.page || 1));
  searchParams.set('limit', String(params.limit || 10));
  
  // Add other params if they exist
  if (params.country) searchParams.set('country', params.country);
  if (params.sector) searchParams.set('sector', params.sector);
  if (params.sort_by) searchParams.set('sort_by', params.sort_by);
  if (params.min_eps !== undefined) searchParams.set('min_eps', String(params.min_eps));
  if (params.min_growth !== undefined) searchParams.set('min_growth', String(params.min_growth));

  redirect(`/analytics?${searchParams.toString()}`);
}

export async function navigateToPage(page: number, currentParams: string) {
  const { redirect } = await import('next/navigation');
  const searchParams = new URLSearchParams(currentParams);
  searchParams.set('page', String(page));
  
  redirect(`/analytics?${searchParams.toString()}`);
}