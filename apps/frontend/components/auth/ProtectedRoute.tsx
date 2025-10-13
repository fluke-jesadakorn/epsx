// ============================================================================
// PROTECTED ROUTE COMPONENT
// Handles authentication flow for protected pages with OpenID + Web3
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Simple authentication check - no permission logic
 * - Backend makes all authorization decisions
 * - Redirect to auth page if not authenticated
 * - Display loading states during authentication checks
 */

'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Loader2 } from 'lucide-react';

interface ProtectedRouteProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  redirectTo?: string;
  requireAuth?: boolean;
}

export function ProtectedRoute({ 
  children, 
  fallback,
  redirectTo = '/auth/signin',
  requireAuth = true 
}: ProtectedRouteProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { isAuthenticated, isLoading, user } = useSharedAuth();
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    const checkAuthentication = async () => {
      if (isLoading) {
        // Still loading authentication state
        return;
      }

      setIsChecking(false);

      if (requireAuth && !isAuthenticated) {
        console.log('User not authenticated, redirecting to auth page', {
          current_path: pathname,
          redirect_to: redirectTo
        });
        
        // Store current path for post-auth redirect
        const returnUrl = encodeURIComponent(pathname);
        router.push(`${redirectTo}?returnUrl=${returnUrl}`);
        return;
      }

      if (isAuthenticated && user) {
        console.log('User authenticated, allowing access', {
          wallet_address: user.wallet_address,
          tier_level: user.tier_level,
          path: pathname
        });
      }
    };

    checkAuthentication();
  }, [isAuthenticated, isLoading, requireAuth, router, pathname, redirectTo, user]);

  // Show loading state while checking authentication
  if (isLoading || isChecking) {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="flex flex-col items-center space-y-4">
          <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
          <p className="text-sm text-gray-600">Checking authentication...</p>
        </div>
      </div>
    );
  }

  // If authentication is required but user is not authenticated, show nothing
  // (router.push will handle the redirect)
  if (requireAuth && !isAuthenticated) {
    return null;
  }

  // Render protected content
  return <>{children}</>;
}

// HOC version for backward compatibility
export function withProtectedRoute<P extends object>(
  Component: React.ComponentType<P>,
  options?: {
    redirectTo?: string;
    requireAuth?: boolean;
  }
) {
  const ProtectedComponent = (props: P) => {
    return (
      <ProtectedRoute 
        redirectTo={options?.redirectTo}
        requireAuth={options?.requireAuth}
      >
        <Component {...props} />
      </ProtectedRoute>
    );
  };

  ProtectedComponent.displayName = `withProtectedRoute(${Component.displayName || Component.name})`;
  
  return ProtectedComponent;
}

// Simple authentication guard hook
export function useAuthGuard(requireAuth: boolean = true) {
  const { isAuthenticated, isLoading, user } = useSharedAuth();
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    if (!isLoading && requireAuth && !isAuthenticated) {
      const returnUrl = encodeURIComponent(pathname);
      router.push(`/auth/signin?returnUrl=${returnUrl}`);
    }
  }, [isAuthenticated, isLoading, requireAuth, router, pathname]);

  return {
    isAuthenticated,
    isLoading,
    user,
    shouldRender: !requireAuth || isAuthenticated
  };
}

export default ProtectedRoute;