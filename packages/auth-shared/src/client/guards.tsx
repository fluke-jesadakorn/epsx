'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';

interface ClientAuthGuardProps {
  children: React.ReactNode;
  requireAuth?: boolean;
  requiredRole?: string;
  fallback?: React.ReactNode;
  redirectTo?: string;
  user?: any;
  loading?: boolean;
}

/**
 * Client-side authentication guard for hybrid scenarios
 * Use this when you need client-side auth checking alongside server guards
 */
export function ClientAuthGuard({ 
  children, 
  requireAuth = false,
  requiredRole,
  fallback,
  redirectTo = '/login',
  user,
  loading
}: ClientAuthGuardProps) {
  const router = useRouter();

  useEffect(() => {
    if (!loading && requireAuth && !user && typeof window !== 'undefined') {
      const currentPath = window.location.pathname + window.location.search;
      const loginUrl = `${redirectTo}?returnUrl=${encodeURIComponent(currentPath)}`;
      router.push(loginUrl);
    }
  }, [user, loading, requireAuth, router, redirectTo]);

  // Show loading while checking auth
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-orange-200 dark:border-orange-800">
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-t-orange-500 dark:border-t-orange-400 absolute top-0 left-0"></div>
        </div>
      </div>
    );
  }

  // If auth is required and user is not authenticated, show fallback or nothing
  if (requireAuth && !user) {
    return fallback || null;
  }

  // Check role if required
  if (requiredRole && user) {
    const ROLE_HIERARCHY: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };

    const userLevel = ROLE_HIERARCHY[user.role?.toLowerCase()] || 0;
    const requiredLevel = ROLE_HIERARCHY[requiredRole.toLowerCase()] || 1;

    if (userLevel < requiredLevel) {
      return fallback || (
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <h1 className="text-2xl font-bold text-gray-900 mb-2">Access Denied</h1>
            <p className="text-gray-600">You don't have permission to access this page.</p>
          </div>
        </div>
      );
    }
  }

  // Render children
  return <>{children}</>;
}

/**
 * Client-side role-based content renderer
 */
interface ClientRoleContentProps {
  children: React.ReactNode;
  requiredRole?: string;
  requiredPermission?: string;
  fallback?: React.ReactNode;
  user?: any;
}

export function ClientRoleContent({ 
  children, 
  requiredRole,
  requiredPermission,
  fallback,
  user
}: ClientRoleContentProps) {
  if (!user) {
    return fallback || null;
  }
  
  // Check permission if required
  if (requiredPermission) {
    const hasPermission = user.permissions?.includes(requiredPermission) ||
      user.permissions?.some((permission: string) => {
        if (permission.endsWith('.*') || permission.endsWith(':*')) {
          const prefix = permission.slice(0, -2);
          return requiredPermission.startsWith(prefix + '.') || requiredPermission.startsWith(prefix + ':');
        }
        if (permission === '*') {
          return true;
        }
        return false;
      });
    
    if (!hasPermission) {
      return fallback || null;
    }
  }
  
  // Check role if required
  if (requiredRole) {
    const ROLE_HIERARCHY: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    };

    const userLevel = ROLE_HIERARCHY[user.role?.toLowerCase()] || 0;
    const requiredLevel = ROLE_HIERARCHY[requiredRole.toLowerCase()] || 1;

    if (userLevel < requiredLevel) {
      return fallback || null;
    }
  }
  
  return <>{children}</>;
}

/**
 * Loading spinner component
 */
export function AuthLoadingSpinner() {
  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="animate-spin rounded-full h-12 w-12 border-4 border-orange-200 dark:border-orange-800">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-t-orange-500 dark:border-t-orange-400 absolute top-0 left-0"></div>
      </div>
    </div>
  );
}