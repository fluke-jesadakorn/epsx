/**
 * @deprecated This client-side auth hook is deprecated.
 * 
 * Use server-side alternatives instead:
 * - import { getCurrentUser } from '@/lib/auth/server-auth' for server components
 * - import { getUserSession } from '@/actions/auth-actions' for server actions
 * - Use PageGuard, FeatureAccess, or TierAccess components for protection
 * 
 * For existing client components that must remain client-side,
 * use useSession from next-auth/react directly.
 */

'use client';

import { useSession } from 'next-auth/react';

/**
 * Minimal client-side session hook for legacy components
 * @deprecated Use server-side auth instead
 */
export function useAuth() {
  const { data: session, status } = useSession();
  
  console.warn('useAuth is deprecated. Use server-side auth components and actions instead.');
  
  return {
    user: session?.user ? {
      user_id: session.user.id,
      email: session.user.email,
      role: session.user.role,
      subscription_tier: session.user.subscription_tier,
      isAuthenticated: true,
    } : null,
    loading: status === 'loading',
    isAuthenticated: !!session?.user,
  };
}

/**
 * @deprecated Use server-side permission checking instead
 */
export function usePermission(feature: string) {
  const { data: session, status } = useSession();
  
  console.warn('usePermission is deprecated. Use FeatureAccess component or checkPermission server action instead.');
  
  if (status === 'loading') {
    return { hasPermission: false, loading: true };
  }
  
  const hasPermission = session?.user?.permissions?.includes(feature) || false;
  
  return { hasPermission, loading: false };
}

/**
 * @deprecated Use server-side permission checking instead
 */
export function usePermissions(features: string[], requireAll: boolean = false) {
  const { data: session, status } = useSession();
  
  console.warn('usePermissions is deprecated. Use server-side permission validation instead.');
  
  if (status === 'loading') {
    return { hasAccess: false, loading: true, permissions: {} };
  }
  
  const userPermissions = session?.user?.permissions || [];
  const permissionResults = features.reduce((acc, feature) => {
    acc[feature] = userPermissions.includes(feature);
    return acc;
  }, {} as {[key: string]: boolean});
  
  const hasAccess = requireAll 
    ? features.every(feature => userPermissions.includes(feature))
    : features.some(feature => userPermissions.includes(feature));
  
  return {
    hasAccess,
    loading: false,
    permissions: permissionResults,
    hasPermission: (feature: string) => permissionResults[feature] || false,
  };
}