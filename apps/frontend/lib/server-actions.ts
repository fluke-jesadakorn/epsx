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
export async function getCurrentUser(): Promise<User | null> {
  try {
    const cookieStore = await cookies();
    const token = cookieStore.get('auth-token')?.value;
    
    if (!token) {
      return null;
    }

    const secret = process.env.JWT_SECRET || 'default-secret';
    const payload = await verifyJWT(token, secret);
    
    if (!payload) {
      return null;
    }

    return {
      id: payload.uid,
      uid: payload.uid,
      email: payload.email,
      firebaseUid: payload.firebaseUid,
      role: payload.role,
      name: payload.email?.split('@')[0], // Fallback name from email
    };
  } catch (error) {
    console.error('Failed to get current user:', error);
    return null;
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

export type { PaymentStatus, PaymentTransaction } from './api-client';